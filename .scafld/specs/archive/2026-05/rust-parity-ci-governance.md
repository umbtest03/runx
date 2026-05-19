---
spec_version: '2.0'
task_id: rust-parity-ci-governance
created: '2026-05-15T12:51:06Z'
updated: '2026-05-19T03:33:01Z'
status: completed
harden_status: not_run
size: medium
risk_level: medium
---

# Rust parity CI governance

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T03:33:01Z
Review gate: pass

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

Validation:
- [x] `v1` command - Rust workspace checks pass.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml --workspace --all-targets -- -D warnings && cargo test --manifest-path crates/Cargo.toml --workspace && cargo package --manifest-path crates/runx-cli/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `v2` command - Rust kernel parity wrapper passes.
  - Command: `node scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38
- [x] `v3` command - cargo-deny passes.
  - Command: `cargo deny --manifest-path crates/Cargo.toml check bans licenses sources`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-39
- [x] `v4` command - public-API snapshot matches.
  - Command: `node scripts/check-rust-kernel-parity.mjs --api-only`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-40
- [x] `v5` command - TypeScript fast verification remains green.
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-41
- [x] `v6` command - Rust style guard passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-42
- [x] `v7` command - docs name advisory/blocking policy.
  - Command: `rg -n 'advisory|blocking|source of truth|kernel parity|check-rust-kernel-parity' README.md CONTRIBUTING.md docs crates scripts package.json .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-43
- [x] `v8` command - docs name the CLI feature-parity matrix as cutover gate.
  - Command: `rg -n 'cli feature parity|feature-parity matrix|one-to-one|No npm-to-Rust CLI cutover' README.md CONTRIBUTING.md docs crates .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-44

## Phase 1: Local parity command

Status: completed
Dependencies: `rust-kernel-parity-fixtures`, `rust-state-machine-parity`

Objective: Complete this phase.

Changes:
- `scripts/check-rust-kernel-parity.mjs` (all, exclusive) - Run cargo fmt, clippy, tests, `scripts/check-rust-crate-graph.mjs`, `scripts/check-rust-core-style.mjs`, cargo-deny, and public-API diff with clear missing-Rust diagnostics and an `--install-tools` hint when `cargo-deny` or `cargo-public-api` are missing.
- `crates/runx-core/api-snapshot.txt` (all, exclusive) - Initial simplified snapshot of the public API generated by `cargo public-api --manifest-path crates/runx-core/Cargo.toml -sss > crates/runx-core/api-snapshot.txt`. Snapshot comparison is owned by `scripts/check-rust-kernel-parity.mjs --api-only` because current `cargo-public-api diff` supports crate versions, commits, and rustdoc JSON files, not text snapshots.
- `package.json` (partial, shared) - Add a script alias matching existing script style (`rust:check` or `kernel:parity`).
- `crates/README.md` (partial, shared) - Document the local command and the optional tools (`cargo install cargo-deny cargo-public-api`).

Acceptance:
- [x] `ac1_1` command - parity wrapper passes.
  - Command: `node scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac1_2` command - missing Cargo message is explicit in script source.
  - Command: `rg -n 'cargo.*not.*installed|Install Rust|rustup|missing Cargo' scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `ac1_3` command - script covers style guard, cargo-deny, and public-API diff.
  - Command: `rg -n 'check-rust-crate-graph|check-rust-core-style|cargo deny|cargo public-api|cargo-public-api' scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8

## Phase 2: Advisory CI integration

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `.github/workflows/ci.yml` (partial, shared) - Add steps for `scripts/check-rust-crate-graph.mjs`, `scripts/check-rust-core-style.mjs`, `cargo-deny`, and `cargo public-api` diff against the snapshot. Mark them with `continue-on-error: true` for the advisory phase. Add caching for `~/.cargo/registry`, `~/.cargo/git`, and `crates/target` keyed on `crates/Cargo.lock`. Install `cargo-deny` and `cargo-public-api` via cached toolchain action or `cargo install`.

Acceptance:
- [x] `ac2_1` command - CI workflow includes the new checks.
  - Command: `rg -n 'cargo deny|cargo-deny|cargo public-api|cargo-public-api|check-rust-kernel-parity' .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13
- [x] `ac2_2` command - new checks are explicitly advisory in this phase.
  - Command: `rg -n 'continue-on-error|advisory' .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `ac2_3` command - full local validation remains green.
  - Command: `pnpm verify:fast && node scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15

## Phase 3: Governance docs

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `docs/rust-kernel-architecture.md` (partial, shared) - Confirm and refine section 12 (dual-tree maintenance policy) based on what the wrapper script ended up doing.
- `docs/trusted-kernel-package-truth.md` (partial, shared) - Document parity status, TS authority, future cutover requirements, and the full CLI feature-parity matrix.
- `CONTRIBUTING.md` (partial, shared) - Document local Rust checks for kernel changes and the optional `cargo install` commands.
- `README.md` (partial, shared) - Mention Cargo launcher and Rust parity only if it improves user/developer clarity. Otherwise skip.

