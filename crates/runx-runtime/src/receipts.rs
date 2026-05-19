// rust-style-allow: large-file because receipt construction and local proof
// sealing stay together until the runtime receipt builder is split out.
use runx_contracts::{
    Act, ActForm, Authority, AuthorityAttenuation, Closure, ClosureDisposition, CriterionBinding,
    Decision, DecisionChoice, DecisionInputs, DecisionJustification, FanoutReceiptSyncPoint,
    Harness, HarnessEnforcement, HarnessIdempotency, HarnessReceipt, HarnessReceiptSchema,
    HarnessRevision, HarnessSandbox, HarnessSeal, HarnessState, Intent, ReceiptIssuer,
    ReceiptIssuerType, ReceiptVerificationSummary, Reference, ReferenceType, SealCriterion,
    SignatureAlgorithm, SuccessCriterion,
};
use runx_receipts::{
    ReceiptProofContext, ReceiptSignature, SignatureVerificationFailure, SignatureVerifier,
    canonical_receipt_body_digest, validate_harness_receipt_proof, validate_receipt_tree,
};

use crate::adapter::SkillOutput;
use crate::{RuntimeError, StepRun};

pub fn step_receipt(
    graph_name: &str,
    step_id: &str,
    attempt: u32,
    output: &SkillOutput,
    created_at: &str,
) -> Result<HarnessReceipt, RuntimeError> {
    let disposition = disposition(output);
    let act = observation_act(step_id, output, created_at, disposition.clone());
    let seal = seal(
        disposition,
        format!("{step_id}_closed"),
        format!("step {step_id} completed"),
        created_at,
        Vec::new(),
    );
    let mut receipt = HarnessReceipt {
        schema: HarnessReceiptSchema::V1,
        id: step_receipt_id(graph_name, step_id, attempt),
        created_at: created_at.to_owned(),
        issuer: local_issuer(),
        signature: placeholder_signature(),
        harness: harness(
            graph_name,
            step_id,
            HarnessState::Sealed,
            vec![act],
            Vec::new(),
            seal.clone(),
        ),
        seal,
        sync_points: Vec::new(),
        metadata: None,
    };
    seal_receipt(&mut receipt)?;
    Ok(receipt)
}

pub fn graph_receipt(
    graph_name: &str,
    steps: &[StepRun],
    sync_points: Vec<FanoutReceiptSyncPoint>,
    created_at: &str,
) -> Result<HarnessReceipt, RuntimeError> {
    let child_refs = steps
        .iter()
        .map(|step| reference(ReferenceType::HarnessReceipt, &step.receipt.id))
        .collect::<Vec<_>>();
    let seal = seal(
        ClosureDisposition::Closed,
        "graph_closed".to_owned(),
        format!("graph {graph_name} completed"),
        created_at,
        Vec::new(),
    );
    let mut receipt = HarnessReceipt {
        schema: HarnessReceiptSchema::V1,
        id: format!("hrn_rcpt_{graph_name}"),
        created_at: created_at.to_owned(),
        issuer: local_issuer(),
        signature: placeholder_signature(),
        harness: harness(
            graph_name,
            "graph",
            HarnessState::Sealed,
            Vec::new(),
            child_refs,
            seal.clone(),
        ),
        seal,
        sync_points,
        metadata: None,
    };
    seal_receipt(&mut receipt)?;
    let children = steps
        .iter()
        .map(|step| step.receipt.clone())
        .collect::<Vec<_>>();
    validate_receipt_tree(&receipt, &children).map_err(receipt_error)?;
    Ok(receipt)
}

fn step_receipt_id(graph_name: &str, step_id: &str, attempt: u32) -> String {
    if attempt <= 1 {
        format!("hrn_rcpt_{graph_name}_{step_id}")
    } else {
        format!("hrn_rcpt_{graph_name}_{step_id}_attempt_{attempt}")
    }
}

