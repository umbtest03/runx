use serde::Deserialize;

use runx_contracts::{
    CredentialDeliveryObservation, CredentialDeliveryProfile, CredentialDeliveryRequest,
    CredentialDeliveryResponse,
};

const FIXTURES: &[&str] = &[
    include_str!("../../../fixtures/contracts/credential-delivery/response.json"),
    include_str!("../../../fixtures/contracts/credential-delivery/observation.json"),
    include_str!("../../../fixtures/contracts/credential-delivery/profile.json"),
    include_str!("../../../fixtures/contracts/credential-delivery/request.json"),
];

#[derive(Debug, Deserialize)]
struct Fixture {
    fixture_kind: FixtureKind,
    expected: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum FixtureKind {
    #[serde(rename = "credential_delivery_response")]
    Response,
    #[serde(rename = "credential_delivery_observation")]
    Observation,
    #[serde(rename = "credential_delivery_profile")]
    Profile,
    #[serde(rename = "credential_delivery_request")]
    Request,
}

#[test]
fn credential_delivery_fixtures_match_typescript_wire_shapes() -> Result<(), serde_json::Error> {
    for fixture_json in FIXTURES {
        let fixture: Fixture = serde_json::from_str(fixture_json)?;
        assert_roundtrip(fixture)?;
    }
    Ok(())
}

#[test]
fn credential_delivery_public_frames_reject_raw_secret_material() -> Result<(), serde_json::Error> {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../fixtures/contracts/credential-delivery/response.json"
    ))?;
    let mut response = fixture.expected;
    response["api_key"] = serde_json::Value::String("super-secret-token".to_owned());

    let result = serde_json::from_value::<CredentialDeliveryResponse>(response);

    assert!(result.is_err());
    Ok(())
}

fn assert_roundtrip(fixture: Fixture) -> Result<(), serde_json::Error> {
    match fixture.fixture_kind {
        FixtureKind::Response => roundtrip::<CredentialDeliveryResponse>(fixture.expected),
        FixtureKind::Observation => roundtrip::<CredentialDeliveryObservation>(fixture.expected),
        FixtureKind::Profile => roundtrip::<CredentialDeliveryProfile>(fixture.expected),
        FixtureKind::Request => roundtrip::<CredentialDeliveryRequest>(fixture.expected),
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
