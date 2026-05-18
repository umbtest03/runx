---
spec_version: '2.0'
task_id: rust-parity-ci-governance
created: '2026-05-15T12:51:06Z'
updated: '2026-05-16T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust parity CI governance

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: `rust-kernel-parity-fixtures`, `rust-state-machine-parity`,
`rust-policy-parity`, and `rust-cli-feature-parity-matrix` should complete
first
Allowed follow-up command: `scafld harden rust-parity-ci-governance`
Latest runner update: none
Review gate: not_started

## Summary

Promote Rust kernel parity from local experiment to governed advisory CI
signal after the fixture, state-machine, and policy phases exist. This task
decides what remains advisory, how blocking promotion is handed off, and how
future runtime/CLI rewrites must prove they preserve one-to-one TypeScript
behavior before cutover.

The goal is to improve current TypeScript development, not to make Rust a
second authority prematurely.

Important: `.github/workflows/ci.yml` already installs the Rust toolchain and
runs `cargo fmt`, `cargo clippy --workspace`, `cargo test --workspace`, and
`cargo package -p runx-cli`. This spec does not introduce Rust to CI for the
first time. It adds:
- a wrapper script with explicit missing-cargo diagnostics,
- `cargo-deny` configuration check,
- public-API snapshot check via `cargo-public-api`,
- the staged advisory policy and handoff to `rust-kernel-blocking-promotion`.

