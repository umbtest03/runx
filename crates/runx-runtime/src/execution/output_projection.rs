use runx_contracts::{JsonObject, JsonValue};

use crate::adapter::SkillOutput;

pub(crate) struct ProjectedSkillOutput {
    pub(crate) outputs: JsonObject,
    pub(crate) claim: JsonObject,
}

#[must_use]
pub(crate) fn project_skill_output(output: &SkillOutput) -> ProjectedSkillOutput {
    let mut outputs = JsonObject::new();
    let parsed_stdout = serde_json::from_slice::<JsonValue>(output.stdout.as_bytes()).ok();
    let stdout = JsonValue::String(output.stdout.clone());
    if let Some(parsed) = parsed_stdout.as_ref() {
        outputs.insert("raw".to_owned(), stdout.clone());
        outputs.insert("skill_claim".to_owned(), parsed.clone());
    }
    outputs.insert("stdout".to_owned(), stdout);
    outputs.insert(
        "stderr".to_owned(),
        JsonValue::String(output.stderr.clone()),
    );
    outputs.insert(
        "status".to_owned(),
        JsonValue::String(if output.succeeded() {
            "success".to_owned()
        } else {
            "failure".to_owned()
        }),
    );
    let claim = match parsed_stdout {
        Some(JsonValue::Object(object)) => object,
        _ => JsonObject::new(),
    };
    ProjectedSkillOutput { outputs, claim }
}
