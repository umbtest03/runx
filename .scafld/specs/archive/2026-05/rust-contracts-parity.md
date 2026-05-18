---
spec_version: '2.0'
task_id: rust-contracts-parity
created: '2026-05-17T01:30:00Z'
updated: '2026-05-18T06:52:03Z'
status: completed
harden_status: passed
size: large
risk_level: high
---

# Rust contracts parity

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-18T06:52:03Z
Review gate: pass

## Summary

Upgrade the `runx-contracts` placeholder into the first real Rust contracts
crate. This crate owns typed serde shapes for public JSON and wire contracts
that are shared by `runx-sdk`, `runx-runtime`, `runx-cli`, and the pure crates.

The immediate driver is `rust-sdk-surface-parity`: SDK v0 must not hand-write
host-protocol or capability-execution types that later move to contracts.
`runx-contracts` therefore ships before SDK Phase 2 and becomes the single
Rust home for host protocol, capability execution, and idempotency hash
shapes. CLI JSON contracts get a deferred module home here, but actual CLI
JSON parity is owned by `rust-contracts-cli-json-parity` after
`rust-cli-feature-parity-matrix` produces its oracle.

This crate contains no runtime behavior. It does not execute skills, evaluate
policy, spawn processes, perform network IO, or read/write receipts. It is a
typed contract layer plus deterministic hashing helpers where the hash is part
of the public contract.

## Context

CWD: `.`

Packages:
- `crates/runx-contracts`
- `@runxhq/contracts`
- `@runxhq/runtime-local`
- `@runxhq/core`

Current TypeScript sources:
- `packages/contracts/src/index.ts`
- `packages/contracts/src/schemas/capability-execution.ts`
- `packages/contracts/src/schemas/receipt.ts`
- `packages/contracts/src/schemas/local-receipt.ts`
- `packages/contracts/src/schemas/registry.ts`
- `packages/contracts/src/schemas/tool-manifest.ts`
- `packages/contracts/src/schemas/list.ts`
- `packages/contracts/src/schemas/doctor.ts`
- `packages/runtime-local/src/sdk/capability-execution.ts`
- `packages/runtime-local/src/sdk/host-protocol.ts`

Files impacted:
- `crates/runx-contracts/Cargo.toml`
- `crates/runx-contracts/README.md`
- `crates/runx-contracts/src/lib.rs`
- `crates/runx-contracts/src/json.rs`
- `crates/runx-contracts/src/capability_execution.rs`
- `crates/runx-contracts/src/capability_execution/hash.rs`
- `crates/runx-contracts/src/host_protocol.rs`
- `crates/runx-contracts/src/cli.rs`
- `crates/runx-contracts/src/receipts.rs`
- `crates/runx-contracts/src/registry.rs`
- `crates/runx-contracts/src/tools.rs`
- `crates/runx-contracts/tests/capability_execution_fixtures.rs`
- `crates/runx-contracts/tests/host_protocol_fixtures.rs`
- `fixtures/contracts/capability-execution/**`
- `fixtures/contracts/host-protocol/**`
- `scripts/generate-rust-contract-fixtures.ts`
- `scripts/check-contract-fixture-key-order.ts`
- `scripts/check-rust-core-style.mjs`
- `docs/rust-kernel-architecture.md`

Invariants:
- TypeScript contracts remain authoritative until a cutover spec says
  otherwise.
- `runx-contracts` owns shared Rust contract types. `runx-sdk` and
  `runx-runtime` must consume them instead of duplicating them.
- No IO dependencies: no `tokio`, `reqwest`, `hyper`, `rmcp`, `clap`,
  `std::fs`, `std::process`, `std::net`, or `std::env`.
- Public APIs use typed structs/enums and concrete errors. `thiserror` is the
  preferred helper for concrete error enums when manual `Display`/`Error`
  implementations would add noise. No public `serde_json::Value`, no
  `HashMap` at serialized boundaries, no wildcard re-exports, no `anyhow` or
  `eyre`.
- Serialized maps use deterministic key ordering. Use `BTreeMap` for map-like
  contract fields.
- Hash helpers match the current TypeScript `hashStable` and `hashString`
  behavior only for fixture-backed, ASCII-key capability contract shapes in
  this spec. Undefined-like fields are omitted by fixture generation, SHA-256
  hex digests are lower-case, and the `sha256:` prefix is used where the TS
  contract uses it.
- `runx-contracts` owns capability-execution and idempotency hash semantics.
  Other workspace crates may use SHA-256 for non-contract work such as receipt
  verification or adapter content addressing if their Cargo manifest documents
  the rationale.
- This spec does not change TypeScript `stableStringify` ordering. The global
  `localeCompare` to canonical code-point ordering migration is deferred to
  `hash-stable-codepoint-cutover`.
- Phase 1 `ac1_2` inherits workspace-wide Rust style state because
  `scripts/check-rust-core-style.mjs` walks all seven crate roots. Unrelated
  crate violations can block Phase 1 until fixed in their owning spec.
- Contract modules are small and direct. Do not generate giant Rust code from
  TypeBox output in this phase.

Related docs:
- `docs/rust-kernel-architecture.md`
- `docs/api-surface.md`
- `packages/contracts/src/index.ts`
- `packages/runtime-local/src/sdk/capability-execution.ts`
- `packages/runtime-local/src/sdk/host-protocol.ts`

## Objectives

- Replace the placeholder `runx-contracts` surface with real modules and
  typed serde contracts.
- Add a small Rust JSON model for contract-owned arbitrary JSON values without
  exposing `serde_json::Value` publicly.
- Port capability-execution contracts and idempotency hash helpers first,
  including cross-language fixtures against TypeScript.
- Port the host protocol result/state subset required by `runx-sdk` v0.
- Create a deferred CLI JSON module home. Actual CLI JSON parity belongs to
  `rust-contracts-cli-json-parity` after the CLI feature parity matrix exists.
- Add receipts, registry, and tool contract skeleton modules with explicit
  deferred parity markers so downstream crates have stable module homes.
- Wire the Rust style guard and fixture key-order checks around the contract
  crate.

## Scope

In scope:
- `runx-contracts` placeholder upgrade.
- Capability-execution contract and idempotency helpers.
- Host protocol serializable wire/result/state subset consumed by SDK v0.
  Function and closure surfaces from the TypeScript module are not contracts.
- CLI JSON deferred module home only, with a doc comment naming
  `rust-contracts-cli-json-parity` as owner.
- Receipt, registry, and tool modules as typed skeletons or explicit deferred
  modules, depending on fixture readiness.
- JSON Schema backed fixtures generated from TypeScript.

Out of scope:
- Native runtime execution.
- Policy/state-machine logic. That belongs in `runx-core`.
- Parser behavior. That belongs in `runx-parser`.
- Receipt verification logic. That belongs in `runx-receipts`.
- MCP process/client/server behavior. That belongs under `runx-runtime`
  feature flags.
- Publishing to crates.io.
- Full TypeBox/OpenAPI code generation.
- Host-protocol closure/function types such as `Caller`, `AuthResolver`,
  `HostBoundaryResolver`, `HostBridge`, and `HostStateInspector`.
- CLI JSON envelopes and fixtures. Those require the CLI parity matrix oracle
  and belong to `rust-contracts-cli-json-parity`.
- Global `stableStringify` comparator migration, including receipt signatures,
  state-machine stable strings, push_outbox stable strings, authoring equality,
  and any persisted hash/signature migration. That belongs to
  `hash-stable-codepoint-cutover`.

## Dependencies

- `rust-contracts-bootstrap` completed and approved.
- `crates/runx-contracts` placeholder exists as a workspace member.
- `docs/rust-kernel-architecture.md` defines the seven-crate graph and section
  17 Rust implementation quality bar.
- `scripts/check-rust-core-style.mjs` exists and scans `crates/runx-contracts/src`.
- TypeScript contract tests pass before fixture generation.
- `rust-contracts-cli-json-parity` is the follow-up owner for CLI JSON. That
  follow-up depends on `rust-cli-feature-parity-matrix` completed and
  `fixtures/cli-parity/**` existing.

Downstream dependency:
- `rust-sdk-surface-parity` Phase 2 may depend on this spec for JSON,
  capability-execution, and host-protocol types. SDK CLI JSON parsing depends
  on the follow-up `rust-contracts-cli-json-parity`.

## Assumptions

- The first Rust contract subset can be smaller than the full TS package, but
  every exposed Rust type must have fixture evidence.
- `runx-contracts` starts with `serde` as its runtime dependency.
  `serde_json` may stay test-only unless a private runtime serializer needs it.
  Phase 2 adds `sha2` beside the first hashing implementation. `thiserror` is
  deferred until a phase introduces the first concrete public error enum. The
  crate does not need `tokio`, HTTP, MCP, or CLI parsing crates.
- The existing bootstrap JSON model in `crates/runx-contracts/src/lib.rs`
  (`JsonValue`, `JsonNumber`, and `JsonObject`) is already shipped local work.
  Phase 1 migrates it into `src/json.rs`; it does not delete or replace it
  with placeholder constants.
- Stable hash parity in this spec is intentionally narrow. Fixtures pin the
  current TypeScript `hashStable` behavior for ASCII-key capability execution
  shapes. The fixture generator rejects non-ASCII object keys in this phase.
- The fixture-generator non-ASCII-key guard does not protect runtime callers
  of `hashStable`. Any TS callsite that hashes user-controlled object keys can
  still produce a TS-only hash until `hash-stable-codepoint-cutover` replaces
  `localeCompare` ordering with a canonical comparator and proves receipt,
  state-machine, push_outbox, authoring, knowledge, A2A, and capability hashes.
- This spec never edits `packages/core/src/util/hash.ts`,
  `packages/core/src/state-machine/index.ts`, or
  `packages/cli/tools/thread/push_outbox/src/index.ts`.
- The published crate package includes only library sources and README files.
  Workspace integration tests and `fixtures/contracts/**` are intentionally
  not vendored into the crate package.
- Some TS contracts are TypeBox schemas with no direct runtime behavior. Rust
  may model the consumed wire shape directly instead of mirroring the TypeBox
  implementation.

## Risks

- High: contract drift silently breaks SDK/runtime interoperability. Mitigated
  by fixture generation from TypeScript and cross-language hash fixtures.
- High: the crate can turn into a dumping ground for every TS interface.
  Mitigated by exposed-type fixture requirements and module ownership rules.
- Medium: stable JSON hashing can diverge on ordering, omission, or numeric
  formatting. Mitigated by fixture-backed ASCII-key capability hashes and
  BTreeMap-backed map fields for serde boundaries only. The non-ASCII/global
  ordering problem is explicitly deferred to `hash-stable-codepoint-cutover`.
- Medium: callers may assume this spec stabilizes every runx hash. Mitigated
  by naming the narrow fixture-backed hash scope in invariants, README, and
  Phase 2 acceptance.
- Medium: CLI JSON can be treated as stable before the CLI matrix covers it.
  Mitigated by deferring actual CLI JSON parity to
  `rust-contracts-cli-json-parity`.
- Medium: public API can become verbose if TypeScript interfaces are copied
  mechanically. Mitigated by the section 18 Rust quality bar.
- Low: external helper crates could add unnecessary surface. Mitigated by
  keeping dependencies to `serde` plus phase-local additions (`sha2` for
  hashing, `thiserror` only when a concrete public error enum exists).

## Acceptance

Profile: strict

Validation:
- [x] `v1` command - TypeScript contract tests pass before fixture generation.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/contracts/src/index.test.ts packages/contracts/src/handoff-contracts.test.ts packages/runtime-local/src/sdk/capability-execution.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-41
- [x] `v2` command - Rust contracts tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-42
- [x] `v3` command - Rust formatting and clippy pass.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-contracts --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-43
- [x] `v4` command - Rust style guard passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-44
- [x] `v5` command - contract fixtures regenerate cleanly.
  - Command: `pnpm exec tsx scripts/generate-rust-contract-fixtures.ts --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-45
- [x] `v6` command - fixture JSON key order is deterministic.
  - Command: `pnpm exec tsx scripts/check-contract-fixture-key-order.ts fixtures/contracts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-46
- [x] `v7` command - dependency boundary is clean.
  - Command: `! rg -n 'tokio|reqwest|hyper|rmcp|clap|std::fs|std::process|std::net|std::env|Command::new' crates/runx-contracts/Cargo.toml crates/runx-contracts/src crates/runx-contracts/tests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-47
- [x] `v8` command - package check passes.
  - Command: `cargo package --manifest-path crates/Cargo.toml -p runx-contracts --allow-dirty`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-48

## Phase 1: Contract crate skeleton

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- `crates/runx-contracts/Cargo.toml` (partial, exclusive) - Ensure `serde` is a runtime dependency. Keep `serde_json` as a dev-dependency in Phase 1 unless `json.rs` needs a private runtime serializer. Defer `sha2` to Phase 2 and defer `thiserror` to the phase that introduces the first concrete public error enum. Keep edition 2024, workspace lints, and package metadata.
- `crates/runx-contracts/src/lib.rs` (partial, exclusive) - Declare modules and explicit re-exports. No wildcard re-exports. Move the existing bootstrap `JsonValue`, `JsonNumber`, `JsonObject`, serde impls, and unit tests out of `lib.rs` into `json.rs`; do not discard that shipped surface.
- `crates/runx-contracts/src/json.rs` (all, exclusive) - Contract-owned JSON value type and deterministic map helpers.
- `scripts/check-contract-fixture-key-order.ts` (all, exclusive) - Contract-fixture sorted-key validator. This is a TypeScript script and must be invoked through `pnpm exec tsx`, not `node`.
- `crates/runx-contracts/README.md` (partial, exclusive) - Document contract ownership and non-runtime scope.

Acceptance:
- [x] `ac1_1` command - module skeleton compiles.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac1_2` command - style guard passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `ac1_3` command - key-order checker exists and rejects misordered JSON.
  - Command: `tmp="$(mktemp -d)" && mkdir "$tmp/good" "$tmp/bad" && printf '{"a":2,"b":1}\n' > "$tmp/good/good.json" && printf '{"b":1,"a":2}\n' > "$tmp/bad/bad.json" && pnpm exec tsx scripts/check-contract-fixture-key-order.ts "$tmp/good" && ! pnpm exec tsx scripts/check-contract-fixture-key-order.ts "$tmp/bad" && pnpm exec tsx scripts/check-contract-fixture-key-order.ts fixtures/contracts --allow-missing`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8

## Phase 2: Capability execution contracts

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `crates/runx-contracts/src/capability_execution.rs` (all, exclusive) - Capability execution types, actor/transport/idempotency contracts, builder input, normalization, and stable hash helper wiring.
- `crates/runx-contracts/src/capability_execution/hash.rs` (all, exclusive) - Private stable hash writer and SHA-256 prefix helper used only by `capability_execution`.
- `crates/runx-contracts/Cargo.toml` (partial, exclusive) - Add `sha2` as a runtime dependency beside the first SHA-256 implementation. Promote `serde_json` to a runtime dependency only if the implementation needs private serde serialization for stable hashing; do not expose `serde_json::Value` in the public API.
- `fixtures/contracts/capability-execution/*.json` (all, exclusive) - Golden TS-generated cases, including intent key, trigger key, and content hash. Fixture keys are restricted to the ASCII object-key shapes supported by the current TypeScript `hashStable` behavior.
- `crates/runx-contracts/tests/capability_execution_fixtures.rs` (all, exclusive) - Fixture and hash-stability tests.
- `scripts/generate-rust-contract-fixtures.ts` (partial, exclusive) - Generate capability fixtures from TypeScript source of truth. Reject non-ASCII object keys in this phase and document that canonical non-ASCII ordering is owned by `hash-stable-codepoint-cutover`.

