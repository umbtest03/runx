// rust-style-allow: large-file because fanout parity keeps threshold,
// conflict, and quorum rules adjacent to mirror the TypeScript oracle.
use std::collections::{BTreeMap, BTreeSet};

use runx_contracts::{JsonObject, JsonValue};

use super::types::{
    FanoutBranchFailurePolicy, FanoutBranchResult, FanoutConflictGate, FanoutGate,
    FanoutGroupPolicy, FanoutSyncDecision, FanoutSyncOutcome, FanoutSyncStrategy,
    FanoutThresholdGate, GraphStepStatus,
};

#[must_use]
pub fn fanout_sync_decision_key(group_id: &str, rule_fired: &str) -> String {
    format!("{group_id}:{rule_fired}")
}

#[must_use]
pub fn evaluate_fanout_sync(
    policy: &FanoutGroupPolicy,
    results: &[FanoutBranchResult],
    resolved_gate_keys: Option<&BTreeSet<String>>,
) -> FanoutSyncDecision {
    let counts = Counts::from_results(policy, results);

    if policy.on_branch_failure == FanoutBranchFailurePolicy::Halt && counts.failure_count > 0 {
        return branch_failure_decision(policy, counts);
    }

    if let Some(decision) = threshold_decision(policy, results, resolved_gate_keys, counts) {
        return decision;
    }

    if let Some(decision) = conflict_decision(policy, results, resolved_gate_keys, counts) {
        return decision;
    }

    if counts.success_count >= counts.required_successes {
        return quorum_decision(policy, FanoutSyncOutcome::Proceed, counts);
    }

    quorum_decision(policy, FanoutSyncOutcome::Halt, counts)
}

fn branch_failure_decision(policy: &FanoutGroupPolicy, counts: Counts) -> FanoutSyncDecision {
    sync_decision(
        policy,
        FanoutSyncOutcome::Halt,
        "quorum",
        counts,
        DecisionDetails::new(
            "branch_failure.halt",
            format!(
                "{}/{} branches failed and on_branch_failure is halt",
                counts.failure_count, counts.branch_count
            ),
        ),
    )
}

fn quorum_decision(
    policy: &FanoutGroupPolicy,
    outcome: FanoutSyncOutcome,
    counts: Counts,
) -> FanoutSyncDecision {
    sync_decision(
        policy,
        outcome,
        "quorum",
        counts,
        DecisionDetails::new(
            format!("{}.min_success", strategy_name(&policy.strategy)),
            format!(
                "{}/{} branches succeeded; required {}",
                counts.success_count, counts.branch_count, counts.required_successes
            ),
        ),
    )
}

#[derive(Clone, Copy)]
struct Counts {
    branch_count: usize,
    success_count: usize,
    failure_count: usize,
    required_successes: usize,
}

impl Counts {
    fn from_results(policy: &FanoutGroupPolicy, results: &[FanoutBranchResult]) -> Self {
        let branch_count = results.len();
        let success_count = results
            .iter()
            .filter(|result| result.status == GraphStepStatus::Succeeded)
            .count();
        let failure_count = results
            .iter()
            .filter(|result| result.status == GraphStepStatus::Failed)
            .count();
        let required_successes = required_success_count(policy, branch_count);

        Self::new(
            branch_count,
            success_count,
            failure_count,
            required_successes,
        )
    }

    fn new(
        branch_count: usize,
        success_count: usize,
        failure_count: usize,
        required_successes: usize,
    ) -> Self {
        Self {
            branch_count,
            success_count,
            failure_count,
            required_successes,
        }
    }
}

struct DecisionDetails {
    rule_fired: String,
    reason: String,
    gate: Option<FanoutGate>,
}

impl DecisionDetails {
    fn new(rule_fired: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            rule_fired: rule_fired.into(),
            reason: reason.into(),
            gate: None,
        }
    }

    fn with_gate(mut self, gate: FanoutGate) -> Self {
        self.gate = Some(gate);
        self
    }
}

