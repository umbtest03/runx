use std::fs;
use std::path::PathBuf;

use runx_runtime::{DoctorOptions, run_doctor};

const DOCTOR_FIXTURES: &[&str] = &[
    "cross-package-reach-in",
    "empty-success",
    "file-budget-exceeded",
    "removed-tool-yaml",
    "skill-fixture-missing",
    "tool-fixture-missing",
];

#[test]
fn doctor_runtime_matches_all_fixture_reports() -> Result<(), Box<dyn std::error::Error>> {
    let fixture_root = fixture_root();
    for fixture_name in DOCTOR_FIXTURES {
        let case_root = fixture_root.join(fixture_name);
        let expected_json = fs::read_to_string(case_root.join("expected.json"))?;
        let expected: serde_json::Value = serde_json::from_str(&expected_json)?;

        let report = run_doctor(&case_root.join("workspace"), &DoctorOptions)?;
        let actual = serde_json::to_value(report)?;

        assert_eq!(actual, expected, "doctor fixture {fixture_name}");
    }
    Ok(())
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("doctor")
}
