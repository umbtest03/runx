//! Act receipt envelope returned across adapter boundaries.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};

use crate::schema::RunxSchema;
use crate::{JsonObject, ResolutionRequest};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActReceiptTerminalStatus {
    Sealed,
    Failure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum ActReceiptNeedsAgentStatus {
    #[serde(rename = "needs_agent")]
    NeedsAgent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum ActReceiptSignal {
    SIGABRT,
    SIGALRM,
    SIGBUS,
    SIGCHLD,
    SIGCONT,
    SIGFPE,
    SIGHUP,
    SIGILL,
    SIGINT,
    SIGIO,
    SIGIOT,
    SIGKILL,
    SIGPIPE,
    SIGPOLL,
    SIGPROF,
    SIGPWR,
    SIGQUIT,
    SIGSEGV,
    SIGSTKFLT,
    SIGSTOP,
    SIGSYS,
    SIGTERM,
    SIGTRAP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGUNUSED,
    SIGURG,
    SIGUSR1,
    SIGUSR2,
    SIGVTALRM,
    SIGWINCH,
    SIGXCPU,
    SIGXFSZ,
    SIGBREAK,
    SIGLOST,
    SIGINFO,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActReceiptNull;

impl Serialize for ActReceiptNull {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for ActReceiptNull {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<Value>::deserialize(deserializer)?
            .is_none()
            .then_some(Self)
            .ok_or_else(|| serde::de::Error::custom("field must be null"))
    }
}

impl RunxSchema for ActReceiptNull {
    fn json_schema() -> Value {
        json!({ "type": "null" })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActReceiptTerminalEnvelope {
    pub status: ActReceiptTerminalStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i64>,
    pub signal: Option<ActReceiptSignal>,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonObject>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActReceiptNeedsAgentEnvelope {
    pub status: ActReceiptNeedsAgentStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: ActReceiptNull,
    pub signal: ActReceiptNull,
    pub duration_ms: u64,
    pub request: ResolutionRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonObject>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(untagged)]
#[runx_schema(spec_id = "https://runx.ai/spec/act-receipt.schema.json")]
pub enum ActReceiptEnvelope {
    Terminal(ActReceiptTerminalEnvelope),
    NeedsAgent(ActReceiptNeedsAgentEnvelope),
}