This spec depends on `oss/docs/rust-kernel-architecture.md`, especially
sections 10 (boundary enforcement) and 12 (dual-tree maintenance policy).

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-cli`
- `crates/runx-core`

Files impacted:
- `.github/workflows/ci.yml`
- `package.json`
- `scripts/check-rust-kernel-parity.mjs`
- `crates/README.md`
- `crates/deny.toml` (created earlier; CI references it)
- `crates/runx-core/api-snapshot.txt`
- `docs/rust-kernel-architecture.md`
- `docs/trusted-kernel-package-truth.md`
- `docs/api-surface.md`
- `CONTRIBUTING.md`
- `README.md`

Invariants:
- TypeScript remains authoritative until a separate cutover spec changes a
  consumer.
- The CLI feature-parity matrix is the required oracle for any Rust CLI or
  runtime cutover; kernel parity alone is not enough.
- CI should block on stable fixture parity only after the fixture set is
  deliberate and small enough to maintain.
- Rust checks must not require network access except during normal toolchain
  installation or dependency resolution in CI.
- Cargo package publication remains separate from kernel-authority cutover.
- The advisory phase lasts until 5 clean kernel-touching PRs land in green,
  per arch doc section 12. Calendar time is not the trigger.

Related docs:
- `docs/rust-kernel-architecture.md` (prerequisite reading)
- `docs/trusted-kernel-package-truth.md`
- `crates/README.md`
- `AGENTS.md`

## Objectives

- Add a single documented Rust parity validation command or script.
- Extend the existing CI Rust step with `cargo-deny` and public-API snapshot
  diffs.
- Document advisory vs blocking stages for Rust parity, with an explicit
  promotion trigger.
- Define the next cutover bar before any runtime or CLI logic depends on Rust,
  including the full CLI feature-parity matrix.
- Keep npm CLI release behavior unchanged.

## Scope

In scope:
- A wrapper script `scripts/check-rust-kernel-parity.mjs` for local and CI
  parity checks with explicit missing-cargo diagnostics.
- Extension of the existing CI Rust step to run `cargo-deny` and a
  `cargo-public-api` snapshot diff.
- Public-API snapshot file at `crates/runx-core/api-snapshot.txt`. Generated
  the first time the spec runs and committed; CI diffs against it.
- Docs for how TS developers update fixtures and validate Rust parity,
  including the advisory-to-blocking promotion criteria and the follow-up
  `rust-kernel-blocking-promotion` spec.
- Docs for how Rust CLI/runtime candidates consume the CLI feature-parity
  matrix before replacing npm CLI behavior.
- Release gate language for future Rust-backed kernel consumers.

Out of scope:
- Publishing `runx-core` or `runx-cli`.
- Replacing npm CLI implementation.
- Porting parser, receipts, runtime-local, MCP, A2A, or provider adapters.
- Adding N-API, WASM, or FFI bindings.
- Authority-proof and public-work re-export parity (separate follow-up spec).
- Removing `continue-on-error` from Rust parity checks. Blocking promotion is
  owned by `rust-kernel-blocking-promotion` after 5 clean kernel-touching PRs.

## Dependencies

- `rust-kernel-parity-fixtures` completed.
- `rust-state-machine-parity` completed.
- `rust-policy-parity` completed.
- `rust-cli-feature-parity-matrix` completed.
- Rust toolchain setup exists in CI.

## Assumptions

- `runx-core` tests are fast enough to run in every OSS CI invocation.
- `runx-cli` package checks can remain in CI but are not part of the trusted
  kernel parity proof.
- A later spec can add `runx-parser-parity`, `runx-receipt-parity`,
  `runx-policy-authority-proof-parity`, or `runx-runtime-rust-spike` if the
  pure kernel phases prove useful.
- `cargo-deny` and `cargo-public-api` are stable enough to depend on in CI;
  if either becomes flaky, the workflow can pin a specific version.
- Phase A (advisory) lasts until 5 clean kernel-touching PRs (PRs touching
  `packages/core/src/state-machine/` or `packages/core/src/policy/`) land
  in green. The decision to promote is made by re-running this spec's
  Phase 4 with the promotion check.

## Touchpoints

- GitHub Actions CI.
- Local developer commands.
- Package scripts.
- Contributor docs.
- Trusted kernel documentation.
- Cargo workspace docs.

## Risks

- Medium: making Rust parity blocking too early can slow TypeScript kernel
  work. Mitigated by the explicit advisory phase.
- Medium: CI can become slower or flaky if it fetches too much Cargo state on
  every run. Mitigated by caching `~/.cargo/registry`, `~/.cargo/git`, and
  `target/` keyed on `Cargo.lock`.
- Medium: `cargo-public-api` snapshot churn can be noisy during early parity
  work. Mitigated by making snapshot diff advisory in Phase A.
- Low: developers without Rust installed need clear local fallback guidance.
- Low: `cargo-deny` advisory database changes (new vulnerability advisories)
  can fail builds. The workflow opts into bans/licenses/sources checks only,
  not the advisories check, to avoid that failure mode.

## Acceptance

Profile: standard

Definition of done:
- [ ] `dod1` A single documented command validates Rust kernel parity locally.
- [ ] `dod2` CI runs Rust formatting, clippy, tests, package checks,
  `cargo-deny`, repository Rust style guard, and public-API snapshot diff.
- [ ] `dod3` Docs explain that TS remains the source of truth and how parity
  fixtures should be updated.
- [ ] `dod4` Future Rust runtime/CLI cutover requirements are documented.
- [ ] `dod5` The advisory-to-blocking promotion criterion is named in
  `docs/rust-kernel-architecture.md` and `CONTRIBUTING.md`.
- [ ] `dod6` CI/docs distinguish kernel parity checks from full CLI feature
  parity checks.

Validation:
- [ ] `v1` command - Rust workspace checks pass.
  - Command: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo package -p runx-cli`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v2` command - Rust kernel parity wrapper passes.
  - Command: `node scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v3` command - cargo-deny passes.
  - Command: `cargo deny --manifest-path crates/Cargo.toml check bans licenses sources`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v4` command - public-API snapshot matches.
  - Command: `cargo public-api --manifest-path crates/runx-core/Cargo.toml diff crates/runx-core/api-snapshot.txt`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v5` command - TypeScript fast verification remains green.
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v6` command - Rust style guard passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v7` command - docs name advisory/blocking policy.
  - Command: `rg -n 'advisory|blocking|source of truth|kernel parity|check-rust-kernel-parity' README.md CONTRIBUTING.md docs crates scripts package.json .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v8` command - docs name the CLI feature-parity matrix as cutover gate.
  - Command: `rg -n 'cli feature parity|feature-parity matrix|one-to-one|No npm-to-Rust CLI cutover' README.md CONTRIBUTING.md docs crates .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none

