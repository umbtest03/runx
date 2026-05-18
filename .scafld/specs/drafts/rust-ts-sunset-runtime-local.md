---
spec_version: '2.0'
task_id: rust-ts-sunset-runtime-local
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: large
risk_level: very_high
---

# TS sunset: runtime-local

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Final TS sunset; the
single largest package to retire.
Blockers: `rust-ts-sunset-marketplaces` complete, every adapter spec
(`rust-runtime-adapters-{agent,catalog,a2a,mcp}`) complete, MCP server +
harness + dev + journal-local + connect + scaffold + tool-catalogs +
doctor all consumed.
Allowed follow-up command: `scafld harden rust-ts-sunset-runtime-local`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/runtime-local/` and `packages/adapters/` in one
coordinated sunset. These two packages are consumed as a unit by every
caller; runtime-local imports adapters, and both retire as adapter logic
lands in `runx-runtime::adapters::{cli_tool, agent, catalog, a2a, mcp}`
via the four adapter port specs.

This is the last big rip. After it lands, the Rust takeover is complete
for OSS purposes; cloud-side hosted surfaces (`agent-runner`, `worker`,
`api`, `auth`) remain TS unless and until separate cloud cutover specs
target them. The disposition of every remaining TS package is documented
in `rust-ts-interop-boundary`.

## Context

CWD: `.`

Packages:
- `@runxhq/runtime-local`
- `@runxhq/adapters`
- `crates/runx-runtime`
- Every TS importer of `@runxhq/runtime-local` or `@runxhq/adapters`

Current TypeScript sources:
- `packages/runtime-local/**` (to be deleted)
- `packages/adapters/**` (to be deleted)

Files impacted:
- `packages/runtime-local/` (deleted)
- `packages/adapters/` (deleted)
- `pnpm-workspace.yaml` (remove workspace members)
- Every TS file currently importing from `@runxhq/runtime-local` or
  `@runxhq/adapters`

Invariants:
- Every importer either re-routed to Rust (via CLI subprocess, in-process
  binding, or `runx-runtime-service` daemon) or is itself sunset.
- The cloud `agent-runner` package has a stable boundary against the Rust
  runtime (settled in `rust-aster-runtime-cutover`).

## Objectives

- Enumerate every importer (large list; Phase 1 produces it).
- Verify migration for each.
- Delete `packages/runtime-local/`.
- Update workspace config.

## Scope

In scope:
- TS runtime-local deletion.

Out of scope:
- Any runtime feature change.
- Cloud-side TS package deletions (their own specs).

## Dependencies

- `rust-ts-sunset-marketplaces`.
- Every runtime adapter spec complete.
- Every CLI surface spec complete.

## Open Questions

- Whether the workspace retains an empty `@runxhq/runtime-local` shim for
  external consumers depending on the npm package, or fully removes it.
  Defer to Phase 1 ingest of npm consumer impact.