fn threshold_decision(
    policy: &FanoutGroupPolicy,
    results: &[FanoutBranchResult],
    resolved_gate_keys: Option<&BTreeSet<String>>,
    counts: Counts,
) -> Option<FanoutSyncDecision> {
    for gate in policy.threshold_gates.as_deref().unwrap_or(&[]) {
        let Some(above) = gate.above.as_f64() else {
            continue;
        };
        let Some(result) = results
            .iter()
            .find(|candidate| candidate.step_id == gate.step)
        else {
            continue;
        };
        if result.status != GraphStepStatus::Succeeded {
            continue;
        }
        let Some(value) = resolve_structured_field(result.outputs.as_ref(), &gate.field) else {
            return Some(threshold_missing_decision(policy, gate, counts));
        };
        let Some(number) = json_value_as_f64(value) else {
            return Some(threshold_non_numeric_decision(
                policy,
                gate,
                value.clone(),
                counts,
            ));
        };
        if number > above {
            let decision = threshold_above_decision(policy, gate, value.clone(), number, counts);
            if !is_resolved(&decision, resolved_gate_keys) {
                return Some(decision);
            }
        }
    }
    None
}

fn threshold_missing_decision(
    policy: &FanoutGroupPolicy,
    gate: &FanoutThresholdGate,
    counts: Counts,
) -> FanoutSyncDecision {
    sync_decision(
        policy,
        FanoutSyncOutcome::Halt,
        "threshold",
        counts,
        DecisionDetails::new(
            format!("threshold.{}.{}.missing", gate.step, gate.field),
            format!(
                "threshold field {}.{} was not produced",
                gate.step, gate.field
            ),
        )
        .with_gate(FanoutGate::Threshold {
            step_id: Some(gate.step.clone()),
            field: gate.field.clone(),
            value: None,
            compared_to: None,
            action: gate.action.clone(),
        }),
    )
}

fn threshold_non_numeric_decision(
    policy: &FanoutGroupPolicy,
    gate: &FanoutThresholdGate,
    value: JsonValue,
    counts: Counts,
) -> FanoutSyncDecision {
    sync_decision(
        policy,
        FanoutSyncOutcome::Halt,
        "threshold",
        counts,
        DecisionDetails::new(
            format!("threshold.{}.{}.non_numeric", gate.step, gate.field),
            format!(
                "threshold field {}.{} must be numeric",
                gate.step, gate.field
            ),
        )
        .with_gate(FanoutGate::Threshold {
            step_id: Some(gate.step.clone()),
            field: gate.field.clone(),
            value: Some(value),
            compared_to: None,
            action: gate.action.clone(),
        }),
    )
}

fn threshold_above_decision(
    policy: &FanoutGroupPolicy,
    gate: &FanoutThresholdGate,
    value: JsonValue,
    number: f64,
    counts: Counts,
) -> FanoutSyncDecision {
    sync_decision(
        policy,
        gate.action.clone().into(),
        "threshold",
        counts,
        DecisionDetails::new(
            format!("threshold.{}.{}.above", gate.step, gate.field),
            format!(
                "{}.{}={number} exceeded {}",
                gate.step, gate.field, gate.above
            ),
        )
        .with_gate(FanoutGate::Threshold {
            step_id: Some(gate.step.clone()),
            field: gate.field.clone(),
            value: Some(value),
            compared_to: Some(gate.above.clone()),
            action: gate.action.clone(),
        }),
    )
}