Acceptance:
- [x] `ac3_1` command - docs describe the cutover bar.
  - Command: `rg -n 'cutover|source of truth|kernel parity|feature parity|Rust parity|TypeScript' docs CONTRIBUTING.md README.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20
- [x] `ac3_2` command - advisory-to-blocking criterion is named.
  - Command: `rg -n 'advisory.*phase|5 clean|five clean|promote.*blocking|Phase A|Phase B' docs CONTRIBUTING.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21

## Phase 4: Blocking-promotion handoff

Status: completed
Dependencies: Phase 3

Objective: Complete this phase.

Changes:
- `.scafld/specs/drafts/rust-kernel-blocking-promotion.md` (all, exclusive) - Add the follow-up spec that waits for 5 clean kernel-touching PRs and then removes `continue-on-error` from the Rust parity checks.
- `docs/rust-kernel-architecture.md` (partial, shared) - Point Phase B promotion to `rust-kernel-blocking-promotion`, not back into this spec.
- `CONTRIBUTING.md` (partial, shared) - Keep local/CI Rust parity docs in advisory language and name the follow-up promotion spec.

Acceptance:
- [x] `ac4_1` command - blocking-promotion spec exists and validates.
  - Command: `scafld validate --json rust-kernel-blocking-promotion | jq -e '.result.valid == true'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-32
- [x] `ac4_2` command - docs name the promotion handoff.
  - Command: `rg -n 'rust-kernel-blocking-promotion|5 clean kernel-touching PRs|five clean kernel-touching PRs' docs CONTRIBUTING.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-33
