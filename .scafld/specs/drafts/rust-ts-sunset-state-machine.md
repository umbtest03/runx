---
spec_version: '2.0'
task_id: rust-ts-sunset-state-machine
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# TS sunset: state-machine

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. First TS sunset; the
template for subsequent sunsets.
Blockers: `rust-cli-rust-cutover` complete, every TS consumer of
`@runxhq/core` state-machine identified and either re-routed to
`runx-core::state_machine` (via the Rust runtime) or itself sunset.
Allowed follow-up command: `scafld harden rust-ts-sunset-state-machine`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/core/src/state-machine/`. By the time this spec runs, the
Rust runtime is authoritative for skill and graph execution and no live TS
consumer reads from `@runxhq/core/state-machine`. This spec verifies that
claim, removes the TS implementation, and updates the boundary check.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-core`
- Every TS package that imports from `@runxhq/core/state-machine`

Current TypeScript sources:
- `packages/core/src/state-machine/**` (to be deleted)
- `packages/core/src/index.ts` (re-export removed)
- All TS importers (enumerated in Phase 1 ingest via grep)

Files impacted:
- `packages/core/src/state-machine/` (deleted)
- `packages/core/src/index.ts`
- `oss/scripts/check-boundaries.mjs` (remove from `pureCoreDomains`)
- Every TS file currently importing state-machine

Invariants:
- No consumer regression. Phase 1 ingest enumerates every importer;
  Phase 2 verifies each has been re-routed.
- Rust runtime is the only authoritative state-machine evaluator after
  this spec.
- Receipts produced before and after deletion remain verifiable.

## Objectives

- Enumerate every TS importer of `@runxhq/core/state-machine`.
- Verify each importer has been re-routed via the Rust runtime or is
  itself going away.
- Delete the TS state-machine implementation.
- Update the boundary check.

## Scope

In scope:
- TS state-machine deletion.

Out of scope:
- Any consumer migration work (that lives in the consumer's own port
  spec).

## Dependencies

- `rust-cli-rust-cutover` complete.
- All Rust-runtime adapter specs that produce state-machine transitions.

## Open Questions

- Whether `@runxhq/core` survives as a shrinking shim package or gets
  retired entirely. Likely shim through subsequent sunsets, then deleted
  when empty.
