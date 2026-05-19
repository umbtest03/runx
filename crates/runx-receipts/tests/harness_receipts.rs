use serde::Deserialize;

use runx_contracts::{
    ActForm, HarnessReceipt, HarnessState, ReceiptIssuerType, Reference, ReferenceType,
};
use runx_receipts::{
    ReceiptFindingCode, ReceiptVerification, canonical_receipt_digest, canonical_receipt_json,
    validate_harness, validate_harness_receipt, validate_receipt_tree, verify_harness_receipt,
    verify_receipt_tree,
};

const SUCCESS_RECEIPT: &str =
    include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-success.json");
const ABNORMAL_RECEIPT: &str =
    include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-abnormal.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    expected: HarnessReceipt,
}

#[test]
fn harness_spine_success_receipt_verifies_basic_invariants() -> Result<(), serde_json::Error> {
    let receipt = fixture(SUCCESS_RECEIPT)?;

    assert!(validate_harness_receipt(&receipt).is_ok());
    assert!(matches!(
        canonical_receipt_json(&receipt),
        Ok(json) if json.starts_with(r#"{"created_at":"#)
    ));
    assert!(matches!(
        canonical_receipt_digest(&receipt),
        Ok(digest) if digest.starts_with("sha256:")
    ));
    Ok(())
}

#[test]
fn harness_spine_abnormal_failed_receipt_verifies_basic_invariants() -> Result<(), serde_json::Error>
{
    let receipt = fixture(ABNORMAL_RECEIPT)?;

    assert!(validate_harness_receipt(&receipt).is_ok());
    assert_eq!(receipt.harness.state, HarnessState::Failed);
    assert!(receipt.harness.acts.is_empty());
    Ok(())
}

#[test]
fn receipt_rejects_when_top_level_seal_differs_from_harness_seal() -> Result<(), serde_json::Error>
{
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    receipt.seal.reason_code = "different".to_owned();

    let verification = verify_harness_receipt(&receipt);

    assert_finding(&verification, ReceiptFindingCode::ReceiptSealMismatch);
    Ok(())
}

#[test]
fn terminal_harness_state_requires_seal() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    receipt.harness.seal = None;

    let verification = verify_harness_receipt(&receipt);

    assert_finding(
        &verification,
        ReceiptFindingCode::TerminalHarnessMissingSeal,
    );
    assert_finding(&verification, ReceiptFindingCode::ReceiptSealMismatch);
    Ok(())
}

#[test]
fn nonterminal_harness_state_rejects_seal() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    receipt.harness.state = HarnessState::Running;

    let result = validate_harness(&receipt.harness);

    assert!(result.is_err());
    if let Err(verification) = result {
        assert_finding(&verification, ReceiptFindingCode::NonTerminalHarnessHasSeal);
    }
    Ok(())
}

#[test]
fn revision_act_requires_revision_details() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    receipt.harness.acts[0].revision = None;

    let verification = verify_harness_receipt(&receipt);

    assert_finding(&verification, ReceiptFindingCode::ActFormDetailsInvalid);
    Ok(())
}

#[test]
fn verification_act_requires_verification_details() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    let act = &mut receipt.harness.acts[0];
    act.form = ActForm::Verification;
    act.revision = None;

    let verification = verify_harness_receipt(&receipt);

    assert_finding(&verification, ReceiptFindingCode::ActFormDetailsInvalid);
    Ok(())
}

#[test]
fn decision_selected_act_id_must_exist() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    receipt.harness.decisions[0].selected_act_id = Some("missing".to_owned());

    let verification = verify_harness_receipt(&receipt);

    assert_finding(
        &verification,
        ReceiptFindingCode::DecisionSelectedActMissing,
    );
    Ok(())
}

#[test]
fn seal_criterion_act_id_must_exist() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    if let Some(seal) = receipt.harness.seal.as_mut() {
        seal.criteria[0].act_id = Some("missing".to_owned());
        receipt.seal = seal.clone();
    }

    let verification = verify_harness_receipt(&receipt);

    assert_finding(&verification, ReceiptFindingCode::SealCriterionActMissing);
    Ok(())
}

