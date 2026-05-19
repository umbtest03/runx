use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use runx_contracts::{DoctorDiagnosticSeverity, DoctorReport, DoctorStatus};
use runx_runtime::{RuntimeError, default_doctor_options, run_doctor};

use crate::launcher::DoctorPlan;

pub fn run_native_doctor(plan: DoctorPlan) -> ExitCode {
    let env = crate::history::env_map();
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            let _ignored = write_stderr(&format!("runx: failed to resolve cwd: {error}\n"));
            return ExitCode::from(1);
        }
    };

    match run_doctor_command(&plan, &env, &cwd) {
        Ok(output) => write_stdout(&output.stdout, output.exit_code),
        Err(error) => {
            let _ignored = write_stderr(&format!("runx: {error}\n"));
            ExitCode::from(1)
        }
    }
}

struct DoctorCliOutput {
    stdout: String,
    exit_code: u8,
}

fn run_doctor_command(
    plan: &DoctorPlan,
    env: &BTreeMap<String, String>,
    cwd: &Path,
) -> Result<DoctorCliOutput, DoctorCliError> {
    let root = resolve_doctor_root(plan, env, cwd);
    let report = run_doctor(&root, &default_doctor_options())?;
    let exit_code = match report.status {
        DoctorStatus::Success => 0,
        DoctorStatus::Failure => 1,
    };
    let stdout = if plan.json {
        json_line(&report)?
    } else {
        render_doctor_report(&report)
    };
    Ok(DoctorCliOutput { stdout, exit_code })
}

fn resolve_doctor_root(plan: &DoctorPlan, env: &BTreeMap<String, String>, cwd: &Path) -> PathBuf {
    match plan.path.as_deref() {
        Some(path) => {
            runx_runtime::resolve_path_from_user_input(&path.to_string_lossy(), env, cwd, true)
        }
        None => workspace_base(env, cwd),
    }
}

fn workspace_base(env: &BTreeMap<String, String>, cwd: &Path) -> PathBuf {
    env.get("RUNX_CWD")
        .map(PathBuf::from)
        .or_else(|| find_runx_workspace_root(cwd))
        .or_else(|| env.get("INIT_CWD").map(PathBuf::from))
        .unwrap_or_else(|| cwd.to_path_buf())
}

fn find_runx_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("pnpm-workspace.yaml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn json_line<T: serde::Serialize>(value: &T) -> Result<String, DoctorCliError> {
    serde_json::to_string_pretty(value)
        .map(|json| format!("{json}\n"))
        .map_err(DoctorCliError::Serialize)
}

fn render_doctor_report(report: &DoctorReport) -> String {
    let mut lines = vec![
        String::new(),
        format!(
            "  {}  doctor  {} error(s), {} warning(s)",
            status_icon(&report.status),
            report.summary.errors,
            report.summary.warnings
        ),
    ];
    for diagnostic in &report.diagnostics {
        lines.push(format!(
            "  {}  {}  {}",
            diagnostic_icon(&diagnostic.severity),
            diagnostic.id,
            diagnostic.location.path
        ));
        lines.push(format!("     {}", diagnostic.message));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn status_icon(status: &DoctorStatus) -> &'static str {
    match status {
        DoctorStatus::Success => "✓",
        DoctorStatus::Failure => "✗",
    }
}

fn diagnostic_icon(severity: &DoctorDiagnosticSeverity) -> &'static str {
    match severity {
        DoctorDiagnosticSeverity::Error => "✗",
        DoctorDiagnosticSeverity::Warning | DoctorDiagnosticSeverity::Info => "·",
    }
}

fn write_stdout(message: &str, exit_code: u8) -> ExitCode {
    let mut stdout = io::stdout().lock();
    if stdout.write_all(message.as_bytes()).is_ok() {
        ExitCode::from(exit_code)
    } else {
        ExitCode::from(1)
    }
}

fn write_stderr(message: &str) -> ExitCode {
    let mut stderr = io::stderr().lock();
    if stderr.write_all(message.as_bytes()).is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

#[derive(Debug)]
enum DoctorCliError {
    Runtime(RuntimeError),
    Serialize(serde_json::Error),
}

impl std::fmt::Display for DoctorCliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::Serialize(error) => {
                write!(formatter, "failed to serialize doctor report: {error}")
            }
        }
    }
}

impl From<RuntimeError> for DoctorCliError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}
