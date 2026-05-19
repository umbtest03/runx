---
spec_version: '2.0'
task_id: rust-runtime-fanout-parity
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T03:00:01Z'
status: completed
harden_status: not_run
size: medium
risk_level: medium
---

# Rust runtime fanout parity

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T03:00:01Z
Review gate: pass

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

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: All three prior completion blockers are repaired. The generator (`scripts/generate-rust-fanout-fixtures.ts`) is now genuinely parameterized: it accepts `--branches`/`--scenario`, drives the TS `evaluateFanoutSync` oracle, emits N-branch fanout YAMLs for partial-failure and retry (`fixtures/runtime/fanout/generated/*.yaml`), and pins assertions against the TS-derived sync-point. Sync-point receipt metadata is real: `runx-contracts::FanoutReceiptSyncPoint` is now part of `HarnessReceipt.sync_points`, the runner populates `sync_points` via `record_proceeding_fanout_sync_point` / `push_sync_point`, and `fanout_parity.rs` asserts equality of `run.sync_points` and `run.receipt.sync_points` against the TS-derived oracle on All/Quorum/partialFailure cases. The previously-blocking workspace mutation is gone (ambient_drift = none per session). Previous non-blocking gaps are also substantially closed: proptest now covers All/Any/Quorum strategies plus halt-on-failure plus dedicated threshold and conflict gate tests; the runtime-error-aborts-fanout gap is fixed via `runtime_error_step_run` and verified by `fanout-generated-missing-skill.yaml`; a real retry path is exercised end-to-end through checkpoint resume. One remaining minor parity gap is recorded as low/non-blocking: terminal Halt/Pause/Escalate paths error out instead of emitting a graph receipt with sync_points, but the spec invariants scope receipt parity to the partial-failure (proceed-with-failures) case which is covered.

