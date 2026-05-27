---
spec_version: '2.0'
task_id: runx-runtime-perf-gate-release-probes-v1
created: '2026-05-28T00:00:00Z'
updated: '2026-05-27T15:58:38Z'
status: completed
harden_status: not_run
size: small
risk_level: low
---

# Runtime perf gate release probes

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-27T15:58:38Z
Review gate: pass

## Summary

The runtime throughput harness already owns the right performance surface. This
cut tightens that harness so process/protocol probe rows measure the current
release-built Rust binaries under the perf target directory, not stale debug
artifacts from a previous local build.

No broad runtime refactor, no new benchmark framework, and no speculative
optimization. This is a trustworthiness cut for the existing S-tier gate.

## Objectives

- Measure `native_cli_launch` against the current release `runx` binary built
  under `crates/target/runx-perf`.
- Measure MCP session probes against the current release `runx-mcp-session-probe`
  binary built under the same perf target directory.
- Ask Cargo to refresh release probe binaries on each capture so existing perf
  artifacts cannot silently stand in for the current checkout.
- Remove the stale `crates/target/debug/runx` shortcut from the perf harness.
- Pin the release-probe behavior in the existing perf harness check.
- Document the release-probe invariant.

## Scope

- In scope:
  - `scripts/runtime-throughput.mjs`
  - `scripts/check-runtime-perf-harness.mjs`
  - `docs/runtime-throughput.md`
- Out of scope:
  - Runtime algorithm refactors.
  - New Criterion workloads.
  - CI matrix restructuring.
  - Editing unrelated operational draft specs or `docs/thesis.md`.

## Risks

- Release probe builds can make first-run capture slower. Mitigation: only
  process/protocol probe binaries use release builds; Cargo reuses artifacts
  while still refreshing them against the current checkout.
- Static checks can become brittle. Mitigation: check only durable invariants:
  release path, `--release`, and absence of the stale debug shortcut.

## Acceptance

Validation:
- `pnpm perf:harness-check`
- `node scripts/runtime-throughput.mjs capture --workloads ts_bridge_framing,native_cli_launch --output /tmp/runx-runtime-perf-smoke.json`
- `pnpm runtime:architecture-check -- --phase session-pooling`
- `git diff --check`
- `scafld review runx-runtime-perf-gate-release-probes-v1 --provider claude --review-depth deep`

## Phase 1: Release Probe Builds

Status: completed
Dependencies: none

Objective: Process/protocol perf rows use release-built current binaries.

Changes:
- Build and run `runx-mcp-session-probe` from `crates/target/runx-perf/release`.
- Build and run `runx` from `crates/target/runx-perf/release`.
- Remove the fallback to `crates/target/debug/runx`.

Acceptance:
- [x] `p1_ac1` command - Release probe invariant is pinned by the perf harness check.
  - Command: `pnpm perf:harness-check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-5
- [x] `p1_ac2` command - Release native launch smoke capture works.
  - Command: `node scripts/runtime-throughput.mjs capture --workloads ts_bridge_framing,native_cli_launch --output /tmp/runx-runtime-perf-smoke.json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6

## Phase 2: Boundary And Review Preparation

Status: completed
Dependencies: phase1

Objective: Existing runtime architecture gate passes, and the Claude review
command is prepared for the lifecycle review gate.

Changes:
- Keep the actual Claude review in the top-level validation gate, because `scafld review` requires task status `review`.

Acceptance:
- [x] `p2_ac1` command - Session pooling architecture check still passes.
  - Command: `pnpm runtime:architecture-check -- --phase session-pooling`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17
- [x] `p2_ac2` command - Diff has no whitespace errors.
  - Command: `git diff --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18
- [x] `p2_ac3` command - Claude review command is recorded for the lifecycle gate.
  - Command: `rg -n "scafld review runx-runtime-perf-gate-release-probes-v1 --provider claude --review-depth deep" .scafld/specs/active/runx-runtime-perf-gate-release-probes-v1.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Discover-mode re-review of runx-runtime-perf-gate-release-probes-v1. The task scope matches the diff: scripts/runtime-throughput.mjs builds/runs runx and runx-mcp-session-probe from crates/target/runx-perf/release with --release, the crates/target/debug fallback is gone, scripts/check-runtime-perf-harness.mjs statically pins the release-probe behavior (with regex checks scoped via functionSource so they target each cargo invocation), and docs/runtime-throughput.md documents the invariant. No new completion blockers found. The two prior non-blocking findings recorded in the spec re-read as not applicable against the current code: (a) `perf-invariant-regex-too-permissive` assumed the --release/--bin/runx regex ran against the full source, but check-runtime-perf-harness.mjs:114-119 first extracts each function body via functionSource() and runs the regex against the scoped range, where only one "--release" literal exists; and (b) `perf-probe-binary-staleness-reuse` assumed cargo build was gated on existsSync, but nativeCliProbe (lines 447-470) and mcpSessionProbe (lines 400-425) invoke cargo build unconditionally and use existsSync only as post-build verification, so Cargo refreshes against the current checkout. Acceptance commands p1_ac1, p1_ac2, p2_ac1, p2_ac2, p2_ac3 were already recorded passing; not re-executed in read-only mode. Ambient drift (scripts/test-boundaries.mjs and out-of-scope draft specs) was not attributed to this task.