fn conflict_decision(
    policy: &FanoutGroupPolicy,
    results: &[FanoutBranchResult],
    resolved_gate_keys: Option<&BTreeSet<String>>,
    counts: Counts,
) -> Option<FanoutSyncDecision> {
    for gate in policy.conflict_gates.as_deref().unwrap_or(&[]) {
        let candidates = conflict_candidates(results, gate);
        let values = conflict_values(candidates.iter().copied(), gate);
        let distinct: BTreeSet<String> = candidates
            .iter()
            .map(|result| {
                resolve_structured_field(result.outputs.as_ref(), &gate.field)
                    .map_or_else(|| "undefined".to_owned(), stable_value)
            })
            .collect();
        if distinct.len() > 1 {
            let decision = sync_decision(
                policy,
                gate.action.clone().into(),
                "conflict",
                counts,
                DecisionDetails::new(
                    format!("conflict.{}", gate.field),
                    format!(
                        "fanout branches disagreed on structured field {}",
                        gate.field
                    ),
                )
                .with_gate(FanoutGate::Conflict {
                    field: gate.field.clone(),
                    values: Some(values),
                    action: gate.action.clone(),
                }),
            );
            if !is_resolved(&decision, resolved_gate_keys) {
                return Some(decision);
            }
        }
    }
    None
}

fn conflict_candidates<'a>(
    results: &'a [FanoutBranchResult],
    gate: &FanoutConflictGate,
) -> Vec<&'a FanoutBranchResult> {
    results
        .iter()
        .filter(|result| {
            result.status == GraphStepStatus::Succeeded
                && (gate.steps.is_empty() || gate.steps.contains(&result.step_id))
        })
        .collect()
}

fn conflict_values<'a>(
    results: impl Iterator<Item = &'a FanoutBranchResult>,
    gate: &FanoutConflictGate,
) -> BTreeMap<String, JsonValue> {
    results
        .filter_map(|result| {
            resolve_structured_field(result.outputs.as_ref(), &gate.field)
                .cloned()
                .map(|value| (result.step_id.clone(), value))
        })
        .collect()
}

fn is_resolved(
    decision: &FanoutSyncDecision,
    resolved_gate_keys: Option<&BTreeSet<String>>,
) -> bool {
    resolved_gate_keys.is_some_and(|keys| {
        keys.contains(&fanout_sync_decision_key(
            &decision.group_id,
            &decision.rule_fired,
        ))
    })
}

fn sync_decision(
    policy: &FanoutGroupPolicy,
    decision: FanoutSyncOutcome,
    _type: &str,
    counts: Counts,
    details: DecisionDetails,
) -> FanoutSyncDecision {
    FanoutSyncDecision {
        group_id: policy.group_id.clone(),
        decision,
        strategy: policy.strategy.clone(),
        rule_fired: details.rule_fired,
        reason: details.reason,
        branch_count: counts.branch_count,
        success_count: counts.success_count,
        failure_count: counts.failure_count,
        required_successes: counts.required_successes,
        gate: details.gate,
    }
}

fn required_success_count(policy: &FanoutGroupPolicy, branch_count: usize) -> usize {
    match policy.strategy {
        FanoutSyncStrategy::All => branch_count,
        FanoutSyncStrategy::Any => 1,
        FanoutSyncStrategy::Quorum => policy
            .min_success
            .map_or(branch_count, |value| value as usize),
    }
}

fn resolve_structured_field<'a>(
    outputs: Option<&'a JsonObject>,
    field_path: &str,
) -> Option<&'a JsonValue> {
    let mut current = outputs?;
    let mut parts = field_path.split('.').peekable();
    while let Some(part) = parts.next() {
        let value = current.get(part)?;
        if parts.peek().is_none() {
            return Some(value);
        }
        let JsonValue::Object(next) = value else {
            return None;
        };
        current = next;
    }
    None
}

fn json_value_as_f64(value: &JsonValue) -> Option<f64> {
    match value {
        JsonValue::Number(number) => number.as_f64(),
        _ => None,
    }
}

fn stable_value(value: &JsonValue) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "undefined".to_owned())
}

fn strategy_name(strategy: &FanoutSyncStrategy) -> &'static str {
    match strategy {
        FanoutSyncStrategy::All => "all",
        FanoutSyncStrategy::Any => "any",
        FanoutSyncStrategy::Quorum => "quorum",
    }
}
