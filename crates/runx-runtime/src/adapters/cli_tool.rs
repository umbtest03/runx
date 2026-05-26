use std::time::Duration;

use runx_contracts::JsonValue;

use crate::RuntimeError;
use crate::adapter::{
    FanoutExecutionMode, InvocationStatus, SkillAdapter, SkillInvocation, SkillOutput,
};
use crate::adapter_pipeline::AdapterInvocationPlan;
use crate::credentials::CredentialDelivery;
use crate::process::{CapturedOutput, ProcessOutcome, ProcessSpec, ProcessStdin, run_process};
use crate::services::SandboxServices;

const OUTPUT_LIMIT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct CliToolAdapter;

impl SkillAdapter for CliToolAdapter {
    fn adapter_type(&self) -> &'static str {
        "cli-tool"
    }

    fn invoke(&self, request: SkillInvocation) -> Result<SkillOutput, RuntimeError> {
        let plan = AdapterInvocationPlan::from_invocation(self.adapter_type(), &request);
        let credential_delivery = request.credential_delivery.clone();
        credential_delivery.reject_process_env_boundary(plan.adapter_type())?;
        let sandbox = SandboxServices.process_plan(
            &request.source,
            &request.skill_directory,
            &request.inputs,
            &request.env,
        )?;
        let stdin = cli_tool_stdin(&request)?;
        let outcome = run_process(
            ProcessSpec::new("cli-tool", sandbox.command.clone(), OUTPUT_LIMIT_BYTES)
                .args(sandbox.args.clone())
                .cwd(sandbox.cwd.clone())
                .env(sandbox.env.clone())
                .stdin(stdin)
                .timeout(request.source.timeout_seconds.map(Duration::from_secs))
                .cleanup_paths(sandbox.cleanup_paths.clone()),
        )
        .map_err(|error| match error {
            crate::process::ProcessSupervisorError::Io { context, source } => {
                RuntimeError::io(context, source)
            }
        })?;
        let cleanup_errors = outcome.cleanup_errors.clone();
        let mut output = cli_tool_output(outcome, &credential_delivery, sandbox.metadata.clone());
        if !cleanup_errors.is_empty() {
            output.metadata.insert(
                "cleanup_errors".to_owned(),
                JsonValue::Array(cleanup_errors.into_iter().map(JsonValue::String).collect()),
            );
        }
        Ok(output)
    }

    fn fanout_execution_mode(&self, source: &runx_parser::SkillSource) -> FanoutExecutionMode {
        if source.source_type == runx_parser::SourceKind::CliTool {
            FanoutExecutionMode::IsolatedParallel
        } else {
            FanoutExecutionMode::Serial
        }
    }

    fn clone_for_fanout(&self) -> Option<Box<dyn SkillAdapter + Send + Sync>> {
        Some(Box::new(*self))
    }
}

fn cli_tool_stdin(request: &SkillInvocation) -> Result<Option<ProcessStdin>, RuntimeError> {
    if request.source.input_mode != Some(runx_parser::InputMode::Stdin) {
        return Ok(None);
    }
    let bytes = serde_json::to_vec(&request.inputs)
        .map_err(|source| RuntimeError::json("serializing stdin inputs", source))?;
    Ok(Some(ProcessStdin::new(bytes, "writing cli-tool stdin")))
}

fn redacted_capture(
    output: CapturedOutput,
    credential_delivery: &CredentialDelivery,
) -> CapturedText {
    if output.truncated {
        return CapturedText {
            text: String::new(),
            truncated: true,
        };
    }
    CapturedText {
        text: credential_delivery.redact_bytes_to_string(output.bytes, OUTPUT_LIMIT_BYTES),
        truncated: false,
    }
}

fn cli_tool_output(
    outcome: ProcessOutcome,
    credential_delivery: &CredentialDelivery,
    metadata: runx_contracts::JsonObject,
) -> SkillOutput {
    let stdout = redacted_capture(outcome.stdout, credential_delivery);
    let stderr = redacted_capture(outcome.stderr, credential_delivery);
    let output_truncated = stdout.truncated || stderr.truncated;
    let success = outcome.status.success() && !outcome.timed_out && !output_truncated;
    let (stdout, stderr) = if output_truncated {
        (
            String::new(),
            format!(
                "runx cli-tool output exceeded {OUTPUT_LIMIT_BYTES} byte capture limit; stdout/stderr omitted"
            ),
        )
    } else {
        (stdout.text, stderr.text)
    };
    SkillOutput {
        status: if success {
            InvocationStatus::Success
        } else {
            InvocationStatus::Failure
        },
        stdout,
        stderr,
        exit_code: outcome.status.code(),
        duration_ms: outcome.duration_ms,
        metadata,
    }
}

struct CapturedText {
    text: String,
    truncated: bool,
}

pub fn output_object(output: &SkillOutput) -> runx_contracts::JsonObject {
    let mut object = runx_contracts::JsonObject::new();
    if let Ok(parsed) = serde_json::from_str::<JsonValue>(&output.stdout) {
        object.insert("skill_claim".to_owned(), parsed);
    }
    object.insert(
        "stdout".to_owned(),
        JsonValue::String(output.stdout.clone()),
    );
    object.insert(
        "stderr".to_owned(),
        JsonValue::String(output.stderr.clone()),
    );
    object.insert(
        "status".to_owned(),
        JsonValue::String(if output.succeeded() {
            "success".to_owned()
        } else {
            "failure".to_owned()
        }),
    );
    object
}
