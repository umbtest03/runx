---
spec_version: '2.0'
task_id: rust-runtime-adapters-catalog
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust runtime catalog adapter

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Catalog adapter
covers tool-catalog-driven execution paths.
Blockers: `rust-runtime-skeleton`, `rust-tool-catalogs` (catalog surface
contract).
Allowed follow-up command: `scafld harden rust-runtime-adapters-catalog`
Latest runner update: none
Review gate: not_started

## Summary

Port the `catalog` adapter family to `runx-runtime`. Catalog adapters
resolve and invoke tools published in the runx tool catalog
(`packages/runtime-local/src/tool-catalogs/*`) using deterministic
manifest-driven execution.

## Context

CWD: `.`

Packages:
- `@runxhq/runtime-local` (tool-catalogs)
- `@runxhq/adapters` (catalog subpath if present)
- `crates/runx-runtime`
- `crates/runx-contracts` (tools manifest)

Current TypeScript sources:
- `packages/runtime-local/src/tool-catalogs/**`
- `packages/contracts/src/schemas/tool-manifest.ts`

Files impacted:
- `crates/runx-runtime/src/adapters/catalog.rs`
- `crates/runx-runtime/tests/catalog_parity.rs`
- `fixtures/runtime/adapters/catalog/**`

Invariants:
- Tool manifests come from `runx-contracts::tools` types.
- Catalog resolution is deterministic given a manifest snapshot.
- No live network in fixture tests.

## Objectives

- Port catalog resolution and invocation.
- Add fixture suite covering local catalog, OCI catalog, and registry-
  backed catalog paths if present in TS.

## Scope

In scope:
- Catalog resolution + invocation under the adapter trait.

Out of scope:
- Catalog publishing (`runx tool build`); that's `rust-tool-catalogs`.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-tool-catalogs` (CLI surface; catalog producer side).

## Open Questions

- Whether OCI-backed catalogs require a Rust OCI client today, or whether
  CLI-shelled `oci` invocation suffices for v0.
