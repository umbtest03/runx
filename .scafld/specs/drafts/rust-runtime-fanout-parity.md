---
spec_version: '2.0'
task_id: rust-runtime-fanout-parity
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust runtime fanout parity

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Fanout semantics are
the most error-prone part of the runner; needs its own parity slice.
Blockers: `rust-runtime-skeleton` complete.
Allowed follow-up command: `scafld harden rust-runtime-fanout-parity`
Latest runner update: none
Review gate: not_started

## Summary

Prove Rust runtime fanout behavior matches TypeScript fanout behavior
end-to-end using `oss/fixtures/graphs/fanout/graph.yaml` and a generated
suite of generated fanout scenarios. Sync points, fanout gates, partial
failure, retry, and sync-decision propagation all match TS via fixture
oracle plus differential proptests.

`runx-core::state_machine` already encodes the pure fanout decisions. This
spec proves the runtime applies them faithfully.

## Context

CWD: `.`

Packages:
- `@runxhq/runtime-local`
- `@runxhq/core` (state-machine)
- `crates/runx-runtime`
- `crates/runx-core`

Current TypeScript sources:
- `packages/runtime-local/src/runner-local/fanout.ts`
- `packages/runtime-local/src/runner-local/graph-fanout-gates.ts`
- `packages/runtime-local/src/runner-local/graph-ledger.ts`
- `packages/runtime-local/src/runner-local/execution-targets.ts`

Files impacted:
- `crates/runx-runtime/src/fanout.rs`
- `crates/runx-runtime/tests/fanout_parity.rs`
- `crates/runx-runtime/tests/fanout_proptest.rs`
- `fixtures/runtime/fanout/**`
- `scripts/generate-rust-fanout-fixtures.ts`

Invariants:
- TS fanout remains authoritative.
- Fanout sync points use the same decision enum across languages
  (`runx-core::state_machine::FanoutSyncDecision`).
- Partial failure receipts emit identical structure across languages.
- Proptest counterexamples that fail become pinned fixtures.

## Objectives

- Run `fixtures/graphs/fanout/graph.yaml` end to end on Rust runtime with
  green receipt parity.
- Add a generator that emits N-branch fanout fixtures with deterministic
  inputs across success, partial-failure, and retry scenarios.
- Add a differential proptest harness reusing `state-machine-parity`
  patterns.

## Scope

In scope:
- Fanout sync, fanout gates, fanout receipts.
- Differential proptest harness for fanout transitions.

Out of scope:
- Adapter-specific fanout cases (covered by adapter specs).
- Cloud-side durable fanout (agent-runner durable-step) which has its own
  TS implementation and stays authoritative.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-state-machine-parity` proptest harness reused.

## Open Questions

- Whether the proptest harness lives in `runx-runtime` tests or a new
  `crates/runx-parity-tools` workspace member. Defer to Phase 1.
