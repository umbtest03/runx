use runx_contracts::{JsonNumber, JsonObject, JsonValue};
use runx_registry_client::{
    AcquiredRegistrySkill, InstallCandidate, InstallLocalSkillResult, InstallStatus,
    RegistryPublisher, TrustTier,
};

#[derive(Clone, Copy, Debug)]
pub struct RegistryInstallMetadataInput<'a> {
    pub candidate: &'a InstallCandidate,
    pub install: &'a InstallLocalSkillResult,
    pub acquisition: Option<&'a AcquiredRegistrySkill>,
}

#[must_use]
pub fn registry_install_receipt_metadata(input: RegistryInstallMetadataInput<'_>) -> JsonObject {
    let mut metadata = JsonObject::new();
    insert_string(&mut metadata, "ref", &input.candidate.r#ref);
    insert_optional_string(&mut metadata, "skill_id", input.install.skill_id.as_deref());
    insert_optional_string(&mut metadata, "version", input.install.version.as_deref());
    insert_string(&mut metadata, "digest", &input.install.digest);
    insert_optional_string(
        &mut metadata,
        "profile_digest",
        input.install.profile_digest.as_deref(),
    );
    insert_optional_string(
        &mut metadata,
        "trust_tier",
        input.install.trust_tier.as_ref().map(trust_tier_value),
    );
    if let Some(acquisition) = input.acquisition {
        metadata.insert(
            "publisher".to_owned(),
            JsonValue::Object(publisher_metadata(&acquisition.publisher)),
        );
        metadata.insert(
            "install_count".to_owned(),
            JsonValue::Number(JsonNumber::U64(acquisition.install_count)),
        );
    }
    insert_string(&mut metadata, "source_label", &input.install.source_label);
    insert_string(
        &mut metadata,
        "destination",
        &input.install.destination.display().to_string(),
    );
    insert_string(
        &mut metadata,
        "status",
        match input.install.status {
            InstallStatus::Installed => "installed",
            InstallStatus::Unchanged => "unchanged",
        },
    );
    metadata
}

fn publisher_metadata(publisher: &RegistryPublisher) -> JsonObject {
    let mut metadata = JsonObject::new();
    insert_string(&mut metadata, "kind", &publisher.kind);
    insert_string(&mut metadata, "id", &publisher.id);
    insert_optional_string(&mut metadata, "handle", publisher.handle.as_deref());
    insert_optional_string(
        &mut metadata,
        "display_name",
        publisher.display_name.as_deref(),
    );
    metadata
}

fn insert_string(metadata: &mut JsonObject, key: &str, value: &str) {
    metadata.insert(key.to_owned(), JsonValue::String(value.to_owned()));
}

fn insert_optional_string(metadata: &mut JsonObject, key: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        insert_string(metadata, key, value);
    }
}

fn trust_tier_value(value: &TrustTier) -> &'static str {
    match value {
        TrustTier::FirstParty => "first_party",
        TrustTier::Verified => "verified",
        TrustTier::Community => "community",
    }
}
