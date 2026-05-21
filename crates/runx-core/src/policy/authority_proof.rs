// rust-style-allow: large-file - authority-proof parity keeps the TS oracle mapping in one reviewable module.
use runx_contracts::{JsonObject, JsonValue, sha256_hex};

use super::{
    LocalAdmissionGrant, LocalScopeAdmissionOptions, ScopeAdmission, ScopeAdmissionStatus,
    connected_auth::{
        ConnectedAuthRequirement, connected_auth_requirement, find_matching_grant,
        has_grant_reference,
    },
    scope::{scope_allows, unique_strings},
    types::{
        AuthorityProof, AuthorityProofApproval, AuthorityProofApprovalDecision,
        AuthorityProofCredentialMaterial, AuthorityProofMetadata, AuthorityProofRedaction,
        AuthorityProofRequested, AuthorityProofSandbox, AuthorityProofSandboxDeclaration,
        AuthorityProofSandboxFilesystem, AuthorityProofSandboxNetwork,
        AuthorityProofSandboxRuntime, BuildAuthorityProofOptions, CredentialBindingDecision,
        CredentialBindingRequest, CredentialEnvelope, CredentialGrantReference,
    },
};

const AUTHORITY_PROOF_SCHEMA_VERSION: &str = "runx.authority-proof.v1";

#[must_use]
pub fn build_local_scope_admission(
    auth: Option<&JsonValue>,
    grants: &[LocalAdmissionGrant],
    options: &LocalScopeAdmissionOptions,
) -> ScopeAdmission {
    let Some(requirement) = connected_auth_requirement(auth) else {
        return scope_admission_allow(Vec::new(), Vec::new(), None, "no connected auth requested");
    };

    let requested_scopes = unique_strings(&requirement.scopes);
    if options.denied_before_grant_resolution.unwrap_or(false) {
        return scope_admission_deny(
            requested_scopes,
            Vec::new(),
            vec!["structural policy denied before connected auth grant resolution".to_owned()],
            "structural policy denied before grant resolution",
        );
    }

    match find_matching_grant(
        &requirement,
        grants,
        options.connected_auth_checked_at.as_deref(),
        options.wildcard_scopes_trusted,
    ) {
        Some(grant) => scope_admission_allow(
            requested_scopes,
            unique_strings(&grant.scopes),
            Some(grant.grant_id.clone()),
            "matching active grant admitted",
        ),
        None => scope_admission_deny(
            requested_scopes,
            Vec::new(),
            vec![format!(
                "connected auth grant required for provider '{}'",
                requirement.provider
            )],
            "no matching active grant resolved",
        ),
    }
}

#[must_use]
pub fn build_authority_proof(options: &BuildAuthorityProofOptions) -> AuthorityProof {
    let requirement = connected_auth_requirement(options.auth.as_ref());
    let scope_admission = options.scope_admission.clone().unwrap_or_else(|| {
        build_local_scope_admission(
            options.auth.as_ref(),
            &options.grants,
            &LocalScopeAdmissionOptions {
                connected_auth_checked_at: options.connected_auth_checked_at.clone(),
                ..LocalScopeAdmissionOptions::default()
            },
        )
    });
    let sandbox = summarize_authority_sandbox(
        options.sandbox_metadata.as_ref(),
        options.sandbox_declaration.as_ref(),
        options.approval.as_ref(),
    );

    AuthorityProof {
        schema_version: AUTHORITY_PROOF_SCHEMA_VERSION.to_owned(),
        run_id: options.run_id.clone(),
        skill_name: options.skill_name.clone(),
        source_type: options.source_type.clone(),
        requested: authority_proof_requested(&requirement, &sandbox, options),
        scope_admission: scope_admission.clone(),
        credential_material: credential_material_proof(
            options.credential.as_ref(),
            requirement.as_ref(),
            &scope_admission,
        ),
        sandbox,
        approval_gate: options.approval.as_ref().map(approval_decision),
        redaction: authority_redaction(),
    }
}

#[must_use]
pub fn build_authority_proof_metadata(
    options: &BuildAuthorityProofOptions,
) -> AuthorityProofMetadata {
    AuthorityProofMetadata {
        authority_proof: build_authority_proof(options),
    }
}

