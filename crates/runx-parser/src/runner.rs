use std::collections::BTreeMap;

use runx_contracts::{JsonObject, JsonValue};
use serde::{Deserialize, Serialize};

use crate::skill::{
    CatalogMetadata, RunnerHarnessManifest, SkillRunnerDefinition, validate_catalog_metadata,
    validate_harness_manifest, validate_runner_definition,
};
use crate::{ParseError, ValidationError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawRunnerManifestIr {
    pub document: JsonObject,
    pub raw: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillRunnerManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog: Option<CatalogMetadata>,
    pub runners: BTreeMap<String, SkillRunnerDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harness: Option<RunnerHarnessManifest>,
    pub raw: RawRunnerManifestIr,
}

pub fn parse_runner_manifest_yaml(yaml: &str) -> Result<RawRunnerManifestIr, ParseError> {
    let parsed: JsonValue = serde_yml::from_str(yaml).map_err(|error| ParseError::InvalidYaml {
        field: "runner_manifest".to_owned(),
        message: error.to_string(),
    })?;
    let JsonValue::Object(document) = parsed else {
        return Err(ParseError::InvalidDocument {
            field: "runner_manifest".to_owned(),
            message: "Runner manifest YAML must parse to an object.".to_owned(),
        });
    };
    Ok(RawRunnerManifestIr {
        document,
        raw: yaml.to_owned(),
    })
}

pub fn validate_runner_manifest(
    raw: RawRunnerManifestIr,
) -> Result<SkillRunnerManifest, ValidationError> {
    let runners_record = required_object(raw.document.get("runners"), "runners")?;
    let mut runners = BTreeMap::new();
    for (name, value) in runners_record {
        let JsonValue::Object(runner) = value else {
            return Err(validation_error(format!(
                "runners.{name} must be an object."
            )));
        };
        runners.insert(
            name.clone(),
            validate_runner_definition(name, runner.clone())?,
        );
    }

    let harness = validate_harness_manifest(
        optional_object(raw.document.get("harness"), "harness")?,
        "harness",
    )?;
    validate_harness_runners(&harness, &runners)?;

    Ok(SkillRunnerManifest {
        skill: optional_string(raw.document.get("skill"), "skill")?,
        catalog: validate_catalog_metadata(
            optional_object(raw.document.get("catalog"), "catalog")?,
            "catalog",
        )?,
        runners,
        harness,
        raw,
    })
}

pub fn resolve_post_run_reflect_policy(
    runx: Option<&JsonObject>,
    field: &str,
) -> Result<String, ValidationError> {
    let post_run = optional_object(field_value(runx, "post_run"), &format!("{field}.post_run"))?;
    let reflect = optional_string(
        field_value(post_run.as_ref(), "reflect"),
        &format!("{field}.post_run.reflect"),
    )?
    .unwrap_or_else(|| "never".to_owned());
    if matches!(reflect.as_str(), "auto" | "always" | "never") {
        return Ok(reflect);
    }
    Err(validation_error(format!(
        "{field}.post_run.reflect must be auto, always, or never."
    )))
}

fn validate_harness_runners(
    harness: &Option<RunnerHarnessManifest>,
    runners: &BTreeMap<String, SkillRunnerDefinition>,
) -> Result<(), ValidationError> {
    for entry in harness.iter().flat_map(|harness| harness.cases.iter()) {
        if let Some(runner) = &entry.runner {
            if !runners.contains_key(runner) {
                return Err(validation_error(format!(
                    "harness.cases runner {runner} is not declared in runners."
                )));
            }
        }
    }
    Ok(())
}

fn validation_error(message: impl Into<String>) -> ValidationError {
    ValidationError::InvalidField {
        field: "runner_manifest".to_owned(),
        message: message.into(),
    }
}

fn required_object<'a>(
    value: Option<&'a JsonValue>,
    field: &str,
) -> Result<&'a JsonObject, ValidationError> {
    match value {
        Some(JsonValue::Object(value)) => Ok(value),
        None | Some(JsonValue::Null) => Err(validation_error(format!("{field} is required."))),
        Some(_) => Err(validation_error(format!("{field} must be an object."))),
    }
}

fn optional_object(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<JsonObject>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::Object(value)) => Ok(Some(value.clone())),
        Some(_) => Err(validation_error(format!("{field} must be an object."))),
    }
}

fn optional_string(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<String>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(validation_error(format!("{field} must be a string."))),
    }
}

fn field_value<'a>(object: Option<&'a JsonObject>, field: &str) -> Option<&'a JsonValue> {
    object.and_then(|object| object.get(field))
}
