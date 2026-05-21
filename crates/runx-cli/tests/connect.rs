use std::ffi::OsString;
use std::process::Command;

use runx_cli::connect::{ConnectPlan, parse_connect_plan};

#[test]
fn parses_connect_as_oss_unavailable_stub() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        parse_connect_plan(&args(["connect", "--json"]))?,
        ConnectPlan { json: true }
    );
    let error = match parse_connect_plan(&args(["connect", "github", "--scope", "repo:read"])) {
        Ok(_) => return Err("provider-specific connect arguments must be rejected".into()),
        Err(error) => error,
    };
    assert!(error.contains("unknown runx connect argument"));
    Ok(())
}

#[test]
fn connect_command_is_explicitly_unavailable_in_oss() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_runx"))
        .arg("connect")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stdout)?.is_empty());
    assert!(
        String::from_utf8(output.stderr)?
            .contains("runx connect is not available in the MIT OSS CLI")
    );
    Ok(())
}

#[test]
fn connect_json_stub_reports_error_without_brokerage() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_runx"))
        .args(["connect", "--json"])
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stderr)?.is_empty());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains(r#""status":"error""#));
    assert!(stdout.contains("hosted/private CLI distribution"));
    Ok(())
}

fn args<const N: usize>(values: [&str; N]) -> Vec<OsString> {
    values.into_iter().map(OsString::from).collect()
}
