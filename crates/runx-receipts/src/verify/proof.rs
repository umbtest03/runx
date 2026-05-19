use std::collections::BTreeSet;

use runx_contracts::{
    AuthorityAttenuation, AuthoritySubsetResult, HarnessReceipt, HashCommitment,
    ReceiptVerificationSummary, Reference, SignatureAlgorithm,
};

use crate::canonical_receipt_body_digest;

use super::{ReceiptFinding, ReceiptFindingCode, ReceiptVerification, verify_harness_receipt};

mod signature;

pub use signature::{SignatureVerificationFailure, SignatureVerifier};
use signature::{signature_failure_code, signature_failure_message, signature_failure_path};

pub fn validate_harness_receipt_proof(
    receipt: &HarnessReceipt,
    context: &ReceiptProofContext<'_>,
) -> Result<(), ReceiptVerification> {
    let verification = verify_harness_receipt_proof(receipt, context);
    if verification.valid {
        Ok(())
    } else {
        Err(verification)
    }
}

#[must_use]
pub fn verify_harness_receipt_proof(
    receipt: &HarnessReceipt,
    context: &ReceiptProofContext<'_>,
) -> ReceiptVerification {
    let mut findings = verify_harness_receipt(receipt).findings;
    let mut verifier = ProofVerifier {
        context,
        findings: Vec::new(),
    };
    verifier.check_body_digest(receipt);
    verifier.check_signature(receipt);
    verifier.check_summary(receipt);
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

impl ProofVerifier<'_> {
    fn check_body_digest(&mut self, receipt: &HarnessReceipt) {
        let Ok(body_digest) = canonical_receipt_body_digest(receipt) else {
            self.push(
                ReceiptFindingCode::SealDigestMismatch,
                "seal.digest",
                "receipt body digest could not be recomputed",
            );
            return;
        };
        if receipt.seal.digest != body_digest {
            self.push(
                ReceiptFindingCode::SealDigestMismatch,
                "seal.digest",
                "top-level seal digest must match the canonical receipt body commitment",
            );
        }
        if receipt
            .harness
            .seal
            .as_ref()
            .is_some_and(|seal| seal.digest != body_digest)
        {
            self.push(
                ReceiptFindingCode::SealDigestMismatch,
                "harness.seal.digest",
                "harness seal digest must match the canonical receipt body commitment",
            );
        }
    }

    fn check_signature(&mut self, receipt: &HarnessReceipt) {
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

    fn check_summary(&mut self, receipt: &HarnessReceipt) {
        let Some(summary) = &receipt.seal.verification_summary else {
            self.push(
                ReceiptFindingCode::VerificationSummaryInvalid,
                "seal.verification_summary",
                "strict proof verification requires a seal verification summary",
            );
            return;
        };
        self.check_signature_summary(summary);
        self.check_authority_summary(receipt, summary);
        self.check_criteria_summary(receipt, summary);
        self.check_redaction_summary(receipt, summary);
        self.check_hash_summary(receipt, summary);
        self.check_external_attestation_summary(summary);
    }

    fn check_signature_summary(&mut self, summary: &ReceiptVerificationSummary) {
        if summary.signature_valid && self.context.signature_verifier.is_none() {
            self.push(
                ReceiptFindingCode::VerificationSummaryInvalid,
                "seal.verification_summary.signature_valid",
                "signature_valid cannot be true without signature verification input",
            );
        }
    }

    fn check_authority_summary(
        &mut self,
        receipt: &HarnessReceipt,
        summary: &ReceiptVerificationSummary,
    ) {
        if !summary.authority_attenuation_valid {
            return;
        }
        if !has_verified_attenuation_shape(&receipt.harness.authority.attenuation) {
            self.push(
                ReceiptFindingCode::VerificationSummaryInvalid,
                "seal.verification_summary.authority_attenuation_valid",
                "authority_attenuation_valid cannot be true without a matching subset proof",
            );
            return;
        }
        if !self.context.authority_verified {
            self.push(
                ReceiptFindingCode::AuthorityProofMissing,
                "seal.verification_summary.authority_attenuation_valid",
                "authority_attenuation_valid cannot be true without a verified authority result",
            );
        }
    }

    fn check_criteria_summary(
        &mut self,
        receipt: &HarnessReceipt,
        summary: &ReceiptVerificationSummary,
    ) {
        if !summary.criteria_bound {
            return;
        }
        let structural = verify_harness_receipt(receipt);
        if structural.findings.iter().any(|finding| {
            matches!(
                finding.code,
                ReceiptFindingCode::SealCriterionActMissing
                    | ReceiptFindingCode::SealCriterionUnbound
            )
        }) {
            self.push(
                ReceiptFindingCode::VerificationSummaryInvalid,
                "seal.verification_summary.criteria_bound",
                "criteria_bound cannot be true while seal criteria are unbound",
            );
        }
    }

    fn check_redaction_summary(
        &mut self,
        receipt: &HarnessReceipt,
        summary: &ReceiptVerificationSummary,
    ) {
        if !summary.redaction_valid {
            return;
        }
        for (path, reference) in redaction_refs(receipt) {
            if !self
                .context
                .verified_redaction_refs
                .contains(&reference.uri)
            {
                self.push(
                    ReceiptFindingCode::RedactionProofMissing,
                    path,
                    "redaction_valid cannot be true without verified redaction material",
                );
            }
        }
    }

    fn check_hash_summary(
        &mut self,
        receipt: &HarnessReceipt,
        summary: &ReceiptVerificationSummary,
    ) {
        if !summary.hash_commitments_valid {
            return;
        }
        for (path, commitment) in hash_commitments(receipt) {
            if !self
                .context
                .verified_hash_commitments
                .contains(&commitment.value)
            {
                self.push(
                    ReceiptFindingCode::HashCommitmentProofMissing,
                    path,
                    "hash_commitments_valid cannot be true without verified commitment material",
                );
            }
        }
    }

    fn check_external_attestation_summary(&mut self, summary: &ReceiptVerificationSummary) {
        if summary.external_attestations_present && !self.context.external_attestations_verified {
            self.push(
                ReceiptFindingCode::ExternalAttestationMissing,
                "seal.verification_summary.external_attestations_present",
                "external_attestations_present cannot be true without verified attestation material",
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

fn has_verified_attenuation_shape(attenuation: &AuthorityAttenuation) -> bool {
    let (Some(parent), Some(proof)) = (
        attenuation.parent_authority_ref.as_ref(),
        attenuation.subset_proof.as_ref(),
    ) else {
        return false;
    };
    proof.parent_authority_ref == *parent && matches!(proof.result, AuthoritySubsetResult::Subset)
}

fn redaction_refs(receipt: &HarnessReceipt) -> Vec<(String, &Reference)> {
    let enforcement = receipt
        .harness
        .enforcement
        .redaction_refs
        .iter()
        .enumerate()
        .map(|(index, reference)| {
            (
                format!("harness.enforcement.redaction_refs[{index}]"),
                reference,
            )
        });
    let seal = receipt
        .seal
        .redaction_refs
        .iter()
        .enumerate()
        .map(|(index, reference)| (format!("seal.redaction_refs[{index}]"), reference));
    enforcement.chain(seal).collect()
}

fn hash_commitments(receipt: &HarnessReceipt) -> Vec<(String, &HashCommitment)> {
    let stdout = receipt
        .harness
        .enforcement
        .stdout_hash
        .as_ref()
        .map(|commitment| ("harness.enforcement.stdout_hash".to_owned(), commitment));
    let stderr = receipt
        .harness
        .enforcement
        .stderr_hash
        .as_ref()
        .map(|commitment| ("harness.enforcement.stderr_hash".to_owned(), commitment));
    let seal = receipt
        .seal
        .hash_commitments
        .iter()
        .enumerate()
        .map(|(index, commitment)| (format!("seal.hash_commitments[{index}]"), commitment));
    stdout.into_iter().chain(stderr).chain(seal).collect()
}
