---
spec_version: '2.0'
task_id: rust-ts-sunset-policy
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# TS sunset: policy

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Second TS sunset.
Blockers: `rust-ts-sunset-state-machine` complete (sunsets run serially per
section 12 of the architecture doc).
Allowed follow-up command: `scafld harden rust-ts-sunset-policy`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/core/src/policy/`. By the time this spec runs, the Rust
runtime owns policy evaluation and no live TS consumer reads from
`@runxhq/core/policy`. Mirrors the `rust-ts-sunset-state-machine` template.
This does not delete contract schemas such as `runx.operational_policy.v1`;
those remain in `@runxhq/contracts` until the contract package itself has a
separate Rust ownership/sunset path.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-core`
- Every TS package that imports from `@runxhq/core/policy`

Current TypeScript sources:
- `packages/core/src/policy/**` (to be deleted)
- `packages/core/src/index.ts` (re-export removed)
- All TS importers (enumerated in Phase 1 ingest via grep)

Files impacted:
- `packages/core/src/policy/` (deleted)
- `packages/core/src/index.ts`
- `oss/scripts/check-boundaries.mjs` (remove from `pureCoreDomains`)

Invariants:
- No policy decision regressions: cross-validated through receipt parity
  before and after.
- Operational policy fixtures and `runx policy inspect|lint` keep validating
  against the same schema/readback shape after the implementation moves.
- The authority-proof helpers in `packages/core/src/policy/authority-proof.ts`
  have a Rust replacement (`rust-policy-authority-proof-parity` provides it).

## Objectives

- Enumerate importers; verify migration.
- Delete TS policy implementation.
- Update boundary check.

## Scope

In scope:
- TS policy deletion.

Out of scope:
- Consumer migration (lives in each consumer's port spec).

## Dependencies

- `rust-ts-sunset-state-machine`.
- `rust-policy-authority-proof-parity`.

## Open Questions

- None at draft time.
