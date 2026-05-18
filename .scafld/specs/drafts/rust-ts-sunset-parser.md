---
spec_version: '2.0'
task_id: rust-ts-sunset-parser
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# TS sunset: parser

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Third TS sunset.
Blockers: `rust-ts-sunset-policy` complete.
Allowed follow-up command: `scafld harden rust-ts-sunset-parser`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/core/src/parser/`. By the time this spec runs, the Rust
runtime parses skills, graphs, and execution profiles, and no live TS
consumer reads from `@runxhq/core/parser`.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-parser`
- Every TS package that imports from `@runxhq/core/parser`

Current TypeScript sources:
- `packages/core/src/parser/**` (to be deleted)
- `packages/core/src/index.ts`
- All TS importers (enumerated in Phase 1)

Files impacted:
- `packages/core/src/parser/` (deleted)
- `packages/core/src/index.ts`

Invariants:
- Parsed AST shape consumers (authoring, marketplaces, registry, executor)
  have either migrated to Rust or are themselves sunsetting.

## Objectives

- Enumerate importers; verify migration.
- Delete TS parser implementation.

## Scope

In scope:
- TS parser deletion.

Out of scope:
- Authoring tools that consume parsed AST today (their own sunset path).

## Dependencies

- `rust-ts-sunset-policy`.
- `rust-parser-parity` complete and consumed.

## Open Questions

- None at draft time.
