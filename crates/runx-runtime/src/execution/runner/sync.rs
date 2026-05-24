use runx_contracts::{
    ExecutionEvent, FanoutReceiptDecision, FanoutReceiptStrategy, FanoutReceiptSyncPoint,
    JsonObject,
};
use runx_core::state_machine::{FanoutSyncDecision, FanoutSyncOutcome, FanoutSyncStrategy};
use runx_parser::ExecutionGraph;

use super::StepRun;

pub(super) fn latest_fanout_receipt_ids(
    runs: &[StepRun],
    graph: &ExecutionGraph,
    group_id: &str,
) -> Vec<String> {
    graph
        .steps
        .iter()
        .filter(|step| step.fanout_group.as_deref() == Some(group_id))
        .filter_map(|step| {
            runs.iter()
                .rev()
                .find(|run| run.step_id == step.id)
                .map(|run| run.receipt.id.to_string())
        })
        .collect()
}

pub(super) fn fanout_sync_point(
    decision: &FanoutSyncDecision,
    branch_receipts: &[String],
) -> FanoutReceiptSyncPoint {
    FanoutReceiptSyncPoint {
        group_id: decision.group_id.clone().into(),
        strategy: receipt_strategy(&decision.strategy),
        decision: receipt_decision(&decision.decision),
        rule_fired: decision.rule_fired.clone().into(),
        reason: decision.reason.clone().into(),
        branch_count: decision.branch_count,
        success_count: decision.success_count,
        failure_count: decision.failure_count,
        required_successes: decision.required_successes,
        branch_receipts: branch_receipts.iter().cloned().map(Into::into).collect(),
        gate: decision_gate(&decision.gate),
    }
}

pub(super) fn receipt_strategy(strategy: &FanoutSyncStrategy) -> FanoutReceiptStrategy {
    match strategy {
        FanoutSyncStrategy::All => FanoutReceiptStrategy::All,
        FanoutSyncStrategy::Any => FanoutReceiptStrategy::Any,
        FanoutSyncStrategy::Quorum => FanoutReceiptStrategy::Quorum,
    }
}

pub(super) fn receipt_decision(decision: &FanoutSyncOutcome) -> FanoutReceiptDecision {
    match decision {
        FanoutSyncOutcome::Proceed => FanoutReceiptDecision::Proceed,
        FanoutSyncOutcome::Halt => FanoutReceiptDecision::Halt,
        FanoutSyncOutcome::Pause => FanoutReceiptDecision::Pause,
        FanoutSyncOutcome::Escalate => FanoutReceiptDecision::Escalate,
    }
}

pub(super) fn decision_gate(
    gate: &Option<runx_core::state_machine::FanoutGate>,
) -> Option<JsonObject> {
    let value = serde_json::to_value(gate.as_ref()?).ok()?;
    let runx_contracts::JsonValue::Object(object) = serde_json::from_value(value).ok()? else {
        return None;
    };
    Some(object)
}

pub(super) fn started_event(step_id: &str) -> ExecutionEvent {
    ExecutionEvent::StepStarted {
        message: format!("step {step_id} started"),
        data: None,
    }
}

pub(super) fn completed_event(step_id: &str) -> ExecutionEvent {
    ExecutionEvent::StepCompleted {
        message: format!("step {step_id} completed"),
        data: None,
    }
}

pub(super) fn failed_event(step_id: &str) -> ExecutionEvent {
    ExecutionEvent::Warning {
        message: format!("step {step_id} failed"),
        data: None,
    }
}
