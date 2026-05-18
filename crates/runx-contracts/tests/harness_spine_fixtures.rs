use serde::Deserialize;

use runx_contracts::{Act, GovernedActRef, HarnessReceipt, Signal};

const FIXTURES: &[&str] = &[
    include_str!("../../../fixtures/contracts/harness-spine/act-ref.json"),
    include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-abnormal.json"),
    include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-success.json"),
    include_str!("../../../fixtures/contracts/harness-spine/signal-fingerprint-links.json"),
    include_str!("../../../fixtures/contracts/harness-spine/verification-act.json"),
];

#[derive(Debug, Deserialize)]
struct Fixture {
    fixture_kind: FixtureKind,
    expected: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FixtureKind {
    Act,
    GovernedActRef,
    HarnessReceipt,
    Signal,
}

#[test]
fn harness_spine_fixtures_roundtrip() -> Result<(), serde_json::Error> {
    for fixture_json in FIXTURES {
        let fixture: Fixture = serde_json::from_str(fixture_json)?;
        assert_roundtrip(fixture)?;
    }
    Ok(())
}

#[test]
fn harness_receipt_rejects_unknown_governed_fields() {
    let mut fixture: Fixture = serde_json::from_str(include_str!(
        "../../../fixtures/contracts/harness-spine/harness-receipt-success.json"
    ))
    .expect("fixture parses");
    fixture.expected["harness"]["unexpected"] = serde_json::json!(true);

    let result = serde_json::from_value::<HarnessReceipt>(fixture.expected);

    assert!(result.is_err());
}

#[test]
fn governed_act_ref_requires_harness_receipt_context() {
    let value = serde_json::json!({
        "act_ref": {
            "act_id": "act_revision_1"
        }
    });

    let result = serde_json::from_value::<GovernedActRef>(value);

    assert!(result.is_err());
}

#[test]
fn provider_workflow_act_form_is_rejected() {
    let value = serde_json::json!({
        "act_id": "act_legacy",
        "form": "pull_request",
        "intent": {
            "purpose": "Do legacy work",
            "legitimacy": "Not admitted"
        },
        "summary": "Provider workflow names are not act forms",
        "closure": {
            "disposition": "blocked",
            "reason_code": "provider_workflow_form",
            "summary": "Provider workflow names are not act forms",
            "closed_at": "2026-05-18T00:00:00Z"
        },
        "source_refs": [],
        "target_refs": [],
        "surface_refs": [],
        "artifact_refs": [],
        "verification_refs": [],
        "harness_refs": [],
        "performed_at": "2026-05-18T00:00:00Z"
    });

    let result = serde_json::from_value::<Act>(value);

    assert!(result.is_err());
}

fn assert_roundtrip(fixture: Fixture) -> Result<(), serde_json::Error> {
    match fixture.fixture_kind {
        FixtureKind::Act => roundtrip::<Act>(fixture.expected),
        FixtureKind::GovernedActRef => roundtrip::<GovernedActRef>(fixture.expected),
        FixtureKind::HarnessReceipt => roundtrip::<HarnessReceipt>(fixture.expected),
        FixtureKind::Signal => roundtrip::<Signal>(fixture.expected),
    }
}

fn roundtrip<T>(expected: serde_json::Value) -> Result<(), serde_json::Error>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = serde_json::from_value(expected.clone())?;
    let actual = serde_json::to_value(parsed)?;
    assert_eq!(actual, expected);
    Ok(())
}
