use runx_contracts::JsonValue;

use super::helpers::{
    optional_object, required_array, required_object, required_string, validation_error,
};
use super::types::{GraphPolicy, GraphTransitionGate};
use crate::ValidationError;

pub fn validate_graph_policy(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<GraphPolicy>, ValidationError> {
    let Some(policy) = optional_object(value, field)? else {
        return Ok(None);
    };
    let Some(transitions_value) = policy.get("transitions") else {
        return Ok(None);
    };
    if matches!(transitions_value, JsonValue::Null) {
        return Ok(None);
    }
    let transitions = required_array(Some(transitions_value), &format!("{field}.transitions"))?
        .iter()
        .enumerate()
        .map(|(index, raw_gate)| transition_gate(raw_gate, &format!("{field}.transitions.{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(GraphPolicy { transitions }))
}

fn transition_gate(
    raw_gate: &JsonValue,
    gate_field: &str,
) -> Result<GraphTransitionGate, ValidationError> {
    let gate = required_object(Some(raw_gate), gate_field)?;
    let equals = gate.get("equals").cloned();
    let not_equals = gate.get("not_equals").cloned();
    if equals.is_some() && not_equals.is_some() {
        return Err(validation_error(format!(
            "{gate_field} must not declare both equals and not_equals."
        )));
    }
    if equals.is_none() && not_equals.is_none() {
        return Err(validation_error(format!(
            "{gate_field} must declare equals or not_equals."
        )));
    }
    Ok(GraphTransitionGate {
        to: required_string(gate.get("to"), &format!("{gate_field}.to"))?,
        field: required_string(gate.get("field"), &format!("{gate_field}.field"))?,
        equals,
        not_equals,
    })
}
