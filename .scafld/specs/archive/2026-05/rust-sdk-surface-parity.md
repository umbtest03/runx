---
spec_version: '2.0'
task_id: rust-sdk-surface-parity
created: '2026-05-17T00:45:00Z'
updated: '2026-05-19T03:51:15Z'
status: completed
harden_status: not_run
size: large
risk_level: medium
---

# Rust SDK surface parity

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T03:51:15Z
Review gate: pass

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
- `crates/runx-sdk/src/act/assignment.rs`
- `crates/runx-sdk/tests/client_cli.rs`
- `crates/runx-sdk/tests/host_protocol.rs`
- `crates/runx-sdk/tests/act/assignment.rs`
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

Validation:
- [x] `v1` command - Rust SDK tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-sdk`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-41
- [x] `v2` command - Rust formatting and clippy pass.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-sdk --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-42
- [x] `v3` command - Rust style guard passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-43
- [x] `v4` command - SDK builds without package-publication assumptions.
  - Command: `cargo check --manifest-path crates/Cargo.toml -p runx-sdk`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-44
- [x] `v5` command - v0 dependency boundary is clean.
  - Command: `rg -n 'runx-contracts' crates/runx-sdk/Cargo.toml && ! rg -n 'runx-core|sha2|tokio|reqwest|hyper|rmcp|clap' crates/runx-sdk/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-45
- [x] `v6` command - docs do not claim native runtime support.
  - Command: `! rg -n 'native runtime|executes skills directly|without the runx CLI' crates/runx-sdk/README.md docs/api-surface.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-46
- [x] `v7` command - cargo metadata confirms SDK depends on contracts only
  - Command: `cargo metadata --manifest-path crates/Cargo.toml --format-version 1 --no-deps | jq -e '.packages[] | select(.name == "runx-sdk") | .dependencies | (map(.name) | index("runx-contracts") != null and index("runx-core") == null)'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-47

## Phase 1: SDK crate upgrade and style guard

Status: completed
Dependencies: `rust-contracts-parity` Phase 1

Objective: Complete this phase.

Changes:
- `crates/Cargo.toml` (partial, shared) - Verify `runx-sdk` remains a workspace member.
- `crates/runx-sdk/Cargo.toml` (partial, exclusive) - Keep package metadata, edition 2024, workspace lints, library target, and depend on `runx-contracts` plus `serde_json` only as needed for CLI decoding.
- `crates/runx-sdk/README.md` (partial, exclusive) - Expand CLI-backed v0 docs.
- `crates/runx-sdk/src/lib.rs` (partial, exclusive) - Replace placeholder constants with declared modules and explicit re-exports. No wildcard re-exports.
- `scripts/check-rust-core-style.mjs` (partial, shared) - Verify `crates/runx-sdk/src` remains in the scan roots.

Acceptance:
- [x] `ac1_1` command - workspace metadata sees runx-sdk.
  - Command: `cargo metadata --manifest-path crates/Cargo.toml --format-version 1 --no-deps | jq -e '.packages[] | select(.name == "runx-sdk")'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac1_2` command - style guard passes on skeleton.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Blocking CLI client

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `crates/runx-sdk/src/client.rs` (all, exclusive) - `RunxClient`, `RunxClientOptions`, typed methods `search_skills`, `run_skill`, `resume_run`, `connect_list`.
- `crates/runx-sdk/src/command.rs` (all, exclusive) - Command planning and subprocess invocation. No shell interpolation.
- `crates/runx-sdk/src/error.rs` (all, exclusive) - Concrete `RunxError` enum with command status, stderr, JSON parse, and contract-shape variants.
- `crates/runx-sdk/tests/client_cli.rs` (all, exclusive) - Fake-runx tests matching the Python SDK behavior.

Acceptance:
- [x] `ac2_1` command - CLI client tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-sdk --test client_cli`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17
- [x] `ac2_2` command - no shell invocation.
  - Command: `! rg -n 'sh -c|cmd /C|Command::new\\(\"sh\"\\)|Command::new\\(\"cmd\"\\)' crates/runx-sdk/src crates/runx-sdk/tests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18

## Phase 3: Capability execution SDK wrappers

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `crates/runx-sdk/src/act/assignment.rs` (all, exclusive) - Typed SDK constructors and re-exports around `runx-contracts` act assignment types. No independent hash implementation.
- `fixtures/sdk-rust/act-assignment/*.json` (all, exclusive) - Golden cases generated from TypeScript and shared with `runx-contracts`.
- `crates/runx-sdk/tests/act/assignment.rs` (all, exclusive) - Fixture tests proving SDK wrappers produce the same contract values and hashes.

Acceptance:
- [x] `ac3_1` command - act assignment tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-sdk --test act`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `ac3_2` command - fixture JSON is deterministic.
  - Command: `node scripts/check-contract-fixture-key-order.ts fixtures/sdk-rust/act-assignment`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24
- [x] `ac3_3` command - SDK does not implement hashing independently.
  - Command: `! rg -n 'sha2|Sha256|Digest' crates/runx-sdk/src crates/runx-sdk/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-25

