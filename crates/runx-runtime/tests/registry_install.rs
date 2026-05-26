use runx_contracts::sha256_prefixed;
use runx_runtime::registry::{
    InstallCandidate, InstallError, InstallLocalSkillOptions, RegistryManifestSignature,
    RegistryManifestSigner, RegistrySignedManifest, TrustTier, TrustedRegistryManifestKey,
    install_local_skill,
};
use tempfile::tempdir;

const TEST_MANIFEST_KEY_ID: &str = "runx-registry-test-key";
const TEST_MANIFEST_SIGNER_ID: &str = "runx-registry-test-signer";
const TEST_MANIFEST_PUBLIC_KEY_BASE64: &str = "K9U/1+6tuu9O5YfBO++MHrdr95NlPe1Okyg9XS7eWm0=";
const TEST_MANIFEST_SIGNATURE: &str =
    "base64:e-DzjjAZRv4inUscSd43cfT5287lIkvkM1YqgsFy1pZ9PkHEJCKp5Hm-zdlAY1D7ItVLNEw8HTM03lhgPk4hCg";
const TEST_MANIFEST_OTHER_SKILL_SIGNATURE: &str =
    "base64:h0WA5oT6vN3L5jQ76o79l533P3kE2tw1tphqgDcmmQu0_DcsfhPNAI05w1njHzyYib_CUnjPpYpx0c8MsJOMAw";

#[test]
fn trusted_signed_manifest_installs() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let candidate = install_candidate()?;

    let install = install_local_skill(
        &candidate,
        &InstallLocalSkillOptions {
            destination_root: temp.path().join("skills"),
            expected_digest: None,
            trusted_manifest_keys: trusted_manifest_keys()?,
        },
    )?;

    assert_eq!(install.skill_id.as_deref(), Some("acme/echo"));
    assert_eq!(install.digest, skill_digest());
    assert!(install.destination.exists());
    Ok(())
}

#[test]
fn tampered_content_fails_against_signed_manifest() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut candidate = install_candidate()?;
    candidate.markdown = candidate.markdown.replace("Echo", "Tampered");

    let error = install_error(&candidate, temp.path())?;

    assert!(matches!(error, InstallError::DigestMismatch { .. }));
    assert!(!temp.path().join("skills").exists());
    Ok(())
}

#[test]
fn unsigned_candidate_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut candidate = install_candidate()?;
    candidate.signed_manifest = None;

    let error = install_error(&candidate, temp.path())?;

    assert!(matches!(error, InstallError::UnsignedManifest(_)));
    assert!(!temp.path().join("skills").exists());
    Ok(())
}

#[test]
fn mismatched_manifest_identity_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut candidate = install_candidate()?;
    candidate.signed_manifest = Some(signed_manifest(
        "acme/other",
        "1.0.0",
        &skill_digest(),
        Some(&profile_digest()),
        TEST_MANIFEST_OTHER_SKILL_SIGNATURE,
    ));

    let error = install_error(&candidate, temp.path())?;

    assert!(matches!(
        error,
        InstallError::ManifestIdentityMismatch { .. }
    ));
    assert!(!temp.path().join("skills").exists());
    Ok(())
}

#[test]
fn missing_manifest_identity_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut candidate = install_candidate()?;
    candidate.skill_id = None;

    let error = install_error(&candidate, temp.path())?;

    assert!(matches!(
        error,
        InstallError::ManifestIdentityMissing {
            field: "skill_id",
            ..
        }
    ));
    assert!(!temp.path().join("skills").exists());
    Ok(())
}

#[test]
fn unknown_manifest_key_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut candidate = install_candidate()?;
    let manifest = candidate
        .signed_manifest
        .as_mut()
        .ok_or("signed manifest missing from fixture")?;
    manifest.signer.key_id = "unknown-key".to_owned();

    let error = install_error(&candidate, temp.path())?;

    assert!(matches!(error, InstallError::UnknownManifestKey { .. }));
    assert!(!temp.path().join("skills").exists());
    Ok(())
}

