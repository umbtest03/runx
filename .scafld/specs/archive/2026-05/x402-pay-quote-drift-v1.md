---
spec_version: '2.0'
task_id: x402-pay-quote-drift-v1
created: '2026-05-21T09:36:00Z'
updated: '2026-05-21T09:36:00Z'
status: completed
harden_status: not_run
size: small
risk_level: medium
---

# x402-pay quote drift v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: implemented with fixture-only CLI coverage
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T09:36:00Z
Review gate: not_run

## Summary

Close P1.14 by adding a native CLI harness fixture that routes a quote-drifted
x402 reservation through Rust runtime authority admission. The reservation keeps
the child authority subset-valid and bounded to the quoted `125` minor-unit
spend, but binds the spend capability at `175`, above the reserved bounds.

## Scope And Touchpoints

In scope:

- `fixtures/graphs/payment/x402-pay-negative-quote-drift.yaml`
- `fixtures/harness/x402-pay-negative-quote-drift.yaml`
- `fixtures/skills/x402-pay-negative-quote-drift-reserve/SKILL.md`
- `fixtures/skills/x402-pay-negative-quote-drift-reserve/run.sh`
- `crates/runx-cli/tests/x402_native_dogfood.rs`
- `tests/x402-pay-dogfood-mock.test.ts`
- `.scafld/specs/archive/2026-05/x402-pay-phase1-mock-scenario-punchlist.md`

Out of scope:

- Rust runtime source changes.
- Live x402 rails, Stripe, refunds, disputes, or money movement.
- Closing unrelated Phase 1 punch-list rows.

## Acceptance

Profile: strict

Validation:
- [x] `v1` scafld - Spec validates.
  - Command: `scafld validate x402-pay-quote-drift-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
- [x] `v2` native dogfood - Native x402 negative fixture test passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
- [x] `v3` dogfood sentinel - Mock scenario coverage accounting recognizes P1.14 closure.
  - Command: `cargo build --quiet --manifest-path crates/Cargo.toml -p runx-cli --bin runx && RUNX_KERNEL_EVAL_BIN=crates/target/debug/runx pnpm exec vitest run tests/x402-pay-dogfood-mock.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0

## Evidence

The crafted reservation fixture reserves a child authority with
`max_per_call_minor: 125` under a parent that allows `10000`, so the authority
subset check remains valid. The `spend_capability_binding.amount_minor` is
`175`, which diverges above the reserved child bounds and causes native
authority admission to return `payment spend capability binding does not match`
before the fulfill skill can emit mock credential or rail session material.

## Rollback

Strategy: per_file

Commands:
- `git checkout HEAD -- crates/runx-cli/tests/x402_native_dogfood.rs tests/x402-pay-dogfood-mock.test.ts .scafld/specs/archive/2026-05/x402-pay-phase1-mock-scenario-punchlist.md`
- `rm -f fixtures/graphs/payment/x402-pay-negative-quote-drift.yaml fixtures/harness/x402-pay-negative-quote-drift.yaml .scafld/specs/archive/2026-05/x402-pay-quote-drift-v1.md`
- `rm -rf fixtures/skills/x402-pay-negative-quote-drift-reserve`
