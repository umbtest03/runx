---
spec_version: '2.0'
task_id: runx-live-rail-verification-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T03:25:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# runx-live-rail-verification-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: prove live rails only after zero-funded dogfood and readiness gates are clean
Blockers: dedicated funded testnet wallets and provider test credentials
Allowed follow-up command: `scafld approve runx-live-rail-verification-v1`
Latest runner update: none
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
- Official x402 conformance `--run` succeeds when funded env is present.
- x402-rs interop `--run` succeeds when funded env is present.
- CDP hosted-facilitator profile exists and can be preflighted without secrets.
- Stripe SPT live/test-mode path produces receipt artifacts that verify offline.

## Phase 1: Upstream and x402-rs live runs

Status: pending
Dependencies: funded testnet env

Objective: prove official and independent x402 implementations.

Changes:
- Run and capture artifacts for upstream x402 and x402-rs.
- Tighten wrappers if their env validation or artifact capture is insufficient.

Acceptance:
- [ ] `p1_ac1` command - zero-funded lane remains green
  - Command: `pnpm x402:dogfood:local`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac2` command - official upstream x402 conformance succeeds with funded env
  - Command: `node scripts/x402-upstream-conformance.mjs --run`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac3` command - x402-rs interop succeeds with funded env
  - Command: `node scripts/x402-interop.mjs --target x402-rs --run`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: CDP hosted-facilitator profile

Status: pending
Dependencies: CDP API credentials

Objective: add the hosted-facilitator run profile without a Runx-specific shim.

Changes:
- Extend `scripts/x402-interop.mjs --target cdp` from check-only to preflight/run.
- Use official CDP authentication and the same Base Sepolia exact flow.

Acceptance:
- [ ] `p2_ac1` command - CDP preflight works without printing secrets
  - Command: `node scripts/x402-interop.mjs --target cdp --check`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Stripe SPT test-mode proof

Status: pending
Dependencies: Stripe test credentials

Objective: prove the Stripe test-mode leg and offline receipts.

Acceptance:
- [ ] `p3_ac1` command - Stripe SPT live/test-mode receipt verifies offline
  - Command: `RUNX_STRIPE_DEMO_MODE=live sh examples/governed-spend/stripe-spt.sh`
  - Expected kind: `exit_code_zero`
  - Status: pending

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

- none

## Planning Log

- none
