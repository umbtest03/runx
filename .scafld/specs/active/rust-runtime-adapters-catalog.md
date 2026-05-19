---
spec_version: '2.0'
task_id: rust-runtime-adapters-catalog
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T08:25:22Z'
status: active
harden_status: passed
size: medium
risk_level: medium
---

# Rust runtime catalog adapter

## Current State

Status: active
Current phase: final
Next: build
Reason: final acceptance opened
Blockers: none
Allowed follow-up command: `scafld handoff rust-runtime-adapters-catalog`
Latest runner update: 2026-05-19T08:25:22Z
Review gate: not_started

## Summary

Port the `catalog` adapter family to `runx-runtime`. Catalog adapters
execute a tool referenced by `source.catalog_ref`. They do not build,
publish, search, or inspect tool catalogs; those producer and reader surfaces
are owned by `rust-tool-catalogs`. This spec consumes that completed Rust
tool-catalog surface and wires catalog-backed invocation into the runtime
adapter trait.

This is a hard cutover spec for the catalog adapter path. Once the Rust
adapter is routed, it must not dispatch to the TypeScript catalog adapter at
runtime.

## Context

CWD: `.`

Read-only TypeScript references:
- `packages/adapters/src/catalog/index.ts`
- `packages/runtime-local/src/tool-catalogs/index.ts`
- `packages/runtime-local/src/tool-catalogs/mcp.ts`
- `packages/runtime-local/src/tool-catalogs/fixture.ts`
- `packages/runtime-local/src/runner-local/execution-targets.ts`
- `packages/adapters/src/runtime.test.ts`
- `packages/runtime-local/src/tool-catalogs/index.test.ts`

Rust packages:
- `crates/runx-runtime`
- `crates/runx-contracts` (tools manifest)
- `crates/runx-parser` (validated `source.type = "catalog"` fields)

Implementation impact files:
- `crates/runx-runtime/src/adapters/mod.rs`
- `crates/runx-runtime/src/adapters/catalog.rs`
- `crates/runx-runtime/src/lib.rs`
- `crates/runx-runtime/Cargo.toml`
- `crates/runx-runtime/tests/catalog_adapter.rs`
- `crates/runx-runtime/tests/support/catalog_oracles.rs` if shared helpers
  keep the test clearer
- `scripts/generate-runtime-catalog-adapter-oracles.ts`
- `scripts/check-runtime-catalog-adapter-oracles.sh`
- `fixtures/runtime/adapters/catalog/**`

Do not modify in this spec execution:
- `crates/runx-runtime/src/tool_catalogs/**`, except for a small public API
  exposure required by the adapter and justified in the implementation notes.
- `crates/runx-contracts/src/tools.rs`, except if `rust-tool-catalogs`
  explicitly left a missing exported type needed by this adapter.
- TypeScript runtime files. TypeScript is the oracle source for committed
  fixture bytes before routing changes, not a runtime dispatch path.

Invariants:
- Tool manifests come from `runx-contracts::tools` types.
- Catalog resolution is deterministic given the adapter list and catalog
  snapshot.
- `source.catalog_ref` is required. Missing metadata returns the exact failure
  message `Catalog source requires source.catalog_ref metadata.`.
- A missing imported tool returns the exact failure message
  `Imported tool '<ref>' was not found in configured tool catalogs.`.
- Successful invocation preserves the resolved tool's status, stdout, stderr,
  error message, and metadata shape. It must not add catalog-specific fields
  to user stdout.
- Dynamic fields such as duration are normalized only in oracle comparison
  helpers; production output must carry the measured duration.
- No live network in fixture tests.
- The `catalog` runtime feature is opt-in and must not enable `mcp`, `agent`,
  or `a2a` implicitly. A test must prove the feature builds with only the
  dependencies it needs.

## Objectives

- Port catalog-backed invocation under the established Rust adapter trait.
- Reuse the completed `runx-runtime::tool_catalogs` resolver rather than
  reimplementing search, inspect, manifest parsing, or scoring.
- Add oracle-backed fixtures for missing metadata, missing catalog entry,
  successful fixture catalog invocation, propagated failure, and local
  precedence over a fixture catalog collision.
- Add a check script that regenerates TypeScript oracle bytes and fails on
  drift.

## Scope

In scope:
- `source.type = "catalog"` adapter invocation in `runx-runtime`.
- Feature-gated `catalog` adapter registration in the Rust runtime.
- Local fixture catalog invocation through deterministic fixture snapshots.
- Error and output parity against TypeScript for adapter-level behavior.

Out of scope:
- Catalog publishing (`runx tool build`); that's `rust-tool-catalogs`.
- `runx tool search` and `runx tool inspect`; those remain in
  `rust-tool-catalogs`.
