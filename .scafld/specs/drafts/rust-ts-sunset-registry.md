---
spec_version: '2.0'
task_id: rust-ts-sunset-registry
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# TS sunset: registry (core domain)

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Sixth TS sunset.
Blockers: `rust-ts-sunset-receipts` complete, `rust-registry-client`
consumed by every CLI surface.
Allowed follow-up command: `scafld harden rust-ts-sunset-registry`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/core/src/registry/`. The registry client now lives in
Rust (`rust-registry-client`); the TS core domain has no live readers.
This spec only touches the OSS-side TS package; cloud-side registry logic
stays put.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-runtime` (registry client lives here per
  `rust-registry-client` open question)
- `cloud/packages/api` (registry HTTP routes; not touched)

Current TypeScript sources:
- `packages/core/src/registry/**` (to be deleted)

Files impacted:
- `packages/core/src/registry/` (deleted)
- `packages/core/src/index.ts`

Invariants:
- Registry HTTP contract is unchanged.
- Trust tier semantics preserved cross-language.

## Objectives

- Enumerate importers; verify migration.
- Delete TS registry core domain.

## Scope

In scope:
- TS registry core deletion.

Out of scope:
- Cloud-side registry routes / logic.
- Registry signing / attestation hierarchy.

## Dependencies

- `rust-ts-sunset-receipts`.
- `rust-registry-client`.

## Open Questions

- None at draft time.
