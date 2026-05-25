//! Dev-loop report contracts.
use serde::{Deserialize, Serialize};

use crate::schema::RunxSchema;
use crate::{DoctorReport, JsonObject, JsonValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum DevReportSchema {
    #[serde(rename = "runx.dev.v1")]
    V1,
}

impl PartialEq<&str> for DevReportSchema {
    fn eq(&self, other: &&str) -> bool {
        matches!((self, *other), (Self::V1, "runx.dev.v1"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum DevReportStatus {
    Success,
    Failure,
    Skipped,
    NeedsApproval,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum DevFixtureStatus {
    Success,
    Failure,
    Skipped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum DevFixtureAssertionKind {
    SubsetMiss,
    ExactMismatch,
    PacketInvalid,
    StatusMismatch,
    TypeMismatch,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct DevFixtureAssertion {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<JsonValue>,
    pub kind: DevFixtureAssertionKind,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct DevFixtureResult {
    pub name: String,
    pub lane: String,
    pub target: JsonObject,
    pub status: DevFixtureStatus,
    pub duration_ms: u64,
    pub assertions: Vec<DevFixtureAssertion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.dev.v1")]
pub struct DevReport {
    pub schema: DevReportSchema,
    pub status: DevReportStatus,
    pub doctor: DoctorReport,
    pub fixtures: Vec<DevFixtureResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_id: Option<String>,
}
