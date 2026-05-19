use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

use runx_cli::launcher::{LauncherAction, plan_launcher_with_rust_harness, shim_help};

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    match plan_launcher_with_rust_harness(
        args,
        env::var_os("RUNX_NPM_PACKAGE"),
        env::var_os("RUNX_JS_BIN"),
        env::var_os("RUNX_RUST_HARNESS"),
    ) {
        LauncherAction::Error(message) => {
            let _ignored = write_stderr_line(&format!("runx: {message}"));
            ExitCode::from(2)
        }
        LauncherAction::PrintHelp => write_stdout(&shim_help()),
        LauncherAction::PrintVersion => {
            write_stdout_line(&format!("runx-cli {}", env!("CARGO_PKG_VERSION")))
        }
        LauncherAction::RunInit(plan) => runx_cli::scaffold::run_native_init(plan),
        LauncherAction::RunNew(plan) => runx_cli::scaffold::run_native_new(plan),
        LauncherAction::RunHistory(plan) => run_native_history(plan.args),
        LauncherAction::RunHarness(plan) => run_native_harness(PathBuf::from(plan.fixture_path)),
        LauncherAction::RunTool(plan) => runx_cli::tool::run_native_tool(plan),
        LauncherAction::Delegate(command) => match run_command(command) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                let _ignored = write_stderr_line(&format!("runx: {error}"));
                ExitCode::from(1)
            }
        },
    }
}

fn run_native_history(args: Vec<OsString>) -> ExitCode {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            let _ignored = write_stderr_line(&format!("runx: failed to resolve cwd: {error}"));
            return ExitCode::from(1);
        }
    };
    match runx_cli::history::run_history_command(&args, &runx_cli::history::env_map(), &cwd) {
        Ok(output) => write_stdout(&output.output),
        Err(runx_cli::history::HistoryCliError::InvalidArgs(message)) => {
            let _ignored = write_stderr_line(&format!("runx: {message}"));
            ExitCode::from(2)
        }
        Err(error) => {
            let _ignored = write_stderr_line(&format!("runx: {error}"));
            ExitCode::from(1)
        }
    }
}

fn run_native_harness(fixture_path: PathBuf) -> ExitCode {
    match runx_runtime::run_harness_fixture(&fixture_path) {
        Ok(output) => match serde_json::to_string_pretty(&output.receipt) {
            Ok(json) => write_stdout_line(&json),
            Err(error) => {
                let _ignored = write_stderr_line(&format!(
                    "runx: failed to serialize harness receipt: {error}"
                ));
                ExitCode::from(1)
            }
        },
        Err(error) => {
            let _ignored = write_stderr_line(&format!(
                "runx: native harness replay failed for {}: {error}",
                fixture_path.display()
            ));
            ExitCode::from(1)
        }
    }
}

fn write_stdout(message: &str) -> ExitCode {
    let mut stdout = io::stdout().lock();
    if stdout.write_all(message.as_bytes()).is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn write_stdout_line(message: &str) -> ExitCode {
    let mut stdout = io::stdout().lock();
    if writeln!(stdout, "{message}").is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn write_stderr_line(message: &str) -> ExitCode {
    let mut stderr = io::stderr().lock();
    if writeln!(stderr, "{message}").is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_command(plan: runx_cli::launcher::CommandPlan) -> Result<u8, String> {
    let status = Command::new(&plan.program)
        .args(plan.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| {
            format!(
                "failed to launch {}: {error}",
                plan.program.to_string_lossy()
            )
        })?;

    Ok(exit_code(status))
}

fn exit_code(status: std::process::ExitStatus) -> u8 {
    if let Some(code) = status.code() {
        return code.clamp(0, 255) as u8;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return (128 + signal).clamp(1, 255) as u8;
        }
    }

    1
}
