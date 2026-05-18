use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::{Command, ExitCode, Stdio};

use runx_cli::launcher::{LauncherAction, plan_launcher, shim_help};

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    match plan_launcher(
        args,
        env::var_os("RUNX_NPM_PACKAGE"),
        env::var_os("RUNX_JS_BIN"),
    ) {
        LauncherAction::PrintHelp => write_stdout(&shim_help()),
        LauncherAction::PrintVersion => {
            write_stdout_line(&format!("runx-cli {}", env!("CARGO_PKG_VERSION")))
        }
        LauncherAction::Delegate(command) => match run_command(command) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                let _ignored = write_stderr_line(&format!("runx: {error}"));
                ExitCode::from(1)
            }
        },
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
