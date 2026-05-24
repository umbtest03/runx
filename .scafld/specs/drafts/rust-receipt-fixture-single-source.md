---
spec_version: '2.0'
task_id: rust-receipt-fixture-single-source
created: '2026-05-24T00:00:00Z'
updated: '2026-05-24T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust-driven receipt/digest fixtures (kill the canonicalization divergence)

## Current State

Status: draft
Current phase: planning
Next: harden
Reason: A+ roadmap step 2. `scripts/generate-rust-harness-fixtures.ts` recomputes
`body_digest`/`receipt_digest` in TypeScript via `canonicalJsonStringify` from
`@runxhq/contracts` (lines 5/111/116). That TS canonicalizer diverges from the
Rust binary's canonicalizer, so `pnpm fixtures:harness:check` is red-by-design and
is deliberately excluded from the CI gate. The digests committed in the `.yaml`
fixtures are the Rust-correct ones; the TS check reports them stale forever.
Blockers: none. The Rust binary already emits authoritative receipts.

## Summary

Make the harness/receipt fixtures Rust-driven: the digests and canonical bodies
are produced by the authoritative Rust binary (or the `runx-receipts`
canonicalizer), not reimplemented in TypeScript. This removes the single most
dangerous silent-drift surface in the repo, the one that produced the
aster-control `harness_receipt_ref` bug class, and turns `fixtures:harness:check`
green by construction so it can re-enter the gate.

## Objectives

- Replace the TS digest recomputation in `generate-rust-harness-fixtures.ts` with
  invocation of the Rust binary (`runx harness <fixture> --json`) or a thin
  Rust regeneration `bin`, so fixture digests are Rust-true by construction.
- Stop importing `canonicalJsonStringify`/`sha256Prefixed` for digest computation
  in the fixture generators; the only canonicalizer that computes committed
  digests is Rust.
- Re-enable `fixtures:harness:check` (now wire-true) and add it back to the CI
  gate.

## Scope

In scope: `scripts/generate-rust-harness-fixtures.ts`, related harness fixture
generators, the `fixtures:harness:check` package script and its CI wiring.

Out of scope: the contract-shape pipeline inversion (owned by
`rust-contract-pipeline-inversion`); changing any wire shape or digest value.

## Dependencies

- Relies on the Rust binary being buildable in the fixture/CI environment
  (`RUNX_KERNEL_EVAL_BIN`/release binary availability), shared with
  `heavy-test-suite-gating`.

## Acceptance

- [ ] `dod1` Harness/receipt fixture digests are produced by Rust, not by a TS
  reimplementation; the TS generators no longer compute digests via
  `canonicalJsonStringify`.
- [ ] `dod2` `fixtures:harness:check` passes and is part of the CI gate (no
  longer red-by-design).
- [ ] `dod3` No committed digest value changes (the fixtures already carry the
  Rust-true digests); only the generator's source of truth changes.

## Origin

A+ roadmap (2026-05-24), step 2. The TS↔Rust canonicalization divergence is the
deepest cross-language inconsistency and the root of the worst silent-drift bug
class. Captured during the contract-spine inversion work.
