//! Local, no-network per-run credential provision boundary.
//!
//! `cli-tool` execution no longer accepts process-env local credentials. This
//! keeps the secret boundary fail-closed until a non-env delivery channel exists.

#![cfg(feature = "cli-tool")]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use runx_runtime::orchestrator::LocalCredentialDescriptor;
use runx_runtime::{LocalOrchestrator, RunResult, SkillRunRequest};
use tempfile::tempdir;

const SECRET: &str = "ghs_local_provision_secret_value";
#[test]
fn local_credential_for_cli_tool_is_rejected_before_spawn() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempdir()?;
    let skill_dir = write_echo_token_skill(temp.path())?;
    let receipt_dir = temp.path().join("receipts");

    let request = SkillRunRequest {
        skill_path: skill_dir,
        receipt_dir: Some(receipt_dir.clone()),
        run_id: None,
        answers_path: None,
        inputs: BTreeMap::new(),
        env: BTreeMap::new(),
        cwd: temp.path().to_path_buf(),
        local_credential: Some(LocalCredentialDescriptor {
            provider: "github".to_owned(),
            auth_mode: "bearer".to_owned(),
            env_var: "GITHUB_TOKEN".to_owned(),
            material_ref: "local://github/main".to_owned(),
            scopes: vec!["repo".to_owned()],
            secret: SECRET.to_owned(),
        }),
    };

    let error = match run_skill(request) {
        Ok(_) => {
            return Err(
                std::io::Error::other("cli-tool local credential unexpectedly succeeded").into(),
            );
        }
        Err(error) => error,
    };
    let message = error.to_string();
    assert!(
        message.contains("local credential process-env delivery is not supported for cli-tool"),
        "unexpected error: {message}",
    );
    assert!(
        !message.contains(SECRET),
        "raw secret leaked into the error output",
    );
    assert!(
        !receipt_dir.exists(),
        "rejected credential run must not write receipts",
    );

    Ok(())
}

#[test]
fn run_without_descriptor_delivers_no_credential() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let skill_dir = write_echo_token_skill(temp.path())?;

    let request = SkillRunRequest {
        skill_path: skill_dir,
        receipt_dir: Some(temp.path().join("receipts")),
        run_id: None,
        answers_path: None,
        inputs: BTreeMap::new(),
        env: BTreeMap::new(),
        cwd: temp.path().to_path_buf(),
        local_credential: None,
    };

    let result = run_skill(request)?;
    let serialized = serde_json::to_string(&result.output)?;
    assert!(
        !serialized.contains(SECRET),
        "no credential was provided, the secret must not appear anywhere",
    );
    Ok(())
}

fn run_skill(mut request: SkillRunRequest) -> Result<RunResult, Box<dyn std::error::Error>> {
    crate::support::insert_test_signing_env(&mut request.env);
    LocalOrchestrator::default()
        .run_skill(&request)
        .map_err(Into::into)
}

/// A cli-tool skill that echoes the delivered `$GITHUB_TOKEN`. The command is a
/// local shell process: no network, no hosted dependency.
fn write_echo_token_skill(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let skill_dir = root.join("echo-token");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: echo-token\n---\n# Echo Token\n",
    )?;
    fs::write(
        skill_dir.join("X.yaml"),
        r#"
skill: echo-token
runners:
  echo:
    default: true
    type: cli-tool
    command: sh
    args:
      - "-c"
      - "printf '%s' \"$GITHUB_TOKEN\""
    sandbox:
      profile: readonly
"#,
    )?;
    Ok(skill_dir)
}
