// rust-style-allow: large-file because graph planning parity keeps linear and
// fanout planning helpers together until policy parity lands.
use std::collections::{BTreeMap, BTreeSet};

use super::fanout::evaluate_fanout_sync;
use super::types::{
    FanoutBranchFailurePolicy, FanoutBranchResult, FanoutGroupPolicy, FanoutSyncDecision,
    FanoutSyncOutcome, FanoutSyncStrategy, GraphStatus, GraphStepStatus, SequentialGraphEvent,
    SequentialGraphPlan, SequentialGraphState, SequentialGraphStepDefinition,
    SequentialGraphStepState,
};

#[must_use]
pub fn create_sequential_graph_state(
    graph_id: impl Into<String>,
    steps: &[SequentialGraphStepDefinition],
) -> SequentialGraphState {
    SequentialGraphState {
        graph_id: graph_id.into(),
        status: GraphStatus::Pending,
        steps: steps
            .iter()
            .map(|step| SequentialGraphStepState {
                step_id: step.id.clone(),
                status: GraphStepStatus::Pending,
                attempts: 0,
                started_at: None,
                completed_at: None,
                receipt_id: None,
                outputs: None,
                error: None,
            })
            .collect(),
    }
}

#[must_use]
pub fn plan_sequential_graph_transition(
    state: &SequentialGraphState,
    steps: &[SequentialGraphStepDefinition],
    fanout_policies: &BTreeMap<String, FanoutGroupPolicy>,
    resolved_fanout_gate_keys: Option<&BTreeSet<String>>,
) -> SequentialGraphPlan {
    let step_index = SequentialGraphStepIndex::new(steps);
    plan_sequential_graph_transition_indexed(
        state,
        steps,
        &step_index,
        fanout_policies,
        resolved_fanout_gate_keys,
    )
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SequentialGraphStepIndex {
    positions: BTreeMap<String, usize>,
    context_positions: Vec<Vec<ContextSourcePosition>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContextSourcePosition {
    step_id: String,
    position: Option<usize>,
}

impl SequentialGraphStepIndex {
    #[must_use]
    pub fn new(steps: &[SequentialGraphStepDefinition]) -> Self {
        let positions = steps
            .iter()
            .enumerate()
            .map(|(index, step)| (step.id.clone(), index))
            .collect::<BTreeMap<_, _>>();
        let context_positions = steps
            .iter()
            .map(|step| {
                step.context_from
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .map(|step_id| ContextSourcePosition {
                        step_id: step_id.clone(),
                        position: positions.get(step_id).copied(),
                    })
                    .collect()
            })
            .collect();
        Self {
            positions,
            context_positions,
        }
    }

    fn state_for<'a>(
        &self,
        state: &'a SequentialGraphState,
        step_id: &str,
    ) -> Option<&'a SequentialGraphStepState> {
        self.positions
            .get(step_id)
            .and_then(|index| state.steps.get(*index))
            .filter(|step| step.step_id == step_id)
    }

    fn state_at<'a>(
        &self,
        state: &'a SequentialGraphState,
        position: usize,
        step_id: &str,
    ) -> Option<&'a SequentialGraphStepState> {
        state
            .steps
            .get(position)
            .filter(|step| step.step_id == step_id)
            .or_else(|| self.state_for(state, step_id))
    }
}

#[must_use]
pub fn create_sequential_graph_step_index(
    steps: &[SequentialGraphStepDefinition],
) -> SequentialGraphStepIndex {
    SequentialGraphStepIndex::new(steps)
}

#[must_use]
pub fn plan_sequential_graph_transition_indexed(
    state: &SequentialGraphState,
    steps: &[SequentialGraphStepDefinition],
    step_index: &SequentialGraphStepIndex,
    fanout_policies: &BTreeMap<String, FanoutGroupPolicy>,
    resolved_fanout_gate_keys: Option<&BTreeSet<String>>,
) -> SequentialGraphPlan {
    if let Some(running_step) = state
        .steps
        .iter()
        .find(|step| step.status == GraphStepStatus::Running)
    {
        return SequentialGraphPlan::Blocked {
            step_id: running_step.step_id.clone(),
            reason: "step is already running".to_owned(),
            sync_decision: None,
        };
    }

    let mut index = 0;
    while index < steps.len() {
        let step_definition = &steps[index];
        if let Some(group_id) = fanout_group_id(step_definition) {
            let group_steps = contiguous_fanout_group(steps, index, group_id);
            match plan_fanout_group(
                state,
                step_index,
                index,
                group_steps,
                fanout_policies.get(group_id),
                resolved_fanout_gate_keys,
            ) {
                FanoutGroupPlan::Proceed => {
                    index += group_steps.len();
                    continue;
                }
                FanoutGroupPlan::Plan(plan) => return *plan,
            }
        }

        if let Some(plan) = plan_step(state, step_index, index, step_definition) {
            return plan;
        }
        index += 1;
    }

    SequentialGraphPlan::Complete
}

#[must_use]
pub fn transition_sequential_graph(
    state: &SequentialGraphState,
    event: &SequentialGraphEvent,
) -> SequentialGraphState {
    let mut next = state.clone();
    apply_sequential_graph_event(&mut next, event);
    next
}

