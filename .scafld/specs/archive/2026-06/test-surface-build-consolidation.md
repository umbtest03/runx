---
spec_version: '2.0'
task_id: test-surface-build-consolidation
created: '2026-05-27T13:45:00Z'
updated: '2026-06-05T01:13:56Z'
status: completed
harden_status: not_run
size: large
risk_level: medium
---

# Consolidate the test build surface

## Current State

Status: completed
Current phase: final
Next: done
Reason: finalization receipt passed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-05T01:11:48Z
Review gate: pass

## Summary

The long runx CI job is build-bound, not test-bound. Test execution is already
fast: cloud vitest runs 527 tests in about 11s, and oss `verify:fast` is about
1.5min. The time goes into Rust compilation and linking, paid more than once per
run.

The dominant cause is that every top-level `tests/*.rs` file is compiled and
linked as its own crate. Cargo documents this and explicitly recommends a single
integration test split into modules when many integration tests make compile or
run time inefficient (Cargo Book, "Integration tests":
https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests).
For runx the case is stronger because `runx-runtime` links heavy adapter and
runtime deps (reqwest, tokio, rustls, rmcp) into every test binary.

This spec collapses the integration-test binaries to one per crate (the layout
several runx crates already imply), adds a guard so the `autotests = false`
layout cannot silently drop coverage, adopts cargo-nextest as the runner with
doctests kept as a separate `cargo test --doc` step, installs advisory tools as
prebuilt binaries, and tightens caching and job structure. No test assertions,
fixtures, or coverage are removed.

## Measured Baseline

Isolated worktree, clean target dir, deps and lib pre-warmed identically for both
runs, `runx-runtime --all-features`, `cargo test --no-run`:

- Current (42 separate test binaries): 244s to build, 382 tests.
- Merged (1 binary, same files as modules): 7s to build, same 382 tests.
- Result: about 35x on the test-build phase for one crate, identical tests.

Mechanism confirmed: test-source codegen is cheap; the cost is 42 separate linker
invocations, each statically linking `runx_runtime` plus reqwest/tokio/rustls/rmcp
into a ~51MB executable. `runx-runtime` is 42 of the workspace's integration
binaries, so the absolute saving is larger again.

Supporting signal: `oss/crates/target` was 183GB locally (cache bloat from
feature and toolchain permutations).

## Phase 1 Evidence (implemented)

Consolidated to one `tests/integration.rs` binary per crate, `autotests = false`,
each former test file kept intact as a module:

| crate | files merged | tests | suite result |
| --- | --- | --- | --- |
| runx-runtime | 42 | 383 | behavior-neutral vs baseline (see below) |
| runx-contracts | 17 | 80 | 80 passed |
| runx-cli | 13 + shared `support` | 74 | 74 passed |
| runx-core | 6 | 50 | 50 passed |
| runx-receipts | 3 | 26 | 26 passed |
| runx-parser | 5 | 20 | 20 passed |
| runx-sdk | 3 | 6 | 6 passed |

- Behavior-neutrality proof: under an identical local invocation, `runx-runtime`
  reports 372 pass / 11 fail both before consolidation (pristine HEAD worktree,
  42 separate binaries) and after (1 binary). The 11 failures are pre-existing,
  environment-dependent tests (receipt-signing issuer-key resolution) that only
  pass under the full verify-fast/CI orchestration; they are unrelated to and
  unaffected by consolidation.
- nextest validation: `cargo nextest run --workspace --all-features` runs 789
  tests, 778 pass, the same 11 runtime failures, nothing new. Process-per-test
  execution is green.
- `act.rs` (sdk) needed a structural fix because it owned submodules: it had a
  redundant inline `mod {}` wrapper that, once the file itself became a module,
  nested submodule paths one level too deep; the wrapper was removed so
  submodules resolve under `tests/<name>/`. `runx-pay/tests/payment.rs` remains
  unconsolidated because it is already a single integration-test file. `runx-cli`'s
  shared `tests/support/` is declared once in integration.rs and the five files
  that used it now reference `crate::support`.
