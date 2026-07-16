use std::collections::BTreeMap;

use runx_parser::{parse_runner_manifest_yaml, validate_runner_manifest};

use super::validate_declared_credential;
use crate::credentials::RUNX_HOSTED_CREDENTIAL_HANDLES_JSON_ENV;
use crate::execution::orchestrator::LocalCredentialDescriptor;

#[test]
fn selected_local_credential_validates_before_ambient_hosted_handles()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest = validate_runner_manifest(parse_runner_manifest_yaml(
        r#"
skill: credential-precedence
credentials:
  example:
    provider: example
    auth:
      api_key:
        delivery:
          env: EXAMPLE_TOKEN
runners:
  default:
    default: true
    type: cli-tool
    command: "true"
    credential: example
"#,
    )?)?;
    let runner = manifest
        .runners
        .get("default")
        .ok_or("default runner should exist")?;
    let local = LocalCredentialDescriptor {
        profile: Some("local-profile".to_owned()),
        provider: "example".to_owned(),
        auth_mode: "api_key".to_owned(),
        env_var: "EXAMPLE_TOKEN".to_owned(),
        material_ref: "local:example:local-profile".to_owned(),
        scopes: Vec::new(),
        secret: "selected-local-secret".to_owned(),
    };
    let env = BTreeMap::from([(
        RUNX_HOSTED_CREDENTIAL_HANDLES_JSON_ENV.to_owned(),
        r#"[{"credential_ref":{"type":"credential","uri":"runx:credential:hosted"},"provider":"example","purpose":"provider_api"}]"#.to_owned(),
    )]);

    validate_declared_credential(&manifest, runner, Some(&local), &env)?;
    Ok(())
}
