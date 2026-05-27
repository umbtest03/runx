---
spec_version: '2.0'
task_id: runx-runtime-test-gate-dx-v1
created: '2026-05-27T12:58:25Z'
updated: '2026-05-27T13:39:27Z'
status: blocked
harden_status: passed
size: large
risk_level: medium
---

# Runtime test and verification gate DX

## Current State

Status: blocked
Current phase: phase4
Next: repair
Reason: phase phase4 acceptance failed
Blockers: phase phase4 acceptance failed
Allowed follow-up command: `scafld handoff runx-runtime-test-gate-dx-v1`
Latest runner update: 2026-05-27T13:39:27Z
Review gate: not_started

## Summary

Make the Rust runtime test surface and `verify:fast` gate behave like trustworthy
developer tools instead of wrapper-dependent orchestration. Runtime integration
tests must self-provision their fixture signing context and eval binaries when
run directly through Cargo/nextest. The boundary check must scan source, not
stale built artifacts. The fast verification script should preserve signal from
independent checks instead of hiding later failures behind the first red step.

This is a clean infrastructure cutover: no compatibility shims, no fallback test
modes, and no weakening of gates. The existing active
`runx-rust-95-release-readiness` spec is explicitly out of scope.

## Objectives

- Make `runx-runtime` tests that need production-like signing or eval binaries
  self-provision through shared test support, so direct Cargo/nextest invocations
  are understandable and reproducible.
- Harden `scripts/check-boundaries.mjs` against stale `.build/` artifacts and
  emit source-owned findings only.
- Refactor `scripts/verify-fast.mjs` so independent JS/Rust/package checks report
  as separate steps and continue where safe, while preserving a nonzero final
  exit if any required check fails.
- Record the concurrent-agent operational rule in the spec result: worktree
  isolation is recommended for separate agents, but this task uses targeted edits
  in the current checkout because the operator explicitly allowed collisions.
- Audit the cold compile floor and doctest opportunity only far enough to avoid
  accidental dependency broadening; do not perform a dependency swap inside this
  spec.

## Scope

- In scope:
  - `crates/runx-runtime/tests/support.rs` and runtime tests that should consume
    shared fixture signing/runtime helpers.
  - `scripts/check-boundaries.mjs` and focused boundary regression coverage.
  - `scripts/verify-fast.mjs`, limited to check orchestration/reporting.
  - Package scripts needed to expose the new focused checks.
  - Minimal docs/spec notes only when they describe new commands or gates.
- Out of scope:
  - `.scafld/specs/active/runx-rust-95-release-readiness.md`.
  - Changing product behavior, receipt schemas, harness spine contracts, or
    runtime architecture ownership.
  - Replacing reqwest/tokio/rustls/rmcp or changing feature defaults.
  - Merging OSS/cloud repository boundaries or renaming `X.yaml`.
  - Broad decomposition of runtime modules.

## Dependencies

- `test-surface-build-consolidation` remains active and owns the larger CI
  consolidation story. This spec fixes the observed 11 runtime-test
  self-provisioning failures underneath that plan.
- Existing dirty files may be edited with targeted patches, per operator
  instruction. Do not revert or stage unrelated changes.

## Assumptions

- Direct `cargo test` or `cargo nextest` on `runx-runtime` should not require the
  JS `verify:fast` wrapper to inject signing keys or binary paths.
- Test fixture signing keys are non-secret deterministic test material and must
  remain confined to test support.
- Boundary checks should ignore generated/built output directories regardless of
  whether a cache restores them before the build step.

## Touchpoints

- `crates/runx-runtime/tests/support.rs`
- `crates/runx-runtime/tests/{skill_run,skill_issue_intake,skill_issue_to_pr,local_credential_provision,hello_graph,mcp_server}.rs`
- `scripts/check-boundaries.mjs`
- `scripts/verify-fast.mjs`
- `package.json`
- focused tests for the boundary script, if absent

## Risks

- Test support can accidentally mask production signing enforcement if applied
  inside production code. Mitigation: keep helpers under `crates/runx-runtime/tests`.
- A fan-out verification script can overload the Rust linker/eval binary if it
  parallelizes heavy gates. Mitigation: only fan out light independent JS checks;
  keep Rust binary builds and Rust-heavy checks serialized.
- Boundary script fixtures can become another stale mirror. Mitigation: test the
  behavior by creating temporary source/build files during the test, not by
  checking in generated artifacts.

## Acceptance

Profile: strict

