use serde::Deserialize;

use runx_contracts::{
    FeedEntry, Opportunity, ReflectionEntry, Selection, SelectionCycle, SkillBinding, Target,
    TargetTransitionEntry, ThesisAssessment,
};

const FIXTURE: &str =
    include_str!("../../../fixtures/contracts/aster-control/public-feed-proof.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    fixture_kind: FixtureKind,
    expected: AsterControlSet,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FixtureKind {
    AsterControlSet,
}

#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct AsterControlSet {
    feed_entry: FeedEntry,
    opportunity: Opportunity,
    reflection_entry: ReflectionEntry,
    selection: Selection,
    selection_cycle: SelectionCycle,
    skill_binding: SkillBinding,
    target: Target,
    target_transition_entry: TargetTransitionEntry,
    thesis_assessment: ThesisAssessment,
}

#[test]
fn aster_control_fixture_roundtrips() -> Result<(), serde_json::Error> {
    let fixture: Fixture = serde_json::from_str(FIXTURE)?;
    match fixture.fixture_kind {
        FixtureKind::AsterControlSet => {
            let actual = serde_json::to_value(&fixture.expected)?;
            let envelope: serde_json::Value = serde_json::from_str(FIXTURE)?;
            assert_eq!(actual, envelope["expected"]);
        }
    }
    Ok(())
}

#[test]
fn feed_entry_rejects_unknown_governed_fields() {
    let mut envelope: serde_json::Value = serde_json::from_str(FIXTURE).expect("fixture parses");
    envelope["expected"]["feed_entry"]["unexpected"] = serde_json::json!(true);

    let result = serde_json::from_value::<AsterControlSet>(envelope["expected"].clone());

    assert!(result.is_err());
}
