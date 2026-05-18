---
spec_version: '2.0'
task_id: rust-contracts-bootstrap
created: '2026-05-17T02:10:00Z'
updated: '2026-05-17T13:04:55Z'
status: completed
harden_status: passed
size: small
risk_level: medium
---

# Rust contracts bootstrap

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-17T13:04:55Z
Review gate: pass

## Summary

Lock the Rust crate graph and `runx-contracts` placeholder before any kernel
implementation spec executes. This is the deterministic pre-kernel gate: it
does not port contract behavior, but it makes the `runx-contracts` crate a
stable workspace dependency and prevents dependency drift while the kernel port
runs. Placeholder crates use the crates.io reservation version `0.0.1`; the
separate `rust-placeholder-crates-publish` spec owns the irreversible publish
step that claims those names on crates.io.

Full contract behavior remains in `rust-contracts-parity`. This bootstrap
exists so `runx-core` can depend on `runx-contracts` from day one without
forcing the full contracts port to block state-machine and policy parity.

## Context

CWD: `.`

Packages:
- `crates/runx-contracts`
- `crates/runx-core`
- `crates/runx-parser`
- `crates/runx-receipts`
- `crates/runx-runtime`
- `crates/runx-sdk`
- `crates/runx-cli`

Files impacted:
- `crates/Cargo.toml`
- `crates/runx-contracts/Cargo.toml`
- `crates/runx-core/Cargo.toml`
- `crates/runx-parser/Cargo.toml`
- `crates/runx-receipts/Cargo.toml`
- `crates/runx-runtime/Cargo.toml`
- `crates/runx-sdk/Cargo.toml`
- `crates/README.md`
- `scripts/check-rust-crate-graph.mjs`
- `scripts/check-rust-core-style.mjs`
- `scripts/verify-fast.mjs`
- `package.json`
- `docs/rust-kernel-architecture.md`

Invariants:
- The initial Rust workspace has seven crates only: `runx-cli`,
  `runx-contracts`, `runx-core`, `runx-parser`, `runx-receipts`,
  `runx-runtime`, and `runx-sdk`.
- `runx-authoring` and `runx-adapters` are not initial crates.
- `runx-contracts`, `runx-core`, `runx-parser`, `runx-receipts`,
  `runx-runtime`, and `runx-sdk` use placeholder reservation version `0.0.1`.
- `runx-cli` remains publishable because it is the usable launcher package.
- `runx-sdk` depends on `runx-contracts`, not `runx-core`.
- `runx-core` may depend on `runx-contracts`, but not parser, receipts,
  runtime, SDK, or CLI.
- `runx-runtime` owns adapter families as features, not a separate
  `runx-adapters` crate.

Related docs:
- `docs/rust-kernel-architecture.md`
- `crates/README.md`
- `.scafld/specs/drafts/rust-contracts-parity.md`
- `.scafld/specs/drafts/rust-kernel-port-orchestration.md`

## Objectives

- Enforce the seven-crate workspace shape in code.
- Keep non-usable placeholder crates at reservation version `0.0.1`.
- Add a crate graph checker that fails on dependency-direction drift.
- Wire the crate graph and Rust style checks into `verify:fast`.
- Keep this bootstrap intentionally smaller than `rust-contracts-parity`.

## Scope

In scope:
- Workspace crate membership and local dependency direction.
- Placeholder publish policy.
- Rust crate graph validation.
- Rust style validation wiring.

Out of scope:
- Porting any TypeScript contract behavior.
- Adding serde models to `runx-contracts`.
- Adding hashing helpers or fixtures.
- Publishing any crate to crates.io.

## Dependencies

- `rust-runx-cli-placeholder` has created the initial Cargo workspace.
- `docs/rust-kernel-architecture.md` documents the seven-crate graph.

## Assumptions

- Full `runx-contracts` behavior can follow in `rust-contracts-parity`.
- Kernel parity can start after this bootstrap because `runx-core` only needs
  the crate boundary, not the full contract surface, for its first phase.

## Risks

- Medium: this bootstrap can be mistaken for contract parity. Mitigated by
  docs and spec wording: no contract behavior is ported here.
- Medium: dependency rules can become stale as real implementation specs land.
  Mitigated by treating every dependency relaxation as a spec-level change.
- Low: placeholder publishing could imply feature readiness. Mitigated by
  README/crate docs that clearly mark placeholders and by publishing only the
  explicit `0.0.1` reservation release until implementation specs land.

## Acceptance

Profile: strict

