use std::collections::BTreeSet;

use runx_contracts::{HarnessReceipt, Reference, ReferenceType};

use crate::{
    ReceiptFinding, ReceiptFindingCode, ReceiptVerification, validate_harness_receipt,
    verify_harness_receipt,
};

pub fn validate_receipt_tree(
    root: &HarnessReceipt,
    children: &[HarnessReceipt],
) -> Result<(), ReceiptVerification> {
    let verification = verify_receipt_tree(root, children);
    if verification.valid {
        Ok(())
    } else {
        Err(verification)
    }
}

#[must_use]
pub fn verify_receipt_tree(
    root: &HarnessReceipt,
    children: &[HarnessReceipt],
) -> ReceiptVerification {
    let mut findings = verify_harness_receipt(root).findings;
    findings.extend(duplicate_child_findings(children));
    findings.extend(child_receipt_findings(children));
    findings.extend(missing_child_findings("harness", root, children));
    for (index, child) in children.iter().enumerate() {
        findings.extend(missing_child_findings(
            &format!("children[{index}].harness"),
            child,
            children,
        ));
    }
    ReceiptVerification::from_findings(findings)
}

fn duplicate_child_findings(children: &[HarnessReceipt]) -> Vec<ReceiptFinding> {
    let mut seen = BTreeSet::new();
    children
        .iter()
        .enumerate()
        .filter_map(|(index, receipt)| {
            if seen.insert(receipt.id.as_str()) {
                None
            } else {
                Some(ReceiptFinding {
                    code: ReceiptFindingCode::DuplicateChildReceipt,
                    path: format!("children[{index}].id"),
                    message: "child receipt ids must be unique".to_owned(),
                })
            }
        })
        .collect()
}

fn child_receipt_findings(children: &[HarnessReceipt]) -> Vec<ReceiptFinding> {
    children
        .iter()
        .enumerate()
        .flat_map(|(index, receipt)| {
            validate_harness_receipt(receipt)
                .err()
                .map_or_else(Vec::new, |verification| {
                    verification
                        .findings
                        .into_iter()
                        .map(|finding| child_finding(index, finding))
                        .collect()
                })
        })
        .collect()
}

fn missing_child_findings(
    path: &str,
    root: &HarnessReceipt,
    children: &[HarnessReceipt],
) -> Vec<ReceiptFinding> {
    root.harness
        .child_harness_receipt_refs
        .iter()
        .enumerate()
        .filter(|(_, reference)| reference.reference_type == ReferenceType::HarnessReceipt)
        .filter(|(_, reference)| {
            !children
                .iter()
                .any(|child| receipt_ref_matches(reference, child))
        })
        .map(|(index, _)| ReceiptFinding {
            code: ReceiptFindingCode::ChildReceiptMissing,
            path: format!("{path}.child_harness_receipt_refs[{index}]"),
            message: "child harness receipt ref must resolve to a supplied child receipt"
                .to_owned(),
        })
        .collect()
}

fn child_finding(index: usize, finding: ReceiptFinding) -> ReceiptFinding {
    ReceiptFinding {
        path: format!("children[{index}].{}", finding.path),
        ..finding
    }
}

fn receipt_ref_matches(reference: &Reference, receipt: &HarnessReceipt) -> bool {
    if reference.reference_type != ReferenceType::HarnessReceipt {
        return false;
    }
    let suffix = format!(":{}", receipt.id);
    reference.uri == receipt.id || reference.uri.ends_with(&suffix)
}
