use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use runx_contracts::{JsonObject, JsonValue};
use runx_parser::{
    SkillRunnerDefinition, SkillRunnerManifest, parse_runner_manifest_yaml,
    validate_runner_manifest,
};

use super::{
    SkillCredentialContext, SkillCredentialError, SkillCredentialRequest, resolve_skill_credential,
};
use crate::services::WorkspaceEnv;

pub fn resolve_skill_credential_for_path(
    skill_path: &Path,
    selected_runner: Option<&str>,
    explicit_profile: Option<&str>,
    workspace: &WorkspaceEnv,
) -> Result<Option<SkillCredentialContext>, SkillCredentialError> {
    let skill_dir = skill_directory(skill_path)?;
    let source = match fs::read_to_string(skill_dir.join("X.yaml")) {
        Ok(source) => source,
        Err(error) if error.kind() == io::ErrorKind::NotFound && explicit_profile.is_none() => {
            return Ok(None);
        }
        Err(error) => return Err(error.into()),
    };
    let raw = parse_runner_manifest_yaml(&source)
        .map_err(|error| SkillCredentialError::InvalidSkill(error.to_string()))?;
    let manifest = validate_runner_manifest(raw)
        .map_err(|error| SkillCredentialError::InvalidSkill(error.to_string()))?;
    credential_context_for_runner(
        &skill_dir,
        &manifest,
        selected_runner,
        explicit_profile,
        workspace,
    )
}

fn credential_context_for_runner(
    skill_dir: &Path,
    manifest: &SkillRunnerManifest,
    selected_runner: Option<&str>,
    explicit_profile: Option<&str>,
    workspace: &WorkspaceEnv,
) -> Result<Option<SkillCredentialContext>, SkillCredentialError> {
    let runner = selected_runner_definition(manifest, selected_runner)?;
    let Some(requirement_name) = runner.credential.as_ref() else {
        if explicit_profile.is_some() {
            return Err(SkillCredentialError::InvalidSkill(
                "--profile is only valid when the selected runner declares a credential".to_owned(),
            ));
        }
        return Ok(None);
    };
    let requirement = manifest
        .credentials
        .get(requirement_name)
        .cloned()
        .ok_or_else(|| {
            SkillCredentialError::InvalidSkill(format!(
                "runner credential '{requirement_name}' is not declared"
            ))
        })?;
    let request = SkillCredentialRequest {
        skill_name: manifest.skill.clone().unwrap_or_else(|| {
            skill_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("skill")
                .to_owned()
        }),
        requirement_name: requirement_name.clone(),
        requirement,
        scopes: declared_scopes(&runner.raw),
        explicit_profile: explicit_profile.map(str::to_owned),
    };
    let resolution = resolve_skill_credential(&request, workspace)?;
    Ok(Some(SkillCredentialContext {
        request,
        resolution,
    }))
}

fn skill_directory(skill_path: &Path) -> Result<PathBuf, SkillCredentialError> {
    if skill_path.is_dir() {
        return Ok(skill_path.to_path_buf());
    }
    if skill_path.file_name().and_then(|value| value.to_str()) == Some("SKILL.md") {
        return skill_path.parent().map(Path::to_path_buf).ok_or_else(|| {
            SkillCredentialError::InvalidSkill(format!(
                "skill path has no parent: {}",
                skill_path.display()
            ))
        });
    }
    Err(SkillCredentialError::InvalidSkill(format!(
        "skill reference must point to a package directory or SKILL.md: {}",
        skill_path.display()
    )))
}

fn selected_runner_definition<'a>(
    manifest: &'a SkillRunnerManifest,
    selected: Option<&str>,
) -> Result<&'a SkillRunnerDefinition, SkillCredentialError> {
    if let Some(selected) = selected {
        return manifest.runners.get(selected).ok_or_else(|| {
            SkillCredentialError::InvalidSkill(format!("skill has no runner '{selected}'"))
        });
    }
    let mut defaults = manifest.runners.values().filter(|runner| runner.default);
    match (defaults.next(), defaults.next()) {
        (Some(runner), None) => Ok(runner),
        (None, _) if manifest.runners.len() == 1 => {
            manifest.runners.values().next().ok_or_else(|| {
                SkillCredentialError::InvalidSkill("skill declares no runners".into())
            })
        }
        (None, _) => Err(SkillCredentialError::InvalidSkill(
            "skill manifest has no default runner".to_owned(),
        )),
        (Some(_), Some(_)) => Err(SkillCredentialError::InvalidSkill(
            "skill manifest declares multiple default runners".to_owned(),
        )),
    }
}

fn declared_scopes(value: &JsonObject) -> Vec<String> {
    let mut scopes = Vec::new();
    collect_declared_scopes(&JsonValue::Object(value.clone()), &mut scopes);
    scopes.sort();
    scopes.dedup();
    scopes
}

fn collect_declared_scopes(value: &JsonValue, scopes: &mut Vec<String>) {
    match value {
        JsonValue::Object(object) => {
            if let Some(values) = object.get("scopes").and_then(JsonValue::as_array) {
                scopes.extend(
                    values
                        .iter()
                        .filter_map(JsonValue::as_str)
                        .map(str::to_owned),
                );
            }
            for (key, value) in object {
                if key != "scopes" {
                    collect_declared_scopes(value, scopes);
                }
            }
        }
        JsonValue::Array(values) => {
            for value in values {
                collect_declared_scopes(value, scopes);
            }
        }
        _ => {}
    }
}