Validation:
- [x] `v1` command - crate graph check passes.
  - Command: `node scripts/check-rust-crate-graph.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26
- [x] `v2` command - Rust style check passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-27
- [x] `v3` command - Cargo workspace checks pass.
  - Command: `cd crates && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo package --workspace --allow-dirty`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-28
- [x] `v4` command - fast workspace verification includes Rust guardrails.
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29

## Phase 1: Guardrail implementation

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- `crates/*/Cargo.toml` (partial, shared) - Keep placeholder crates at reservation version `0.0.1`; keep local workspace dependencies and do not set `publish = false`.
- `scripts/check-rust-crate-graph.mjs` (all, exclusive) - Enforce crate membership, publish policy, and dependency direction.
- `scripts/verify-fast.mjs` (partial, shared) - Run Rust crate-graph and style guardrails.
- `package.json` (partial, shared) - Add `rust:crate-graph` and `rust:style` scripts.
- `crates/README.md` (partial, shared) - Document commands and publish policy.

Acceptance:
- [x] `ac1_1` command - graph guardrails pass.
  - Command: `pnpm rust:crate-graph && pnpm rust:style`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16
- [x] `ac1_2` command - placeholder publish policy is enforced.
  - Command: `for c in runx-contracts runx-core runx-parser runx-receipts runx-runtime runx-sdk; do rg -n '^version = "0\.0\.1"$' "crates/$c/Cargo.toml" >/dev/null || exit 1; if rg -n '^publish = false$' "crates/$c/Cargo.toml" >/dev/null; then exit 1; fi; done; if rg -n '^publish = false$' crates/runx-cli/Cargo.toml >/dev/null; then exit 1; fi`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17

## Rollback

Strategy: per_phase

Commands:
- Phase 1: revert placeholder versions if needed, remove crate graph script,
  remove package scripts, remove `verify:fast` calls, and restore any
  accidental `publish = false` settings if they appear.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode pass on rust-contracts-bootstrap. The prior review's only finding, F1 (`dependencyNamesFromSubtables` feeding subtable bodies back into `dependencyNamesFromSection` and thereby treating nested TOML keys like `version`/`features` as dependency names), is fixed. The new `dependencyNamesFromSubtables` (scripts/check-rust-crate-graph.mjs:194) only pushes the subtable's own name from the header and delegates to a new `dependencyPackageNameFromTable` helper (scripts/check-rust-crate-graph.mjs:210) that exclusively recognizes `package = "..."` rename lines. The previously identified regression vector — adding `[dependencies.tokio]\nversion = "1"` to a non-CLI crate would have polluted the parsed dep list with `version`/`features` but still caught `tokio`; that pollution is now gone, while the real `tokio` (or `package = "tokio"`) detection paths are preserved. Inline-form workspace deps (`runx-contracts.workspace = true`) remain captured by `dependencyNamesFromSection`. No other behavior changed: workspace member list, placeholder publish policy enforcement, allowed/required Runx-deps graph, and premature-runtime-dep guard remain identical. The seven crate manifests still pin to 0.0.1 with no `publish = false`, runx-cli stays at 0.1.0, and the trusted-kernel dep direction (contracts is a sink; core depends only on contracts; parser on contracts+core; receipts on contracts; runtime on contracts+core+parser+receipts; sdk on contracts only) matches both AGENTS.md and crates/README.md. Task-scope classifier reports a single in-scope change (the F1 fix) and zero ambient drift, with no review-time self-mutation. Acceptance evidence v1..v4 and ac1_1/ac1_2 remains recorded as pass; verify_open_blockers rerun policy has no prior open blockers to re-execute. No new findings; F1 is marked fixed.

Attack log:
- `scripts/check-rust-crate-graph.mjs (F1 fix)`: Verify the changed dependencyNamesFromSubtables no longer leaks nested TOML keys and that the new dependencyPackageNameFromTable correctly captures package renames -> clean (New code at lines 194-218 pushes only the subtable's own dep name from the [dependencies.<name>] header and consults a dedicated helper that matches `^package = "..."` only. Nested keys like `version`, `features`, `default-features`, `optional` can no longer be mistaken for deps. Inline form like `runx-contracts.workspace = true` is still caught by the unchanged dependencyNamesFromSection.)
- `scripts/check-rust-crate-graph.mjs (regression sweep)`: Spec compliance: confirm expectedMembers, placeholderReservationCrates, allowedRunxDeps, requiredRunxDeps still encode the trusted-kernel direction unchanged by the F1 fix -> clean (Lines 8-43 unchanged: seven-crate set, six placeholders pinned to 0.0.1, runx-cli optionally depending on runtime/contracts, contracts is a leaf, core->contracts, parser->contracts+core, receipts->contracts, runtime->contracts+core+parser+receipts, sdk->contracts only. Matches AGENTS.md trusted-kernel rules and crates/README.md.)
- `crates/*/Cargo.toml`: Acceptance ac1_2 manual cross-check: every placeholder crate pinned to 0.0.1 with no `publish = false`; runx-cli at 0.1.0 publishable -> clean (runx-contracts/core/parser/receipts/runtime/sdk all carry explicit `version = "0.0.1"` in their [package] block (so parsePackageVersion regex matches); runx-cli at 0.1.0. No `publish = false` anywhere. checkPublishingReadiness will flag any future drift.)
- `Workspace classifier`: Ambient drift / scope drift: confirm only the F1 fix landed in task scope and nothing outside scope was modified or self-mutated during review -> clean (Session reports a single task-scope change to scripts/check-rust-crate-graph.mjs (the F1 fix). Ambient drift: none. No spec or review-target self-mutation.)
- `AGENTS.md trusted-kernel invariants`: Convention check: domain boundaries — does the F1 fix or any allowed-dep edit introduce paths that let core depend on runtime, parser on runtime, or sdk on core? -> clean (allowedRunxDeps unchanged and still acts as the strict inverse of the trusted-kernel rule. F1 fix is purely in the TOML-parsing helper layer; it does not touch the graph spec.)
- `scripts/check-rust-crate-graph.mjs dependency parser edge cases`: Dark patterns: probe regex edge cases again post-fix — subtable header matching, body termination at next top-level `[`, [[bin]] tables, [workspace.dependencies] leakage, inline `dep.workspace = true` form, and feature-then-dependency ordering -> clean (Header pattern requires the inner name to match `[A-Za-z0-9_-]+]`, so `[[bin]]` and `[workspace.dependencies]` are not picked up. sectionBody anchors on `^\[name\]\s*$` and terminates at the next top-level `[`; verified against runx-runtime/Cargo.toml where `[features]` precedes `[dependencies]`. Inline workspace form is captured by dependencyNamesFromSection in the main body. New subtable handler only reads the subtable's header name and an explicit `package = "..."` rename. Edge: subtable name itself uses `_`/`-` are both allowed (matches Cargo crate naming). The only remaining cosmetic concern — that `/^\[/m` body termination would stop early on a nested table like `[dependencies.foo.bar]` — does not affect real Cargo manifests (no such nesting). Not a finding.)
- `Acceptance evidence v1..v4 / ac1_1 / ac1_2`: Verify recorded acceptance signals under verify_open_blockers rerun policy -> skipped (All six entries recorded as status=pass with exit-code-zero evidence. Re-run policy is verify_open_blockers; no prior open blockers exist to re-execute, and review mode is read-only.)

Findings:
- [low/non-blocking] `F1-subtable-dep-parse-false-positives` dependencyNamesFromSubtables previously treated nested TOML keys as dependency names; now fixed via dedicated package-name helper
  - Location: `scripts/check-rust-crate-graph.mjs:194`
  - Evidence: scripts/check-rust-crate-graph.mjs:194-208 no longer calls dependencyNamesFromSection on subtable bodies. It pushes match[1] (the actual subtable name from the [dependencies.<name>] header) and consults dependencyPackageNameFromTable (scripts/check-rust-crate-graph.mjs:210-218), which only matches `^package\s*=\s*"..."`. Nested TOML keys such as `version`, `features`, `optional`, `default-features` can no longer leak into the names set.
  - Impact: Fix removes a latent false-positive vector. Real dep detection paths (inline workspace form, subtable header, package = "..." rename) all still work. No regression observed: the only callers (checkRunxDependencies / checkPrematureRuntimeDependencies) compare against the prefix `runx-` and the placeholderOnlyDisallowedDeps allowlist, both of which still receive the correct names. Verified mentally against runx-runtime/Cargo.toml ([features] preceding [dependencies], multiple inline workspace deps) and against the hypothetical `[dependencies.tokio]\nversion = "1"` case.
  - Validation: Conceptual regression fixture: parseDependencyNames on `[dependencies.tokio]\nversion = "1"\nfeatures = [\"rt\"]` should now return ["tokio"], not ["features", "tokio", "version"].

## Self Eval

Status: not_started
Completeness: none
Architecture fidelity: none
Spec alignment: none
Validation depth: none
Total: none
Second pass performed: none

Threshold: 8

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
- contracts
- crates
- guardrails

## Origin

Source:
- pre-execution refinement requested before running the Rust specs.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- precedes: rust-kernel-parity-fixtures
- precedes: rust-contracts-parity
- precedes: rust-sdk-surface-parity

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-17T12:49:51Z
Ended: 2026-05-17T12:50:33Z

Checks:
- path audit
  - Grounded in: code:crates/Cargo.toml:1
  - Result: passed
  - Evidence: Scope is limited to the Rust workspace manifests, placeholder
- command audit
  - Grounded in: code:package.json:1
  - Result: passed
  - Evidence: Acceptance commands are executable through `pnpm rust:crate-graph`,
- scope/migration audit
  - Grounded in: code:scripts/check-rust-crate-graph.mjs:1
  - Result: passed
  - Evidence: No runtime behavior is ported in this bootstrap. The script
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: Validation runs after guardrail files exist and before dependent
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback is local to manifests, guardrail scripts, package
- design challenge
  - Grounded in: spec_gap:scope
  - Result: passed
  - Evidence: Bootstrap intentionally avoids contract behavior so kernel parity

Questions:
- none


## Planning Log

- 2026-05-17T02:10:00Z: Drafted as the deterministic pre-kernel bootstrap.
  This keeps the full contracts port separate while making `runx-contracts`
  and the Rust crate graph stable before kernel execution begins.
- 2026-05-17T02:25:00Z: Initially adjusted publishing policy to keep
  placeholders publishable and to defer crates.io name claims to the publish
  spec.
- 2026-05-17T12:35:00Z: Updated bootstrap to match the actual crates.io
  reservation state. Placeholder crates now use `0.0.1`; the publish spec owns
  the irreversible publish evidence.
