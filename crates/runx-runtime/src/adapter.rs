use std::collections::BTreeMap;
use std::path::PathBuf;

use runx_contracts::JsonObject;
use runx_parser::SkillSource;
use serde::{Deserialize, Serialize};

use crate::RuntimeError;
use crate::credentials::CredentialDelivery;

/// Metadata key under which a skill's non-secret credential-delivery
/// observations are recorded on [`SkillOutput::metadata`].
pub const CREDENTIAL_DELIVERY_OBSERVATIONS_METADATA: &str = "credential_delivery_observations";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvocationStatus {
    Success,
    Failure,
}

#[derive(Clone, Debug)]
pub struct SkillInvocation {
    pub skill_name: String,
    pub source: SkillSource,
    pub inputs: JsonObject,
    pub resolved_inputs: JsonObject,
    pub skill_directory: PathBuf,
    pub env: BTreeMap<String, String>,
    pub credential_delivery: CredentialDelivery,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillOutput {
    pub status: InvocationStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub metadata: JsonObject,
}

impl SkillOutput {
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.status == InvocationStatus::Success
    }
}

pub trait SkillAdapter {
    fn adapter_type(&self) -> &'static str;
    fn invoke(&self, request: SkillInvocation) -> Result<SkillOutput, RuntimeError>;
}
