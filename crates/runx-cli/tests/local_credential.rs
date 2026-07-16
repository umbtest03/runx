//! End-to-end proof that the OSS CLI fails closed for local process-env
//! credential delivery.
//!
//! Drives the real `runx skill` binary with `--credential` and `--secret-env`.
//! `cli-tool` runners must reject that process-env delivery path before spawn
//! so local secrets cannot enter an unbounded child process.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;

const SECRET: &str = "ghs_cli_local_provision_secret_value";

#[test]
fn workspace_env_does_not_block_native_help() -> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-workspace-env-help");
    fs::create_dir_all(&temp)?;
    fs::write(temp.join(".env"), format!("GITHUB_TOKEN='{SECRET}\n"))?;

    let output = native_command()?
        .current_dir(&temp)
        .arg("skill")
        .arg("--help")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(output.status.success(), "native help failed: {stderr}");
    assert!(stdout.contains("runx skill"));
    assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
    Ok(())
}

#[test]
fn malformed_workspace_env_fails_json_safe_without_secret_exposure()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-workspace-env-malformed");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_env_probe_skill(&temp)?;
    fs::write(temp.join(".env"), format!("GITHUB_TOKEN='{SECRET}\n"))?;

    let output = native_command()?
        .current_dir(&temp)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--json")
        .arg("--skip-operator-context")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let value = serde_json::from_slice::<Value>(&output.stdout)?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(value["status"], "failure");
    assert_eq!(value["error"]["code"], "workspace_env_error");
    assert!(
        value["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("invalid syntax"))
    );
    assert!(stderr.is_empty());
    assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
    Ok(())
}

#[test]
fn cli_tool_receives_allowlisted_workspace_env_without_wrapper()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-workspace-env");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_env_probe_skill(&temp)?;
    fs::write(temp.join(".env"), format!("GITHUB_TOKEN={SECRET}\n"))?;

    let output = native_command()?
        .current_dir(&temp)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--json")
        .arg("--skip-operator-context")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        output.status.success(),
        "workspace env run failed: {stderr}\n{stdout}"
    );
    assert!(stdout.contains(r#"\"configured\":true"#));
    assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
    Ok(())
}

#[test]
fn workspace_env_is_loaded_from_discovered_project_root() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = crate::support::temp_root("runx-cli-workspace-env-nested");
    let nested = temp.join("nested/operator");
    let runx_dir = temp.join(".runx");
    fs::create_dir_all(&nested)?;
    fs::create_dir_all(&runx_dir)?;
    fs::write(runx_dir.join("project.json"), valid_project_state())?;
    let skill_dir = write_env_probe_skill(&temp)?;
    fs::write(temp.join(".env"), format!("GITHUB_TOKEN={SECRET}\n"))?;

    let output = native_command()?
        .current_dir(&nested)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--json")
        .arg("--skip-operator-context")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        output.status.success(),
        "nested workspace env run failed: {stderr}\n{stdout}"
    );
    assert!(stdout.contains(r#"\"configured\":true"#));
    assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
    Ok(())
}

#[test]
fn workspace_env_supports_quoted_values() -> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-workspace-env-quoted");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_env_probe_skill(&temp)?;
    fs::write(
        temp.join(".env"),
        "GITHUB_TOKEN=\"quoted # value\"\nEXPECTED_TOKEN=\"quoted # value\"\n",
    )?;

    let output = native_command()?
        .current_dir(&temp)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--json")
        .arg("--skip-operator-context")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        output.status.success(),
        "quoted workspace env run failed: {stderr}\n{stdout}"
    );
    assert!(stdout.contains(r#"\"configured\":true"#));
    assert!(!stdout.contains("quoted # value") && !stderr.contains("quoted # value"));
    Ok(())
}

#[test]
fn workspace_env_remains_blocked_without_sandbox_allowlist()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-workspace-env-denied");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_env_denial_skill(&temp)?;
    fs::write(temp.join(".env"), format!("GITHUB_TOKEN={SECRET}\n"))?;

    let output = native_command()?
        .current_dir(&temp)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--json")
        .arg("--skip-operator-context")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        output.status.success(),
        "deny-by-default workspace env run failed: {stderr}\n{stdout}"
    );
    assert!(stdout.contains(r#"\"blocked\":true"#));
    assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
    Ok(())
}

#[test]
fn process_env_takes_precedence_over_workspace_env() -> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-workspace-env-precedence");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_env_probe_skill(&temp)?;
    fs::write(temp.join(".env"), "GITHUB_TOKEN=from-file\n")?;

    let output = native_command()?
        .current_dir(&temp)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--json")
        .arg("--skip-operator-context")
        .env("GITHUB_TOKEN", SECRET)
        .env("EXPECTED_TOKEN", SECRET)
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        output.status.success(),
        "workspace env precedence run failed: {stderr}\n{stdout}"
    );
    assert!(stdout.contains(r#"\"configured\":true"#));
    assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
    Ok(())
}

#[test]
fn credential_profile_resolves_secret_from_workspace_env() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = crate::support::temp_root("runx-cli-workspace-profile");
    let runx_dir = temp.join(".runx");
    fs::create_dir_all(&runx_dir)?;
    let skill_dir = write_echo_token_skill(&temp)?;
    fs::write(temp.join(".env"), format!("GITHUB_TOKEN={SECRET}\n"))?;
    fs::write(
        runx_dir.join("credentials.json"),
        r#"{
  "profiles": {
    "github": {
      "credential": "github:bearer:local://github/main",
      "secret_env": "GITHUB_TOKEN",
      "scopes": ["repo:read"]
    }
  }
}"#,
    )?;

    let output = native_command()?
        .current_dir(&temp)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--credential-profile")
        .arg("github")
        .arg("--json")
        .arg("--skip-operator-context")
        .output()?;

    assert!(!output.status.success());
    let message = json_failure_message(&output.stdout)?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        message.contains("local credential process-env delivery is not supported for cli-tool")
    );
    assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
    Ok(())
}

