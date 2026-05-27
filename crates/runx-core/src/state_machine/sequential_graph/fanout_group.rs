use std::collections::{BTreeMap, BTreeSet};

use super::super::fanout::evaluate_fanout_sync;
use super::super::types::{
    FanoutBranchFailurePolicy, FanoutBranchResult, FanoutGroupPolicy, FanoutSyncDecision,
    FanoutSyncOutcome, FanoutSyncStrategy, GraphStepStatus, SequentialGraphPlan,
    SequentialGraphState, SequentialGraphStepDefinition,
};
use super::index::SequentialGraphStepIndex;
use super::step_readiness::{missing_context_at, retry_budget_exhausted};

pub(super) enum FanoutGroupPlan {
    Proceed,
    Plan(Box<SequentialGraphPlan>),
}

enum FanoutCandidatePlan {
    Plan(Box<SequentialGraphPlan>),
    ProceedToSync,
}

enum NonProceedFanoutDecision {
    Halt(FanoutSyncDecision),
    Pause(FanoutSyncDecision),
    Escalate(FanoutSyncDecision),
}

pub(super) fn plan_fanout_group(
    state: &SequentialGraphState,
    step_index: &SequentialGraphStepIndex,
    start_index: usize,
    group_steps: &[SequentialGraphStepDefinition],
    policy: Option<&FanoutGroupPolicy>,
    resolved_fanout_gate_keys: Option<&BTreeSet<String>>,
) -> FanoutGroupPlan {
    let Some(first_step) = group_steps.first() else {
        return FanoutGroupPlan::Plan(Box::new(SequentialGraphPlan::Failed {
            step_id: "unknown".to_owned(),
            reason: "fanout group is empty".to_owned(),
            sync_decision: None,
        }));
    };
    let Some(group_id) = fanout_group_id(first_step) else {
        return FanoutGroupPlan::Plan(Box::new(SequentialGraphPlan::Failed {
            step_id: first_step.id.clone(),
            reason: "fanout group is empty".to_owned(),
            sync_decision: None,
        }));
    };

    match plan_fanout_candidates(state, step_index, start_index, group_steps, group_id) {
        FanoutCandidatePlan::Plan(plan) => return FanoutGroupPlan::Plan(plan),
        FanoutCandidatePlan::ProceedToSync => {}
    }

    let fanout_policy = policy
        .cloned()
        .unwrap_or_else(|| default_fanout_policy(group_id));
    let results = fanout_results(
        state,
        step_index,
        start_index,
        group_steps,
        fanout_policy_requires_outputs(&fanout_policy),
    );
    let decision = evaluate_fanout_sync(&fanout_policy, &results, resolved_fanout_gate_keys);
    let Some(non_proceed_decision) = non_proceed_fanout_decision(decision) else {
        return FanoutGroupPlan::Proceed;
    };

    FanoutGroupPlan::Plan(Box::new(sync_decision_plan(
        first_step,
        non_proceed_decision,
    )))
}

fn plan_fanout_candidates(
    state: &SequentialGraphState,
    step_index: &SequentialGraphStepIndex,
    start_index: usize,
    group_steps: &[SequentialGraphStepDefinition],
    group_id: &str,
) -> FanoutCandidatePlan {
    let mut step_ids = Vec::new();
    let mut attempts = BTreeMap::new();
    let mut context_from = BTreeMap::new();

    for (offset, step_definition) in group_steps.iter().enumerate() {
        let definition_index = start_index + offset;
        let Some(step_state) = step_index.state_at(state, definition_index, &step_definition.id)
        else {
            return FanoutCandidatePlan::Plan(Box::new(SequentialGraphPlan::Failed {
                step_id: step_definition.id.clone(),
                reason: "step state is missing".to_owned(),
                sync_decision: None,
            }));
        };
        if step_state.status == GraphStepStatus::Succeeded
            || retry_budget_exhausted(step_state, step_definition)
        {
            continue;
        }
        if let Some(missing_context) =
            missing_context_at(state, step_index, definition_index, step_definition)
        {
            return FanoutCandidatePlan::Plan(Box::new(SequentialGraphPlan::Blocked {
                step_id: step_definition.id.clone(),
                reason: format!("waiting for context from {missing_context}"),
                sync_decision: None,
            }));
        }
        step_ids.push(step_definition.id.clone());
        attempts.insert(step_definition.id.clone(), step_state.attempts + 1);
        context_from.insert(
            step_definition.id.clone(),
            step_definition.context_from.clone().unwrap_or_default(),
        );
    }

    if step_ids.is_empty() {
        FanoutCandidatePlan::ProceedToSync
    } else {
        FanoutCandidatePlan::Plan(Box::new(SequentialGraphPlan::RunFanout {
            group_id: group_id.to_owned(),
            step_ids,
            attempts,
            context_from,
        }))
    }
}

