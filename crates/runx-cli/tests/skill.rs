use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn native_skill_pauses_and_resumes_with_run_id() -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::support::temp_root("runx-skill");
    let skill_dir = write_agent_task_skill(&root)?;
    let receipt_dir = root.join("receipts");

    let pause = runx_command()
        .args([
            "skill",
            skill_dir.to_str().ok_or("non-utf8 skill dir")?,
            "--receipt-dir",
            receipt_dir.to_str().ok_or("non-utf8 receipt dir")?,
            "--json",
            "--non-interactive",
            "--thread-title",
            "Docs bug",
        ])
        .output()?;
    let pause_json = assert_json(&pause, Some(2))?;
    assert_eq!(pause_json["status"], "needs_agent");
    assert_eq!(pause_json["run_id"], "run_agent_task-issue-intake-output");
    assert_eq!(pause_json["requests"][0]["kind"], "agent_act");

    let answers_path = root.join("answers.json");
    fs::write(
        &answers_path,
        serde_json::json!({
            "answers": {
                "agent_task.issue-intake.output": {
                    "intake_report": {
                        "summary": "Docs bug is bounded."
                    }
                }
            }
        })
        .to_string(),
    )?;

    let resume = runx_command()
        .args([
            "skill",
            skill_dir.to_str().ok_or("non-utf8 skill dir")?,
            "--receipt-dir",
            receipt_dir.to_str().ok_or("non-utf8 receipt dir")?,
            "--run-id",
            "issue-intake-run",
            "--answers",
            answers_path.to_str().ok_or("non-utf8 answers path")?,
            "--json",
            "--non-interactive",
        ])
        .output()?;
    let resume_json = assert_json(&resume, Some(0))?;
    assert_eq!(resume_json["status"], "sealed");
    assert_eq!(resume_json["run_id"], "issue-intake-run");
    assert_eq!(resume_json["closure"]["disposition"], "closed");
    assert_eq!(resume_json["receipt"]["schema"], "runx.receipt.v1");
    let receipt_id = resume_json["receipt_id"]
        .as_str()
        .ok_or("missing receipt_id")?;
    assert!(receipt_dir.join(format!("{receipt_id}.json")).exists());

    Ok(())
}

#[test]
fn native_skill_resolves_bare_local_skill_and_documented_input_flags()
-> Result<(), Box<dyn std::error::Error>> {
    let root = crate::support::temp_root("runx-skill-bare-ref");
    let skills_root = root.join("skills");
    fs::create_dir_all(&skills_root)?;
    let skill_dir = write_agent_task_skill(&skills_root)?;
    let receipt_dir = root.join("receipts");

    let output = runx_command()
        .current_dir(&root)
        .args([
            "skill",
            "issue-intake",
            "--receipt-dir",
            receipt_dir.to_str().ok_or("non-utf8 receipt dir")?,
            "--input",
            "thread-title=Docs bug",
            "--input",
            "severity",
            "low",
            "--json",
            "--non-interactive",
        ])
        .output()?;
    let output_json = assert_json(&output, Some(2))?;
    let inputs = &output_json["requests"][0]["invocation"]["envelope"]["inputs"];
    assert_eq!(inputs["thread_title"], "Docs bug");
    assert_eq!(inputs["severity"], "low");
    let actual_skill_dir = PathBuf::from(
        output_json["requests"][0]["invocation"]["envelope"]["execution_location"]
            ["skill_directory"]
            .as_str()
            .ok_or("missing skill directory")?,
    );
    assert_eq!(actual_skill_dir.canonicalize()?, skill_dir.canonicalize()?);

    Ok(())
}

#[test]
fn native_skill_runner_flag_selects_non_default_runner() -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::support::temp_root("runx-skill-runner-flag");
    let skill_dir = write_multi_runner_skill(&root)?;
    let receipt_dir = root.join("receipts");

    let output = runx_command()
        .args([
            "skill",
            skill_dir.to_str().ok_or("non-utf8 skill dir")?,
            "--runner",
            "second",
            "--receipt-dir",
            receipt_dir.to_str().ok_or("non-utf8 receipt dir")?,
            "--json",
            "--non-interactive",
        ])
        .output()?;
    let output_json = assert_json(&output, Some(2))?;
    assert_eq!(
        output_json["requests"][0]["id"],
        "agent_task.second-task.output"
    );

    Ok(())
}

#[test]
fn native_skill_exported_shim_resolves_to_source_skill() -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::support::temp_root("runx-skill-exported-shim");
    let source_dir = write_agent_task_skill(&root.join("source with spaces"))?;
    let shim_dir = root.join("claude").join("issue-intake");
    fs::create_dir_all(&shim_dir)?;
    fs::write(
        shim_dir.join("SKILL.md"),
        format!(
            "---\nname: issue-intake\n---\n# issue-intake\n<!-- runx-export:claude source={} - generated, do not edit -->\n",
            source_dir.display()
        ),
    )?;

    let output = runx_command()
        .args([
            "skill",
            shim_dir.to_str().ok_or("non-utf8 shim dir")?,
            "--thread-title",
            "Docs bug",
            "--json",
            "--non-interactive",
        ])
        .output()?;
    let output_json = assert_json(&output, Some(2))?;
    let actual_source_dir = PathBuf::from(
        output_json["requests"][0]["invocation"]["envelope"]["execution_location"]
            ["skill_directory"]
            .as_str()
            .ok_or("missing skill directory")?,
    );
    assert_eq!(
        actual_source_dir.canonicalize()?,
        source_dir.canonicalize()?
    );

    Ok(())
}