- Every `--test <name>` invocation that targeted a now-removed standalone target
  was retargeted to `--test integration -- <module>` and verified to run the
  same subset (5 package.json scripts, the CI license-boundary guard, and the
  a2a/agent fixture-generator hint strings).

## Standard adopted

The forward standard for runx Rust testing:

- One integration binary per crate (`tests/integration.rs` + `autotests = false`).
- cargo-nextest as the normal runner (process-per-test isolation; see
  https://nexte.st/docs/design/why-process-per-test/), which removes the only
  regression introduced by sharing one binary: process-global state leaking
  across tests under threaded `cargo test`.
- Doctests run as a separate `cargo test --doc` step, because nextest does not
  execute doctests.
- A module-list and process-global-mutation guard
  (`scripts/check-integration-test-modules.mjs`, wired into verify-fast) so a new
  `tests/*.rs` cannot be silently un-compiled and so `env::set_var` /
  `set_current_dir` style mutations are banned in test code unless explicitly
  isolated and annotated.

## Scope

- In scope:
  - `crates/*/tests/` layout and the corresponding `[[test]]` / `autotests`
    manifest entries (done).
  - The module/process-global guard and its wiring (done).
  - `oss/.github/workflows/ci.yml` runner and advisory-tool steps (done).
  - Retargeting every `--test <name>` reference (done).
  - Follow-up CI structure: feature-surface review, job parallelization, cache.
- Out of scope:
  - Changing any test assertion, fixture, or expected value.
  - Removing or weakening any gate, including heavy graph and all-features tests.
  - JS/vitest test logic (already fast).
  - The stable/nightly toolchain arrangement: nightly exists specifically for
    `cargo-public-api` in the advisory parity step (see archived
    `rust-parity-ci-governance.md`); it is not removable here.

## Dependencies

- `heavy-test-suite-gating` (completed) established that CI must run
  `pnpm test:heavy:graph` and an all-features cargo test gate with a prebuilt
  eval binary. This spec keeps both: the heavy graph step is unchanged and the
  all-features gate is now `cargo nextest run --workspace --all-features` plus
  `cargo test --workspace --all-features --doc`.
- `runx-rust-95-release-readiness` (active) owns the mandatory Rust gates; the
  runner change here keeps those gates green.

## Risks

- Shared process state under threaded `cargo test`: addressed by the guard
  (bans process-global mutation; the scan found none) and by nextest's
  process-per-test execution.
- The 11 environment-dependent runtime tests must be confirmed green in CI under
  nextest; they are runner- and layout-independent (fail identically under
  cargo test and nextest, before and after consolidation), so this is a
  pre-existing orchestration concern, not a regression from this work.
- CI workflow edits cannot be fully validated locally; they take effect only on
  commit, so a dry-run on a branch is the gate before merge.

## Acceptance

- [x] `dod1` Each crate with tests builds exactly one integration binary; total
  test count unchanged. Evidence: 7 crates each emit one `tests/integration.rs`
  executable; per-crate counts recorded above (639 integration tests).
- [x] `dod2` Consolidated suites pass with no shared-state flakiness. Evidence:
  6/7 crates fully green; runtime behavior-neutral vs baseline; nextest green
  except the pre-existing 11.
- [x] `dod3` Heavy graph gate and an all-features cargo gate remain enforced in
  CI. Evidence: heavy graph step unchanged; `cargo nextest run --workspace
  --all-features` + `cargo test --workspace --all-features --doc` in Rust checks.
- [x] `dod4` Advisory tools installed prebuilt, not compiled from source.
  Evidence: `taiki-e/install-action` installs cargo-nextest, cargo-deny,
  cargo-public-api; the `cargo install` step is removed.
- [x] `dod5` Guard prevents silent coverage loss and bans process-global
  mutation. Evidence: `scripts/check-integration-test-modules.mjs` fails on an
  orphaned `tests/*.rs`, directory-style `tests/<name>/main.rs` targets under
  `autotests = false`, unresolved declared modules, and `env::set_var`; wired
  into verify-fast.
- [x] `dod6` Warm CI wall time materially reduced; before/after recorded from a
  real CI run. Evidence: branch `codex/readiness-ci-dry-run` ran `ci`
  workflow_dispatch successfully twice. First run
  `26987384653` (`995dd53b`, checks job
  <https://github.com/runxhq/runx/actions/runs/26987384653/job/79639879363>)
  completed in 8m30s. Warm rerun `26987685828` (`905290f7`, checks job
  <https://github.com/runxhq/runx/actions/runs/26987685828/job/79640782639>)
  completed in 7m34s. Final code run `26988331644` (`63d9350d`,
  checks job
  <https://github.com/runxhq/runx/actions/runs/26988331644/job/79642794879>)
  completed in 7m51s after the directory-target guard hardening.

## Phase 2: CI caching and structure

Status: completed

Implemented:

- Swapped the oss Cargo cache from `actions/cache` (keyed on `Cargo.lock`, target
  tree grew unbounded) to `Swatinem/rust-cache@v2` with `workspaces: crates`, the
  same Rust-aware cache the cloud workflow already uses. It keys on lockfile and
  rustc version and prunes stale artifacts.

Analyzed and deliberately NOT done:

- Job parallelization (split `checks` into parallel `verify` and `rust` jobs) was
  rejected after analysis: on a cold run each job restores its own cache and
  recompiles the dependency graph in its own target, so the expensive ~10min dep
  compile is paid twice and wall time (max of the jobs) does not improve; it only
  helps warm runs (already fast) by a couple of minutes. Low value, and it makes
  the cold case worse. Not pursued.
- Feature-surface "unification": clippy and the test build cannot share compiled
  artifacts (clippy wraps workspace crates), but heavy dependency artifacts ARE
  shared between them, and clippy/test already use the same `--all-features` set.
  There is no safe further consolidation here; closed.

Recommended next lever (needs a CI dry-run before merge):

- sccache as a persistent compilation cache (`RUSTC_WRAPPER=sccache`, GHA-cache
  backend, `CARGO_INCREMENTAL=0`). This is the real lever for the cold case that
  produced the original ~40min run: when `Cargo.lock` changes only some deps,
  sccache reuses cached object files for the unchanged ~297 deps instead of
  recompiling the whole graph. It coexists with clippy's `RUSTC_WORKSPACE_WRAPPER`.
  Left unimplemented because a misconfigured wrapper silently disables caching or
  breaks the build, and it cannot be validated locally.

Optional, lower priority:

- Gate `cargo package -p runx-cli` to pushes on main and tags rather than every
  PR (it does a full isolated rebuild). Mildly reduces per-PR publish safety, so
  decide explicitly.

- [x] `p2_ac1` command - oss Cargo cache uses rust-cache.
  - Command: `rg -n "Swatinem/rust-cache" .github/workflows/ci.yml`
  - Expected kind: `reviewed_output`
  - Status: pass
  - Evidence: `.github/workflows/ci.yml:72` uses `Swatinem/rust-cache@v2`.
- [x] `p2_ac2` manual - branch CI run green end to end under nextest; warm-run
  wall time recorded before/after.
  - Status: pass
  - Evidence: `ci` workflow_dispatch on branch `codex/readiness-ci-dry-run`
    passed at run `26987384653`, warm rerun `26987685828`, and final code
    run `26988331644`; each includes the Rust checks step with
    `cargo nextest run --workspace --all-features`.

## Phase 3: Closure Evidence

Status: pass
Dependencies: none

Objective: record post-review readiness evidence in the scafld ledger after the

Changes:
- none

Acceptance:
- [x] `p3_ac1` command - fast readiness suite stays green
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21

## Rollback

- Phase 1 is the high-value change and the only one touching test layout. Each
  crate's consolidation is independent; if a crate ever cannot run cleanly in one
  binary, leave it unconsolidated rather than weakening tests.
- CI changes are revertible by restoring the prior `ci.yml`; no test content
  changes, so coverage is unaffected by any rollback.
- Keep each lever in its own commit so it can be reverted independently.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-8
Output: claude.mcp_submit_review
Summary: Discover-mode re-review of the test-build consolidation. Session reports baseline clean, task_changes none, ambient_drift none; the implementation landed in earlier commits, so this is an independent re-verification of delivered work. Verified directly: all 7 spec-listed crates (runtime/contracts/cli/core/receipts/parser/sdk) set autotests=false AND declare an explicit [[test]] name="integration" path="tests/integration.rs" target (runtime:88-90, contracts:41-43, cli:52-54, receipts:41-43, parser:37-39, sdk:31-33, core:39-41) — the prior review's guard-missing-test-target-stanza finding is fixed in code (check-integration-test-modules.mjs:76-104,121-128). Module-to-file parity confirmed for every consolidated crate: runtime 42 mods = 42 sibling files, contracts 16=16, cli 13 + support, core 6=6, receipts 3=3, parser 5=5, sdk 3=3; cli's shared support resolves via tests/support/mod.rs. A glob for crates/*/tests/*/main.rs returns nothing, so the directory-target guard neither false-fails nor masks coverage loss. The process-global-mutation ban is currently green: a grep for env::set_var/env::remove_var/set_current_dir across crates/**/tests/**/*.rs returns no matches. Guard wiring intact: verify-fast.mjs:45 runs the guard unconditionally and ci.yml:52 runs pnpm verify:fast. CI gates intact: nextest --all-features (ci.yml:84), cargo test --doc (87), heavy graph (76-77), prebuilt advisory tools via taiki-e/install-action with no cargo install (62-66), Swatinem/rust-cache@v2 workspaces: crates (72-74), and the license-boundary retarget --test integration -- license_boundary (95) maps to runtime's declared mod license_boundary (integration.rs:28). No completion blockers found. Two low non-blocking items: (1) carry-forward — dod6/p2_ac2 greenness and warm-time evidence rests on external GitHub Actions runs a read-only reviewer cannot fetch; (2) NEW — the just-added hasIntegrationTestTarget parser only recognizes double-quoted name/path values, so a valid single-quoted TOML stanza would cause a false guard failure (fails loud, not silent, so safe). No test assertions, fixtures, or coverage removed; no regression introduced.

Attack log:
- `crates/*/Cargo.toml [[test]] stanza vs autotests=false (prior finding fix)`: Verify each autotests=false crate now declares the explicit [[test]] name=integration path=tests/integration.rs target that the prior review flagged as unenforced -> clean (All 7 crates declare the stanza (runtime:88-90, contracts:41-43, cli:52-54, receipts:41-43, parser:37-39, sdk:31-33, core:39-41) and the guard now enforces it via hasIntegrationTestTarget (mjs:76-104,121-128). Prior guard-missing-test-target-stanza finding is fixed.)
- `crates/runx-runtime/tests/integration.rs vs tests/*.rs (dod1)`: Count declared modules against on-disk sibling test files to confirm no file is silently un-compiled -> clean (42 mod declarations (lines 8-49) exactly match 42 tests/*.rs sibling files including support.)
- `contracts/core/receipts/parser/sdk integration.rs vs tests/*.rs (dod1)`: Confirm module/file parity for the remaining consolidated crates -> clean (contracts 16=16, core 6=6, receipts 3=3, parser 5=5, sdk 3=3. Every on-disk test file is declared; every declared module has a file.)
- `crates/runx-cli/tests/support (regression: shared helper resolution)`: Confirm cli's 13 test modules plus shared support resolve and no top-level file is orphaned -> clean (13 test mods + mod support; support resolves via tests/support/mod.rs (glob confirmed). All 13 sibling .rs files declared.)
- `crates/*/tests/*/main.rs (regression: directory-style targets)`: Search for directory-style integration targets that autotests=false would silently drop or that would trip the guard -> clean (Glob returned no files; the directory-target guard (mjs:151-163) neither false-fails nor masks loss.)
- `crates/**/tests/**/*.rs process-global mutation ban (dod5)`: Grep test code for env::set_var/env::remove_var/set_current_dir to confirm the guard's ban is currently green and not silently failing CI -> clean (No matches across all crate test trees, so the banned-mutation scan passes; guard stays green.)
- `scripts/verify-fast.mjs:45 + .github/workflows/ci.yml:52 (guard wiring)`: Confirm the coverage guard actually executes in CI and is not dead code -> clean (verify-fast.mjs:45 runs node scripts/check-integration-test-modules.mjs; ci.yml:52 runs pnpm verify:fast. Cannot be bypassed silently.)
- `.github/workflows/ci.yml (dod3/dod4/p2_ac1)`: Confirm nextest + doctest + heavy-graph gates, prebuilt advisory tools with no cargo install, and Swatinem rust-cache -> clean (62-66 taiki-e/install-action installs cargo-nextest,cargo-deny,cargo-public-api; 72-74 rust-cache workspaces:crates; 84 nextest --all-features; 87 cargo test --doc; 76-77 heavy graph. No cargo install present.)
- `.github/workflows/ci.yml:95 (regression: --test retargeting)`: Confirm the license-boundary guard still runs the same module subset after consolidation -> clean (--test integration -- license_boundary filters to the license_boundary:: module; runtime integration.rs declares mod license_boundary; (line 28). A filter narrows, never widens, coverage gaps.)
- `scripts/check-integration-test-modules.mjs:97 (guard TOML parsing robustness)`: Probe the just-added stanza detector for parsing gaps that could cause false failures or missed detection -> finding (Assignment regex only accepts double-quoted name/path values; valid single-quoted TOML stanzas would false-fail. Fails loud, not silent. Low non-blocking robustness item.)
- `dod6/p2_ac2 external CI evidence`: Cross-check cited CI run commit SHAs against git history and assess read-only verifiability of greenness -> finding (SHAs 995dd53b/905290f7/63d9350d are real repo commits, but Actions job greenness and warm-time numbers are not fetchable in read-only mode (low residual).)
- `workspace classification / spec mutation`: Confirm changes are limited to CI tooling + guard + spec evidence with no out-of-scope drift or production test-logic -> clean (scafld reports baseline clean, task_changes none, ambient_drift none. Guard and ci.yml are CI tooling, not production code; test-logic-separation invariant preserved.)

Findings:
- [low/non-blocking] `dod6-external-run-unverifiable` dod6/p2_ac2 green-CI and warm-time evidence rests on external GitHub Actions runs not fetchable in read-only review.
  - Location: `oss/.scafld/specs/active/test-surface-build-consolidation.md:177`
  - Evidence: dod6 cites workflow_dispatch runs 26987384653 (995dd53b), 26987685828 (905290f7), and 26988331644 (63d9350d). The cited SHAs correspond to real recent commits in this repo, corroborating the references, but a read-only reviewer cannot open the Actions job URLs to confirm the runs were green or the warm wall-time figures.
  - Impact: Criteria depending on real CI behavior (suite green under nextest; warm wall time reduced) cannot be independently confirmed locally.
  - Validation: Cross-checked cited SHAs against the local repo; external run greenness not fetchable in read-only mode.
- [low/non-blocking] `guard-toml-single-quote-narrow` The integration [[test]] stanza detector only recognizes double-quoted TOML values; a valid single-quoted stanza would trip a false guard failure.
  - Location: `oss/scripts/check-integration-test-modules.mjs:97`
  - Evidence: hasIntegrationTestTarget parses name/path via /^([A-Za-z_][A-Za-z0-9_-]*)\s*=\s*"([^"]*)"\s*$/ which only matches double-quoted strings. TOML also permits literal single-quoted strings, so `name = 'integration'` / `path = 'tests/integration.rs'` would not be recognized and the guard would error that the explicit [[test]] target is missing. All 7 crates currently use double quotes, so the path is not exercised today.
  - Impact: Latent brittleness in the just-added fix. It fails loud (false-positive guard error blocking CI), not silently, so it cannot cause silent coverage loss — the failure mode dod5 guards against is preserved. Low risk.
  - Validation: Read check-integration-test-modules.mjs:76-104; grepped all 7 Cargo.toml stanzas and confirmed each uses double-quoted name/path values, so the gap is latent, not active.

