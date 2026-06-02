//! Recursive tool executor for the managed-agent loop.
//!
//! When the model chooses a tool, the agent invokes it through the governed
//! runtime. This reuses the catalog adapter's single resolve-and-invoke path
//! (`resolve_and_invoke_local_tool`) so the agent's tool calls go through the same
//! resolution, sandbox, credential delivery, and receipt machinery as any other
//! local tool. There is no parallel execution route.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use runx_contracts::JsonValue;

use super::agent_loop::ToolExecutor;
use super::catalog::{LocalToolRequest, resolve_and_invoke_local_tool};
use crate::RuntimeError;
use crate::adapter::SkillOutput;
use crate::credentials::CredentialDelivery;

const MANAGED_AGENT_SKILL: &str = "managed-agent";

/// Executes the agent's chosen tools through the governed runtime, carrying the
/// run context (env, skill directory, credential delivery) the resolver captured
/// from the agent invocation.
pub struct RuntimeToolExecutor {
    env: BTreeMap<String, String>,
    skill_directory: PathBuf,
    credential_delivery: CredentialDelivery,
}

impl RuntimeToolExecutor {
    #[must_use]
    pub fn new(
        env: BTreeMap<String, String>,
        skill_directory: PathBuf,
        credential_delivery: CredentialDelivery,
    ) -> Self {
        Self {
            env,
            skill_directory,
            credential_delivery,
        }
    }
}

impl ToolExecutor for RuntimeToolExecutor {
    fn execute(&self, tool: &str, input: &JsonValue) -> Result<SkillOutput, RuntimeError> {
        // The model supplies the tool arguments already resolved, so pass them as
        // both inputs and resolved_inputs.
        let inputs = input.as_object().cloned().unwrap_or_default();
        let request = LocalToolRequest {
            tool_ref: tool,
            inputs: &inputs,
            resolved_inputs: &inputs,
            env: &self.env,
            skill_directory: &self.skill_directory,
            credential_delivery: &self.credential_delivery,
            skill_name: tool,
        };
        match resolve_and_invoke_local_tool(&request, Instant::now())? {
            Some(output) => Ok(output),
            None => Err(RuntimeError::SkillFailed {
                skill_name: MANAGED_AGENT_SKILL.to_owned(),
                message: format!("managed agent tool '{tool}' did not resolve to a local tool"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_tool_is_an_error() {
        let executor = RuntimeToolExecutor::new(
            BTreeMap::new(),
            PathBuf::from("."),
            CredentialDelivery::none(),
        );
        let result = executor.execute("definitely-not-a-real-tool", &JsonValue::Null);
        assert!(
            matches!(&result, Err(RuntimeError::SkillFailed { .. })),
            "an unresolved tool must fail, not panic or succeed; got: {result:?}"
        );
    }

    #[test]
    fn non_object_input_coerces_without_panicking() {
        // A misbehaving model can emit a non-object input. It must coerce to empty
        // args (not panic), and the unresolved tool then fails cleanly.
        let executor = RuntimeToolExecutor::new(
            BTreeMap::new(),
            PathBuf::from("."),
            CredentialDelivery::none(),
        );
        let result =
            executor.execute("definitely-not-a-real-tool", &JsonValue::String("oops".to_owned()));
        assert!(
            matches!(&result, Err(RuntimeError::SkillFailed { .. })),
            "a non-object input must coerce, not panic; got: {result:?}"
        );
    }
}
