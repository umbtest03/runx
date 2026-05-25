//! Run summary report contract.
use serde::{Deserialize, Serialize};

use crate::{JsonObject, schema::RunxSchema};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum RunSummarySchema {
    #[serde(rename = "runx.run-summary.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunSummaryStatus {
    Success,
    Failure,
    Skipped,
    NeedsApproval,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.run-summary.v1")]
pub struct RunSummary {
    pub schema: RunSummarySchema,
    pub run_id: String,
    pub command: String,
    pub status: RunSummaryStatus,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    pub root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<JsonObject>,
    pub steps: Vec<JsonObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_ref: Option<String>,
}
