---
spec_version: '2.0'
task_id: heavy-test-suite-gating
created: '2026-05-24T00:00:00Z'
updated: '2026-05-24T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: low
---

# Make the heavy test suite reliable and gated

## Current State

Status: draft
Current phase: planning
Next: harden
Reason: A+ roadmap step 5. The full `cargo test --workspace --all-features` and
the heavy graph integration tests are slow and flake on the debug `runx` binary's
cold start under parallel load unless `RUNX_KERNEL_EVAL_BIN` (and the parser/CLI
eval binaries) are set. Because of that flakiness the thorough suite is NOT part
of the enforced gate (`verify:fast` runs a fast subset), so a regression can slip
the fast gate and only surface in a slow local run.
Blockers: none.

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

- [ ] `dod1` The heavy/subprocess suites run without cold-start flakiness because
  the eval binary is prebuilt and the env vars are provisioned by the harness.
- [ ] `dod2` `cargo test --workspace` (or the defined superset) is part of the
  enforced gate, not just a manual run.
- [ ] `dod3` Two consecutive full runs are green with no flaky retries.

## Origin

A+ roadmap (2026-05-24), step 5. The flakiness was hit repeatedly during the
contract-spine work; the kernel-parity timeout was patched with a 30s test
timeout but the underlying cold-start provisioning gap remains.
