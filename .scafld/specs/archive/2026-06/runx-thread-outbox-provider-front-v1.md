---
spec_version: '2.0'
task_id: runx-thread-outbox-provider-front-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-05T04:15:27Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# runx-thread-outbox-provider-front-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: phase3 completed; issue-to-pr provider push now routes through the Rust thread-outbox-provider front, the obsolete TS catalog tool is deleted, and the final provider-state publisher seals through the same front.
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-05T04:15:27Z
Review gate: not_required

## Summary

Migrate the provider-mutation boundary from the former TS outbox-push catalog
tool into the kernel's Rust thread-outbox-provider front, and land
governed-execution-layer item 15 (the final provider-state publisher on that
front). The Rust supervisor and `thread-outbox-provider-protocol-v1` contract are
now dispatched as a first-class graph-step source. `issue-to-pr` publishes
through a Rust-front subskill, the obsolete TS mutation package is deleted, and
the post-merge/final-outcome publisher seals through the same front.

## Objectives

- Prove the Rust `ThreadOutboxProviderProcessSupervisor` can supervise the same
  provider-push/readback frames the TS outbox packaging emits, without mutating
  GitHub or cutting over live `issue-to-pr`.
- Add a non-default graph/source dispatch route for the Rust front, fixture-only
  first, with sealed receipts for push and fetch/readback.
- Route `issue-to-pr` provider push through the Rust front after parity holds.
- Land the item-15 final provider-state publisher on the same front after the
  push route is proven.
- Provider tokens delivered via Rust-supervised `CredentialDelivery`.

## Scope

In scope:
- Phase 1: fixture/local Rust-front parity using the existing contract fixtures,
  mock provider, and current TS outbox packaging shape.
- Phase 2: a non-default graph/source dispatch route for `thread-outbox-provider`
  against the fixture provider, sealed and receipt-verified.
- Phase 3: route `issue-to-pr` push through the Rust front after Phase 1/2; then
  add the post-merge publisher on the same front.

Out of scope:
- New providers beyond the GitHub thread/outbox lane.
- A new catch-all plugin surface. This front is only for provider-side
  publication/readback of thread outbox entries.

## Dependencies

- Built-but-inert Rust supervisor: `crates/runx-runtime/src/outbox_provider.rs`
  (`ThreadOutboxProviderProcessSupervisor::invoke_push` / `invoke_fetch`).
- Contract frames: `crates/runx-contracts/src/thread_outbox_provider.rs`.
- Existing tests: `crates/runx-runtime/tests/thread_outbox_provider.rs`.
- Current live graph path: `skills/issue-to-pr/X.yaml` calls
  `./push-outbox` for `push-pull-request` and `push-feed-entry`.
- GitHub provider process path:
  `tools/thread/thread_outbox_provider/github-provider.mjs` adapts Rust-framed
  push requests to `tools/thread/github_adapter.mjs`.

## Assumptions

- The protocol + supervisor are the right contract; this is dispatch wiring and
  migration, not a protocol rebuild.
- The live `issue-to-pr` path must not change until a fixture-only Rust-front
  parity slice proves the same push/readback semantics.
- `SourceKind` and graph-step routing should not grow unless Phase 1 shows the
  dedicated front is ready to carry provider mutation.

## Touchpoints

- `crates/runx-runtime/src/outbox_provider.rs`
- `crates/runx-runtime/tests/thread_outbox_provider.rs`
- `fixtures/contracts/thread-outbox-provider/*.json`
- `fixtures/runtime/thread-outbox-provider/mock-provider.sh`
- SourceKind/graph-step dispatch (`crates/runx-parser/src/skill/source.rs`,
  `crates/runx-parser/src/skill/types.rs`,
  `crates/runx-runtime/src/execution/skill_run.rs`,
  `crates/runx-runtime/src/execution/runner/steps.rs`)
- `skills/issue-to-pr/X.yaml` provider push steps
- former TS outbox-push catalog tool
- `tools/thread/github_adapter.mjs`
- Post-merge publisher

## Risks

- **Skill-safety (highest).** `issue-to-pr` must keep working through the
  migration. Mitigation: Phase 1/2 are additive and fixture-only; do not cut the
  TS path until parity holds; keep the contract surface frozen.
- **Duplicate mutation path.** Running both Rust and TS provider pushes against
  live GitHub could double-post comments or PR updates. Mitigation: live cutover
  must be single-owner; Phase 1/2 use fixture providers only.
- **Credential leakage.** Provider observations must not expose raw token
  material. Mitigation: keep `CredentialDelivery` structured refs and secret-field
  rejection mandatory in all new dispatch tests.

