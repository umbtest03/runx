use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use runx_contracts::JsonValue;
use runx_runtime::{SkillRunRequest, execute_skill_run};

#[derive(Debug, PartialEq)]
pub struct SkillPlan {
    pub skill_path: PathBuf,
    pub receipt_dir: Option<PathBuf>,
    pub run_id: Option<String>,
    pub answers: Option<PathBuf>,
    pub json: bool,
    pub inputs: BTreeMap<String, JsonValue>,
}

pub fn parse_skill_plan(args: &[OsString]) -> Result<SkillPlan, String> {
    let mut skill_path = None;
    let mut receipt_dir = None;
    let mut run_id = None;
    let mut answers = None;
    let mut json = false;
    let mut inputs = BTreeMap::new();
    let mut index = 1;

    while index < args.len() {
        let token = string_arg(args, index)?;
        match token.as_str() {
            value if value.starts_with("--receipt-dir=") => {
                receipt_dir = Some(PathBuf::from(value.trim_start_matches("--receipt-dir=")));
            }
            "--receipt-dir" => {
                index += 1;
                receipt_dir = Some(PathBuf::from(string_arg(args, index)?));
            }
            "--receiptDir" => {
                return Err(
                    "runx skill uses --receipt-dir; --receiptDir is not supported".to_owned(),
                );
            }
            value if value.starts_with("--receiptDir=") => {
                return Err(
                    "runx skill uses --receipt-dir; --receiptDir is not supported".to_owned(),
                );
            }
            value if value.starts_with("--run-id=") => {
                run_id = Some(value.trim_start_matches("--run-id=").to_owned());
            }
            "--run-id" => {
                index += 1;
                run_id = Some(string_arg(args, index)?);
            }
            "--receipt" => {
                return Err("runx skill uses --run-id; --receipt is not supported".to_owned());
            }
            value if value.starts_with("--receipt=") => {
                return Err("runx skill uses --run-id; --receipt is not supported".to_owned());
            }
            value if value.starts_with("--answers=") => {
                answers = Some(PathBuf::from(value.trim_start_matches("--answers=")));
            }
            "--answers" => {
                index += 1;
                answers = Some(PathBuf::from(string_arg(args, index)?));
            }
            "--json" => json = true,
            "--non-interactive" => {}
            value if value.starts_with("--") && value.contains('=') => {
                let (key, value) = value.split_once('=').ok_or_else(|| {
                    "runx skill argument must use --name value or --name=value".to_owned()
                })?;
                inputs.insert(
                    key.trim_start_matches("--").replace('-', "_"),
                    parse_cli_value(value),
                );
            }
            value if value.starts_with("--") => {
                let key = value.trim_start_matches("--").replace('-', "_");
                index += 1;
                inputs.insert(key, parse_cli_value(&string_arg(args, index)?));
            }
            value => {
                if skill_path.is_some() {
                    return Err(format!("unexpected runx skill argument {value}"));
                }
                if is_reserved_skill_action(value) {
                    return Err(format!(
                        "runx skill runs a skill package path directly; runx skill {value} is not supported"
                    ));
                }
                skill_path = Some(PathBuf::from(value));
            }
        }
        index += 1;
    }

    let Some(skill_path) = skill_path else {
        return Err("runx skill requires a skill package path".to_owned());
    };
    if answers.is_some() && run_id.is_none() {
        return Err("runx skill --answers requires --run-id".to_owned());
    }
    if run_id.is_some() && answers.is_none() {
        return Err("runx skill --run-id requires --answers".to_owned());
    }

    Ok(SkillPlan {
        skill_path,
        receipt_dir,
        run_id,
        answers,
        json,
        inputs,
    })
}

pub fn run_native_skill(plan: SkillPlan) -> ExitCode {
    let request = SkillRunRequest {
        skill_path: plan.skill_path,
        receipt_dir: plan.receipt_dir,
        run_id: plan.run_id,
        answers_path: plan.answers,
        inputs: plan.inputs,
        env: env::vars().collect(),
        cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };
    match execute_skill_run(&request) {
        Ok(output) => {
            let exit_code = skill_result_exit_code(&output);
            write_json_with_exit(&output, exit_code)
        }
        Err(error) => {
            let _ignored = writeln!(io::stderr(), "runx: {error}");
            ExitCode::from(1)
        }
    }
}

fn string_arg(args: &[OsString], index: usize) -> Result<String, String> {
    let value = args
        .get(index)
        .ok_or_else(|| "missing value for runx skill argument".to_owned())?;
    value
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| "runx skill arguments must be UTF-8".to_owned())
}

fn parse_cli_value(raw: &str) -> JsonValue {
    serde_json::from_str(raw).unwrap_or_else(|_| JsonValue::String(raw.to_owned()))
}

fn is_reserved_skill_action(value: &str) -> bool {
    matches!(value, "add" | "inspect" | "publish" | "run" | "search")
}

fn write_json_with_exit(value: &JsonValue, exit_code: ExitCode) -> ExitCode {
    match serde_json::to_string_pretty(value) {
        Ok(json) => {
            let mut stdout = io::stdout().lock();
            let result = stdout
                .write_all(json.as_bytes())
                .and_then(|_| stdout.write_all(b"\n"));
            match result {
                Ok(()) => exit_code,
                Err(_) => ExitCode::from(1),
            }
        }
        Err(error) => {
            let _ignored = writeln!(
                io::stderr(),
                "runx: failed to serialize skill result: {error}"
            );
            ExitCode::from(1)
        }
    }
}

fn skill_result_exit_code(value: &JsonValue) -> ExitCode {
    match value {
        JsonValue::Object(object) => match object.get("status") {
            Some(JsonValue::String(status)) if status == "needs_agent" => ExitCode::from(2),
            _ => ExitCode::SUCCESS,
        },
        _ => ExitCode::SUCCESS,
    }
}
