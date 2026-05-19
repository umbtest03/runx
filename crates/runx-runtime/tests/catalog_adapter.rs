use std::collections::BTreeMap;
use std::path::PathBuf;

use runx_contracts::{JsonObject, JsonValue};
use runx_parser::SkillSource;
use runx_runtime::adapters::catalog::CatalogAdapter;
use runx_runtime::{InvocationStatus, RuntimeError, SkillAdapter, SkillInvocation};

#[test]
fn catalog_adapter_reports_missing_catalog_ref_as_user_failure() -> Result<(), RuntimeError> {
    let output = CatalogAdapter::fixture_catalog().invoke(invocation(None, JsonObject::new()))?;

    assert_eq!(output.status, InvocationStatus::Failure);
    assert_eq!(output.stdout, "");
    assert_eq!(
        output.stderr,
        "Catalog source requires source.catalog_ref metadata."
    );
    assert_eq!(output.exit_code, None);
    assert!(output.metadata.is_empty());
    Ok(())
}

#[test]
fn catalog_adapter_reports_missing_imported_tool_as_user_failure() -> Result<(), RuntimeError> {
    let output = CatalogAdapter::fixture_catalog().invoke(invocation(
        Some("fixture-mcp:fixture.nope"),
        JsonObject::new(),
    ))?;

    assert_eq!(output.status, InvocationStatus::Failure);
    assert_eq!(
        output.stderr,
        "Imported tool 'fixture-mcp:fixture.nope' was not found in configured tool catalogs."
    );
    assert_eq!(output.exit_code, None);
    assert!(output.metadata.is_empty());
    Ok(())
}

#[test]
fn catalog_adapter_invokes_fixture_echo_tool() -> Result<(), RuntimeError> {
    let mut inputs = JsonObject::new();
    inputs.insert("message".to_owned(), JsonValue::String("hello".to_owned()));

    let output =
        CatalogAdapter::fixture_catalog().invoke(invocation(Some("fixture.echo"), inputs))?;

    assert_eq!(output.status, InvocationStatus::Success);
    assert_eq!(output.stdout, "hello");
    assert_eq!(output.stderr, "");
    assert_eq!(output.exit_code, Some(0));
    assert_eq!(
        output.metadata.get("mcp"),
        Some(&JsonValue::Object(expected_mcp_metadata("echo")))
    );
    Ok(())
}

#[test]
fn catalog_adapter_propagates_fixture_failure() -> Result<(), RuntimeError> {
    let mut inputs = JsonObject::new();
    inputs.insert("message".to_owned(), JsonValue::String("boom".to_owned()));

    let output =
        CatalogAdapter::fixture_catalog().invoke(invocation(Some("fixture.fail"), inputs))?;

    assert_eq!(output.status, InvocationStatus::Failure);
    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, "MCP error -32000: fixture failure: boom");
    assert_eq!(output.exit_code, None);
    assert_eq!(
        output.metadata.get("mcp"),
        Some(&JsonValue::Object(expected_mcp_metadata("fail")))
    );
    Ok(())
}

#[test]
fn catalog_adapter_keeps_fixture_catalog_opt_in() -> Result<(), RuntimeError> {
    let output = CatalogAdapter::default().invoke(invocation(
        Some("fixture-mcp:fixture.echo"),
        JsonObject::new(),
    ))?;

    assert_eq!(output.status, InvocationStatus::Failure);
    assert_eq!(
        output.stderr,
        "Imported tool 'fixture-mcp:fixture.echo' was not found in configured tool catalogs."
    );
    assert!(output.metadata.is_empty());
    Ok(())
}

fn invocation(catalog_ref: Option<&str>, inputs: JsonObject) -> SkillInvocation {
    SkillInvocation {
        skill_name: "fixture.catalog".to_owned(),
        source: SkillSource {
            source_type: "catalog".to_owned(),
            command: None,
            args: Vec::new(),
            cwd: None,
            timeout_seconds: None,
            input_mode: None,
            sandbox: None,
            server: None,
            catalog_ref: catalog_ref.map(str::to_owned),
            tool: None,
            arguments: None,
            agent_card_url: None,
            agent_identity: None,
            agent: None,
            task: None,
            hook: None,
            outputs: None,
            graph: None,
            raw: JsonObject::new(),
        },
        inputs,
        skill_directory: PathBuf::from("."),
        env: BTreeMap::new(),
    }
}

fn expected_mcp_metadata(tool_name: &str) -> JsonObject {
    let mut mcp = JsonObject::new();
    mcp.insert("tool".to_owned(), JsonValue::String(tool_name.to_owned()));
    mcp.insert(
        "server_args_hash".to_owned(),
        JsonValue::String(
            "d4ae1dfdf0cefbd9a703697ec29358f080df41c1289657e5be139ce8952979b3".to_owned(),
        ),
    );
    mcp.insert(
        "server_command_hash".to_owned(),
        JsonValue::String(
            "545ea538461003efdc8c81c244531b003f6f26cfccf6c0073b3239fdedf49446".to_owned(),
        ),
    );
    mcp
}