#[test]
fn seal_criterion_must_bind_to_act_criteria() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    if let Some(seal) = receipt.harness.seal.as_mut() {
        seal.criteria[0].criterion_id = "missing_criterion".to_owned();
        receipt.seal = seal.clone();
    }

    let verification = verify_harness_receipt(&receipt);

    assert_finding(&verification, ReceiptFindingCode::SealCriterionUnbound);
    Ok(())
}

#[test]
fn child_harness_refs_must_be_harness_receipt_refs() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    receipt.harness.child_harness_receipt_refs[0].reference_type = ReferenceType::Receipt;

    let verification = verify_harness_receipt(&receipt);

    assert_finding(&verification, ReceiptFindingCode::ChildReceiptRefInvalid);
    Ok(())
}

#[test]
fn receipt_tree_requires_supplied_child_receipts() -> Result<(), serde_json::Error> {
    let receipt = fixture(SUCCESS_RECEIPT)?;

    let verification = verify_receipt_tree(&receipt, &[]);

    assert_finding(&verification, ReceiptFindingCode::ChildReceiptMissing);
    Ok(())
}

#[test]
fn receipt_tree_accepts_matching_child_receipts() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(SUCCESS_RECEIPT)?;
    let mut child = fixture(ABNORMAL_RECEIPT)?;
    child.id = "hrn_rcpt_child_1".to_owned();

    assert!(validate_receipt_tree(&receipt, &[child]).is_ok());
    receipt.harness.child_harness_receipt_refs.clear();
    assert!(validate_receipt_tree(&receipt, &[]).is_ok());
    Ok(())
}

#[test]
fn receipt_tree_checks_nested_child_receipt_refs() -> Result<(), serde_json::Error> {
    let receipt = fixture(SUCCESS_RECEIPT)?;
    let mut child = fixture(ABNORMAL_RECEIPT)?;
    child.id = "hrn_rcpt_child_1".to_owned();
    child.harness.child_harness_receipt_refs.push(Reference {
        reference_type: ReferenceType::HarnessReceipt,
        uri: "runx:harness_receipt:missing_grandchild".to_owned(),
        provider: None,
        locator: None,
        label: None,
        observed_at: None,
    });

    let verification = verify_receipt_tree(&receipt, &[child]);

    assert_finding(&verification, ReceiptFindingCode::ChildReceiptMissing);
    assert!(
        verification
            .findings
            .iter()
            .any(|finding| { finding.path == "children[0].harness.child_harness_receipt_refs[0]" })
    );
    Ok(())
}

#[test]
fn receipt_tree_rejects_duplicate_child_receipt_ids() -> Result<(), serde_json::Error> {
    let receipt = fixture(SUCCESS_RECEIPT)?;
    let mut first = fixture(ABNORMAL_RECEIPT)?;
    first.id = "hrn_rcpt_child_1".to_owned();
    let second = first.clone();

    let verification = verify_receipt_tree(&receipt, &[first, second]);

    assert_finding(&verification, ReceiptFindingCode::DuplicateChildReceipt);
    Ok(())
}

#[test]
fn hash_commitments_require_sha256_prefix() -> Result<(), serde_json::Error> {
    let mut receipt = fixture(ABNORMAL_RECEIPT)?;
    if let Some(seal) = receipt.harness.seal.as_mut() {
        seal.hash_commitments[0].value = "stderr".to_owned();
        receipt.seal = seal.clone();
    }

    let verification = verify_harness_receipt(&receipt);

    assert_finding(&verification, ReceiptFindingCode::HashCommitmentInvalid);
    Ok(())
}

#[test]
fn verifier_issuer_type_matches_schema() -> Result<(), serde_json::Error> {
    let json = r#"{"type":"verifier","kid":"key_1","public_key_sha256":"sha256:public"}"#;
    let issuer: runx_contracts::ReceiptIssuer = serde_json::from_str(json)?;

    assert_eq!(issuer.issuer_type, ReceiptIssuerType::Verifier);
    Ok(())
}

fn fixture(json: &str) -> Result<HarnessReceipt, serde_json::Error> {
    serde_json::from_str::<Fixture>(json).map(|fixture| fixture.expected)
}

fn assert_finding(verification: &ReceiptVerification, code: ReceiptFindingCode) {
    assert!(
        verification
            .findings
            .iter()
            .any(|finding| finding.code == code),
        "expected finding {code:?}; got {:?}",
        verification.findings
    );
}
