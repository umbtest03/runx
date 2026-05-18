---
spec_version: '2.0'
task_id: rust-runx-cli-placeholder
created: '2026-05-15T13:05:00Z'
updated: '2026-05-15T13:05:00Z'
status: draft
harden_status: not_run
size: small
risk_level: low
---

# Rust runx-cli placeholder crate

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld harden rust-runx-cli-placeholder`
Latest runner update: none
Review gate: not_started

## Summary

Add the initial Cargo package for runx CLI distribution. The crates.io package
name is `runx-cli` because `runx` is already taken by an unrelated crate, but
the installed binary must be named `runx`.

This is a placeholder/distribution crate, not a rewrite of the CLI runtime. It
delegates to the authoritative npm CLI by default and provides a local JS
entrypoint override for development.

## Context

CWD: `.`

Packages:
- `crates/runx-cli`
- `@runxhq/cli`

Files impacted:
- `crates/Cargo.toml`
- `crates/README.md`
- `crates/rustfmt.toml`
- `crates/runx-cli/Cargo.toml`
- `crates/runx-cli/README.md`
- `crates/runx-cli/src/lib.rs`
- `crates/runx-cli/src/launcher.rs`
- `crates/runx-cli/src/main.rs`
- `.gitignore`
- `.github/workflows/ci.yml`

Invariants:
- The npm package `@runxhq/cli` remains the authoritative implementation.
- The Cargo package name is `runx-cli`; the binary name is `runx`.
- The launcher must not implement runx runtime behavior, parse skill contracts,
  execute MCP, write receipts, or duplicate TypeScript CLI semantics.
- Default delegation uses the latest npm CLI unless explicitly pinned.
- Development delegation through `RUNX_JS_BIN` must execute a local JS entrypoint
  through Node without shell interpolation.

Related docs:
- `docs/trusted-kernel-package-truth.md`
- `plans/runx.md`
- `crates/README.md`
- `rust-cli-feature-parity-matrix`

## Objectives

- Create a modern Rust 2024 Cargo workspace under `crates/`.
- Create a publishable `runx-cli` package that installs a `runx` binary.
- Delegate by default to `npm exec --yes --package @runxhq/cli@latest -- runx`.
- Support `RUNX_NPM_PACKAGE` for pinned npm CLI versions.
- Support `RUNX_JS_BIN` for local checkout development.
- Add unit coverage for launcher planning without spawning npm/node.
- Add CI checks for formatting, clippy, tests, and packaging if not already
  present.

## Scope

In scope:
- Cargo workspace metadata.
- Thin launcher crate.
- Dependency-free or near dependency-free launcher implementation.
- Basic launcher tests.
- Cargo package metadata and README.
- Rust CI check wiring.

Out of scope:
- Rust implementation of `runx` command semantics.
- Kernel parity, policy, state-machine, runtime-local, MCP, A2A, receipt, or
  provider adapter ports.
- Any claim that the Cargo binary is feature-equivalent to the npm CLI.
- Publishing to crates.io.
- Replacing npm distribution.

## Dependencies

- crates.io package name `runx-cli` is available at planning time.
- The exact `runx` crate name is unavailable, so this spec intentionally avoids
  claiming it.
- Rust toolchain is not installed in the current local environment; final
  validation may need CI or a machine with Rust installed.

## Assumptions

- Users who install through Cargo accept requiring Node.js/npm until the Rust
  runtime becomes real.
- `@runxhq/cli@latest` is the desired default for the placeholder crate.
- Pinning remains possible through `RUNX_NPM_PACKAGE` for reproducibility.
- Keeping the launcher dependency-free is preferable for supply-chain and
  package-review simplicity.

## Touchpoints

- Cargo package metadata.
- Binary name collision and PATH behavior.
- npm package invocation.
- local JS development override.
- CI Rust setup.
- `.gitignore` for Cargo `target/`.

## Risks

- Medium: users may assume `cargo install runx-cli` gives a self-contained
  native implementation. README and shim help must be explicit that Node/npm are
  still required.
- Medium: defaulting to latest npm CLI favors freshness over reproducibility.
  Pinning through `RUNX_NPM_PACKAGE` must be documented.
- Low: Cargo package checks cannot be locally verified without Rust installed.

## Acceptance

Profile: standard

Definition of done:
- [ ] `dod1` `crates/runx-cli` exists and installs a binary named `runx`.
- [ ] `dod2` launcher defaults to `@runxhq/cli@latest`.
- [ ] `dod3` launcher planning is unit-tested without process execution.
- [ ] `dod4` README accurately describes placeholder status and runtime
  requirements.
- [ ] `dod5` Rust checks are documented and wired into CI or explicitly noted
  as pending if Rust is unavailable locally.
- [ ] `dod6` The crate docs do not imply native feature parity; they point to
  the CLI feature-parity matrix for any future rewrite.

Validation:
- [ ] `v1` command - Cargo package metadata is valid.
  - Command: `cargo metadata --format-version 1 --no-deps`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v2` command - Rust launcher checks pass.
  - Command: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo package -p runx-cli`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v3` command - default latest npm package is visible in source and docs.
  - Command: `rg -n '@runxhq/cli@latest' crates/runx-cli`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v4` command - binary name is `runx`.
  - Command: `rg -n 'name = "runx"' crates/runx-cli/Cargo.toml`
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
- [ ] `v6` command - placeholder docs avoid feature-parity claims.
  - Command: `! rg -n 'self-contained|native implementation|feature.?equivalent|drop-in replacement' crates/runx-cli/README.md crates/README.md`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none

## Phase 1: Cargo workspace and launcher

Goal: Add the minimal Cargo workspace and launcher crate.

Status: pending
Dependencies: none

Changes:
- `crates/Cargo.toml` (all, exclusive) - Define Rust 2024 workspace, resolver,
  MSRV, package metadata, and shared lints.
- `crates/rustfmt.toml` (all, exclusive) - Set formatting defaults.
- `crates/runx-cli/Cargo.toml` (all, exclusive) - Define package metadata and
  binary named `runx`.
- `crates/runx-cli/src/main.rs` (all, exclusive) - Implement process execution
  boundary only.
- `crates/runx-cli/src/lib.rs` (all, exclusive) - Export testable launcher
  planning code.
- `crates/runx-cli/src/launcher.rs` (all, exclusive) - Plan delegation to npm
  or local JS entrypoint and test the decision logic.

Acceptance:
- [ ] `ac1_1` command - package installs a `runx` binary by metadata.
  - Command: `rg -n 'name = "runx"' crates/runx-cli/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac1_2` command - launcher has no third-party dependencies.
  - Command: `! rg -n '^\\[dependencies\\]|clap|anyhow|tokio|reqwest|rmcp' crates/runx-cli/Cargo.toml crates/runx-cli/src`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac1_3` command - launcher defaults to latest npm CLI.
  - Command: `rg -n '@runxhq/cli@latest' crates/runx-cli/src crates/runx-cli/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Documentation and CI

Goal: Make placeholder status and validation explicit.

Status: pending
Dependencies: Phase 1

Changes:
- `crates/README.md` (all, exclusive) - Document Rust workspace commands and
  current placeholder status.
- `crates/runx-cli/README.md` (partial, exclusive) - Document Cargo install,
  Node/npm requirement, `RUNX_NPM_PACKAGE`, `RUNX_JS_BIN`, and shim flags.
- `.gitignore` (partial, shared) - Ignore Cargo `target/`.
- `.github/workflows/ci.yml` (partial, shared) - Add Rust check steps if this
  repo wants CI coverage immediately.

Acceptance:
- [ ] `ac2_1` command - README states Node/npm requirement.
  - Command: `rg -n 'Node\\.js|npm|@runxhq/cli@latest|RUNX_NPM_PACKAGE|RUNX_JS_BIN' crates/runx-cli/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2_2` command - Cargo target is ignored.
  - Command: `rg -n 'target/' .gitignore`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2_3` command - CI contains Rust checks or docs state local-only.
  - Command: `rg -n 'cargo fmt|cargo clippy|cargo test|runx-cli|local-only' .github/workflows/ci.yml crates/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

Strategy: per_phase

Commands:
- Remove `crates/runx-cli`.
- Remove `crates/Cargo.toml`, `crates/README.md`, and `crates/rustfmt.toml` if
  no other Rust crates remain.
- Revert `.gitignore` and CI workflow changes introduced by this spec.

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

Estimated effort hours: 2
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- cargo
- cli
- placeholder

## Origin

Source:
- user requested the `runx-cli` placeholder crate.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- precedes: rust-kernel-parity-fixtures
- precedes: rust-cli-feature-parity-matrix

## Harden Rounds

- none

## Planning Log

- 2026-05-15T13:05:00Z: Drafted as Cargo placeholder crate plan.
