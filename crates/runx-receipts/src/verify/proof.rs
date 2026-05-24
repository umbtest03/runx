use std::collections::BTreeSet;

use runx_contracts::{
    AuthorityAttenuation, AuthoritySubsetResult, Receipt, ReceiptCommitment,
    ReceiptVerificationSummary, Reference, SignatureAlgorithm,
};

use crate::{canonical_receipt_body_digest, content_addressed_receipt_id};

use super::{ReceiptFinding, ReceiptFindingCode, ReceiptVerification, verify_receipt};

mod signature;

pub use signature::{SignatureVerificationFailure, SignatureVerifier};
use signature::{signature_failure_code, signature_failure_message, signature_failure_path};

pub fn validate_receipt_proof(
    receipt: &Receipt,
    context: &ReceiptProofContext<'_>,
) -> Result<(), ReceiptVerification> {
    let verification = verify_receipt_proof(receipt, context);
    if verification.valid {
        Ok(())
    } else {
        Err(verification)
    }
}

/// Read-time proof verification. Computes signature/digest/attenuation validity
/// on top of the structural checks (which include the inline `selected_act_id`
/// integrity property against `acts[]`).
#[must_use]
pub fn verify_receipt_proof(
    receipt: &Receipt,
    context: &ReceiptProofContext<'_>,
) -> ReceiptVerification {
    let mut findings = verify_receipt(receipt).findings;
    let mut verifier = ProofVerifier {
        context,
        findings: Vec::new(),
    };
    verifier.check_body_digest(receipt);
    verifier.check_signature(receipt);
    findings.extend(verifier.findings);
    ReceiptVerification::from_findings(findings)
}

#[derive(Default)]
pub struct ReceiptProofContext<'a> {
    pub signature_verifier: Option<&'a dyn SignatureVerifier>,
    pub authority_verified: bool,
    pub external_attestations_verified: bool,
    pub verified_redaction_refs: BTreeSet<String>,
    pub verified_hash_commitments: BTreeSet<String>,
}

struct ProofVerifier<'a> {
    context: &'a ReceiptProofContext<'a>,
    findings: Vec<ReceiptFinding>,
}

/// Whether `receipt.id` equals its content address `hash(canonical_body)` under
/// `runx.receipt.c14n.v1`. The runtime asserts this at seal time and the
/// trainable projection verifies it on read; it is intentionally NOT a
/// per-node structural check so synthetic tree fixtures stay address-agnostic.
#[must_use]
pub fn receipt_id_is_content_addressed(receipt: &Receipt) -> bool {
    content_addressed_receipt_id(receipt).is_ok_and(|content_id| receipt.id == content_id)
}

impl ProofVerifier<'_> {
    fn check_body_digest(&mut self, receipt: &Receipt) {
        let Ok(body_digest) = canonical_receipt_body_digest(receipt) else {
            self.push(
                ReceiptFindingCode::SealDigestMismatch,
                "digest",
                "receipt body digest could not be recomputed",
            );
            return;
        };
        if receipt.digest != body_digest {
            self.push(
                ReceiptFindingCode::SealDigestMismatch,
                "digest",
                "receipt digest must match the canonical receipt body commitment",
            );
        }
    }

    fn check_signature(&mut self, receipt: &Receipt) {
        let Ok(body_digest) = canonical_receipt_body_digest(receipt) else {
            return;
        };
        match self.context.signature_verifier {
            Some(verifier) => {
                if receipt.signature.alg != SignatureAlgorithm::Ed25519 {
                    self.push(
                        ReceiptFindingCode::SignatureUnsupportedAlgorithm,
                        "signature.alg",
                        "unsupported receipt signature algorithm",
                    );
                    return;
                }
                if let Err(error) =
                    verifier.verify(&receipt.issuer, &receipt.signature, &body_digest)
                {
                    self.push(
                        signature_failure_code(&error),
                        signature_failure_path(&error),
                        signature_failure_message(&error),
                    );
                }
            }
            None => self.push(
                ReceiptFindingCode::SignatureVerifierMissing,
                "signature",
                "strict receipt proof verification requires an injected signature verifier",
            ),
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

/// Compute the read-time verification summary projection (never part of the
/// signed body).
#[must_use]
pub fn compute_verification_summary(
    receipt: &Receipt,
    context: &ReceiptProofContext<'_>,
) -> ReceiptVerificationSummary {
    let signature_valid = context.signature_verifier.is_some_and(|verifier| {
        canonical_receipt_body_digest(receipt).is_ok_and(|body_digest| {
            receipt.signature.alg == SignatureAlgorithm::Ed25519
                && verifier
                    .verify(&receipt.issuer, &receipt.signature, &body_digest)
                    .is_ok()
        })
    });
    let authority_attenuation_valid = context.authority_verified
        && has_verified_attenuation_shape(&receipt.authority.attenuation);
    let criteria_bound = verify_receipt(receipt).findings.iter().all(|finding| {
        !matches!(
            finding.code,
            ReceiptFindingCode::SealCriterionActMissing | ReceiptFindingCode::SealCriterionUnbound
        )
    });
    let redaction_valid = redaction_refs(receipt)
        .iter()
        .all(|reference| context.verified_redaction_refs.contains(&reference.uri));
    let hash_commitments_valid = hash_commitments(receipt).iter().all(|commitment| {
        context
            .verified_hash_commitments
            .contains(&commitment.value)
    });
    ReceiptVerificationSummary {
        signature_valid,
        content_address_valid: receipt_id_is_content_addressed(receipt),
        hash_commitments_valid,
        authority_attenuation_valid,
        criteria_bound,
        redaction_valid,
        external_attestations_present: context.external_attestations_verified,
    }
}

fn has_verified_attenuation_shape(attenuation: &AuthorityAttenuation) -> bool {
    let (Some(parent), Some(proof)) = (
        attenuation.parent_authority_ref.as_ref(),
        attenuation.subset_proof.as_ref(),
    ) else {
        return false;
    };
    proof.parent_authority_ref == *parent && matches!(proof.result, AuthoritySubsetResult::Subset)
}

fn redaction_refs(receipt: &Receipt) -> Vec<&Reference> {
    receipt
        .authority
        .enforcement
        .redaction_refs
        .iter()
        .collect()
}

fn hash_commitments(receipt: &Receipt) -> Vec<&ReceiptCommitment> {
    receipt.subject.commitments.iter().collect()
}
