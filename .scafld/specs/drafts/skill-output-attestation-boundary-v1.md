---
spec_version: '2.0'
task_id: skill-output-attestation-boundary-v1
created: '2026-05-25T17:51:35+10:00'
updated: '2026-05-25T17:51:35+10:00'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# skill-output-attestation-boundary-v1

## Current State

Status: draft
Current phase: ready for execution
Next: implement the supervisor-attested fact boundary for skill outputs.
Reason: `runx-security-hardening-v1` closed payment proof by requiring a
runtime supervisor, but non-payment output facts can still be skill-asserted and
then promoted into structured outputs, transition gates, receipt references, and
downstream policy decisions.
Blockers: none.
Allowed follow-up command: `scafld exec skill-output-attestation-boundary-v1`
Latest runner update: 2026-05-25T17:51:35+10:00
Review gate: not_started

## Summary

Separate what a skill says from what the harness can attest. Skill stdout may
remain a claimed payload, but policy decisions, receipt references, and
authority-sensitive gates must consume supervisor-attested facts or explicitly
typed skill claims that cannot masquerade as facts.

## Scope

In scope:
- `runx-runtime` skill output parsing and receipt sealing.
- Transition gates that currently read skill-produced structured output.
- Receipt reference collection from skill payloads.
- Fixture updates proving skill-asserted references do not become attested
  facts without supervisor evidence.

Out of scope:
- Payment rail proof; already handled by the payment supervisor.
- Changing the skill author subprocess ABI except where an output field is
  reclassified from attested fact to skill claim.

## Acceptance

Profile: strict

Definition of done:
- [ ] `dod1` Skill-produced stdout is stored as a claim, not a supervisor fact.
- [ ] `dod2` Receipt refs used for proof are supervisor-attested or explicitly
  labeled as skill claims.
- [ ] `dod3` Policy/transition gates do not trust arbitrary skill JSON as
  authority-sensitive facts.
- [ ] `dod4` Regression fixtures show malicious stdout cannot inject proof refs
  or satisfy a gated fact.

Validation:
- [ ] `v1` `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test skill_run`
- [ ] `v2` `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test receipt_refs`
- [ ] `v3` focused grep review for `output_object`, `transition_field_value`,
  and `collect_payload_refs` documents the final trust boundary.

## Review

Reject any patch that merely renames output fields while still letting
attacker-controlled skill stdout satisfy receipt proof, policy, or transition
facts.