## Phase 4: Host protocol SDK wrappers

Status: completed
Dependencies: Phase 3

Objective: Complete this phase.

Changes:
- `crates/runx-sdk/src/host.rs` (all, exclusive) - Host result/state enums, bridge traits or callback structs, normalization from CLI/runtime JSON, and provider-neutral result summaries, all backed by `runx-contracts` types.
- `fixtures/sdk-rust/host-protocol/*.json` (all, exclusive) - Paused, completed, failed, escalated, denied, and terminal state fixtures.
- `crates/runx-sdk/tests/host_protocol.rs` (all, exclusive) - Fixture tests.

Acceptance:
- [x] `ac4_1` command - host protocol tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-sdk --test host_protocol`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30
- [x] `ac4_2` command - no provider-specific adapters in base SDK.
  - Command: `! rg -n 'OpenAI|Anthropic|LangChain|CrewAI|Vercel' crates/runx-sdk/src crates/runx-sdk/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-31

## Phase 5: Docs and workspace check

Status: completed
Dependencies: Phase 4

Objective: Complete this phase.

Changes:
- `crates/runx-sdk/README.md` (partial, exclusive) - Usage examples, CLI requirement, error handling, and native-runtime deferral.
- `docs/api-surface.md` (partial, shared) - Add Rust SDK surface entry.
- `docs/rust-kernel-architecture.md` (partial, shared) - Keep crate graph and early CLI-backed SDK note aligned with implementation.

Acceptance:
- [x] `ac5_1` command - SDK workspace checks pass.
  - Command: `cargo check --manifest-path crates/Cargo.toml -p runx-sdk && cargo test --manifest-path crates/Cargo.toml -p runx-sdk`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `ac5_2` command - docs state CLI-backed v0.
  - Command: `rg -n 'CLI-backed|installed runx|runx --json|native-runtime' crates/runx-sdk/README.md docs/api-surface.md docs/rust-kernel-architecture.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `ac5_3` command - full Rust workspace tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml --workspace`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38

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

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: runx-sdk delivers a working CLI-backed library and every recorded acceptance criterion passes, so completion is not blocked. Four non-blocking scope/parity gaps are worth tracking before publish: (1) `RunSkillOptions::default()` has `non_interactive: false`, diverging from the Python SDK baseline default of `True` (task contract explicitly calls out parity); (2) `fixtures/sdk-rust/host-protocol/` ships only one paused-state and one completed-result fixture even though the spec enumerates paused, completed, failed, escalated, denied, and terminal-state fixtures; (3) `crates/runx-sdk/src/host.rs` ships only enum re-exports and bare `serde_json::from_str` decoders, missing the "bridge traits or callback structs" and "normalization from CLI/runtime JSON" that the spec scope mandates and that the Python SDK provides; (4) the Rust SDK section was hand-added into `docs/api-surface.md`, which is explicitly marked "Generated by scripts/gen-api-index.ts. Do not edit by hand." and will be wiped on the next `pnpm docs:api`, regressing the v5_2 acceptance grep.

