use std::collections::BTreeSet;

use runx_contracts::{
    Act, ActForm, AuthorityAttenuation, Harness, HarnessReceipt, HarnessSeal, HarnessState,
    HashCommitment, Reference, ReferenceType,
};

mod finding;
mod projection;
mod proof;

pub use finding::{ReceiptError, ReceiptFinding, ReceiptFindingCode, ReceiptVerification};
pub use projection::{
    ReceiptProofFindingSummary, ReceiptProofStatus, ReceiptProofStatusKind, receipt_proof_status,
};
pub use proof::{
    ReceiptProofContext, SignatureVerificationFailure, SignatureVerifier,
    validate_harness_receipt_proof, verify_harness_receipt_proof,
};

pub fn validate_harness_receipt(receipt: &HarnessReceipt) -> Result<(), ReceiptVerification> {
    let verification = verify_harness_receipt(receipt);
    if verification.valid {
        Ok(())
    } else {
        Err(verification)
    }
}

#[must_use]
pub fn verify_harness_receipt(receipt: &HarnessReceipt) -> ReceiptVerification {
    let mut verifier = Verifier::default();
    verifier.check_non_empty("id", &receipt.id);
    verifier.check_non_empty("created_at", &receipt.created_at);
    verifier.check_non_empty("issuer.kid", &receipt.issuer.kid);
    verifier.check_sha256_prefix(
        "issuer.public_key_sha256",
        &receipt.issuer.public_key_sha256,
    );
    verifier.check_non_empty("signature.value", &receipt.signature.value);
    verifier.check_harness("harness", &receipt.harness);
    verifier.check_receipt_seal_matches(receipt);
    verifier.finish()
}

pub fn validate_harness(harness: &Harness) -> Result<(), ReceiptVerification> {
    let verification = verify_harness(harness);
    if verification.valid {
        Ok(())
    } else {
        Err(verification)
    }
}

#[must_use]
pub fn verify_harness(harness: &Harness) -> ReceiptVerification {
    let mut verifier = Verifier::default();
    verifier.check_harness("harness", harness);
    verifier.finish()
}

#[derive(Default)]
struct Verifier {
    findings: Vec<ReceiptFinding>,
}

impl Verifier {
    fn finish(self) -> ReceiptVerification {
        ReceiptVerification::from_findings(self.findings)
    }

    fn check_harness(&mut self, path: &str, harness: &Harness) {
        self.check_non_empty(&format!("{path}.harness_id"), &harness.harness_id);
        self.check_terminal_seal_state(path, harness);
        self.check_authority_attenuation(path, &harness.authority.attenuation);
        self.check_hash_prefixes(path, harness);
        self.check_child_receipt_refs(path, &harness.child_harness_receipt_refs);
        self.check_acts(path, &harness.acts);
        self.check_decisions(path, harness);
        if let Some(seal) = &harness.seal {
            self.check_seal_criteria(path, harness, seal);
        }
    }

    fn check_receipt_seal_matches(&mut self, receipt: &HarnessReceipt) {
        if receipt.harness.seal.as_ref() != Some(&receipt.seal) {
            self.push(
                ReceiptFindingCode::ReceiptSealMismatch,
                "seal",
                "top-level receipt seal must match harness.seal",
            );
        }
    }

    fn check_terminal_seal_state(&mut self, path: &str, harness: &Harness) {
        let terminal = is_terminal_state(&harness.state);
        if terminal && harness.seal.is_none() {
            self.push(
                ReceiptFindingCode::TerminalHarnessMissingSeal,
                format!("{path}.seal"),
                "terminal harness states require a seal",
            );
        }
        if !terminal && harness.seal.is_some() {
            self.push(
                ReceiptFindingCode::NonTerminalHarnessHasSeal,
                format!("{path}.seal"),
                "nonterminal harness states must not carry a seal",
            );
        }
    }

