//! Redaction contracts: redacted-field markers and hash commitments.
use serde::{Deserialize, Serialize};

use crate::Reference;
use crate::schema::{IsoDateTime, NonEmptyString, RunxSchema};

pub const REDACTION_SCHEMA: &str = "runx.redaction.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum RedactionSchema {
    #[serde(rename = "runx.redaction.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum HashAlgorithm {
    Sha256,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct HashCommitment {
    pub algorithm: HashAlgorithm,
    pub value: NonEmptyString,
    pub canonicalization: NonEmptyString,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.redaction.v1")]
pub struct Redaction {
    pub schema: RedactionSchema,
    pub redaction_id: NonEmptyString,
    pub policy_ref: Reference,
    #[serde(default)]
    pub redacted_fields: Vec<NonEmptyString>,
    #[serde(default)]
    pub hash_commitments: Vec<HashCommitment>,
    pub canonicalization: NonEmptyString,
    pub performed_by_ref: Reference,
    pub performed_at: IsoDateTime,
}
