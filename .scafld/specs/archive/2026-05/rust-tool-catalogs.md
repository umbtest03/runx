---
spec_version: '2.0'
task_id: rust-tool-catalogs
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T08:24:25Z'
status: completed
harden_status: hardened
size: medium
risk_level: medium
---

# Rust tool catalogs

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T08:24:25Z
Review gate: pass

## Summary

Port the tool-catalog producer and reader surface to Rust. `runx tool build`
produces a tool manifest from source inputs. `runx tool search` reads catalog
and registry snapshots and returns deterministic search results. `runx tool
inspect` reads a manifest or resolved catalog entry and returns the same
presentation data as the TypeScript command for the same input snapshot.

This work is executable only after `rust-scaffold` lands. Until then, this
spec defines the Rust files, fixtures, generator/check scripts, command
contracts, and acceptance commands the implementation must satisfy.

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

Expected Rust files:
- `crates/runx-contracts/src/tools.rs`
- `crates/runx-runtime/src/tool_catalogs.rs`
- `crates/runx-runtime/src/tool_catalogs/build.rs`
- `crates/runx-runtime/src/tool_catalogs/search.rs`
- `crates/runx-runtime/src/tool_catalogs/inspect.rs`
- `crates/runx-runtime/src/tool_catalogs/error.rs`
- `crates/runx-cli/src/commands/tool.rs`

Expected fixture and check files:
- `fixtures/tool-catalogs/build/**`
- `fixtures/tool-catalogs/search/**`
- `fixtures/tool-catalogs/inspect/**`
- `fixtures/tool-catalogs/oracles/**`
- `scripts/generate-tool-catalog-oracles.ts`
- `scripts/check-tool-catalog-oracles.sh`

Invariants:
- Tool manifest schema is owned by `runx-contracts::tools`.
- Build is deterministic given inputs.
- Search / inspect outputs are byte-identical to TS for the same catalog
  snapshot.
- Rust command behavior must be checked against TypeScript byte oracles before
  command routing is switched over.
- Fixtures must avoid network access and must not depend on wall-clock time.
- Output ordering must be stable: sort keys and result lists anywhere the TS
  command does so, and add explicit stable sorting where the TS command depends
  on insertion order.

## Objectives

- Port tool build (manifest emission).
- Port tool search (catalog + registry-backed).
- Port tool inspect (manifest read + presentation).
- Add fixture-backed parity tests for success and failure behavior.
- Add an oracle generation/check path so drift can be reviewed before Rust
  command routing changes.

## Scope

In scope:
- Rust implementation of the tool catalog library surface in `runx-runtime`.
- Rust CLI wiring for `runx tool build`, `runx tool search`, and `runx tool
  inspect` in `runx-cli`.
- Shared manifest/result types in `runx-contracts::tools`.
- Deterministic fixture snapshots and generated TypeScript oracles for build,
  search, and inspect.
- Tests proving Rust results match the generated oracles byte-for-byte where
  the command emits JSON or structured text.

Out of scope:
- Adapter-side catalog invocation (covered by
  `rust-runtime-adapters-catalog`).
- Runtime process execution, sandboxing, or adapter dispatch.
- Remote registry integration tests. Use local registry/catalog fixture
  snapshots only.
- Any changes to existing TypeScript command semantics except adding oracle
  generation/check helpers.

