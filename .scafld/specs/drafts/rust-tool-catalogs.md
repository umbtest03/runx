---
spec_version: '2.0'
task_id: rust-tool-catalogs
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust tool catalogs

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Covers `runx tool
build / search / inspect`.
Blockers: `rust-runtime-skeleton`, `rust-contracts-parity` (tools shapes).
Allowed follow-up command: `scafld harden rust-tool-catalogs`
Latest runner update: none
Review gate: not_started

## Summary

Port the tool-catalog producer surface to Rust. `runx tool build` produces
a tool manifest from sources; `runx tool search` and `runx tool inspect`
read from catalogs and registries. The runtime adapter (catalog) consumes
the same manifest shape.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (tool command)
- `@runxhq/runtime-local` (tool-catalogs)
- `crates/runx-runtime`
- `crates/runx-contracts` (tools manifest)

Current TypeScript sources:
- `packages/cli/src/commands/tool.ts`
- `packages/runtime-local/src/tool-catalogs/**`

Files impacted:
- `crates/runx-runtime/src/tool_catalogs/build.rs`
- `crates/runx-runtime/src/tool_catalogs/search.rs`
- `crates/runx-runtime/src/tool_catalogs/inspect.rs`
- `fixtures/tool-catalogs/**`

Invariants:
- Tool manifest schema is owned by `runx-contracts::tools`.
- Build is deterministic given inputs.
- Search / inspect outputs are byte-identical to TS for the same catalog
  snapshot.

## Objectives

- Port tool build (manifest emission).
- Port tool search (catalog + registry-backed).
- Port tool inspect (manifest read + presentation).

## Scope

In scope:
- Tool surface end to end.

Out of scope:
- Adapter-side catalog invocation (covered by
  `rust-runtime-adapters-catalog`).

## Dependencies

- `rust-runtime-skeleton`.
- `rust-contracts-parity`.

## Open Questions

- Whether OCI manifest emission requires a Rust OCI library at port time.
  Default: no; CLI-shell into `oci` if needed.