- Remote registry or OCI live tests. Use committed snapshots only.
- Any new hosted catalog service.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-tool-catalogs` (CLI surface; catalog producer side).

## Implementation Contract

### Adapter Shape

Implement `CatalogAdapter` in `crates/runx-runtime/src/adapters/catalog.rs`
behind `features = ["catalog"]`. The adapter must implement the same
`SkillAdapter` trait used by `cli-tool`; if the skeleton has evolved that
trait before this spec executes, use the established trait rather than adding
a second catalog-specific trait.

The adapter receives a validated `SkillInvocation` whose source has
`type = "catalog"` and uses `source.catalog_ref` to resolve a tool through
the Rust tool-catalog resolver. The adapter must pass through:
- `inputs`
- `resolved_inputs` if the runtime trait carries them by execution time
- `env`
- cancellation signal or timeout context if the runtime trait carries it
- `skill_directory`
- run and step identifiers if the runtime trait carries them

If a resolved catalog tool points to another adapter family, dispatch through
the runtime adapter registry. Do not call TypeScript and do not special-case
MCP beyond using the catalog resolver output. The catalog adapter is a router
from catalog references to resolved runtime tools, not a second MCP adapter.

### Error Shape

Return normal adapter failure output rather than raising runtime errors for
expected user-facing failures:
- missing `source.catalog_ref`
- catalog ref not found
- resolved tool invocation returned failure

Reserve `RuntimeError` for broken fixtures, invalid catalog snapshots, IO
errors while loading committed fixtures, or internal Rust invariants. The
user-facing failure text must match TypeScript oracle output byte-for-byte
after normalizing dynamic duration fields.

### Feature Boundaries

`runx-runtime` must keep default features empty. Add `catalog` as an opt-in
feature if it is not already present. The feature may depend on the completed
tool-catalog modules and on the adapter trait, but it must not pull in MCP,
agent provider clients, or cloud dependencies.

## Fixture and Oracle Contract

Create deterministic fixtures under `fixtures/runtime/adapters/catalog/`:
- `missing-catalog-ref/`
- `missing-imported-tool/`
- `fixture-success/`
- `fixture-failure/`
- `local-precedence/`

Add `scripts/generate-runtime-catalog-adapter-oracles.ts` to execute the
TypeScript catalog adapter against those fixtures and write oracle files:
- `fixtures/runtime/adapters/catalog/oracles/<case>.stdout`
- `fixtures/runtime/adapters/catalog/oracles/<case>.stderr`
- `fixtures/runtime/adapters/catalog/oracles/<case>.status`
- `fixtures/runtime/adapters/catalog/oracles/<case>.json`

The generator must:
- Run from the OSS workspace root.
- Force `RUNX_ENABLE_FIXTURE_TOOL_CATALOG=1` only for cases that require the
  fixture catalog.
- Clear cache and home paths into a temporary directory.
- Normalize measured duration fields to a sentinel only in the oracle
  comparison layer.
- Record expected failure cases with non-success status and committed stderr.
- Fail if any oracle contains repository-absolute temporary paths, secrets, or
  wall-clock timestamps.

Add `scripts/check-runtime-catalog-adapter-oracles.sh` to run the generator in
check mode.

## Tests

Add Rust tests for:
- Missing catalog ref failure.
- Missing imported tool failure.
- Successful fixture catalog invocation.
- Resolved tool failure propagation.
- Local tool precedence over a fixture catalog entry with the same ref.
- Feature-gated build of `runx-runtime` with `catalog` and without `mcp`.
- No live network access in all catalog adapter fixtures.

Test names should include `catalog_adapter` and the behavior under test.
Normalize only dynamic duration fields before byte comparison.

## Acceptance Commands

Run these after implementation:

```sh
pnpm install --frozen-lockfile
node scripts/test-workspace.mjs packages/adapters/src/catalog/index.test.ts packages/runtime-local/src/tool-catalogs/index.test.ts
pnpm exec tsx scripts/generate-runtime-catalog-adapter-oracles.ts --check
scripts/check-runtime-catalog-adapter-oracles.sh
cargo fmt --manifest-path crates/Cargo.toml --all --check
cargo test --manifest-path crates/Cargo.toml -p runx-runtime catalog_adapter --features catalog -- --nocapture
cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features catalog -- -D warnings
node scripts/check-rust-core-style.mjs
```

If the workspace-level Rust command is already stable when this spec runs,
also run:

```sh
cargo test --manifest-path crates/Cargo.toml --workspace
```

## Completion Criteria

- Rust catalog adapter output matches TypeScript oracle bytes for committed
  fixture cases after duration normalization.
- Runtime dispatch for `source.type = "catalog"` uses Rust only after routing
  is switched.
- The adapter reuses `runx-runtime::tool_catalogs` and does not duplicate
  manifest parsing, scoring, or inspect presentation.
- Default runtime features remain empty; `catalog` is opt-in.
- No live network or external catalog service is required by tests.
- Acceptance commands pass, or any skipped command is recorded with the exact
  blocker and owner.

## Open Questions

- Whether OCI-backed catalogs need adapter-level invocation coverage in this
  spec. Default: no; cover only committed local snapshots unless a TypeScript
  fixture already exists at execution time.
- Whether `resolved_inputs` exists on the Rust adapter trait by execution
  time. Default: pass it through if the trait carries it; do not add a
  catalog-only request type.
