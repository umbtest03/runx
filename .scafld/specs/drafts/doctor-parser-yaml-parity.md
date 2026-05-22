---
spec_version: '2.0'
task_id: doctor-parser-yaml-parity
created: '2026-05-22T00:00:00Z'
updated: '2026-05-22T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: medium
---

# Doctor / parser YAML strictness parity

## Current State

Status: draft
Current phase: design only; the harness-status half landed, the YAML half is
pending
Next: align the Rust YAML parse strictness with the canonical parser, or add a
doctor lint for the ambiguous constructs, so `runx doctor` rejects what publish
rejects
Reason: seeding the catalog surfaced that `runx doctor` accepted skills the
canonical publish parser rejected, so authors hit the failure only at publish.
Blockers: the code fix touches `runx-parser`, which is under a concurrent
stringly-to-enum refactor (`SourceKind`/`InputMode`); sequence this after that
lands to avoid conflict.
Review gate: not_started

## Why this exists

Two parsers must agree: the Rust `runx-parser` (used by `runx doctor` and local
execution, via `serde_norway`) and the canonical publish parser. Seeding found
two divergences where doctor was lenient and publish was strict:

1. A mapping key with an embedded colon, e.g. `email:send:`, which the canonical
   parser reads as a compact nested mapping and rejects, but `serde_norway`
   accepted.
2. A colon-space inside an unquoted scalar, e.g.
   `message: ... (granted: repo.read)`, rejected by the canonical parser,
   accepted by `serde_norway`.

Both were fixed in the affected skills by quoting, but the parser divergence
remains: a future skill with either construct passes `runx doctor` and fails at
publish.

## What already landed (2026-05-22)

`runx doctor` now parses and validates each skill's runner manifest (via
`runx-parser::parse_runner_manifest_yaml` + `validate_runner_manifest`),
emitting `runx.skill.profile.invalid`. This closed the harness-status half of
the gap: an invalid `expect.status` (e.g. `success`) is now caught by doctor
before publish, with a unit test in `runx-runtime/src/doctor.rs`. This spec
covers only the remaining YAML-strictness half.

## Scope

- IN: make the Rust YAML parse reject the two ambiguous constructs above (so
  doctor and publish agree), or, if the YAML library cannot be made strict
  cleanly, add a doctor lint (`runx.skill.profile.ambiguous_yaml`) that flags
  embedded-colon keys and colon-space-in-plain-scalar before publish.
- OUT: the harness-status validation (already landed); changing existing skills
  (already quoted); the canonical publish parser (it is the strict reference).

## Touchpoints

- oss/crates/runx-parser (the `serde_norway` parse path)
- oss/crates/runx-runtime/src/doctor.rs (a lint diagnostic, if the lint route)

## Acceptance

Definition of done:

- A skill X.yaml with an embedded-colon mapping key fails `runx doctor` (parity
  with publish).
- A skill X.yaml with a colon-space in an unquoted scalar fails `runx doctor`.
- All currently-seeded skills still pass `runx doctor` (no false positives).
- A test locks both rejections.

## References

- oss/crates/runx-runtime/src/doctor.rs (`validate_skill_profile`, the landed
  harness-status half)
- oss/crates/runx-parser/src/runner.rs (`parse_runner_manifest_yaml`)
- the canonical publish parser's harness-status error
  ("must be sealed, failure, needs_agent, policy_denied, or escalated")
