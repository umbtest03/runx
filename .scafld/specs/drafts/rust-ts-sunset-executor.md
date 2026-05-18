---
spec_version: '2.0'
task_id: rust-ts-sunset-executor
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# TS sunset: executor

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Fourth TS sunset.
Blockers: `rust-ts-sunset-parser` complete.
Allowed follow-up command: `scafld harden rust-ts-sunset-executor`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/core/src/executor/`. The executor today defines
`ApprovalGate` and related types; once the Rust runtime is authoritative
and `runx-contracts::approval` owns the cross-language contract, the TS
executor can be removed.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-contracts` (approval types live here)
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/core/src/executor/index.ts` (to be deleted)
- All TS importers of `@runxhq/core/executor`

Files impacted:
- `packages/core/src/executor/` (deleted)
- `packages/core/src/index.ts`

Invariants:
- `ApprovalGate` type is preserved on the Rust side under
  `runx-contracts::approval`; TS importers either migrated or sunsetted.
- No new approval semantics introduced in this spec.

## Objectives

- Enumerate importers; verify migration.
- Delete TS executor.

## Scope

In scope:
- TS executor deletion.

Out of scope:
- Approval contract changes.

## Dependencies

- `rust-ts-sunset-parser`.
- `rust-approval-gate-parity`.

## Open Questions

- None at draft time.
