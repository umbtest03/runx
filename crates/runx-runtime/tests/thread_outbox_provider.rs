use std::path::PathBuf;

use runx_contracts::{
    CredentialDeliveryMode, CredentialDeliveryObservation, CredentialDeliveryObservationStatus,
    CredentialDeliveryPurpose, CredentialMaterialRole, Reference, ReferenceType,
    ThreadOutboxProviderFetch, ThreadOutboxProviderIdempotencyStatus, ThreadOutboxProviderManifest,
    ThreadOutboxProviderObservationStatus, ThreadOutboxProviderPush,
};
use runx_core::policy::{CredentialBindingDecision, CredentialEnvelope};
use runx_runtime::{
    CredentialDelivery, CredentialDeliveryProfile, InMemoryMaterialResolver,
    ResolvedCredentialMaterial, ThreadOutboxProviderProcessSupervisor,
    ThreadOutboxProviderSupervisorError,
};

#[derive(Debug, serde::Deserialize)]
struct Fixture<T> {
    expected: T,
}

#[test]
fn provider_process_pushes_idempotently_and_injects_delivery_observation()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest_with_fixture_args(&["push", "created"])?;
    let push = push_fixture()?;
    let delivery = credential_delivery()?;

    let outcome = ThreadOutboxProviderProcessSupervisor::default()
        .invoke_push(&manifest, &push, &delivery)?;

    assert_eq!(
        outcome.observation.status,
        ThreadOutboxProviderObservationStatus::Accepted
    );
    assert_eq!(
        outcome.observation.idempotency.status,
        ThreadOutboxProviderIdempotencyStatus::Created
    );
    assert_eq!(
        outcome
            .observation
            .delivery_observations
            .as_ref()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        outcome
            .observation
            .provider_locator
            .as_ref()
            .map(|locator| locator.locator.as_str()),
        Some("runxhq/runx#77/comment-1001")
    );
    Ok(())
}

#[test]
fn provider_process_reports_idempotent_replay() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest_with_fixture_args(&["push", "replayed"])?;
    let push = push_fixture()?;

    let outcome = ThreadOutboxProviderProcessSupervisor::default().invoke_push(
        &manifest,
        &push,
        &CredentialDelivery::none(),
    )?;

    assert_eq!(
        outcome.observation.idempotency.status,
        ThreadOutboxProviderIdempotencyStatus::Replayed
    );
    Ok(())
}

#[test]
fn provider_process_fetch_shapes_readback_receipt() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest_with_fixture_args(&["fetch"])?;
    let fetch = fetch_fixture()?;

    let outcome = ThreadOutboxProviderProcessSupervisor::default().invoke_fetch(
        &manifest,
        &fetch,
        &CredentialDelivery::none(),
    )?;

    assert_eq!(
        outcome
            .observation
            .readback_summary
            .as_ref()
            .map(|summary| summary.item_count),
        Some(1)
    );
    assert_eq!(
        outcome.observation.provider_event_id_hash.as_deref(),
        Some("sha256:github-comment-1001")
    );
    Ok(())
}

#[test]
fn provider_process_rejects_http_endpoint_manifest() -> Result<(), Box<dyn std::error::Error>> {
    let mut manifest = manifest_with_fixture_args(&["push"])?;
    manifest.transport.endpoint = Some("https://example.test/provider".to_owned());

    let result = ThreadOutboxProviderProcessSupervisor::default().invoke_push(
        &manifest,
        &push_fixture()?,
        &CredentialDelivery::none(),
    );

    assert!(matches!(
        result,
        Err(ThreadOutboxProviderSupervisorError::UnsupportedTransport)
    ));
    Ok(())
}

#[test]
fn provider_process_rejects_secret_like_response_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest = manifest_with_fixture_args(&["secret-field"])?;
    let push = push_fixture()?;

    let result = ThreadOutboxProviderProcessSupervisor::default().invoke_push(
        &manifest,
        &push,
        &CredentialDelivery::none(),
    );

    assert!(matches!(
        result,
        Err(ThreadOutboxProviderSupervisorError::SecretFieldRejected { field })
            if field == "$.access_token"
    ));
    Ok(())
}

#[test]
fn provider_process_redacts_credentials_from_stderr_and_observation()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest_with_fixture_args(&["leaky"])?;
    let push = push_fixture()?;
    let delivery = credential_delivery()?;

    let outcome = ThreadOutboxProviderProcessSupervisor::default()
        .invoke_push(&manifest, &push, &delivery)?;

    assert!(outcome.redacted_stderr.contains("[redacted-credential]"));
    assert!(!outcome.redacted_stderr.contains("ghs_TEST_SECRET_TOKEN"));
    let errors = outcome.observation.errors.as_ref();
    let message = errors
        .and_then(|errors| errors.first())
        .map(|error| error.message.as_str());
    assert_eq!(message, Some("provider mentioned [redacted-credential]"));
    Ok(())
}

