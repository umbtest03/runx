---
spec_version: '2.0'
task_id: x402-pay-idempotency-recovery-v1
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T17:31:48Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# x402-pay idempotency recovery v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T17:31:48Z
Review gate: pass

## Summary

Turn the remaining x402 Phase 1 idempotency and recovery eventualities into an
executable scafld contract:

- P1.7: replaying the same idempotency key returns the first sealed receipt and
  does not execute a second mock spend.
- P1.9: reusing the same single-use spend capability is denied by core, not by
  the mock rail.
- P1.11: a crash or abort after a partial mock rail mutation is recoverable by
  idempotency key and either seals the existing mutation or escalates with a
  typed recovery state.

The original "no observable payment state" blocker is lifted for focused Rust
state tests, same-key replay is executable at the runtime layer, partial rail
mutation recovery has a fail-closed escalation path, and the runnable fixture
set is promoted under `x402-pay-idempotency-*`.

## Context

CWD: `.`

Packages:
- `crates/runx-core`
- `crates/runx-runtime`
- `fixtures/graphs/payment`
- `fixtures/harness`
- `fixtures/skills/payment-fulfill`

Files impacted:
- `.scafld/specs/active/x402-pay-idempotency-recovery-v1.md`
- `fixtures/harness/x402-pay-idempotency-*.yaml`
- `fixtures/graphs/payment/x402-pay-idempotency-*.yaml`
- `fixtures/skills/x402-pay-idempotency-*/SKILL.md`

Invariants:
- Do not depend on a native `runx x402-pay`, `runx receipts`, or `runx ledger`
  command; current observable surfaces are `runx harness`, receipt output, and
  `runx history`.
- Do not touch `.scafld/specs/drafts/rust-nitrosend-dogfood.md`.
- Do not edit `crates/runx-cli/tests/x402_native_dogfood.rs` or
  `tests/x402-pay-dogfood-mock.test.ts` unless coordination confirms no other
  x402 worker owns them.
- New fixtures, when promoted, must live under clearly named
  `x402-pay-idempotency-*` paths.

Related docs:
- `.scafld/specs/archive/2026-05/x402-pay-dogfood-v1.md`
- `.scafld/specs/archive/2026-05/x402-pay-phase1-mock-scenario-fixtures-v1.md`

## Blocker Evidence

The current implementation now has durable payment state, runtime
replay/recovery behavior, and promoted fixture-backed coverage:

- `crates/runx-runtime/src/payment/state.rs:287` exposes persisted consumed
  spend-capability lookup, and `crates/runx-runtime/src/payment/state.rs:301`
  exposes persisted idempotency lookup.
- `crates/runx-runtime/src/payment/state.rs:315` persists payment step state,
  including consumed spend capability records, sealed idempotency entries, and
  mock rail mutation records.
- `crates/runx-runtime/src/execution/runner/steps.rs:96` calls
  `persist_payment_step_state` after the spend step receipt is built.
- `crates/runx-runtime/src/execution/runner/authority.rs:97` injects persisted
  consumed capability refs into core admission, so P1.9 no longer depends only
  on fixture-seeded `consumed_spend_capability_refs`.
- `crates/runx-runtime/src/payment/state.rs` stores replay-safe sealed outputs
  plus the original receipt timestamp and digest for idempotency replay. The
  stored outputs remove rail session material before persistence.
- `crates/runx-runtime/src/execution/runner/authority.rs` now detects sealed
  idempotency entries before persisted spend-consumption admission, revalidates
  the current authority shape without treating the capability as a fresh spend,
  and returns replay material to the runner.
- `crates/runx-runtime/src/execution/runner/steps.rs` short-circuits sealed
  idempotency replay before adapter invocation, rebuilds the original payment
  step receipt from stored material, and fails closed if receipt id, digest, or
  typed rail proof do not match the persisted entry.
- `crates/runx-runtime/tests/payment/execution.rs` proves both P1.7 runtime
  replay with no second `pay-fulfill-rail` call and P1.9 persisted consumed
  capability denial when the second run uses a new idempotency key.
- Partial rail state can be persisted as `in_flight`, and the runner now
  escalates that state by idempotency key before any second rail mutation is
  allowed. P1.11 runtime semantics are covered.
- `fixtures/harness/x402-pay-idempotency-replay.yaml` proves P1.7 fixture
  replay with one rail invocation.
- `fixtures/harness/x402-pay-idempotency-capability-reuse.yaml` proves P1.9
  fixture denial before a second rail invocation.
