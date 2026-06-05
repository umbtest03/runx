---
spec_version: '2.0'
task_id: runx-readiness-gate-hardening-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T03:25:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# runx-readiness-gate-hardening-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: first readiness lane; make shipped-code gates fail loud before more fronts
Blockers: none
Allowed follow-up command: `scafld approve runx-readiness-gate-hardening-v1`
Latest runner update: none
Review gate: not_started

## Summary

Make the current clean shape hard to regress. The core CI path is strong, but the
last advisory lane is still `continue-on-error`, several useful checks live as
manual scripts, and there is no explicit guard for the cleanup quality the project
now expects: no orphan examples, no untracked committed build output, no empty
leftover directories, no resurrected compatibility packages, and no stale demo
surface.

This spec is intentionally first. It does not add product scope. It turns the
existing readiness posture into enforceable gates.

## Objectives

- Remove `continue-on-error` from the Rust parity/advisory lane, or split the
  lane into required checks and explicitly documented non-blocking diagnostics.
- Wire the high-signal local gates into `verify:fast` or CI: demo receipts,
  release smoke, package boundary, orphan example/package checks, empty-dir checks,
  committed-dist policy, and anti-retired-package imports.
- Tighten underconstrained approval gate schema coverage where readiness depends
  on it; do not silently accept unconstrained action `type` values if the runtime
  expects a closed shape.
- Deduplicate overlapping runtime compatibility checks so one source of truth owns
  each boundary.
- Keep CI runtime practical; slow live-funded lanes stay opt-in and preflight-only.

## Scope

In scope:
- `.github/workflows/ci.yml`, `.github/workflows/release.yml`.
- `scripts/verify-fast.mjs`, `scripts/check-verify-fast-plan.mjs`,
  `scripts/check-boundaries.mjs`, `scripts/check-runtime-cutover-legacy.mjs`, and
  new focused guard scripts if needed.
- Guard coverage for examples, package manifests, committed dist policy, empty
  dirs, orphan package dirs, retired `@runxhq/core` imports, and live-lane
  preflight commands.

Out of scope:
- New demos, new fronts, new product runtime behavior, hosted live settlement.

## Dependencies

- Existing gates: `pnpm verify:fast`, `pnpm demos:check`,
  `pnpm x402:dogfood:local`, `cargo fmt`, `cargo clippy`, `cargo nextest`,
  `cargo test --doc`, license boundary checks, and `scripts/check-rust-kernel-parity.mjs`.
- Focused coverage already exists for policy, payment authority, and contract
  schema validation; this spec wires those signals rather than inventing broad
  review checks.
- The demo-prune spec should run after this or in parallel, but this spec owns the
  guard shape that prevents demo cruft from returning.

## Assumptions

- The target is a readiness gate, not a broad review. Any failing guard must point
  to a small concrete fix.
- Live-funded payment lanes stay preflight-only in default CI; they must never
  require private keys or funded wallets.

## Risks

- **CI becomes too slow.** Mitigation: keep heavy Rust gates in the existing Rust
  job, keep live settlement opt-in, and run cheap structural guards in
  `verify:fast`.
- **False-positive cleanup guards block useful generated output.** Mitigation:
  encode explicit allowlists with rationale, not broad regex exceptions.
- **Advisory checks are flaky because tools are missing.** Mitigation: install
  required tools in CI or split install checks into loud preflight failures.

## Acceptance

Profile: strict

Validation:
- `pnpm verify:fast` includes the new structural readiness guards and fails if
  demo/package/orphan/retired-surface rules are violated.
- CI has no `continue-on-error` for gates that are described as required.
- `pnpm x402:dogfood:local` remains zero-funded and green.
- No default gate requires private credentials, funded testnet wallets, or network
  settlement.

## Phase 1: Turn advisory gates into required gates

Status: pending
Dependencies: none

Objective: remove ambiguity from CI.

Changes:
- Audit `.github/workflows/ci.yml` and `scripts/check-rust-kernel-parity.mjs`.
- Either make the advisory Rust parity lane fail CI or split it into required
  subchecks plus explicitly non-blocking diagnostics.
- Update docs and `verify:fast` plan guard so the intended required gates are
  machine-checked.

Acceptance:
- [ ] `p1_ac1` command - no required gate is continue-on-error
  - Command: `! rg -n "continue-on-error:\\s*true" .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac2` command - verify-fast plan still matches required ordering
  - Command: `pnpm verify:fast:plan-check`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Add structural cleanup guards

Status: pending
Dependencies: Phase 1

Objective: prevent the debris ring from reappearing.

Changes:
- Add guards for orphan examples, empty leftover directories, retired package
  imports, committed dist policy, stale archived/draft duplicate specs, and
  untracked demo runners.
- Add or wire focused checks for policy fixtures/proptests, payment authority
  hardening, and schema wire validation where they are not already required.
- Wire the guards into `verify:fast` and CI.

Acceptance:
- [ ] `p2_ac1` command - structural guards pass
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - demo/local payment dogfood remains green
  - Command: `pnpm demos:check && pnpm x402:dogfood:local`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac3` command - focused authority and schema gates pass
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy_fixtures_match_rust_policy && cargo test --manifest-path crates/Cargo.toml -p runx-core --test policy_proptest && cargo test --manifest-path crates/Cargo.toml -p runx-pay authority && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test integration schema_wire_conformance schema_validation`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Final gate proof

Status: pending
Dependencies: Phase 2

Objective: prove the readiness gate set from a clean checkout.

Acceptance:
- [ ] `p3_ac1` command - full local readiness gate
  - Command: `pnpm verify:fast && cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo nextest run --workspace --all-features && cargo test --workspace --all-features --doc`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Revert the guard scripts and CI changes together. Do not leave dead scripts
  unwired.

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
