use serde::Deserialize;

use runx_contracts::{HostRunResult, HostRunState};
use runx_sdk::host::{decode_host_result, decode_host_state, host_result_status};

#[derive(Debug, Deserialize)]
struct ResultFixture {
    expected: HostRunResult,
}

#[derive(Debug, Deserialize)]
struct StateFixture {
    expected: HostRunState,
}

#[test]
fn sdk_decodes_host_run_result_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let fixture: ResultFixture = serde_json::from_str(include_str!(
        "../../../fixtures/sdk-rust/host-protocol/result-host-run-completed.json"
    ))?;
    let expected_json = serde_json::to_string(&fixture.expected)?;
    let decoded = decode_host_result(&expected_json)?;

    assert_eq!(host_result_status(&decoded), "completed");
    assert_eq!(decoded, fixture.expected);
    Ok(())
}

#[test]
fn sdk_decodes_host_state_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let fixture: StateFixture = serde_json::from_str(include_str!(
        "../../../fixtures/sdk-rust/host-protocol/inspect-host-state-paused.json"
    ))?;
    let expected_json = serde_json::to_string(&fixture.expected)?;
    let decoded = decode_host_state(&expected_json)?;

    assert_eq!(decoded, fixture.expected);
    Ok(())
}