fn harness(
    graph_name: &str,
    node_id: &str,
    state: HarnessState,
    acts: Vec<Act>,
    child_refs: Vec<Reference>,
    seal: HarnessSeal,
) -> Harness {
    Harness {
        schema: None,
        harness_id: format!("hrn_{graph_name}_{node_id}"),
        parent_harness_ref: None,
        state,
        host_ref: reference(ReferenceType::Host, "cli"),
        harness_ref: reference(ReferenceType::Harness, &format!("{graph_name}_{node_id}")),
        authority: authority(),
        enforcement: enforcement(),
        idempotency: idempotency(graph_name, node_id),
        revision: HarnessRevision {
            sequence: 1,
            previous_ref: None,
        },
        signal_refs: Vec::new(),
        decisions: decision(node_id),
        acts,
        child_harness_receipt_refs: child_refs,
        artifact_refs: Vec::new(),
        seal: Some(seal),
    }
}

fn observation_act(
    step_id: &str,
    output: &SkillOutput,
    performed_at: &str,
    disposition: ClosureDisposition,
) -> Act {
    Act {
        schema: None,
        act_id: format!("act_{step_id}"),
        form: ActForm::Observation,
        intent: Intent {
            purpose: format!("Run graph step {step_id}"),
            legitimacy: "Runtime graph execution was admitted by the local harness".to_owned(),
            success_criteria: vec![SuccessCriterion {
                criterion_id: "process_exit".to_owned(),
                statement: "cli-tool exits successfully".to_owned(),
                required: true,
            }],
            constraints: Vec::new(),
            derived_from: Vec::new(),
        },
        summary: format!("Executed graph step {step_id}"),
        closure: Closure {
            disposition,
            reason_code: "process_exit".to_owned(),
            summary: output_summary(output),
            closed_at: performed_at.to_owned(),
        },
        criterion_bindings: vec![CriterionBinding {
            criterion_id: "process_exit".to_owned(),
            status: if output.succeeded() {
                runx_contracts::CriterionStatus::Verified
            } else {
                runx_contracts::CriterionStatus::Failed
            },
            evidence_refs: Vec::new(),
            verification_refs: Vec::new(),
            summary: Some(output_summary(output)),
        }],
        source_refs: Vec::new(),
        target_refs: Vec::new(),
        surface_refs: Vec::new(),
        artifact_refs: Vec::new(),
        verification_refs: Vec::new(),
        harness_refs: Vec::new(),
        revision: None,
        verification: None,
        performed_at: performed_at.to_owned(),
    }
}

fn decision(node_id: &str) -> Vec<Decision> {
    vec![Decision {
        decision_id: format!("dec_{node_id}"),
        choice: DecisionChoice::Open,
        inputs: DecisionInputs::default(),
        proposed_intent: Intent {
            purpose: format!("Open runtime harness node {node_id}"),
            legitimacy: "Local graph execution requested this harness node".to_owned(),
            success_criteria: Vec::new(),
            constraints: Vec::new(),
            derived_from: Vec::new(),
        },
        selected_act_id: None,
        selected_harness_ref: None,
        justification: DecisionJustification {
            summary: "runtime graph planner selected this node".to_owned(),
            evidence_refs: Vec::new(),
        },
        closure: None,
        artifact_refs: Vec::new(),
    }]
}

fn seal(
    disposition: ClosureDisposition,
    reason_code: String,
    summary: String,
    closed_at: &str,
    criteria: Vec<SealCriterion>,
) -> HarnessSeal {
    HarnessSeal {
        disposition,
        reason_code,
        summary,
        closed_at: closed_at.to_owned(),
        last_observed_at: closed_at.to_owned(),
        canonicalization: "runx.harness-receipt.c14n.v1".to_owned(),
        digest: "sha256:runtime-skeleton".to_owned(),
        criteria,
        verification_summary: Some(ReceiptVerificationSummary {
            signature_valid: true,
            hash_commitments_valid: true,
            authority_attenuation_valid: true,
            criteria_bound: true,
            redaction_valid: true,
            external_attestations_present: false,
        }),
        redaction_refs: Vec::new(),
        artifact_refs: Vec::new(),
        hash_commitments: Vec::new(),
    }
}

fn authority() -> Authority {
    Authority {
        schema: None,
        actor_ref: reference(ReferenceType::Principal, "local_runtime"),
        authority_proof_refs: Vec::new(),
        grant_refs: Vec::new(),
        scope_refs: Vec::new(),
        policy_refs: Vec::new(),
        terms: Vec::new(),
        attenuation: AuthorityAttenuation {
            parent_authority_ref: None,
            subset_proof: None,
        },
        mandate_ref: None,
    }
}