- `fixtures/harness/x402-pay-idempotency-crash-recovery.yaml` proves P1.11
  fixture recovery escalation before retrying the rail.

## Objectives

- Specify and execute the fixture matrix for P1.7, P1.9, and P1.11.
- Keep all fixture names under `x402-pay-idempotency-*`.

## Scope

- Runtime state and runner replay work for P1.7/P1.9.
- Fixture promotion for P1.7/P1.9/P1.11 after the corresponding runtime
  behavior is satisfied.
- No shared x402 dogfood tests are edited by this spec.

## Dependencies

- Durable idempotency index keyed by rail family, counterparty or grant, and
  idempotency key.
- Durable spend-capability consumption record keyed by capability ref and
  linked to the sealing receipt or recovery state.
- Durable mock rail mutation record with at least: idempotency key, rail,
  amount, currency, counterparty, mutation status, proof ref when known, and
  recovery classification.

## Assumptions

- The mock rail remains deterministic and local.
- Recovery may return either a sealed receipt or a governed escalation, but it
  must not silently execute an additional spend.
- The first implementation can use file-backed state as long as it survives
  process restart within a harness run and is observable by tests.

## Touchpoints

- `.scafld/specs/active/x402-pay-idempotency-recovery-v1.md`
- `crates/runx-runtime/tests/payment/execution.rs`
- `crates/runx-runtime/tests/payment/state.rs`
- `fixtures/harness/x402-pay-idempotency-replay.yaml`
- `fixtures/harness/x402-pay-idempotency-capability-reuse.yaml`
- `fixtures/harness/x402-pay-idempotency-crash-recovery.yaml`
- `fixtures/graphs/payment/x402-pay-idempotency-*.yaml`

## Risks

- Description: Static fixtures could falsely pass by duplicating hard-coded
  inputs.
  Mitigation: require an observable persisted state delta and a no-second-spend
  assertion before adding fixtures.
- Description: Recovery could be confused with retry.
  Mitigation: require recovery classification from persisted rail state before a
  second rail execution is allowed.

## Acceptance

Profile: strict

Validation:
- [x] `v1` spec - This scafld spec validates.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-34
- [x] `v3` fixture - Idempotency replay fixture passes.
  - Command: `runx harness fixtures/harness/x402-pay-idempotency-replay.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-35
- [x] `v4` fixture - Spend capability reuse fixture passes.
  - Command: `runx harness fixtures/harness/x402-pay-idempotency-capability-reuse.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `v5` fixture - Partial mutation recovery fixture passes.
  - Command: `runx harness fixtures/harness/x402-pay-idempotency-crash-recovery.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `v6` runtime P1.9 - Reusing the same single-use spend capability is
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment x402_paid_echo_reused_spend_capability_with_new_idempotency_denied_from_persisted_state_before_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38
- [x] `v7` runtime P1.7 - Replaying the same sealed idempotency key returns the
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment x402_paid_echo_replays_sealed_idempotency_without_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-39
- [x] `v8` runtime regression - Payment execution suite still passes with replay
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-40
- [x] `v9` runtime P1.11 - In-flight rail mutation recovery escalates without
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment x402_paid_echo_partial_mutation_escalates_without_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-41
- [x] `v10` compatibility - v2 payment state opens fail-closed after v3 replay
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-42

## Phase 1: State Layer Contract

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- `crates/runx-runtime/src/payment/state.rs` - persists idempotency entries, spend capability consumption, and mock rail mutation state.
- `crates/runx-runtime/tests/payment/state.rs` - covers the durable state semantics available before fixture-level replay/recovery.

Acceptance:
- [x] `ac1_1` state - A sealed payment receipt can be looked up by
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment persists_sealed_payment_step_state_for_replay_and_reuse_lookups`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-10
- [x] `ac1_2` state - A consumed spend capability ref is rejected when reused
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment x402_paid_echo_reused_spend_capability_with_new_idempotency_denied_from_persisted_state_before_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-11
- [x] `ac1_4` state - A sealed idempotency entry can be replayed from stored
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment x402_paid_echo_replays_sealed_idempotency_without_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-12
- [x] `ac1_3` state - A partial mock rail mutation is recoverable by
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment x402_paid_echo_partial_mutation_escalates_without_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13

## Phase 2: Executable Fixtures

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- `fixtures/harness/x402-pay-idempotency-replay.yaml` - executes one payment, replays the same idempotency key, and asserts the first receipt is returned with one rail mutation.
- `fixtures/harness/x402-pay-idempotency-capability-reuse.yaml` - executes one payment, attempts a second spend with the same capability ref, and asserts a core denial before rail execution.
- `fixtures/harness/x402-pay-idempotency-crash-recovery.yaml` - simulates a crash after partial mock rail mutation, invokes recovery by idempotency key, and asserts escalation before a second rail execution.