Attack log:
- `crates/runx-sdk/src/client.rs RunSkillOptions::default`: Compare Rust default flags against Python SDK baseline (run_skill non_interactive default) -> finding (Python defaults non_interactive=True; Rust derives Default which yields false.)
- `fixtures/sdk-rust/host-protocol/*.json`: Enumerate fixture files against spec scope list (paused/completed/failed/escalated/denied/terminal) -> finding (Only paused-state and completed-result present; 4 result variants and 4 terminal-state variants missing.)
- `crates/runx-sdk/src/host.rs`: Diff Rust host module surface against spec scope (bridge traits/callback structs, normalization from CLI/runtime JSON) -> finding (Bridge and normalization absent; decoders are bare serde_json::from_str.)
- `docs/api-surface.md`: Cross-check hand edits against the file's auto-generation contract -> finding (Line 3 declares the file generated; scripts/gen-api-index.ts has no Rust branch and will overwrite the hand-added section.)
- `crates/runx-sdk/src/command.rs run_command, client.rs run_skill`: Command injection via shell metacharacters in skill_ref, run_id, input keys/values -> clean (Command::new + args (no sh -c); each arg passed as separate argv element. ac2_2 grep also clean.)
- `crates/runx-sdk/src/* clippy/style`: Check for unwrap/expect/panic/dbg/HashMap/anyhow/Box<dyn Error> hits in style guard -> clean (grep for unwrap|expect under crates/runx-sdk/src returned no matches; workspace clippy lints (unwrap_used=deny, expect_used=deny) ratified by ac v2.)
- `crates/runx-sdk/src/client.rs connection_from_json`: ConnectionSummary id resolution preference (connection_id > grant_id > id) divergence vs Python raw payload -> clean (Opinionated normalization; matches included fixture. Python returns raw dict so the new shape is additive, not breaking.)
- `crates/runx-sdk/tests/host_protocol.rs`: Look for tautological roundtrip masking fixture-shape regressions -> clean (Fixture is parsed with #[derive(Deserialize)] before re-encoding, so the original JSON shape IS validated against the contract enum.)

Findings:
- [medium/non-blocking] `F1-run-skill-default-non-interactive-parity` RunSkillOptions::default() sets non_interactive=false, diverging from Python SDK default of True.
  - Location: `crates/runx-sdk/src/client.rs:31`
  - Evidence: client.rs:31-36 derives Default for RunSkillOptions with `non_interactive: bool`, which defaults to false. In packages/sdk-python/runx/__init__.py:101 the Python `run_skill(..., non_interactive: bool = True)` defaults to True, and the Python test at packages/sdk-python/tests/test_runx.py:79 asserts the fake CLI receives `--non-interactive` even though the caller never sets it.
  - Impact: Callers porting from Python see a behavior change: a Rust caller using `RunSkillOptions::default()` (e.g., `RunxClient::new().run_skill("x", RunSkillOptions::default())`) now leaves `--non-interactive` off, so the CLI may prompt or take a different branch. Task contract states the v0 SDK must mirror the Python SDK shape and baseline.
  - Validation: After fix, `cargo test -p runx-sdk --test client_cli` should still pass; add an assertion that a default `RunSkillOptions` produces `--non-interactive` in the args.
- [medium/non-blocking] `F2-host-protocol-fixtures-incomplete` Only paused-state and completed-result fixtures exist; spec scope enumerates paused, completed, failed, escalated, denied, and terminal state.
  - Location: `fixtures/sdk-rust/host-protocol`
  - Evidence: `fixtures/sdk-rust/host-protocol/` contains exactly two files: inspect-host-state-paused.json and result-host-run-completed.json. `crates/runx-sdk/tests/host_protocol.rs` only `include_str!`s those two. The task scope (Task Scope > Host protocol SDK wrappers changes) states the directory should hold "Paused, completed, failed, escalated, denied, and terminal state fixtures."
  - Impact: Failed/Escalated/Denied HostRunResult variants and the Completed/Failed/Escalated/Denied HostTerminalState variants are unexercised. Regression in any of those JSON shapes would slip through the SDK fixture tests even though the host enum declares them.
  - Validation: After adding fixtures, `cargo test -p runx-sdk --test host_protocol` should cover all five HostRunResult variants and all four terminal HostRunState variants.
- [medium/non-blocking] `F3-host-module-missing-bridge-and-normalization` host.rs lacks the bridge traits/callback structs and CLI/runtime normalization the spec scope requires.
  - Location: `crates/runx-sdk/src/host.rs:1`
  - Evidence: `crates/runx-sdk/src/host.rs` re-exports contract enums and exposes only `host_result_status`, `decode_host_result`, and `decode_host_state`. The decoders are thin wrappers around `serde_json::from_str` with no normalization. The Python SDK (packages/sdk-python/runx/host_protocol.py via packages/sdk-python/runx/__init__.py:144-145) ships `normalize_host_result` and `normalize_host_state` that map non-canonical CLI shapes (e.g., `policy_denied`→`denied`, `success`→`completed`) and a `create_host_bridge` that drives the paused→resume loop. The task scope (Host protocol SDK wrappers changes) explicitly lists "bridge traits or callback structs, normalization from CLI/runtime JSON, and provider-neutral result summaries".
  - Impact: If runx CLI ever emits a non-canonical status string the Python SDK accepts (e.g., `policy_denied`), the Rust decoder will return `RunxError::Json` with no graceful path. There is also no Rust analogue to `HostBridge`, so callers cannot reuse the Python-side resolver pattern. The scope item is unmet.
  - Validation: After fix, host.rs should expose a public bridge type and at least one `normalize_*` entry point. Fixture tests should decode a CLI payload using a non-canonical status string and assert it maps to the canonical `HostRunResult` variant.
- [medium/non-blocking] `F4-api-surface-md-rust-section-will-be-regenerated-away` Rust SDK section hand-added to an auto-generated file; next `pnpm docs:api` overwrites it and invalidates acceptance evidence v5_2.
  - Location: `docs/api-surface.md:8`
  - Evidence: `docs/api-surface.md:3` declares "Generated by scripts/gen-api-index.ts. Do not edit by hand." Lines 8-16 are a hand-authored `## Rust SDK` section. `scripts/gen-api-index.ts:46-54` rebuilds the file from a fixed header plus `packages/@runxhq/*` manifests only (no Rust-aware branch), so re-running `pnpm docs:api` will discard the Rust SDK prose. Acceptance evidence v5_2 (`rg -n 'CLI-backed|installed runx|runx --json|native-runtime' ... docs/api-surface.md ...`) silently depends on this prose persisting.
  - Impact: First contributor who runs `pnpm docs:api` (the workflow line 6 of the same file documents) deletes the Rust SDK entry, regressing API discoverability and breaking the acceptance grep guard.
  - Validation: After fix, `pnpm docs:api` must leave the Rust SDK entry in place, and v5_2's grep must continue to match.

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