#[must_use]
pub fn validate_credential_binding(
    request: &CredentialBindingRequest,
) -> CredentialBindingDecision {
    let requirement = connected_auth_requirement(request.auth.as_ref());
    match request.credential.as_ref() {
        None => validate_missing_credential(requirement.as_ref(), &request.scope_admission),
        Some(credential) => validate_resolved_credential(request, requirement.as_ref(), credential),
    }
}

fn validate_missing_credential(
    requirement: Option<&ConnectedAuthRequirement>,
    scope_admission: &ScopeAdmission,
) -> CredentialBindingDecision {
    if requirement.is_some()
        && scope_admission.status == ScopeAdmissionStatus::Allow
        && scope_admission.grant_id.is_some()
    {
        return deny(vec![
            "credential material was not resolved for admitted connected auth grant".to_owned(),
        ]);
    }
    allow(vec!["no credential material resolved".to_owned()])
}

fn validate_resolved_credential(
    request: &CredentialBindingRequest,
    requirement: Option<&ConnectedAuthRequirement>,
    credential: &CredentialEnvelope,
) -> CredentialBindingDecision {
    let Some(requirement) = requirement else {
        return deny(vec![
            "credential material resolved for a skill with no connected auth requirement"
                .to_owned(),
        ]);
    };
    let Some(admitted_grant_id) = admitted_grant_id(&request.scope_admission) else {
        return deny(vec![
            "credential material resolved without an admitted connected auth grant".to_owned(),
        ]);
    };
    let Some(admitted_grant) = request
        .grants
        .iter()
        .find(|grant| grant.grant_id == admitted_grant_id)
    else {
        return deny(vec![format!(
            "credential admission references grant '{admitted_grant_id}' that was not resolved",
        )]);
    };

    let reasons = credential_binding_reasons(
        credential,
        requirement,
        admitted_grant,
        &request.scope_admission,
    );
    if reasons.is_empty() {
        allow(vec![
            "credential material matches admitted grant".to_owned(),
        ])
    } else {
        deny(reasons)
    }
}

fn credential_binding_reasons(
    credential: &CredentialEnvelope,
    requirement: &ConnectedAuthRequirement,
    admitted_grant: &LocalAdmissionGrant,
    scope_admission: &ScopeAdmission,
) -> Vec<String> {
    let mut reasons = Vec::new();
    collect_credential_identity_reasons(credential, requirement, admitted_grant, &mut reasons);
    collect_credential_scope_reasons(credential, admitted_grant, scope_admission, &mut reasons);
    collect_credential_reference_reasons(credential, admitted_grant, &mut reasons);
    reasons
}

fn collect_credential_identity_reasons(
    credential: &CredentialEnvelope,
    requirement: &ConnectedAuthRequirement,
    admitted_grant: &LocalAdmissionGrant,
    reasons: &mut Vec<String>,
) {
    if credential.grant_id != admitted_grant.grant_id {
        reasons.push(format!(
            "credential grant_id '{}' does not match admitted grant '{}'",
            credential.grant_id, admitted_grant.grant_id
        ));
    }
    if credential.provider != requirement.provider || credential.provider != admitted_grant.provider
    {
        reasons.push(format!(
            "credential provider '{}' does not match admitted provider '{}'",
            credential.provider, admitted_grant.provider
        ));
    }
}

fn collect_credential_scope_reasons(
    credential: &CredentialEnvelope,
    admitted_grant: &LocalAdmissionGrant,
    scope_admission: &ScopeAdmission,
    reasons: &mut Vec<String>,
) {
    let missing_requested_scopes = scope_admission
        .requested_scopes
        .iter()
        .filter(|scope| {
            !credential
                .scopes
                .iter()
                .any(|credential_scope| scope_allows(credential_scope, scope, false))
        })
        .cloned()
        .collect::<Vec<_>>();
    if !missing_requested_scopes.is_empty() {
        reasons.push(format!(
            "credential scopes do not include admitted request scope(s): {}",
            missing_requested_scopes.join(", ")
        ));
    }

    let out_of_grant_scopes = credential
        .scopes
        .iter()
        .filter(|scope| {
            !admitted_grant
                .scopes
                .iter()
                .any(|granted_scope| scope_allows(granted_scope, scope, false))
        })
        .cloned()
        .collect::<Vec<_>>();
    if !out_of_grant_scopes.is_empty() {
        reasons.push(format!(
            "credential scopes exceed admitted grant scope(s): {}",
            out_of_grant_scopes.join(", ")
        ));
    }
}