    fn check_authority_attenuation(&mut self, path: &str, attenuation: &AuthorityAttenuation) {
        match (&attenuation.parent_authority_ref, &attenuation.subset_proof) {
            (Some(parent), Some(proof)) if proof.parent_authority_ref == *parent => {}
            (Some(_), Some(_)) => self.push(
                ReceiptFindingCode::AuthorityAttenuationInvalid,
                format!("{path}.authority.attenuation.subset_proof.parent_authority_ref"),
                "subset proof must cite the same parent authority ref",
            ),
            (Some(_), None) => self.push(
                ReceiptFindingCode::AuthorityAttenuationInvalid,
                format!("{path}.authority.attenuation.subset_proof"),
                "parent authority refs require a subset proof",
            ),
            (None, Some(_)) => self.push(
                ReceiptFindingCode::AuthorityAttenuationInvalid,
                format!("{path}.authority.attenuation.subset_proof"),
                "root authority must not carry a subset proof",
            ),
            (None, None) => {}
        }
    }

    fn check_hash_prefixes(&mut self, path: &str, harness: &Harness) {
        self.check_sha256_prefix(
            &format!("{path}.enforcement.enforcement_profile_hash"),
            &harness.enforcement.enforcement_profile_hash,
        );
        self.check_sha256_prefix(
            &format!("{path}.idempotency.intent_key"),
            &harness.idempotency.intent_key,
        );
        self.check_sha256_prefix(
            &format!("{path}.idempotency.trigger_fingerprint"),
            &harness.idempotency.trigger_fingerprint,
        );
        self.check_sha256_prefix(
            &format!("{path}.idempotency.content_hash"),
            &harness.idempotency.content_hash,
        );
        if let Some(stdout_hash) = &harness.enforcement.stdout_hash {
            self.check_hash_commitment(&format!("{path}.enforcement.stdout_hash"), stdout_hash);
        }
        if let Some(stderr_hash) = &harness.enforcement.stderr_hash {
            self.check_hash_commitment(&format!("{path}.enforcement.stderr_hash"), stderr_hash);
        }
        if let Some(seal) = &harness.seal {
            self.check_sha256_prefix(&format!("{path}.seal.digest"), &seal.digest);
            for (index, commitment) in seal.hash_commitments.iter().enumerate() {
                self.check_hash_commitment(
                    &format!("{path}.seal.hash_commitments[{index}]"),
                    commitment,
                );
            }
        }
    }

    fn check_child_receipt_refs(&mut self, path: &str, refs: &[Reference]) {
        for (index, reference) in refs.iter().enumerate() {
            if reference.reference_type != ReferenceType::HarnessReceipt {
                self.push(
                    ReceiptFindingCode::ChildReceiptRefInvalid,
                    format!("{path}.child_harness_receipt_refs[{index}].type"),
                    "child harness receipt refs must use type harness_receipt",
                );
            }
        }
    }

    fn check_acts(&mut self, path: &str, acts: &[Act]) {
        for (index, act) in acts.iter().enumerate() {
            let act_path = format!("{path}.acts[{index}]");
            match act.form {
                ActForm::Revision => self.check_revision_act(&act_path, act),
                ActForm::Verification => self.check_verification_act(&act_path, act),
                ActForm::Reply | ActForm::Review | ActForm::Observation => {
                    self.check_plain_act(&act_path, act);
                }
            }
        }
    }

    fn check_decisions(&mut self, path: &str, harness: &Harness) {
        let act_ids = act_ids(&harness.acts);
        for (index, decision) in harness.decisions.iter().enumerate() {
            if let Some(act_id) = &decision.selected_act_id {
                if !act_ids.contains(act_id) {
                    self.push(
                        ReceiptFindingCode::DecisionSelectedActMissing,
                        format!("{path}.decisions[{index}].selected_act_id"),
                        "selected act id must refer to an act in the same harness",
                    );
                }
            }
        }
    }