fn manifest_with_fixture_args(
    fixture_args: &[&str],
) -> Result<ThreadOutboxProviderManifest, Box<dyn std::error::Error>> {
    let mut manifest = manifest_fixture()?;
    let mut args = vec![fixture_script()?.to_string_lossy().into_owned()];
    args.extend(fixture_args.iter().map(|arg| (*arg).to_owned()));
    manifest.transport.command = Some("sh".to_owned());
    manifest.transport.args = Some(args);
    Ok(manifest)
}

fn manifest_fixture() -> Result<ThreadOutboxProviderManifest, serde_json::Error> {
    let fixture: Fixture<ThreadOutboxProviderManifest> = serde_json::from_str(include_str!(
        "../../../fixtures/contracts/thread-outbox-provider/manifest.json"
    ))?;
    Ok(fixture.expected)
}

fn push_fixture() -> Result<ThreadOutboxProviderPush, serde_json::Error> {
    let fixture: Fixture<ThreadOutboxProviderPush> = serde_json::from_str(include_str!(
        "../../../fixtures/contracts/thread-outbox-provider/push.json"
    ))?;
    Ok(fixture.expected)
}

fn fetch_fixture() -> Result<ThreadOutboxProviderFetch, serde_json::Error> {
    let fixture: Fixture<ThreadOutboxProviderFetch> = serde_json::from_str(include_str!(
        "../../../fixtures/contracts/thread-outbox-provider/fetch.json"
    ))?;
    Ok(fixture.expected)
}

fn fixture_script() -> Result<PathBuf, std::io::Error> {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/runtime/thread-outbox-provider/mock-provider.sh"),
    )
}

fn credential_delivery() -> Result<CredentialDelivery, Box<dyn std::error::Error>> {
    let profile = CredentialDeliveryProfile::env_token("github", "oauth_bearer", "GITHUB_TOKEN")?;
    let credential: CredentialEnvelope = serde_json::from_value(serde_json::json!({
        "kind": "credential_envelope.v1",
        "grant_id": "grant-github",
        "provider": "github",
        "auth_mode": "oauth_bearer",
        "material_kind": "access_token",
        "scopes": ["issues:write"],
        "material_ref": "secret://github/main"
    }))?;
    let resolver = InMemoryMaterialResolver::with_material(
        "secret://github/main",
        ResolvedCredentialMaterial::access_token("secret://github/main", "ghs_TEST_SECRET_TOKEN"),
    );
    let delivery = CredentialDelivery::from_allowed_binding(
        &CredentialBindingDecision::Allow {
            reasons: vec!["test grant".to_owned()],
        },
        &credential,
        &profile,
        &resolver,
    )?
    .with_public_observation(CredentialDeliveryObservation {
        schema: "runx.credential_delivery.observation.v1".to_owned(),
        observation_id: "cred_obs_123".to_owned(),
        request_id: "cred_req_123".to_owned(),
        response_id: Some("cred_resp_123".to_owned()),
        status: CredentialDeliveryObservationStatus::Delivered,
        harness_ref: Reference {
            reference_type: ReferenceType::Harness,
            uri: "runx:harness:hrn_123".to_owned().into(),
            provider: None,
            locator: None,
            label: None,
            observed_at: None,
            proof_kind: None,
        },
        host_ref: Some(Reference {
            reference_type: ReferenceType::Host,
            uri: "runx:host:local-cli".to_owned().into(),
            provider: None,
            locator: None,
            label: None,
            observed_at: None,
            proof_kind: None,
        }),
        profile_id: "github-provider-api-env".to_owned(),
        provider: "github".to_owned(),
        purpose: CredentialDeliveryPurpose::ProviderApi,
        delivery_mode: Some(CredentialDeliveryMode::ProcessEnv),
        credential_refs: vec![Reference {
            reference_type: ReferenceType::Credential,
            uri: "runx:credential:github-installation:123".to_owned().into(),
            provider: Some("github".to_owned().into()),
            locator: None,
            label: None,
            observed_at: None,
            proof_kind: None,
        }],
        material_ref_hash: Some("sha256:material-ref".to_owned()),
        delivered_roles: vec![CredentialMaterialRole::AccessToken],
        redaction_refs: Some(vec![Reference {
            reference_type: ReferenceType::RedactionPolicy,
            uri: "runx:redaction_policy:provider-output".to_owned().into(),
            provider: None,
            locator: None,
            label: None,
            observed_at: None,
            proof_kind: None,
        }]),
        observed_at: "2026-05-22T00:00:00Z".to_owned(),
    });
    Ok(delivery)
}