fn collect_credential_reference_reasons(
    credential: &CredentialEnvelope,
    admitted_grant: &LocalAdmissionGrant,
    reasons: &mut Vec<String>,
) {
    let expected_reference = credential_grant_reference(admitted_grant);
    match (
        expected_reference.as_ref(),
        credential.grant_reference.as_ref(),
    ) {
        (Some(_), None) => {
            reasons.push(
                "credential grant_reference is missing for a targeted admitted grant".to_owned(),
            );
        }
        (Some(expected), Some(actual)) => {
            reasons.extend(grant_reference_mismatches(expected, actual));
        }
        (None, Some(_)) => {
            reasons.push(
                "credential grant_reference is present but the admitted grant is not targeted"
                    .to_owned(),
            );
        }
        (None, None) => {}
    }
}

fn authority_proof_requested(
    requirement: &Option<ConnectedAuthRequirement>,
    sandbox: &Option<AuthorityProofSandbox>,
    options: &BuildAuthorityProofOptions,
) -> AuthorityProofRequested {
    AuthorityProofRequested {
        connected_auth: requirement.is_some(),
        scopes: requirement
            .as_ref()
            .map_or_else(Vec::new, |value| unique_strings(&value.scopes)),
        mutating: options.mutating.unwrap_or(false),
        scope_family: requirement
            .as_ref()
            .and_then(|value| value.scope_family.clone()),
        authority_kind: requirement
            .as_ref()
            .and_then(|value| value.authority_kind.clone()),
        target_repo: requirement
            .as_ref()
            .and_then(|value| value.target_repo.clone()),
        target_locator: requirement
            .as_ref()
            .and_then(|value| value.target_locator.clone()),
        sandbox_profile: sandbox.as_ref().map(|value| value.profile.clone()),
    }
}

fn credential_material_proof(
    credential: Option<&CredentialEnvelope>,
    requirement: Option<&ConnectedAuthRequirement>,
    scope_admission: &ScopeAdmission,
) -> AuthorityProofCredentialMaterial {
    if let Some(credential) = credential {
        return resolved_credential_material(credential);
    }
    match requirement {
        None => AuthorityProofCredentialMaterial {
            status: "not_requested".to_owned(),
            ..empty_credential_material()
        },
        Some(requirement) => unresolved_credential_material(requirement, scope_admission),
    }
}

fn resolved_credential_material(
    credential: &CredentialEnvelope,
) -> AuthorityProofCredentialMaterial {
    AuthorityProofCredentialMaterial {
        status: "resolved".to_owned(),
        grant_id: Some(credential.grant_id.clone()),
        provider: Some(credential.provider.clone()),
        connection_id: credential.connection_id.clone(),
        scopes: Some(credential.scopes.clone()),
        grant_reference: credential.grant_reference.clone(),
        material_ref_hash: Some(sha256_hex(credential.material_ref.as_bytes())),
        ..empty_credential_material()
    }
}

fn unresolved_credential_material(
    requirement: &ConnectedAuthRequirement,
    scope_admission: &ScopeAdmission,
) -> AuthorityProofCredentialMaterial {
    AuthorityProofCredentialMaterial {
        status: if scope_admission.status == ScopeAdmissionStatus::Deny {
            "denied".to_owned()
        } else {
            "not_resolved".to_owned()
        },
        grant_id: scope_admission.grant_id.clone(),
        provider: Some(requirement.provider.clone()),
        scopes: Some(unique_strings(&requirement.scopes)),
        scope_family: requirement.scope_family.clone(),
        authority_kind: requirement.authority_kind.clone(),
        target_repo: requirement.target_repo.clone(),
        target_locator: requirement.target_locator.clone(),
        ..empty_credential_material()
    }
}