## Phase 1: Local parity command

Goal: Give developers one command for Rust parity without memorizing Cargo
subcommands. Distinct from the existing CI Rust step because it adds
`cargo-deny` and the public-API diff.

Status: pending
Dependencies: `rust-kernel-parity-fixtures`, `rust-state-machine-parity`,
`rust-policy-parity`

Changes:
- `scripts/check-rust-kernel-parity.mjs` (all, exclusive) - Run cargo fmt,
  clippy, tests, `scripts/check-rust-crate-graph.mjs`,
  `scripts/check-rust-core-style.mjs`, cargo-deny, and
  public-API diff with clear missing-Rust diagnostics and an
  `--install-tools` hint when `cargo-deny` or `cargo-public-api` are missing.
- `crates/runx-core/api-snapshot.txt` (all, exclusive) - Initial snapshot of
  the public API generated by `cargo public-api --manifest-path
  crates/runx-core/Cargo.toml > crates/runx-core/api-snapshot.txt`.
- `package.json` (partial, shared) - Add a script alias matching existing
  script style (`rust:check` or `kernel:parity`).
- `crates/README.md` (partial, shared) - Document the local command and the
  optional tools (`cargo install cargo-deny cargo-public-api`).

Acceptance:
- [ ] `ac1_1` command - parity wrapper passes.
  - Command: `node scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac1_2` command - missing Cargo message is explicit in script source.
  - Command: `rg -n 'cargo.*not.*installed|Install Rust|rustup|missing Cargo' scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac1_3` command - script covers style guard, cargo-deny, and public-API diff.
  - Command: `rg -n 'check-rust-crate-graph|check-rust-core-style|cargo deny|cargo public-api|cargo-public-api' scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Advisory CI integration

Goal: Extend the existing CI Rust step. New checks run as advisory (non-blocking)
in Phase A.

Status: pending
Dependencies: Phase 1

Changes:
- `.github/workflows/ci.yml` (partial, shared) - Add steps for
  `scripts/check-rust-crate-graph.mjs`, `scripts/check-rust-core-style.mjs`,
  `cargo-deny`, and `cargo public-api`
  diff against the snapshot. Mark them with `continue-on-error: true` for the
  advisory phase. Add caching for
  `~/.cargo/registry`, `~/.cargo/git`, and `crates/target` keyed on
  `crates/Cargo.lock`. Install `cargo-deny` and `cargo-public-api` via
  cached toolchain action or `cargo install`.

Acceptance:
- [ ] `ac2_1` command - CI workflow includes the new checks.
  - Command: `rg -n 'cargo deny|cargo-deny|cargo public-api|cargo-public-api|check-rust-kernel-parity' .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2_2` command - new checks are explicitly advisory in this phase.
  - Command: `rg -n 'continue-on-error|advisory' .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2_3` command - full local validation remains green.
  - Command: `pnpm verify:fast && node scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Governance docs

Goal: Make future Rust migration rules explicit before runtime or CLI cutover
work.

Status: pending
Dependencies: Phase 2

Changes:
- `docs/rust-kernel-architecture.md` (partial, shared) - Confirm and refine
  section 12 (dual-tree maintenance policy) based on what the wrapper script
  ended up doing.
- `docs/trusted-kernel-package-truth.md` (partial, shared) - Document parity
  status, TS authority, future cutover requirements, and the full CLI
  feature-parity matrix.
- `CONTRIBUTING.md` (partial, shared) - Document local Rust checks for kernel
  changes and the optional `cargo install` commands.
- `README.md` (partial, shared) - Mention Cargo launcher and Rust parity only
  if it improves user/developer clarity. Otherwise skip.