## Acceptance

Profile: strict

Validation:
- Phase 1 proves the Rust supervisor can push/fetch the existing provider fixture
  frames, including idempotency, provider locator, readback summary, credential
  delivery observation, redaction, and secret-field rejection.
- Phase 2 proves the front can be dispatched from a graph/source route without
  changing `issue-to-pr`.
- Phase 3 proves `issue-to-pr` push and post-merge publishing seal through the
  Rust front with no duplicate TS provider mutation and no SourceKind/protocol
  surface removed.

## Phase 1: Rust-front parity, fixture-only

Status: completed
Dependencies: the inert supervisor, the protocol contract

Objective: prove the existing Rust supervisor can carry provider push/readback

Changes:
- Keep `skills/issue-to-pr/X.yaml` unchanged.
- Add or tighten Rust fixture parity around `ThreadOutboxProviderProcessSupervisor::invoke_push` / `invoke_fetch`.
- Feed the supervisor frames equivalent to the current TS outbox packaging contract and assert operation, request id, idempotency, provider locator, provider event hash, readback summary, delivery observations, and redaction.

Acceptance:
- [x] `ac1` command - Rust provider fixture parity holds
  - Command: `cargo nextest run --manifest-path crates/Cargo.toml -p runx-runtime --all-features thread_outbox_provider`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac2` command - live issue-to-pr route is not cut over in Phase 1
  - Command: historical route-presence check before Phase 3
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Non-default dispatch route, fixture-only

Status: completed
Dependencies: Phase 1

Objective: dispatch a thread-outbox-provider graph/source route through the Rust

Changes:
- Add the smallest parser/runtime dispatch surface for `thread-outbox-provider`, fixture-only first.
- Seal push/fetch observations as receipts and preserve `CredentialDelivery` behavior.

Acceptance:
- [x] `ac3` command - fixture graph dispatch seals provider push/readback
  - Command: `runx harness examples/thread-outbox-provider-graph --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: `RUNX_RECEIPT_DIR=$(mktemp -d) ... cargo run --manifest-path crates/Cargo.toml -p runx-cli -- harness examples/thread-outbox-provider-graph --json` passed with status `passed`, case `push-and-fetch`, receipt `sha256:914f4bcf7382359175cf481006ed725ebe976a5978df9bfe9b99aac3ccf266c3`.
- [x] `ac4` command - protocol surface remains frozen
  - Command: `pnpm vitest run packages/contracts/src/schemas/thread-outbox-provider.test.ts packages/contracts/src/index.test.ts && cargo nextest run --manifest-path crates/Cargo.toml -p runx-contracts --all-features thread_outbox_provider`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: TS command passed 2 files / 27 tests. Local macOS Rust test binaries blocked in dyld before Rust code; equivalent Linux Docker execution passed `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features thread_outbox_provider` with 5 thread-outbox-provider integration tests passing. Runtime front coverage also passed in Linux Docker: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features thread-outbox-provider --test integration thread_outbox_provider -- --nocapture` with 8 tests passing.

## Phase 3: issue-to-pr cutover + post-merge publisher

Status: completed
Dependencies: Phase 2

Objective: move the live provider push and post-merge publisher onto the Rust
front with no duplicate mutation path.

Changes:
- Route `issue-to-pr` push through the Rust front via `skills/issue-to-pr/push-outbox`.
- Delete the TS outbox-push catalog tool after the Rust route is authoritative.
- Implement the final provider-state publisher on the same front.

Acceptance:
- [x] `ac5` command - issue-to-pr push routes via Rust front and graph contract still works
  - Command: parser contract plus active-surface grep for removed outbox-push identifiers
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: parser contract passed; active-surface grep has no matches after deleting the former TS outbox-push catalog tool.
- [x] `ac6` command - post-merge publish seals
  - Command: `runx harness examples/post-merge-publish/final-outcome.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: `RUNX_RECEIPT_DIR=$(mktemp -d) ... crates/target/debug/runx harness examples/post-merge-publish/final-outcome.yaml --json` sealed a graph receipt with `reason_code: graph_closed`.

## Rollback

- Phase 1 is test/spec-only and reverts cleanly.
- Phase 2 is additive fixture dispatch; remove the route/example if it fails.
- Phase 3 is a single-owner cutover: revert `skills/issue-to-pr/push-outbox`,
  `tools/thread/thread_outbox_provider/github-provider.mjs`, and the runtime
  dynamic-envelope changes together if the Rust front regresses. No contract
  churn.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- none