fn enforcement() -> HarnessEnforcement {
    HarnessEnforcement {
        harness_ref: None,
        version: "runtime-skeleton".to_owned(),
        enforcement_profile_hash: "sha256:runtime-skeleton-enforcement".to_owned(),
        enforcer_ref: None,
        sandbox: HarnessSandbox {
            profile: "process-boundary".to_owned(),
            cwd_policy: "skill-directory".to_owned(),
            network: "declared-by-skill".to_owned(),
            filesystem: "declared-by-skill".to_owned(),
        },
        redaction_refs: Vec::new(),
        stdout_hash: None,
        stderr_hash: None,
        setup_receipt_refs: Vec::new(),
        teardown_receipt_refs: Vec::new(),
    }
}

fn idempotency(graph_name: &str, node_id: &str) -> HarnessIdempotency {
    HarnessIdempotency {
        intent_key: format!("sha256:{graph_name}-{node_id}-intent"),
        trigger_fingerprint: format!("sha256:{graph_name}-{node_id}-trigger"),
        content_hash: format!("sha256:{graph_name}-{node_id}-content"),
    }
}

fn disposition(output: &SkillOutput) -> ClosureDisposition {
    if output.succeeded() {
        ClosureDisposition::Closed
    } else {
        ClosureDisposition::Failed
    }
}

fn output_summary(output: &SkillOutput) -> String {
    if output.succeeded() {
        "cli-tool exited successfully".to_owned()
    } else {
        format!("cli-tool failed with exit code {:?}", output.exit_code)
    }
}

fn reference(reference_type: ReferenceType, id: &str) -> Reference {
    Reference {
        uri: format!("runx:{}:{id}", reference_type_name(&reference_type)),
        reference_type,
        provider: None,
        locator: None,
        label: None,
        observed_at: None,
    }
}

fn reference_type_name(reference_type: &ReferenceType) -> &'static str {
    match reference_type {
        ReferenceType::HarnessReceipt => "harness_receipt",
        ReferenceType::Harness => "harness",
        ReferenceType::Host => "host",
        ReferenceType::Principal => "principal",
        _ => "reference",
    }
}

fn local_issuer() -> ReceiptIssuer {
    ReceiptIssuer {
        issuer_type: ReceiptIssuerType::Local,
        kid: "runtime-skeleton".to_owned(),
        public_key_sha256: "sha256:runtime-skeleton-public".to_owned(),
    }
}

fn placeholder_signature() -> ReceiptSignature {
    ReceiptSignature {
        alg: SignatureAlgorithm::Ed25519,
        value: "sig:pending".to_owned(),
    }
}

fn seal_receipt(receipt: &mut HarnessReceipt) -> Result<(), RuntimeError> {
    let digest =
        canonical_receipt_body_digest(receipt).map_err(|error| RuntimeError::ReceiptInvalid {
            message: error.to_string(),
        })?;
    receipt.seal.digest = digest.clone();
    if let Some(seal) = receipt.harness.seal.as_mut() {
        seal.digest = digest.clone();
    }
    receipt.signature.value = format!("sig:{digest}");

    let verifier = LocalHarnessSignatureVerifier;
    validate_harness_receipt_proof(receipt, &proof_context(&verifier)).map_err(receipt_error)
}

pub(crate) fn proof_context(verifier: &LocalHarnessSignatureVerifier) -> ReceiptProofContext<'_> {
    ReceiptProofContext {
        signature_verifier: Some(verifier),
        authority_verified: true,
        external_attestations_verified: true,
        verified_redaction_refs: std::collections::BTreeSet::new(),
        verified_hash_commitments: std::collections::BTreeSet::new(),
    }
}

pub(crate) struct LocalHarnessSignatureVerifier;

impl SignatureVerifier for LocalHarnessSignatureVerifier {
    fn verify(
        &self,
        _issuer: &ReceiptIssuer,
        signature: &ReceiptSignature,
        body_digest: &str,
    ) -> Result<(), SignatureVerificationFailure> {
        if signature.value == format!("sig:{body_digest}") {
            Ok(())
        } else {
            Err(SignatureVerificationFailure::SignatureMismatch)
        }
    }
}

fn receipt_error(verification: runx_receipts::ReceiptVerification) -> RuntimeError {
    RuntimeError::ReceiptInvalid {
        message: format!("{:?}", verification.findings),
    }
}