Attack log:
- `scripts/generate-rust-fanout-fixtures.ts`: Re-check prior blocker: confirm generator is now parameterized over N branches and emits retry scenarios using a TS oracle (not literal JSON). -> clean (Accepts --branches (>=2) and --scenario; writes fixtures/runtime/fanout/generated/fanout-generated-{partial-failure,retry}-N.yaml; sync-point expectations are derived from packages/core/src/state-machine evaluateFanoutSync via syncPointFromTs; oracle is checked against requested rule_fired/decision via assertSyncPointField.)
- `crates/runx-runtime/src/receipts.rs + crates/runx-contracts/src/harness.rs`: Re-check prior blocker: confirm fanout sync-point metadata is now present in HarnessReceipt and emitted by the runtime. -> clean (HarnessReceipt.sync_points: Vec<FanoutReceiptSyncPoint> exists (harness.rs:218); graph_receipt accepts sync_points argument; runner builds them via record_proceeding_fanout_sync_point/push_sync_point; FanoutReceiptSyncPoint mirrors TS GraphReceiptSyncPoint shape including gate as Option<JsonObject>.)
- `.scafld session ambient_drift`: Re-check prior critical blocker: confirm workspace was not mutated during this review window. -> clean (ambient_drift section reports 'none'; all observed deltas live inside task scope per Task Changes manifest.)
- `crates/runx-runtime/tests/fanout_parity.rs`: Confirm parity test consumes a TS-derived oracle rather than a hand-written Rust expectation. -> clean (Deserializes fixtures/runtime/fanout/expected.json (produced by the TS-oracle generator) into FanoutReceiptSyncPoint and asserts run.sync_points == expected and run.receipt.sync_points == expected for allSuccess, quorumContinue, and generated partial-failure cases.)
- `crates/runx-runtime/tests/fanout_proptest.rs`: Confirm proptest is no longer limited to Quorum+Continue and now covers strategies and gates. -> clean (Proptest varies strategy_index in 0..3 (All/Any/Quorum) and halt_on_failure flag; dedicated threshold_gate_decision_matches_reference_policy and conflict_gate_decision_matches_reference_policy tests exercise gate paths. State-machine parity vs TS is already locked in by rust-state-machine-parity (archived/completed); this proptest is a sufficient Rust-side pin.)
- `crates/runx-runtime/src/runner.rs run_one_step_with_mode + fanout_runtime_error_branch_records_failure_and_continues test`: Re-check prior medium: confirm a non-skill runtime error inside a fanout branch no longer aborts sibling branches. -> clean (Inside RecordAndContinue mode the run_step error is captured by runtime_error_step_run which synthesizes a failed StepRun and continues; fanout-generated-missing-skill.yaml + test asserts market & risk still succeed, missing records a Failed step, and the sync decision still Proceeds.)
- `Retry coverage`: Re-check prior medium: confirm a real retry-budget exhaustion scenario is exercised. -> clean (fanout-generated-retry-5.yaml has retry.max_attempts: 2 on branch_4; generated_retry_records_attempts_before_halt drives checkpoint resume, asserts attempts==2 in state, and asserts run_graph_file errors with GraphPlanningFailed matching the TS oracle sync_point reason.)
- `crates/runx-runtime/src/runner.rs terminal arms (fail_graph/block_graph/pause_for_sync/escalate_for_sync)`: Dark pattern: do Halt/Pause/Escalate paths still emit a HarnessReceipt with sync_points like TS, or do they short-circuit? -> finding (These arms push the sync_point onto self.sync_points but then return RuntimeError, preventing graph_receipt(...) from running. TS handle-terminal.ts breaks (not throws) so finalize still emits the receipt. Recorded as low/non-blocking parity gap (#fanout-parity-terminal-paths-skip-graph-receipt).)
- `crates/runx-runtime/src/runner.rs push_sync_point dedupe`: Race the dedupe key (group_id, rule_fired, decision) for retry/pause loops to see if a legitimate second sync_point can be dropped. -> clean (Retry loop relies on record_proceeding_fanout_sync_point's RunFanout/same-group skip, then a single terminal push from fail_graph; for pause, the only writer is pause_for_sync. No observed flow produces a duplicate that hides a real second decision.)
- `crates/runx-runtime/src/runner.rs latest_fanout_receipt_ids vs packages/runtime-local/.../graph-governance.ts latestFanoutReceiptIds`: Ordering parity for retry receipt IDs (Rust iterates graph order, TS iterates run order with Map insertion-order dedupe). -> clean (Plan_sequential_graph_transition emits fanout step_ids in graph definition order, so for the supplied fixtures the two orderings agree; expected.json retry branch_receipts list ([branch_0..branch_3, branch_4_attempt_2]) matches the Rust output.)

Findings:
- [low/non-blocking] `fanout-parity-terminal-paths-skip-graph-receipt` Halt/Pause/Escalate terminal fanout outcomes return RuntimeError without producing a HarnessReceipt; TS still emits a graph receipt populated with sync_points.
  - Location: `crates/runx-runtime/src/runner.rs:400`
  - Evidence: fail_graph / block_graph / pause_for_sync / escalate_for_sync each push the FanoutReceiptSyncPoint into self.sync_points then return Err(RuntimeError::GraphPaused|GraphPlanningFailed|GraphEscalated), which short-circuits run_graph_file_with_caller before graph_receipt(...) is called. The TS counterparts in packages/runtime-local/src/runner-local/orchestrator/handle-terminal.ts (handleFailedPlan, handleBlockedPlan, handleEscalatedPlan) and handle-paused.ts push the sync point onto ctx.syncPoints and break to the finalize step which still emits a receipt. The fanout_parity.rs threshold-pause and retry-halt tests therefore only assert the error variant's embedded sync_decision; run.receipt.sync_points is never checked for these cases because no GraphRun is produced.
  - Impact: Receipt structure parity across languages is asymmetric on Halt/Pause/Escalate. The spec invariants scope receipt parity to 'partial failure receipts emit identical structure' (the Proceed-with-failures case is covered), so this is observational rather than blocking, but consumers comparing TS and Rust on the same Halt scenario will see a graph receipt only on the TS side.
  - Validation: Extend fanout_parity.rs threshold-pause/retry-halt tests to assert that the emitted graph receipt has the same sync_points as the TS oracle's threshold-pause/retry-halt entries.
