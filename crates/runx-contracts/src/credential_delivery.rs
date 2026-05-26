//! Credential delivery contracts: public refs, handles, and observations only.
use serde::{Deserialize, Serialize};

use crate::Reference;
use crate::schema::{IsoDateTime, NonEmptyString, RunxSchema};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialDeliveryMode {
    ProcessEnv,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialDeliveryPurpose {
    ProviderApi,
    Registry,
    ArtifactStore,
    WebhookVerification,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialMaterialRole {
    PersonalToken,
    ApiKey,
    ClientSecret,
    SessionToken,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialDeliveryStatus {
    Delivered,
    Denied,
    NotFound,
    ProfileMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialDeliveryObservationStatus {
    Delivered,
    Denied,
    NotDelivered,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum CredentialDeliveryProfileSchema {
    #[serde(rename = "runx.credential_delivery.profile.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum CredentialDeliveryRequestSchema {
    #[serde(rename = "runx.credential_delivery.request.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum CredentialDeliveryResponseSchema {
    #[serde(rename = "runx.credential_delivery.response.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum CredentialDeliveryObservationSchema {
    #[serde(rename = "runx.credential_delivery.observation.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct CredentialDeliveryEnvBinding {
    pub role: CredentialMaterialRole,
    pub env_var: String,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.credential_delivery.profile.v1")]
pub struct CredentialDeliveryProfile {
    pub schema: CredentialDeliveryProfileSchema,
    pub profile_id: NonEmptyString,
    pub provider: NonEmptyString,
    pub auth_mode: NonEmptyString,
    pub purpose: CredentialDeliveryPurpose,
    pub delivery_mode: CredentialDeliveryMode,
    pub material_roles: Vec<CredentialMaterialRole>,
    pub env_bindings: Vec<CredentialDeliveryEnvBinding>,
    pub redaction_policy_ref: Reference,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.credential_delivery.request.v1")]
pub struct CredentialDeliveryRequest {
    pub schema: CredentialDeliveryRequestSchema,
    pub request_id: NonEmptyString,
    pub harness_ref: Reference,
    pub host_ref: Reference,
    pub grant_ref: Reference,
    pub credential_ref: Reference,
    pub profile_id: NonEmptyString,
    pub provider: NonEmptyString,
    pub purpose: CredentialDeliveryPurpose,
    pub requested_roles: Vec<CredentialMaterialRole>,
    pub requested_at: IsoDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct CredentialDeliveryHandle {
    pub role: CredentialMaterialRole,
    pub delivery_handle_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_var: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.credential_delivery.response.v1")]
pub struct CredentialDeliveryResponse {
    pub schema: CredentialDeliveryResponseSchema,
    pub response_id: NonEmptyString,
    pub request_id: NonEmptyString,
    pub status: CredentialDeliveryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_mode: Option<CredentialDeliveryMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handles: Option<Vec<CredentialDeliveryHandle>>,
    pub credential_refs: Vec<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_ref_hash: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denied_reasons: Option<Vec<NonEmptyString>>,
    pub issued_at: IsoDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<IsoDateTime>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.credential_delivery.observation.v1")]
pub struct CredentialDeliveryObservation {
    pub schema: CredentialDeliveryObservationSchema,
    pub observation_id: NonEmptyString,
    pub request_id: NonEmptyString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<NonEmptyString>,
    pub status: CredentialDeliveryObservationStatus,
    pub harness_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_ref: Option<Reference>,
    pub profile_id: NonEmptyString,
    pub provider: NonEmptyString,
    pub purpose: CredentialDeliveryPurpose,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_mode: Option<CredentialDeliveryMode>,
    pub credential_refs: Vec<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_ref_hash: Option<NonEmptyString>,
    pub delivered_roles: Vec<CredentialMaterialRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redaction_refs: Option<Vec<Reference>>,
    pub observed_at: IsoDateTime,
}
