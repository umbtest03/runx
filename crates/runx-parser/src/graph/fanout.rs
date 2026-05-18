use std::collections::{BTreeMap, BTreeSet};

use runx_contracts::{JsonObject, JsonValue};

use super::helpers::{
    number_to_positive_integer, optional_number, optional_object, optional_string,
    optional_string_array, required_array, required_number, required_object, required_string,
    validation_error,
};
use super::types::{
    FanoutBranchFailurePolicy, FanoutConflictAction, FanoutConflictGate, FanoutGroupPolicy,
    FanoutSyncStrategy, FanoutThresholdAction, FanoutThresholdGate, GraphStep,
};
use crate::ValidationError;

pub fn validate_fanout_groups(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<BTreeMap<String, FanoutGroupPolicy>, ValidationError> {
    let Some(fanout) = optional_object(value, field)? else {
        return Ok(BTreeMap::new());
    };
    let groups = required_object(fanout.get("groups"), &format!("{field}.groups"))?;
    let mut validated = BTreeMap::new();
    for (group_id, raw_group) in groups {
        let group_field = format!("{field}.groups.{group_id}");
        let group = required_object(Some(raw_group), &group_field)?;
        validated.insert(
            group_id.clone(),
            fanout_group(group_id, group, &group_field)?,
        );
    }
    Ok(validated)
}

pub fn validate_fanout_step_bindings(
    steps: &[GraphStep],
    groups: &BTreeMap<String, FanoutGroupPolicy>,
) -> Result<(), ValidationError> {
    let mut used_groups: BTreeMap<String, Vec<&GraphStep>> = BTreeMap::new();
    let mut step_to_group = BTreeMap::new();
    for step in steps {
        let Some(group_id) = &step.fanout_group else {
            continue;
        };
        if !groups.contains_key(group_id) {
            return Err(validation_error(format!(
                "steps.{}.fanout_group references unknown fanout group '{group_id}'.",
                step.id
            )));
        }
        used_groups.entry(group_id.clone()).or_default().push(step);
        step_to_group.insert(step.id.clone(), group_id.clone());
    }
    for (group_id, group_policy) in groups {
        let group_steps = used_groups.get(group_id).cloned().unwrap_or_default();
        validate_group_membership(group_id, group_policy, &group_steps, steps)?;
    }
    validate_group_context_edges(steps, &step_to_group)
}

fn fanout_group(
    group_id: &str,
    group: &JsonObject,
    group_field: &str,
) -> Result<FanoutGroupPolicy, ValidationError> {
    let strategy =
        optional_sync_strategy(group.get("strategy"), &format!("{group_field}.strategy"))?
            .unwrap_or(FanoutSyncStrategy::All);
    let min_success = min_success(group, group_field, &strategy)?;
    Ok(FanoutGroupPolicy {
        group_id: group_id.to_owned(),
        on_branch_failure: optional_branch_failure_policy(
            group.get("on_branch_failure"),
            &format!("{group_field}.on_branch_failure"),
        )?
        .unwrap_or_else(|| default_branch_failure(&strategy)),
        strategy,
        min_success,
        threshold_gates: validate_threshold_gates(
            group.get("threshold_gates"),
            &format!("{group_field}.threshold_gates"),
        )?,
        conflict_gates: validate_conflict_gates(
            group.get("conflict_gates"),
            &format!("{group_field}.conflict_gates"),
        )?,
    })
}

fn min_success(
    group: &JsonObject,
    group_field: &str,
    strategy: &FanoutSyncStrategy,
) -> Result<Option<u64>, ValidationError> {
    let min_success = optional_number(
        group.get("min_success"),
        &format!("{group_field}.min_success"),
    )?
    .map(|value| number_to_positive_integer(value, &format!("{group_field}.min_success")))
    .transpose()?;
    if *strategy == FanoutSyncStrategy::Quorum && min_success.is_none() {
        return Err(validation_error(format!(
            "{group_field}.min_success must be a positive integer for quorum sync."
        )));
    }
    Ok(min_success)
}

fn validate_threshold_gates(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Vec<FanoutThresholdGate>, ValidationError> {
    if value.is_none() || matches!(value, Some(JsonValue::Null)) {
        return Ok(Vec::new());
    }
    required_array(value, field)?
        .iter()
        .enumerate()
        .map(|(index, raw_gate)| threshold_gate(raw_gate, &format!("{field}.{index}")))
        .collect()
}

fn threshold_gate(
    raw_gate: &JsonValue,
    gate_field: &str,
) -> Result<FanoutThresholdGate, ValidationError> {
    let gate = required_object(Some(raw_gate), gate_field)?;
    reject_unsupported_gate_fields(gate, gate_field)?;
    Ok(FanoutThresholdGate {
        step: required_string(gate.get("step"), &format!("{gate_field}.step"))?,
        field: required_string(gate.get("field"), &format!("{gate_field}.field"))?,
        above: required_number(gate.get("above"), &format!("{gate_field}.above"))?,
        action: required_threshold_action(gate.get("action"), &format!("{gate_field}.action"))?,
    })
}

fn validate_conflict_gates(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Vec<FanoutConflictGate>, ValidationError> {
    if value.is_none() || matches!(value, Some(JsonValue::Null)) {
        return Ok(Vec::new());
    }
    required_array(value, field)?
        .iter()
        .enumerate()
        .map(|(index, raw_gate)| conflict_gate(raw_gate, &format!("{field}.{index}")))
        .collect()
}

fn conflict_gate(
    raw_gate: &JsonValue,
    gate_field: &str,
) -> Result<FanoutConflictGate, ValidationError> {
    let gate = required_object(Some(raw_gate), gate_field)?;
    reject_unsupported_gate_fields(gate, gate_field)?;
    Ok(FanoutConflictGate {
        field: required_string(gate.get("field"), &format!("{gate_field}.field"))?,
        steps: optional_string_array(gate.get("steps"), &format!("{gate_field}.steps"))?
            .unwrap_or_default(),
        action: required_conflict_action(gate.get("action"), &format!("{gate_field}.action"))?,
    })
}

fn validate_group_membership(
    group_id: &str,
    group_policy: &FanoutGroupPolicy,
    group_steps: &[&GraphStep],
    steps: &[GraphStep],
) -> Result<(), ValidationError> {
    if group_steps.is_empty() {
        return Err(validation_error(format!(
            "fanout.groups.{group_id} is not used by any graph step."
        )));
    }
    validate_contiguous_group(group_id, group_steps, steps)?;
    validate_group_min_success(group_id, group_policy, group_steps)?;
    validate_gate_step_refs(group_id, group_policy, group_steps)
}

fn validate_contiguous_group(
    group_id: &str,
    group_steps: &[&GraphStep],
    steps: &[GraphStep],
) -> Result<(), ValidationError> {
    let indexes = group_steps
        .iter()
        .filter_map(|group_step| steps.iter().position(|step| step.id == group_step.id))
        .collect::<Vec<_>>();
    let Some(min_index) = indexes.iter().min().copied() else {
        return Ok(());
    };
    let Some(max_index) = indexes.iter().max().copied() else {
        return Ok(());
    };
    for step in steps.iter().take(max_index + 1).skip(min_index) {
        if step.fanout_group.as_deref() != Some(group_id) {
            return Err(validation_error(format!(
                "fanout group '{group_id}' steps must be contiguous."
            )));
        }
    }
    Ok(())
}

fn validate_group_min_success(
    group_id: &str,
    group_policy: &FanoutGroupPolicy,
    group_steps: &[&GraphStep],
) -> Result<(), ValidationError> {
    if group_policy
        .min_success
        .is_some_and(|min| min > group_steps.len() as u64)
    {
        return Err(validation_error(format!(
            "fanout.groups.{group_id}.min_success cannot exceed the number of branches."
        )));
    }
    Ok(())
}

fn validate_gate_step_refs(
    group_id: &str,
    group_policy: &FanoutGroupPolicy,
    group_steps: &[&GraphStep],
) -> Result<(), ValidationError> {
    let group_step_ids: BTreeSet<&str> = group_steps.iter().map(|step| step.id.as_str()).collect();
    for gate in &group_policy.threshold_gates {
        if !group_step_ids.contains(gate.step.as_str()) {
            return Err(validation_error(format!(
                "fanout.groups.{group_id}.threshold_gates step '{}' is not in the fanout group.",
                gate.step
            )));
        }
    }
    for gate in &group_policy.conflict_gates {
        for step_id in &gate.steps {
            if !group_step_ids.contains(step_id.as_str()) {
                return Err(validation_error(format!(
                    "fanout.groups.{group_id}.conflict_gates step '{step_id}' is not in the fanout group."
                )));
            }
        }
    }
    Ok(())
}

fn validate_group_context_edges(
    steps: &[GraphStep],
    step_to_group: &BTreeMap<String, String>,
) -> Result<(), ValidationError> {
    for step in steps {
        let Some(group_id) = &step.fanout_group else {
            continue;
        };
        for edge in &step.context_edges {
            if step_to_group.get(&edge.from_step) == Some(group_id) {
                return Err(validation_error(format!(
                    "steps.{}.context.{} cannot depend on another branch in the same fanout group.",
                    step.id, edge.input
                )));
            }
        }
    }
    Ok(())
}

fn optional_sync_strategy(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<FanoutSyncStrategy>, ValidationError> {
    match optional_string(value, field)?.as_deref() {
        None => Ok(None),
        Some("all") => Ok(Some(FanoutSyncStrategy::All)),
        Some("any") => Ok(Some(FanoutSyncStrategy::Any)),
        Some("quorum") => Ok(Some(FanoutSyncStrategy::Quorum)),
        Some(_) => Err(validation_error(format!(
            "{field} must be all, any, or quorum."
        ))),
    }
}

fn optional_branch_failure_policy(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<FanoutBranchFailurePolicy>, ValidationError> {
    match optional_string(value, field)?.as_deref() {
        None => Ok(None),
        Some("halt") => Ok(Some(FanoutBranchFailurePolicy::Halt)),
        Some("continue") => Ok(Some(FanoutBranchFailurePolicy::Continue)),
        Some(_) => Err(validation_error(format!(
            "{field} must be halt or continue."
        ))),
    }
}

fn default_branch_failure(strategy: &FanoutSyncStrategy) -> FanoutBranchFailurePolicy {
    match strategy {
        FanoutSyncStrategy::All => FanoutBranchFailurePolicy::Halt,
        FanoutSyncStrategy::Any | FanoutSyncStrategy::Quorum => FanoutBranchFailurePolicy::Continue,
    }
}

fn required_threshold_action(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<FanoutThresholdAction, ValidationError> {
    match optional_string(value, field)?.as_deref() {
        Some("pause") => Ok(FanoutThresholdAction::Pause),
        Some("escalate") => Ok(FanoutThresholdAction::Escalate),
        _ => Err(validation_error(format!(
            "{field} must be pause or escalate."
        ))),
    }
}

fn required_conflict_action(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<FanoutConflictAction, ValidationError> {
    match optional_string(value, field)?.as_deref() {
        Some("pause") => Ok(FanoutConflictAction::Pause),
        Some("escalate") => Ok(FanoutConflictAction::Escalate),
        _ => Err(validation_error(format!(
            "{field} must be pause or escalate."
        ))),
    }
}

fn reject_unsupported_gate_fields(
    gate: &JsonObject,
    gate_field: &str,
) -> Result<(), ValidationError> {
    for field in ["contains", "matches", "semantic", "prompt", "sentiment"] {
        if gate.contains_key(field) {
            return Err(validation_error(format!(
                "{gate_field}.{field} is not supported; graph policy must evaluate structured fields."
            )));
        }
    }
    Ok(())
}
