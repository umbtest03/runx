#![cfg(feature = "cli-tool")]

use std::path::Path;

use runx_contracts::{
    FanoutReceiptDecision, FanoutReceiptStrategy, FanoutReceiptSyncPoint, JsonObject, JsonValue,
};
use runx_core::state_machine::{GraphStatus, GraphStepStatus};
use runx_receipts::validate_receipt_tree;
use runx_runtime::{Runtime, RuntimeError, RuntimeOptions, run_graph_file};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FanoutFixture {
    all_success: ExpectedRun,
    quorum_continue: ExpectedRun,
    threshold_pause: ExpectedPause,
    generated: GeneratedFixture,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeneratedFixture {
    partial_failure: ExpectedRun,
    retry: ExpectedRetry,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedRun {
    graph: String,
    graph_path: Option<String>,
    status: String,
    steps: Vec<ExpectedStep>,
    sync_points: Vec<FanoutReceiptSyncPoint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedStep {
    id: String,
    status: String,
    attempt: Option<u32>,
    fanout_group: Option<String>,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedPause {
    graph: String,
    status: String,
    step_id: String,
    sync_point: FanoutReceiptSyncPoint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedRetry {
    graph: String,
    graph_path: String,
    status: String,
    branch_count: usize,
    retry_step_id: String,
    retry_attempts: u32,
    checkpoint_steps: Vec<ExpectedStep>,
    sync_point: FanoutReceiptSyncPoint,
}

#[test]
fn fanout_all_success_runs_group_then_synthesizes() -> Result<(), Box<dyn std::error::Error>> {
    let expected = fixture()?.all_success;
    let run = run_graph_file(Path::new("../../fixtures/graphs/fanout/all.yaml"))?;

    assert_eq!(run.graph.name, expected.graph);
    assert_eq!(graph_status(&run.state.status), expected.status);
    assert_steps(&run, &expected.steps);
    assert_step_state(&run, "market", GraphStepStatus::Succeeded)?;
    assert_step_state(&run, "risk", GraphStepStatus::Succeeded)?;
    assert_step_state(&run, "finance", GraphStepStatus::Succeeded)?;
    assert_output(
        &run,
        "finance",
        "budget",
        JsonValue::String("approved".to_owned()),
    )?;
    assert_sync_points(&run, &expected.sync_points);
    assert_receipt_tree(&run);
    Ok(())
}

#[test]
fn fanout_quorum_continue_tolerates_failed_branch() -> Result<(), Box<dyn std::error::Error>> {
    let expected = fixture()?.quorum_continue;
    let run = run_graph_file(Path::new("../../fixtures/graphs/fanout/graph.yaml"))?;

    assert_eq!(run.graph.name, expected.graph);
    assert_eq!(graph_status(&run.state.status), expected.status);
    assert_steps(&run, &expected.steps);
    assert_step_state(&run, "market", GraphStepStatus::Succeeded)?;
    assert_step_state(&run, "risk", GraphStepStatus::Succeeded)?;
    assert_step_state(&run, "finance", GraphStepStatus::Failed)?;
    assert_step_state(&run, "synthesize", GraphStepStatus::Succeeded)?;
    assert_sync_points(&run, &expected.sync_points);
    assert_receipt_tree(&run);
    Ok(())
}

#[test]
fn fanout_threshold_pause_blocks_followup() -> Result<(), Box<dyn std::error::Error>> {
    let expected = fixture()?.threshold_pause;
    let error = match run_graph_file(Path::new("../../fixtures/graphs/fanout/threshold.yaml")) {
        Ok(_) => return Err("threshold fanout should pause".into()),
        Err(error) => error,
    };

    let RuntimeError::GraphPaused {
        step_id,
        reason,
        sync_decision,
    } = error
    else {
        return Err(format!("expected GraphPaused, got {error:?}").into());
    };

    assert_eq!(expected.graph, "fanout-threshold");
    assert_eq!(expected.status, "paused");
    assert_eq!(step_id, expected.step_id);
    assert_eq!(reason, expected.sync_point.reason);
    assert_eq!(
        sync_point_without_receipts(&sync_decision),
        expected_without_receipts(&expected.sync_point)
    );

    let runtime = Runtime::new(
        runx_runtime::adapters::cli_tool::CliToolAdapter,
        RuntimeOptions::default(),
    );
    let checkpoint = runtime
        .run_graph_file_until_steps(Path::new("../../fixtures/graphs/fanout/threshold.yaml"), 2)?;
    assert_eq!(
        checkpoint
            .steps
            .iter()
            .map(|step| step.receipt.id.clone())
            .collect::<Vec<_>>(),
        expected.sync_point.branch_receipts
    );
    Ok(())
}

#[test]
fn generated_n_branch_partial_failure_uses_sync_point_oracle()
-> Result<(), Box<dyn std::error::Error>> {
    let expected = fixture()?.generated.partial_failure;
    let graph_path = expected
        .graph_path
        .as_deref()
        .ok_or("generated partial-failure fixture is missing graphPath")?;
    let run = run_graph_file(Path::new(graph_path))?;

    assert_eq!(run.graph.name, expected.graph);
    assert_eq!(graph_status(&run.state.status), expected.status);
    assert_steps(&run, &expected.steps);
    assert_sync_points(&run, &expected.sync_points);
    assert_receipt_tree(&run);
    Ok(())
}

#[test]
fn generated_retry_records_attempts_before_halt() -> Result<(), Box<dyn std::error::Error>> {
    let expected = fixture()?.generated.retry;
    let runtime = Runtime::new(
        runx_runtime::adapters::cli_tool::CliToolAdapter,
        RuntimeOptions::default(),
    );
    let checkpoint = runtime.run_graph_file_until_steps(
        Path::new(&expected.graph_path),
        expected.checkpoint_steps.len(),
    )?;

    assert_eq!(checkpoint.graph_name, expected.graph);
    assert_steps_in_checkpoint(&checkpoint, &expected.checkpoint_steps);
    assert_eq!(
        checkpoint
            .state
            .steps
            .iter()
            .find(|step| step.step_id == expected.retry_step_id)
            .map(|step| step.attempts),
        Some(expected.retry_attempts)
    );

    let error = match run_graph_file(Path::new(&expected.graph_path)) {
        Ok(_) => return Err("retry fanout should halt after exhausting retry budget".into()),
        Err(error) => error,
    };
    let RuntimeError::GraphPlanningFailed { reason, .. } = error else {
        return Err(format!("expected GraphPlanningFailed, got {error:?}").into());
    };
    assert_eq!(expected.status, "failed");
    assert_eq!(expected.branch_count, expected.sync_point.branch_count);
    assert_eq!(reason, expected.sync_point.reason);
    Ok(())
}

#[test]
fn fanout_runtime_error_branch_records_failure_and_continues()
-> Result<(), Box<dyn std::error::Error>> {
    let run = run_graph_file(Path::new(
        "../../fixtures/runtime/fanout/generated/fanout-generated-missing-skill.yaml",
    ))?;

    assert_eq!(run.graph.name, "fanout-generated-missing-skill");
    assert_eq!(run.state.status, GraphStatus::Succeeded);
    assert_step_state(&run, "market", GraphStepStatus::Succeeded)?;
    assert_step_state(&run, "missing", GraphStepStatus::Failed)?;
    assert_step_state(&run, "risk", GraphStepStatus::Succeeded)?;
    assert_step_state(&run, "synthesize", GraphStepStatus::Succeeded)?;
    assert!(
        run.steps
            .iter()
            .find(|step| step.step_id == "missing")
            .is_some_and(|step| step.output.stderr.contains("skill file is missing"))
    );
    assert_eq!(run.sync_points.len(), 1);
    assert_eq!(run.sync_points[0].decision, FanoutReceiptDecision::Proceed);
    assert_eq!(run.sync_points[0].success_count, 2);
    assert_eq!(run.sync_points[0].failure_count, 1);
    assert_receipt_tree(&run);
    Ok(())
}

fn fixture() -> Result<FanoutFixture, serde_json::Error> {
    serde_json::from_str(include_str!(
        "../../../fixtures/runtime/fanout/expected.json"
    ))
}

fn assert_steps(run: &runx_runtime::GraphRun, expected: &[ExpectedStep]) {
    assert_eq!(run.steps.len(), expected.len());
    for (actual, expected) in run.steps.iter().zip(expected) {
        assert_eq!(actual.step_id, expected.id);
        if let Some(attempt) = expected.attempt {
            assert_eq!(actual.attempt, attempt);
        }
        if let Some(fanout_group) = &expected.fanout_group {
            assert_eq!(actual.fanout_group.as_deref(), Some(fanout_group.as_str()));
        }
        assert_eq!(output_status(actual), expected.status);
        assert_eq!(actual.output.stdout, expected.stdout);
        assert_eq!(actual.output.stderr, expected.stderr);
    }
}

fn assert_steps_in_checkpoint(run: &runx_runtime::GraphCheckpoint, expected: &[ExpectedStep]) {
    assert_eq!(run.steps.len(), expected.len());
    for (actual, expected) in run.steps.iter().zip(expected) {
        assert_eq!(actual.step_id, expected.id);
        if let Some(attempt) = expected.attempt {
            assert_eq!(actual.attempt, attempt);
        }
        if let Some(fanout_group) = &expected.fanout_group {
            assert_eq!(actual.fanout_group.as_deref(), Some(fanout_group.as_str()));
        }
        assert_eq!(output_status(actual), expected.status);
        assert_eq!(actual.output.stdout, expected.stdout);
        assert_eq!(actual.output.stderr, expected.stderr);
    }
}

fn assert_step_state(
    run: &runx_runtime::GraphRun,
    step_id: &str,
    status: GraphStepStatus,
) -> Result<(), String> {
    let step = run
        .state
        .steps
        .iter()
        .find(|candidate| candidate.step_id == step_id)
        .ok_or_else(|| format!("missing step state {step_id}"))?;
    assert_eq!(step.status, status);
    Ok(())
}

fn assert_output(
    run: &runx_runtime::GraphRun,
    step_id: &str,
    key: &str,
    expected: JsonValue,
) -> Result<(), String> {
    let step = run
        .steps
        .iter()
        .find(|candidate| candidate.step_id == step_id)
        .ok_or_else(|| format!("missing step run {step_id}"))?;
    assert_eq!(step.outputs.get(key), Some(&expected));
    Ok(())
}

fn assert_receipt_tree(run: &runx_runtime::GraphRun) {
    let children = run
        .steps
        .iter()
        .map(|step| step.receipt.clone())
        .collect::<Vec<_>>();
    assert!(validate_receipt_tree(&run.receipt, &children).is_ok());
}

fn assert_sync_points(run: &runx_runtime::GraphRun, expected: &[FanoutReceiptSyncPoint]) {
    assert_eq!(run.sync_points, expected);
    assert_eq!(run.receipt.sync_points, expected);
}

fn output_status(step: &runx_runtime::StepRun) -> &'static str {
    if step.output.succeeded() {
        "success"
    } else {
        "failure"
    }
}

fn graph_status(status: &GraphStatus) -> &'static str {
    match status {
        GraphStatus::Pending => "pending",
        GraphStatus::Running => "running",
        GraphStatus::Succeeded => "succeeded",
        GraphStatus::Failed => "failed",
        GraphStatus::Paused => "paused",
        GraphStatus::Escalated => "escalated",
    }
}

fn sync_point_without_receipts(
    decision: &runx_core::state_machine::FanoutSyncDecision,
) -> FanoutReceiptSyncPoint {
    FanoutReceiptSyncPoint {
        group_id: decision.group_id.clone(),
        strategy: match decision.strategy {
            runx_core::state_machine::FanoutSyncStrategy::All => FanoutReceiptStrategy::All,
            runx_core::state_machine::FanoutSyncStrategy::Any => FanoutReceiptStrategy::Any,
            runx_core::state_machine::FanoutSyncStrategy::Quorum => FanoutReceiptStrategy::Quorum,
        },
        decision: match decision.decision {
            runx_core::state_machine::FanoutSyncOutcome::Proceed => FanoutReceiptDecision::Proceed,
            runx_core::state_machine::FanoutSyncOutcome::Halt => FanoutReceiptDecision::Halt,
            runx_core::state_machine::FanoutSyncOutcome::Pause => FanoutReceiptDecision::Pause,
            runx_core::state_machine::FanoutSyncOutcome::Escalate => {
                FanoutReceiptDecision::Escalate
            }
        },
        rule_fired: decision.rule_fired.clone(),
        reason: decision.reason.clone(),
        branch_count: decision.branch_count,
        success_count: decision.success_count,
        failure_count: decision.failure_count,
        required_successes: decision.required_successes,
        branch_receipts: Vec::new(),
        gate: decision_gate(&decision.gate),
    }
}

fn expected_without_receipts(sync_point: &FanoutReceiptSyncPoint) -> FanoutReceiptSyncPoint {
    FanoutReceiptSyncPoint {
        branch_receipts: Vec::new(),
        ..sync_point.clone()
    }
}

fn decision_gate(gate: &Option<runx_core::state_machine::FanoutGate>) -> Option<JsonObject> {
    let value = serde_json::to_value(gate.as_ref()?).ok()?;
    let runx_contracts::JsonValue::Object(object) = serde_json::from_value(value).ok()? else {
        return None;
    };
    Some(object)
}
