---
spec_version: '2.0'
task_id: x402-pay-phase1-negative-fixtures-v1
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T10:25:04Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# x402-pay Phase 1 negative fixtures v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T10:25:04Z
Review gate: pass

## Summary

Add the next x402-pay Phase 1 negative fixture lane for the first punch-list
negatives: malformed challenge, cap exceeded, ambiguous bounds, and proofless
rail. The lane stays fixture/spec/test scoped and proves that refusals stop
before reserve, approval, fulfillment, settlement, or paid-echo propagation as
appropriate.

## Scope And Touchpoints

In scope:

- `.scafld/specs/active/x402-pay-phase1-negative-fixtures-v1.md`
- `.scafld/specs/archive/2026-05/x402-pay-phase1-mock-scenario-punchlist.md`
- `fixtures/graphs/payment/x402-pay-negative-*.yaml`
- `fixtures/harness/x402-pay-negative-*.yaml`
- `fixtures/skills/x402-pay-negative-*/SKILL.md`
- `fixtures/skills/x402-pay-negative-*/run.sh`
- `crates/runx-cli/tests/x402_native_dogfood.rs`
- `tests/x402-pay-dogfood-mock.test.ts`

Out of scope:

- Runtime implementation changes.
- Live x402 rails, Stripe, refunds, disputes, or money movement.
- New user-facing CLI commands or payment schemas.
- Closing the remaining Phase 1 rows P1.7 through P1.11, P1.13, P1.14, or
  P1.17.

## Planned Phases

Phase 1: fixture lane.
: Add deterministic negative graph/harness fixtures for P1.2, P1.3, P1.4, and
P1.12 using the existing x402 paid-echo and native harness patterns.

Phase 2: executable dogfood.
: Extend the native x402 dogfood test to assert blocked child receipt sets,
pre-rail cap refusal, and proofless rail denial before paid echo.

Phase 3: punch-list accounting.
: Mark the four closed rows in the append-only punch-list and update the mock
coverage sentinel so only the remaining uncovered scenarios must stay open.

## Acceptance

Profile: strict

Validation:
- [x] `v1` scafld - Spec validates.
  - Command: `scafld validate x402-pay-phase1-negative-fixtures-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-3
- [x] `v2` native dogfood - Narrow native x402 negative fixture test passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-4
- [x] `v3` dogfood sentinel - Mock scenario coverage accounting remains green.
  - Command: `cargo build --quiet --manifest-path crates/Cargo.toml -p runx-cli --bin runx && RUNX_KERNEL_EVAL_BIN=crates/target/debug/runx pnpm exec vitest run tests/x402-pay-dogfood-mock.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-5
- [x] `v4` native dogfood full file - Existing x402 native dogfood remains green.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6

## Rollback

Strategy: per_phase

Commands:
- `git checkout HEAD -- .scafld/specs/archive/2026-05/x402-pay-phase1-mock-scenario-punchlist.md crates/runx-cli/tests/x402_native_dogfood.rs tests/x402-pay-dogfood-mock.test.ts`
- `rm -f .scafld/specs/active/x402-pay-phase1-negative-fixtures-v1.md fixtures/graphs/payment/x402-pay-negative-*.yaml fixtures/harness/x402-pay-negative-*.yaml`
- `rm -rf fixtures/skills/x402-pay-negative-malformed-challenge-quote fixtures/skills/x402-pay-negative-cap-exceeded-reserve fixtures/skills/x402-pay-negative-ambiguous-bounds-reserve fixtures/skills/x402-pay-negative-proofless-fulfill`

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-21T00:00:00Z
Ended: 2026-05-21T00:00:00Z

Checks:
- path audit
  - Grounded in: code:fixtures/harness/x402-pay-paid-echo.yaml:1
  - Result: passed
  - Evidence: New fixtures follow the existing graph/harness/skill layout.
- scope audit
  - Grounded in: spec:Scope And Touchpoints
  - Result: passed
  - Evidence: Changes are limited to x402/payment fixtures, tests, and specs.
- negative-stop audit
  - Grounded in: code:crates/runx-cli/tests/x402_native_dogfood.rs:1
  - Result: passed
  - Evidence: Assertions check child receipt boundaries and failure stderr.

Issues:
- none

## Planning Log

- 2026-05-21T00:00:00Z: Filed as the follow-up lane closing P1.2, P1.3,
  P1.4, and P1.12 from the Phase 1 mock scenario punch-list.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: local
Output: local.fixture
Summary: Human-reviewed override accepted: Reviewed scoped diff and reran scafld validate, cargo x402_native_dogfood, and x402-pay dogfood mock vitest in the shared worktree; no task-scope findings.

Attack log:
- `review gate`: manual human audit -> clean (Reviewed scoped diff and reran scafld validate, cargo x402_native_dogfood, and x402-pay dogfood mock vitest in the shared worktree; no task-scope findings.)

Findings:
- none