Acceptance:
- [ ] `ac3_1` command - docs describe the cutover bar.
  - Command: `rg -n 'cutover|source of truth|kernel parity|feature parity|Rust parity|TypeScript' docs CONTRIBUTING.md README.md`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac3_2` command - advisory-to-blocking criterion is named.
  - Command: `rg -n 'advisory.*phase|5 clean|five clean|promote.*blocking|Phase A|Phase B' docs CONTRIBUTING.md`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 4: Blocking-promotion handoff

Goal: Leave the new Rust checks advisory and create the handoff to the
separate blocking-promotion spec. This phase does not flip CI to blocking.

Status: pending
Dependencies: Phase 3

Changes:
- `.scafld/specs/drafts/rust-kernel-blocking-promotion.md` (all, exclusive) -
  Add the follow-up spec that waits for 5 clean kernel-touching PRs and then
  removes `continue-on-error` from the Rust parity checks.
- `docs/rust-kernel-architecture.md` (partial, shared) - Point Phase B
  promotion to `rust-kernel-blocking-promotion`, not back into this spec.
- `CONTRIBUTING.md` (partial, shared) - Keep local/CI Rust parity docs in
  advisory language and name the follow-up promotion spec.

Acceptance:
- [ ] `ac4_1` command - blocking-promotion spec exists and validates.
  - Command: `scafld validate --json rust-kernel-blocking-promotion | jq -e '.result.valid == true'`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac4_2` command - docs name the promotion handoff.
  - Command: `rg -n 'rust-kernel-blocking-promotion|5 clean kernel-touching PRs|five clean kernel-touching PRs' docs CONTRIBUTING.md`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac4_3` command - CI remains advisory in this spec.
  - Command: `rg -n 'continue-on-error: true' .github/workflows/ci.yml | rg -qE 'cargo-deny|cargo public-api|check-rust-kernel-parity'`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

Strategy: per_phase

Commands:
- Remove `scripts/check-rust-kernel-parity.mjs` and
  `crates/runx-core/api-snapshot.txt`.
- Revert the new Rust CI workflow steps (`cargo-deny`, public-API diff).
  Do not remove pre-existing Rust CI steps; they were not introduced by
  this spec.
- Revert docs and package script changes introduced by this spec.

## Review

Status: not_started
Verdict: none
Timestamp: none
Review rounds: none
Reviewer mode: none
Reviewer session: none
Round status: none
Override applied: none
Override reason: none
Override confirmed at: none
Reviewed head: none
Reviewed dirty: none
Reviewed diff: none
Blocking count: none
Non-blocking count: none

Findings:
- none

Passes:
- none

## Self Eval

Status: not_started
Completeness: none
Architecture fidelity: none
Spec alignment: none
Validation depth: none
Total: none
Second pass performed: none

Notes:
none

Improvements:
- none

## Deviations

- none

## Metadata

Estimated effort hours: 5
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- ci
- parity
- governance

## Origin

Source:
- user requested phased scafld plans for Rust kernel parity.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- depends_on: rust-kernel-parity-fixtures
- depends_on: rust-state-machine-parity
- depends_on: rust-policy-parity
- depends_on: rust-cli-feature-parity-matrix

## Harden Rounds

- none

## Planning Log

- 2026-05-15T12:58:00Z: Drafted as governance phase after Rust kernel parity.
- 2026-05-15T13:30:00Z: Revised after architectural review. Clarified that
  Rust toolchain + workspace clippy/test/package is already in CI; this spec
  adds the parity-specific checks (`cargo-deny`, public-API snapshot diff).
  Added Phase 4 for the deferred advisory-to-blocking promotion. Pulled the
  advisory/blocking phasing into the invariants. Now depends on
  `docs/rust-kernel-architecture.md`. Estimate bumped from 3h to 5h.
- 2026-05-16T00:00:00Z: Independent review correction. Dropped the
  `--no-default-features` build step from the wrapper, CI, and validation
  in line with the std-default decision in the arch doc. Replaced
  calendar-based 2-week advisory soak with "5 clean kernel-touching PRs"
  promotion criterion. Updated arch doc section references after
  renumbering (boundary enforcement is now section 10; dual-tree
  maintenance is now section 12).
- 2026-05-17T00:30:00Z: Corrected the lifecycle shape after checking the
  installed scafld CLI. Phase 4 now creates the `rust-kernel-blocking-
  promotion` handoff and keeps CI advisory. The actual removal of
  `continue-on-error` is in the follow-up spec because completed scafld specs
  cannot be reopened with a `deviate` command.