#[test]
fn resume_loads_workspace_env_from_discovered_project_root()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-workspace-env-resume");
    let nested = temp.join("nested/operator");
    let runx_dir = temp.join(".runx");
    fs::create_dir_all(&nested)?;
    fs::create_dir_all(&runx_dir)?;
    fs::write(runx_dir.join("project.json"), valid_project_state())?;
    let skill_dir = write_resume_env_skill(&temp)?;
    let receipt_dir = temp.join("receipts");
    fs::write(temp.join(".env"), "RESUME_PROBE_TOKEN=before-resume\n")?;

    let pause = native_command()?
        .current_dir(&nested)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--receipt-dir")
        .arg(&receipt_dir)
        .arg("--json")
        .arg("--non-interactive")
        .arg("--skip-operator-context")
        .output()?;
    assert_eq!(pause.status.code(), Some(2));
    let pause_json = serde_json::from_slice::<Value>(&pause.stdout)?;
    assert_eq!(pause_json["status"], "needs_agent");
    let run_id = pause_json["run_id"].as_str().ok_or("missing run id")?;

    // Resume is a new command and must capture the current workspace snapshot,
    // rather than retaining the pause command's values.
    fs::write(temp.join(".env"), "RESUME_PROBE_TOKEN=after-resume\n")?;
    let answers_path = temp.join("answers.json");
    fs::write(
        &answers_path,
        serde_json::json!({
            "answers": {
                "agent_task.approve-workspace-env.output": {
                    "approved": true
                }
            }
        })
        .to_string(),
    )?;

    let resume = native_command()?
        .current_dir(&nested)
        .arg("resume")
        .arg(run_id)
        .arg(&answers_path)
        .arg("--receipt-dir")
        .arg(&receipt_dir)
        .arg("--json")
        .output()?;
    let resume_json = serde_json::from_slice::<Value>(&resume.stdout)?;
    assert!(
        resume.status.success(),
        "resume failed: {}\n{}",
        String::from_utf8_lossy(&resume.stderr),
        String::from_utf8_lossy(&resume.stdout)
    );
    assert_eq!(resume_json["status"], "sealed");
    assert!(!String::from_utf8(resume.stdout)?.contains("after-resume"));
    assert!(!String::from_utf8(resume.stderr)?.contains("after-resume"));
    Ok(())
}

