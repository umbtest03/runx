#![cfg(all(feature = "cli-tool", any(feature = "mcp", feature = "mcp-rmcp")))]

use std::collections::BTreeMap;

use runx_core::policy::{CredentialBindingDecision, CredentialEnvelope};
use runx_parser::{SkillSandbox, SkillSource};
use runx_runtime::adapters::cli_tool::CliToolAdapter;
use runx_runtime::adapters::mcp::{FixtureMcpTransport, McpAdapter};
use runx_runtime::{
    CredentialDelivery, CredentialDeliveryError, CredentialDeliveryProfile,
    InMemoryMaterialResolver, InvocationStatus, ResolvedCredentialMaterial, SkillAdapter,
    SkillInvocation,
};

#[test]
fn delivery_profile_requires_allowed_binding() -> Result<(), Box<dyn std::error::Error>> {
    let result = CredentialDelivery::from_allowed_binding(
        &CredentialBindingDecision::Deny {
            reasons: vec!["grant mismatch".to_owned()],
        },
        &credential(),
        &github_profile()?,
        &resolver(),
    );

    match result {
        Err(CredentialDeliveryError::BindingDenied { reasons }) => {
            assert_eq!(reasons, vec!["grant mismatch"]);
        }
        Ok(_) => {
            return Err(std::io::Error::other(
                "credential delivery must fail closed on denied binding",
            )
            .into());
        }
        Err(error) => {
            return Err(std::io::Error::other(format!("unexpected error: {error}")).into());
        }
    }
    Ok(())
}

#[test]
fn delivery_profile_rejects_provider_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let result = CredentialDelivery::from_allowed_binding(
        &CredentialBindingDecision::Allow {
            reasons: vec!["allowed".to_owned()],
        },
        &credential(),
        &CredentialDeliveryProfile::env_token("slack", "oauth_bearer", "SLACK_TOKEN")?,
        &resolver(),
    );

    match result {
        Err(CredentialDeliveryError::ProviderMismatch {
            credential_provider,
            profile_provider,
        }) => {
            assert_eq!(credential_provider, "github");
            assert_eq!(profile_provider, "slack");
        }
        Ok(_) => {
            return Err(std::io::Error::other(
                "credential delivery must reject mismatched providers",
            )
            .into());
        }
        Err(error) => {
            return Err(std::io::Error::other(format!("unexpected error: {error}")).into());
        }
    }
    Ok(())
}

#[test]
fn cli_tool_injects_secret_env_and_redacts_process_output() -> Result<(), Box<dyn std::error::Error>>
{
    let delivery = allowed_delivery()?;
    let output = CliToolAdapter.invoke(SkillInvocation {
        skill_name: "credential.echo".to_owned(),
        source: cli_source(),
        inputs: Default::default(),
        resolved_inputs: Default::default(),
        skill_directory: std::env::current_dir()?,
        env: process_env(),
        credential_delivery: delivery,
    })?;

    assert_eq!(output.status, InvocationStatus::Success);
    assert_eq!(output.stdout.trim(), "[redacted-credential]");
    assert!(!output.stdout.contains("ghs_secret_token"));
    assert!(
        !serde_json::to_string(&output.metadata)?.contains("ghs_secret_token"),
        "credential material must not enter sandbox metadata"
    );
    Ok(())
}

#[test]
fn mcp_adapter_delivers_secret_env_and_redacts_tool_result()
-> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = runx_contracts::JsonObject::new();
    inputs.insert(
        "name".to_owned(),
        runx_contracts::JsonValue::String("GITHUB_TOKEN".to_owned()),
    );
    let output = McpAdapter::new(FixtureMcpTransport).invoke(SkillInvocation {
        skill_name: "credential.mcp".to_owned(),
        source: mcp_source(),
        inputs,
        resolved_inputs: Default::default(),
        skill_directory: std::env::current_dir()?,
        env: process_env(),
        credential_delivery: allowed_delivery()?,
    })?;

    assert_eq!(output.status, InvocationStatus::Success);
    assert_eq!(output.stdout.trim(), "[redacted-credential]");
    assert!(!output.stdout.contains("ghs_secret_token"));
    assert!(!serde_json::to_string(&output.metadata)?.contains("ghs_secret_token"));
    Ok(())
}

fn allowed_delivery() -> Result<CredentialDelivery, CredentialDeliveryError> {
    CredentialDelivery::from_allowed_binding(
        &CredentialBindingDecision::Allow {
            reasons: vec!["credential material matches admitted grant".to_owned()],
        },
        &credential(),
        &github_profile()?,
        &resolver(),
    )
}

fn resolver() -> InMemoryMaterialResolver {
    InMemoryMaterialResolver::with_material(
        "secret://github/main",
        ResolvedCredentialMaterial::access_token("secret://github/main", "ghs_secret_token"),
    )
}

fn github_profile() -> Result<CredentialDeliveryProfile, CredentialDeliveryError> {
    CredentialDeliveryProfile::env_token("github", "oauth_bearer", "GITHUB_TOKEN")
}

fn credential() -> CredentialEnvelope {
    CredentialEnvelope {
        kind: "runx.credential-envelope.v1".to_owned(),
        grant_id: "grant_github_main".to_owned(),
        provider: "github".to_owned(),
        auth_mode: "oauth_bearer".to_owned(),
        material_kind: "access_token".to_owned(),
        connection_id: Some("conn_github_main".to_owned()),
        scopes: vec!["repo".to_owned()],
        grant_reference: None,
        material_ref: "secret://github/main".to_owned(),
    }
}

fn cli_source() -> SkillSource {
    SkillSource {
        source_type: "cli-tool".to_owned(),
        command: Some("sh".to_owned()),
        args: vec![
            "-c".to_owned(),
            "printf '%s\\n' \"$GITHUB_TOKEN\"".to_owned(),
        ],
        cwd: None,
        timeout_seconds: Some(5),
        input_mode: None,
        sandbox: Some(readonly_sandbox()),
        server: None,
        catalog_ref: None,
        tool: None,
        arguments: None,
        agent_card_url: None,
        agent_identity: None,
        agent: None,
        task: None,
        hook: None,
        outputs: None,
        graph: None,
        raw: Default::default(),
    }
}

fn mcp_source() -> SkillSource {
    let mut source = cli_source();
    source.source_type = "mcp".to_owned();
    source.command = None;
    source.args = Vec::new();
    source.server = Some(runx_parser::SkillMcpServer {
        command: "fixture".to_owned(),
        args: Vec::new(),
        cwd: None,
    });
    source.tool = Some("env".to_owned());
    source
}

fn readonly_sandbox() -> SkillSandbox {
    SkillSandbox {
        profile: "readonly".to_owned(),
        cwd_policy: None,
        env_allowlist: Some(vec!["PATH".to_owned()]),
        network: None,
        writable_paths: Vec::new(),
        require_enforcement: None,
        approved_escalation: None,
        raw: Default::default(),
    }
}

fn process_env() -> BTreeMap<String, String> {
    std::env::vars().collect()
}
