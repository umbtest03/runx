use std::collections::BTreeSet;

use runx_contracts::JsonObject;

use super::fanout::{validate_fanout_groups, validate_fanout_step_bindings};
use super::helpers::{
    optional_string, required_array, required_object, required_string, validation_error,
};
use super::policy::validate_graph_policy;
use super::step::validate_step;
use super::types::{ExecutionGraph, RawGraphIr};
use crate::{ParseError, ValidationError};

pub fn parse_graph_yaml(source: &str) -> Result<RawGraphIr, ParseError> {
    let document: JsonObject =
        serde_yml::from_str(source).map_err(|error| ParseError::InvalidYaml {
            field: "graph".to_owned(),
            message: error.to_string(),
        })?;
    Ok(RawGraphIr { document })
}

pub fn validate_graph(raw: RawGraphIr) -> Result<ExecutionGraph, ValidationError> {
    validate_graph_document(raw.document.clone(), Some(raw))
}

pub fn validate_graph_document(
    document: JsonObject,
    raw: Option<RawGraphIr>,
) -> Result<ExecutionGraph, ValidationError> {
    reject_unsupported_top_level(&document)?;

    let name = required_string(document.get("name"), "name")?;
    let owner = optional_string(document.get("owner"), "owner")?;
    let raw_steps = required_array(document.get("steps"), "steps")?;
    let fanout_groups = validate_fanout_groups(document.get("fanout"), "fanout")?;
    let policy = validate_graph_policy(document.get("policy"), "policy")?;
    let mut seen_step_ids = BTreeSet::new();
    let mut steps = Vec::new();

    for (index, raw_step) in raw_steps.iter().enumerate() {
        let field = format!("steps.{index}");
        let raw_step = required_object(Some(raw_step), &field)?;
        let step = validate_step(raw_step, &field, &seen_step_ids)?;
        seen_step_ids.insert(step.id.clone());
        steps.push(step);
    }

    validate_fanout_step_bindings(&steps, &fanout_groups)?;

    Ok(ExecutionGraph {
        name,
        owner,
        steps,
        fanout_groups,
        policy,
        raw: raw.unwrap_or(RawGraphIr { document }),
    })
}

fn reject_unsupported_top_level(document: &JsonObject) -> Result<(), ValidationError> {
    for field in ["sync", "schedule", "schedules"] {
        if document.contains_key(field) {
            return Err(validation_error(format!(
                "{field} is not supported by the local sequential graph runner."
            )));
        }
    }
    Ok(())
}