    fn check_seal_criteria(&mut self, path: &str, harness: &Harness, seal: &HarnessSeal) {
        let act_ids = act_ids(&harness.acts);
        for (index, criterion) in seal.criteria.iter().enumerate() {
            let criterion_path = format!("{path}.seal.criteria[{index}]");
            if let Some(act_id) = &criterion.act_id {
                if !act_ids.contains(act_id) {
                    self.push(
                        ReceiptFindingCode::SealCriterionActMissing,
                        format!("{criterion_path}.act_id"),
                        "seal criterion act id must refer to an act in the same harness",
                    );
                    continue;
                }
                if !act_binds_criterion(&harness.acts, act_id, &criterion.criterion_id) {
                    self.push(
                        ReceiptFindingCode::SealCriterionUnbound,
                        format!("{criterion_path}.criterion_id"),
                        "seal criterion must bind to the cited act intent or criterion bindings",
                    );
                }
            }
        }
    }

    fn check_revision_act(&mut self, path: &str, act: &Act) {
        if act.revision.is_none() || act.verification.is_some() {
            self.push(
                ReceiptFindingCode::ActFormDetailsInvalid,
                path,
                "revision acts require revision details and must not carry verification details",
            );
        }
    }

    fn check_verification_act(&mut self, path: &str, act: &Act) {
        if act.verification.is_none() || act.revision.is_some() {
            self.push(
                ReceiptFindingCode::ActFormDetailsInvalid,
                path,
                "verification acts require verification details and must not carry revision details",
            );
        }
    }

    fn check_plain_act(&mut self, path: &str, act: &Act) {
        if act.revision.is_some() || act.verification.is_some() {
            self.push(
                ReceiptFindingCode::ActFormDetailsInvalid,
                path,
                "reply, review, and observation acts must not carry revision or verification details",
            );
        }
    }

    fn check_hash_commitment(&mut self, path: &str, commitment: &HashCommitment) {
        self.check_sha256_prefix(&format!("{path}.value"), &commitment.value);
    }

    fn check_sha256_prefix(&mut self, path: &str, value: &str) {
        if !value.starts_with("sha256:") {
            self.push(
                ReceiptFindingCode::HashCommitmentInvalid,
                path,
                "hash values must use the sha256: prefix",
            );
        }
    }

    fn check_non_empty(&mut self, path: &str, value: &str) {
        if value.is_empty() {
            self.push(
                ReceiptFindingCode::EmptyEnvelopeField,
                path,
                "receipt envelope fields must not be empty",
            );
        }
    }

    fn push(
        &mut self,
        code: ReceiptFindingCode,
        path: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.findings.push(ReceiptFinding {
            code,
            path: path.into(),
            message: message.into(),
        });
    }
}

fn is_terminal_state(state: &HarnessState) -> bool {
    matches!(
        state,
        HarnessState::Sealed
            | HarnessState::Killed
            | HarnessState::TimedOut
            | HarnessState::Failed
            | HarnessState::Superseded
    )
}

fn act_ids(acts: &[Act]) -> BTreeSet<String> {
    acts.iter().map(|act| act.act_id.clone()).collect()
}

fn act_binds_criterion(acts: &[Act], act_id: &str, criterion_id: &str) -> bool {
    acts.iter()
        .find(|act| act.act_id == act_id)
        .is_some_and(|act| criterion_declared_or_bound(act, criterion_id))
}

fn criterion_declared_or_bound(act: &Act, criterion_id: &str) -> bool {
    act.intent
        .success_criteria
        .iter()
        .any(|criterion| criterion.criterion_id == criterion_id)
        || act
            .criterion_bindings
            .iter()
            .any(|binding| binding.criterion_id == criterion_id)
        || act.verification.as_ref().is_some_and(|details| {
            details
                .criterion_ids
                .iter()
                .any(|criterion| criterion == criterion_id)
        })
}