#[test]
fn inline_graph_cli_tool_preserves_timeout_policy() -> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-inline-graph-timeout");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_inline_graph_timeout_skill(&temp)?;

    let started = Instant::now();
    let output = native_command()?
        .current_dir(&temp)
        .arg("skill")
        .arg(&skill_dir)
        .arg("--json")
        .arg("--skip-operator-context")
        .output()?;
    let elapsed = started.elapsed();
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    assert!(
        elapsed < Duration::from_secs(4),
        "inline graph ignored its one-second timeout and ran for {elapsed:?}"
    );
    assert!(
        !output.status.success(),
        "timed-out inline graph unexpectedly succeeded: {stderr}\n{stdout}"
    );
    Ok(())
}

#[test]
fn cli_rejects_local_credential_for_cli_tool_before_spawn() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = crate::support::temp_root("runx-cli-local-credential");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_echo_token_skill(&temp)?;
    let receipt_dir = temp.join("receipts");

    let output = native_command()?
        .arg("skill")
        .arg(&skill_dir)
        .arg("--receipt-dir")
        .arg(&receipt_dir)
        .arg("--credential")
        .arg("github:bearer:local://github/main")
        .arg("--credential-scope")
        .arg("repo:read")
        .arg("--secret-env")
        .arg("GITHUB_TOKEN")
        .arg("--json")
        .arg("--skip-operator-context")
        .env("GITHUB_TOKEN", SECRET)
        .output()?;

    assert!(
        !output.status.success(),
        "expected local credential delivery to fail closed"
    );
    let message = json_failure_message(&output.stdout)?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        message.contains("local credential process-env delivery is not supported for cli-tool"),
        "unexpected failure message: {message}",
    );
    assert!(
        stderr.is_empty(),
        "json failures should keep stderr clean, got: {stderr}"
    );
    assert!(
        !stdout.contains(SECRET) && !stderr.contains(SECRET),
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
    let temp = crate::support::temp_root("runx-cli-local-credential-bad");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_echo_token_skill(&temp)?;

    let output = native_command()?
        .arg("skill")
        .arg(&skill_dir)
        .arg("--secret-env")
        .arg("GITHUB_TOKEN")
        .arg("--json")
        .arg("--skip-operator-context")
        .env("GITHUB_TOKEN", SECRET)
        .output()?;

    assert!(
        !output.status.success(),
        "expected provisioning without --credential to fail"
    );
    let message = json_failure_message(&output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        message.contains("--credential"),
        "expected an error pointing at --credential, got: {message}"
    );
    assert!(
        stderr.is_empty(),
        "json failures should keep stderr clean, got: {stderr}"
    );
    assert!(
        !String::from_utf8(output.stdout)?.contains(SECRET) && !stderr.contains(SECRET),
        "raw secret leaked into the error output"
    );

    Ok(())
}

#[test]
fn cli_rejects_empty_secret_value() -> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-local-credential-empty");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_echo_token_skill(&temp)?;

    let output = native_command()?
        .arg("skill")
        .arg(&skill_dir)
        .arg("--credential")
        .arg("github:bearer:local://github/main")
        .arg("--credential-scope")
        .arg("repo:read")
        .arg("--secret-env")
        .arg("GITHUB_TOKEN")
        .arg("--json")
        .arg("--skip-operator-context")
        .env("GITHUB_TOKEN", "")
        .output()?;

    assert!(
        !output.status.success(),
        "expected an empty --secret-env value to be rejected at parse time"
    );
    let message = json_failure_message(&output.stdout)?;
    assert!(
        message.contains("non-empty secret value"),
        "expected an error about the empty secret value, got: {message}"
    );

    Ok(())
}

#[test]
fn cli_rejects_secret_env_value_on_argv() -> Result<(), Box<dyn std::error::Error>> {
    let temp = crate::support::temp_root("runx-cli-local-credential-argv-secret");
    fs::create_dir_all(&temp)?;
    let skill_dir = write_echo_token_skill(&temp)?;

    let output = native_command()?
        .arg("skill")
        .arg(&skill_dir)
        .arg("--credential")
        .arg("github:bearer:local://github/main")
        .arg("--credential-scope")
        .arg("repo:read")
        .arg("--secret-env")
        .arg(format!("GITHUB_TOKEN={SECRET}"))
        .arg("--json")
        .arg("--skip-operator-context")
        .output()?;

    assert!(
        !output.status.success(),
        "expected argv secret material to be rejected"
    );
    let message = json_failure_message(&output.stdout)?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        message.contains("not an inline value"),
        "expected an error about argv secret material, got: {message}"
    );
    assert!(
        stderr.is_empty(),
        "json failures should keep stderr clean, got: {stderr}"
    );
    assert!(
        !stdout.contains(SECRET) && !stderr.contains(SECRET),
        "raw secret leaked into the error output"
    );

    Ok(())
}