Validation:
- `cargo nextest run --manifest-path crates/Cargo.toml -p runx-runtime --all-features --test integration -- skill_run local_credential_provision skill_issue_intake skill_issue_to_pr`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --all-features --test integration -- skill_run local_credential_provision skill_issue_intake skill_issue_to_pr`
- `pnpm boundary:check`
- focused boundary regression test
- `pnpm verify:fast`
- `scafld review runx-runtime-test-gate-dx-v1 --provider claude`

## Phase 1: Runtime test self-provisioning

Status: completed
Dependencies: none

Objective: Runtime tests that need signing/eval context pass under direct Cargo

Changes:
- Add shared runtime test helpers for fixture signing env, signed runtime options, and eval binary discovery/provisioning if a test needs a binary path.
- Replace local duplicated signing env construction in runtime tests with shared helpers.
- Keep the one negative test that checks missing production signing env explicit; do not globally inject signing into process env.

Acceptance:
- [x] `p1_ac1` command - Runtime nextest subset self-provisions.
  - Command: `cargo nextest run --manifest-path crates/Cargo.toml -p runx-runtime --all-features --test integration -- skill_run local_credential_provision skill_issue_intake skill_issue_to_pr`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `p1_ac2` command - Runtime cargo-test subset self-provisions.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --all-features --test integration -- skill_run local_credential_provision skill_issue_intake skill_issue_to_pr`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Boundary source scan hardening

Status: completed
Dependencies: phase1

Objective: Boundary checks report source-owned violations and ignore restored

Changes:
- Keep `.build`, `dist`, `target`, and `target-*` ignored by boundary walks.
- Add a focused regression test that creates a forbidden term under `.build/` and verifies `boundary:check` ignores it while still rejecting the same term under an active source root.
- Improve any failure text needed to point at the owning source file.

Acceptance:
- [x] `p2_ac1` command - Boundary check passes in the real workspace.
  - Command: `pnpm boundary:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-12
- [x] `p2_ac2` command - Boundary build-artifact regression passes.
  - Command: `pnpm test:boundary`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13

## Phase 3: verify:fast signal fan-out

Status: completed
Dependencies: phase2

Objective: `verify:fast` keeps independent check results visible and only

Changes:
- Refactor `scripts/verify-fast.mjs` into named steps with a final summary.
- Run safe independent JS checks with parallel reporting where they do not share generated output or heavy Rust linker work.
- Keep Rust binary builds and Rust-heavy checks serialized.
- Continue executing independent checks after a failure when safe; exit nonzero at the end if any required step failed.

Acceptance:
- [x] `p3_ac1` command - Fast gate passes with the new orchestrator.
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `p3_ac2` command - Verify package and Rust-heavy checks stay serialized.
  - Command: `pnpm verify:fast:plan-check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24

## Phase 4: Review and completion

Status: blocked
Dependencies: phase3

Objective: Record evidence and run the requested Claude review gate.

Changes:
- none

Acceptance:
- [x] `p4_ac1` command - Rust style remains green.
  - Command: `pnpm rust:style`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29
- [x] `p4_ac2` command - TypeScript remains green.
  - Command: `pnpm typecheck`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30
- [ ] `p4_ac3` command - Claude review gate passes.
  - Command: `scafld review runx-runtime-test-gate-dx-v1 --provider claude`
  - Expected kind: `exit_code_zero`
  - Status: fail
  - Evidence: exit code was 4
  - Source event: entry-31

## Rollback

- Revert the test-support helper changes and the tests that consume them.
- Restore `scripts/verify-fast.mjs` to the prior serial command loop if the new
  orchestrator hides output or causes false failures.
- Remove the boundary regression test and script changes if it proves flaky.

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

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-27T12:59:34Z
Ended: 2026-05-27T13:00:28Z

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/tests/support.rs:15
  - Result: passed
  - Evidence: Runtime fixture signing helpers already exist in test support;
- command audit
  - Grounded in: code:scripts/verify-fast.mjs:12
  - Result: passed
  - Evidence: `verify:fast` currently owns Rust binary prebuild and env
- scope/migration audit
  - Grounded in: code:scripts/check-boundaries.mjs:33
  - Result: passed
  - Evidence: Boundary scanning already ignores `.build`, `dist`, and `target`;
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: The active test-surface spec records the 11 runtime failures as
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback is per surface: test support, boundary regression, and
- design challenge
  - Grounded in: code:crates/runx-runtime/src/execution/runner.rs:66
  - Result: passed
  - Evidence: Production runtime still reads signing config from explicit env;

Issues:
- none


## Planning Log

- none
