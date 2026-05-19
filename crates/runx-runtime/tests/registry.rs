use std::path::PathBuf;

use runx_contracts::{JsonNumber, JsonValue};
use runx_registry_client::{
    AcquiredRegistrySkill, InstallCandidate, InstallLocalSkillResult, InstallStatus,
    RegistryPublisher, TrustTier,
};
use runx_runtime::{RegistryInstallMetadataInput, registry_install_receipt_metadata};

#[test]
fn registry_install_metadata_records_installed_digest() {
    let candidate = install_candidate();
    let install = install_result(
        "sha256:installed",
        Some("sha256:profile-installed"),
        InstallStatus::Installed,
    );
    let acquisition = acquisition("sha256:remote-advertised", 7);

    let metadata = registry_install_receipt_metadata(RegistryInstallMetadataInput {
        candidate: &candidate,
        install: &install,
        acquisition: Some(&acquisition),
    });

    assert_eq!(metadata.get("ref"), Some(&string("acme/echo@1.0.0")));
    assert_eq!(metadata.get("skill_id"), Some(&string("acme/echo")));
    assert_eq!(metadata.get("version"), Some(&string("1.0.0")));
    assert_eq!(metadata.get("digest"), Some(&string("sha256:installed")));
    assert_eq!(
        metadata.get("profile_digest"),
        Some(&string("sha256:profile-installed"))
    );
    assert_eq!(metadata.get("trust_tier"), Some(&string("verified")));
    assert_eq!(metadata.get("source_label"), Some(&string("runx registry")));
    assert_eq!(
        metadata.get("destination"),
        Some(&string("/tmp/runx/skills/acme/echo/SKILL.md"))
    );
    assert_eq!(metadata.get("status"), Some(&string("installed")));
    assert_eq!(
        metadata.get("install_count"),
        Some(&JsonValue::Number(JsonNumber::U64(7)))
    );
    assert_eq!(
        metadata.get("publisher"),
        Some(&JsonValue::Object(
            [
                ("display_name".to_owned(), string("Acme")),
                ("handle".to_owned(), string("acme")),
                ("id".to_owned(), string("pub_1")),
                ("kind".to_owned(), string("organization")),
            ]
            .into_iter()
            .collect()
        ))
    );
}

#[test]
fn registry_install_metadata_omits_absent_remote_fields() {
    let candidate = install_candidate();
    let install = install_result("sha256:installed", None, InstallStatus::Unchanged);

    let metadata = registry_install_receipt_metadata(RegistryInstallMetadataInput {
        candidate: &candidate,
        install: &install,
        acquisition: None,
    });

    assert_eq!(metadata.get("digest"), Some(&string("sha256:installed")));
    assert_eq!(metadata.get("status"), Some(&string("unchanged")));
    assert!(!metadata.contains_key("profile_digest"));
    assert!(!metadata.contains_key("publisher"));
    assert!(!metadata.contains_key("install_count"));
}

fn install_candidate() -> InstallCandidate {
    InstallCandidate {
        markdown: "---\nname: echo\n---\n# Echo\n".to_owned(),
        profile_document: None,
        source: "runx-registry".to_owned(),
        source_label: "runx registry".to_owned(),
        r#ref: "acme/echo@1.0.0".to_owned(),
        skill_id: Some("acme/echo".to_owned()),
        version: Some("1.0.0".to_owned()),
        digest: Some("sha256:advertised".to_owned()),
        profile_digest: None,
        runner_names: Vec::new(),
        trust_tier: Some(TrustTier::Verified),
    }
}

fn install_result(
    digest: &str,
    profile_digest: Option<&str>,
    status: InstallStatus,
) -> InstallLocalSkillResult {
    InstallLocalSkillResult {
        status,
        destination: PathBuf::from("/tmp/runx/skills/acme/echo/SKILL.md"),
        skill_name: "echo".to_owned(),
        source: "runx-registry".to_owned(),
        source_label: "runx registry".to_owned(),
        skill_id: Some("acme/echo".to_owned()),
        version: Some("1.0.0".to_owned()),
        digest: digest.to_owned(),
        profile_digest: profile_digest.map(str::to_owned),
        profile_state_path: None,
        runner_names: Vec::new(),
        trust_tier: Some(TrustTier::Verified),
    }
}

fn acquisition(digest: &str, install_count: u64) -> AcquiredRegistrySkill {
    AcquiredRegistrySkill {
        skill_id: "acme/echo".to_owned(),
        owner: "acme".to_owned(),
        name: "echo".to_owned(),
        version: "1.0.0".to_owned(),
        digest: digest.to_owned(),
        markdown: "---\nname: echo\n---\n# Echo\n".to_owned(),
        profile_document: None,
        profile_digest: None,
        runner_names: Vec::new(),
        trust_tier: TrustTier::Verified,
        publisher: RegistryPublisher {
            kind: "organization".to_owned(),
            id: "pub_1".to_owned(),
            handle: Some("acme".to_owned()),
            display_name: Some("Acme".to_owned()),
        },
        source_metadata: None,
        attestations: Vec::new(),
        install_count,
    }
}

fn string(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}