fn native_command() -> Result<Command, Box<dyn std::error::Error>> {
    Ok(crate::support::isolated_runx_command_with_inherited_cwd(
        "local-credential-test-key",
    ))
}

fn valid_project_state() -> &'static str {
    r#"{"version":1,"project_id":"proj_workspace_env_test","created_at":"2026-07-16T00:00:00Z"}"#
}

fn json_failure_message(stdout: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let value = serde_json::from_slice::<Value>(stdout)?;
    assert_eq!(value["status"], "failure");
    Ok(value["error"]["message"]
        .as_str()
        .ok_or("missing failure message")?
        .to_owned())
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

fn write_env_probe_skill(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let skill_dir = root.join("env-probe");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: env-probe\n---\n# Environment Probe\n",
    )?;
    fs::write(
        skill_dir.join("X.yaml"),
        r#"
skill: env-probe
runners:
  probe:
    default: true
    type: cli-tool
    command: sh
    args:
      - "-c"
      - 'test -n "$GITHUB_TOKEN" && { test -z "$EXPECTED_TOKEN" || test "$GITHUB_TOKEN" = "$EXPECTED_TOKEN"; } && printf ''{"configured":true}'''
    sandbox:
      profile: readonly
      cwd_policy: skill-directory
      env_allowlist:
        - GITHUB_TOKEN
        - EXPECTED_TOKEN
      require_enforcement: false
"#,
    )?;
    Ok(skill_dir)
}

fn write_env_denial_skill(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let skill_dir = root.join("env-denial");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: env-denial\n---\n# Environment Denial\n",
    )?;
    fs::write(
        skill_dir.join("X.yaml"),
        r#"
skill: env-denial
runners:
  probe:
    default: true
    type: cli-tool
    command: sh
    args:
      - "-c"
      - 'test -z "$GITHUB_TOKEN" && printf ''{"blocked":true}'''
    sandbox:
      profile: readonly
      cwd_policy: skill-directory
      require_enforcement: false
"#,
    )?;
    Ok(skill_dir)
}

fn write_resume_env_skill(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let skill_dir = root.join("resume-env");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: resume-env\n---\n# Resume Environment\n",
    )?;
    fs::write(
        skill_dir.join("X.yaml"),
        r#"
skill: resume-env
runners:
  resume-env:
    default: true
    type: graph
    graph:
      name: resume-env
      steps:
        - id: approve
          run:
            type: agent-task
            agent: reviewer
            task: approve-workspace-env
            outputs:
              approved: boolean
        - id: probe
          run:
            type: cli-tool
            command: sh
            args:
              - "-c"
              - 'payload="$(cat)"; if test -z "$payload"; then echo missing-stdin >&2; exit 9; elif test -z "$RESUME_PROBE_TOKEN"; then echo missing-probe >&2; exit 10; elif test "$RESUME_PROBE_TOKEN" != "after-resume"; then echo stale-probe >&2; exit 11; fi'
            timeout_seconds: 5
            input_mode: stdin
            sandbox:
              profile: readonly
              cwd_policy: skill-directory
              env_allowlist:
                - RESUME_PROBE_TOKEN
              require_enforcement: false
"#,
    )?;
    Ok(skill_dir)
}

fn write_inline_graph_timeout_skill(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let skill_dir = root.join("inline-graph-timeout");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: inline-graph-timeout\n---\n# Inline Graph Timeout\n",
    )?;
    fs::write(
        skill_dir.join("X.yaml"),
        r#"
skill: inline-graph-timeout
runners:
  timeout:
    default: true
    type: graph
    graph:
      name: inline-graph-timeout
      steps:
        - id: timeout
          run:
            type: cli-tool
            command: sh
            args:
              - "-c"
              - "sleep 5"
            timeout_seconds: 1
            sandbox:
              profile: readonly
              cwd_policy: skill-directory
              require_enforcement: false
"#,
    )?;
    Ok(skill_dir)
}