pub fn apply_sequential_graph_event(
    state: &mut SequentialGraphState,
    event: &SequentialGraphEvent,
) {
    match event {
        SequentialGraphEvent::StartStep { step_id, at } => {
            update_step_in_place(state, step_id, |step| start_step_in_place(step, at));
            state.status = GraphStatus::Running;
        }
        SequentialGraphEvent::StepSucceeded {
            step_id,
            at,
            receipt_id,
            admission_witness,
            outputs,
        } => update_step_in_place(state, step_id, |step| {
            succeed_step_in_place(step, at, receipt_id, admission_witness, outputs.clone())
        }),
        SequentialGraphEvent::StepFailed { step_id, at, error } => {
            update_step_in_place(state, step_id, |step| fail_step_in_place(step, at, error));
        }
        SequentialGraphEvent::Complete if is_graph_complete(state) => {
            state.status = GraphStatus::Succeeded;
        }
        SequentialGraphEvent::Complete => {}
        SequentialGraphEvent::PauseGraph { .. } => {
            state.status = GraphStatus::Paused;
        }
        SequentialGraphEvent::EscalateGraph { .. } => {
            state.status = GraphStatus::Escalated;
        }
        SequentialGraphEvent::FailGraph { .. } => {
            state.status = GraphStatus::Failed;
        }
    }
}

enum FanoutGroupPlan {
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

fn plan_step(
    state: &SequentialGraphState,
    step_index: &SequentialGraphStepIndex,
    definition_index: usize,
    step_definition: &SequentialGraphStepDefinition,
) -> Option<SequentialGraphPlan> {
    let Some(step_state) = step_index.state_at(state, definition_index, &step_definition.id) else {
        return Some(SequentialGraphPlan::Failed {
            step_id: step_definition.id.clone(),
            reason: "step state is missing".to_owned(),
            sync_decision: None,
        });
    };

    if step_state.status == GraphStepStatus::Succeeded {
        return None;
    }
    if retry_budget_exhausted(step_state, step_definition) {
        return Some(SequentialGraphPlan::Failed {
            step_id: step_definition.id.clone(),
            reason: "step failed and retry budget is exhausted".to_owned(),
            sync_decision: None,
        });
    }
    if let Some(missing_context) =
        missing_context_at(state, step_index, definition_index, step_definition)
    {
        return Some(SequentialGraphPlan::Blocked {
            step_id: step_definition.id.clone(),
            reason: format!("waiting for context from {missing_context}"),
            sync_decision: None,
        });
    }

    Some(SequentialGraphPlan::RunStep {
        step_id: step_definition.id.clone(),
        attempt: step_state.attempts + 1,
        context_from: step_definition.context_from.clone().unwrap_or_default(),
    })
}

fn plan_fanout_group(
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

fn contiguous_fanout_group<'a>(
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

fn fanout_group_id(step: &SequentialGraphStepDefinition) -> Option<&str> {
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

fn start_step_in_place(step: &mut SequentialGraphStepState, at: &str) {
    if matches!(
        step.status,
        GraphStepStatus::Running | GraphStepStatus::Succeeded
    ) {
        return;
    }
    step.status = GraphStepStatus::Running;
    step.attempts += 1;
    step.started_at = Some(at.to_owned());
    step.completed_at = None;
    step.outputs = None;
    step.error = None;
}

fn succeed_step_in_place(
    step: &mut SequentialGraphStepState,
    at: &str,
    receipt_id: &str,
    admission_witness: &super::types::StepAdmissionWitness,
    outputs: Option<runx_contracts::JsonObject>,
) {
    if step.status != GraphStepStatus::Running {
        return;
    }
    if !admission_witness.matches_step_receipt(&step.step_id, receipt_id) {
        return;
    }
    step.status = GraphStepStatus::Succeeded;
    step.completed_at = Some(at.to_owned());
    step.receipt_id = Some(receipt_id.to_owned());
    step.outputs = outputs;
    step.error = None;
}

fn fail_step_in_place(step: &mut SequentialGraphStepState, at: &str, error: &str) {
    if step.status != GraphStepStatus::Running {
        return;
    }
    step.status = GraphStepStatus::Failed;
    step.completed_at = Some(at.to_owned());
    step.outputs = None;
    step.error = Some(error.to_owned());
}

fn update_step_in_place(
    state: &mut SequentialGraphState,
    step_id: &str,
    update: impl FnOnce(&mut SequentialGraphStepState),
) {
    if let Some(step) = state.steps.iter_mut().find(|step| step.step_id == step_id) {
        update(step);
    }
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

fn retry_budget_exhausted(
    step_state: &SequentialGraphStepState,
    step_definition: &SequentialGraphStepDefinition,
) -> bool {
    step_state.status == GraphStepStatus::Failed
        && step_state.attempts
            >= step_definition
                .retry
                .as_ref()
                .map_or(1, |retry| retry.max_attempts)
}

fn missing_context(
    state: &SequentialGraphState,
    step_index: &SequentialGraphStepIndex,
    step_definition: &SequentialGraphStepDefinition,
) -> Option<String> {
    step_definition
        .context_from
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .find(|step_id| {
            step_index
                .state_for(state, step_id)
                .is_none_or(|step| step.status != GraphStepStatus::Succeeded)
        })
        .cloned()
}

fn missing_context_at(
    state: &SequentialGraphState,
    step_index: &SequentialGraphStepIndex,
    definition_index: usize,
    step_definition: &SequentialGraphStepDefinition,
) -> Option<String> {
    let Some(context_sources) = step_index.context_positions.get(definition_index) else {
        return missing_context(state, step_index, step_definition);
    };
    if context_sources.is_empty() {
        return None;
    }
    context_sources
        .iter()
        .find(|source| {
            source
                .position
                .and_then(|position| state.steps.get(position))
                .filter(|step| step.step_id == source.step_id)
                .is_none_or(|step| step.status != GraphStepStatus::Succeeded)
        })
        .map(|source| source.step_id.clone())
}

fn is_graph_complete(state: &SequentialGraphState) -> bool {
    state.steps.iter().all(|step| {
        !matches!(
            step.status,
            GraphStepStatus::Pending | GraphStepStatus::Running
        )
    })
}