#[test]
fn invalid_manifest_signature_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut candidate = install_candidate()?;
    let manifest = candidate
        .signed_manifest
        .as_mut()
        .ok_or("signed manifest missing from fixture")?;
    manifest.signature = RegistryManifestSignature {
        alg: "ed25519".to_owned(),
        value: "base64:invalid".to_owned(),
    };

    let error = install_error(&candidate, temp.path())?;

    assert!(matches!(
        error,
        InstallError::InvalidManifestSignature { .. }
    ));
    assert!(!temp.path().join("skills").exists());
    Ok(())
}

#[test]
fn malformed_signed_manifest_payload_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut candidate = install_candidate()?;
    let manifest = candidate
        .signed_manifest
        .as_mut()
        .ok_or("signed manifest missing from fixture")?;
    manifest.skill_id = "acme/echo\nversion=1.0.0".to_owned();

    let error = install_error(&candidate, temp.path())?;

    assert!(matches!(
        error,
        InstallError::InvalidManifestSignature { reason, .. } if reason == "malformed payload"
    ));
    assert!(!temp.path().join("skills").exists());
    Ok(())
}

fn install_error(
    candidate: &InstallCandidate,
    temp_path: &std::path::Path,
) -> Result<InstallError, Box<dyn std::error::Error>> {
    match install_local_skill(
        candidate,
        &InstallLocalSkillOptions {
            destination_root: temp_path.join("skills"),
            expected_digest: None,
            trusted_manifest_keys: trusted_manifest_keys()?,
        },
    ) {
        Ok(_) => Err("install should fail".into()),
        Err(error) => Ok(error),
    }
}

fn install_candidate() -> Result<InstallCandidate, Box<dyn std::error::Error>> {
    Ok(InstallCandidate {
        markdown: include_str!("../../../fixtures/registry/install/echo-SKILL.md").to_owned(),
        profile_document: Some(
            include_str!("../../../fixtures/registry/install/echo-X.yaml").to_owned(),
        ),
        source: "runx-registry".to_owned(),
        source_label: "runx registry".to_owned(),
        r#ref: "acme/echo@1.0.0".to_owned(),
        skill_id: Some("acme/echo".to_owned()),
        version: Some("1.0.0".to_owned()),
        signed_manifest: Some(signed_manifest(
            "acme/echo",
            "1.0.0",
            &skill_digest(),
            Some(&profile_digest()),
            TEST_MANIFEST_SIGNATURE,
        )),
        profile_digest: None,
        runner_names: vec!["default".to_owned()],
        trust_tier: Some(TrustTier::Community),
    })
}

fn skill_digest() -> String {
    sha256_prefixed(include_str!("../../../fixtures/registry/install/echo-SKILL.md").as_bytes())
}

fn profile_digest() -> String {
    sha256_prefixed(include_str!("../../../fixtures/registry/install/echo-X.yaml").as_bytes())
}

fn trusted_manifest_keys() -> Result<Vec<TrustedRegistryManifestKey>, Box<dyn std::error::Error>> {
    Ok(vec![TrustedRegistryManifestKey::from_base64(
        TEST_MANIFEST_KEY_ID.to_owned(),
        TEST_MANIFEST_PUBLIC_KEY_BASE64,
    )?])
}

fn signed_manifest(
    skill_id: &str,
    version: &str,
    digest: &str,
    profile_digest: Option<&str>,
    signature: &str,
) -> RegistrySignedManifest {
    RegistrySignedManifest {
        schema: runx_runtime::registry::REGISTRY_SIGNED_MANIFEST_SCHEMA.to_owned(),
        skill_id: skill_id.to_owned(),
        version: version.to_owned(),
        digest: digest.to_owned(),
        profile_digest: profile_digest.map(str::to_owned),
        signer: RegistryManifestSigner {
            id: TEST_MANIFEST_SIGNER_ID.to_owned(),
            key_id: TEST_MANIFEST_KEY_ID.to_owned(),
        },
        signature: RegistryManifestSignature {
            alg: "ed25519".to_owned(),
            value: signature.to_owned(),
        },
    }
}
