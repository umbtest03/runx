---
spec_version: '2.0'
task_id: runx-live-rail-verification-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T04:22:06Z'
status: active
harden_status: passed
size: medium
risk_level: high
---

# runx-live-rail-verification-v1

## Current State

Status: active
Current phase: phase1
Next: funded-x402-env
Reason: zero-funded dogfood, CDP preflight/refusal, and real Stripe SPT test-mode dogfood passed; official x402 and x402-rs live runs remain blocked on funded testnet resources
Blockers: dedicated funded Base Sepolia payer wallet/RPC/facilitator env for upstream x402 and x402-rs
Allowed follow-up command: `node scripts/x402-upstream-conformance.mjs --run` after funded env is configured
Latest runner update: 2026-06-05T04:22:06Z
Review gate: not_started

## Summary

Run the real payment verification lanes without expanding the demo surface:
official upstream x402 conformance, x402-rs interop, CDP hosted-facilitator
preflight/run profile, and Stripe SPT test mode. The default local dogfood lane
already proves Runx authority/refusal/receipt behavior without funds. This spec
proves actual external settlement with isolated test credentials and records the
artifacts in a reproducible way.

## Objectives

- Official upstream x402 HTTP 402 conformance succeeds from a clean upstream
  checkout at a recorded commit.
- x402-rs independent implementation interop succeeds at a recorded commit.
- CDP hosted-facilitator profile is implemented using official authentication,
  with the signup-free x402.org testnet facilitator as a documented fallback.
- Stripe SPT test-mode path runs with test credentials and sealed receipts.
- No secrets, private keys, `.env` files, or generated wallets are committed.

## Scope

In scope:
- `scripts/x402-upstream-conformance.mjs`,
  `scripts/x402-interop.mjs`, `scripts/x402-local-dogfood.mjs`.
- `examples/governed-spend/{README.md,x402.sh,stripe-spt.sh}`.
- Artifact capture: upstream SHA, run command, output JSON/log, receipt ids,
  transaction/charge ids where applicable.

Out of scope:
- Mainnet, real-money settlement, custodial wallet handling, demo-gallery polish.
- Expanding mock rails.

## Dependencies

- Dedicated funded testnet wallets and RPC endpoints for upstream x402/x402-rs.
- Stripe test-mode key and webhook secret held only in the shell or secret store.
- CDP API credentials for the hosted-facilitator profile.

## Assumptions

- A real testnet settlement cannot be honestly proven without some funded balance.
- The zero-funded lane remains the default contributor path.

## Risks

- **Secret leakage.** Mitigation: never write `.env`; scripts print missing env
  names only, never values.
- **Upstream churn.** Mitigation: record upstream SHA and artifact directory for
  every run.
- **Live flake.** Mitigation: keep live lanes opt-in and artifact-backed, not part
  of default CI.

## Acceptance

Profile: strict

Validation:
- `pnpm x402:dogfood:local` passes with zero funds.
- `node scripts/stripe-spt-charge.mjs --check` passes without secrets and
  reports `can_run: false` with missing env names when Stripe test credentials
  are absent.
- Official x402 conformance `--run` succeeds when funded env is present.
- x402-rs interop `--run` succeeds when funded env is present.
- CDP hosted-facilitator profile exists and can be preflighted without secrets.
- Stripe SPT live/test-mode path produces receipt artifacts that verify offline.

## Phase 1: Upstream and x402-rs live runs

Status: active
Dependencies: funded testnet env

Objective: prove official and independent x402 implementations.

Changes:
- Run and capture artifacts for upstream x402 and x402-rs.
- Tighten wrappers if their env validation or artifact capture is insufficient.

Acceptance:
- [x] `p1_ac1` command - zero-funded lane remains green
  - Command: `pnpm x402:dogfood:local`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0. The deterministic payment, x402 mock, and
  - Source event: local-shell-2026-06-05
- [ ] `p1_ac2` command - official upstream x402 conformance succeeds with funded env
  - Command: `node scripts/x402-upstream-conformance.mjs --run`
  - Expected kind: `exit_code_zero`
  - Status: blocked
  - Evidence: not run. `node scripts/x402-upstream-conformance.mjs --check`
  - Source event: local-shell-2026-06-05
