---
spec_version: '2.0'
task_id: x402-pay-idempotency-fixtures-v1
created: '2026-05-21T14:25:00Z'
updated: '2026-05-21T17:31:48Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# x402-pay idempotency fixtures v1

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

Promote the x402 idempotency/recovery runtime scenarios into executable harness
fixtures under `x402-pay-idempotency-*`. The fixture runner owns the sequence
semantics because one scenario needs two graph executions over the same
payment-state file and a no-second-rail assertion.

## Objectives

- Add `x402-pay-idempotency-replay`, `x402-pay-idempotency-capability-reuse`,
  and `x402-pay-idempotency-crash-recovery` harness fixtures.
- Add deterministic cli-tool skills that can vary idempotency key and rail mode
  through allowlisted fixture env.
- Extend the Rust harness with a narrow `x402_idempotency_sequence` graph shape
  that runs two graph executions against one temporary payment-state file and
  asserts the rail mutation count remains one.
- Keep the existing runtime tests as lower-level coverage.

## Scope

In scope:
- `fixtures/harness/x402-pay-idempotency-*.yaml`
- `fixtures/graphs/payment/x402-pay-idempotency-*.yaml`
- `fixtures/skills/x402-pay-idempotency-*`
- `crates/runx-runtime/src/execution/harness/runner.rs`
- `crates/runx-runtime/tests/harness_fixtures.rs`

Out of scope:
- Provider rail implementation.
- Stripe/API dogfood.
- Payment authority algebra changes already covered by prior specs.

## Acceptance

- [x] `dod1` P1.7 fixture proves same idempotency key returns the original
  sealed fulfill receipt and rail invocation count stays one.
- [x] `dod2` P1.9 fixture proves a second idempotency key with the same
  consumed spend capability is denied before a second rail mutation.
- [x] `dod3` P1.11 fixture proves a partial rail mutation escalates by
  idempotency key before retrying the rail.
- [x] `dod4` All new fixture paths are named `x402-pay-idempotency-*`.

## Validation

- [x] `v1` spec validates.
  - Command: `scafld validate x402-pay-idempotency-fixtures-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: returned `ok:true` on 2026-05-22T00:55:00+10:00.
- [x] `v2` harness fixture tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test harness_fixtures x402_idempotency -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: 1 focused test passed on 2026-05-22T00:42:00+10:00; the full
    `harness_fixtures` suite also passed with 12 tests on
    2026-05-22T00:55:00+10:00.
- [x] `v3` focused payment runtime tests still pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: 18 payment execution tests passed on
    2026-05-22T00:55:00+10:00.
- [x] `v4` native CLI harness fixtures pass.
  - Command: `cargo run --quiet --manifest-path crates/Cargo.toml -p runx-cli -- harness fixtures/harness/x402-pay-idempotency-*.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: replay, capability-reuse, and crash-recovery fixture commands all
    exited 0 on 2026-05-22T00:55:00+10:00 with closed, blocked, and deferred
    graph receipts respectively.

## Origin

Follow-up from `x402-pay-idempotency-recovery-v1`: runtime blockers are cleared;
fixtures are the remaining proof surface.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: command
Output: command.stdout
Summary: Command-provider verification pass. Rechecked x402-pay-idempotency-fixtures-v1: all x402-pay-idempotency-* fixture paths are present, focused harness fixture coverage passed, payment_execution passed, and native CLI harness replay supports the promoted fixture matrix with closed/blocked/deferred receipts. No completion blockers found.

Attack log:
- `fixture naming`: verify promoted fixture paths use x402-pay-idempotency-* -> clean
- `harness sequence coverage`: run focused harness_fixtures x402_idempotency test -> clean
- `payment runtime regression`: run payment_execution suite -> clean
- `native CLI fixture matrix`: run cargo run -p runx-cli -- harness fixtures/harness/x402-pay-idempotency-*.yaml --json and verify three receipts -> clean

Findings:
- none

