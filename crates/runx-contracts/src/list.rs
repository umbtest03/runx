//! Native list command report contracts.
use serde::{Deserialize, Serialize};

use crate::schema::RunxSchema;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, RunxSchema)]
pub enum RunxListSchema {
    #[serde(rename = "runx.list.v1")]
    V1,
}

impl PartialEq<&str> for RunxListSchema {
    fn eq(&self, other: &&str) -> bool {
        matches!((self, *other), (Self::V1, "runx.list.v1"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "lowercase")]
pub enum RunxListRequestedKind {
    All,
    Tools,
    Skills,
    Graphs,
    Packets,
    Overlays,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "lowercase")]
pub enum RunxListItemKind {
    Tool,
    Skill,
    Graph,
    Packet,
    Overlay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RunxListSource {
    Local,
    Workspace,
    Dependencies,
    BuiltIn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "lowercase")]
pub enum RunxListStatus {
    Ok,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct RunxListEmit {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct RunxListItem {
    pub kind: RunxListItemKind,
    pub name: String,
    pub source: RunxListSource,
    pub path: String,
    pub status: RunxListStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emits: Option<Vec<RunxListEmit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixtures: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harness_cases: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wraps: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.list.v1")]
pub struct RunxListReport {
    pub schema: RunxListSchema,
    pub root: String,
    pub requested_kind: RunxListRequestedKind,
    pub items: Vec<RunxListItem>,
}