- [ ] `p1_ac3` command - x402-rs interop succeeds with funded env
  - Command: `node scripts/x402-interop.mjs --target x402-rs --run`
  - Expected kind: `exit_code_zero`
  - Status: blocked
  - Evidence: not run. `node scripts/x402-interop.mjs --target x402-rs
  - Source event: local-shell-2026-06-05

## Phase 2: CDP hosted-facilitator profile

Status: blocked
Dependencies: CDP API credentials

Objective: preflight the hosted-facilitator profile without a Runx-specific shim
and record the live-run implementation blocker.

Changes:
- Keep `scripts/x402-interop.mjs --target cdp --check` no-secret and explicit
  about the hosted-facilitator requirements.
- Block the live run until official CDP authentication and the same Base Sepolia
  exact flow are wired without inventing a credential env contract.

Acceptance:
- [x] `p2_ac1` command - CDP preflight works without printing secrets
  - Command: `node scripts/x402-interop.mjs --target cdp --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0. Report schema was `runx.x402.interop.v1`,
    target was `cdp`, `can_run: false`, and the blocker was explicit: CDP API
    credentials, a dedicated funded Base Sepolia payer wallet, an
    operator-owned artifact directory, and a credential env contract that is
    `not_implemented`.
  - Source event: local-shell-2026-06-05
- [x] `p2_ac2` command - CDP planned live run refuses instead of faking success
  - Command: `node scripts/x402-interop.mjs --target cdp --run`
  - Expected kind: `exit_code_nonzero`
  - Status: pass
  - Evidence: exit code was 1. The command emitted the same
    `runx.x402.interop.v1` blocker report with `mode: run`, `can_run: false`,
    `target_status: planned`, and then failed with: `CDP hosted-facilitator
    live run is not implemented; use --check for the no-secret preflight
    report`.
  - Source event: local-shell-2026-06-05

## Phase 3: Stripe SPT test-mode proof

Status: blocked
Dependencies: Stripe test credentials

Objective: prove the Stripe test-mode leg and offline receipts.

Acceptance:
- [x] `p3_ac0` command - Stripe SPT preflight works without secrets
  - Command: `node scripts/stripe-spt-charge.mjs --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0. Report schema was
    `runx.stripe_spt.preflight.v1`, no Stripe API call was made, no secret
    values were printed, and `can_run: false` named the missing
    `STRIPE_SECRET_KEY or STRIPE_TEST_KEY` and `STRIPE_WEBHOOK_SECRET` env.
  - Source event: local-shell-2026-06-05
- [x] `p3_ac1` command - Stripe SPT live/test-mode receipt verifies offline
  - Command: `RUNX_STRIPE_DEMO_MODE=live sh examples/governed-spend/stripe-spt.sh`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0 with operator-provided Stripe test-mode env held
    only in the shell. Stripe returned `PaymentIntent`
    `pi_3TepI2QY1nIHXeog4uw5I24T` and charge
    `ch_3TepI2QY1nIHXeog4t4DzKjg`; the local webhook signature check passed;
    settlement receipt `sha256:37433467f751ca734689c25bc9e80cd798447927e3dd9336bebf523d8a64d4ab`
    and refusal receipt
    `sha256:33e354612e389657d8d680dacd2ebdf9c0e99bf8367576c82dfdf312d38ecdfe`
    both verified offline.
  - Source event: local-shell-2026-06-05

## Rollback

- Remove only run-profile changes. Keep zero-funded dogfood and docs intact.

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

- created_by: codex

## Origin

Created by: Codex
Source: operator readiness queue

## Harden Rounds

### round-1

Status: passed
Started: 2026-06-05T03:47:32Z
Ended: 2026-06-05T03:53:31Z

Checks:
- Path audit
  - Grounded in: code:examples/governed-spend/README.md:77
  - Result: passed
  - Evidence: The governed-spend README is already the local/no-funded and live-rail operator surface for x402, x402-rs, CDP, and Stripe SPT.
- Command audit
  - Grounded in: code:scripts/x402-local-dogfood.mjs:31
  - Result: passed
  - Evidence: The zero-funded dogfood lane preflights upstream x402, x402-rs, CDP, and Stripe SPT. Stripe SPT now runs through a no-secret `--check` report and is treated as an optional live-readiness blocker, not a local dogfood failure.
- Scope/migration audit
  - Grounded in: code:scripts/x402-upstream-conformance.mjs:36
  - Result: passed
  - Evidence: The upstream wrapper emits JSON containing required env names, missing env names, command, artifact dir, and upstream SHA without reading or writing secrets.
- Acceptance timing audit
  - Grounded in: spec_gap:Acceptance
  - Result: passed
  - Evidence: This local pass keeps funded `--run` commands blocked on external resources while proving no-funded readiness through `pnpm x402:dogfood:local` and explicit live prerequisite reports.
- Rollback/repair audit
  - Grounded in: code:scripts/stripe-spt-charge.mjs:410
  - Result: passed
  - Evidence: Stripe live mode refuses live-mode keys and validates test-key prefixes, while `--check` reports missing or invalid env names without calling Stripe or writing receipt artifacts.
- Design challenge
  - Grounded in: code:scripts/x402-interop.mjs:82
  - Result: passed
  - Evidence: CDP remains a hosted-facilitator plan with `can_run: false`, explicit required external resources, and no Runx-specific shim or guessed credential env contract.

Issues:
- none


## Planning Log

- none