Acceptance:
- [x] `ac2_1` fixture - P1.7 has a runnable fixture.
  - Command: `test -f fixtures/harness/x402-pay-idempotency-replay.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18
- [x] `ac2_2` fixture - P1.9 has a runnable fixture.
  - Command: `test -f fixtures/harness/x402-pay-idempotency-capability-reuse.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19
- [x] `ac2_3` fixture - P1.11 has a runnable fixture.
  - Command: `test -f fixtures/harness/x402-pay-idempotency-crash-recovery.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20

## Rollback

Strategy: per_phase

Commands:
- `rm .scafld/specs/active/x402-pay-idempotency-recovery-v1.md`

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: command
Output: command.stdout
Summary: Command-provider verification pass. Rechecked x402-pay-idempotency-recovery-v1: durable payment-state tests pass, runtime P1.7/P1.9/P1.11 payment_execution coverage passes, the full payment execution regression suite passes, and the promoted harness fixtures pass when run through the native Rust CLI binary. The prior exit-127 blocker was command PATH, not runtime behavior; build was rerun with crates/target/debug on PATH and the spec advanced to review. No completion blockers found.

Attack log:
- `durable payment state`: run payment_state coverage for sealed replay, consumed capability, and partial mutation records -> clean
- `P1.7 replay`: run x402_paid_echo_replays_sealed_idempotency_without_second_rail -> clean
- `P1.9 consumed capability`: run x402_paid_echo_reused_spend_capability_with_new_idempotency_denied_from_persisted_state_before_second_rail -> clean
- `P1.11 recovery escalation`: run x402_paid_echo_partial_mutation_escalates_without_second_rail -> clean
- `fixture matrix`: run native CLI harness fixtures for replay, capability reuse, and crash recovery -> clean
- `PATH blocker`: inspect failed diagnostics showing sh: runx: command not found, then verify rerun with crates/target/debug on PATH advanced to review -> clean

Findings:
- none

## Self Eval

Status: complete
Completeness: durable state semantics, runtime replay, runtime recovery
escalation, and standalone fixture promotion covered
Architecture fidelity: current harness and runtime state surfaces respected
Spec alignment: P1.7, P1.9, and P1.11 mapped directly
Validation depth: focused Rust runtime state tests plus full payment execution
suite
Total: complete
Second pass performed: yes

Notes:
Runtime replay, fail-closed recovery escalation, and standalone harness
fixtures are now executable.

Improvements:
- Keep these fixtures in the payment regression suite when x402 provider
  projectors evolve.

## Deviations

- The harness promotion is intentionally sequence-aware because each fixture
  needs two graph executions over one shared payment-state file.

## Metadata

Estimated effort hours: 4
Actual effort hours: 1
AI model: gpt-5-codex
React cycles: 0

Tags:
- x402
- payments
- idempotency
- recovery

## Origin

Source:
- Worker E OSS x402 idempotency/recovery spec lane

Repo:
- `/Users/kam/dev/runx/runx/oss`

Git:
- dirty worktree; unrelated `rust-nitrosend-dogfood.md` modification preserved

Sync:
- none

Supersession:
- none

## Harden Rounds

- none

## Planning Log

- 2026-05-21T00:00:00Z: Filed blocked spec for P1.7/P1.9/P1.11 after code
  evidence showed missing observable persisted payment state.
- 2026-05-21T13:05:33Z: Confirmed durable payment state primitives exist,
  added focused runtime state tests, and kept fixtures blocked on runner
  replay/recovery behavior.
- 2026-05-21T13:17:07Z: Added runtime P1.9 coverage proving persisted consumed
  spend capability state denies a second paid echo run before a second rail
  invocation.
- 2026-05-21T14:09:12Z: Added replay-safe sealed output persistence, runner
  idempotency replay before rail invocation, runtime P1.7 coverage, and updated
  P1.9 to prove a new idempotency key with the consumed spend capability still
  denies before rail.
- 2026-05-21T14:17:26Z: Added fail-closed in-flight rail mutation recovery
  escalation before a second rail invocation, v2 payment-state compatibility,
  and runtime P1.11 coverage.
- 2026-05-22T00:55:00+10:00: Promoted P1.7/P1.9/P1.11 into standalone
  `x402-pay-idempotency-*` harness fixtures and validated them through the Rust
  harness.