Do not introduce migration-era mode names in code, fixture names, CLI output,
test names, or documentation added for this spec. The TypeScript path is the
oracle source for parity, not a user-facing mode name.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-contracts-parity`.
- `rust-scaffold` must have created the Rust crate and CLI module structure
  before implementation starts.

Preflight note: earlier planning referred to a `rust-tools-parity` contract
surface. The implementation target for this spec is `rust-tool-catalogs`, and
the contract module name must be `crates/runx-contracts/src/tools.rs` exposed
as `runx_contracts::tools`. Do not create a separate `rust_tools_parity` or
`tools_parity` module.

## Implementation Contract

### Contract Types

Define or complete these types in `crates/runx-contracts/src/tools.rs`:
- `ToolManifest`
- `ToolManifestTool`
- `ToolManifestCommand`
- `ToolCatalogEntry`
- `ToolSearchQuery`
- `ToolSearchResult`
- `ToolInspectRequest`
- `ToolInspectResult`
- `ToolCatalogError` or the shared error type expected by scaffolded
  contracts

Use serde derives and strict field names matching the TypeScript manifest and
command output shapes. Optional fields must serialize the same way the
TypeScript command does. If TypeScript omits absent fields, Rust must use
`skip_serializing_if = "Option::is_none"` for those fields.

### Build

Implement `crates/runx-runtime/src/tool_catalogs/build.rs` with:
- A pure function that accepts explicit input paths and returns `ToolManifest`.
- A command-facing function that writes the manifest bytes exactly as the CLI
  contract requires.
- Deterministic source traversal, stable object key order, normalized path
  separators, and stable error messages for invalid source data.

The build path must not shell out to TypeScript. If OCI manifest emission is
still unresolved at implementation time, isolate it behind a small Rust trait
or function boundary and keep the fixture-backed manifest output testable
without OCI network or registry access.

### Search

Implement `crates/runx-runtime/src/tool_catalogs/search.rs` with:
- Loading of local catalog and registry snapshot fixtures.
- Query normalization matching TypeScript behavior.
- Stable result scoring, tie-breaking, filtering, and presentation.
- Explicit tests for empty query, no results, multiple catalogs, duplicate
  names, tag/category matches, and malformed catalog data.

### Inspect

Implement `crates/runx-runtime/src/tool_catalogs/inspect.rs` with:
- Manifest-file inspection.
- Catalog-entry inspection from a local snapshot.
- Stable structured output that matches TypeScript for all fixture cases.
- Explicit errors for missing tool, ambiguous tool, malformed manifest, and
  missing source path.

### CLI Wiring

Implement `crates/runx-cli/src/commands/tool.rs` so the Rust CLI exposes the
same flags, positional arguments, default output mode, exit status behavior,
stdout, and stderr as the current TypeScript command for:
- `runx tool build`
- `runx tool search`
- `runx tool inspect`

If scaffold creates a different module path, adapt the path through the
scaffolded command tree while keeping this spec's command contract unchanged.

## Fixture and Oracle Contract

Create fixture groups under `fixtures/tool-catalogs/`:
- `build/`: source inputs for one minimal tool, one multi-command tool, one
  metadata-heavy tool, and one invalid tool.
- `search/`: catalog snapshots covering exact name, partial text, tag,
  category, duplicate name across catalogs, empty result, and malformed
  catalog cases.
- `inspect/`: manifest and catalog-entry snapshots covering success, missing,
  ambiguous, and malformed cases.
- `oracles/`: generated TypeScript stdout/stderr/status files for each
  fixture case.

Oracle files must include status and bytes, not only parsed JSON. Use this
layout unless scaffold dictates a stricter local convention:
- `fixtures/tool-catalogs/oracles/<case>.stdout`
- `fixtures/tool-catalogs/oracles/<case>.stderr`
- `fixtures/tool-catalogs/oracles/<case>.status`
- `fixtures/tool-catalogs/oracles/<case>.json` when structured data is useful
  for test diagnostics

Add `scripts/generate-tool-catalog-oracles.ts` to run the TypeScript command
against the fixtures and rewrite the oracle files. Add
`scripts/check-tool-catalog-oracles.sh` to run the generator in check mode and
fail if generated bytes differ from committed oracle files.

The generator must:
- Pin environment variables needed for deterministic output.
- Run from the repository root.
- Clear or redirect any cache paths into a temporary directory.
- Write stable fixture paths into outputs, or normalize repository-absolute
  paths before comparing.
- Record non-zero statuses for expected failure cases.

## Tests

Add focused Rust tests for:
- Contract serialization and deserialization of every tool manifest/result
  shape used by fixtures.
- Build output bytes against generated TS oracles.
- Search output bytes against generated TS oracles.
- Inspect output bytes against generated TS oracles.
- Error status, stdout, and stderr against generated TS oracles.

Test names should include `tool_catalogs`, `build`, `search`, or `inspect`
and the behavior under test. Avoid migration-era mode naming.

## Acceptance Commands

Run these after implementation:

```sh
pnpm install --frozen-lockfile
pnpm exec tsx scripts/generate-tool-catalog-oracles.ts --check
scripts/check-tool-catalog-oracles.sh
cargo fmt --check
cargo test -p runx-contracts tools
cargo test -p runx-runtime tool_catalogs
cargo test -p runx-cli tool
```

If the repository uses a workspace-level command after `rust-scaffold`, also
run the scaffolded equivalent of:

```sh
cargo test --workspace
```

## Completion Criteria

- Rust build/search/inspect behavior matches TypeScript oracle bytes for all
  committed fixture cases.
- `runx_contracts::tools` owns the manifest and result shapes used by runtime
  and CLI.
- CLI exit statuses, stdout, and stderr match the TypeScript command for
  success and expected failure cases.
- Oracle generation and check scripts are committed and documented by their
  command names in this spec.
- No migration-era mode naming is introduced by this work.
- Acceptance commands pass, or any skipped command is recorded with the exact
  blocker and follow-up owner.

## Open Questions

- Whether OCI manifest emission requires a Rust OCI library at port time.
  Default: no; CLI-shell into `oci` if needed.
- Whether scaffold names the CLI crate `runx-cli` exactly. If not, keep the
  command behavior and update only the concrete path/package names during
  implementation.

## Review

Status: completed
Verdict: pass
Mode: verify
Summary: Human-reviewed override accepted: manual audit after Claude provider context timeout; tool-catalog acceptance gates passed, workspace cargo passed, focused reviewer blockers were fixed, and no completion-blocking findings remain

Attack log:
- `review gate`: manual human audit -> clean (manual audit after Claude provider context timeout; tool-catalog acceptance gates passed, workspace cargo passed, focused reviewer blockers were fixed, and no completion-blocking findings remain)

Findings:
- none
