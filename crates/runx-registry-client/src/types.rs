use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    FirstParty,
    Verified,
    Community,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileMode {
    Portable,
    Profiled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryPublisher {
    pub kind: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegistryAttestation {
    pub kind: String,
    pub id: String,
    pub status: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegistrySourceMetadata {
    pub provider: String,
    pub repo: String,
    pub repo_url: String,
    pub skill_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<String>,
    pub r#ref: String,
    pub sha: String,
    pub default_branch: String,
    pub event: String,
    pub immutable: bool,
    pub live: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tombstoned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_handle: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegistrySearchResult {
    pub skill_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub source_type: String,
    pub profile_mode: ProfileMode,
    pub runner_names: Vec<String>,
    pub required_scopes: Vec<String>,
    pub tags: Vec<String>,
    pub trust_tier: TrustTier,
    pub install_command: String,
    pub run_command: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegistrySkillDetail {
    pub skill_id: String,
    pub owner: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: String,
    pub digest: String,
    pub markdown: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_digest: Option<String>,
    pub runner_names: Vec<String>,
    pub source_type: String,
    pub trust_tier: TrustTier,
    pub required_scopes: Vec<String>,
    pub tags: Vec<String>,
    pub publisher: RegistryPublisher,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_metadata: Option<RegistrySourceMetadata>,
    pub attestations: Vec<RegistryAttestation>,
    pub install_command: String,
    pub run_command: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcquiredRegistrySkill {
    pub skill_id: String,
    pub owner: String,
    pub name: String,
    pub version: String,
    pub digest: String,
    pub markdown: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_document: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_digest: Option<String>,
    pub runner_names: Vec<String>,
    pub trust_tier: TrustTier,
    pub publisher: RegistryPublisher,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_metadata: Option<RegistrySourceMetadata>,
    pub attestations: Vec<RegistryAttestation>,
    pub install_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRegistryRef {
    pub skill_id: String,
    pub version: Option<String>,
}