- [x] `ac4_3` command - CI remains advisory in this spec.
  - Command: `rg -n 'continue-on-error: true.*(cargo-deny|cargo public-api|check-rust-kernel-parity)' .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-34

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

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Spec compliance is solid: wrapper script implements cargo + tool diagnostics, public-API snapshot diff, and is wired through `pnpm rust:check`. CI gains advisory `cargo-deny` + `cargo-public-api` via the wrapper with `continue-on-error: true`, cargo caches keyed on `crates/Cargo.lock`, and a separate `rust-kernel-blocking-promotion` follow-up spec exists with 5-PR trigger language. Docs (`rust-kernel-architecture.md`, `trusted-kernel-package-truth.md`, `CONTRIBUTING.md`, `README.md`, `crates/README.md`) name the advisory/blocking phasing and the CLI feature-parity matrix as the cutover oracle, and all listed task changes fall inside scope with no ambient drift. Two non-blocking smells worth fixing: the CI step adds `dtolnay/rust-toolchain@nightly` after `@stable` (which sets nightly as the default toolchain so the pre-existing `Rust checks` `cargo fmt`/`clippy -D warnings`/`test` would silently run on nightly), and `crates/README.md` instructs running the wrapper from `oss/crates/` with `node ../scripts/check-rust-kernel-parity.mjs` while the script uses CWD-relative paths (`crates/Cargo.toml`, `scripts/check-rust-crate-graph.mjs`) that only resolve from `oss/`. A minor inconsistency: the `--api-only` missing-tool branch omits the nightly-toolchain hint that `checkTooling` provides. None block the gate, but each is a real footgun for the next operator.

Attack log:
- `acceptance_evidence`: Re-walk every acceptance command against committed files (v1-v8, ac1-ac4 rg patterns) -> clean (All rg patterns match in the current tree (Phase A/B in arch+docs+CONTRIBUTING, feature-parity matrix wording, continue-on-error comment includes the advisory check names).)
- `task_changes`: Scope drift: compare workspace_classification task_changes against declared scope -> clean (9 task changes (.github/workflows/ci.yml, CONTRIBUTING.md, README.md, crates/README.md, crates/runx-core/api-snapshot.txt, docs/rust-kernel-architecture.md, docs/trusted-kernel-package-truth.md, package.json, scripts/check-rust-kernel-parity.mjs) all listed in scope; 0 ambient drift.)
- `.github/workflows/ci.yml`: Toolchain ordering / default rustup channel hijack -> finding (See F1-nightly-overrides-stable-toolchain.)
- `scripts/check-rust-kernel-parity.mjs`: CWD assumption vs documented invocation locations -> finding (See F2-wrapper-script-cwd-mismatch.)
- `scripts/check-rust-kernel-parity.mjs`: Missing-tool diagnostics parity between modes -> finding (See F3-api-only-missing-nightly-hint.)
- `scripts/check-rust-kernel-parity.mjs`: Public-API snapshot trailing-newline and missing-file behavior -> clean (Actual stdout is normalized to end with newline; expected file ends with newline; mismatch produces an actionable regenerate command. Missing snapshot file would throw an unhandled ENOENT, but this is recoverable and not in scope of completion blocking.)
- `crates/runx-core/api-snapshot.txt`: Duplicate impl entries on FanoutGateAction -> FanoutSyncOutcome From -> clean (cargo-public-api intentionally emits each impl once per public participating type; the duplication on lines 130-131 vs 137-138 matches the source and target enum listings and is not a snapshot bug.)
- `.scafld/specs/drafts/rust-kernel-blocking-promotion.md`: Follow-up handoff coverage -> clean (Draft spec exists, names the 5 clean kernel-touching PR trigger, restricts CI promotion to advisory checks only, and explicitly leaves runtime/CLI cutover to the feature-parity matrix.)
- `docs/rust-kernel-architecture.md + docs/trusted-kernel-package-truth.md`: Authority/cutover language regression hunt -> clean (Both docs continue to assert TypeScript authority, name `rust-cli-feature-parity-matrix` as the cutover oracle, and route Phase B promotion to `rust-kernel-blocking-promotion`.)
- `package.json + CONTRIBUTING.md + README.md`: Convention check: script alias style, contributor guidance accuracy -> clean (`rust:check`, `rust:crate-graph`, `rust:style` match existing script style; CONTRIBUTING.md documents Phase A advisory wording and the optional cargo-install steps including nightly minimal; README.md mentions `pnpm rust:check` in the dev setup block.)

Findings:
- [medium/non-blocking] `F1-nightly-overrides-stable-toolchain` `dtolnay/rust-toolchain@nightly` after `@stable` sets nightly as the default `rustup` toolchain, so the subsequent `Rust checks` step (cargo fmt/clippy -D warnings/test/package) runs on nightly, not stable.
  - Location: `.github/workflows/ci.yml:58`
  - Evidence: `Setup Rust` uses `dtolnay/rust-toolchain@stable` (line 53). `Setup Rust nightly` uses `dtolnay/rust-toolchain@nightly` (line 58) with no `override: false`. The dtolnay action runs `rustup default <toolchain>` on each invocation, so the second call wins and nightly becomes the default. The next step `Rust checks` (lines 71-77) runs `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo package -p runx-cli` with no `+stable` selector, so it executes against nightly. Nightly clippy carries lints that are not on stable; combined with `-D warnings` this is brittle (a single new-in-nightly lint can fail merges that pass locally on stable).
  - Impact: CI can flake or break unexpectedly when a new nightly version lands a clippy/rustfmt lint, even though no project code changed. Conversely, contributors checking locally on stable cannot reproduce the failure.
  - Validation: Run CI on a branch and verify `rustc --version` reported during `Rust checks` is the stable channel; or revert the nightly step and observe the wrapper's cargo-public-api invocation requires `+nightly`.
- [medium/non-blocking] `F2-wrapper-script-cwd-mismatch` `crates/README.md` documents running the wrapper from `oss/crates/` with `node ../scripts/check-rust-kernel-parity.mjs`, but the script uses CWD-relative paths that only resolve from `oss/`.
  - Location: `crates/README.md:40`
  - Evidence: crates/README.md:17 lists `node ../scripts/check-rust-kernel-parity.mjs` under the `Commands` section, and crates/README.md:40-44 explicitly says "run `pnpm rust:check` from `oss/` or `node ../scripts/check-rust-kernel-parity.mjs` from `oss/crates/`". However scripts/check-rust-kernel-parity.mjs uses CWD-relative paths: `--manifest-path crates/Cargo.toml` (lines 13-15, 21, 68), `node scripts/check-rust-crate-graph.mjs` (line 16), `node scripts/check-rust-core-style.mjs` (line 17), and the snapshot read of `crates/runx-core/api-snapshot.txt` (line 77). Run from `oss/crates/`, these resolve to `crates/crates/Cargo.toml`, `scripts/check-rust-crate-graph.mjs` (no such file under `oss/crates/scripts/`), etc., so the wrapper aborts on the first cargo or node invocation.
  - Impact: Contributors who follow the README literally hit confusing path-not-found errors instead of the intended diagnostics. The wrapper is effectively only usable from `oss/`.
  - Validation: From `oss/crates/`, run `node ../scripts/check-rust-kernel-parity.mjs` and confirm it succeeds without `cargo` errors about missing `crates/Cargo.toml`.
- [low/non-blocking] `F3-api-only-missing-nightly-hint` The `--api-only` branch's missing-tool message omits the nightly-toolchain hint that the non-api-only path provides.
  - Location: `scripts/check-rust-kernel-parity.mjs:60`
  - Evidence: In scripts/check-rust-kernel-parity.mjs:60-64, when `cargo public-api` is missing in `--api-only` mode the script prints only `Install it with: cargo install cargo-public-api`. The other branch (`checkTooling`, lines 50-55) additionally prints `cargo-public-api also needs nightly rustdoc JSON: rustup toolchain install nightly --profile minimal`. cargo-public-api requires nightly rustdoc JSON either way, so a developer running `pnpm rust:check -- --api-only` (or `node scripts/check-rust-kernel-parity.mjs --api-only`) only learns half the install story.
  - Impact: First-time `--api-only` users may install cargo-public-api and then hit an opaque `rustup component add rust-docs-json` / nightly missing error on the next run instead of being told upfront.
  - Validation: On a host without nightly installed, run `node scripts/check-rust-kernel-parity.mjs --api-only` and verify the error message names the nightly install step.

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