fn summarize_authority_sandbox(
    metadata: Option<&JsonValue>,
    declaration: Option<&AuthorityProofSandboxDeclaration>,
    approval: Option<&AuthorityProofApproval>,
) -> Option<AuthorityProofSandbox> {
    let record = json_object(metadata);
    let profile = string_field(record, "profile")
        .or_else(|| declaration.and_then(|value| optional_string(value.profile.as_deref())))?;
    let network = summarize_network(
        record.and_then(|value| object_field(value, "network")),
        declaration,
    );
    let filesystem =
        summarize_filesystem(record.and_then(|value| object_field(value, "filesystem")));
    let runtime = summarize_runtime(record.and_then(|value| object_field(value, "runtime")));

    Some(AuthorityProofSandbox {
        profile: profile.clone(),
        cwd_policy: string_field(record, "cwd_policy")
            .or_else(|| declaration.and_then(|value| optional_string(value.cwd_policy.as_deref()))),
        require_enforcement: bool_field(record, "require_enforcement")
            .or_else(|| declaration.and_then(|value| value.require_enforcement)),
        network,
        filesystem,
        runtime,
        approval_required: bool_field(
            record.and_then(|value| object_field(value, "approval")),
            "required",
        )
        .or(Some(profile == "unrestricted-local-dev")),
        approval_approved: bool_field(
            record.and_then(|value| object_field(value, "approval")),
            "approved",
        )
        .or_else(|| approval.map(|value| value.approved)),
    })
}

fn summarize_network(
    network: Option<&JsonObject>,
    declaration: Option<&AuthorityProofSandboxDeclaration>,
) -> Option<AuthorityProofSandboxNetwork> {
    if network.is_none() && declaration.and_then(|value| value.network).is_none() {
        return None;
    }
    let summary = AuthorityProofSandboxNetwork {
        declared: bool_field(network, "declared")
            .or_else(|| declaration.and_then(|value| value.network)),
        enforcement: string_field(network, "enforcement"),
    };
    if summary.declared.is_none() && summary.enforcement.is_none() {
        None
    } else {
        Some(summary)
    }
}

fn summarize_filesystem(
    filesystem: Option<&JsonObject>,
) -> Option<AuthorityProofSandboxFilesystem> {
    filesystem.and_then(|value| {
        let summary = AuthorityProofSandboxFilesystem {
            enforcement: string_field(Some(value), "enforcement"),
            readonly_paths: bool_field(Some(value), "readonly_paths"),
            writable_paths_enforced: bool_field(Some(value), "writable_paths_enforced"),
            private_tmp: bool_field(Some(value), "private_tmp"),
        };
        if summary.enforcement.is_none()
            && summary.readonly_paths.is_none()
            && summary.writable_paths_enforced.is_none()
            && summary.private_tmp.is_none()
        {
            None
        } else {
            Some(summary)
        }
    })
}

fn summarize_runtime(runtime: Option<&JsonObject>) -> Option<AuthorityProofSandboxRuntime> {
    runtime.and_then(|value| {
        let summary = AuthorityProofSandboxRuntime {
            enforcer: string_field(Some(value), "enforcer"),
            reason: string_field(Some(value), "reason"),
        };
        if summary.enforcer.is_none() && summary.reason.is_none() {
            None
        } else {
            Some(summary)
        }
    })
}

fn approval_decision(approval: &AuthorityProofApproval) -> AuthorityProofApprovalDecision {
    AuthorityProofApprovalDecision {
        gate_id: approval.gate.id.clone(),
        gate_type: approval
            .gate
            .gate_type
            .clone()
            .unwrap_or_else(|| "unspecified".to_owned()),
        decision: if approval.approved {
            "approved".to_owned()
        } else {
            "denied".to_owned()
        },
        reason: approval.gate.reason.clone(),
    }
}

fn authority_redaction() -> AuthorityProofRedaction {
    AuthorityProofRedaction {
        status: "applied".to_owned(),
        secret_material: "omitted".to_owned(),
        stdout: "hashed".to_owned(),
        stderr: "hashed".to_owned(),
        metadata_secret_keys: vec![
            "token-like metadata keys".to_owned(),
            "api-key-like metadata keys".to_owned(),
            "password-like metadata keys".to_owned(),
            "client-secret-like metadata keys".to_owned(),
            "raw-secret-like metadata keys".to_owned(),
        ],
    }
}

