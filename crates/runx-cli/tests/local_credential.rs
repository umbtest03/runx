//! End-to-end proof that the OSS CLI fails closed for local process-env
//! credential delivery.
//!
//! Drives the real `runx skill` binary with `--credential` and `--secret-env`.
//! `cli-tool` runners must reject that process-env delivery path before spawn
//! so local secrets cannot enter an unbounded child process.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SECRET: &str = "ghs_cli_local_provision_secret_value";

#[test]
fn cli_rejects_local_credential_for_cli_tool_before_spawn() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = temp_root("runx-cli-local-credential");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_echo_token_skill(&temp)?;
    let receipt_dir = temp.join("receipts");

    let output = native_command()?
        .arg("skill")
        .arg(&skill_dir)
        .arg("--receipt-dir")
        .arg(&receipt_dir)
        .arg("--credential")
        .arg("github:bearer:local://github/main:repo")
        .arg("--secret-env")
        .arg(format!("GITHUB_TOKEN={SECRET}"))
        .arg("--json")
        .output()?;

    assert!(
        !output.status.success(),
        "expected local credential delivery to fail closed"
    );
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("local credential process-env delivery is not supported for cli-tool"),
        "unexpected stderr: {stderr}",
    );
    assert!(
        !stderr.contains(SECRET),
        "raw secret leaked into the error output"
    );
    assert!(
        !receipt_dir.exists(),
        "rejected credential run must not write receipts"
    );

    Ok(())
}

#[test]
fn cli_rejects_secret_env_without_credential() -> Result<(), Box<dyn std::error::Error>> {
    let temp = temp_root("runx-cli-local-credential-bad");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_echo_token_skill(&temp)?;

    let output = native_command()?
        .arg("skill")
        .arg(&skill_dir)
        .arg("--secret-env")
        .arg(format!("GITHUB_TOKEN={SECRET}"))
        .arg("--json")
        .output()?;

    assert!(
        !output.status.success(),
        "expected provisioning without --credential to fail"
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains("--credential"),
        "expected an error pointing at --credential, got: {stderr}"
    );
    assert!(
        !stderr.contains(SECRET),
        "raw secret leaked into the error output"
    );

    Ok(())
}

#[test]
fn cli_rejects_empty_secret_value() -> Result<(), Box<dyn std::error::Error>> {
    let temp = temp_root("runx-cli-local-credential-empty");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_echo_token_skill(&temp)?;

    let output = native_command()?
        .arg("skill")
        .arg(&skill_dir)
        .arg("--credential")
        .arg("github:bearer:local://github/main:repo")
        .arg("--secret-env")
        .arg("GITHUB_TOKEN=")
        .arg("--json")
        .output()?;

    assert!(
        !output.status.success(),
        "expected an empty --secret-env value to be rejected at parse time"
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains("non-empty secret value"),
        "expected an error about the empty secret value, got: {stderr}"
    );

    Ok(())
}

fn native_command() -> Result<Command, Box<dyn std::error::Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_runx"));
    command.env_clear();
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command.env("NO_COLOR", "1");
    Ok(command)
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

fn temp_root(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("{prefix}-{nanos}"))
}
