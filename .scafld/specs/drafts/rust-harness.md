---
spec_version: '2.0'
task_id: rust-harness
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T14:06:29Z'
status: draft
harden_status: in_progress
size: medium
risk_level: medium
---

# Rust harness replay

## Current State

Status: draft
Current phase: none
Next: harden
Reason: external hardening provider running
Blockers: provider hardening not yet recorded
Allowed follow-up command: `scafld harden rust-harness --provider <provider>`
Latest runner update: none
Review gate: not_started

## Summary

Port the skill harness replay runner to Rust. `runx harness <path>` runs a
skill inside a deterministic fixture-backed governed harness and asserts the
sealed harness receipt plus output match expectations. It is not a second
meaning of `harness`; it is replay mode for the contract spine ratified in
`runx-contract-spine-hard-cutover`.

Today this lives in `packages/runtime-local/src/harness/runner.ts` plus
harness/quality.ts and harness/framing-patterns.ts.

The existing CLI verb stays. The conceptual framing changes:

- production mode: a harness runs against live adapters and seals to a receipt
- replay mode: the same harness contract runs against fixtures and asserts the
  sealed receipt/output
- publish verification: replay mode proves a skill has at least one
  deterministic governed example before publication

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (`runx harness` dispatch)
- `@runxhq/runtime-local` (current harness replay implementation)
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/runtime-local/src/harness/runner.ts`
- `packages/runtime-local/src/harness/quality.ts`
- `packages/runtime-local/src/harness/framing-patterns.ts`
- `packages/runtime-local/src/harness/mcp-fixture.ts`
- `packages/runtime-local/src/harness/a2a-fixture.ts`

Files impacted:
- `crates/runx-cli/src/launcher.rs`
- `crates/runx-runtime/src/harness/runner.rs`
- `crates/runx-runtime/src/harness/quality.rs`
- `crates/runx-runtime/src/harness/fixtures.rs`
- `fixtures/harness/**`

Invariants:
- Replay harness runs are deterministic: fixtures stand in for live adapters.
- Quality checks match TS thresholds and rubrics.
- The Rust replay path emits the same canonical harness receipt shape as the
  contract spine.
- Harness receipts are byte-identical for the same canonical harness input
  within one contract shape.
- Byte-identical comparison across the Rust cutover is measured after the
  contract spine hard cutover, not against retired receipt shapes.
- Byte-identical means canonical JSON serialization: stable key order,
  normalized timestamps/ids in fixtures, deterministic array order where order
  is semantically relevant, and exact hash inputs shared with `runx-receipts`.
- Replay mode must exercise the same authority attenuation, act containment,
  decision containment, seal, and verification semantics as production mode
  where the fixture can represent them.

## Objectives

- Port harness replay runner, quality checks, fixture loaders for MCP and a2a.
- Match the post-cutover TS canonical harness receipt JSON output.
- Keep `runx harness <path>` as the CLI surface; do not add a parallel replay
  verb.
- Update the Rust CLI launcher so `runx harness <path>` can dispatch to the
  native replay runner once this spec is the active implementation path.
- Ensure fixture replay produces or validates the canonical harness receipt,
  contained decision payloads, contained act payloads, child harness receipt
  refs, and verification proof.

## Scope

In scope:
- Harness replay runner and fixture infrastructure.
- Fixture-to-harness expansion for skills, inline harness cases, MCP, A2A, and
  cli-tool profiles.
- Post-cutover fixture refresh for `fixtures/harness/**` so old receipt fields
  are replaced by canonical harness receipt payloads.
- Receipt equality checks against canonical post-cutover harness receipts.
- Quality/framing checks as verification effects or harness receipt checks
  where applicable.

Out of scope:
- Harness authoring (`write-harness` skill belongs in a separate authoring
  pass).
- Live production harness scheduling beyond what is required to share the
  canonical runtime path.

## Dependencies

- `rust-runtime-skeleton`.
- `runx-contract-spine-hard-cutover` for canonical harness, act, decision,
  signal, authority, and harness receipt shapes.
- `rust-runtime-adapters-{agent,a2a,mcp}` for adapter-specific fixture
  formats; harness can ship with cli-tool-only initially and gain coverage
  as adapters land.

Sequencing:

- `rust-harness` implementation is blocked until the implementation phase of
  `runx-contract-spine-hard-cutover` has landed the canonical harness receipt
  shape, fixture shape, and serialization rules.
- If the contract spine lands before Rust runtime cutover, `rust-harness`
  ports against the ratified harness receipt shape and byte-identical means
  TS-post-cutover versus Rust.
- If Rust runtime cutover lands first, this spec must not claim byte-identical
  receipt preservation across the later contract cutover. It may preserve the
  old TS harness runner only until the hard cutover replaces the shape.
- The preferred sequence is contract spine hard cutover first, then
  `rust-harness` against the final canonical shape.
- Existing pre-cutover fixtures are upgraded by the contract spine cutover.
  `rust-harness` consumes and validates the upgraded fixtures; it does not
  preserve the old fixture receipt fields.

## Open Questions

- Whether harness JSON output stays identical to post-cutover TS canonical
  output or gets a Rust-side cleanup. Default: identical; cleanup is a separate
  spec if motivated.
- Which fixture fields become harness authority algebra fixtures versus
  replay-only convenience inputs.
- How much of production abnormal seal behavior must be fixture-testable in
  the first Rust replay slice.

## Harden Rounds

### round-1

Status: failed
Started: 2026-05-18T13:56:58Z
Ended: 2026-05-18T13:56:58Z
Verdict: needs_revision
Provider: gemini
Model: gemini-3-flash-preview
Output format: gemini.mcp_submit_harden
Summary: I have audited the rust-harness spec and identified a critical sequencing risk. The spec intends to port against a 'post-cutover' contract model that is itself still in draft. Implementing the harness before the contract spine is locked risks significant rework. Additionally, I've identified missing touchpoints in the Rust CLI and inconsistencies in existing fixtures. The spec is high-quality but needs revision to align with the broader Rust takeover sequencing.

Checks:
- path audit
  - Grounded in: packages/runtime-local/src/harness/
  - Result: passed
  - Evidence: Verified existence of runner.ts, quality.ts, etc.
- command audit
  - Grounded in: crates/runx-cli/src/launcher.rs:24
  - Result: passed
  - Evidence: runx harness <path> is intended, but crates/runx-cli needs to handle it.
- scope/migration audit
  - Grounded in: rust-harness.md:92
  - Result: passed
  - Evidence: Scope is clearly limited to replay mode.
- acceptance timing audit
  - Grounded in: runx-contract-spine-hard-cutover.md
  - Result: failed
  - Evidence: Acceptance depends on a draft contract spine that hasn't landed.
- rollback/repair audit
  - Grounded in: crates/runx-cli/src/launcher.rs
  - Result: passed
  - Evidence: TS implementation remains authoritative and usable via delegation if port fails.
- design challenge
  - Grounded in: crates/runx-runtime/src/lib.rs
  - Result: passed
  - Evidence: The move to native Rust replay is architecturally sound but foundationally blocked.

Issues:
- [high/blocks approval] `harden-1` spec_gap:dependencies - Sequencing dependency on draft contract spine.
  - Status: open
  - Grounded in: .scafld/specs/drafts/runx-contract-spine-hard-cutover.md
  - Evidence: runx-contract-spine-hard-cutover.md is a draft; rust-harness.md:108 acknowledges this dependency.
  - Recommendation: Explicitly block rust-harness implementation on the implementation phase of runx-contract-spine-hard-cutover to ensure a stable target.
- [medium/advisory] `harden-2` code_gap:integration - Rust CLI integration is missing from the spec.
  - Status: open
  - Grounded in: crates/runx-cli/src/launcher.rs:24
  - Evidence: crates/runx-cli/src/launcher.rs delegates all commands.
  - Recommendation: Add crates/runx-cli to the list of impacted files; update plan_launcher to intercept the harness command.
- [medium/advisory] `harden-3` spec_gap:fixtures - Existing fixtures reflect the old model, not the target model.
  - Status: open
  - Grounded in: fixtures/harness/echo-skill.yaml
  - Evidence: fixtures/harness/echo-skill.yaml contains old receipt fields like skill_name and source_type.
  - Recommendation: Clarify whether existing fixtures will be updated as part of the contract cutover or if rust-harness is responsible for the upgrade.
- [low/advisory] `harden-4` spec_gap:verification - Byte-identical requirement lacks serialization standard.
  - Status: open
  - Grounded in: rust-harness.md:73
  - Evidence: The spec requires byte-identical receipts.
  - Recommendation: Define the canonical serialization requirement (e.g. key order) to ensure byte-parity is achievable across runtimes.

### round-2

Status: in_progress
Started: 2026-05-18T14:06:29Z
Ended: none

Checks:
- none

Issues:
- none


