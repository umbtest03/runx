use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use runx_contracts::JsonValue;
use runx_runtime::SkillRunRequest;
use runx_runtime::orchestrator::LocalCredentialDescriptor;

mod inputs;
mod output;
mod parser;
mod resolver;

use output::{skill_result_exit_code, write_skill_output};
pub use parser::parse_skill_plan;
use resolver::resolve_skill_ref;

#[derive(Debug, PartialEq)]
pub struct SkillPlan {
    pub skill_path: PathBuf,
    pub runner: Option<String>,
    pub receipt_dir: Option<PathBuf>,
    pub run_id: Option<String>,
    pub answers: Option<PathBuf>,
    pub registry: Option<String>,
    pub expected_digest: Option<String>,
    pub json: bool,
    pub inputs: BTreeMap<String, JsonValue>,
    /// One-shot, per-run local credential descriptor supplied via
    /// `--credential` and `--secret-env`. The secret is read from the named
    /// process environment variable so raw secret material never appears on
    /// argv. Runner-specific execution validates whether that delivery channel
    /// is supported before any child process starts.
    pub local_credential: Option<LocalCredentialDescriptor>,
}

pub fn run_native_skill(plan: SkillPlan) -> ExitCode {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let env = env::vars().collect();
    let skill_path = match resolve_skill_ref(
        &plan.skill_path,
        &cwd,
        resolver::SkillResolverOptions {
            env: &env,
            registry: plan.registry.as_deref(),
            expected_digest: plan.expected_digest.as_deref(),
        },
    ) {
        Ok(skill_path) => skill_path,
        Err(error) => {
            let _ignored = writeln!(io::stderr(), "runx: {error}");
            return ExitCode::from(1);
        }
    };
    let request = SkillRunRequest {
        skill_path,
        receipt_dir: plan.receipt_dir,
        run_id: plan.run_id,
        answers_path: plan.answers,
        inputs: plan.inputs,
        env,
        cwd,
        local_credential: plan.local_credential,
    };
    let orchestrator = crate::runtime::local_orchestrator();
    let result = match plan.runner.as_deref() {
        Some(runner) => orchestrator.run_skill_with_runner(&request, runner),
        None => orchestrator.run_skill(&request),
    };
    match result {
        Ok(result) => {
            let exit_code = skill_result_exit_code(&result.output);
            write_skill_output(&result.output, plan.json, exit_code)
        }
        Err(error) => {
            let _ignored = writeln!(io::stderr(), "runx: {error}");
            ExitCode::from(1)
        }
    }
}
