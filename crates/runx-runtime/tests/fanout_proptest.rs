use proptest::prelude::*;
use runx_contracts::{JsonNumber, JsonObject, JsonValue};
use runx_core::state_machine::{
    FanoutBranchFailurePolicy, FanoutBranchResult, FanoutConflictGate, FanoutGate,
    FanoutGateAction, FanoutGroupPolicy, FanoutSyncDecision, FanoutSyncOutcome, FanoutSyncStrategy,
    FanoutThresholdGate, GraphStepStatus,
};

proptest! {
    #[test]
    fn generated_fanout_counts_match_reference_policy(
        branch_count in 1usize..8,
        success_count in 0usize..8,
        min_success in 1usize..8,
        strategy_index in 0usize..3,
        halt_on_failure in any::<bool>(),
    ) {
        let success_count = success_count.min(branch_count);
        let min_success = min_success.min(branch_count);
        let policy = FanoutGroupPolicy {
            group_id: "generated".to_owned(),
            strategy: strategy(strategy_index),
            min_success: Some(u32::try_from(min_success).unwrap_or(u32::MAX)),
            on_branch_failure: if halt_on_failure {
                FanoutBranchFailurePolicy::Halt
            } else {
                FanoutBranchFailurePolicy::Continue
            },
            threshold_gates: None,
            conflict_gates: None,
        };
        let results = branch_results(branch_count, success_count);
        let decision = runx_core::state_machine::evaluate_fanout_sync(&policy, &results, None);
        let expected = expected_count_decision(&policy, branch_count, success_count, min_success);

        prop_assert_eq!(decision, expected);
    }
}

#[test]
fn threshold_gate_decision_matches_reference_policy() {
    let policy = FanoutGroupPolicy {
        group_id: "advisors".to_owned(),
        strategy: FanoutSyncStrategy::All,
        min_success: None,
        on_branch_failure: FanoutBranchFailurePolicy::Continue,
        threshold_gates: Some(vec![FanoutThresholdGate {
            step: "risk".to_owned(),
            field: "risk_score".to_owned(),
            above: JsonNumber::F64(0.8),
            action: FanoutGateAction::Pause,
        }]),
        conflict_gates: None,
    };
    let results = vec![
        branch_result("market", GraphStepStatus::Succeeded, JsonObject::new()),
        branch_result(
            "risk",
            GraphStepStatus::Succeeded,
            object([("risk_score", JsonValue::Number(JsonNumber::F64(0.91)))]),
        ),
    ];

    let decision = runx_core::state_machine::evaluate_fanout_sync(&policy, &results, None);

    assert_eq!(
        decision,
        FanoutSyncDecision {
            group_id: "advisors".to_owned(),
            decision: FanoutSyncOutcome::Pause,
            strategy: FanoutSyncStrategy::All,
            rule_fired: "threshold.risk.risk_score.above".to_owned(),
            reason: "risk.risk_score=0.91 exceeded 0.8".to_owned(),
            branch_count: 2,
            success_count: 2,
            failure_count: 0,
            required_successes: 2,
            gate: Some(FanoutGate::Threshold {
                step_id: Some("risk".to_owned()),
                field: "risk_score".to_owned(),
                value: Some(JsonValue::Number(JsonNumber::F64(0.91))),
                compared_to: Some(JsonNumber::F64(0.8)),
                action: FanoutGateAction::Pause,
            }),
        }
    );
}

