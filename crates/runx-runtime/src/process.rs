mod capture;
mod signal;

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use thiserror::Error;
use wait_timeout::ChildExt;

pub(crate) use self::capture::CapturedOutput;
use self::capture::{CaptureHandle, capture_pipe, join_capture};
use self::signal::{ProcessSignal, configure_process_group, signal_timed_out_process};

const DEFAULT_FORCE_KILL_GRACE: Duration = Duration::from_millis(100);

#[derive(Clone, Debug)]
pub(crate) struct ProcessSpec {
    label: &'static str,
    command: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    env: BTreeMap<String, String>,
    stdin: Option<ProcessStdin>,
    timeout: Option<Duration>,
    output_limit_bytes: usize,
    cleanup_paths: Vec<PathBuf>,
}

impl ProcessSpec {
    pub(crate) fn new(
        label: &'static str,
        command: impl Into<String>,
        output_limit_bytes: usize,
    ) -> Self {
        Self {
            label,
            command: command.into(),
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
            stdin: None,
            timeout: None,
            output_limit_bytes,
            cleanup_paths: Vec::new(),
        }
    }

    pub(crate) fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub(crate) fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub(crate) fn env(mut self, env: BTreeMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub(crate) fn stdin(mut self, stdin: Option<ProcessStdin>) -> Self {
        self.stdin = stdin;
        self
    }

    pub(crate) fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    #[cfg(feature = "cli-tool")]
    pub(crate) fn cleanup_paths(mut self, cleanup_paths: Vec<PathBuf>) -> Self {
        self.cleanup_paths = cleanup_paths;
        self
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessStdin {
    bytes: Vec<u8>,
    write_context: &'static str,
}

impl ProcessStdin {
    pub(crate) fn new(bytes: Vec<u8>, write_context: &'static str) -> Self {
        Self {
            bytes,
            write_context,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ProcessOutcome {
    pub(crate) status: ExitStatus,
    pub(crate) timed_out: bool,
    pub(crate) stdout: CapturedOutput,
    pub(crate) stderr: CapturedOutput,
    pub(crate) duration_ms: u64,
    pub(crate) cleanup_errors: Vec<String>,
}

#[derive(Debug, Error)]
pub(crate) enum ProcessSupervisorError {
    #[error("process I/O failed while {context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
}

impl ProcessSupervisorError {
    pub(crate) fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }
}

pub(crate) fn run_process(spec: ProcessSpec) -> Result<ProcessOutcome, ProcessSupervisorError> {
    let started = Instant::now();
    let mut child = match spawn_process(&spec) {
        Ok(child) => child,
        Err(error) => {
            cleanup_paths_quietly(&spec.cleanup_paths);
            return Err(error);
        }
    };
    let stdout = match capture_pipe(
        child.stdout.take(),
        open_pipe_context(spec.label, "stdout"),
        spec.output_limit_bytes,
    ) {
        Ok(stdout) => stdout,
        Err(error) => {
            cleanup_child_after_startup_error(&mut child, &spec, None, None);
            return Err(error);
        }
    };
    let stderr = match capture_pipe(
        child.stderr.take(),
        open_pipe_context(spec.label, "stderr"),
        spec.output_limit_bytes,
    ) {
        Ok(stderr) => stderr,
        Err(error) => {
            cleanup_child_after_startup_error(&mut child, &spec, Some(stdout), None);
            return Err(error);
        }
    };

    if let Err(error) = write_stdin(&mut child, spec.stdin.as_ref()) {
        cleanup_child_after_startup_error(&mut child, &spec, Some(stdout), Some(stderr));
        return Err(error);
    }

    let (status, timed_out) = wait_for_exit(&mut child, &spec)?;
    let stdout = join_capture(stdout, collect_context(spec.label, "stdout"))?;
    let stderr = join_capture(stderr, collect_context(spec.label, "stderr"))?;
    let cleanup_errors = cleanup_paths(&spec.cleanup_paths);
    Ok(ProcessOutcome {
        status,
        timed_out,
        stdout,
        stderr,
        duration_ms: duration_ms(started),
        cleanup_errors,
    })
}

fn spawn_process(spec: &ProcessSpec) -> Result<Child, ProcessSupervisorError> {
    let mut command = Command::new(&spec.command);
    command
        .args(&spec.args)
        .env_clear()
        .envs(&spec.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = spec.cwd.as_ref() {
        command.current_dir(cwd);
    }
    configure_process_group(&mut command);
    command
        .spawn()
        .map_err(|source| ProcessSupervisorError::io(spawn_context(spec.label), source))
}

fn write_stdin(
    child: &mut Child,
    stdin: Option<&ProcessStdin>,
) -> Result<(), ProcessSupervisorError> {
    let Some(stdin) = stdin else {
        return Ok(());
    };
    let Some(mut pipe) = child.stdin.take() else {
        return Ok(());
    };
    pipe.write_all(&stdin.bytes)
        .map_err(|source| ProcessSupervisorError::io(stdin.write_context, source))
}

fn wait_for_exit(
    child: &mut Child,
    spec: &ProcessSpec,
) -> Result<(ExitStatus, bool), ProcessSupervisorError> {
    let Some(timeout) = spec.timeout else {
        let status = child
            .wait()
            .map_err(|source| ProcessSupervisorError::io(wait_context(spec.label), source))?;
        return Ok((status, false));
    };

    match child
        .wait_timeout(timeout)
        .map_err(|source| ProcessSupervisorError::io(wait_timeout_context(spec.label), source))?
    {
        Some(status) => Ok((status, false)),
        None => {
            signal_timed_out_process(child, ProcessSignal::Terminate, spec)?;
            thread::sleep(DEFAULT_FORCE_KILL_GRACE);
            signal_timed_out_process(child, ProcessSignal::Force, spec)?;
            let status = child.wait().map_err(|source| {
                ProcessSupervisorError::io(wait_timed_out_context(spec.label), source)
            })?;
            Ok((status, true))
        }
    }
}

fn cleanup_child_after_startup_error(
    child: &mut Child,
    spec: &ProcessSpec,
    stdout: Option<CaptureHandle>,
    stderr: Option<CaptureHandle>,
) {
    let _ = signal_timed_out_process(child, ProcessSignal::Force, spec);
    let _ = child.wait();
    if let Some(stdout) = stdout {
        let _ = join_capture(stdout, collect_context(spec.label, "stdout"));
    }
    if let Some(stderr) = stderr {
        let _ = join_capture(stderr, collect_context(spec.label, "stderr"));
    }
    cleanup_paths_quietly(&spec.cleanup_paths);
}

fn cleanup_paths(paths: &[PathBuf]) -> Vec<String> {
    let mut errors = Vec::new();
    for path in paths {
        if let Err(error) = fs::remove_dir_all(path) {
            errors.push(format!("{}: {error}", path.display()));
        }
    }
    errors
}

fn cleanup_paths_quietly(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_dir_all(path);
    }
}

fn duration_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn spawn_context(label: &str) -> String {
    format!("spawning {label} process")
}

fn open_pipe_context(label: &str, stream: &str) -> String {
    format!("opening {label} {stream} pipe")
}

fn collect_context(label: &str, stream: &str) -> String {
    format!("collecting {label} {stream}")
}

fn wait_context(label: &str) -> String {
    format!("waiting for {label} process")
}

fn wait_timeout_context(label: &str) -> String {
    format!("waiting for {label} process with timeout")
}

fn wait_timed_out_context(label: &str) -> String {
    format!("waiting for timed out {label} process")
}

fn poll_timed_out_context(label: &str) -> String {
    format!("polling timed out {label} process")
}

fn kill_timed_out_context(label: &str) -> String {
    format!("killing timed out {label} process")
}
