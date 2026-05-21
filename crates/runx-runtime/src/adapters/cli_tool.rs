use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use runx_contracts::JsonValue;

use crate::RuntimeError;
use crate::adapter::{InvocationStatus, SkillAdapter, SkillInvocation, SkillOutput};
use crate::sandbox::prepare_process_sandbox;

const OUTPUT_LIMIT_BYTES: usize = 1024 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Default)]
pub struct CliToolAdapter;

impl SkillAdapter for CliToolAdapter {
    fn adapter_type(&self) -> &'static str {
        "cli-tool"
    }

    fn invoke(&self, request: SkillInvocation) -> Result<SkillOutput, RuntimeError> {
        let started = Instant::now();
        let credential_delivery = request.credential_delivery.clone();
        let sandbox = prepare_process_sandbox(
            &request.source,
            &request.skill_directory,
            &request.inputs,
            &request.env,
        )?;
        let mut child = Command::new(&sandbox.command)
            .args(&sandbox.args)
            .current_dir(&sandbox.cwd)
            .env_clear()
            .envs(&sandbox.env)
            .envs(credential_delivery.secret_env().iter())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| RuntimeError::io("spawning cli-tool process", source))?;
        write_stdin(&mut child, &request)?;
        let timed_out = wait_for_exit(&mut child, request.source.timeout_seconds)?;
        let output = child
            .wait_with_output()
            .map_err(|source| RuntimeError::io("collecting cli-tool output", source))?;
        let stdout = credential_delivery.redact_text(truncate_utf8(output.stdout));
        let stderr = credential_delivery.redact_text(truncate_utf8(output.stderr));
        let success = output.status.success() && !timed_out;
        Ok(SkillOutput {
            status: if success {
                InvocationStatus::Success
            } else {
                InvocationStatus::Failure
            },
            stdout,
            stderr,
            exit_code: output.status.code(),
            duration_ms: duration_ms(started),
            metadata: sandbox.metadata,
        })
    }
}

fn write_stdin(
    child: &mut std::process::Child,
    request: &SkillInvocation,
) -> Result<(), RuntimeError> {
    let Some(mut stdin) = child.stdin.take() else {
        return Ok(());
    };
    if request.source.input_mode.as_deref() == Some("stdin") {
        let bytes = serde_json::to_vec(&request.inputs)
            .map_err(|source| RuntimeError::json("serializing stdin inputs", source))?;
        stdin
            .write_all(&bytes)
            .map_err(|source| RuntimeError::io("writing cli-tool stdin", source))?;
    }
    Ok(())
}

fn wait_for_exit(
    child: &mut std::process::Child,
    timeout_seconds: Option<u64>,
) -> Result<bool, RuntimeError> {
    let timeout = timeout_seconds.map(Duration::from_secs);
    let started = Instant::now();
    loop {
        if child
            .try_wait()
            .map_err(|source| RuntimeError::io("polling cli-tool process", source))?
            .is_some()
        {
            return Ok(false);
        }
        if timeout.is_some_and(|timeout| started.elapsed() >= timeout) {
            child
                .kill()
                .map_err(|source| RuntimeError::io("killing timed out cli-tool process", source))?;
            return Ok(true);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn truncate_utf8(bytes: Vec<u8>) -> String {
    let limit = bytes.len().min(OUTPUT_LIMIT_BYTES);
    String::from_utf8_lossy(&bytes[..limit]).into_owned()
}

fn duration_ms(started: Instant) -> u64 {
    let millis = started.elapsed().as_millis();
    u64::try_from(millis).unwrap_or(u64::MAX)
}

pub fn output_object(output: &SkillOutput) -> runx_contracts::JsonObject {
    let mut object = runx_contracts::JsonObject::new();
    if let Ok(JsonValue::Object(parsed)) = serde_json::from_str::<JsonValue>(&output.stdout) {
        object.extend(parsed);
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
