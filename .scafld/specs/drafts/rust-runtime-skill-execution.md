---
spec_version: '2.0'
task_id: rust-runtime-skill-execution
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T14:04:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust runtime skill execution

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. First real skill end
to end on Rust runtime; the credibility anchor before adapter expansion.
Blockers: `rust-runtime-skeleton` complete.
Allowed follow-up command: `scafld harden rust-runtime-skill-execution`
Latest runner update: none
Review gate: not_started

## Summary

Execute two production-shaped skills end to end on `runx-runtime` with
the `cli-tool` adapter: `oss/skills/issue-to-pr` and
`oss/skills/issue-intake`. Both ship `SKILL.md` plus execution profile.
`issue-intake` is the additional anchor because nitrosend's production
deployment depends on it (see `rust-nitrosend-dogfood`); proving it
runs on Rust runtime is a direct prerequisite for the nitrosend
cutover.

The receipt that Rust runtime emits must verify against
`runx-receipts::verify`, match the post-cutover TS canonical harness receipt
shape, and pass the existing TS skill harness assertions.

## Context

CWD: `.`

Packages:
- `@runxhq/runtime-local`
- `@runxhq/adapters` (cli-tool)
- `crates/runx-runtime`
- `oss/skills/issue-to-pr` (real artifact)

Current TypeScript sources:
- `packages/runtime-local/src/runner-local/index.ts`
- `packages/runtime-local/src/harness/runner.ts`

Files impacted:
- `crates/runx-runtime/tests/skill_issue_to_pr.rs`
- `crates/runx-runtime/tests/skill_issue_intake.rs`
- `fixtures/runtime/skills/issue-to-pr/**`
- `fixtures/runtime/skills/issue-intake/**`
- `scripts/generate-rust-skill-fixtures.ts`
- `oss/skills/issue-to-pr/{SKILL.md,X.yaml}` (not modified; consumed)
- `oss/skills/issue-intake/{SKILL.md,X.yaml}` (not modified; consumed)
- `crates/runx-contracts` harness, signal, decision, act, artifact,
  verification, Reference, and harness receipt contracts

Invariants:
- The skill source is not modified to make the Rust run easier.
- The fixture is generated from a fully deterministic input (mocked
  external calls; no live network).
- Receipt parity with the post-cutover TS runner is byte-identical modulo
  documented non-deterministic fields.
- The `issue-intake` fixture preserves the production intake behavior using
  signal refs, evidence refs, artifact refs, contained decisions, contained
  acts, and harness receipt proof. Retired issue-control payload names are not
  preserved.

## Objectives

- Run `oss/skills/issue-to-pr` to a green receipt on Rust runtime.
- Run `oss/skills/issue-intake` to a green receipt on Rust runtime
  (nitrosend production dependency).
- Include the current issue-intake harness shape with signal, evidence ref,
  artifact, decision, act, and verification context so the Rust runtime proves
  it can execute the production intake contract, not an older thin issue-only
  input.
- Document the deterministic harness setup so subsequent skills can be
  added without per-skill scaffolding.
- Add a skill-execution test pattern that other skill ports follow.

## Scope

In scope:
- `issue-to-pr` and `issue-intake` end-to-end execution.
- Deterministic harness configuration (mocked github, mocked subprocess
  outputs).

Out of scope:
- Adding more skills beyond the two anchors. Other skills become opt-in
  follow-up specs.
- Live network calls.
- Approval-gated steps within the skill flows (covered by
  `rust-approval-gate-parity`).
- Nitrosend's wrapper layer (workflow, policy file, slash command
  parsing). That lives in `rust-nitrosend-dogfood`; this spec only
  proves the underlying skill execution.

## Dependencies

- `rust-runtime-skeleton`.
- `runx-contract-spine-hard-cutover` approved and `rust-receipts-parity`
  reframed to post-cutover harness receipts before the `issue-intake` fixture
  claims typed harness/signal/decision/act/receipt parity. If this runtime spec
  executes first, the fixture may only prove old TS behavior and must be rerun
  after the hard cutover before it can gate the Rust launcher flip.
- The skill's `X.yaml` must remain stable; any change to it during this
  spec triggers an explicit fixture refresh.

## Open Questions

- Whether the skill harness lives under `crates/runx-runtime/tests` or
  spawns a `runx-harness`-equivalent helper crate. Default: tests/ for
  this spec; helper-crate decision deferred to `rust-harness` spec.