#[test]
fn conflict_gate_decision_matches_reference_policy() {
    let policy = FanoutGroupPolicy {
        group_id: "advisors".to_owned(),
        strategy: FanoutSyncStrategy::All,
        min_success: None,
        on_branch_failure: FanoutBranchFailurePolicy::Continue,
        threshold_gates: None,
        conflict_gates: Some(vec![FanoutConflictGate {
            field: "recommendation".to_owned(),
            steps: vec!["market".to_owned(), "risk".to_owned()],
            action: FanoutGateAction::Escalate,
        }]),
    };
    let results = vec![
        branch_result(
            "market",
            GraphStepStatus::Succeeded,
            object([("recommendation", JsonValue::String("go".to_owned()))]),
        ),
        branch_result(
            "risk",
            GraphStepStatus::Succeeded,
            object([("recommendation", JsonValue::String("stop".to_owned()))]),
        ),
    ];

    let decision = runx_core::state_machine::evaluate_fanout_sync(&policy, &results, None);

    assert_eq!(decision.decision, FanoutSyncOutcome::Escalate);
    assert_eq!(decision.rule_fired, "conflict.recommendation");
    assert_eq!(decision.branch_count, 2);
    assert_eq!(decision.success_count, 2);
    assert_eq!(decision.failure_count, 0);
    assert!(matches!(decision.gate, Some(FanoutGate::Conflict { .. })));
}

fn expected_count_decision(
    policy: &FanoutGroupPolicy,
    branch_count: usize,
    success_count: usize,
    min_success: usize,
) -> FanoutSyncDecision {
    let failure_count = branch_count - success_count;
    let required_successes = required_successes(policy, branch_count, min_success);
    if policy.on_branch_failure == FanoutBranchFailurePolicy::Halt && failure_count > 0 {
        return FanoutSyncDecision {
            group_id: policy.group_id.clone(),
            decision: FanoutSyncOutcome::Halt,
            strategy: policy.strategy.clone(),
            rule_fired: "branch_failure.halt".to_owned(),
            reason: format!(
                "{failure_count}/{branch_count} branches failed and on_branch_failure is halt"
            ),
            branch_count,
            success_count,
            failure_count,
            required_successes,
            gate: None,
        };
    }

    let decision = if success_count >= required_successes {
        FanoutSyncOutcome::Proceed
    } else {
        FanoutSyncOutcome::Halt
    };
    let rule_fired = format!("{}.min_success", strategy_name(&policy.strategy));
    FanoutSyncDecision {
        group_id: policy.group_id.clone(),
        decision,
        strategy: policy.strategy.clone(),
        rule_fired,
        reason: format!(
            "{success_count}/{branch_count} branches succeeded; required {required_successes}"
        ),
        branch_count,
        success_count,
        failure_count,
        required_successes,
        gate: None,
    }
}

fn required_successes(
    policy: &FanoutGroupPolicy,
    branch_count: usize,
    min_success: usize,
) -> usize {
    match policy.strategy {
        FanoutSyncStrategy::All => branch_count,
        FanoutSyncStrategy::Any => usize::from(branch_count > 0),
        FanoutSyncStrategy::Quorum => min_success,
    }
}

fn strategy(index: usize) -> FanoutSyncStrategy {
    match index % 3 {
        0 => FanoutSyncStrategy::All,
        1 => FanoutSyncStrategy::Any,
        _ => FanoutSyncStrategy::Quorum,
    }
}

fn strategy_name(strategy: &FanoutSyncStrategy) -> &'static str {
    match strategy {
        FanoutSyncStrategy::All => "all",
        FanoutSyncStrategy::Any => "any",
        FanoutSyncStrategy::Quorum => "quorum",
    }
}

fn branch_results(branch_count: usize, success_count: usize) -> Vec<FanoutBranchResult> {
    (0..branch_count)
        .map(|index| {
            branch_result(
                &format!("branch_{index}"),
                if index < success_count {
                    GraphStepStatus::Succeeded
                } else {
                    GraphStepStatus::Failed
                },
                JsonObject::new(),
            )
        })
        .collect()
}

fn branch_result(
    step_id: &str,
    status: GraphStepStatus,
    outputs: JsonObject,
) -> FanoutBranchResult {
    FanoutBranchResult {
        step_id: step_id.to_owned(),
        status,
        outputs: Some(outputs),
    }
}

fn object(items: impl IntoIterator<Item = (&'static str, JsonValue)>) -> JsonObject {
    items
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}
