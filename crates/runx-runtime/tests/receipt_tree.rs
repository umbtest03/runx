use runx_contracts::{
    FanoutReceiptDecision, FanoutReceiptStrategy, FanoutReceiptSyncPoint, HarnessReceipt,
    JsonObject,
};
use runx_receipts::{ReceiptFindingCode, ReceiptTreeConfig};
use runx_runtime::receipts::{graph_receipt, step_receipt};
use runx_runtime::{
    InvocationStatus, RuntimeReceiptResolver, SkillOutput, StepRun, validate_runtime_receipt_tree,
    verify_runtime_receipt_tree,
};

const CREATED_AT: &str = "2026-05-18T00:00:00Z";

#[test]
fn runtime_resolver_verifies_graph_receipt_with_children() -> Result<(), Box<dyn std::error::Error>>
{
    let (root, children) = graph_with_steps("tree_runtime_graph", &["plan", "apply"])?;
    let resolver = RuntimeReceiptResolver::new(children.clone());

    assert_eq!(resolver.receipts().len(), 2);
    assert!(
        runx_receipts::validate_receipt_tree_with_resolver(
            &root,
            &resolver,
            ReceiptTreeConfig::default()
        )
        .is_ok()
    );
    assert!(validate_runtime_receipt_tree(&root, children, ReceiptTreeConfig::default()).is_ok());
    Ok(())
}

#[test]
fn runtime_tree_rejects_structurally_valid_root_ref_tamper()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut root, children) = graph_with_steps("tree_runtime_exact", &["child"])?;
    root.harness.child_harness_receipt_refs[0].uri = children[0].id.clone();

    assert!(runx_receipts::verify_receipt_tree(&root, &children).valid);
    let verification = verify_runtime_receipt_tree(&root, children, ReceiptTreeConfig::default());

    assert_finding(
        &verification,
        ReceiptFindingCode::SealDigestMismatch,
        "seal.digest",
    );
    assert_finding(
        &verification,
        ReceiptFindingCode::SignatureInvalid,
        "signature.value",
    );
    Ok(())
}

#[test]
fn runtime_resolver_reports_ambiguous_scoped_receipts() -> Result<(), Box<dyn std::error::Error>> {
    let (root, mut children) = graph_with_steps("tree_runtime_ambiguous", &["child"])?;
    children.push(children[0].clone());

    let verification = verify_runtime_receipt_tree(&root, children, ReceiptTreeConfig::default());

    assert_finding(
        &verification,
        ReceiptFindingCode::DuplicateChildReceipt,
        "runtime_receipts[1].id",
    );
    assert_finding(
        &verification,
        ReceiptFindingCode::ChildReceiptAmbiguous,
        "harness.child_harness_receipt_refs[0]",
    );
    Ok(())
}

#[test]
fn runtime_fanout_receipt_tree_uses_explicit_receipts() -> Result<(), Box<dyn std::error::Error>> {
    let steps = vec![
        step_run(
            "tree_runtime_fanout",
            "market",
            Some("advisors"),
            InvocationStatus::Success,
        )?,
        step_run(
            "tree_runtime_fanout",
            "risk",
            Some("advisors"),
            InvocationStatus::Failure,
        )?,
        step_run(
            "tree_runtime_fanout",
            "synthesize",
            None,
            InvocationStatus::Success,
        )?,
    ];
    let sync_point = fanout_sync_point(&steps[..2]);
    let root = graph_receipt(
        "tree_runtime_fanout",
        &steps,
        vec![sync_point.clone()],
        CREATED_AT,
    )?;
    let children = child_receipts(&steps);

    assert_eq!(root.sync_points, vec![sync_point]);
    assert!(validate_runtime_receipt_tree(&root, children, ReceiptTreeConfig::default()).is_ok());
    Ok(())
}

#[test]
fn runtime_tree_rejects_structurally_valid_child_proof_tamper()
-> Result<(), Box<dyn std::error::Error>> {
    let (root, mut children) = graph_with_steps("tree_runtime_child_tamper", &["child"])?;
    children[0].harness.acts[0].summary = "tampered child proof body".to_owned();

    assert!(runx_receipts::verify_receipt_tree(&root, &children).valid);
    let verification = verify_runtime_receipt_tree(&root, children, ReceiptTreeConfig::default());

    assert_finding(
        &verification,
        ReceiptFindingCode::SealDigestMismatch,
        "runtime_receipts[0].seal.digest",
    );
    assert_finding(
        &verification,
        ReceiptFindingCode::SignatureInvalid,
        "runtime_receipts[0].signature.value",
    );
    Ok(())
}

fn graph_with_steps(
    graph_name: &str,
    step_ids: &[&str],
) -> Result<(HarnessReceipt, Vec<HarnessReceipt>), Box<dyn std::error::Error>> {
    let steps = step_ids
        .iter()
        .map(|step_id| step_run(graph_name, step_id, None, InvocationStatus::Success))
        .collect::<Result<Vec<_>, _>>()?;
    let root = graph_receipt(graph_name, &steps, Vec::new(), CREATED_AT)?;
    Ok((root, child_receipts(&steps)))
}

fn child_receipts(steps: &[StepRun]) -> Vec<HarnessReceipt> {
    steps.iter().map(|step| step.receipt.clone()).collect()
}

fn step_run(
    graph_name: &str,
    step_id: &str,
    fanout_group: Option<&str>,
    status: InvocationStatus,
) -> Result<StepRun, Box<dyn std::error::Error>> {
    let output = skill_output(status);
    let receipt = step_receipt(graph_name, step_id, 1, &output, CREATED_AT)?;
    Ok(StepRun {
        step_id: step_id.to_owned(),
        attempt: 1,
        skill: step_id.to_owned(),
        runner: None,
        fanout_group: fanout_group.map(str::to_owned),
        output,
        outputs: JsonObject::new(),
        receipt,
    })
}

fn skill_output(status: InvocationStatus) -> SkillOutput {
    let (stdout, stderr, exit_code) = match status {
        InvocationStatus::Success => ("ok".to_owned(), String::new(), Some(0)),
        InvocationStatus::Failure => (String::new(), "failed".to_owned(), Some(1)),
    };
    SkillOutput {
        status,
        stdout,
        stderr,
        exit_code,
        duration_ms: 1,
        metadata: JsonObject::new(),
    }
}

fn fanout_sync_point(steps: &[StepRun]) -> FanoutReceiptSyncPoint {
    FanoutReceiptSyncPoint {
        group_id: "advisors".to_owned(),
        strategy: FanoutReceiptStrategy::Quorum,
        decision: FanoutReceiptDecision::Proceed,
        rule_fired: "quorum.min_success".to_owned(),
        reason: "1/2 branches succeeded".to_owned(),
        branch_count: 2,
        success_count: 1,
        failure_count: 1,
        required_successes: 1,
        branch_receipts: child_receipts(steps)
            .into_iter()
            .map(|receipt| receipt.id)
            .collect(),
        gate: None,
    }
}

fn assert_finding(
    verification: &runx_receipts::ReceiptVerification,
    code: ReceiptFindingCode,
    path: &str,
) {
    assert!(
        verification
            .findings
            .iter()
            .any(|finding| finding.code == code && finding.path == path),
        "expected finding {code:?} at {path}; got {:?}",
        verification.findings
    );
}
