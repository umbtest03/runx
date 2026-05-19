use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use runx_contracts::{JsonObject, JsonValue};
use runx_parser::{SkillSandbox, SkillSource};

use crate::RuntimeError;

#[derive(Clone, Debug, PartialEq)]
pub struct SandboxPlan {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub metadata: JsonObject,
}

pub fn prepare_process_sandbox(
    source: &SkillSource,
    skill_directory: &Path,
    inputs: &JsonObject,
    base_env: &BTreeMap<String, String>,
) -> Result<SandboxPlan, RuntimeError> {
    let command = source.command.clone().ok_or(RuntimeError::MissingCommand)?;
    let sandbox = source.sandbox.as_ref();
    validate_sandbox(sandbox)?;
    let cwd = resolve_cwd(source, sandbox, skill_directory)?;
    let env = child_env(sandbox, base_env, inputs)?;
    let args = source
        .args
        .iter()
        .map(|arg| resolve_template(arg, inputs))
        .collect();
    Ok(SandboxPlan {
        command,
        args,
        cwd,
        env,
        metadata: sandbox_metadata(sandbox),
    })
}

fn resolve_cwd(
    source: &SkillSource,
    sandbox: Option<&SkillSandbox>,
    skill_directory: &Path,
) -> Result<PathBuf, RuntimeError> {
    let policy = sandbox
        .and_then(|sandbox| sandbox.cwd_policy.as_deref())
        .unwrap_or("skill-directory");
    match (policy, source.cwd.as_deref()) {
        ("custom", Some(cwd)) => Ok(resolve_path(skill_directory, cwd)),
        ("workspace", Some(cwd)) => Ok(resolve_path(skill_directory, cwd)),
        ("workspace", None) => std::env::current_dir()
            .map_err(|source| RuntimeError::io("resolving workspace cwd", source)),
        (_, Some(cwd)) => Ok(resolve_path(skill_directory, cwd)),
        _ => Ok(skill_directory.to_path_buf()),
    }
}

fn resolve_path(base: &Path, path: &str) -> PathBuf {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        candidate
    } else {
        base.join(candidate)
    }
}

fn child_env(
    sandbox: Option<&SkillSandbox>,
    base_env: &BTreeMap<String, String>,
    inputs: &JsonObject,
) -> Result<BTreeMap<String, String>, RuntimeError> {
    let mut env = allowed_base_env(sandbox, base_env);
    let serialized = serde_json::to_string(inputs)
        .map_err(|source| RuntimeError::json("serializing runtime inputs", source))?;
    env.insert("RUNX_INPUTS_JSON".to_owned(), serialized);
    for (key, value) in inputs {
        env.insert(input_env_name(key), json_value_env(value)?);
    }
    Ok(env)
}

fn allowed_base_env(
    sandbox: Option<&SkillSandbox>,
    base_env: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut allowed = ["PATH", "SystemRoot", "PATHEXT"]
        .into_iter()
        .filter_map(|key| {
            base_env
                .get(key)
                .cloned()
                .map(|value| (key.to_owned(), value))
        })
        .collect::<BTreeMap<_, _>>();
    if let Some(env_allowlist) = sandbox.and_then(|sandbox| sandbox.env_allowlist.as_ref()) {
        for key in env_allowlist {
            if let Some(value) = base_env.get(key) {
                allowed.insert(key.clone(), value.clone());
            }
        }
    }
    allowed
}

fn validate_sandbox(sandbox: Option<&SkillSandbox>) -> Result<(), RuntimeError> {
    let Some(sandbox) = sandbox else {
        return Ok(());
    };
    if sandbox.require_enforcement == Some(true) {
        return Err(sandbox_violation(
            "platform isolation helpers are not available in the runtime skeleton",
        ));
    }
    match sandbox.profile.as_str() {
        "readonly" => validate_readonly_sandbox(sandbox),
        "workspace-write" | "network" => Ok(()),
        "unrestricted-local-dev" => validate_unrestricted_sandbox(sandbox),
        profile => Err(sandbox_violation(format!(
            "unsupported sandbox profile '{profile}'"
        ))),
    }
}

fn validate_readonly_sandbox(sandbox: &SkillSandbox) -> Result<(), RuntimeError> {
    if sandbox.network == Some(true) {
        return Err(sandbox_violation("readonly sandbox cannot request network"));
    }
    if !sandbox.writable_paths.is_empty() {
        return Err(sandbox_violation(
            "readonly sandbox cannot declare writable paths",
        ));
    }
    Ok(())
}

fn validate_unrestricted_sandbox(sandbox: &SkillSandbox) -> Result<(), RuntimeError> {
    if sandbox.approved_escalation == Some(true) {
        Ok(())
    } else {
        Err(sandbox_violation(
            "unrestricted-local-dev requires approved escalation",
        ))
    }
}

fn sandbox_violation(message: impl Into<String>) -> RuntimeError {
    RuntimeError::SandboxViolation {
        message: message.into(),
    }
}

fn input_env_name(key: &str) -> String {
    let suffix = key
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("RUNX_INPUT_{suffix}")
}

fn json_value_env(value: &JsonValue) -> Result<String, RuntimeError> {
    match value {
        JsonValue::Null => Ok(String::new()),
        JsonValue::Bool(value) => Ok(value.to_string()),
        JsonValue::Number(value) => serde_json::to_string(value)
            .map_err(|source| RuntimeError::json("serializing input number", source)),
        JsonValue::String(value) => Ok(value.clone()),
        JsonValue::Array(_) | JsonValue::Object(_) => serde_json::to_string(value)
            .map_err(|source| RuntimeError::json("serializing structured input", source)),
    }
}

fn resolve_template(template: &str, inputs: &JsonObject) -> String {
    let mut resolved = template.to_owned();
    for (key, value) in inputs {
        if let Ok(value) = json_value_env(value) {
            resolved = resolved.replace(&format!("{{{{{key}}}}}"), &value);
            resolved = resolved.replace(&format!("{{{{ {key} }}}}"), &value);
        }
    }
    resolved
}

fn sandbox_metadata(sandbox: Option<&SkillSandbox>) -> JsonObject {
    let mut metadata = JsonObject::new();
    if let Some(sandbox) = sandbox {
        metadata.insert(
            "profile".to_owned(),
            JsonValue::String(sandbox.profile.clone()),
        );
    }
    metadata
}
