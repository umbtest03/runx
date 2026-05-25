---
spec_version: '2.0'
task_id: heavy-test-suite-gating
created: '2026-05-24T00:00:00Z'
updated: '2026-05-25T16:51:19+10:00'
status: completed
harden_status: not_run
size: small
risk_level: low
---

# Make the heavy test suite reliable and gated

## Current State

Status: completed
Current phase: final
Next: done
Reason: `scripts/verify-fast.mjs` now builds the Rust CLI and harness fixture
oracle once, exports `RUNX_KERNEL_EVAL_BIN` / `RUNX_PARSER_EVAL_BIN` /
`RUNX_RUST_CLI_BIN` / `RUNX_HARNESS_FIXTURE_ORACLE_BIN`, and includes
`fixtures:harness:check`. CI also runs `pnpm test:heavy:graph` and
`cargo test --workspace --all-features`. Two consecutive heavy graph runs are
recorded green after the eval-binary/oracle provisioning fix.
Blockers: none for this gating slice.
Allowed follow-up command: `none`
Latest runner update: 2026-05-25T16:51:19+10:00
Review gate: pass

## Summary

Build the eval binary once and point the suites at it (default
`RUNX_KERNEL_EVAL_BIN` / `RUNX_PARSER_EVAL_BIN` / `RUNX_RUST_CLI_BIN` in the test
harness), remove the cold-start flakiness, then add the heavy suite to the
enforced gate so the thorough run is actually required.

## Objectives

- A harness step builds the binary and exports the eval-binary env vars so
  subprocess-backed and kernel-parity tests do not cold-start a debug binary
  under load.
- The previously-flaky heavy suite runs deterministically.
- Once deterministic, the heavy suite (or a defined superset beyond
  `test:fast`) is added to CI so it gates merges.

## Scope

In scope: `scripts/verify-fast.mjs` / the verify pipeline, the vitest configs and
any test harness that shells out to the binary, CI workflow wiring, the
`RUNX_*_EVAL_BIN` provisioning.

Out of scope: changing test assertions or fixtures.

## Acceptance

- [x] `dod1` The heavy/subprocess suites run without cold-start flakiness because
  the eval binary is prebuilt and the env vars are provisioned by the harness.
  - Command: `pnpm test:heavy:graph`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 full heavy graph suite passed after the shared
    eval-binary/oracle provisioning was wired.
- [x] `dod2` `cargo test --workspace` (or the defined superset) is part of the
  enforced gate, not just a manual run.
  - Command: `rg -n "pnpm test:heavy:graph|cargo test --workspace --all-features" .github/workflows/ci.yml`
  - Expected kind: `reviewed_output`
  - Status: reviewed
  - Evidence: CI contains both the heavy graph suite and workspace all-features
    cargo gate.
- [x] `dod3` Two consecutive full runs are green with no flaky retries.
  - Command: `pnpm test:heavy:graph`
  - Expected kind: `exit_code_zero`
  - Status: passed twice
  - Evidence: 2026-05-25 full heavy graph suite passed twice consecutively; the
    second recorded run passed 13 files / 66 tests in 65.90s.

Evidence (static, 2026-05-25):
- `scripts/verify-fast.mjs` builds `runx` and `runx-harness-fixture-oracles`,
  then exports the eval/oracle env vars for the fast gate.
- `.github/workflows/ci.yml` runs `pnpm test:heavy:graph` and
  `cargo test --workspace --all-features`.
- Two consecutive `pnpm test:heavy:graph` runs were recorded green on
  2026-05-25.

## Origin

A+ roadmap (2026-05-24), step 5. The flakiness was hit repeatedly during the
contract-spine work; the kernel-parity timeout was patched with a 30s test
timeout but the underlying cold-start provisioning gap remains.