fn credential_grant_reference(grant: &LocalAdmissionGrant) -> Option<CredentialGrantReference> {
    if !has_grant_reference(grant) {
        return None;
    }
    Some(CredentialGrantReference {
        grant_id: grant.grant_id.clone(),
        scope_family: grant.scope_family.clone(),
        authority_kind: grant.authority_kind.clone(),
        target_repo: grant.target_repo.clone(),
        target_locator: grant.target_locator.clone(),
    })
}

fn grant_reference_mismatches(
    expected: &CredentialGrantReference,
    actual: &CredentialGrantReference,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if actual.grant_id != expected.grant_id {
        reasons
            .push("credential grant_reference.grant_id does not match admitted grant".to_owned());
    }
    if actual.scope_family != expected.scope_family {
        reasons.push(
            "credential grant_reference.scope_family does not match admitted grant".to_owned(),
        );
    }
    if actual.authority_kind != expected.authority_kind {
        reasons.push(
            "credential grant_reference.authority_kind does not match admitted grant".to_owned(),
        );
    }
    if actual.target_repo != expected.target_repo {
        reasons.push(
            "credential grant_reference.target_repo does not match admitted grant".to_owned(),
        );
    }
    if actual.target_locator != expected.target_locator {
        reasons.push(
            "credential grant_reference.target_locator does not match admitted grant".to_owned(),
        );
    }
    reasons
}

fn scope_admission_allow(
    requested_scopes: Vec<String>,
    granted_scopes: Vec<String>,
    grant_id: Option<String>,
    summary: &str,
) -> ScopeAdmission {
    ScopeAdmission {
        status: ScopeAdmissionStatus::Allow,
        requested_scopes,
        granted_scopes,
        grant_id,
        reasons: None,
        decision_summary: summary.to_owned(),
    }
}

fn scope_admission_deny(
    requested_scopes: Vec<String>,
    granted_scopes: Vec<String>,
    reasons: Vec<String>,
    summary: &str,
) -> ScopeAdmission {
    ScopeAdmission {
        status: ScopeAdmissionStatus::Deny,
        requested_scopes,
        granted_scopes,
        grant_id: None,
        reasons: Some(reasons),
        decision_summary: summary.to_owned(),
    }
}

fn admitted_grant_id(scope_admission: &ScopeAdmission) -> Option<&str> {
    if scope_admission.status != ScopeAdmissionStatus::Allow {
        return None;
    }
    scope_admission.grant_id.as_deref()
}

fn empty_credential_material() -> AuthorityProofCredentialMaterial {
    AuthorityProofCredentialMaterial {
        status: String::new(),
        grant_id: None,
        provider: None,
        connection_id: None,
        scopes: None,
        scope_family: None,
        authority_kind: None,
        target_repo: None,
        target_locator: None,
        grant_reference: None,
        material_ref_hash: None,
    }
}

fn allow(reasons: Vec<String>) -> CredentialBindingDecision {
    CredentialBindingDecision::Allow { reasons }
}

fn deny(reasons: Vec<String>) -> CredentialBindingDecision {
    CredentialBindingDecision::Deny { reasons }
}

fn json_object(value: Option<&JsonValue>) -> Option<&JsonObject> {
    match value {
        Some(JsonValue::Object(object)) => Some(object),
        _ => None,
    }
}

fn object_field<'a>(object: &'a JsonObject, field: &str) -> Option<&'a JsonObject> {
    match object.get(field) {
        Some(JsonValue::Object(value)) => Some(value),
        _ => None,
    }
}

fn string_field(object: Option<&JsonObject>, field: &str) -> Option<String> {
    match object.and_then(|value| value.get(field)) {
        Some(JsonValue::String(value)) if !value.trim().is_empty() => Some(value.trim().to_owned()),
        _ => None,
    }
}

fn optional_string(value: Option<&str>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

fn bool_field(object: Option<&JsonObject>, field: &str) -> Option<bool> {
    match object.and_then(|value| value.get(field)) {
        Some(JsonValue::Bool(value)) => Some(*value),
        _ => None,
    }
}
