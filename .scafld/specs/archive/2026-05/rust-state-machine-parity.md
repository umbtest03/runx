---
spec_version: '2.0'
task_id: rust-state-machine-parity
created: '2026-05-15T12:51:06Z'
updated: '2026-05-17T16:45:22Z'
status: completed
harden_status: passed
size: large
risk_level: medium
---

# Rust state-machine parity

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-17T16:45:22Z
Review gate: pass

## Summary

Upgrade the `runx-core` placeholder into the first real Rust kernel crate and
port the pure state-machine planning behavior to Rust behind fixture parity.
The TypeScript implementation remains authoritative. Rust must pass the shared
fixture suite; it must not own runtime behavior, subprocesses, filesystem
reads, network, MCP, or CLI presentation.

This spec depends on the architecture decisions in
`oss/docs/rust-kernel-architecture.md`. In particular, it inherits:
- the target crate graph (section 3),
- the `runx-core` public API stance (section 4),
- the decision model: enums, not `Result<_, _>` (section 5),
- serde conventions: camelCase fields, kebab-case payload-free variants,
  tagged unions on discriminator (section 6),
- standard library posture: `std` by default, no `no_std` gate (section 8),
- MSRV 1.85.0, edition 2024, resolver 3 (section 9),
- Rust-side boundary enforcement via `cargo-deny` and public-API snapshots
  (section 10),
