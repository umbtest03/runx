---
spec_version: '2.0'
task_id: rust-ts-sunset-marketplaces
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: medium
---

# TS sunset: marketplaces

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Seventh TS sunset.
Blockers: `rust-ts-sunset-registry` complete, marketplaces consumers
re-routed.
Allowed follow-up command: `scafld harden rust-ts-sunset-marketplaces`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/core/src/marketplaces/` (245 lines per the architecture
doc's pure-by-imports inventory). The marketplaces domain is small; this
sunset is correspondingly small.

A new `rust-marketplaces-port` spec ships the Rust equivalent before this
sunset runs, or marketplaces folds into `runx-registry-client` if the
domain doesn't justify its own crate at port time.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-runtime` (or merged into registry)

Current TypeScript sources:
- `packages/core/src/marketplaces/**` (to be deleted)

Files impacted:
- `packages/core/src/marketplaces/` (deleted)
- `packages/core/src/index.ts`

Invariants:
- Marketplaces consumers (CLI surfaces, registry resolver, ai-search merge)
  have a Rust path.

## Objectives

- Enumerate importers; verify migration.
- Delete TS marketplaces.

## Scope

In scope:
- TS marketplaces deletion.

Out of scope:
- Marketplaces functionality changes.

## Dependencies

- `rust-ts-sunset-registry`.
- A `rust-marketplaces-port` spec (or merger into `rust-registry-client`).

## Open Questions

- Whether marketplaces ships as its own Rust module or folds into the
  registry client. Defer until `rust-registry-client` Phase 1 ingest.
