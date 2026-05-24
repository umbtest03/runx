//! Non-authoritative wire-compatibility gate for the type-driven JSON Schema
//! emitter (Phase 1 of `rust-contract-pipeline-inversion`).
//!
//! For each covered contract: the Rust-emitted schema must preserve schema
//! identity (`$id`, `x-runx-schema`) and agree with the committed
//! `oss/schemas/*.json` on accept/reject for every corpus value. The schema
//! *document* shape may differ from the committed one; only the validated value
//! domain must match (dod1). The committed TypeBox-generated schemas remain the
//! source of truth until the pipeline inversion flips.

use std::path::PathBuf;

use runx_contracts::artifact::Artifact;
use runx_contracts::doctor::DoctorReport;
use runx_contracts::redaction::Redaction;
use runx_contracts::reference::Reference;
use runx_contracts::schema::RunxSchema;
use serde_json::{Value, json};

struct Covered {
    file_name: &'static str,
    emitted: Value,
    corpus: Vec<(&'static str, Value)>,
}

fn covered() -> Vec<Covered> {
    vec![
        Covered {
            file_name: "reference.schema.json",
            emitted: Reference::json_schema(),
            corpus: reference_corpus(),
        },
        Covered {
            file_name: "doctor.schema.json",
            emitted: DoctorReport::json_schema(),
            corpus: doctor_corpus(),
        },
        Covered {
            file_name: "redaction.schema.json",
            emitted: Redaction::json_schema(),
            corpus: redaction_corpus(),
        },
        Covered {
            file_name: "artifact.schema.json",
            emitted: Artifact::json_schema(),
            corpus: artifact_corpus(),
        },
    ]
}

fn a_ref() -> Value {
    json!({ "type": "act", "uri": "runx:act:1" })
}

fn hash_commitment() -> Value {
    json!({ "algorithm": "sha256", "value": "abc", "canonicalization": "json-c14n" })
}

fn doctor_corpus() -> Vec<(&'static str, Value)> {
    let summary = json!({ "errors": 0, "warnings": 0, "infos": 0 });
    vec![
        (
            "minimal valid",
            json!({
                "schema": "runx.doctor.v1",
                "status": "success",
                "summary": summary,
                "diagnostics": [],
            }),
        ),
        (
            "full valid",
            json!({
                "schema": "runx.doctor.v1",
                "status": "failure",
                "summary": summary,
                "diagnostics": [{
                    "id": "d1",
                    "instance_id": "i1",
                    "severity": "warning",
                    "title": "t",
                    "message": "m",
                    "target": {},
                    "location": { "path": "p", "json_pointer": "/a" },
                    "evidence": { "e": 1 },
                    "repairs": [{
                        "id": "r1",
                        "kind": "edit_json",
                        "confidence": "high",
                        "risk": "low",
                        "path": "p",
                        "requires_human_review": false,
                    }],
                }],
            }),
        ),
        (
            "missing status",
            json!({ "schema": "runx.doctor.v1", "summary": summary, "diagnostics": [] }),
        ),
        (
            "missing summary",
            json!({ "schema": "runx.doctor.v1", "status": "success", "diagnostics": [] }),
        ),
        (
            "missing schema",
            json!({ "status": "success", "summary": summary, "diagnostics": [] }),
        ),
        (
            "unknown status variant",
            json!({
                "schema": "runx.doctor.v1",
                "status": "maybe",
                "summary": summary,
                "diagnostics": [],
            }),
        ),
        (
            "additional property",
            json!({
                "schema": "runx.doctor.v1",
                "status": "success",
                "summary": summary,
                "diagnostics": [],
                "bogus": true,
            }),
        ),
        (
            "diagnostic missing required field",
            json!({
                "schema": "runx.doctor.v1",
                "status": "failure",
                "summary": summary,
                "diagnostics": [{
                    "id": "d1",
                    "severity": "error",
                    "title": "t",
                    "message": "m",
                    "target": {},
                    "location": { "path": "p" },
                    "repairs": [],
                }],
            }),
        ),
        ("not an object", json!("nope")),
    ]
}

fn redaction_corpus() -> Vec<(&'static str, Value)> {
    let valid = json!({
        "schema": "runx.redaction.v1",
        "redaction_id": "red_1",
        "policy_ref": a_ref(),
        "redacted_fields": ["a", "b"],
        "hash_commitments": [hash_commitment()],
        "canonicalization": "json-c14n",
        "performed_by_ref": a_ref(),
        "performed_at": "2026-01-01T00:00:00Z",
    });
    vec![
        ("full valid", valid.clone()),
        (
            "minimal valid",
            json!({
                "schema": "runx.redaction.v1",
                "redaction_id": "red_1",
                "policy_ref": a_ref(),
                "redacted_fields": [],
                "hash_commitments": [],
                "canonicalization": "json-c14n",
                "performed_by_ref": a_ref(),
                "performed_at": "2026-01-01T00:00:00Z",
            }),
        ),
        ("missing schema", {
            let mut v = valid.clone();
            v.as_object_mut().unwrap().remove("schema");
            v
        }),
        ("missing redaction_id", {
            let mut v = valid.clone();
            v.as_object_mut().unwrap().remove("redaction_id");
            v
        }),
        ("empty redaction_id", {
            let mut v = valid.clone();
            v["redaction_id"] = json!("");
            v
        }),
        ("empty canonicalization", {
            let mut v = valid.clone();
            v["canonicalization"] = json!("");
            v
        }),
        ("empty redacted_fields item", {
            let mut v = valid.clone();
            v["redacted_fields"] = json!([""]);
            v
        }),
        ("malformed performed_at", {
            let mut v = valid.clone();
            v["performed_at"] = json!("not-a-timestamp");
            v
        }),
        ("additional property", {
            let mut v = valid.clone();
            v["bogus"] = json!(true);
            v
        }),
    ]
}

fn artifact_corpus() -> Vec<(&'static str, Value)> {
    let valid = json!({
        "schema": "runx.artifact.v1",
        "artifact_id": "art_1",
        "artifact_ref": a_ref(),
        "produced_by": { "receipt_ref": a_ref() },
        "media_type": "text/plain",
        "created_at": "2026-01-01T00:00:00Z",
        "size_bytes": 12,
        "hash": hash_commitment(),
        "redaction_refs": [],
        "source_refs": [],
    });
    vec![
        ("minimal valid", valid.clone()),
        ("full valid", {
            let mut v = valid.clone();
            v["produced_by"] = json!({
                "receipt_ref": a_ref(),
                "act_ref": { "receipt_ref": a_ref(), "act_id": "act_1" },
            });
            v["data_ref"] = a_ref();
            v["summary"] = json!("a summary");
            v["extensions"] = json!({ "k": 1 });
            v
        }),
        ("missing schema", {
            let mut v = valid.clone();
            v.as_object_mut().unwrap().remove("schema");
            v
        }),
        ("missing artifact_id", {
            let mut v = valid.clone();
            v.as_object_mut().unwrap().remove("artifact_id");
            v
        }),
        ("missing hash", {
            let mut v = valid.clone();
            v.as_object_mut().unwrap().remove("hash");
            v
        }),
        ("empty artifact_id", {
            let mut v = valid.clone();
            v["artifact_id"] = json!("");
            v
        }),
        ("empty media_type", {
            let mut v = valid.clone();
            v["media_type"] = json!("");
            v
        }),
        ("malformed created_at", {
            let mut v = valid.clone();
            v["created_at"] = json!("nope");
            v
        }),
        ("empty hash value", {
            let mut v = valid.clone();
            v["hash"] = json!({ "algorithm": "sha256", "value": "", "canonicalization": "c" });
            v
        }),
        ("additional property", {
            let mut v = valid.clone();
            v["bogus"] = json!(true);
            v
        }),
    ]
}

fn reference_corpus() -> Vec<(&'static str, Value)> {
    vec![
        (
            "minimal valid",
            json!({ "type": "github_issue", "uri": "runx:github_issue:1" }),
        ),
        (
            "full valid",
            json!({
                "type": "act",
                "uri": "runx:act:1",
                "provider": "github",
                "locator": "owner/repo#1",
                "label": "an act",
                "observed_at": "2026-01-01T00:00:00.000Z",
                "proof_kind": "payment_rail",
            }),
        ),
        (
            "optional schema marker",
            json!({ "schema": "runx.reference.v1", "type": "act", "uri": "x" }),
        ),
        ("missing uri", json!({ "type": "act" })),
        ("missing type", json!({ "uri": "x" })),
        (
            "unknown type variant",
            json!({ "type": "not_a_type", "uri": "x" }),
        ),
        ("empty uri", json!({ "type": "act", "uri": "" })),
        (
            "malformed observed_at",
            json!({ "type": "act", "uri": "x", "observed_at": "not-a-timestamp" }),
        ),
        (
            "additional property",
            json!({ "type": "act", "uri": "x", "bogus": true }),
        ),
        (
            "bad proof_kind",
            json!({ "type": "act", "uri": "x", "proof_kind": "wire" }),
        ),
    ]
}

fn committed_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas")
}