Acceptance:
- [x] `ac2_1` command - capability fixture tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test capability_execution_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13
- [x] `ac2_2` command - TypeScript fixture check is clean.
  - Command: `pnpm exec tsx scripts/generate-rust-contract-fixtures.ts --check --scope capability-execution`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `ac2_3` command - Rust SDK v0 does not duplicate contract hashing.
  - Command: `rg -n 'sha2|Sha256|Digest' crates/runx-contracts/src/capability_execution.rs && ! rg -n 'sha2|Sha256|Digest' crates/runx-sdk/src crates/runx-sdk/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `ac2_4` command - hash fixture scope is explicit and defers canonical
  - Command: `for tok in ASCII non-ASCII hashStable hash-stable-codepoint-cutover localeCompare; do rg -q "$tok" scripts/generate-rust-contract-fixtures.ts fixtures/contracts/capability-execution crates/runx-contracts/src/capability_execution.rs || { echo "missing $tok"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16

## Phase 3: Host protocol contracts

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `crates/runx-contracts/src/host_protocol.rs` (all, exclusive) - Host result and state projection contracts for paused, completed, failed, escalated, denied, resume handling, and inspect projections. Only serializable wire data is in scope. Do not port or expose the TypeScript function/closure surfaces (`Caller`, `AuthResolver`, `HostBoundaryResolver`, `HostBridge`, `HostStateInspector`) from this crate. Phase 3 also owns the transitive serializable wire subset required by those host types: `ExecutionEvent { type, message, data }`, `ResolutionResponse { actor, payload }`, and tagged `ResolutionRequest` variants for `input`, `approval`, and `cognitive_work`. The immediate `ResolutionRequest` payloads must be typed Rust shapes: `Input { questions: Vec<Question> }`, `Approval { gate: ApprovalGate }`, and `CognitiveWork { work: AgentWorkRequest }`. Fields two levels deeper than `Question`, `ApprovalGate`, and `AgentWorkRequest` may be represented as `JsonValue` with an explicit deferred-parity comment naming `rust-resolution-payload-parity` as follow-up owner. `AgentWorkRequest.envelope` is the sole Phase 3 depth-1 exception: it remains an intentionally opaque `JsonValue` because the envelope is a protocol-owned payload whose typed internals belong to `rust-resolution-payload-parity`; the owning struct itself (`AgentWorkRequest`) still remains typed. Model `ResolutionRequest` as a Rust enum with `#[serde(tag = "kind", rename_all = "snake_case")]` to match `packages/contracts/src/schemas/resolution.ts`; model `ExecutionEvent` with `#[serde(tag = "type", rename_all = "snake_case")]` to match `packages/runtime-local/src/runner-local/index.ts`. Model `HostRunVerification`, `HostRunLineage`, and `HostRunApproval` as typed Rust structs, not as opaque `JsonValue`, because they are direct fields on every terminal host-state projection. Model `ExecutionEvent.type` as a closed Rust enum with `#[serde(rename_all = "snake_case")]` covering `skill_loaded`, `inputs_resolved`, `auth_resolved`, `resolution_requested`, `resolution_resolved`, `admitted`, `executing`, `step_started`, `step_waiting_resolution`, `step_completed`, `warning`, and `completed`.
- `fixtures/contracts/host-protocol/*.json` (all, exclusive) - Golden cases generated from TypeScript host protocol helpers. Include one fixture per `ExecutionEvent.type` variant. Host result fixtures use the filename pattern `result-host-run-<status>.json`; inspect projection fixtures use `inspect-host-state-<status>.json`, where `<status>` is one of `paused`, `completed`, `failed`, `escalated`, or `denied`.
- `crates/runx-contracts/tests/host_protocol_fixtures.rs` (all, exclusive) - Host protocol fixture tests.

Acceptance:
- [x] `ac3_1` command - host protocol fixture tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test host_protocol_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21
- [x] `ac3_2` command - fixtures cover required host outcomes.
  - Command: `for status in paused completed failed escalated denied; do test -f "fixtures/contracts/host-protocol/result-host-run-$status.json" || { echo "missing result $status"; exit 1; }; done && for status in paused completed failed escalated denied; do test -f "fixtures/contracts/host-protocol/inspect-host-state-$status.json" || { echo "missing inspect $status"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `ac3_3` command - host protocol transitive wire closure is explicit.
  - Command: `for tok in ExecutionEvent ResolutionRequest ResolutionResponse Input Approval CognitiveWork Question ApprovalGate AgentWorkRequest HostRunVerification HostRunLineage HostRunApproval skill_loaded inputs_resolved auth_resolved resolution_requested resolution_resolved admitted executing step_started step_waiting_resolution step_completed warning completed; do rg -q "$tok" crates/runx-contracts/src/host_protocol.rs fixtures/contracts/host-protocol || { echo "missing $tok"; exit 1; }; done && rg -q 'tag = "kind"' crates/runx-contracts/src/host_protocol.rs && rg -q 'tag = "type"' crates/runx-contracts/src/host_protocol.rs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23

## Phase 4: Deferred CLI JSON module home

Status: completed
Dependencies: Phase 3

Objective: Complete this phase.

Changes:
- `crates/runx-contracts/src/cli.rs` (all, exclusive) - Deferred contract module home only. It ships with a module doc comment naming `rust-contracts-cli-json-parity` as owner. No public CLI JSON types are exposed in this spec.
- `crates/runx-contracts/README.md` (partial, exclusive) - Document that actual CLI JSON parity is deferred until `rust-cli-feature-parity-matrix` produces a fixture oracle.

Acceptance:
- [x] `ac4_1` command - CLI module is explicitly deferred.
  - Command: `for tok in 'Deferred contract module home' rust-contracts-cli-json-parity rust-cli-feature-parity-matrix; do rg -q "$tok" crates/runx-contracts/src/cli.rs crates/runx-contracts/README.md || { echo "missing $tok"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-28
- [x] `ac4_2` command - this spec does not create CLI JSON fixtures or tests.
  - Command: `test ! -d fixtures/contracts/cli-json && test ! -f crates/runx-contracts/tests/cli_json_fixtures.rs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29

## Phase 5: Deferred module homes and docs

Status: completed
Dependencies: Phase 4

Objective: Complete this phase.

Changes:
- `crates/runx-contracts/src/receipts.rs` (all, exclusive) - Deferred contract module home only. Module doc comment names future `rust-receipts-parity` as owner. No public receipt types are exposed in this spec.
- `crates/runx-contracts/src/registry.rs` (all, exclusive) - Deferred contract module home only. Module doc comment names future `rust-registry-parity` as owner. No public registry types are exposed in this spec.
- `crates/runx-contracts/src/tools.rs` (all, exclusive) - Deferred contract module home only. Module doc comment names future `rust-tools-parity` as owner. No public tool manifest or catalog types are exposed in this spec.
- `crates/runx-contracts/README.md` (partial, exclusive) - Document exposed vs deferred contract surfaces, the `contracts-first-ordering:` marker, and that workspace fixture tests are deliberately excluded from the packaged crate by the Cargo `include` allowlist.
- `docs/rust-kernel-architecture.md` (partial, shared) - Keep contract crate graph, publishing stance, and `contracts-first-ordering:` marker aligned.

Acceptance:
- [x] `ac5_1` command - package check passes.
  - Command: `cargo package --manifest-path crates/Cargo.toml -p runx-contracts --allow-dirty`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-34
- [x] `ac5_2` command - docs describe contracts-first SDK dependency.
  - Command: `rg -n 'contracts-first-ordering:' docs/rust-kernel-architecture.md crates/runx-contracts/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-35
- [x] `ac5_3` command - crate-scoped Rust checks pass.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-contracts --all-targets -- -D warnings && cargo test --manifest-path crates/Cargo.toml -p runx-contracts && node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `ac5_4` command - packaged-crate fixture policy is documented and src
  - Command: `for tok in 'include allowlist' fixtures/contracts 'not vendored' 'excluded from the packaged crate'; do rg -q "$tok" crates/runx-contracts/README.md || { echo "missing $tok"; exit 1; }; done && ! rg -n 'include_str!\\(.*fixtures/contracts' crates/runx-contracts/src`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `ac5_5` command - deferred module owners are named.
  - Command: `for tok in rust-contracts-cli-json-parity rust-receipts-parity rust-registry-parity rust-tools-parity; do rg -q "$tok" crates/runx-contracts/src crates/runx-contracts/README.md || { echo "missing $tok"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38

## Follow-up Specs

These task ids are reserved for future drafts. They do not need to exist under
`.scafld/specs/` before this spec is approved; each will be drafted when its
preconditions land.

- `hash-stable-codepoint-cutover`: Owns global TypeScript stable-stringify
  comparator migration, including receipt signatures, state-machine stable
  strings, push_outbox stable strings, authoring equality, knowledge IDs, A2A
  IDs, and all persisted hash/signature migration evidence.
- `rust-contracts-cli-json-parity`: Owns actual CLI JSON envelope types and
  fixtures after `rust-cli-feature-parity-matrix` produces the consumed CLI
  JSON oracle.
- `rust-resolution-payload-parity`: Owns deeper typed payload parity for
  nested resolution request fields that Phase 3 intentionally leaves as
  `JsonValue`.
- `rust-receipts-parity`: Owns receipt contract parity and verification logic
  split with `runx-receipts`.
- `rust-registry-parity`: Owns registry and marketplace contract parity.
- `rust-tools-parity`: Owns tool manifest and catalog contract parity.

## Rollback

Strategy: per_phase

Commands:
- Phase 1: revert `runx-contracts` to the bootstrap-era JSON surface
  (`JsonValue`, `JsonNumber`, `JsonObject`, serde impls, and tests) and remove
  only the new module declarations/dependencies added by this spec. Do not
  delete the `rust-contracts-bootstrap` deliverable.
- Phase 2: remove capability execution module, fixtures, tests, and fixture
  generator scope. No TypeScript stable-stringify files are edited by this
  phase.
- Phase 3: remove host protocol module, fixtures, and tests.
- Phase 4: remove the deferred CLI JSON module home and README note.
- Phase 5: remove deferred module homes and docs updates.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Discover review of rust-contracts-parity. The runx-contracts crate now ships typed JSON, capability execution, and host protocol contracts with fixture-backed parity against the TypeScript oracle. I verified spec-scoped Cargo dependencies (serde, sha2 runtime; serde_json dev-only; no thiserror), module/exports composition in lib.rs, the contract-owned JsonValue/JsonNumber model with non-finite rejection at the public boundary, the stable hash writer that mirrors JSON.stringify semantics (NaN/Inf -> null, raw U+2028/U+2029, escape set, negative zero), capability execution typed shapes + normalization, host protocol enums with internally-tagged discriminators and rename_all_fields camelCase mapping, ResolutionRequest/Response/ApprovalGate/Question/AgentWorkRequest typing matching packages/contracts and packages/runtime-local sources, ExecutionEvent's 12 snake_case variants, HostRunResult and HostRunState shapes (events on every result variant; verification/lineage/approval as typed structs on terminal states), the deferred module homes (cli/receipts/registry/tools) carrying only doc comments naming their parity owners, the README's contracts-first-ordering marker and packaging include allowlist, the fixture generator's ASCII/codepoint-ordering guard, and the check-contract-fixture-key-order.ts pnpm-tsx invocation. All 26 host-protocol fixtures and 4 capability-execution fixtures are wired through include_str! and covered by the style script's coverage check; the workspace crate graph (check-rust-crate-graph.mjs) and style script (check-rust-core-style.mjs) inputs accept the new module set. No production source uses panic/unwrap/expect/serde_json::Value/HashMap. No spec items are missing and no scope leakage was found (envelope remains intentionally opaque per the documented deferred-parity owner). No completion-blocking issues identified.

Attack log:
- `crates/runx-contracts/Cargo.toml`: Spec compliance: verify deps match Phase 1+2 spec (serde runtime, sha2 added in Phase 2, serde_json dev-only, no thiserror, workspace lints, edition 2024, include allowlist excludes tests/fixtures from package) -> clean
- `crates/runx-contracts/src/lib.rs`: No wildcard re-exports; modules declared per spec; re-exports cover capability_execution, host_protocol, json items consumed by SDK v0 -> clean
- `crates/runx-contracts/src/json.rs`: JsonValue/JsonNumber parity: round-trip ordering via BTreeMap, public serde rejects non-finite floats, whole-float-as-integer formatting matches JSON.stringify integer emission, negative-zero handled -> clean
- `crates/runx-contracts/src/capability_execution/hash.rs`: stable_hash_json parity with TS hashStable: non-finite -> null, U+2028/U+2029 raw, control-char \u escapes, negative-zero collapses to '0'; sha256 prefix matches withSha256Prefix -> clean
- `crates/runx-contracts/src/capability_execution.rs`: BuildCapabilityExecution.build parity: transport/actor/scope_set/string normalization, empty-object pruning at top level only, intent/trigger/content hash composition mirrors TS deriveCapabilityExecution* functions and schema 'runx.capability_execution.v1' -> clean
- `crates/runx-contracts/src/host_protocol.rs`: ExecutionEvent has all 12 snake_case variants; ResolutionRequest tag='kind' matches packages/contracts/src/schemas/resolution.ts; Question/ApprovalGate/AgentWorkRequest fields align with agent-work schema (envelope kept JsonValue per spec exception) -> clean
- `crates/runx-contracts/src/host_protocol.rs`: HostRunResult / HostRunState tagging: internally tagged status discriminator, rename_all_fields=camelCase, events present on every result variant, verification/lineage/approval typed structs (not JsonValue) on terminal states matching packages/runtime-local/src/sdk/host-protocol.ts -> clean
- `crates/runx-contracts/src/{cli,receipts,registry,tools}.rs`: Deferred contract module homes expose no public types and reference owner specs (rust-contracts-cli-json-parity, rust-receipts-parity, rust-registry-parity, rust-tools-parity) -> clean
- `fixtures/contracts/{capability-execution,host-protocol}/*.json`: Required fixture coverage: 4 capability fixtures, 12 event variants, 5 result statuses, 5 inspect statuses, 4 resolution shapes — all wired via include_str! and validated by check-rust-core-style.mjs coverage check -> clean
- `scripts/generate-rust-contract-fixtures.ts`: Generator uses TS source of truth (packages/runtime-local/src/sdk/capability-execution), assertAsciiObjectKeys rejects non-ASCII keys and non-integer numbers, validates localeCompare vs codepoint ordering agreement to keep BTreeMap/localeCompare parity stable -> clean
- `scripts/check-contract-fixture-key-order.ts`: Script is tsx-invoked TypeScript, validates sorted-key emission with trailing newline, supports --allow-missing -> clean
- `scripts/check-rust-crate-graph.mjs / check-rust-core-style.mjs`: Workspace allows contracts crate with empty runx-* dep set; style script scans contracts src/ for panic/unwrap/HashMap/serde_json::Value/wildcard re-exports — none found -> clean
- `docs/rust-kernel-architecture.md`: Section 17 status updated to reflect typed capability-execution and host-protocol contracts with TS-generated parity fixtures; contracts-first-ordering marker present in §3 -> clean
- `JsonNumber f64 edge cases (very-large whole, very-small fractional, scientific notation)`: Hash divergence vs TS JSON.stringify for non-integer / extreme floats -> clean (Out of scope: assertAsciiObjectKeys rejects non-integer numeric fixture values; the divergence is explicitly deferred to hash-stable-codepoint-cutover. No fixture currently exercises this path.)

Findings:
- none

## Self Eval

Status: not_started
Completeness: none
Architecture fidelity: none
Spec alignment: none
Validation depth: none
Total: none
Second pass performed: none

Threshold: 8

Notes:
none

Improvements:
- none

## Deviations

- none

## Metadata

Estimated effort hours: 18
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- contracts
- sdk
- deferred-cli-json
- host-protocol

## Origin

Source:
- user requested placeholders for all proposed packages and called out that
  `runx-contracts` is load-bearing for SDK and runtime.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- prerequisite_for: rust-sdk-surface-parity

## Harden Rounds

### round-1

Status: failed
Started: 2026-05-18T03:32:23Z
Ended: 2026-05-18T03:32:23Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: The spec is architecturally sound — contracts-before-SDK ordering is correct and the dep-isolation guardrails are well thought out — but several concrete defects will block execution. A Phase 1 migration of the existing inline `JsonValue`/`JsonNumber` from `src/lib.rs` to `src/json.rs` is implicit but not called out (so the Phase 1 rollback to "placeholder-only constants" would lose code that already shipped). The `node scripts/check-contract-fixture-key-order.ts` validation command (v6) is unrunnable as written — Node cannot execute a `.ts` file, and the existing repo pattern uses `tsx`. Phase 4 has a hidden dependency on `rust-cli-feature-parity-matrix`, which is still a draft with no `fixtures/cli-parity/` yet, so Phase 4 cannot ship until that spec produces consumed CLI JSON cases. Hash parity has a real edge case: TS `stableStringify` sorts keys with `localeCompare` while Rust `BTreeMap` sorts by byte order, which can diverge for non-ASCII keys — fixtures must pin this or the policy must restrict keys to ASCII. Host-protocol scope is ambiguous because the TS module mixes serializable wire types with closure types (`Caller`, `AuthResolver`, `HostBoundaryResolver`, `HostBridge`). The `ac5_3` acceptance command runs `cargo test --workspace`, which is slow and couples this spec's pass to unrelated workspace state.

Checks:
- path audit
  - Grounded in: code:crates/runx-contracts/src/lib.rs:1
  - Result: failed
  - Evidence: Phase 1 declares src/json.rs as 'all, exclusive' and src/lib.rs as 'partial, exclusive — Declare modules and explicit re-exports'. The existing lib.rs already contains JsonValue, JsonNumber, JsonObject, Serialize/Deserialize impls, and a test module. Phase 1 therefore requires migrating that code out of lib.rs into json.rs (a real cutover), but the spec describes it as additive. The Phase 1 rollback line 'revert runx-contracts to placeholder-only constants' would also delete already-shipped JsonValue/JsonNumber code from rust-contracts-bootstrap.
- command audit
  - Grounded in: spec_gap:acceptance.v6
  - Result: failed
  - Evidence: v6 and the per-phase fixture-key checks are written as `node scripts/check-contract-fixture-key-order.ts fixtures/contracts`. Node.js cannot execute a TypeScript source file directly. The analogous existing script (`scripts/check-fixture-key-order.ts`) is run via `tsx scripts/check-fixture-key-order.ts` per package.json line 36. The script also does not exist yet; Files Impacted lists it for creation but Phase 1–4 never own it, only Phase 5 changes are recorded in the phases section.
- scope/migration audit
  - Grounded in: code:.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:1
  - Result: failed
  - Evidence: Phase 4 dependencies state 'consumed CLI JSON cases defined by rust-cli-feature-parity-matrix'. That spec is still in `.scafld/specs/drafts/` with `status: draft, harden_status: not_run` and `fixtures/cli-parity/**` does not exist (glob returned no files). Phase 4 ac4_2 hard-codes the identifiers `search_skills|run_skill|resume_run|connect_list` without any oracle to pin them, and no fixture-generator emits them.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase5.ac5_3
  - Result: failed
  - Evidence: ac5_3 runs `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && node scripts/check-rust-core-style.mjs`. This couples the contracts-parity spec's Phase 5 pass to the entire Rust workspace's state (runx-core, runx-runtime, runx-sdk, runx-parser, etc.). Any unrelated workspace breakage blocks Phase 5 acceptance. The crate-scoped equivalents already exist in v2/v3/v4; ac5_3 should be narrowed to `-p runx-contracts` plus the style guard, with a workspace check moved to a separate gate.
- rollback/repair audit
  - Grounded in: code:crates/runx-contracts/src/lib.rs:9
  - Result: failed
  - Evidence: Phase 1 rollback says 'revert runx-contracts to placeholder-only constants and remove added dependencies.' The current placeholder is not constants — it ships JsonValue, JsonNumber, JsonObject, Serialize/Deserialize impls, and tests (lib.rs 184 lines). Rollback as written would erase the rust-contracts-bootstrap deliverable. The rollback target should be the bootstrap-era surface, not 'constants'.
- design challenge
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Result: failed
  - Evidence: TS `stableStringify` sorts entries with `left.localeCompare(right)`. Rust BTreeMap sorts by lexicographic byte order. For ASCII keys these match, but for non-ASCII keys (or composed-vs-decomposed Unicode) they can diverge — which would silently break the cross-language `hashStable` invariant. The spec's Invariants section claims hash helpers must match exactly but does not pin the key-ordering rule beyond 'sorted object keys', and TS uses a locale-sensitive sort.

Questions:
- Is Phase 1 a migration of the existing inline JsonValue/JsonNumber code from src/lib.rs into src/json.rs, or an additive overlay?
  - Grounded in: code:crates/runx-contracts/src/lib.rs:9
  - Recommended answer: Treat Phase 1 as an explicit migration: move JsonValue, JsonNumber, JsonObject, and the serde impls into src/json.rs, and reduce src/lib.rs to module declarations and explicit re-exports (`pub use json::{JsonObject, JsonValue, JsonNumber};`). Call out the move in the Phase 1 Changes block so the diff intent is recorded.
  - If unanswered: Default to the migration interpretation; otherwise the file-ownership labels ('all, exclusive' for json.rs, 'partial, exclusive' for lib.rs) and the rollback wording are mutually inconsistent.
- What is the right way to run the contract fixture key-order check given the repo uses tsx for TypeScript scripts?
  - Grounded in: code:package.json:36
  - Recommended answer: Rewrite v6 and any phase-level fixture-key-order acceptance to `pnpm exec tsx scripts/check-contract-fixture-key-order.ts fixtures/contracts` (matching scripts/check-fixture-key-order.ts at line 1 which already uses ESM/tsx). Decide which phase owns creating the script and add it to that phase's Changes block; today none of Phase 1–4 owns it, and Phase 5 only mentions README/docs updates.
  - If unanswered: Default to making Phase 2 own creating scripts/check-contract-fixture-key-order.ts since fixtures first land in Phase 2, and update v6 to use `pnpm exec tsx`.
- Can Phase 4 ship before `rust-cli-feature-parity-matrix` produces fixtures, given the dependency is currently to a draft spec with no fixtures/cli-parity directory?
  - Grounded in: code:.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:1
  - Recommended answer: Either (a) defer Phase 4 entirely behind that spec and mark contracts-parity 'complete' at Phase 3 + receipts/registry/tools skeleton, treating the CLI JSON port as a follow-up; or (b) narrow Phase 4 to only the CLI JSON cases the Python SDK already exercises and pin those snapshot inputs inside this spec, removing the implicit dependency. Recommended path: option (b), and add a Phase 4 fixture inventory listing the exact request/response cases owned here.
  - If unanswered: Default to deferring Phase 4 and shipping the receipts/registry/tools deferred module homes plus docs as Phase 4, leaving CLI JSON parity for a follow-up spec.
- Should the host-protocol port include only the serializable wire/state shapes, or also the runtime callback types currently in host-protocol.ts?
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:44
  - Recommended answer: Restrict the port to the data shapes: HostRunResult variants (paused/completed/failed/escalated/denied), HostRunState variants (paused + the four terminal states), HostRunVerification, HostRunLineage, HostRunApproval, HostRunOptions JSON projection. Exclude HostSkillExecutor, HostBoundaryResolver, HostStateInspector, HostBridge, Caller, AuthResolver — those are runtime concerns owned by runx-runtime/runx-sdk. Add an explicit 'host-protocol port subset' subsection to the Scope block listing the included and excluded TS exports.
  - If unanswered: Default to data-shapes only; runtime callback types stay in runx-sdk/runx-runtime and never enter runx-contracts.
- How should the cross-language hash invariant handle TS `localeCompare` key ordering vs Rust BTreeMap byte ordering?
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Recommended answer: Pin the key-ordering rule explicitly in the spec: hashed JSON object keys are sorted by Unicode code-point order (the BTreeMap default for `String`). Update `stableStringify` in packages/core/src/util/hash.ts to sort by raw string comparison (`a < b ? -1 : ...`) instead of `localeCompare`, and add a fixture case that exercises non-ASCII keys to lock both sides. Otherwise document a key-charset constraint (ASCII only) at every callsite — but updating stableStringify is the cleaner fix and avoids a quiet contract trap.
  - If unanswered: Default to fixing TypeScript: switch `stableStringify` to code-point comparison and add at least one non-ASCII-key fixture case under fixtures/contracts/capability-execution/.
- Should ac5_3 run the whole workspace, or be scoped to runx-contracts?
  - Grounded in: spec_gap:phases.phase5.ac5_3
  - Recommended answer: Scope ac5_3 to `cargo fmt --all --check && cargo clippy -p runx-contracts --all-targets -- -D warnings && cargo test -p runx-contracts && node scripts/check-rust-core-style.mjs`. The workspace-wide check belongs to rust-parity-ci-governance, not to a single-crate spec, and ties this spec's acceptance to other in-flight Rust work.
  - If unanswered: Default to the crate-scoped variant; the contracts crate's package check (cargo package -p runx-contracts) plus crate-scoped clippy/test is sufficient evidence for this spec.
- What does 'deferred parity marker' look like in runx-contracts/src/{receipts,registry,tools}.rs?
  - Grounded in: spec_gap:phases.phase5
  - Recommended answer: Each module ships as a `//! Deferred contract module home. See <follow-up spec id>.` doc comment plus no public items in this spec, instead of an empty `pub struct Deferred;` placeholder. The follow-up spec id (e.g. `rust-receipts-parity`) goes into the doc comment so reviewers can navigate. The README's 'exposed vs deferred' section lists the modules and their planned owner specs.
  - If unanswered: Default to module-doc-comment-only deferred markers naming the follow-up spec, with zero public items beyond what fixtures back.
- Is `thiserror` always added in Phase 1, or only if/when a concrete error enum is introduced?
  - Grounded in: spec_gap:phases.phase1.changes
  - Recommended answer: Defer `thiserror` until an actual concrete error enum lands. None of the contract types in scope (JsonValue, capability_execution shapes, host_protocol shapes, CLI envelopes) need a Display-deriving error type for v0 — serde errors carry their own. Adding it now bloats the published surface and the dep graph for no consumer. Add it in the phase where the first concrete error enum lands (likely host_protocol resume validation or CLI JSON envelope decoding).
  - If unanswered: Default to omitting thiserror from Phase 1 dependencies; add it conditionally only when the first error enum requires Display.

Design objections:
- `objection-1` high - Phase 1 silently requires migrating existing JSON code, and the rollback text would erase it.
  - Grounded in: code:crates/runx-contracts/src/lib.rs:9
  - Evidence: lib.rs already contains 184 lines of JsonValue/JsonNumber/JsonObject + serde impls + a test module shipped by rust-contracts-bootstrap (completed 2026-05-17). Phase 1 declares src/json.rs as 'all, exclusive' and src/lib.rs as 'partial, exclusive — Declare modules and explicit re-exports.' That is a code move, not an addition. The Phase 1 rollback says 'revert runx-contracts to placeholder-only constants' which describes a state the placeholder is no longer in.
  - Recommendation: Rewrite Phase 1 Changes to make the migration explicit ('Move JsonValue/JsonNumber/JsonObject from src/lib.rs to src/json.rs and re-export via lib.rs') and rewrite the Phase 1 rollback to 'revert to the rust-contracts-bootstrap surface (single-file lib.rs with JsonValue/JsonNumber) and remove serde_json/sha2/thiserror runtime dependencies if added.'
- `objection-2` high - v6 acceptance command cannot run as written; node cannot execute .ts files.
  - Grounded in: spec_gap:acceptance.v6
  - Evidence: v6 is `node scripts/check-contract-fixture-key-order.ts fixtures/contracts`. The existing analogous script scripts/check-fixture-key-order.ts is registered in package.json line 36 as `tsx scripts/check-fixture-key-order.ts`. The contract script is also not owned by any phase's Changes block (it appears only in Files impacted).
  - Recommendation: Change v6 to `pnpm exec tsx scripts/check-contract-fixture-key-order.ts fixtures/contracts` (matching the repo's tsx invocation pattern), add the script to Phase 2's Changes block since that is when fixtures first land, and confirm the script handles a missing fixtures/contracts/<scope> directory cleanly before scopes exist.
- `objection-3` high - Phase 4 depends on a draft spec whose fixtures do not exist yet.
  - Grounded in: code:.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:1
  - Evidence: rust-cli-feature-parity-matrix.md has `status: draft, harden_status: not_run` and fixtures/cli-parity/** glob returns no files. Phase 4 ac4_2 grep regex hard-codes `search_skills|run_skill|resume_run|connect_list` with no oracle defining those method names, and ac4_2 cannot pin a fixture that does not exist.
  - Recommendation: Either narrow Phase 4 to a minimal in-spec set of CLI JSON cases that are owned by this spec (snapshot the current `runx --json` output for search/run/resume/connect list and check it in under fixtures/contracts/cli-json), or remove Phase 4 from this spec and make it a successor task that lands after rust-cli-feature-parity-matrix completes. Pick one before approval.
- `objection-4` medium - Host-protocol scope is ambiguous: TS module mixes wire shapes with runtime callback types.
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:23
  - Evidence: host-protocol.ts exports HostSkillExecutor (line 23), HostBoundaryResolver (44), HostStateInspector (125), HostBridge (184), and consumes Caller/AuthResolver from runner-local. None of those are serializable; they hold closures and runtime state. The spec only says 'host result and state projection contracts.'
  - Recommendation: Add an explicit subsection to the Scope block listing the TS exports that are included (HostRunResult/State variants, HostRunVerification, HostRunLineage, HostRunApproval, ResolutionRequest/Response shape) and excluded (HostBridge, HostSkillExecutor, HostBoundaryResolver, HostStateInspector, Caller, AuthResolver). Make it a review-gate finding so the contracts crate cannot accidentally absorb runtime types.
- `objection-5` medium - TS stableStringify uses localeCompare; Rust BTreeMap uses byte order. Hash parity can silently diverge on non-ASCII keys.
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Evidence: packages/core/src/util/hash.ts line 12 sorts entries with `left.localeCompare(right)`. The spec invariant says 'stable JSON stringification with sorted object keys' but does not pin the comparison function. BTreeMap<String, _> sorts by Rust's String Ord, which is byte-wise on UTF-8 — equivalent for ASCII, divergent for non-ASCII (and for some composed-vs-decomposed forms).
  - Recommendation: Replace `localeCompare` with raw string comparison (`a < b ? -1 : a > b ? 1 : 0`) in packages/core/src/util/hash.ts as part of Phase 2, then add a fixture case under fixtures/contracts/capability-execution/ that uses at least one non-ASCII or non-alphanumeric key to lock both sides. Record the change as a backwards-compatible normalization in the spec's Invariants.
- `objection-6` medium - ac5_3 runs the whole workspace, coupling this spec's acceptance to unrelated crates.
  - Grounded in: spec_gap:phases.phase5.ac5_3
  - Evidence: ac5_3 = `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && node scripts/check-rust-core-style.mjs`. v2/v3/v4 already cover crate-scoped formatting, clippy, tests, and the style guard. The workspace fan-out adds latency and false negatives when other crates churn.
  - Recommendation: Narrow ac5_3 to `cargo fmt --all --check && cargo clippy -p runx-contracts --all-targets -- -D warnings && cargo test -p runx-contracts && node scripts/check-rust-core-style.mjs`. Workspace-wide gating belongs to rust-parity-ci-governance, not this spec.
- `objection-7` low - thiserror dependency is added speculatively in Phase 1 with no concrete error enum justifying it.
  - Grounded in: spec_gap:phases.phase1.changes
  - Evidence: Phase 1 Changes adds `thiserror` 'if concrete error enums need derived Display.' None of the Phase 1 contract types (JsonValue/JsonNumber) require a Display-deriving error — serde::de::Error and serde::ser::Error suffice. Adding thiserror pre-emptively bloats the published crate surface and dep graph.
  - Recommendation: Drop thiserror from Phase 1 deps. Add it in the phase where the first concrete error enum lands (likely Phase 3's host_protocol resume validation or Phase 4's CLI JSON envelope decoding), with a one-line justification.

Recommended edits:
- Phase 1 Changes
  - Grounded in: code:crates/runx-contracts/src/lib.rs:9
  - Recommendation: Make the JSON code migration explicit: 'Move JsonValue, JsonNumber, JsonObject, and the serde Number serialize/deserialize logic from src/lib.rs into src/json.rs. Reduce src/lib.rs to module declarations and explicit re-exports (`pub use json::{JsonObject, JsonValue, JsonNumber}`). Move the existing JsonValue/JsonNumber tests into src/json.rs alongside the implementation.'
- Phase 1 Rollback (Rollback Commands)
  - Grounded in: code:crates/runx-contracts/src/lib.rs:9
  - Recommendation: Rewrite Phase 1 rollback to 'restore the rust-contracts-bootstrap surface — single-file lib.rs containing JsonValue/JsonNumber and serde impls — and remove serde_json (promoted from dev-dep to dep), sha2, and thiserror if added.' 'Placeholder-only constants' no longer describes the starting state.
- Acceptance v6
  - Grounded in: code:package.json:36
  - Recommendation: Rewrite v6 as `pnpm exec tsx scripts/check-contract-fixture-key-order.ts fixtures/contracts`. Add the script to Phase 2 Changes (since fixtures first land in Phase 2) and confirm it tolerates a missing scope directory before the corresponding phase runs.
- Phase 4 Dependencies
  - Grounded in: code:.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:1
  - Recommendation: Resolve the dependency-on-a-draft: either (1) move Phase 4 to a successor spec to be opened once rust-cli-feature-parity-matrix completes, replacing the current Phase 4 with the receipts/registry/tools deferred module homes; or (2) explicitly own a minimal in-spec set of CLI JSON cases (search/run/resume/connect list snapshots) under fixtures/contracts/cli-json and document this spec as the temporary oracle. Pick (1) by default.
- Scope (In/Out scope)
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:23
  - Recommendation: Add 'Host-protocol port subset' listing included TS exports (HostRunResult variants, HostRunState variants, HostRunVerification, HostRunLineage, HostRunApproval, HostRunOptions JSON projection, ResolutionRequest/Response shape) and excluded TS exports (HostBridge, HostSkillExecutor, HostBoundaryResolver, HostStateInspector, Caller, AuthResolver). This blocks runtime callback types from leaking into runx-contracts.
- Invariants (hash helpers)
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Recommendation: Pin the key-ordering rule: 'Hashed JSON object keys are sorted by Unicode code-point order (raw string comparison). Update packages/core/src/util/hash.ts to replace localeCompare with raw comparison as part of Phase 2, and add a non-ASCII-key fixture case under fixtures/contracts/capability-execution/.'
- Phase 5 Acceptance ac5_3
  - Grounded in: spec_gap:phases.phase5.ac5_3
  - Recommendation: Narrow ac5_3 to `cargo fmt --all --check && cargo clippy -p runx-contracts --all-targets -- -D warnings && cargo test -p runx-contracts && node scripts/check-rust-core-style.mjs`. Move workspace-wide gating to rust-parity-ci-governance.
- Phase 5 Changes (deferred modules)
  - Grounded in: spec_gap:phases.phase5
  - Recommendation: Define the deferred-marker convention: each of receipts.rs/registry.rs/tools.rs ships as a module with only a `//! Deferred contract module home. See <follow-up spec id>.` doc comment and no public items in this spec. README documents the planned owner spec for each module.
- Phase 1 Cargo.toml deps
  - Grounded in: spec_gap:phases.phase1.changes
  - Recommendation: Drop thiserror from Phase 1 deps. Promote serde_json from dev-dep to dep, add sha2. Defer thiserror to the phase that introduces the first concrete error enum.

### round-2

Status: failed
Started: 2026-05-18T03:40:32Z
Ended: 2026-05-18T03:40:32Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 harden: the draft has absorbed all six round-1 findings. Phase 1 now explicitly migrates `JsonValue`/`JsonNumber`/`JsonObject`/serde impls/tests into `src/json.rs` (and the rollback preserves the `rust-contracts-bootstrap` surface). The fixture-key-order acceptance (`v6`) and Phase 1 acceptance (`ac1_3`) correctly use `pnpm exec tsx`, and the script is owned by Phase 1 changes. Phase 5 acceptance (`ac5_3`) is narrowed to `-p runx-contracts`. Host-protocol scope explicitly excludes `Caller`, `AuthResolver`, `HostBoundaryResolver`, `HostBridge`, `HostStateInspector`. The non-ASCII hash divergence between TS `localeCompare` and Rust BTreeMap byte-order is mitigated by a fixture-generator ban on non-ASCII keys (`ac2_4`) and an Assumptions entry, with the underlying `stableStringify` fix deferred. Phase 4 has a hard upstream gate (`ac4_0`) on `rust-cli-feature-parity-matrix` status + `fixtures/cli-parity/`. Residual issues: (1) Phase 1 Cargo.toml change line bundles `serde, serde_json, sha2, thiserror` with a trailing conditional that is grammatically ambiguous — `sha2` is not used until Phase 2 and `thiserror` may never be needed; (2) the underlying `packages/core/src/util/hash.ts` `localeCompare` hazard persists at TS runtime (mitigation is only at fixture-generation time, not at callers like `capability-execution.ts`) without a tracked follow-up; (3) `ac4_0` silently relies on `jq` being installed; (4) `ac5_2` doc-grep regex `runx-contracts.*runx-sdk|SDK v0.*runx-contracts|contracts.*before SDK` is brittle to phrasing changes; (5) `Cargo.toml include = ["Cargo.toml", "README.md", "src/**/*.rs"]` excludes `tests/**`, so the `tests/*.rs` files plus any `fixtures/contracts/**` data are not packaged — fine for `cargo package` (libraries only) but should be made explicit so implementers don't try to `include_str!` workspace-root fixtures.

Checks:
- path audit
  - Grounded in: code:crates/runx-contracts/src/lib.rs:9
  - Result: passed
  - Evidence: All Phase 1–5 declared paths exist as files-to-create or pre-existing surfaces. `crates/runx-contracts/{Cargo.toml,src/lib.rs,README.md}` exist (bootstrap deliverable). Phase 1 now explicitly states 'Move the existing bootstrap JsonValue, JsonNumber, JsonObject, serde impls, and unit tests out of lib.rs into json.rs; do not discard that shipped surface.' Workspace members `crates/runx-{core,parser,receipts,runtime,sdk}/Cargo.toml` all exist so the `cargo --manifest-path crates/Cargo.toml -p runx-contracts` invocations resolve. The TS test files referenced in v1 (`packages/contracts/src/index.test.ts`, `packages/contracts/src/handoff-contracts.test.ts`, `packages/runtime-local/src/sdk/capability-execution.test.ts`) all exist.
- command audit
  - Grounded in: spec_gap:acceptance.v6
  - Result: passed
  - Evidence: v6 now reads `pnpm exec tsx scripts/check-contract-fixture-key-order.ts fixtures/contracts` (matching the existing `scripts/check-fixture-key-order.ts` tsx pattern). Phase 1 changes explicitly own `scripts/check-contract-fixture-key-order.ts` and require `pnpm exec tsx` invocation. `ac1_3` adds `--allow-missing` so the check works before fixture scopes land. v2/v3/ac1_1/ac1_2/ac2_1/ac3_1/ac4_1 all use the manifest-path form. v7 (negative dep boundary) and v8 (`cargo package --allow-dirty`) are sound. The only soft assumption is that the implementer wires `--allow-missing` into the new script — that is part of Phase 1's deliverable, not a spec defect.
- scope/migration audit
  - Grounded in: code:.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:1
  - Result: passed
  - Evidence: Phase 4 still depends on the draft `rust-cli-feature-parity-matrix` (status: draft, harden_status: not_run; `fixtures/cli-parity/` does not yet exist), but the spec now gates Phase 4 with `ac4_0 = scafld status --json --task rust-cli-feature-parity-matrix | jq -e '.status == "completed" or .status == "complete"' && test -d fixtures/cli-parity`. Phases 1–3 can ship without it. Host-protocol scope explicitly excludes `Caller`, `AuthResolver`, `HostBoundaryResolver`, `HostBridge`, `HostStateInspector` in both the Scope block and Phase 3 changes — matching the closure surfaces at packages/runtime-local/src/sdk/host-protocol.ts:23,44,125,184. Cross-spec coupling is now an explicit hard gate rather than an implicit dependency.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase5.ac5_3
  - Result: passed
  - Evidence: ac5_3 is now `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-contracts --all-targets -- -D warnings && cargo test --manifest-path crates/Cargo.toml -p runx-contracts && node scripts/check-rust-core-style.mjs`. The workspace-wide fan-out is gone; only `runx-contracts` is checked. `ac4_0` correctly gates Phase 4 before any CLI fixture writes can happen. `ac2_3` runs after Phase 2 capability_execution.rs lands and uses a sound positive/negative pair (`rg sha2 ... && ! rg sha2 ...crates/runx-sdk/...`). The current `crates/runx-sdk/Cargo.toml` has only `runx-contracts.workspace = true` and no `sha2`, so the negative check is currently satisfiable and will remain so as long as SDK consumes contract helpers.
- rollback/repair audit
  - Grounded in: code:crates/runx-contracts/src/lib.rs:9
  - Result: passed
  - Evidence: Phase 1 rollback now reads 'revert runx-contracts to the bootstrap-era JSON surface (JsonValue, JsonNumber, JsonObject, serde impls, and tests) and remove only the new module declarations/dependencies added by this spec. Do not delete the rust-contracts-bootstrap deliverable.' This matches the actual 184-line state at crates/runx-contracts/src/lib.rs (JsonObject + JsonValue + JsonNumber + Serialize/Deserialize impls + #[cfg(test)] mod tests). Phases 2–5 rollbacks are additive removals of files/fixtures/modules whose creation is owned by that phase, so reverting per-phase is mechanically clean.
- design challenge
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Result: passed
  - Evidence: The TS `stableStringify` at packages/core/src/util/hash.ts line 12 still sorts entries with `left.localeCompare(right)`. Rust BTreeMap<String,_> sorts by raw byte order on UTF-8. These match for ASCII but diverge on some non-ASCII / composed-vs-decomposed Unicode. The spec acknowledges the gap in Assumptions ('Stable hash parity must not rely on Rust BTreeMap ordering as a substitute for TypeScript localeCompare. Phase 2 fixtures pin hash output for sorted ASCII contract keys. Non-ASCII object keys are explicitly rejected by the fixture generator in this phase until a follow-up implements and proves a locale-compatible ordering rule.') and enforces it via ac2_4 grep for ascii/non-ASCII/hashStable markers. The crate's separation from `runx-core`/`runx-runtime`/`runx-sdk` is sound (workspace dep graph confirms runx-sdk depends on runx-contracts, not the reverse). Host-protocol callbacks are explicitly out-of-scope. The contracts-first ordering for SDK v0 is documented. The design is the right architectural move for now; the residual is that the TS-side hash hazard at runtime callers persists — see design_objections.

Questions:
- Should `sha2` move to Phase 2 dependencies instead of Phase 1, since no hashing code lands until Phase 2's capability_execution.rs?
  - Grounded in: spec_gap:phases.phase1.changes
  - Recommended answer: Yes — keep Phase 1 deps minimal (`serde` only; promote `serde_json` from dev-dep to dep only if json.rs needs runtime serde_json). Defer `sha2` to Phase 2 alongside the first SHA-256 call site, and defer `thiserror` to whichever phase introduces the first concrete error enum (Phase 3 resume validation or Phase 4 CLI envelope decoding). Phase 1 adding `sha2` is harmless but it spends published-surface budget before any consumer code uses it.
  - If unanswered: Leave Phase 1 deps as written (add `serde`, `sha2`, conditionally `thiserror`) but rewrite the Cargo.toml change line to remove the ambiguous trailing conditional and explicitly state which deps are unconditional vs phase-conditional.
- Should the `localeCompare` → code-point comparison fix in `packages/core/src/util/hash.ts` be tracked as a named follow-up spec, given that the current mitigation only blocks non-ASCII keys at fixture generation, not at TS runtime callers (e.g., `packages/runtime-local/src/sdk/capability-execution.ts` at lines 67/83/90 which all call `hashStable` on capability records that could contain user-controlled keys)?
  - Grounded in: code:packages/runtime-local/src/sdk/capability-execution.ts:67
  - Recommended answer: Yes — add a 'Follow-up tracking' subsection naming a successor spec id (e.g., `hash-stable-codepoint-cutover`) that swaps `localeCompare` for raw string comparison and adds a non-ASCII-key fixture. Until that spec lands, document the TS-runtime hazard in the README so consumers know that non-ASCII object keys passed to `hashStable` produce TS-only hashes that any future Rust port will not reproduce.
  - If unanswered: Add a single line to the Assumptions / Risks block: 'Runtime callers of hashStable that accept user-controlled object keys remain at risk of TS-only hash output until a successor spec fixes stableStringify; this spec only gates fixture generation.'
- Should `ac4_0` document the `jq` runtime dependency or use a `scafld` flag that does not require jq?
  - Grounded in: spec_gap:phases.phase4.ac4_0
  - Recommended answer: Replace the jq pipe with a scafld-native check if one exists (e.g., `scafld status --task rust-cli-feature-parity-matrix --require-status completed`); if not, add a note to the Phase 4 changes block listing `jq` as a required local tool and add the same to AGENTS.md / repo prerequisites. Hidden tool dependencies in acceptance commands surprise CI environments.
  - If unanswered: Document the jq dependency in the spec's Dependencies block as a soft prerequisite and leave the command as written.
- Should `Cargo.toml` `include` be expanded to cover `tests/**/*.rs` (so the published crate contains its test sources), or should the spec explicitly state that fixture-based tests live under `tests/` for workspace-only consumption and are not packaged?
  - Grounded in: code:crates/runx-contracts/Cargo.toml:13
  - Recommended answer: Leave `include` as `["Cargo.toml", "README.md", "src/**/*.rs"]`. Add a Phase 5 README note explaining that fixture tests live at workspace root (`fixtures/contracts/**`) and at `crates/runx-contracts/tests/`, and that they are intentionally excluded from the published crate so downstream consumers do not need to vendor workspace fixtures. Implementers should load fixtures with `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/contracts/..."))` for workspace builds, and confirm `cargo package -p runx-contracts --allow-dirty` succeeds before claiming Phase 5 done.
  - If unanswered: Leave the include list unchanged and add a single explanatory sentence to Phase 5 README changes.

Design objections:
- `objection-2-1` medium - Hash-parity mitigation lives only at fixture-generation; TS runtime callers of hashStable on non-ASCII content still emit hashes a future Rust port cannot reproduce.
  - Grounded in: code:packages/runtime-local/src/sdk/capability-execution.ts:67
  - Evidence: packages/runtime-local/src/sdk/capability-execution.ts calls `hashStable({...})` at lines 67, 83, and 90 on capability records that contain transport-supplied actor/transport/idempotency fields. packages/core/src/util/hash.ts line 12 sorts object entries with `left.localeCompare(right)`. The spec's Assumptions block correctly notes the mismatch and bans non-ASCII keys at fixture generation (ac2_4 grep enforces). But the mitigation is fixture-only — any runtime caller that builds a capability record with non-ASCII object keys at runtime (which TS does not reject) will produce a hash that the eventual Rust port cannot match. This is a deferred-correctness gap, not a hard defect, but the spec should name the follow-up.
  - Recommendation: Add a 'Follow-up specs' subsection naming the locale-compatible ordering fix (e.g., `hash-stable-codepoint-cutover`) and add a README note explaining the runtime caller hazard. Optionally, add a TS-side check in `capability-execution.ts` that rejects non-ASCII object keys at runtime until the cutover lands — this would make the contract guarantee symmetrical with the fixture generator's restriction.
- `objection-2-2` low - Phase 1 Cargo.toml change line bundles four deps with an ambiguous trailing conditional; `sha2` is not used until Phase 2 and `thiserror` may never be used.
  - Grounded in: spec_gap:phases.phase1.changes
  - Evidence: Phase 1 reads 'Add `serde`, `serde_json`, `sha2`, and `thiserror` if concrete error enums need derived `Display`; keep edition 2024, workspace lints, and package metadata.' The grammar attaches the `if`-clause to thiserror but a careful reader could parse it as applying to all four. Phase 2 is where `capability_execution.rs` introduces the first `Sha256/Digest` call, so `sha2` has no Phase 1 consumer. Adding a dep before a consumer exists wastes published-surface budget on the first publishable revision.
  - Recommendation: Rewrite Phase 1 Cargo.toml change as: 'Add `serde` as a runtime dependency, promote `serde_json` from dev-dependency to runtime dependency if json.rs requires runtime serde_json (otherwise keep dev-only), and keep edition 2024, workspace lints, and package metadata.' Then add `sha2` to Phase 2's `capability_execution.rs` change line as an explicit dep addition, and add `thiserror` only to the phase that introduces the first concrete error enum.
- `objection-2-3` low - ac4_0 silently depends on `jq` being installed in the operator's environment.
  - Grounded in: spec_gap:phases.phase4.ac4_0
  - Evidence: ac4_0 = `scafld status --json --task rust-cli-feature-parity-matrix | jq -e '.status == "completed" or .status == "complete"' && test -d fixtures/cli-parity`. The jq dependency is unstated. Most dev environments have jq, but CI images and ephemeral runners may not; an empty `which jq` would silently misreport the gate as failed.
  - Recommendation: Either use a scafld-native flag that exits non-zero on status mismatch (`scafld status --require-status completed --task rust-cli-feature-parity-matrix`) or add `jq` to the spec's Dependencies block / AGENTS.md prerequisites. Document the tool requirement once, not implicitly in a grep pipe.
- `objection-2-4` low - ac5_2 doc-grep regex is brittle to phrasing changes.
  - Grounded in: spec_gap:phases.phase5.ac5_2
  - Evidence: ac5_2 = `rg -n 'runx-contracts.*runx-sdk|SDK v0.*runx-contracts|contracts.*before SDK' docs/rust-kernel-architecture.md crates/runx-contracts/README.md .scafld/specs/drafts/rust-sdk-surface-parity.md`. Any of these alternatives can match unrelated prose, and small editorial changes to the docs (e.g., 'runx-sdk depends on runx-contracts' phrased instead as 'the SDK depends on contracts') break the gate. The check also references a draft spec at a fragile path; if `rust-sdk-surface-parity` archives or moves between drafts/approved/archive, the file won't be at that path.
  - Recommendation: Define a canonical marker string in the README (e.g., `<!-- contracts-first-ordering: runx-contracts ships before runx-sdk Phase 2 -->`) and grep for that exact marker. Replace the draft-path-pinned ripgrep with a path-glob like `.scafld/specs/**/rust-sdk-surface-parity.md` so the gate survives lifecycle transitions.
- `objection-2-5` low - Tests under crates/runx-contracts/tests/ are not in the Cargo.toml include list and fixtures live at workspace root; implementers may waste cycles trying to bundle them.
  - Grounded in: code:crates/runx-contracts/Cargo.toml:13
  - Evidence: crates/runx-contracts/Cargo.toml line 13 sets `include = ["Cargo.toml", "README.md", "src/**/*.rs"]`. The spec's Phase 2/3/4 tests live under `crates/runx-contracts/tests/` and consume fixtures at workspace root `fixtures/contracts/**`. `cargo package -p runx-contracts --allow-dirty` (v8 / ac5_1) only builds the library by default, so this works in practice, but the spec does not explain the choice, and a contributor might assume the tests are packaged.
  - Recommendation: Add a single line to Phase 5 README changes: 'Fixture-driven tests under crates/runx-contracts/tests/ load JSON from workspace root fixtures/contracts/**. They are deliberately excluded from the published crate via the Cargo.toml include allowlist. Downstream Rust consumers run their own integration tests against contract types; they do not need vendored fixtures.'

Recommended edits:
- Phase 1 Changes (Cargo.toml line)
  - Grounded in: spec_gap:phases.phase1.changes
  - Recommendation: Rewrite the Phase 1 Cargo.toml change line to disambiguate dependency additions: 'Add `serde` as a runtime dependency. Keep `serde_json` as a dev-dependency in Phase 1 (promote to runtime dependency in the phase that first needs `serde_json::to_string` at runtime). Defer `sha2` to Phase 2 alongside capability_execution.rs. Defer `thiserror` to the phase that introduces the first concrete error enum. Keep edition 2024, workspace lints, and package metadata.'
- Phase 2 Changes
  - Grounded in: spec_gap:phases.phase2.changes
  - Recommendation: Add `crates/runx-contracts/Cargo.toml` (partial, exclusive) to the Phase 2 changes list with the note 'Promote `serde_json` from dev-dep to runtime dep if hashing code needs it; add `sha2` as a runtime dependency.' This keeps dependency additions co-located with the code that requires them and makes Phase 1's rollback (the bootstrap-era surface with only `serde`) cleanly recoverable.
- Assumptions / Risks (locale ordering)
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Recommendation: Add a sentence to the Assumptions block: 'The fixture-generator non-ASCII-key ban does not protect TS runtime callers of hashStable. Any TS callsite that hashes user-controlled object keys (e.g., packages/runtime-local/src/sdk/capability-execution.ts at lines 67/83/90) can produce a TS-only hash that a future Rust port cannot reproduce. A successor spec (`hash-stable-codepoint-cutover`) is required to replace `localeCompare` with raw code-point comparison and lock both languages to a non-ASCII fixture case.' Mirror in Risks (Medium).
- Phase 4 Acceptance ac4_0
  - Grounded in: spec_gap:phases.phase4.ac4_0
  - Recommendation: Either replace the jq pipe with a scafld-native exit-code check (e.g., `scafld status --task rust-cli-feature-parity-matrix --require-status completed && test -d fixtures/cli-parity`), or add `jq` to the spec Dependencies block as a required local tool. Hidden tool deps in acceptance commands surprise CI.
- Phase 5 Changes (README)
  - Grounded in: code:crates/runx-contracts/Cargo.toml:13
  - Recommendation: Add a README subsection covering: (a) which Cargo.toml fields control packaged sources (`include` allowlist); (b) why test sources and workspace-root fixtures are excluded from the published crate; (c) the canonical contracts-first-ordering marker line that ac5_2 should grep for; (d) the list of exposed vs deferred contract modules with their planned owner specs (e.g., `receipts.rs` → future `rust-receipts-parity`).
- Phase 5 Acceptance ac5_2
  - Grounded in: spec_gap:phases.phase5.ac5_2
  - Recommendation: Replace the brittle `runx-contracts.*runx-sdk|SDK v0.*runx-contracts|contracts.*before SDK` regex with a canonical marker check, e.g., `rg -n 'contracts-first-ordering:' docs/rust-kernel-architecture.md crates/runx-contracts/README.md && rg -n 'contracts-first-ordering:' .scafld/specs/**/rust-sdk-surface-parity.md`. The marker is editorial-stable and the path glob survives draft → approved → archive transitions.
- Phase 5 Changes (deferred module convention)
  - Grounded in: spec_gap:phases.phase5.changes
  - Recommendation: Define the deferred-module convention explicitly: each of `receipts.rs`, `registry.rs`, `tools.rs` ships with only a `//! Deferred contract module home. Owner: <follow-up spec id>.` module doc comment and zero public items in this spec. README's 'exposed vs deferred' table names the planned owner spec for each module so reviewers can navigate without git archaeology.

### round-3

Status: failed
Started: 2026-05-18T03:49:31Z
Ended: 2026-05-18T03:49:31Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Spec is structurally strong: clear boundary against IO/runtime, contracts-first ordering, per-phase rollback, and fixture-backed parity. Three real defects block approval. (1) Phase 2 silently changes `stableStringify` from `localeCompare` to code-point order in `packages/core/src/util/hash.ts`, which is consumed by receipts, idempotency keys, capability content hashes, knowledge entry IDs, and A2A task IDs; the spec describes the change without a migration story or proof that existing persisted hashes are unaffected. (2) Phase 5 acceptance `ac5_2` requires the `contracts-first-ordering:` marker to be present in `.scafld/specs/.../rust-sdk-surface-parity.md`, but Phase 5's Changes list does not include that spec, so the acceptance cannot pass within declared scope. (3) Phase 3's host protocol types transitively depend on `ExecutionEvent`, `ResolutionRequest`, `ResolutionResponse` from runtime/core, but the change list does not enumerate them; without a closure, Phase 3 risks unbounded growth or shipping unbuildable Rust shapes.

Checks:
- path audit
  - Grounded in: code:scripts/check-fixture-key-order.ts:1
  - Result: passed
  - Evidence: A scripts/check-fixture-key-order.ts already exists for kernel fixtures and reuses stableFixtureJson (Object.keys().sort()) from scripts/generate-kernel-parity-fixtures.ts. The new scripts/check-contract-fixture-key-order.ts in Phase 1 is intentionally distinct — different fixture root — but the existing helper (stableFixtureJson at scripts/generate-kernel-parity-fixtures.ts:279) is reusable and should be cited as the canonical key-order utility to avoid divergent implementations.
- command audit
  - Grounded in: spec_gap:phases.acceptance
  - Result: failed
  - Evidence: Phase 4 ac4_0 invokes `scafld status rust-cli-feature-parity-matrix --json` and requires status `completed`. The dependency spec is itself a draft (.scafld/specs/drafts/rust-cli-feature-parity-matrix.md, harden_status: not_run), so Phase 4 acceptance is blocked by an external spec lifecycle, not by this spec's own work. Validation v8 `cargo package` requires all phases finished, including the unreachable Phase 4. Need explicit handling: either gate v8 on phases 1-3 + Phase 5 when Phase 4 is intentionally deferred, or block this spec until rust-cli-feature-parity-matrix is at minimum approved.
- scope/migration audit
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Result: failed
  - Evidence: Phase 2 swaps `localeCompare` for a code-point comparator in stableStringify. hashStable is called at packages/core/src/receipts/index.ts:472 (input_hash), packages/runtime-local/src/sdk/capability-execution.ts:67/83/90 (intent/trigger/content hash), packages/runtime-local/src/runner-local/runner-helpers.ts:93 (idempotencyKeyHash), packages/runtime-local/src/runner-local/execution-semantics.ts:80 (value_hash), packages/adapters/src/a2a/index.ts:146/214/215 (taskId, message_hash, output_hash), packages/core/src/knowledge/{file-thread,local-store}.ts (entry_id), packages/cli/src/{scaffold,authoring-utils}.ts. Default JS localeCompare differs from code-point order for mixed-case ASCII keys (e.g. 'A' vs 'a') and for many non-ASCII pairs. The spec acknowledges the cutover but provides no migration plan or fixture proof that current production payloads are unaffected. Persisted receipts, idempotency dedup, and knowledge entry ids could silently diverge.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase5.acceptance
  - Result: failed
  - Evidence: Phase 5 ac5_2 runs `rg -n 'contracts-first-ordering:' .scafld/specs --glob 'rust-sdk-surface-parity.md'`. Phase 5 Changes lists crates/runx-contracts/src/{receipts,registry,tools}.rs, crates/runx-contracts/README.md, and docs/rust-kernel-architecture.md. The rust-sdk-surface-parity.md draft is not declared as a Phase 5 change and currently lacks the marker (grep over .scafld/specs returns only rust-contracts-parity.md). The acceptance is unreachable without scope expansion or marker relocation.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: failed
  - Evidence: Per-phase rollback covers Rust deletions, but the Phase 2 change to packages/core/src/util/hash.ts is a shared file that fans out into receipts and idempotency hashes. Rollback for Phase 2 says only `remove capability execution module, fixtures, tests, and fixture generator scope` — it does not say `restore localeCompare ordering in stableStringify`. If Phase 2 ships and is later reverted, the hash-ordering cutover would silently remain in the TypeScript side without the Rust side that justified it.
- design challenge
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:1
  - Result: failed
  - Evidence: Host result types (HostPausedResult, HostCompletedResult, ...) reference ResolutionRequest, ResolutionResponse, and ExecutionEvent imported from @runxhq/core/executor and ../runner-local/index.js. Phase 3's Changes list only `crates/runx-contracts/src/host_protocol.rs`, with no enumeration of those transitive serializable types. Without an explicit subset, Phase 3 either (a) grows to port executor/runner-local wire types it has not declared, or (b) ships host result structs whose `request`, `requests`, and `events` fields are typed as the contract's JsonValue or untyped, undermining the parity goal. Need an explicit transitive type closure list.

Questions:
- What happens to existing persisted hashes (receipt input_hash/value_hash, idempotencyKeyHash, capability intent/trigger/content hashes, knowledge entry_id, A2A taskId) when Phase 2 replaces localeCompare ordering with code-point ordering in stableStringify?
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Recommended answer: Add a Phase 2 sub-task that (a) audits all hashStable call sites and enumerates which payload shapes contain mixed-case ASCII or non-ASCII keys, (b) provides a TypeScript-only fixture proving the new comparator yields identical output for every observed payload shape today, and (c) documents that the cutover is a behavior change for payloads outside that observed shape set. Add the Phase 2 rollback note `restore localeCompare ordering in packages/core/src/util/hash.ts` so reverting Phase 2 also reverts the TypeScript-side cutover.
  - If unanswered: Default to splitting the comparator cutover into its own predecessor spec so the contracts work does not bundle a hash-output behavior change with crate construction.
- Phase 5 acceptance ac5_2 requires `contracts-first-ordering:` in rust-sdk-surface-parity.md, but Phase 5 does not list that spec as a change. Should ac5_2 instead grep only files Phase 5 owns, or should the SDK spec be added to Phase 5's Changes section?
  - Grounded in: spec_gap:phases.phase5
  - Recommended answer: Drop the SDK spec from the ac5_2 regex and rely on rust-sdk-surface-parity owning its own marker placement when it is hardened/built. Keep the marker in docs/rust-kernel-architecture.md and crates/runx-contracts/README.md as the canonical homes.
  - If unanswered: Add rust-sdk-surface-parity.md (partial, shared) to Phase 5 Changes.
- Phase 3 hosts HostPausedResult/etc. which transitively reference ExecutionEvent, ResolutionRequest, and ResolutionResponse. Which of those transitive shapes are in scope for runx-contracts in this spec, and which stay as opaque JsonValue placeholders until a later spec?
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:1
  - Recommended answer: Enumerate the transitive wire types Phase 3 must port (likely ExecutionEvent, ResolutionRequest, ResolutionResponse subset consumed by SDK v0) inside Phase 3 Changes, OR explicitly state Phase 3 models these as `JsonValue` placeholders with deferred-parity markers similar to receipts/registry/tools.
  - If unanswered: Default to modelling the transitive types as JsonValue with deferred-parity markers so Phase 3 stays bounded.
- Phase 4 cannot start until rust-cli-feature-parity-matrix is `completed`, but that spec is currently a draft with harden_status: not_run. Should this spec gate approval on the matrix reaching `approved`, or explicitly accept Phase 4 staying pending while Phases 1-3 + 5 ship?
  - Grounded in: spec_gap:dependencies
  - Recommended answer: Make Phase 4 explicitly optional within this spec by adding an `awaiting:<task>` gate and allowing v8 `cargo package` to run after Phases 1-3 + 5 when Phase 4 is deferred; the SDK v0 spec already names which CLI methods it needs.
  - If unanswered: Default to blocking this spec's approval until rust-cli-feature-parity-matrix is at minimum approved, to keep dod4 honest.

Design objections:
- `objection-1` high - Hash-ordering cutover is bundled with crate construction without migration evidence
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Evidence: Phase 2 replaces localeCompare ordering in stableStringify, which feeds hashStable, which is the source of input_hash, value_hash, idempotencyKeyHash, capability content/intent/trigger hashes, knowledge entry_id, and A2A taskId across packages/core, packages/runtime-local, packages/adapters, and packages/cli. The spec presents this as a deterministic ordering improvement but offers no proof that current payloads remain hash-stable, nor a plan for receipts written before the cutover.
  - Recommendation: Either split the comparator change into a predecessor spec with explicit `hashStable parity preserved` evidence, or add a Phase 2 sub-task that produces a TypeScript-only fixture proving identical output for every observed payload shape, plus a documented exception list for payload shapes (e.g. mixed-case or non-ASCII keys) whose hashes will change.
- `objection-2` medium - Phase 3 host protocol does not enumerate transitive serializable types
  - Grounded in: spec_gap:phases.phase3.changes
  - Evidence: HostPausedResult.requests is `readonly ResolutionRequest[]`; all host results carry `readonly ExecutionEvent[]`. host-protocol.ts:1-3 imports those from @runxhq/core/executor and ../runner-local/index.js. Phase 3 Changes lists only host_protocol.rs and its fixture pair; no Rust shape for ResolutionRequest/ResolutionResponse/ExecutionEvent is declared.
  - Recommendation: Either enumerate the consumed wire subset of those types in Phase 3 Changes, or explicitly state they are modelled as JsonValue with a deferred-parity marker like receipts/registry/tools.
- `objection-3` medium - ac5_2 acceptance references a spec not in Phase 5's change list
  - Grounded in: spec_gap:phases.phase5.acceptance
  - Evidence: ac5_2 (`rg -n 'contracts-first-ordering:' .scafld/specs --glob 'rust-sdk-surface-parity.md'`) requires the marker to exist in rust-sdk-surface-parity.md, which is not in Phase 5 Changes. Repo-wide grep currently finds the marker only in this spec.
  - Recommendation: Either drop the SDK spec from ac5_2 and place the marker only in docs/rust-kernel-architecture.md and the contracts README, or add rust-sdk-surface-parity.md to Phase 5 Changes.
- `objection-4` medium - Phase 2 rollback does not address the shared TypeScript hash change
  - Grounded in: spec_gap:rollback.phase2
  - Evidence: Rollback for Phase 2 says `remove capability execution module, fixtures, tests, and fixture generator scope`. It omits restoring localeCompare ordering in packages/core/src/util/hash.ts, even though that file is a Phase 2 declared change in the changeset (`packages/core/src/util/hash.ts (partial, shared)`).
  - Recommendation: Extend Phase 2 rollback to restore localeCompare ordering in packages/core/src/util/hash.ts so the TypeScript-side cutover is reversible alongside the Rust deletions.
- `objection-5` low - ac1_3 substring check on the new key-order script is trivially satisfiable
  - Grounded in: spec_gap:phases.phase1.acceptance
  - Evidence: ac1_3 greps for the strings `fixtures/contracts`, `process.argv`, or `check` in scripts/check-contract-fixture-key-order.ts and runs it with --allow-missing. A stub script that prints those keywords would pass.
  - Recommendation: Tighten ac1_3 to require both (a) an exit-non-zero on a deliberately-misordered seed fixture and (b) an exit-zero on a known-good seed fixture, so behavior is verified rather than substring presence.

Recommended edits:
- Phase 2 / packages/core/src/util/hash.ts change
  - Grounded in: code:packages/core/src/util/hash.ts:12
  - Recommendation: Add a Phase 2 sub-task `hashStable parity audit` that enumerates current hashStable call sites and pins a TypeScript-only fixture covering every payload shape used in receipts, capability execution, idempotency, knowledge entries, and a2a taskIds. The fixture must demonstrate that the new code-point comparator produces the same hash as today's localeCompare comparator for those payloads, or call out exactly which payload shapes will change.
- Phase 2 Rollback
  - Grounded in: spec_gap:rollback.phase2
  - Recommendation: Append: `also restore the localeCompare ordering branch in packages/core/src/util/hash.ts so the TypeScript hash output reverts in lockstep with the Rust removals.`
- Phase 3 Changes
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:1
  - Recommendation: Either add explicit Rust homes for the transitive wire types (ExecutionEvent, ResolutionRequest, ResolutionResponse subset consumed by HostPausedResult/HostCompletedResult/etc.) or state they are modelled as `JsonValue` with a deferred-parity marker in host_protocol.rs.
- Phase 5 ac5_2
  - Grounded in: spec_gap:phases.phase5.acceptance
  - Recommendation: Remove the `--glob 'rust-sdk-surface-parity.md'` portion of ac5_2 unless rust-sdk-surface-parity.md is added to Phase 5 Changes. Keeping ac5_2 limited to docs/rust-kernel-architecture.md and crates/runx-contracts/README.md aligns acceptance with declared scope.
- Phase 4 dependency
  - Grounded in: spec_gap:dependencies
  - Recommendation: Either gate this spec's approval on rust-cli-feature-parity-matrix reaching `approved`, or explicitly allow Phase 4 to remain pending while validation v8 (`cargo package`) runs after Phases 1-3 + 5. Without one of these, dod4 and v8 cannot both be honored.
- Phase 1 ac1_3
  - Grounded in: spec_gap:phases.phase1.acceptance
  - Recommendation: Replace the substring grep with a behavioral check: a tiny inline fixture pair where one file is intentionally key-disordered must cause exit 1, and a sorted twin must exit 0.

### round-4

Status: failed
Started: 2026-05-18T03:57:38Z
Ended: 2026-05-18T03:57:38Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: The spec is unusually well-scoped and the contract-crate skeleton is grounded in a real placeholder. Three load-bearing issues block approval: (1) the Phase 2 `stableStringify` cutover changes the input to `signPayloadString` for receipts and a half-dozen direct `stableStringify` callers, but the audit fixture set is keyed to `hashStable` payload classes only — receipt-signature payloads, state-machine internal stable strings, and CLI authoring stringify equality are not covered, and historical receipts can fail verification after the cutover; (2) two vendored copies of `stableStringify` (in `packages/core/src/state-machine/index.ts:655` and `packages/cli/tools/thread/push_outbox/src/index.ts:413`) keep `localeCompare`, so Phase 2 silently introduces hash divergence between the very modules whose interop the spec is trying to prove; (3) Phase 4 hard-depends on `rust-cli-feature-parity-matrix` being `completed`, but that spec is still `status: draft` in `.scafld/specs/drafts/`, leaving `dod4` and Phases 4–5 unbounded. Plus the ac2_5 ordering rule ("edit is allowed only after audit passes") is enforced only by prose, not by automation.

Checks:
- path audit
  - Grounded in: code:crates/runx-contracts/src/lib.rs:1 and code:crates/runx-contracts/Cargo.toml:13
  - Result: passed
  - Evidence: Existing placeholder paths (`crates/runx-contracts/Cargo.toml`, `src/lib.rs`) are correctly marked `partial, exclusive`. The Phase 1 plan to migrate `JsonValue`/`JsonNumber`/`JsonObject` from `lib.rs` to `src/json.rs` is consistent with the shipped surface I read at `lib.rs:9-27`. New paths (`capability_execution.rs`, `host_protocol.rs`, `cli.rs`, `receipts.rs`, `registry.rs`, `tools.rs`) are all under the crate root and conform to the rustRoots list at `scripts/check-rust-core-style.mjs:8`. `packages/core/src/util/hash.ts` is correctly marked `partial, shared` since it is touched by other code paths. The Cargo `include` allowlist at `crates/runx-contracts/Cargo.toml:13` already excludes fixtures, matching the Phase 5 documentation requirement.
- command audit
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md (acceptance, ac4_0)
  - Result: failed
  - Evidence: ac4_0 runs `execFileSync('scafld', [...])` and depends on `scafld` resolving on PATH. The repo CLAUDE.md says `Inside the scafld repo, use ./bin/scafld or go run ./cmd/scafld; do not use a copied compiled binary` — runx CI does not guarantee `scafld` on PATH. The other commands are well-formed: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts` matches the workspace at `crates/Cargo.toml:2`; `node scripts/check-rust-core-style.mjs` matches the .mjs file at `scripts/check-rust-core-style.mjs`. `! rg -n 'tokio|...|std::env|Command::new' ...` for v7 will correctly bash-negate ripgrep exit codes. The grep in ac2_3 `! rg ... crates/runx-sdk/src crates/runx-sdk/Cargo.toml` assumes those paths exist; the workspace at `crates/Cargo.toml:9` lists `runx-sdk`, so this is fine.
- scope/migration audit
  - Grounded in: code:packages/core/src/state-machine/index.ts:655 and code:packages/cli/tools/thread/push_outbox/src/index.ts:413
  - Result: failed
  - Evidence: Phase 2 edits only `packages/core/src/util/hash.ts:12` to swap `localeCompare` for a code-point comparator. But there are two further vendored stable-stringify implementations that the spec does not touch: `packages/core/src/state-machine/index.ts:655` (`function stableValue` uses `.sort(([left], [right]) => left.localeCompare(right))`) and `packages/cli/tools/thread/push_outbox/src/index.ts:413` (`function stableStringify` likewise). After the cutover, the state-machine domain (which the kernel-architecture doc calls a `Pure crate` reference) and the push_outbox tool (which produces `entry_id` hashes that `packages/core/src/knowledge/file-thread.ts:97` consumes) silently diverge from `hashStable`. The spec invariant that Rust and TS hashes match `exactly` cannot hold while these copies use the old comparator. The spec must either fold them into the cutover, declare them out of scope with a follow-up task, or convert them to delegate to `stableStringify`.
- acceptance timing audit
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md (phases ac2, Phase 4 dependency)
  - Result: failed
  - Evidence: Two timing problems: (a) Phase 2 prose says `This edit is allowed only after the hashStable observed-shape audit passes` and `Also generate the observed-shape hash audit fixtures before changing packages/core/src/util/hash.ts`, but the only machine check is `ac2_5` which greps for token presence in fixtures. Nothing prevents an implementer from running the generator *after* the hash.ts cutover, regenerating fixtures with only the new comparator, and still passing the rg. The audit must capture a frozen pre-cutover value so post-cutover regeneration is checkable. (b) Phase 4 depends on `rust-cli-feature-parity-matrix completed`. `.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:6` shows `status: draft`; the dependency is therefore against a spec that has not even been approved yet. The acceptance `ac4_0` will fail until that spec is completed, leaving `dod4` indefinitely incomplete.
- rollback/repair audit
  - Grounded in: code:packages/core/src/receipts/index.ts:496 and spec rollback Phase 2
  - Result: failed
  - Evidence: Phase 2 rollback says `restore localeCompare ordering in packages/core/src/util/hash.ts`. But `packages/core/src/receipts/index.ts:496-547` calls `signPayloadString(stableStringify(signedPayload), ...)` and the matching `verifyPayloadString(stableStringify(signedPayload), signature, ...)` on the same function. If any receipt is signed *during* the cutover window (between hash.ts change and rollback), reverting hash.ts will break verification of that receipt forever — the signature was computed over the code-point ordering but verification will recompute under `localeCompare`. The rollback strategy needs either a versioned `stable_stringify_variant` field stamped into receipts, a documented receipt-quarantine window during the cutover, or an explicit statement that no new receipts may be issued between the audit and Phase 3.
- design challenge
  - Grounded in: code:packages/core/src/receipts/index.ts:472 and code:packages/core/src/receipts/index.ts:496
  - Result: failed
  - Evidence: The audit fixture set in `ac2_5` enumerates `receipt_input|idempotency_key|capability_intent|capability_trigger|capability_content|knowledge_entry|a2a_task|a2a_message|a2a_output`. These are all `hashStable(...)` callers. But `stableStringify` is also called directly to produce signature inputs at `packages/core/src/receipts/index.ts:496`, `index.ts:537`, `index.ts:547`, `packages/core/src/receipts/outcome-resolution.ts:169`, `outcome-resolution.ts:182`, and for equality at `packages/cli/src/authoring-utils.ts:196`. None of these payload classes are named in the audit. The implicit claim that they are unaffected because their keys are all ASCII is plausible for current schemas, but it is not proven by the spec and not enforced by ac2_5. Receipts are append-only and signed; this is the wrong place to have an unproven invariant.

Questions:
- Should Phase 2 also cover the two vendored stableStringify copies at `packages/core/src/state-machine/index.ts:655` and `packages/cli/tools/thread/push_outbox/src/index.ts:413`, or are they intentionally out of scope with a documented follow-up?
  - Grounded in: code:packages/core/src/state-machine/index.ts:655 and code:packages/cli/tools/thread/push_outbox/src/index.ts:413
  - Recommended answer: Fold them into Phase 2. Have `state-machine/index.ts` delegate to `stableStringify` from `packages/core/src/util/hash.ts` (or share the comparator helper) and replace the push_outbox vendored block with a call into core. If push_outbox cannot import from core because it ships as a standalone tool, add a `packages/cli/tools/thread/push_outbox/src/index.ts` change to use the same code-point comparator and pin it with a fixture that proves entry_id parity with `packages/core/src/knowledge/file-thread.ts:97`.
  - If unanswered: Default to expanding Phase 2 scope to cover both vendored copies and add a v9 validation `! rg -n "localeCompare" packages/core/src/state-machine/index.ts packages/cli/tools/thread/push_outbox/src/index.ts packages/core/src/util/hash.ts`.
- How are already-signed receipts (whose signatures are computed over `stableStringify(signedPayload)` in `packages/core/src/receipts/index.ts:496`) protected across the Phase 2 cutover, and what happens if Phase 2 is rolled back after receipts have been issued under the new comparator?
  - Grounded in: code:packages/core/src/receipts/index.ts:496
  - Recommended answer: Add a Phase 2 fixture explicitly covering signed receipt payload classes (skill_execution, graph_execution, outcome_resolution) and prove byte-identical `stableStringify` output under both comparators for the canonical receipt key sets, including any non-ASCII payloads that can reach `signedPayload` (e.g., user-supplied input keys passed through `redactReceiptMetadata`). If that proof fails, gate the cutover behind a versioned `stable_stringify_variant` field on the receipt envelope rather than mutating the comparator in place.
  - If unanswered: Default to adding a `fixtures/contracts/hash-stable-observed/signed-receipts/*.json` set covering each receipt schema and extend the `ac2_5` rg to require those filenames.
- Phase 4 depends on `rust-cli-feature-parity-matrix` being `completed`, but `.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:6` shows `status: draft`. Is the intent that this spec ships Phases 1–3 first and Phase 4 is gated separately, or do you want approval to block on the CLI matrix?
  - Grounded in: code:.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:6
  - Recommended answer: Split the spec so Phases 1–3 (plus Phase 5 stubs that do not require CLI fixtures) can complete independently, and move Phase 4 into a follow-up `rust-contracts-cli-json` spec whose own dependency on `rust-cli-feature-parity-matrix` is enforced at its approval gate. Otherwise this spec cannot satisfy `dod4` until two cross-team specs are both done.
  - If unanswered: Default to splitting Phase 4 into a follow-up spec and updating `dod4` so this spec only requires the host-protocol and capability-execution surfaces that block `rust-sdk-surface-parity` Phase 2.
- ac2_5 prose says `edit is allowed only after the hashStable observed-shape audit passes`, but the only machine check is an rg for token presence in fixtures. How should the audit pin a *frozen* pre-cutover value so a later regeneration cannot silently overwrite the proof?
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md acceptance ac2_5
  - Recommended answer: Each observed-shape fixture should record three fields: `payload`, `legacyLocaleCompareHash`, and `codePointHash`. The fixture is generated *before* the hash.ts edit; thereafter `generate-rust-contract-fixtures.ts --check` recomputes both hashes (the legacy one through a frozen vendored comparator helper that is never edited again) and asserts both match the recorded values. Add a v9 acceptance that the legacy comparator helper has not changed since Phase 2 landed (a git-blame check or a hash of the helper file).
  - If unanswered: Default to writing both hashes into each audit fixture and shipping a `scripts/_legacy-locale-compare.ts` helper module that is forbidden by `check-rust-core-style.mjs`-equivalent guard from being edited.
- Phase 3 lists the ResolutionRequest variants `input`, `approval`, `cognitive_work` (matching `packages/contracts/src/schemas/resolution.ts:55-86`), but does not pin the closed `ExecutionEvent.type` enum from `packages/runtime-local/src/runner-local/index.ts:139` (11 variants). Should Rust model that field as an enum or as `String`?
  - Grounded in: code:packages/runtime-local/src/runner-local/index.ts:139
  - Recommended answer: Model it as a closed Rust enum with `#[serde(rename_all = "snake_case")]` and 11 variants. Generate a fixture per variant so the rename mapping is locked. If the TS surface adds a 12th variant later, the fixture test fails loudly. This matches the spec's `idiomatic, not mechanically translated` direction and matches the closed enum already in TypeScript.
  - If unanswered: Default to a closed Rust enum with `#[serde(rename_all = "snake_case")]` and one fixture per variant.
- ac4_0 invokes `scafld` via `execFileSync('scafld', ...)`. Does runx CI provide `scafld` on PATH, or should the acceptance use `./bin/scafld` / a documented absolute path?
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md acceptance ac4_0; project:CLAUDE.md `Inside the scafld repo, use ./bin/scafld or go run ./cmd/scafld`
  - Recommended answer: Read the scafld binary from `process.env.SCAFLD_BIN ?? 'scafld'`, document that runx CI must set `SCAFLD_BIN`, and fall back to a `--skip-cli-matrix-gate` env var for the rare case where the gate must be bypassed (e.g., dependency-graph dry runs).
  - If unanswered: Default to `process.env.SCAFLD_BIN ?? 'scafld'` and add a CI note that the env var must be set.

Design objections:
- `objection-1` critical - `stableStringify` cutover silently changes signed-receipt verification input.
  - Grounded in: code:packages/core/src/receipts/index.ts:496 and audit list in ac2_5
  - Evidence: `buildLocalSkillReceipt` and `buildLocalGraphReceipt` sign the payload with `signPayloadString(stableStringify(signedPayload), keyPair.privateKey)` (`packages/core/src/receipts/index.ts:496` and `:537`). `verifyLocalReceipt` recomputes `stableStringify(signedPayload)` on read (`:547`). Changing the comparator changes that input string. The ac2_5 audit only enumerates `hashStable`-call payload classes (`receipt_input`, `idempotency_key`, etc.); the signed-receipt envelope itself is not in the list. Even though current receipt schemas appear to be ASCII-only and likely sort identically under both comparators, this is unproven. Any user-controlled key that lands in `metadata`, `outcome`, or `input_context` could mismatch.
  - Recommendation: Add explicit `hash-stable-observed/signed-receipts/*.json` audit fixtures for skill/graph/outcome-resolution receipts using the canonical key sets *and* representative user-controlled payloads, with both `legacyLocaleCompareHash` and `codePointHash` recorded. Block the hash.ts edit on those fixtures showing identical hashes. If they diverge, gate the cutover behind a versioned `stable_stringify_variant` field on the receipt envelope rather than swapping comparators in place.
- `objection-2` critical - Two vendored stableStringify copies keep localeCompare; Phase 2 introduces hidden divergence.
  - Grounded in: code:packages/core/src/state-machine/index.ts:655 and code:packages/cli/tools/thread/push_outbox/src/index.ts:413
  - Evidence: `packages/core/src/state-machine/index.ts:655` defines a local `stableValue` that sorts entries via `localeCompare`. `packages/cli/tools/thread/push_outbox/src/index.ts:413` defines a vendored `stableStringify` likewise. Phase 2 only edits `packages/core/src/util/hash.ts:12`. After cutover, the state-machine domain (the spec's pure decision domain) and push_outbox (whose `entry_id` is consumed by `packages/core/src/knowledge/file-thread.ts:97`) will sort keys under `localeCompare`, while everything that flows through `hashStable` will sort under code-point. The spec's invariant that Rust and TypeScript hashes match `exactly` cannot hold while these copies disagree. Worse, no acceptance forbids the comparator from being re-introduced (no v9 rg).
  - Recommendation: Either (a) make state-machine and push_outbox delegate to `stableStringify` from `packages/core/src/util/hash.ts` (push_outbox may need a shared helper file vendored into the tool), or (b) update both vendored copies inside Phase 2 with their own fixture proofs. Add a v9 validation `! rg -n "localeCompare" packages/core/src/util/hash.ts packages/core/src/state-machine/index.ts packages/cli/tools/thread/push_outbox/src/index.ts` to lock the absence.
- `objection-3` high - Phase 4 depends on a spec that is still in drafts/.
  - Grounded in: code:.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:6
  - Evidence: Phase 4 dependencies state `rust-cli-feature-parity-matrix completed, and fixtures/cli-parity/ containing the consumed CLI JSON cases`. `.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:6` shows `status: draft`. `fixtures/cli-parity/` does not exist at HEAD (no Glob match). The ac4_0 acceptance therefore cannot pass at any point in the current repo state. This leaves `dod4` and Phase 5's `cargo package` check (which transitively depends on Phase 4 having landed code) unbounded.
  - Recommendation: Either (a) move Phase 4 into a follow-up `rust-contracts-cli-json` spec, dropping it from this spec's DoD; or (b) make this spec's approval conditional on the CLI matrix being approved first, and downgrade Phase 4 acceptance from `completed` to `approved` so phases 1–3 can land independently. Option (a) better matches the `runx-sdk Phase 2 depends on this` driver — SDK v0 only needs host-protocol and capability-execution shapes; CLI JSON envelopes can come second.
- `objection-4` high - ac2_5 does not enforce the prose timing rule.
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md Phase 2 prose vs. ac2_5
  - Evidence: Phase 2 prose says `This edit is allowed only after the hashStable observed-shape audit passes` and `Also generate the observed-shape hash audit fixtures before changing packages/core/src/util/hash.ts`. But ac2_5 is `pnpm exec tsx scripts/generate-rust-contract-fixtures.ts --check --scope hash-stable-observed && rg -n '<token list>' fixtures/contracts/hash-stable-observed scripts/generate-rust-contract-fixtures.ts`. The `--check` mode presumably just verifies that the generator produces identical bytes to the on-disk fixtures. Nothing prevents an implementer from running the generator *after* the hash.ts cutover, regenerating fixtures with only the new comparator, and still passing ac2_5. The proof of `unchanged hashes for observed shapes` is therefore non-machine-checked.
  - Recommendation: Capture both the legacy `localeCompare` hash and the new code-point hash in each audit fixture (as separate JSON fields). Ship a frozen `scripts/_legacy-locale-compare.ts` helper that the generator uses to compute the legacy hash — and forbid further edits to it via a check (file content hash pinned in CI). The audit then proves both values are stable across generator runs, which is what the prose intends.
- `objection-5` medium - ac4_0 assumes `scafld` is on PATH.
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md acceptance ac4_0; project:CLAUDE.md
  - Evidence: The acceptance shells out via `execFileSync('scafld', ['status', ...])`. The repo CLAUDE.md says `Inside the scafld repo, use ./bin/scafld or go run ./cmd/scafld`. runx CI may not provision `scafld` on PATH the same way as scafld's own CI. If `scafld` is missing, the command errors with ENOENT before reading the status JSON, which would block the phase for an environmental reason unrelated to the dependency state.
  - Recommendation: Resolve the binary via `process.env.SCAFLD_BIN ?? 'scafld'`, document the env var in this spec's `## Context`, and add a CI step that sets `SCAFLD_BIN` to the installed path.
- `objection-6` medium - ExecutionEvent.type closed enum is not pinned in the spec.
  - Grounded in: code:packages/runtime-local/src/runner-local/index.ts:139
  - Evidence: The TypeScript surface at `packages/runtime-local/src/runner-local/index.ts:139` is an 11-variant string union (`skill_loaded`, `inputs_resolved`, `auth_resolved`, `resolution_requested`, `resolution_resolved`, `admitted`, `executing`, `step_started`, `step_waiting_resolution`, `step_completed`, `warning`, `completed`). Phase 3 says `ExecutionEvent { type, message, data }` without saying whether Rust should model `type` as a closed enum or `String`. A `String` field would silently accept future drift; an enum locks it.
  - Recommendation: Make Phase 3 explicit: model `ExecutionEvent.kind` (or `type`) as a closed Rust enum with `#[serde(rename_all = "snake_case")]`, ship one fixture per variant, and document that adding a new variant in TypeScript is a contract change that must update Rust first.
- `objection-7` medium - Phase 2 rollback does not address receipts signed under the new comparator.
  - Grounded in: code:packages/core/src/receipts/index.ts:472 and spec rollback Phase 2
  - Evidence: Phase 2 rollback says `remove ... and restore localeCompare ordering in packages/core/src/util/hash.ts`. But `packages/core/src/receipts/index.ts:472` and `:524` compute `input_hash = hashStable(options.inputs)`, and `:496` signs the payload via `stableStringify`. If any new receipt is issued between the cutover and the rollback, its `input_hash` and signature are bound to the code-point comparator. Reverting hash.ts will break `verifyLocalReceipt` on those receipts.
  - Recommendation: Add to the Phase 2 rollback: `Before reverting hash.ts, identify any receipts issued under the new comparator (timestamp window) and either re-sign them with the restored comparator or version-stamp the receipt envelope.` Or: do not allow rollback once Phase 3 lands; document that the cutover is one-way after the audit passes.

Recommended edits:
- Phase 2 Changes
  - Grounded in: code:packages/core/src/state-machine/index.ts:655 and code:packages/cli/tools/thread/push_outbox/src/index.ts:413
  - Recommendation: Add two `partial, exclusive` Changes entries: `packages/core/src/state-machine/index.ts` (replace local `stableValue` localeCompare with a code-point comparator, or delegate to `stableStringify`) and `packages/cli/tools/thread/push_outbox/src/index.ts` (replace vendored `stableStringify` likewise). Add a new validation `v9` that greps for `localeCompare` across all three files and fails if found.
- Phase 2 Acceptance
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md Phase 2 prose
  - Recommendation: Replace ac2_5 with two acceptances: (a) `--check` mode rebuilds each fixture and asserts byte-identical output (this part already exists); (b) every observed-shape fixture must contain both a `legacyLocaleCompareHash` and a `codePointHash` field whose recorded values match what a frozen `scripts/_legacy-locale-compare.ts` helper and the new comparator compute. Forbid edits to the frozen helper via a file-content hash pin in CI (or `check-rust-core-style.mjs`-equivalent).
- Phase 2 Changes (audit scope)
  - Grounded in: code:packages/core/src/receipts/index.ts:496 and :547
  - Recommendation: Extend the `hash-stable-observed` payload class list to include `receipt_signature_payload` (skill_execution and graph_execution envelopes) and `outcome_resolution_signature_payload`. Cross-link the new fixtures from `ac2_5`'s rg token list.
- Phase 4 Dependencies / Spec scope
  - Grounded in: code:.scafld/specs/drafts/rust-cli-feature-parity-matrix.md:6
  - Recommendation: Either delete Phase 4 from this spec (moving it into a follow-up `rust-contracts-cli-json` whose approval gate enforces the CLI matrix dependency cleanly), or downgrade Phase 4's dependency from `rust-cli-feature-parity-matrix completed` to `approved` and acknowledge in the rollback section that Phase 4 may need to land in a follow-up if the CLI matrix slips. Update `dod4` accordingly.
- Phase 4 Acceptance ac4_0
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md acceptance ac4_0; project:CLAUDE.md
  - Recommendation: Change `execFileSync('scafld', ...)` to `execFileSync(process.env.SCAFLD_BIN ?? 'scafld', ...)` and document the env var in `## Context`. Add a CI provisioning note in this spec or its dependency notes.
- Phase 3 Changes (ExecutionEvent shape)
  - Grounded in: code:packages/runtime-local/src/runner-local/index.ts:139
  - Recommendation: Explicitly state that Rust models `ExecutionEvent.type` as a closed enum mirroring the 11 TypeScript variants with `#[serde(rename_all = "snake_case")]`, and require one fixture per variant under `fixtures/contracts/host-protocol/execution-events/`. Add the variant list to the spec to lock against silent drift.
- Phase 2 Rollback
  - Grounded in: code:packages/core/src/receipts/index.ts:472
  - Recommendation: Add: `If receipts have been issued under the code-point comparator before rollback, the rollback is unsafe — document the cutover window and require operator confirmation that no production receipts span the window before reverting hash.ts. Alternatively, mark Phase 2 as one-way once the audit passes and remove the localeCompare-restore step.`

### round-5

Status: failed
Started: 2026-05-18T04:10:08Z
Ended: 2026-05-18T04:10:08Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Spec has internalized all rounds 1-4 findings and is architecturally sound: Phase 1 explicitly migrates the bootstrap JSON surface, fixture-key checks use `pnpm exec tsx`, Phase 4 is narrowed to a deferred CLI module home with the actual CLI JSON parity moved to `rust-contracts-cli-json-parity`, the global `stableStringify` cutover is moved to `hash-stable-codepoint-cutover`, host-protocol scope explicitly excludes closure types and pins the 11-variant `ExecutionEvent` enum, and Phase 5 acceptance no longer fans out to the workspace. The remaining defects are acceptance sharpness rather than design: (1) `ac2_4`, `ac3_2`, `ac3_3`, and `ac4_1` use OR-alternation regex against `rg`, which exits 0 on any single match, so a stub `host_protocol.rs` that mentions only `completed` would pass `ac3_3` even though the gate's stated intent is to cover all 5 host outcomes / 11 ExecutionEvent variants / closed-enum ResolutionRequest variants; (2) Phase 3 prose lets "deep nested request payloads" be modelled as `JsonValue` but does not pin the threshold, so the immediate `ResolutionRequest` payloads (`questions`, `gate`, `work`) could legitimately be opaque, which would defeat SDK v0's typed-payload goal; (3) `ac2_3`'s hashing-exclusivity check is SDK-only and does not cover `runx-runtime`/`runx-core`/`runx-parser`/`runx-receipts`, while the spec invariant says hashing for contract semantics lives in `runx-contracts`; (4) `ac1_2` couples Phase 1 acceptance to the style state of every other Rust crate in the workspace because `check-rust-core-style.mjs` walks all seven crate roots. None block approval architecturally, but they are easy sharpenings that turn presence smoke tests into coverage proofs.

Checks:
- path audit
  - Grounded in: code:crates/runx-contracts/src/lib.rs:1 and code:crates/runx-contracts/Cargo.toml:13
  - Result: passed
  - Evidence: All Phase 1-5 paths resolve. crates/runx-contracts/src/lib.rs:1-183 contains the bootstrap JsonValue/JsonNumber/JsonObject + serde impls + #[cfg(test)] tests that Phase 1 explicitly migrates to src/json.rs (verified line 357-360 of spec). crates/runx-contracts/Cargo.toml:13 sets include = ["Cargo.toml", "README.md", "src/**/*.rs"], matching Phase 5's documented package allowlist. crates/Cargo.toml workspace at line 4 includes runx-contracts, so `cargo --manifest-path crates/Cargo.toml -p runx-contracts` resolves. crates/runx-sdk/Cargo.toml:19 already has `runx-contracts.workspace = true` with no sha2, so ac2_3 negative grep is satisfiable today.
- command audit
  - Grounded in: spec_gap:phases.phase3.ac3_3 and code:packages/runtime-local/src/runner-local/index.ts:138
  - Result: failed
  - Evidence: ac3_3 = `rg -n 'ExecutionEvent|ResolutionRequest|ResolutionResponse|input|approval|cognitive_work|skill_loaded|inputs_resolved|auth_resolved|resolution_requested|resolution_resolved|admitted|executing|step_started|step_waiting_resolution|step_completed|warning|completed' ...`. rg exits 0 if any single alternative matches. The 11 ExecutionEvent variants (verified at packages/runtime-local/src/runner-local/index.ts:138-154) plus 3 ResolutionRequest kinds are the coverage target, but the gate would pass with a one-line stub that mentions `completed`. Same defect in ac3_2 (5 outcomes), ac2_4 (4 hash-scope markers), and ac4_1 (3 deferred markers). The previous round-2 finding about `runx-contracts.*runx-sdk` regex brittleness was fixed via the `contracts-first-ordering:` marker; the same fix-shape (require literal token presence per variant or use a count assertion) should be applied here.
- scope/migration audit
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:55
  - Result: failed
  - Evidence: Phase 3 prose says `tagged ResolutionRequest variants for input, approval, and cognitive_work` and `Deep nested request payloads that are not needed by SDK v0 may be represented as JsonValue`. packages/contracts/src/schemas/resolution.ts:55-86 shows each variant's payload field: input -> questions:Question[], approval -> gate:ApprovalGate, cognitive_work -> work:AgentWorkRequest. Those reference deeply nested types from agent-work.ts (Question, ApprovalGate, AgentWorkRequest). The spec does not pin where typed Rust shapes stop and JsonValue begins. An implementer could legitimately ship `enum ResolutionRequest { Input { id: String, questions: JsonValue }, Approval { id: String, gate: JsonValue }, CognitiveWork { id: String, work: JsonValue } }` and pass every acceptance gate, even though the SDK v0 consumer almost certainly needs typed access to questions/gate/work. The threshold needs to be named: either (a) typed Question/ApprovalGate/AgentWorkRequest are in scope for this spec, or (b) opaque-JsonValue is acceptable and a follow-up owner is named for each.
- acceptance timing audit
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md phase1.ac1_2 and code:scripts/check-rust-core-style.mjs:6
  - Result: failed
  - Evidence: ac1_2 runs `node scripts/check-rust-core-style.mjs`. scripts/check-rust-core-style.mjs:6-14 walks all seven crate roots (crates/runx-cli/src, runx-contracts/src, runx-core/src, runx-parser/src, runx-receipts/src, runx-runtime/src, runx-sdk/src). It enforces line-count limits, function-length limits, no .unwrap()/.expect(), no panic!/todo!/unimplemented!/dbg!, no HashMap, no serde_json::Value, etc. across ALL of those crates. Phase 1 of this spec only touches runx-contracts/src/{lib.rs,json.rs}, but ac1_2 will fail Phase 1 acceptance for unrelated style breakage anywhere in the workspace (e.g., if a parallel rust-state-machine-parity branch lands a violation in runx-core/src). Round-1 narrowed ac5_3 to `-p runx-contracts`, but ac1_2 still inherits the workspace-wide fan-out via the style guard. Either scope the style guard via a crate-name flag in this spec, or accept this as a standing-guardrail coupling and document it.
- rollback/repair audit
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md rollback Phase 1 and code:crates/runx-contracts/src/lib.rs:1
  - Result: passed
  - Evidence: Phase 1 rollback (line 569-572) reads `revert runx-contracts to the bootstrap-era JSON surface (JsonValue, JsonNumber, JsonObject, serde impls, and tests) and remove only the new module declarations/dependencies added by this spec. Do not delete the rust-contracts-bootstrap deliverable.` This matches the actual shipped state at crates/runx-contracts/src/lib.rs:1-183 (184 lines of JsonObject + JsonValue + JsonNumber + Serialize/Deserialize + #[cfg(test)] mod tests). Phases 2-5 rollbacks remove additive module files plus their fixtures and tests; that is mechanically clean because each phase owns its own files exclusively. The previous round-3/4 receipts-signature concern is no longer applicable because the locale-compare cutover was relocated to `hash-stable-codepoint-cutover` and this spec only edits typed Rust shapes.
- design challenge
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md ac2_3 and code:scripts/check-rust-core-style.mjs
  - Result: failed
  - Evidence: ac2_3 = `rg -n 'sha2|Sha256|Digest' crates/runx-contracts/src/capability_execution.rs && ! rg -n 'sha2|Sha256|Digest' crates/runx-sdk/src crates/runx-sdk/Cargo.toml`. The negative half excludes sha2 only from runx-sdk. But the spec invariant (line 89-93) says contract hashing semantics live in runx-contracts. The other six workspace crates (runx-cli, runx-core, runx-parser, runx-receipts, runx-runtime) are not blocked from adding sha2. runx-runtime in particular is a strong candidate for picking up sha2 once it adds receipt verification, which would silently duplicate hashing logic that the spec wants centralized in runx-contracts. Either widen the negative grep to all consumer crates (`crates/runx-{cli,core,parser,receipts,runtime,sdk}/{Cargo.toml,src}`) and document permitted exceptions, or rewrite the invariant to clarify that other crates may carry sha2 for non-contract uses (e.g., receipt signature verification) and explain why that doesn't violate contract hash centralization.

Questions:
- How should the immediate `ResolutionRequest` variant payloads (`questions: Question[]`, `gate: ApprovalGate`, `work: AgentWorkRequest`) be modelled in Rust — typed Rust shapes ported from packages/contracts/src/schemas/{resolution,agent-work}.ts, or opaque JsonValue with a deferred-parity marker?
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:55
  - Recommended answer: Type the first level: the `Input`/`Approval`/`CognitiveWork` Rust variants should carry typed fields (`questions: Vec<Question>`, `gate: ApprovalGate`, `work: AgentWorkRequest`) so SDK v0 consumers can pattern-match without a JSON walk. Anything *inside* Question/ApprovalGate/AgentWorkRequest that is deeper than one level of nesting may stay as JsonValue with a deferred-parity marker naming a follow-up spec (likely `rust-resolution-payload-parity`). Pin this threshold in Phase 3 Changes and update ac3_3 to check for the typed Rust enum/struct names (`Question`, `ApprovalGate`, `AgentWorkRequest`) rather than just the variant kind strings.
  - If unanswered: Default to typing the immediate variant payloads (Question, ApprovalGate, AgentWorkRequest are in scope) and modelling anything two levels deeper as JsonValue.
- Should `ac3_3`, `ac3_2`, `ac2_4`, and `ac4_1` be rewritten so each required token has its own grep, instead of OR-alternation that passes on the first match?
  - Grounded in: spec_gap:phases.phase3.ac3_3
  - Recommended answer: Yes. Replace the `|`-alternation with a small TS or shell script (or a `for token in ...; do rg -q ... || exit 1; done` loop) that checks each required token individually. For ac3_3 specifically, list the 11 ExecutionEvent variants and 3 ResolutionRequest kinds, each as a separate `rg -q` gated by `|| exit 1`. The round-2 fix that introduced the canonical `contracts-first-ordering:` marker is a model: a single literal token whose presence is meaningful, not an alternation. Apply the same shape here.
  - If unanswered: Default to splitting each acceptance into one `rg -q` per required token chained with `&&`, so any missing variant fails the gate.
- Should `ac2_3`'s hashing-exclusivity check be widened to all consumer crates, or should the spec invariant be relaxed to clarify that non-contract hashing (e.g., receipt verification in runx-receipts/runx-runtime) is permitted?
  - Grounded in: spec.invariants and code:crates/runx-sdk/Cargo.toml:19
  - Recommended answer: Relax the invariant. Receipt verification, idempotency replay, and adapter content addressing legitimately need SHA-256 outside runx-contracts. Rewrite the invariant to say `runx-contracts owns capability-execution and idempotency hash semantics; downstream crates that need SHA-256 for non-contract uses (receipt signature verification, adapter content hashing) may take a direct dependency on sha2 with documented rationale.` Keep ac2_3 as the SDK-specific check it already is, but rename it `Rust SDK v0 does not duplicate contract hashing` so the scope matches the wording.
  - If unanswered: Default to keeping ac2_3 narrow (SDK only) and renaming it to make the SDK-specific scope explicit, while adding an Invariants note that other crates may carry sha2 for non-contract purposes.
- Should `ac1_2`'s coupling of Phase 1 acceptance to the entire workspace's style state be accepted as a standing guardrail, or should the style guard learn a crate-name flag so Phase 1 can verify only `crates/runx-contracts/src`?
  - Grounded in: code:scripts/check-rust-core-style.mjs:6
  - Recommended answer: Accept the coupling and document it. The style guard is a standing project guardrail (it must run on the full workspace to catch drift in any crate), and forcing this spec to teach it a crate-name flag is scope creep. Add a one-line Invariants note: `Phase 1's ac1_2 inherits workspace-wide style state; unrelated crate violations can block Phase 1 acceptance until fixed in their owning spec.` That matches the operator expectation when running scafld build inside a multi-spec branch.
  - If unanswered: Default to accepting the coupling and adding the one-line note to Invariants.
- Are the follow-up spec ids `hash-stable-codepoint-cutover`, `rust-contracts-cli-json-parity`, `rust-receipts-parity`, `rust-registry-parity`, and `rust-tools-parity` placeholder ids that this spec is reserving, or do they need to exist as drafts before this spec can approve?
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md follow-up-specs
  - Recommended answer: Reserve them as future task ids; they do not need to exist as drafts before this spec approves. The cli-json-parity follow-up has a hard dependency on `rust-cli-feature-parity-matrix`, which is still a draft, so creating the cli-json-parity draft before the matrix is hardened would be premature. Add a single line to the Follow-up Specs section noting that these are reserved ids that will be drafted when their preconditions land, so future readers don't expect to find drafts under those names today.
  - If unanswered: Default to treating the follow-up ids as reserved-but-not-yet-drafted, and add a clarifying sentence to the Follow-up Specs section.

Design objections:
- `objection-5-1` high - ac3_2, ac3_3, ac2_4, and ac4_1 use OR-alternation grep that passes on a single match, so stub implementations pass coverage gates.
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md ac3_3 and code:packages/runtime-local/src/runner-local/index.ts:138
  - Evidence: ac3_3 alternates 14+ tokens (`ExecutionEvent|ResolutionRequest|ResolutionResponse|input|approval|cognitive_work|skill_loaded|inputs_resolved|auth_resolved|resolution_requested|resolution_resolved|admitted|executing|step_started|step_waiting_resolution|step_completed|warning|completed`). rg exits 0 if any single alternative matches. The stated intent of ac3_3 is to verify the closed 11-variant ExecutionEvent enum is fully pinned (verified at packages/runtime-local/src/runner-local/index.ts:138-154) plus the 3 ResolutionRequest kinds. A `host_protocol.rs` containing only `pub fn completed() {}` would pass. The same defect applies to ac3_2 (5 host outcomes), ac2_4 (4 hash-scope markers including `localeCompare`), and ac4_1 (3 CLI-deferred markers including the explicit spec name).
  - Recommendation: Rewrite each acceptance as a chain of `rg -q` per required token, joined with `&&`, so missing any single variant fails the gate. Example for ac3_3: `for tok in skill_loaded inputs_resolved auth_resolved resolution_requested resolution_resolved admitted executing step_started step_waiting_resolution step_completed warning completed Input Approval CognitiveWork; do rg -q "$tok" crates/runx-contracts/src/host_protocol.rs fixtures/contracts/host-protocol || { echo missing $tok; exit 1; }; done`. Apply the same shape to ac3_2, ac2_4, and ac4_1.
- `objection-5-2` high - Phase 3's ResolutionRequest payload typing threshold is unspecified; an implementer could legitimately ship opaque JsonValue payloads and pass every gate, defeating SDK v0's typed-payload goal.
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:55 and spec phase3.changes
  - Evidence: packages/contracts/src/schemas/resolution.ts:55-86 shows each ResolutionRequest variant carries a structured payload: `input -> questions: Question[]`, `approval -> gate: ApprovalGate`, `cognitive_work -> work: AgentWorkRequest`. Phase 3 prose (line 446-453) says deep nested request payloads `may be represented as JsonValue with an explicit deferred-parity module comment.` There is no rule for where `typed` stops and `JsonValue` begins. The SDK v0 driver explicitly needs to pattern-match resolution requests; if the immediate payload is JsonValue, SDK consumers must walk the JSON tree by hand, undermining the contract crate's purpose. ac3_3's OR-alternation gate would pass either an all-typed or all-opaque implementation.
  - Recommendation: Pin the threshold in Phase 3 Changes: the immediate variant payloads (`questions: Vec<Question>`, `gate: ApprovalGate`, `work: AgentWorkRequest`) MUST be typed Rust shapes ported from agent-work.ts. Anything two levels deeper than that (e.g., per-question schemas, gate inner shapes) MAY be JsonValue with a deferred-parity comment naming a follow-up spec. Add ac3_4: `rg -q 'struct Question' crates/runx-contracts/src/host_protocol.rs && rg -q 'struct ApprovalGate' crates/runx-contracts/src/host_protocol.rs && rg -q 'struct AgentWorkRequest' crates/runx-contracts/src/host_protocol.rs` to enforce the threshold.
- `objection-5-3` medium - ac2_3 enforces hashing exclusivity only against runx-sdk while the spec invariant claims contract hashing belongs to runx-contracts; the invariant is broader than the check.
  - Grounded in: spec.invariants and spec:.scafld/specs/drafts/rust-contracts-parity.md ac2_3
  - Evidence: ac2_3 = `rg -n 'sha2|Sha256|Digest' crates/runx-contracts/src/capability_execution.rs && ! rg -n 'sha2|Sha256|Digest' crates/runx-sdk/src crates/runx-sdk/Cargo.toml`. The negative grep covers only runx-sdk. Invariants line 89-93 say runx-contracts owns shared Rust contract types and consumers must not duplicate them, but six other workspace crates (runx-cli, runx-core, runx-parser, runx-receipts, runx-runtime) are not gated. runx-receipts/runx-runtime in particular have legitimate non-contract reasons to take a sha2 dep (receipt signature verification, adapter content hashing), so the invariant overreaches. Either the check is too narrow or the invariant is too broad.
  - Recommendation: Rewrite the invariant to scope it precisely: `Capability-execution and idempotency hash semantics live in runx-contracts. Other crates may take direct sha2 dependencies for non-contract uses (receipt verification, adapter content addressing) with a one-line rationale in their Cargo.toml comment.` Rename ac2_3 to `Rust SDK v0 does not duplicate contract hashing` so its scope matches the wording. Optionally add a separate invariant audit (not blocking ac2 acceptance) listing the permitted non-contract sha2 callers.
- `objection-5-4` low - ac1_2 inherits workspace-wide style state, so Phase 1 acceptance can fail for unrelated crate breakage; coupling is undocumented.
  - Grounded in: code:scripts/check-rust-core-style.mjs:6
  - Evidence: ac1_2 runs `node scripts/check-rust-core-style.mjs`. scripts/check-rust-core-style.mjs:6-14 walks all seven crate roots. If a parallel rust-state-machine-parity branch introduces a style violation in runx-core/src, this spec's Phase 1 acceptance fails for unrelated reasons. Round-1's narrowing of ac5_3 to `-p runx-contracts` solved the cargo-side fan-out, but the style guard's workspace fan-out persists at Phase 1.
  - Recommendation: Either (a) accept the coupling and add a one-line Invariants note (`Phase 1's ac1_2 inherits workspace-wide style state; unrelated crate violations block Phase 1 until fixed in their owning spec`); or (b) teach `check-rust-core-style.mjs` a `--scope <crate>` flag in Phase 1's Changes block and update ac1_2 to use `--scope runx-contracts`. Option (a) is lower-cost and matches how the style guard is used today.
- `objection-5-5` low - Five named follow-up spec ids are referenced as future owners but none exist as drafts yet; this is normal forward referencing but should be explicit so readers don't search for missing files.
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md follow-up-specs
  - Evidence: The spec names `hash-stable-codepoint-cutover`, `rust-contracts-cli-json-parity`, `rust-receipts-parity`, `rust-registry-parity`, and `rust-tools-parity` as future owners. Glob over `.scafld/specs/**` returns no matches for any of those ids. The cli-json-parity follow-up has a hard dependency on `rust-cli-feature-parity-matrix` which is itself still a draft, so creating cli-json-parity now would be premature. The other four can be drafted independently but aren't yet.
  - Recommendation: Add a single sentence to the Follow-up Specs section: `These task ids are reserved for future drafts; they do not yet exist under .scafld/specs/ and will be created when their respective preconditions land (e.g., rust-contracts-cli-json-parity will be drafted only after rust-cli-feature-parity-matrix is approved).` This avoids future readers searching for non-existent drafts.

Recommended edits:
- Phase 3 Changes (ResolutionRequest payload threshold)
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:55
  - Recommendation: Add: `ResolutionRequest variant payloads (questions, gate, work) MUST be typed Rust shapes (Question, ApprovalGate, AgentWorkRequest ported from packages/contracts/src/schemas/agent-work.ts). Fields two levels deeper than that MAY be JsonValue with a deferred-parity module comment naming a follow-up owner (likely rust-resolution-payload-parity). The threshold is fixed in this phase to prevent silent opaque-JsonValue drift.`
- Phase 3 Acceptance ac3_3 (and analogous ac3_2, ac2_4, ac4_1)
  - Grounded in: spec_gap:phases.phase3.ac3_3
  - Recommendation: Replace the OR-alternation rg with a per-token loop: `for tok in skill_loaded inputs_resolved auth_resolved resolution_requested resolution_resolved admitted executing step_started step_waiting_resolution step_completed warning completed Input Approval CognitiveWork Question ApprovalGate AgentWorkRequest; do rg -q "$tok" crates/runx-contracts/src/host_protocol.rs fixtures/contracts/host-protocol || { echo "missing $tok"; exit 1; }; done`. Apply the same shape to ac3_2 (5 host outcomes), ac2_4 (4 hash-scope markers), and ac4_1 (3 deferred-cli markers).
- Invariants (hashing exclusivity scope)
  - Grounded in: spec.invariants and code:crates/runx-sdk/Cargo.toml:19
  - Recommendation: Rewrite the hashing invariant to: `runx-contracts owns capability-execution and idempotency hash semantics. Other workspace crates MAY take a direct sha2 dependency for non-contract uses (receipt signature verification, adapter content addressing) with a one-line rationale in their Cargo.toml comment.` Rename ac2_3 to `Rust SDK v0 does not duplicate contract hashing` so the check's scope matches the wording.
- Invariants (Phase 1 style guard coupling)
  - Grounded in: code:scripts/check-rust-core-style.mjs:6
  - Recommendation: Add one line: `Phase 1's ac1_2 inherits workspace-wide style state because scripts/check-rust-core-style.mjs walks all seven crate roots. Unrelated crate violations can block Phase 1 acceptance until fixed in their owning spec.`
- Follow-up Specs
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md follow-up-specs
  - Recommendation: Add: `These task ids (hash-stable-codepoint-cutover, rust-contracts-cli-json-parity, rust-receipts-parity, rust-registry-parity, rust-tools-parity) are reserved for future drafts and do not yet exist under .scafld/specs/. They will be created once their preconditions land (e.g., rust-contracts-cli-json-parity is gated on rust-cli-feature-parity-matrix completing).`

### round-6

Status: failed
Started: 2026-05-18T04:19:27Z
Ended: 2026-05-18T04:19:27Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-6 harden: all round-5 acceptance sharpenings are in place (per-token loops in ac2_4, ac3_2, ac3_3, ac4_1, ac4_2, ac5_4, ac5_5), the ResolutionRequest payload threshold is pinned to typed Question/ApprovalGate/AgentWorkRequest with `JsonValue` allowed only two levels deeper, ac2_3 is correctly scoped to SDK-v0 hashing duplication, and the workspace-wide style-guard coupling at ac1_2 is documented in Invariants. Verified spec claims against the codebase: crates/runx-contracts/src/lib.rs holds the 184-line bootstrap surface that Phase 1 migrates into json.rs; Cargo.toml has only `serde` as runtime dep with serde_json as dev-dep, matching the Phase 1 plan; the include allowlist already excludes tests/, matching the Phase 5 packaging policy; crates/runx-sdk/Cargo.toml has no sha2, so ac2_3's negative grep is satisfiable; scripts/check-rust-core-style.mjs walks all seven crate roots, justifying the documented workspace coupling. Three residual low-severity defects remain that are easy edits but real coverage gaps: (1) the ResolutionRequest discriminator FIELD name is `kind` in TypeScript (packages/contracts/src/schemas/resolution.ts:58) but Phase 3 only pins the variant kind values, leaving a Rust implementer free to choose `#[serde(tag = "type")]` (mirroring ExecutionEvent) and break wire parity until a fixture round-trip catches it; (2) ac3_3's token list pins ExecutionEvent and ResolutionRequest shapes but omits nested host types HostRunVerification / HostRunLineage / HostRunApproval, so a stub host_protocol.rs that inlines those as JsonValue would still pass ag3_3; (3) ac3_2's `host_state` token is an opaque fingerprint that any unrelated metadata field could satisfy — a fixture-filename convention or two per-status loops would be sharper. The architecture is sound, the migration story is mechanically clean, and the per-phase rollback recovers the rust-contracts-bootstrap deliverable — fixing the three coverage gaps above is the only remaining work before approval.

Checks:
- path audit
  - Grounded in: code:crates/runx-contracts/src/lib.rs:1 and code:crates/runx-contracts/Cargo.toml:13
  - Result: passed
  - Evidence: All Phase 1–5 paths resolve in the workspace. crates/runx-contracts/src/lib.rs holds 184 lines: JsonObject (line 9), JsonValue (11–20), JsonNumber (22–27), Serialize impl (41–58), Deserialize impl (73–80), JsonNumberVisitor (82–115), Display (117–128), and #[cfg(test)] mod tests (130–183) — exactly the surface Phase 1 migrates to src/json.rs. crates/runx-contracts/Cargo.toml line 13 sets include = ["Cargo.toml", "README.md", "src/**/*.rs"], confirming the packaging policy Phase 5 README documents. crates/Cargo.toml lines 2–10 list all seven workspace members so the `--manifest-path crates/Cargo.toml -p runx-contracts` invocations resolve. The TS sources referenced by Phase 2/3 (packages/contracts/src/schemas/resolution.ts, packages/contracts/src/schemas/agent-work.ts, packages/runtime-local/src/sdk/host-protocol.ts, packages/runtime-local/src/runner-local/index.ts) all exist.
- command audit
  - Grounded in: spec_gap:phases.phase3.ac3_3 and code:packages/contracts/src/schemas/resolution.ts:55
  - Result: failed
  - Evidence: Most acceptance commands are well-formed (per-token loops, --manifest-path consistency, tsx for TS scripts, behavioral ac1_3 round-trip). ac3_2 still uses an opaque `host_state` token alongside the five status values; `rg -q host_state fixtures/contracts/host-protocol` matches any JSON field literal, not a structural shape. A fixture file that mentions `host_state` only as a metadata label could satisfy the gate while skipping the actual HostStateInspector projection. Either replace with a filename convention `inspect-host-state-<status>.json` and grep the filename pattern, or split ac3_2 into two loops (HostRunResult statuses against `result-*.json`, HostRunState statuses against `inspect-*.json`).
- scope/migration audit
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:23 and code:packages/contracts/src/schemas/resolution.ts:55
  - Result: passed
  - Evidence: Host-protocol scope correctly excludes closure types — HostSkillExecutor at host-protocol.ts:23, HostBoundaryResolver at :44, HostStateInspector at :125, HostBridge at :184, and Caller/AuthResolver imported from runner-local — matching the Phase 3 Changes prose. ResolutionRequest payload threshold is pinned: questions: Vec<Question>, gate: ApprovalGate, work: AgentWorkRequest are typed Rust shapes (matching resolution.ts:55–86 + agent-work.ts), with deeper fields allowed as JsonValue under a deferred-parity comment naming rust-resolution-payload-parity. CLI JSON parity is deferred to rust-contracts-cli-json-parity behind rust-cli-feature-parity-matrix (still draft, status:draft, harden_status:not_run), so this spec ships only Phase 4's deferred module home — ac4_0 was removed and ac4_2 actively rejects a fixtures/contracts/cli-json/ directory. Global stableStringify cutover is moved to hash-stable-codepoint-cutover and explicitly out of scope here.
- acceptance timing audit
  - Grounded in: spec:phase1.ac1_2 and code:scripts/check-rust-core-style.mjs:6
  - Result: passed
  - Evidence: Phase 5's ac5_3 is scoped to `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy ... -p runx-contracts --all-targets -- -D warnings && cargo test ... -p runx-contracts && node scripts/check-rust-core-style.mjs` — no workspace-wide cargo fan-out. Phase 1's ac1_2 still walks all seven crate roots because scripts/check-rust-core-style.mjs:6–14 lists `crates/runx-{cli,contracts,core,parser,receipts,runtime,sdk}/src`, but the coupling is now documented in the Invariants block (`Phase 1 ac1_2 inherits workspace-wide Rust style state because scripts/check-rust-core-style.mjs walks all seven crate roots. Unrelated crate violations can block Phase 1 until fixed in their owning spec.`) — accepting the standing-guardrail coupling instead of expanding scope. ac3_3 / ac3_2 / ac2_4 each enforce coverage through per-token loops, not single-match alternations.
- rollback/repair audit
  - Grounded in: spec:rollback.phase1 and code:crates/runx-contracts/src/lib.rs:1
  - Result: passed
  - Evidence: Phase 1 rollback reads `revert runx-contracts to the bootstrap-era JSON surface (JsonValue, JsonNumber, JsonObject, serde impls, and tests) and remove only the new module declarations/dependencies added by this spec. Do not delete the rust-contracts-bootstrap deliverable.` This matches the actual lib.rs head state (184 lines, JsonObject/JsonValue/JsonNumber + Serialize/Deserialize + Display + #[cfg(test)] mod tests). Phases 2–5 rollbacks are additive removals of phase-owned files (capability_execution.rs, host_protocol.rs, cli.rs, receipts.rs, registry.rs, tools.rs, their fixture/test pairs, and fixture-generator scope), which is mechanically clean because each phase's Changes block uses `all, exclusive` ownership. The earlier round-3/4 receipt-signature rollback concern no longer applies because the global stableStringify cutover was relocated to hash-stable-codepoint-cutover — this spec does not edit packages/core/src/util/hash.ts, state-machine/index.ts, or push_outbox/src/index.ts (verified in the Assumptions block and the per-phase rollback).
- design challenge
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:58 and code:packages/runtime-local/src/runner-local/index.ts:138
  - Result: failed
  - Evidence: The contracts-first architecture is sound and the SDK ordering is correctly enforced (rust-sdk-surface-parity:152 names rust-contracts-parity as a hard precondition for SDK Phase 2). The remaining design gap is the tagged-enum discriminator naming. TypeScript uses `kind: Type.Literal("input")` at resolution.ts:58 (also :69, :80) for the ResolutionRequest discriminator, but `readonly type: ...` at runner-local/index.ts:138 for ExecutionEvent. Phase 3 Changes only says `tagged ResolutionRequest variants for input, approval, and cognitive_work` without naming the tag field, while explicitly modelling ExecutionEvent.type as a closed enum. A Rust implementer choosing #[serde(tag = "type", rename_all = "snake_case")] for both (by symmetry with ExecutionEvent) would break wire parity for ResolutionRequest silently — fixture round-trip at ac3_1 would eventually catch it, but ac3_3 alone does not. ac3_3 also omits the three nested host-state shapes (HostRunVerification, HostRunLineage, HostRunApproval) from host-protocol.ts:102–118, so a stub host_protocol.rs that inlines verification/lineage/approval as JsonValue could still pass.

Questions:
- Should the spec explicitly pin the ResolutionRequest discriminator field name (`kind`) so the Rust serde tag is unambiguous?
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:58
  - Recommended answer: Yes. The TypeScript schemas use `kind: Type.Literal("input")`, `kind: Type.Literal("approval")`, and `kind: Type.Literal("cognitive_work")` (resolution.ts:58, 69, 80). ExecutionEvent uses `type` (runner-local/index.ts:138–151). Add one line to the Phase 3 Changes block: `ResolutionRequest is modelled as a Rust enum with #[serde(tag = "kind", rename_all = "snake_case")] to match packages/contracts/src/schemas/resolution.ts; ExecutionEvent uses #[serde(tag = "type", rename_all = "snake_case")].` Without this, an implementer might choose tag = "type" for both and only catch the parity break at fixture round-trip.
  - If unanswered: Default to recording the tag names in Phase 3 Changes (kind for ResolutionRequest, type for ExecutionEvent) so the JSON shape is locked to the TypeScript source.
- Should ac3_3 also enumerate nested host shapes (HostRunVerification, HostRunLineage, HostRunApproval) so their Rust homes are locked, given they appear in every HostTerminalState projection?
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:102
  - Recommended answer: Yes. host-protocol.ts:102–118 declares HostRunVerification (status, reason), HostRunLineage (kind, sourceRunId, sourceReceiptId), and HostRunApproval (gateId, gateType, decision, reason); HostTerminalState at lines 143–159 references them in every completed/failed/escalated/denied result. Add `HostRunVerification HostRunLineage HostRunApproval` to ac3_3's token loop so the gate fails if those Rust structs are missing. The fixture round-trip will catch silent omissions, but explicit tokens stop a stub host_protocol.rs from passing ac3_3 while skipping the nested shapes.
  - If unanswered: Default to adding HostRunVerification, HostRunLineage, and HostRunApproval to ac3_3's per-token loop.
- Is `host_state` in ac3_2's token loop a fixture-filename fingerprint, a JSON content marker, or something else? It is not one of the five host status values (paused/completed/failed/escalated/denied).
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md ac3_2
  - Recommended answer: Document the intent inline. The `host_state` token most likely fingerprints the HostStateInspector projection (HostRunState — distinct from HostRunResult), since the spec mentions `inspect projections`. Add a fixture-filename convention, e.g., `inspect-host-state-<status>.json`, so the rg gate catches missing inspect coverage. Otherwise rename the token to `inspect` or split ac3_2 into two checks (one per family of projections).
  - If unanswered: Default to renaming the token to `inspect` and adding a fixture-filename convention `inspect-host-state-<status>.json` in Phase 3 Changes so the gate's intent is explicit.

Design objections:
- `objection-6-1` low - ResolutionRequest discriminator field name (`kind`) is not pinned in the spec; a Rust implementer could mirror ExecutionEvent's `type` and silently break wire parity.
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:58
  - Evidence: packages/contracts/src/schemas/resolution.ts:58 uses `kind: Type.Literal("input")` (also lines 69, 80 for approval/cognitive_work). ExecutionEvent at packages/runtime-local/src/runner-local/index.ts:138 uses `readonly type: ...`. Phase 3 Changes says only `tagged ResolutionRequest variants for input, approval, and cognitive_work` without naming the tag field, while explicitly modelling ExecutionEvent.type as a closed enum. A Rust implementer could pick `#[serde(tag = "type", rename_all = "snake_case")]` for both by symmetry, which would silently break wire parity for ResolutionRequest. Fixture round-trip at ac3_1 will catch it, but ac3_3 alone does not, and the discriminator field is the most error-prone surface in a tagged-enum port.
  - Recommendation: Add one line to Phase 3 Changes: `ResolutionRequest uses #[serde(tag = "kind", rename_all = "snake_case")] (matching packages/contracts/src/schemas/resolution.ts); ExecutionEvent uses #[serde(tag = "type", rename_all = "snake_case")] (matching packages/runtime-local/src/runner-local/index.ts:138).` Optionally extend ac3_3 to grep for both tag-attribute literals in host_protocol.rs.
- `objection-6-2` low - ac3_3 token list omits HostRunVerification, HostRunLineage, and HostRunApproval, leaving the nested host-state shapes unenforced by the coverage gate.
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:102
  - Evidence: host-protocol.ts:102–118 declares HostRunVerification, HostRunLineage, and HostRunApproval. HostTerminalState at lines 143–159 holds verification/lineage/approval references in every completed/failed/escalated/denied state. ac3_3's per-token loop covers ExecutionEvent, ResolutionRequest, ResolutionResponse, Input, Approval, CognitiveWork, Question, ApprovalGate, AgentWorkRequest, and the 12 ExecutionEvent variant strings — but not the three nested host projections. A host_protocol.rs that skips Verification/Lineage/Approval and inlines those fields as opaque JsonValue would still pass ac3_3, defeating the typed-payload goal that ac3_3 is meant to enforce.
  - Recommendation: Append `HostRunVerification HostRunLineage HostRunApproval` to ac3_3's `for tok in ...` list so the gate fails if those Rust structs are missing from host_protocol.rs or the fixture corpus.
- `objection-6-3` low - ac3_2's `host_state` token has no documented meaning; the gate's intent is ambiguous and trivially satisfiable by an unrelated metadata field.
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md ac3_2
  - Evidence: ac3_2 = `for tok in paused completed failed escalated denied host_state; do rg -q "$tok" fixtures/contracts/host-protocol || { echo missing $tok; exit 1; }; done`. Five of the six tokens are HostRunResult/HostRunState status values; `host_state` is not. The spec's Phase 3 prose mentions `inspect projections` (HostStateInspector returns HostRunState), so `host_state` is presumably a fingerprint for that family — but the spec never names a fixture-filename convention. An implementer could satisfy the gate with any single fixture that happens to contain the literal `host_state` in any field, even unrelated metadata, while shipping zero actual HostStateInspector projections.
  - Recommendation: Replace `host_state` with a documented filename pattern such as `inspect-host-state-completed.json` (or a metadata-field convention `"projection": "host_state"`) and document the pattern in Phase 3 Changes. Alternatively split ac3_2 into two loops — one over the five HostRunResult statuses against `fixtures/contracts/host-protocol/result-*.json` and one over the five HostRunState statuses against `fixtures/contracts/host-protocol/inspect-*.json`.

Recommended edits:
- Phase 3 Changes
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:58 and code:packages/runtime-local/src/runner-local/index.ts:138
  - Recommendation: Pin the serde tag attributes: `ResolutionRequest is modelled as a Rust enum with #[serde(tag = "kind", rename_all = "snake_case")] matching packages/contracts/src/schemas/resolution.ts; ExecutionEvent is modelled with #[serde(tag = "type", rename_all = "snake_case")] matching packages/runtime-local/src/runner-local/index.ts:138.` This eliminates the only remaining tagged-enum ambiguity in the host-protocol port.
- Phase 3 Acceptance ac3_3
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:102
  - Recommendation: Extend the token list to include `HostRunVerification HostRunLineage HostRunApproval` so the per-token loop also enforces the three nested host projections that appear in every HostTerminalState (verified at host-protocol.ts:143–159).
- Phase 3 Acceptance ac3_2
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md ac3_2
  - Recommendation: Replace the opaque `host_state` token with either a fixture-filename pattern (`inspect-host-state-<status>.json`) or two separate per-status loops (one over HostRunResult statuses against result-*.json, one over HostRunState statuses against inspect-*.json). Document the convention in Phase 3 Changes so reviewers know the gate's intent.

### round-7

Status: passed
Started: 2026-05-18T04:28:51Z
Ended: 2026-05-18T04:28:51Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-7 harden: spec is mature and approval-ready. All round-6 findings are mechanically addressed: ResolutionRequest is pinned to `#[serde(tag = "kind", rename_all = "snake_case")]` (matching packages/contracts/src/schemas/resolution.ts:58) while ExecutionEvent stays on `#[serde(tag = "type")]` (matching packages/runtime-local/src/runner-local/index.ts:138), and ac3_3 grep-verifies both tag literals; HostRunVerification, HostRunLineage, and HostRunApproval are in ac3_3's per-token loop (host-protocol.ts:102-118); the loose `host_state` token is gone, replaced by documented filename conventions `result-host-run-<status>.json` and `inspect-host-state-<status>.json` checked via `test -f` per status. Codebase grounding holds: crates/runx-contracts/src/lib.rs has the 184-line bootstrap surface that Phase 1 migrates (and Phase 1 rollback preserves); Cargo.toml has only `serde` as runtime dep with serde_json as dev-dep and `include = ["Cargo.toml", "README.md", "src/**/*.rs"]` so tests/fixtures are correctly excluded from the packaged crate; crates/runx-sdk/Cargo.toml has no sha2 so ac2_3's negative grep is satisfiable; scripts/check-rust-core-style.mjs walks all 7 crate roots and the resulting Phase 1 coupling is documented in Invariants; rust-cli-feature-parity-matrix is still in drafts but Phase 4 is correctly narrowed to a deferred module home with no upstream dependency check. The previously load-bearing risks (locale-compare cutover, CLI JSON parity, deeper resolution payloads) have been excised from this spec into named follow-up specs (hash-stable-codepoint-cutover, rust-contracts-cli-json-parity, rust-resolution-payload-parity). Verdict: pass.

Checks:
- path audit
  - Grounded in: code:crates/runx-contracts/src/lib.rs:1 and code:crates/runx-contracts/Cargo.toml:13
  - Result: passed
  - Evidence: Verified all spec paths in the codebase. crates/runx-contracts/src/lib.rs is the 184-line bootstrap surface: JsonObject (line 9), JsonValue (11-20), JsonNumber (22-27), Serialize/Deserialize/Display impls (41-128), and #[cfg(test)] mod tests (130-183) — exactly what Phase 1 migrates to src/json.rs and what Phase 1 rollback preserves. crates/runx-contracts/Cargo.toml has `serde` as the only runtime dep, serde_json as dev-dep, and `include = ["Cargo.toml", "README.md", "src/**/*.rs"]` on line 13 so tests/ and fixtures/ are not packaged. crates/Cargo.toml lists all 7 workspace members so `--manifest-path crates/Cargo.toml -p runx-contracts` resolves. TS sources cited by Phase 2/3 (resolution.ts, agent-work.ts, host-protocol.ts, runner-local/index.ts) all exist. Follow-up spec ids are correctly marked as reserved-not-yet-drafted.
- command audit
  - Grounded in: spec:.scafld/specs/drafts/rust-contracts-parity.md (acceptance, phase commands) and code:package.json:36
  - Result: passed
  - Evidence: All TS scripts run through `pnpm exec tsx` (v6, ac1_3, ac2_2) matching the repo convention at package.json:36 (`tsx scripts/check-fixture-key-order.ts`). All cargo invocations use `--manifest-path crates/Cargo.toml -p runx-contracts` consistent with the workspace at crates/Cargo.toml. ac1_3 is a behavioral round-trip (good-fixture exit 0, bad-fixture exit non-zero) rather than a substring check. ac3_3 now greps `tag = "kind"` and `tag = "type"` separately to enforce the serde discriminator field. ac3_2 uses `test -f` against documented filename conventions (`result-host-run-<status>.json` and `inspect-host-state-<status>.json`), removing the loose `host_state` token. v7 negative-grep boundary is sound. Per-token loops in ac2_4, ac3_2, ac3_3, ac4_1, ac5_4, ac5_5 use `&&` chaining so missing any single token fails the gate.
- scope/migration audit
  - Grounded in: code:packages/runtime-local/src/sdk/host-protocol.ts:23 and code:packages/contracts/src/schemas/resolution.ts:55
  - Result: passed
  - Evidence: Host-protocol scope correctly excludes closure types HostSkillExecutor (host-protocol.ts:23), HostBoundaryResolver (:44), HostStateInspector (:125), HostBridge (:184), and Caller/AuthResolver from runner-local. ResolutionRequest payload threshold is pinned: `Input { questions: Vec<Question> }`, `Approval { gate: ApprovalGate }`, `CognitiveWork { work: AgentWorkRequest }` are typed Rust shapes matching resolution.ts:55-86 plus agent-work.ts:14/31/48 schemas; deeper fields may stay as JsonValue with a deferred-parity comment naming rust-resolution-payload-parity. CLI JSON parity is deferred to rust-contracts-cli-json-parity behind rust-cli-feature-parity-matrix; Phase 4 ships only the deferred module home and ac4_2 actively rejects fixtures/contracts/cli-json/. Global stableStringify cutover is excised to hash-stable-codepoint-cutover — assumptions explicitly state this spec does not edit packages/core/src/util/hash.ts, state-machine/index.ts, or push_outbox/src/index.ts. Capability hash scope correctly restricted to ASCII-key shapes via fixture generator guard (ac2_4).
- acceptance timing audit
  - Grounded in: spec:phase1.ac1_2 and code:scripts/check-rust-core-style.mjs:6
  - Result: passed
  - Evidence: Phase 5 ac5_3 is correctly narrowed to `-p runx-contracts`; no workspace-wide cargo fan-out remains. Phase 1's ac1_2 still walks all 7 crate roots because scripts/check-rust-core-style.mjs:6-14 lists every crate, but this standing-guardrail coupling is explicitly documented in the Invariants block ('Phase 1 ac1_2 inherits workspace-wide Rust style state because scripts/check-rust-core-style.mjs walks all seven crate roots. Unrelated crate violations can block Phase 1 until fixed in their owning spec.'). The Phase 4 dependency on rust-cli-feature-parity-matrix completing was removed by narrowing Phase 4 to a deferred module home — no acceptance depends on an external spec lifecycle.
- rollback/repair audit
  - Grounded in: spec:rollback.phase1 and code:crates/runx-contracts/src/lib.rs:1
  - Result: passed
  - Evidence: Phase 1 rollback reads 'revert runx-contracts to the bootstrap-era JSON surface (JsonValue, JsonNumber, JsonObject, serde impls, and tests) and remove only the new module declarations/dependencies added by this spec. Do not delete the rust-contracts-bootstrap deliverable.' This matches the actual lib.rs head state byte-for-byte (184 lines covering JsonObject + JsonValue + JsonNumber + Serialize/Deserialize + Display + #[cfg(test)] mod tests). Phases 2-5 rollbacks are additive removals of phase-exclusive files plus their fixture/test pairs — mechanically clean because each phase's Changes block uses `all, exclusive` ownership for the new files. The earlier-round receipt-signature rollback hazard no longer applies because the global stableStringify cutover was relocated to hash-stable-codepoint-cutover; this spec does not touch any persisted-hash codepath. Phase 2's rollback note explicitly states 'No TypeScript stable-stringify files are edited by this phase.'
- design challenge
  - Grounded in: code:packages/contracts/src/schemas/resolution.ts:58 and code:packages/runtime-local/src/runner-local/index.ts:138
  - Result: passed
  - Evidence: Tagged-enum discriminator naming is now unambiguous: ResolutionRequest uses `#[serde(tag = "kind", rename_all = "snake_case")]` matching resolution.ts:58/69/80 (which use `kind: Type.Literal(...)`), while ExecutionEvent uses `#[serde(tag = "type", rename_all = "snake_case")]` matching runner-local/index.ts:138. ac3_3 grep-enforces both tag literals so a symmetric mis-port (tag = "type" for both) fails the gate. The 12-variant ExecutionEvent closed enum is enumerated in Phase 3 prose and pinned by ac3_3's per-token loop. HostRunVerification/HostRunLineage/HostRunApproval are typed Rust structs (per spec) and present in ac3_3's token list, so the typed-payload goal cannot be defeated by opaque JsonValue inlining. Contracts-first SDK ordering is enforced via the `contracts-first-ordering:` marker checked by ac5_2. The seven-crate dep graph (contracts at the root, no IO/runtime deps) is locked by v7's negative grep against tokio/reqwest/hyper/rmcp/clap/std::fs/std::process/std::net/std::env/Command::new.

Questions:
- none


## Planning Log

- 2026-05-17T01:30:00Z: Drafted as the missing contracts-first spec. It makes
  `runx-contracts` the owner of host-protocol, capability-execution, idempotency
  hash, and consumed CLI JSON contracts before `runx-sdk` Phase 2 can ship.
- 2026-05-18T04:18:00Z: Addressed round-2 Claude harden findings. Phase 1 now
  keeps dependencies minimal, Phase 2 owns `sha2` and the TypeScript
  `stableStringify` code-point-ordering cutover, Phase 4 no longer depends on
  `jq`, Phase 5 uses a stable `contracts-first-ordering:` marker, and README
  changes must document that workspace fixture tests are intentionally excluded
  from the packaged crate.
- 2026-05-18T04:31:00Z: Addressed round-3 Claude harden findings. The
  stable-stringify cutover now requires an observed-shape migration audit and
  explicit rollback, Phase 3 enumerates the host-protocol transitive wire
  closure, Phase 5 no longer reaches into the SDK spec for its marker check,
  and approval is blocked until `rust-cli-feature-parity-matrix` is approved.
- 2026-05-18T04:45:00Z: Addressed round-4 Claude harden findings by splitting
  risky work out of this spec. The global `stableStringify` comparator
  migration now belongs to `hash-stable-codepoint-cutover`; CLI JSON parity now
  belongs to `rust-contracts-cli-json-parity` after
  `rust-cli-feature-parity-matrix`; Phase 4 is only a deferred `cli.rs` module
  home; and host-protocol fixtures must pin every closed `ExecutionEvent`
  variant.
- 2026-05-18T05:02:00Z: Addressed round-5 Claude harden findings. Replaced
  OR-alternation acceptance greps with per-token loops, pinned the
  `ResolutionRequest` immediate payload threshold to typed `Question`,
  `ApprovalGate`, and `AgentWorkRequest` shapes, scoped `ac2_3` to SDK v0
  contract-hash duplication, documented the workspace-wide style guard
  coupling, and clarified that follow-up spec ids are reserved but not yet
  required to exist.
- 2026-05-18T05:20:00Z: Addressed round-6 Claude harden findings. Pinned
  `ResolutionRequest` to serde tag `kind` while keeping `ExecutionEvent` on
  serde tag `type`, added `HostRunVerification`, `HostRunLineage`, and
  `HostRunApproval` to the explicit host-protocol closure, and replaced the
  loose `host_state` fixture token with result and inspect fixture filename
  conventions per host status.