fn sync_decision_plan(
    first_step: &SequentialGraphStepDefinition,
    decision: NonProceedFanoutDecision,
) -> SequentialGraphPlan {
    match decision {
        NonProceedFanoutDecision::Halt(decision) => SequentialGraphPlan::Failed {
            step_id: first_step.id.clone(),
            reason: decision.reason.clone(),
            sync_decision: Some(decision),
        },
        NonProceedFanoutDecision::Pause(decision) => SequentialGraphPlan::Paused {
            step_id: first_step.id.clone(),
            reason: decision.reason.clone(),
            sync_decision: decision,
        },
        NonProceedFanoutDecision::Escalate(decision) => SequentialGraphPlan::Escalated {
            step_id: first_step.id.clone(),
            reason: decision.reason.clone(),
            sync_decision: decision,
        },
    }
}

fn non_proceed_fanout_decision(decision: FanoutSyncDecision) -> Option<NonProceedFanoutDecision> {
    match decision.decision {
        FanoutSyncOutcome::Proceed => None,
        FanoutSyncOutcome::Halt => Some(NonProceedFanoutDecision::Halt(decision)),
        FanoutSyncOutcome::Pause => Some(NonProceedFanoutDecision::Pause(decision)),
        FanoutSyncOutcome::Escalate => Some(NonProceedFanoutDecision::Escalate(decision)),
    }
}

pub(super) fn contiguous_fanout_group<'a>(
    steps: &'a [SequentialGraphStepDefinition],
    start_index: usize,
    group_id: &str,
) -> &'a [SequentialGraphStepDefinition] {
    let mut end_index = start_index;
    while end_index < steps.len() && fanout_group_id(&steps[end_index]) == Some(group_id) {
        end_index += 1;
    }
    &steps[start_index..end_index]
}

pub(super) fn fanout_group_id(step: &SequentialGraphStepDefinition) -> Option<&str> {
    step.fanout_group
        .as_deref()
        .filter(|group_id| !group_id.is_empty())
}

fn fanout_results(
    state: &SequentialGraphState,
    step_index: &SequentialGraphStepIndex,
    start_index: usize,
    group_steps: &[SequentialGraphStepDefinition],
    include_outputs: bool,
) -> Vec<FanoutBranchResult> {
    group_steps
        .iter()
        .enumerate()
        .map(|(offset, step)| {
            let step_state = step_index.state_at(state, start_index + offset, &step.id);
            FanoutBranchResult {
                step_id: step.id.clone(),
                status: step_state.map_or(GraphStepStatus::Failed, |state| state.status.clone()),
                outputs: if include_outputs {
                    step_state.and_then(|state| state.outputs.clone())
                } else {
                    None
                },
            }
        })
        .collect()
}

fn fanout_policy_requires_outputs(policy: &FanoutGroupPolicy) -> bool {
    policy
        .threshold_gates
        .as_ref()
        .is_some_and(|gates| !gates.is_empty())
        || policy
            .conflict_gates
            .as_ref()
            .is_some_and(|gates| !gates.is_empty())
}

fn default_fanout_policy(group_id: &str) -> FanoutGroupPolicy {
    FanoutGroupPolicy {
        group_id: group_id.to_owned(),
        strategy: FanoutSyncStrategy::All,
        min_success: None,
        on_branch_failure: FanoutBranchFailurePolicy::Halt,
        threshold_gates: Some(Vec::new()),
        conflict_gates: Some(Vec::new()),
    }
}