Attack log:
- `scripts/check-runtime-perf-harness.mjs assertReleaseProbeInvariant`: re-verify prior finding 1: would dropping --release from runx-cli args still satisfy the regex via tokens leaking from mcpSessionProbe block or the binaryName literal? -> clean (Regex now runs against nativeProbeSource (functionSource start=function nativeCliProbe( end=function runNativeCliProbe(). Within that range only one "--release" literal exists; removing it from the args array leaves no match. Same scoping applies to mcpProbeSource for the runx-mcp-session-probe check.)
- `scripts/runtime-throughput.mjs nativeCliProbe / mcpSessionProbe`: re-verify prior finding 2: does existsSync(perfBinary) gate cargo build and let a stale prior binary be reused? -> clean (Both probes spawnSync cargo build unconditionally (lines 447-464 and 400-419). existsSync is invoked AFTER the build only to assert the binary materialized. Cargo handles freshness against the current checkout.)
- `spec acceptance evidence`: verify recorded p1_ac1/p1_ac2/p2_ac1/p2_ac2/p2_ac3 map to the changes in scope and remain coherent -> clean (All five acceptance criteria recorded as exit 0. Commands map cleanly to scripts/runtime-throughput.mjs, scripts/check-runtime-perf-harness.mjs, docs/runtime-throughput.md.)
- `ambient drift classification`: ensure ambient changes (scripts/test-boundaries.mjs, draft specs, docs/thesis.md) are not attributed to this task; ensure in-scope edits stay limited to declared paths -> clean (Spec scope explicitly excludes test-boundaries.mjs and unrelated drafts. The three ambient drift entries (docs/runtime-throughput.md, scripts/check-runtime-perf-harness.mjs, scripts/runtime-throughput.mjs) are all in declared scope.)
- `regression hunt: callers of crates/target/runx-perf and crates/target/debug`: grep oss/ for runx-perf, cargoPerfProfileDir, crates/target/debug to find consumers that could be broken by the cutover -> clean (Only scripts/runtime-throughput.mjs, scripts/check-runtime-perf-harness.mjs, and docs/runtime-throughput.md reference runx-perf or cargoPerfProfileDir. Remaining crates/target/debug references live in unrelated archived specs, README quickstart commands, docs/getting-started.md, docs/skill-to-graph.md, dogfood script, and TS test fixtures — none of those reach the perf harness path.)
- `runx-mcp-session-probe build viability under --release --features mcp`: confirm that the cargo invocation pinned by the harness can actually build a release binary: bin target exists, feature gating works, no required-features mismatch -> clean (crates/runx-runtime/src/bin/runx-mcp-session-probe.rs exists and gates real main on #[cfg(feature = "mcp")] with a stub for the no-feature branch. Cargo.toml exposes the `mcp` feature. The harness passes --features mcp, so the release binary builds the real probe. (Not actually executed in read-only review; verified by source.))
- `env preservation through cargoBenchEnv / spawnSync calls`: verify PATH and toolchain env reach cargo and the spawned probe binaries; check no env override clobbers process.env -> clean (cargoBenchEnv() spreads process.env first then sets CARGO_TARGET_DIR and CARGO_TERM_COLOR. runNativeCliProbe and measureMcpSessionProbe omit env, so they inherit process.env, preserving PATH.)
- `Windows portability`: ensure .exe suffix handled for both probes and that the static regex still passes on Windows source -> clean (Both probes branch on process.platform === "win32" to append .exe to the binary path. The check regex tests the source text (platform-independent), which contains the cross-platform args literal independent of runtime platform.)
- `docs/runtime-throughput.md release-probe invariant claim`: diff doc statements against code to verify release dir, no debug reuse, and warm-up launch description match the implementation -> clean (docs/runtime-throughput.md:60-67 describes release dir crates/target/runx-perf/release, no reuse of crates/target/debug/runx, Cargo refresh per capture, and warm-up launch before samples — all match scripts/runtime-throughput.mjs:9-10, 429-441, 444-470.)
- `configured invariants (config_from_env, domain_boundaries, no_legacy_code, no_test_logic_in_production, public_api_stable, error_envelope)`: check the diff against each invariant -> clean (No production code touched (only Node perf harness, perf-harness static check, and a doc). No public contract types, schemas, or kernel boundaries affected. No legacy fallbacks added; the crates/target/debug shortcut is removed and statically forbidden. No hardcoded secrets. No test-only logic in production code.)

Findings:
- none