#[test]
fn native_skill_text_output_is_concise_for_pending_agent_request()
-> Result<(), Box<dyn std::error::Error>> {
    let root = crate::support::temp_root("runx-skill-text-output");
    let skill_dir = write_agent_task_skill(&root)?;

    let output = runx_command()
        .args([
            "skill",
            skill_dir.to_str().ok_or("non-utf8 skill dir")?,
            "--thread-title",
            "Docs bug",
            "--non-interactive",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8(output.stderr)?, "");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("status: needs_agent"));
    assert!(stdout.contains("pending_requests: 1"));
    assert!(stdout.contains("agent_task.issue-intake.output"));
    assert!(!stdout.trim_start().starts_with('{'));

    Ok(())
}

#[test]
fn native_skill_rejects_answers_without_run_id() -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::support::temp_root("runx-skill-reject-answers");
    let skill_dir = write_agent_task_skill(&root)?;
    let answers_path = root.join("answers.json");
    fs::write(&answers_path, "{}")?;
    let output = runx_command()
        .args([
            "skill",
            skill_dir.to_str().ok_or("non-utf8 skill dir")?,
            "--answers",
            answers_path.to_str().ok_or("non-utf8 answers path")?,
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(64));
    assert!(String::from_utf8(output.stderr)?.contains("runx skill --answers requires --run-id"));
    assert_eq!(String::from_utf8(output.stdout)?, "");

    Ok(())
}

#[test]
fn native_skill_rejects_run_id_without_answers() -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::support::temp_root("runx-skill-reject-run-id");
    let skill_dir = write_agent_task_skill(&root)?;
    let output = runx_command()
        .args([
            "skill",
            skill_dir.to_str().ok_or("non-utf8 skill dir")?,
            "--run-id",
            "issue-intake-run",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(64));
    assert!(String::from_utf8(output.stderr)?.contains("runx skill --run-id requires --answers"));
    assert_eq!(String::from_utf8(output.stdout)?, "");

    Ok(())
}

#[test]
fn native_skill_rejects_retired_receipt_options() -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::support::temp_root("runx-skill-reject-retired-receipt");
    let skill_dir = write_agent_task_skill(&root)?;
    let receipt_dir = root.join("receipts");
    let retired_receipt = format!("--{}", "receipt");
    let retired_receipt_dir = format!("--{}", ["receipt", "Dir"].concat());
    let retired_receipt_dir_equals = format!(
        "{}={}",
        retired_receipt_dir,
        receipt_dir.to_str().ok_or("non-utf8 receipt dir")?
    );

    for args in [
        vec![
            "skill".to_owned(),
            skill_dir.to_str().ok_or("non-utf8 skill dir")?.to_owned(),
            retired_receipt,
            receipt_dir
                .to_str()
                .ok_or("non-utf8 receipt dir")?
                .to_owned(),
        ],
        vec![
            "skill".to_owned(),
            skill_dir.to_str().ok_or("non-utf8 skill dir")?.to_owned(),
            retired_receipt_dir,
            receipt_dir
                .to_str()
                .ok_or("non-utf8 receipt dir")?
                .to_owned(),
        ],
        vec![
            "skill".to_owned(),
            skill_dir.to_str().ok_or("non-utf8 skill dir")?.to_owned(),
            retired_receipt_dir_equals,
        ],
    ] {
        let output = runx_command().args(args).output()?;
        assert_eq!(output.status.code(), Some(64));
        assert!(String::from_utf8(output.stderr)?.contains("retired runx skill receipt option"));
        assert_eq!(String::from_utf8(output.stdout)?, "");
    }

    Ok(())
}

fn runx_command() -> Command {
    crate::support::signed_runx_command("skill-test-key")
}

fn assert_json(
    output: &std::process::Output,
    expected_status: Option<i32>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if let Some(expected_status) = expected_status {
        assert_eq!(
            output.status.code(),
            Some(expected_status),
            "stderr={}\nstdout={}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }
    assert!(
        output.status.success() || expected_status.is_some(),
        "status={:?}\nstderr={}\nstdout={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(String::from_utf8(output.stderr.clone())?, "");
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn write_agent_task_skill(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let skill_dir = root.join("issue-intake");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: issue-intake\n---\n# Issue Intake\n",
    )?;
    fs::write(
        skill_dir.join("X.yaml"),
        r#"
skill: issue-intake
runners:
  intake:
    default: true
    type: agent-task
    agent: builder
    task: issue-intake
    outputs:
      intake_report: object
    inputs:
      thread_title:
        type: string
        required: false
"#,
    )?;
    Ok(skill_dir)
}

fn write_multi_runner_skill(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let skill_dir = root.join("multi-runner");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: multi-runner\n---\n# Multi Runner\n",
    )?;
    fs::write(
        skill_dir.join("X.yaml"),
        r#"
skill: multi-runner
runners:
  first:
    default: true
    type: agent-task
    agent: builder
    task: first-task
    outputs:
      result: object
  second:
    type: agent-task
    agent: builder
    task: second-task
    outputs:
      result: object
"#,
    )?;
    Ok(skill_dir)
}
