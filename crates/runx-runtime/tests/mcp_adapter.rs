#![cfg(feature = "mcp")]

use runx_contracts::{JsonNumber, JsonObject, JsonValue};
use runx_runtime::RuntimeError;
use runx_runtime::adapters::mcp::map_mcp_arguments;

#[test]
fn mcp_argument_templates_map_structured_and_embedded_values() -> Result<(), RuntimeError> {
    let mut inputs = JsonObject::new();
    inputs.insert("name".to_owned(), JsonValue::String("Ada".to_owned()));
    inputs.insert("count".to_owned(), JsonValue::Number(JsonNumber::U64(3)));

    let mut nested = JsonObject::new();
    nested.insert("ok".to_owned(), JsonValue::Bool(true));

    let mut resolved_inputs = JsonObject::new();
    resolved_inputs.insert("payload".to_owned(), JsonValue::Object(nested.clone()));

    let mut template = JsonObject::new();
    template.insert(
        "exact".to_owned(),
        JsonValue::String("{{ payload }}".to_owned()),
    );
    template.insert(
        "embedded".to_owned(),
        JsonValue::String("hello {{name}} #{{ count }}".to_owned()),
    );
    template.insert(
        "invalid".to_owned(),
        JsonValue::String("keep {{ not valid }}".to_owned()),
    );

    let mapped = map_mcp_arguments(Some(&template), &inputs, &resolved_inputs)?;

    assert_eq!(mapped.get("exact"), Some(&JsonValue::Object(nested)));
    assert_eq!(
        mapped.get("embedded"),
        Some(&JsonValue::String("hello Ada #3".to_owned()))
    );
    assert_eq!(
        mapped.get("invalid"),
        Some(&JsonValue::String("keep {{ not valid }}".to_owned()))
    );
    Ok(())
}
