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
Current phase: implemented and fixture-check validated
Next: keep `fixtures:harness:check` in the fast/CI gate while the broader
runtime cutover lands.
Reason: `scripts/generate-rust-harness-fixtures.ts` now delegates to the Rust
`runx-harness-fixture-oracles` binary (or builds/runs it through Cargo) instead
of importing `canonicalJsonStringify` / `sha256Prefixed` and recomputing receipt
digests in TypeScript. `fixtures:harness:check` is included in `verify:fast`,
and the fast gate prebuilds/passes `RUNX_HARNESS_FIXTURE_ORACLE_BIN`. The
fixture check passed, and this pass found no `fixtures/harness` digest/oracle
diff.
Blockers: none for this fixture single-source slice.

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

- [x] `dod1` Harness/receipt fixture digests are produced by Rust, not by a TS
  reimplementation; the TS generators no longer compute digests via
  `canonicalJsonStringify`.
  - Command: `rg -n "canonicalJsonStringify|sha256Prefixed|runx-harness-fixture-oracles|RUNX_HARNESS_FIXTURE_ORACLE_BIN|fixtures:harness:check" scripts/generate-rust-harness-fixtures.ts scripts/verify-fast.mjs package.json`
  - Expected kind: `reviewed_output`
  - Status: reviewed
  - Evidence: generator delegates to the Rust oracle binary; `verify:fast`
    exports `RUNX_HARNESS_FIXTURE_ORACLE_BIN` and runs
    `fixtures:harness:check`.
- [x] `dod2` `fixtures:harness:check` passes and is part of the CI gate (no
  longer red-by-design).
  - Command: `pnpm fixtures:harness:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 harness fixture check passed using the Rust oracle.
- [x] `dod3` No committed digest value changes (the fixtures already carry the
  Rust-true digests); only the generator's source of truth changes.
  - Command: `git diff --name-only -- fixtures/harness`
  - Expected kind: `reviewed_output`
  - Status: reviewed
  - Evidence: no `fixtures/harness` fixture/oracle digest files changed in this
    pass.

Evidence (static, 2026-05-25):
- `scripts/generate-rust-harness-fixtures.ts` delegates to
  `runx-harness-fixture-oracles`; it no longer imports contract canonicalization
  helpers.
- `scripts/verify-fast.mjs` builds the Rust oracle and exports
  `RUNX_HARNESS_FIXTURE_ORACLE_BIN`; `package.json` keeps
  `fixtures:harness:check` on that generator.
- `pnpm fixtures:harness:check` passed on 2026-05-25.
- `git diff --name-only -- fixtures/harness` returned no changed harness
  fixture/oracle files.

## Origin

A+ roadmap (2026-05-24), step 2. The TS↔Rust canonicalization divergence is the
deepest cross-language inconsistency and the root of the worst silent-drift bug
class. Captured during the contract-spine inversion work.