#[test]
fn emitted_schemas_are_wire_compatible_with_committed() {
    let dir = committed_dir();
    let mut failures: Vec<String> = Vec::new();

    for contract in covered() {
        let name = contract.file_name;
        let raw = match std::fs::read_to_string(dir.join(name)) {
            Ok(raw) => raw,
            Err(error) => {
                failures.push(format!("{name}: cannot read committed schema: {error}"));
                continue;
            }
        };
        let committed: Value = match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(error) => {
                failures.push(format!(
                    "{name}: committed schema is not valid JSON: {error}"
                ));
                continue;
            }
        };

        if contract.emitted.get("$id") != committed.get("$id")
            || contract.emitted.get("x-runx-schema") != committed.get("x-runx-schema")
        {
            failures.push(format!(
                "{name}: schema identity ($id / x-runx-schema) diverged"
            ));
            continue;
        }

        let Ok(committed_validator) = jsonschema::validator_for(&committed) else {
            failures.push(format!(
                "{name}: committed schema is not a usable validator"
            ));
            continue;
        };
        let Ok(emitted_validator) = jsonschema::validator_for(&contract.emitted) else {
            failures.push(format!("{name}: emitted schema is not a usable validator"));
            continue;
        };

        for (label, value) in &contract.corpus {
            let committed_accepts = committed_validator.is_valid(value);
            let emitted_accepts = emitted_validator.is_valid(value);
            if committed_accepts != emitted_accepts {
                failures.push(format!(
                    "{name} / {label}: committed accepts={committed_accepts}, emitted accepts={emitted_accepts}"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "schema wire-compat drift:\n{}",
        failures.join("\n")
    );
}
