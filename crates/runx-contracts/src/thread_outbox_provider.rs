//! Thread outbox provider contract types.
use serde::{Deserialize, Serialize};

use crate::schema::{IsoDateTime, NonEmptyString, RunxSchema};
use crate::{
    CredentialDeliveryMode, CredentialDeliveryObservation, CredentialDeliveryPurpose, Reference,
};

pub const THREAD_OUTBOX_PROVIDER_PROTOCOL_VERSION: &str = "runx.thread_outbox_provider.v1";

/// The const `protocol_version` discriminant shared by every thread-outbox
/// provider frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum ThreadOutboxProviderProtocolVersion {
    #[serde(rename = "runx.thread_outbox_provider.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum ThreadOutboxProviderManifestSchema {
    #[serde(rename = "runx.thread_outbox_provider.manifest.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum ThreadOutboxProviderPushSchema {
    #[serde(rename = "runx.thread_outbox_provider.push.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum ThreadOutboxProviderFetchSchema {
    #[serde(rename = "runx.thread_outbox_provider.fetch.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum ThreadOutboxProviderObservationSchema {
    #[serde(rename = "runx.thread_outbox_provider.observation.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadOutboxProviderOperation {
    Push,
    Fetch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadOutboxProviderTransportKind {
    Process,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadOutboxProviderPayloadFormat {
    Markdown,
    PlainText,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadOutboxProviderObservationStatus {
    Accepted,
    Skipped,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadOutboxProviderIdempotencyStatus {
    Created,
    Replayed,
    Skipped,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderTransport {
    pub kind: ThreadOutboxProviderTransportKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<NonEmptyString>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderCredentialNeed {
    pub provider: NonEmptyString,
    pub purpose: CredentialDeliveryPurpose,
    pub profile_id: NonEmptyString,
    pub delivery_mode: CredentialDeliveryMode,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_refs: Option<Vec<Reference>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderReceiptCapabilities {
    pub idempotent_push: bool,
    pub readback: bool,
    pub stable_provider_event_hash: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderRedactionCapabilities {
    pub redacts_credentials: bool,
    pub redacts_provider_payloads: bool,
    pub supports_redaction_refs: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.thread_outbox_provider.manifest.v1")]
pub struct ThreadOutboxProviderManifest {
    pub schema: ThreadOutboxProviderManifestSchema,
    pub protocol_version: ThreadOutboxProviderProtocolVersion,
    pub adapter_id: NonEmptyString,
    pub provider: NonEmptyString,
    pub name: NonEmptyString,
    pub version: NonEmptyString,
    pub supported_operations: Vec<ThreadOutboxProviderOperation>,
    pub transport: ThreadOutboxProviderTransport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_needs: Option<Vec<ThreadOutboxProviderCredentialNeed>>,
    pub receipt_capabilities: ThreadOutboxProviderReceiptCapabilities,
    pub redaction_capabilities: ThreadOutboxProviderRedactionCapabilities,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderThreadLocator {
    pub provider: NonEmptyString,
    pub thread_ref: Reference,
    pub locator: NonEmptyString,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderLocator {
    pub provider: NonEmptyString,
    pub locator: NonEmptyString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_ref: Option<Reference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderIdempotency {
    pub key: NonEmptyString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<NonEmptyString>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderIdempotencyObservation {
    pub key: NonEmptyString,
    pub status: ThreadOutboxProviderIdempotencyStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_observation_ref: Option<Reference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderRenderedPayload {
    pub format: ThreadOutboxProviderPayloadFormat,
    pub body: NonEmptyString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_sha256: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redaction_refs: Option<Vec<Reference>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderCredentialProfile {
    pub provider: NonEmptyString,
    pub purpose: CredentialDeliveryPurpose,
    pub profile_id: NonEmptyString,
    pub delivery_mode: CredentialDeliveryMode,
    pub credential_refs: Vec<Reference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderReceiptContext {
    pub harness_ref: Reference,
    pub host_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_proof_refs: Option<Vec<Reference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_refs: Option<Vec<Reference>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.thread_outbox_provider.push.v1")]
pub struct ThreadOutboxProviderPush {
    pub schema: ThreadOutboxProviderPushSchema,
    pub protocol_version: ThreadOutboxProviderProtocolVersion,
    pub push_id: NonEmptyString,
    pub adapter_id: NonEmptyString,
    pub provider: NonEmptyString,
    pub outbox_entry_id: NonEmptyString,
    pub thread_locator: ThreadOutboxProviderThreadLocator,
    pub idempotency: ThreadOutboxProviderIdempotency,
    pub payload: ThreadOutboxProviderRenderedPayload,
    pub provider_profile: ThreadOutboxProviderCredentialProfile,
    pub credential_delivery_refs: Vec<Reference>,
    pub receipt_context: ThreadOutboxProviderReceiptContext,
    pub requested_at: IsoDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(untagged)]
pub enum ThreadOutboxProviderFetchTarget {
    Thread(ThreadOutboxProviderFetchThreadTarget),
    Provider(ThreadOutboxProviderFetchProviderTarget),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderFetchThreadTarget {
    pub thread_locator: ThreadOutboxProviderThreadLocator,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderFetchProviderTarget {
    pub provider_locator: ThreadOutboxProviderLocator,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.thread_outbox_provider.fetch.v1")]
pub struct ThreadOutboxProviderFetch {
    pub schema: ThreadOutboxProviderFetchSchema,
    pub protocol_version: ThreadOutboxProviderProtocolVersion,
    pub fetch_id: NonEmptyString,
    pub adapter_id: NonEmptyString,
    pub provider: NonEmptyString,
    pub target: ThreadOutboxProviderFetchTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readback_cursor: Option<NonEmptyString>,
    pub idempotency: ThreadOutboxProviderIdempotency,
    pub provider_profile: ThreadOutboxProviderCredentialProfile,
    pub credential_delivery_refs: Vec<Reference>,
    pub receipt_context: ThreadOutboxProviderReceiptContext,
    pub requested_at: IsoDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderReadbackSummary {
    pub item_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_provider_event_id_hash: Option<NonEmptyString>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadOutboxProviderError {
    pub code: NonEmptyString,
    pub message: NonEmptyString,
    pub retryable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.thread_outbox_provider.observation.v1")]
pub struct ThreadOutboxProviderObservation {
    pub schema: ThreadOutboxProviderObservationSchema,
    pub protocol_version: ThreadOutboxProviderProtocolVersion,
    pub observation_id: NonEmptyString,
    pub adapter_id: NonEmptyString,
    pub provider: NonEmptyString,
    pub operation: ThreadOutboxProviderOperation,
    pub request_id: NonEmptyString,
    pub status: ThreadOutboxProviderObservationStatus,
    pub idempotency: ThreadOutboxProviderIdempotencyObservation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_locator: Option<ThreadOutboxProviderLocator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_event_id_hash: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readback_summary: Option<ThreadOutboxProviderReadbackSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_observations: Option<Vec<CredentialDeliveryObservation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redaction_refs: Option<Vec<Reference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ThreadOutboxProviderError>>,
    pub observed_at: IsoDateTime,
}
