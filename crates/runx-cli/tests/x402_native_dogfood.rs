use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

#[test]
fn native_x402_mock_dogfood_fixtures_run_without_typescript()
-> Result<(), Box<dyn std::error::Error>> {
    let approved = run_harness_fixture("fixtures/harness/payment-approval-graph.yaml")?;
    assert_eq!(approved["schema"], "runx.harness_receipt.v1");
    assert_eq!(approved["harness"]["state"], "sealed");
    assert_eq!(approved["seal"]["disposition"], "closed");
    assert_eq!(
        child_receipt_uris(&approved),
        vec![
            "runx:harness_receipt:hrn_rcpt_payment-approval_approve-spend",
            "runx:harness_receipt:hrn_rcpt_payment-approval_fulfill",
        ]
    );

    let denied = run_harness_fixture("fixtures/harness/payment-approval-denied.yaml")?;
    assert_eq!(denied["schema"], "runx.harness_receipt.v1");
    assert_eq!(denied["harness"]["state"], "sealed");
    assert_eq!(denied["seal"]["disposition"], "blocked");
    assert_eq!(denied["seal"]["reason_code"], "graph_blocked");
    assert_eq!(
        child_receipt_uris(&denied),
        vec!["runx:harness_receipt:hrn_rcpt_payment-approval_approve-spend",]
    );

    Ok(())
}

fn run_harness_fixture(fixture: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let output = native_command()?
        .args(["harness", fixture, "--json"])
        .output()?;
    assert_success(&output)?;
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        !stdout.contains("rail_session_material_ref"),
        "native CLI receipt output must not expose raw rail session material fields"
    );
    assert!(
        !stdout.contains("rail-session-material:mock:payment-execution-001"),
        "native CLI receipt output must not expose raw rail session material refs"
    );
    Ok(serde_json::from_str(&stdout)?)
}

fn native_command() -> Result<Command, Box<dyn std::error::Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_runx"));
    command.current_dir(repo_root()?);
    command.env_clear();
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command.env("NO_COLOR", "1");
    Ok(command)
}

fn assert_success(output: &Output) -> Result<(), Box<dyn std::error::Error>> {
    assert!(
        output.status.success(),
        "status={:?}\nstderr={}\nstdout={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(String::from_utf8(output.stderr.clone())?, "");
    Ok(())
}

fn child_receipt_uris(receipt: &Value) -> Vec<String> {
    receipt["harness"]["child_harness_receipt_refs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|reference| reference["uri"].as_str().map(str::to_owned))
        .collect()
}

fn repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}
