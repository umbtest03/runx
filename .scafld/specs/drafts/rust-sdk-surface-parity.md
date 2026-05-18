---
spec_version: '2.0'
task_id: rust-sdk-surface-parity
created: '2026-05-17T00:45:00Z'
updated: '2026-05-17T01:30:00Z'
status: draft
harden_status: not_run
size: large
risk_level: medium
---

# Rust SDK surface parity

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld harden rust-sdk-surface-parity`
Latest runner update: none
Review gate: not_started

## Summary

Plan and implement the first Rust SDK crate for runx. The Cargo package is
`runx-sdk` and the Rust crate name is `runx_sdk`. The first version is a
library-only SDK that talks to the authoritative `runx` CLI JSON interface,
mirrors the existing Python SDK shape, and uses `runx-contracts` for typed
host-protocol and act-assignment helpers.

This is not a native Rust runtime. It must not execute skills directly,
evaluate policy, run MCP, write receipts, or bypass the TypeScript runtime.
Until `runx-runtime` exists and passes the runtime/CLI parity gates, the Rust
SDK shells out to the installed `runx` binary and parses documented JSON.

`cargo search runx-sdk` returned no matching crate during planning on
2026-05-17, but crate-name availability must be rechecked immediately before
publish. Publishing is out of scope for this spec.

## Context

CWD: `.`

Packages:
- `crates/runx-contracts`
- `crates/runx-sdk`
- `crates/runx-cli`
- `@runxhq/runtime-local`
- `packages/sdk-python`

Existing SDK surfaces:
- `packages/runtime-local/src/sdk/index.ts` exposes the TypeScript runtime SDK:
  `RunxSdk`, `createRunxSdk`, `runSkill`, `hostRun`, `hostResume`,
  `inspectHost`, search/add/publish/history/receipt/tool/connect methods.
- `packages/runtime-local/src/sdk/act-assignment.ts` owns capability
  execution envelopes and idempotency hash helpers.
- `packages/runtime-local/src/sdk/host-protocol.ts` owns host bridge types,
  paused/completed/failed/escalated/denied results, resume handling, and host
  state inspection projections.
- `packages/sdk-python` is the best first-version precedent: a thin CLI JSON
  client plus host-protocol helpers, not a native runtime.

Files impacted:
- `crates/Cargo.toml`
- `crates/runx-contracts/**` (dependency only; owned by `rust-contracts-parity`)
- `crates/runx-sdk/Cargo.toml`
- `crates/runx-sdk/README.md`
- `crates/runx-sdk/src/lib.rs`
- `crates/runx-sdk/src/client.rs`
- `crates/runx-sdk/src/error.rs`
- `crates/runx-sdk/src/command.rs`
- `crates/runx-sdk/src/host.rs`
- `crates/runx-sdk/src/act_assignment.rs`
- `crates/runx-sdk/tests/client_cli.rs`
- `crates/runx-sdk/tests/host_protocol.rs`
- `crates/runx-sdk/tests/act_assignment.rs`
- `fixtures/sdk-rust/**`
- `scripts/check-rust-core-style.mjs`
- `docs/api-surface.md`
- `docs/rust-kernel-architecture.md`

Invariants:
- TypeScript CLI/runtime remains authoritative.
- Rust SDK v0 is a client SDK over `runx --json`, not a native runtime.
- The SDK never parses human CLI output.
- The SDK never shells through a system shell; use `std::process::Command`
  with explicit argv.
- Public Rust API follows `docs/rust-kernel-architecture.md` section 18:
  idiomatic names, typed results, concrete errors, no wildcard re-exports,
  no public `serde_json::Value`, and no dynamic error erasure.
- SDK v0 depends on `runx-contracts`, not `runx-core`. This is what keeps the
  SDK shippable before kernel parity is complete.
- SDK v0 must not duplicate host-protocol or act-assignment contract
  types that belong in `runx-contracts`.
- Blocking subprocess behavior is acceptable in v0. Async runtime support is a
  follow-up feature, not a default dependency.
- `runx-sdk` may depend on `runx-contracts` and `serde_json`. It must not
  depend on `runx-core`, `sha2`, `tokio`, `reqwest`, `hyper`, `rmcp`, or
  `clap` in v0.
- The SDK crate must not depend on `runx-cli`; it calls a configurable `runx`
  command on PATH.

Related docs:
- `docs/rust-kernel-architecture.md`
- `docs/api-surface.md`
- `packages/sdk-python/README.md`
- `packages/runtime-local/src/sdk/index.ts`
- `packages/runtime-local/src/sdk/host-protocol.ts`
- `packages/runtime-local/src/sdk/act-assignment.ts`

## Objectives

- Upgrade the existing `runx-sdk` placeholder into a publishable Rust SDK
  crate surface.
- Implement a typed blocking CLI client matching the Python SDK baseline:
  search skills, run skill, resume run, and connect list.
- Reuse `runx-contracts` host protocol models and normalization helpers for
  paused, completed, failed, escalated, and denied outcomes.
- Reuse `runx-contracts` act-assignment envelope and idempotency helpers
  with stable hashing compatible with TypeScript.
- Add deterministic fixture tests using fake `runx` executables and checked-in
  JSON payloads.
- Document that native runtime support is future work behind a later
  `native-runtime` feature.

## Scope

In scope:
- `runx-sdk` placeholder upgrade.
- Blocking CLI client over `runx --json`.
- SDK-facing wrappers around `runx-contracts` host protocol types.
- SDK-facing wrappers around `runx-contracts` act assignment helpers.
- Fixtures proving parity with Python SDK behavior and TypeScript SDK JSON
  shapes.
- Rust style guard coverage for `runx-sdk`.

Out of scope:
- Publishing `runx-sdk` to crates.io.
- Async client or tokio integration.
- Native runtime execution through `runx-runtime`.
- MCP client/server helpers.
- Provider-framework adapters for OpenAI, Anthropic, LangChain, Vercel AI, or
  CrewAI. Those are follow-up crates/features after the base SDK is stable.
- Registry/auth/connect service implementations beyond CLI JSON calls.

## Dependencies

- `rust-runx-cli-placeholder` should exist so Rust users can install a `runx`
  binary through Cargo if they want to.
- `rust-contracts-parity` must be complete before SDK Phase 2. SDK v0 is not
  allowed to hand-write host-protocol or act-assignment types that later
  move to `runx-contracts`.
- `rust-cli-feature-parity-matrix` must define the JSON CLI cases the SDK can
  rely on before those methods are exposed. The SDK may start with the subset
  already covered by Python SDK tests, but it must not claim broader CLI
  parity.
- `scripts/check-rust-core-style.mjs` exists and is extended to scan
  `crates/runx-sdk/src`.

## Assumptions

- The CLI JSON output for the initial SDK methods is stable enough to treat as
  an SDK contract because `rust-cli-feature-parity-matrix` owns the consumed
  cases. If a method lacks a CLI JSON fixture, it is not exposed.
- `runx-contracts` owns the stable host-protocol, act-assignment, and
  idempotency hash contracts. SDK code may wrap those types but must not fork
  them.
- Rust callers can tolerate blocking subprocess calls in v0. This avoids a
  default async runtime dependency and keeps the first crate small. This is a
  transitional SDK shape, not the final async story.
- The future async SDK path is explicit: after the consumed contracts and
  runtime surfaces exist, a follow-up spec may make `runx-sdk` expose async
  APIs by default with a `blocking` facade feature or sibling facade module.
  That follow-up owns `tokio`/runtime integration; v0 does not.
- `runx-sdk` can use typed wrappers that expose stable fields while preserving
  raw JSON internally as private implementation detail where needed.

## Risks

- High: SDK method surface can imply runtime maturity that does not exist.
  Mitigated by explicit docs: v0 is CLI-backed only.
- Medium: CLI JSON output may be less stable than internal TS SDK types.
  Mitigated by adding SDK-specific fixtures, requiring
  `rust-cli-feature-parity-matrix`, and only exposing methods covered by
  fixtures.
- Medium: SDK and contracts can diverge if SDK redefines contract types.
  Mitigated by the hard dependency on `rust-contracts-parity` before Phase 2
  and validation that `runx-sdk` depends on `runx-contracts`, not `runx-core`.
- Medium: public API can become verbose if TypeScript interfaces are copied
  mechanically. Mitigated by the section 18 Rust quality bar and
  `check-rust-core-style.mjs`.
- Medium: blocking subprocess calls can surprise async Rust users. Mitigated
  by naming the client blocking and explicitly planning the async follow-up
  instead of letting `tokio` drift into v0.
- Low: `runx-sdk` is already reserved on crates.io at `0.0.1`; publication
  risk shifts to avoiding user confusion until the first real SDK release.

## Acceptance

Profile: strict
Self-eval threshold: 8
Review provider: external Claude; local review does not satisfy
complete.
Harden required before approve: yes

Definition of done:
- [ ] `dod1` `crates/runx-sdk` exists as a workspace member and packages
  cleanly.
- [ ] `dod2` `RunxClient` supports the initial CLI-backed methods with typed
  results and concrete errors.
- [ ] `dod3` Host protocol and act-assignment helpers match checked-in
  JSON fixtures through `runx-contracts`.
- [ ] `dod4` The SDK docs clearly state v0 is CLI-backed and does not replace
  the TypeScript runtime.
- [ ] `dod5` Rust style guard, fmt, clippy, and tests pass.
- [ ] `dod6` No async runtime, HTTP, MCP, or CLI parser crates are added in
  v0. Async support requires a follow-up spec.
- [ ] `dod7` `runx-sdk` depends on `runx-contracts` and does not depend on
  `runx-core` in v0.

Validation:
- [ ] `v1` command - Rust SDK tests pass.
  - Command: `cargo test -p runx-sdk`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v2` command - Rust formatting and clippy pass.
  - Command: `cargo fmt --all --check && cargo clippy -p runx-sdk --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v3` command - Rust style guard passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v4` command - SDK builds without package-publication assumptions.
  - Command: `cargo check -p runx-sdk`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v5` command - v0 dependency boundary is clean.
  - Command: `rg -n 'runx-contracts' crates/runx-sdk/Cargo.toml && ! rg -n 'runx-core|sha2|tokio|reqwest|hyper|rmcp|clap' crates/runx-sdk/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v6` command - docs do not claim native runtime support.
  - Command: `! rg -n 'native runtime|executes skills directly|without the runx CLI' crates/runx-sdk/README.md docs/api-surface.md`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v7` command - cargo metadata confirms SDK depends on contracts only
  among runx libraries.
  - Command: `cargo metadata --format-version 1 --no-deps | jq -e '.packages[] | select(.name == "runx-sdk") | .dependencies | (map(.name) | index("runx-contracts") != null and index("runx-core") == null)'`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none

## Phase 1: SDK crate upgrade and style guard

Goal: Replace the placeholder-only `runx-sdk` surface with the initial SDK
module layout, without behavior.

Status: pending
Dependencies: `rust-contracts-parity` Phase 1

Changes:
- `crates/Cargo.toml` (partial, shared) - Verify `runx-sdk` remains a
  workspace member.
- `crates/runx-sdk/Cargo.toml` (partial, exclusive) - Keep package metadata,
  edition 2024, workspace lints, library target, and depend on
  `runx-contracts` plus `serde_json` only as needed for CLI decoding.
- `crates/runx-sdk/README.md` (partial, exclusive) - Expand CLI-backed v0
  docs.
- `crates/runx-sdk/src/lib.rs` (partial, exclusive) - Replace placeholder
  constants with declared modules and explicit re-exports. No wildcard
  re-exports.
- `scripts/check-rust-core-style.mjs` (partial, shared) - Verify
  `crates/runx-sdk/src` remains in the scan roots.

Acceptance:
- [ ] `ac1_1` command - workspace metadata sees runx-sdk.
  - Command: `cargo metadata --format-version 1 --no-deps | jq -e '.packages[] | select(.name == "runx-sdk")'`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac1_2` command - style guard passes on skeleton.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Blocking CLI client

Goal: Implement the first Rust SDK client over `runx --json`.

Status: pending
Dependencies: Phase 1
Additional dependency: `rust-contracts-parity` complete and consumed
`rust-cli-feature-parity-matrix` CLI JSON cases defined.

Changes:
- `crates/runx-sdk/src/client.rs` (all, exclusive) - `RunxClient`,
  `RunxClientOptions`, typed methods `search_skills`, `run_skill`,
  `resume_run`, `connect_list`.
- `crates/runx-sdk/src/command.rs` (all, exclusive) - Command planning and
  subprocess invocation. No shell interpolation.
- `crates/runx-sdk/src/error.rs` (all, exclusive) - Concrete `RunxError` enum
  with command status, stderr, JSON parse, and contract-shape variants.
- `crates/runx-sdk/tests/client_cli.rs` (all, exclusive) - Fake-runx tests
  matching the Python SDK behavior.

Acceptance:
- [ ] `ac2_1` command - CLI client tests pass.
  - Command: `cargo test -p runx-sdk --test client_cli`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2_2` command - no shell invocation.
  - Command: `! rg -n 'sh -c|cmd /C|Command::new\\(\"sh\"\\)|Command::new\\(\"cmd\"\\)' crates/runx-sdk/src crates/runx-sdk/tests`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Capability execution SDK wrappers

Goal: Expose SDK-facing wrappers around `runx-contracts` act assignment
types and idempotency helpers without duplicating hashing logic.

Status: pending
Dependencies: Phase 2

Changes:
- `crates/runx-sdk/src/act_assignment.rs` (all, exclusive) - Typed
  SDK constructors and re-exports around `runx-contracts` act assignment
  types. No independent hash implementation.
- `fixtures/sdk-rust/act-assignment/*.json` (all, exclusive) - Golden
  cases generated from TypeScript and shared with `runx-contracts`.
- `crates/runx-sdk/tests/act_assignment.rs` (all, exclusive) - Fixture
  tests proving SDK wrappers produce the same contract values and hashes.

Acceptance:
- [ ] `ac3_1` command - act assignment tests pass.
  - Command: `cargo test -p runx-sdk --test act_assignment`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac3_2` command - fixture JSON is deterministic.
  - Command: `node scripts/check-fixture-key-order.ts fixtures/sdk-rust/act-assignment`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac3_3` command - SDK does not implement hashing independently.
  - Command: `! rg -n 'sha2|Sha256|Digest' crates/runx-sdk/src crates/runx-sdk/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 4: Host protocol SDK wrappers

Goal: Expose SDK-facing wrappers around `runx-contracts` host-protocol
result/state normalization.

Status: pending
Dependencies: Phase 3

Changes:
- `crates/runx-sdk/src/host.rs` (all, exclusive) - Host result/state enums,
  bridge traits or callback structs, normalization from CLI/runtime JSON, and
  provider-neutral result summaries, all backed by `runx-contracts` types.
- `fixtures/sdk-rust/host-protocol/*.json` (all, exclusive) - Paused,
  completed, failed, escalated, denied, and terminal state fixtures.
- `crates/runx-sdk/tests/host_protocol.rs` (all, exclusive) - Fixture tests.

Acceptance:
- [ ] `ac4_1` command - host protocol tests pass.
  - Command: `cargo test -p runx-sdk --test host_protocol`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac4_2` command - no provider-specific adapters in base SDK.
  - Command: `! rg -n 'OpenAI|Anthropic|LangChain|CrewAI|Vercel' crates/runx-sdk/src crates/runx-sdk/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 5: Docs and workspace check

Goal: Document the SDK honestly and verify workspace readiness without
publishing. Package verification is deferred until `runx-contracts` is
publishable or a crates.io reservation spec approves local publication order.

Status: pending
Dependencies: Phase 4

Changes:
- `crates/runx-sdk/README.md` (partial, exclusive) - Usage examples, CLI
  requirement, error handling, and native-runtime deferral.
- `docs/api-surface.md` (partial, shared) - Add Rust SDK surface entry.
- `docs/rust-kernel-architecture.md` (partial, shared) - Keep crate graph and
  early CLI-backed SDK note aligned with implementation.

Acceptance:
- [ ] `ac5_1` command - SDK workspace checks pass.
  - Command: `cargo check -p runx-sdk && cargo test -p runx-sdk`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac5_2` command - docs state CLI-backed v0.
  - Command: `rg -n 'CLI-backed|installed runx|runx --json|native-runtime' crates/runx-sdk/README.md docs/api-surface.md docs/rust-kernel-architecture.md`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac5_3` command - full Rust workspace tests pass.
  - Command: `cargo test --workspace`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

Strategy: per_phase

Commands:
- Phase 1: revert `crates/runx-sdk` to its placeholder-only surface and keep
  workspace membership.
- Phase 2: remove CLI client modules and tests.
- Phase 3: remove act assignment module, fixtures, and tests.
- Phase 4: remove host protocol module, fixtures, and tests.
- Phase 5: revert docs updates.

## Review

Status: not_started
Verdict: none
Timestamp: none
Review rounds: none
Reviewer mode: none
Reviewer session: none
Round status: none
Override applied: none
Override reason: none
Override confirmed at: none
Reviewed head: none
Reviewed dirty: none
Reviewed diff: none

Reviewer requirements:
- Verify the SDK does not imply native runtime support.
- Verify the public API is idiomatic Rust and not TypeScript-shaped.
- Verify every public method is backed by CLI JSON or fixture evidence.
- Verify no async/runtime/MCP/provider dependency slipped into v0.

Findings:
- none

Passes:
- none

## Self Eval

Status: not_started
Completeness: none
Architecture fidelity: none
Spec alignment: none
Validation depth: none
Total: none
Second pass performed: none

## Deviations

- none

## Metadata

Estimated effort hours: 18
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- sdk
- cargo
- host-protocol

## Origin

Source:
- User requested a Rust SDK plan in addition to the Rust CLI/kernel/runtime
  planning.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- after: rust-runx-cli-placeholder
- coordinates_with: rust-cli-feature-parity-matrix
- before: rust-runtime-orchestration

## Harden Rounds

- none

## Planning Log

- 2026-05-17T00:45:00Z: Drafted as a CLI-backed Rust SDK plan. Anchored the
  first version on the existing Python SDK pattern and TypeScript
  `@runxhq/runtime-local/sdk` host/capability surfaces. Deferred async,
  provider adapters, MCP, and native runtime support to follow-up specs.
- 2026-05-17T12:35:00Z: Clarified that blocking SDK v0 is transitional.
  Future async SDK work needs its own spec and should not pull `tokio` into
  the CLI-backed v0 surface.