- differential testing strategy with `proptest` (section 11),
- Rust implementation quality bar (section 18).

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-core`
- `crates/runx-cli`

Files impacted:
- `crates/Cargo.toml`
- `crates/runx-contracts/Cargo.toml`
- `crates/runx-contracts/src/lib.rs`
- `crates/runx-core/Cargo.toml`
- `crates/runx-core/src/lib.rs`
- `crates/runx-core/src/state_machine.rs`
- `crates/runx-core/src/state_machine/single_step.rs`
- `crates/runx-core/src/state_machine/sequential_graph.rs`
- `crates/runx-core/src/state_machine/fanout.rs`
- `crates/runx-core/tests/state_machine_fixtures.rs`
- `crates/runx-core/tests/state_machine_proptest.rs`
- `crates/runx-core/src/serde_conventions.rs`
- `crates/deny.toml`
- `crates/runx-core/rust-toolchain.toml`
- `scripts/check-rust-core-style.mjs`
- `fixtures/kernel/state-machine/*.json`
- `packages/core/src/state-machine/index.ts`
- `packages/core/src/state-machine/index.test.ts`
- `docs/rust-kernel-architecture.md`

Invariants:
- TypeScript state-machine remains the source of truth.
- Rust `runx-core` state-machine code is deterministic and side-effect free.
- Rust state-machine code must not import filesystem, subprocess, network, async
  runtimes, or runtime-local behavior.
- Rust public names match the existing runx concepts. `createSequentialGraphState`
  becomes `create_sequential_graph_state`. No invented aliases.
- `runx-core` uses `std` by default; no `no_std` gate in this phase.
- All public types derive `serde::Serialize` and `serde::Deserialize`.
- Decision outcomes are tagged enums, not `Result<_, _>`.
- Rust code follows the architecture doc section 18 quality bar: no mechanical
  TypeScript-shaped Rust, no public `serde_json::Value`, no `HashMap` at a
  serialized boundary, no wildcard re-exports, no macro-generated model code,
  and no dynamic error erasure.

Related docs:
- `docs/rust-kernel-architecture.md` (prerequisite reading)
- `docs/trusted-kernel-package-truth.md`
- `plans/runx.md`
- `fixtures/kernel/README.md`

## Objectives

- Upgrade the existing `crates/runx-core` placeholder into a library crate
  with real parity behavior.
- Port state-machine types and pure transition planning to Rust.
- Test Rust against the shared state-machine fixture set.
- Keep TypeScript tests and Rust tests running side by side.
- Document known intentional gaps if any fixture is deferred.

## Scope

In scope (all seven direct exports from `@runxhq/core/state-machine`):
- `createSingleStepState` -> `create_single_step_state`
- `transitionSingleStep` -> `transition_single_step`
- `createSequentialGraphState` -> `create_sequential_graph_state`
- `planSequentialGraphTransition` -> `plan_sequential_graph_transition`
- `transitionSequentialGraph` -> `transition_sequential_graph`
- `evaluateFanoutSync` -> `evaluate_fanout_sync`
- `fanoutSyncDecisionKey` -> `fanout_sync_decision_key`
- direct supporting value types: `StepStatus`, `GraphStatus`,
  `GraphStepStatus`, `FanoutSyncStrategy`, `FanoutBranchFailurePolicy`,
  `FanoutGateAction`, `SingleStepState`, `SequentialGraphStepDefinition`,
  `FanoutThresholdGate`, `FanoutConflictGate`, `FanoutGroupPolicy`,
  `FanoutBranchResult`, `FanoutSyncDecision`, `SequentialGraphStepState`,
  `SequentialGraphState`, `SequentialGraphEvent`, `SequentialGraphPlan`,
  `SingleStepEvent`.
- a minimal `runx_contracts::JsonValue` and `JsonObject` boundary type for
  arbitrary fixture-compatible JSON payloads in `outputs`, fanout gate
  `value`, and fanout conflict `values`. This avoids public
  `serde_json::Value` while keeping the JSON contract typed and shared.
- `cargo-deny` configuration that forbids runtime crates as transitive
  dependencies of `runx-core`.
- `proptest` strategies for state-machine inputs.

Out of scope:
- Parser, YAML, graph hydration, runtime-local orchestration, receipts, policy,
  MCP, adapters, CLI command parsing, and hosted runtime.
- Replacing TypeScript callers with Rust bindings.
- Publishing the Rust core crate.
- Policy parity (separate spec).
- Differential testing across the language boundary; we adopt `proptest`
  on the Rust side only in this spec. Cross-language proptest is a future
  spec if the value justifies the harness cost.

## Dependencies

- `rust-contracts-bootstrap` completed and approved.
- `rust-kernel-parity-fixtures` completed and approved.
- Rust toolchain available in CI.
- Existing Rust workspace under `crates/`.

## Assumptions

- Rust 2024, resolver 3, MSRV 1.85.0. Pinned in `crates/Cargo.toml`
  `[workspace.package]` and optionally re-pinned by `rust-toolchain.toml`
  in `crates/runx-core/`.
- `serde` and `serde_json` are acceptable dependencies with default features.
  Future `no_std` consumers, if any, get their own follow-up spec.
- `runx-contracts` owns arbitrary JSON boundary values needed by multiple
  future crates. `runx-core` must use `runx_contracts::JsonValue` for public
  payload fields instead of defining a local duplicate or exposing
  `serde_json::Value`.
- `cargo-deny` may not be installed in a fresh local environment. The
  implementation should document `cargo install cargo-deny --locked` as the
  recovery command instead of weakening the acceptance gate.
- `proptest` is a dev-dependency only; it never appears in `runx-core`
  production deps.
- The Rust implementation uses idiomatic enums/structs while preserving JSON
  compatibility at the fixture boundary via the serde conventions in the
  arch doc.
- TS-side test reorganization is not in scope. Existing
  `index.test.ts` continues to test TS behavior directly; fixture tests live
  in `tests/kernel-parity-fixtures.test.ts` from the fixtures spec.

## Touchpoints

- Rust workspace membership.
- Cargo lockfile policy, if generated.
- Fixture runner and TypeScript fixture generator.
- State-machine tests in both languages.

## Risks

- Medium: fixture shape may expose TypeScript implementation detail rather than
  stable behavior. Mitigated by the Rust-shaped sanity check in the fixtures
  spec.
- Medium: Rust names can drift from TS names and make parity hard to review.
  Mitigated by the naming invariant and a one-time review pass.
- Medium: `proptest` shrinking can be slow on the fanout state space if
  strategies are too broad. Bound generators carefully and set a CI time cap.
- Low: adding Rust dependencies can create supply-chain overhead. `cargo-deny`
  enforces the allowlist.

## Acceptance

Profile: strict

Validation:
- [x] `v1` command - Rust state-machine tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-43
- [x] `v2` command - Rust formatting and clippy pass.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-core --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-44
- [x] `v3` test - TypeScript state-machine tests pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/core/src/state-machine/index.test.ts tests/kernel-parity-fixtures.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-45
- [x] `v4` command - runx-core state-machine code does not use runtime APIs.
  - Command: `! rg -n 'std::fs|std::process|std::net|std::env|std::time::SystemTime|tokio|reqwest|ureq|hyper|rmcp|Command::new' crates/runx-core/src crates/runx-core/tests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-46
- [x] `v5` command - cargo-deny is clean.
  - Command: `cargo deny --manifest-path crates/Cargo.toml check bans licenses sources`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-47
- [x] `v6` command - proptest cases run within the cap.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core --test state_machine_proptest`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-48
- [x] `v7` command - Rust graph and style guards pass.
  - Command: `node scripts/check-rust-crate-graph.mjs && node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-49

## Phase 1: Core crate upgrade

Status: completed
Dependencies: rust-kernel-parity-fixtures

Objective: Complete this phase.

Changes:
- `crates/Cargo.toml` (partial, shared) - Verify `runx-core` remains a workspace member. Workspace `[workspace.package]` MSRV `1.85.0` is already pinned.
- `crates/runx-contracts/Cargo.toml` (partial, exclusive) - Add `serde` as a dependency if not already present.
- `crates/runx-contracts/src/lib.rs` (partial, exclusive) - Replace the placeholder-only surface with a small typed JSON boundary model: `JsonValue`, `JsonObject = BTreeMap<String, JsonValue>`, and round-trip tests. No host-protocol or receipt contracts in this phase.
- `crates/runx-core/Cargo.toml` (partial, exclusive) - Keep package metadata, edition 2024, lints (deny `unsafe`, `clippy::panic`, `clippy::unwrap_used`, `clippy::expect_used`, `clippy::todo`, `clippy::unimplemented`, `clippy::dbg_macro`, `clippy::wildcard_imports`, `clippy::print_stdout`, `clippy::print_stderr`), `default = ["std"]` feature, `serde`/`serde_json` with default features, `proptest` as dev-dependency.
- `crates/runx-core/rust-toolchain.toml` (all, exclusive) - Pin toolchain to the workspace MSRV.
- `crates/runx-core/src/lib.rs` (partial, exclusive) - Replace placeholder constants with `pub mod state_machine;` and `pub mod serde_conventions;`. No `#![no_std]`.
- `crates/runx-core/src/serde_conventions.rs` (all, exclusive) - Comment-only module documenting the rename/tagging rules from arch doc section 5, plus a round-trip test of a couple of golden values.
- `crates/deny.toml` (all, exclusive) - cargo-deny bans entry for runtime crates as transitive deps of `runx-core`.
- `scripts/check-rust-core-style.mjs` (all, exclusive) - Repository-specific Rust style guard for `runx-core`: reject public `serde_json::Value`, reject `HashMap` in `crates/runx-core/src`, reject wildcard re-exports, reject `anyhow`, `eyre`, `Box<dyn Error>`, `macro_rules!`, and `proc_macro`, and warn/fail on files or functions that exceed the section 18 size limits without an explicit `rust-style-allow` comment naming the reason.

Acceptance:
- [x] `ac1_1` command - Rust workspace metadata is valid.
  - Command: `cargo metadata --manifest-path crates/Cargo.toml --format-version 1 --no-deps`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `ac1_2` command - core crate has no runtime dependency names.
  - Command: `! rg -n 'tokio|reqwest|hyper|rmcp|clap|runtime' crates/runx-core/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `ac1_3` command - cargo-deny passes on an empty crate.
  - Command: `cargo deny --manifest-path crates/Cargo.toml check bans`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16
- [x] `ac1_4` command - Rust style guard passes on the upgraded skeleton.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17
- [x] `ac1_5` command - public Rust code does not expose `serde_json::Value`.
  - Command: `! rg -n 'pub .*serde_json::Value|serde_json::Value' crates/runx-contracts/src crates/runx-core/src`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18

## Phase 2: State-machine types and single-step

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `crates/runx-core/src/state_machine.rs` (all, exclusive) - Module root that re-exports submodules.
- `crates/runx-core/src/state_machine/types.rs` (all, exclusive) - All state-machine value types with serde derives per the conventions.
- `crates/runx-core/src/state_machine/single_step.rs` (all, exclusive) - `create_single_step_state` and `transition_single_step`.
- `crates/runx-core/tests/state_machine_fixtures.rs` (all, exclusive) - Fixture loader that dispatches to the function under test based on a fixture `input.kind` field. Single-step fixtures pass; graph and fanout fixtures declared as not-yet-supported and skipped explicitly.

Acceptance:
- [x] `ac2_1` command - single-step fixtures pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core --test state_machine_fixtures single_step`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `ac2_2` command - clippy is clean on the new code.
  - Command: `cargo clippy --manifest-path crates/Cargo.toml -p runx-core --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24

## Phase 3: Sequential graph and fanout sync

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `crates/runx-core/src/state_machine/sequential_graph.rs` (all, exclusive) - `create_sequential_graph_state`, `plan_sequential_graph_transition`, `transition_sequential_graph`.
- `crates/runx-core/src/state_machine/fanout.rs` (all, exclusive) - `evaluate_fanout_sync`, `fanout_sync_decision_key`.
- `crates/runx-core/tests/state_machine_fixtures.rs` (partial, exclusive) - Wire up all remaining fixture categories.

Acceptance:
- [x] `ac3_1` command - all state-machine fixtures pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core --test state_machine_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29
- [x] `ac3_2` command - TypeScript state-machine tests still pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/core/src/state-machine/index.test.ts tests/kernel-parity-fixtures.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30

## Phase 4: Property testing

Status: completed
Dependencies: Phase 3

Objective: Complete this phase.

Changes:
- `crates/runx-core/Cargo.toml` (partial, shared) - Add `proptest` as `[dev-dependencies]`.
- `crates/runx-core/tests/state_machine_proptest.rs` (all, exclusive) - Strategies for `SingleStepState` + `SingleStepEvent` pairs and `SequentialGraphState` + `SequentialGraphEvent` pairs. Assert: transitions are deterministic, single-step terminal states are absorbing, graph status override events are unconditional per the TypeScript contract, `Complete` only succeeds when no graph steps are pending/running, and decision keys are stable under serde round-trip.

Acceptance:
- [x] `ac4_1` command - proptest run completes within the cap.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core --test state_machine_proptest`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-35

## Phase 5: Gap docs

Status: completed
Dependencies: Phase 4

Objective: Complete this phase.

Changes:
- `docs/rust-kernel-architecture.md` (partial, shared) - Update section 14 with state-machine status if any decisions changed during implementation.
- `docs/trusted-kernel-package-truth.md` (partial, shared) - Note that `runx-core` currently provides state-machine parity only and is not yet a replacement for TypeScript core.
- `fixtures/kernel/README.md` (partial, shared) - Add Rust fixture runner notes.

Acceptance:
- [x] `ac5_1` command - docs state that TS remains authoritative.
  - Command: `rg -n 'TypeScript.*source of truth|state-machine parity|runx-core' docs fixtures/kernel crates/runx-core`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-40

## Rollback

Strategy: per_phase

Commands:
- Revert `crates/runx-core` to its placeholder-only surface.
- Revert `crates/runx-contracts` JSON boundary additions only if no later
  crate has started depending on `JsonValue`.
- Keep the `runx-core` workspace member in `crates/Cargo.toml`.
- Remove `crates/deny.toml` if no other crate depends on it.
- Revert docs changes that mention state-machine Rust parity.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Rust state-machine parity port faithfully mirrors the TypeScript oracle in packages/core/src/state-machine/index.ts. Traced all transition functions (single_step, sequential_graph, fanout), serde tagging conventions, fanout gate semantics, retry budget logic, conflict/threshold gate evaluation, and stable-value computation. Boundary is clean: no fs/process/net/tokio imports anywhere under crates/runx-core. Style guard, cargo-deny bans, fixture coverage check, and proptest invariants are all wired correctly. Ambient drift (Cargo.lock, README updates, runx-cli/src/main.rs) is outside this spec's scope. No completion-blocking findings.

Attack log:
- `Acceptance criteria mapping`: Spec compliance: walk every v* and ac* acceptance ID and confirm the implementation/tests that satisfy it exist (fixtures runner, proptest file, style script with fixture coverage, deny.toml bans, no IO imports). -> clean (All exported functions enumerated in scope are re-exported from state_machine.rs; tests reference each fixture; proptest covers determinism/absorbing/override/complete/decision-key invariants per spec.)
- `crates/runx-core/src/state_machine/sequential_graph.rs vs packages/core/src/state-machine/index.ts`: Regression hunt: line-by-line trace of planSequentialGraphTransition, planFanoutGroup, transitionSequentialGraph, including empty fanoutGroup truthiness, collectContiguousFanoutGroup boundary, retry budget, missing-context blocking, and updateStep status overrides. -> clean (Outer-loop index arithmetic (index += groupSteps.length) accounts for TS's implicit +1; defensive 'fanout group is empty' branch preserved; complete sets succeeded regardless of prior graph status, matching TS.)
- `crates/runx-core/src/state_machine/fanout.rs vs evaluateFanoutSync`: Dark patterns: threshold ordering (branch_failure first, threshold next, conflict next, quorum last), resolved-gate-key short-circuit, distinct conflict value computation, rule_fired keys, and stable_value vs JSON.stringify semantics. -> clean (Rule-firing order, gate-key semantics, and reason strings match TS verbatim; conflict_values filter_map mirrors JSON.stringify dropping undefined entries; serde_json::to_string of sorted BTreeMap matches TS stableValue for ASCII keys.)
- `runx_contracts::JsonNumber serialization`: Convention check: floating-point edge cases (NaN/Infinity rejection on deserialize and serialize, whole-f64 emitted as integer to match JSON.stringify(1.0)='1', fractional preservation, i64/u64 fast path). -> clean (Visitor rejects non-finite; serialize_whole_f64 narrows to i64/u64 when in range; round-trip tests for sorted objects, fractional numbers, and NaN rejection pass. Note: 1e21-class values would diverge between TS (1e+21) and Rust serde_json (1e21), but no fixture exercises this range.)
- `scripts/check-rust-core-style.mjs + crates/deny.toml`: Convention check: ensure boundary enforcement is real - banned patterns are detected, fixture coverage check fails closed on mismatches, cargo-deny bans cover all runtime crates listed in the architecture doc. -> clean (Bans list (async-std/axum/clap/hyper/reqwest/rmcp/tokio/ureq), pattern detectors (serde_json::Value, HashMap, anyhow/eyre, Box<dyn Error>, macro_rules!/proc_macro, panic/unwrap/expect/todo, wildcard re-export), and 350-line/60-line size guards all wired with documented `rust-style-allow` escape hatches used only by sequential_graph.rs and fanout.rs.)
- `Workspace classification`: Ambient drift triage: confirm changes to crates/Cargo.lock, crates/README.md, crates/runx-cli/src/main.rs, crates/runx-core/README.md are unrelated to the kernel parity surface and do not regress task scope. -> clean (Cargo.lock churn is the natural product of adding proptest and serde to runx-core/runx-contracts; README edits are documentation; runx-cli main.rs is outside the pure-kernel boundary. None re-enter crates/runx-core/src or alter fixture semantics.)
- `Proptest invariants vs TS contract`: Dark patterns: verify single-step terminal absorbing, deterministic transitions, graph status override unconditional, Complete gated by no-pending-no-running, decision-key roundtrip stability, and threshold compared_to JSON shape ('comparedTo':1 not 1.0). -> clean (All 7 proptest invariants encode TS behavior; the threshold_compared_to_serializes_whole_numbers_like_javascript unit test pins the JSON shape against the JsonNumber whole-float collapse.)

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

Notes:
none

Improvements:
- none

## Deviations

- none

## Metadata

Estimated effort hours: 16
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- trusted-kernel
- state-machine
- parity

## Origin

Source:
- user requested phased scafld plans for Rust kernel parity.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- depends_on: rust-kernel-parity-fixtures

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-17T15:27:59Z
Ended: 2026-05-17T15:31:09Z

Checks:
- path audit
  - Grounded in: code:crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: Phase 1 owns the placeholder-to-library upgrade and now also
- command audit
  - Grounded in: code:crates/Cargo.toml:1
  - Result: passed
  - Evidence: Acceptance commands use the existing Cargo workspace. Missing
- scope/migration audit
  - Grounded in: code:packages/core/src/state-machine/index.ts:140
  - Result: passed
  - Evidence: Scope covers all seven direct state-machine exports and keeps
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: Phase 1 validates workspace skeleton and boundary types before
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback is per phase and preserves the `runx-core` workspace
- design challenge
  - Grounded in: code:packages/core/src/state-machine/index.ts:51
  - Result: passed
  - Evidence: Harden resolved arbitrary JSON payload ownership before Rust

Questions:
- Where should arbitrary JSON payloads live now that state-machine outputs and
  - Grounded in: code:packages/core/src/state-machine/index.ts:51
  - Recommended answer: Add a minimal `runx_contracts::JsonValue` plus
  - If unanswered: Default to `runx_contracts::JsonValue`; do not define a
  - Answered with: Use `runx_contracts::JsonValue` and deterministic
- Should state-machine public types move into `runx-contracts` because the
  - Grounded in: code:crates/runx-contracts/src/lib.rs:1
  - Recommended answer: No. Keep state-machine decision types in
  - If unanswered: Keep state-machine types in `runx-core`.
  - Answered with: State-machine types remain in `runx-core`.
- How should the Rust fixture runner choose which operation to dispatch?
  - Grounded in: code:fixtures/kernel/state-machine/single-step-create-pending.json:7
  - Recommended answer: Dispatch on `input.kind`, matching the fixture schema
  - If unanswered: Use `input.kind`.
  - Answered with: Use `input.kind`.
- What is the human recovery path when `cargo deny` is unavailable locally?
  - Grounded in: spec_gap:acceptance
  - Recommended answer: Keep the `cargo deny` acceptance gate and document
  - If unanswered: Keep the gate and document installation.
  - Answered with: Keep the gate and document installation.
- How broad should proptest be for this first state-machine port?
  - Grounded in: code:fixtures/kernel/state-machine/fanout-plan-conflict-escalates.json:1
  - Recommended answer: Bound generators to small graph/fanout shapes and
  - If unanswered: Bound strategies and test invariants only.
  - Answered with: Bound strategies and test invariants only.


## Planning Log

- 2026-05-15T12:58:00Z: Drafted as second phase of Rust kernel parity.
- 2026-05-15T13:30:00Z: Revised after architectural review. Expanded scope to
  include all seven direct state-machine exports (single-step was missing).
  Added MSRV pin, `cargo-deny` configuration,
  serde conventions module, and a dedicated `proptest` phase. Restructured
  into five phases (skeleton, single-step, graph+fanout, proptest, docs).
  Estimate bumped from 6h to 16h. Now depends on
  `docs/rust-kernel-architecture.md`.
- 2026-05-16T00:00:00Z: Independent review correction. MSRV bumped from
  1.84.0 to 1.85.0 to match actual `crates/Cargo.toml`. `no_std` posture
  dropped; `runx-core` ships with `std` default. Removed
  `--no-default-features` validation/dod and the `no_std` risk entry.
- 2026-05-17T15:40:00Z: Harden round resolved the arbitrary JSON payload
  boundary before implementation. Added `runx_contracts::JsonValue` and
  `JsonObject` to Phase 1, clarified fixture dispatch on `input.kind`, kept
  state-machine types in `runx-core`, and kept `cargo-deny` as a hard gate
  with local installation as the recovery path.
