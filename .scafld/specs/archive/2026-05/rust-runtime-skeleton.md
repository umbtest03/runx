---
spec_version: '2.0'
task_id: rust-runtime-skeleton
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T02:10:42Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# Rust runtime skeleton

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T02:10:42Z
Review gate: pass

## Summary

Stand up `runx-runtime` as the first impure crate. Wire the local runner
loop, define the `Caller` and adapter traits, port the `cli-tool` adapter,
and execute `oss/examples/hello-graph/graph.yaml` end to end producing a
deterministic post-cutover harness receipt that matches the checked-in
post-cutover runtime fixture.

The runtime crate owns side effects: filesystem, subprocess, network IO,
sandbox enforcement, MCP, and adapter concurrency. Pure crates feed it
contracts and decisions; runtime translates decisions into effects.

Section 13 of `docs/rust-kernel-architecture.md` reserves `runx-cli` as a
launcher until the runtime exists and one adapter is ported. This spec
satisfies that prerequisite. It does not flip the launcher; that is
`rust-cli-rust-cutover`.

## Context

CWD: `.`

Packages:
- `@runxhq/runtime-local`
- `@runxhq/adapters` (cli-tool adapter only)
- `@runxhq/core` (executor + state-machine + policy)
- `crates/runx-runtime`
- `crates/runx-contracts`
- `crates/runx-core`
- `crates/runx-parser`
- `crates/runx-receipts`

Current TypeScript sources:
- `packages/runtime-local/src/runner-local/index.ts`
- `packages/runtime-local/src/runner-local/orchestrator/`
- `packages/runtime-local/src/runner-local/execution-semantics.ts`
- `packages/runtime-local/src/runner-local/process-sandbox.ts`
- `packages/runtime-local/src/runner-local/caller-adapters.ts`
- `packages/adapters/src/cli-tool/*`

Files impacted:
- `crates/runx-runtime/Cargo.toml`
- `crates/runx-runtime/src/lib.rs`
- `crates/runx-runtime/src/runner.rs`
- `crates/runx-runtime/src/caller.rs`
- `crates/runx-runtime/src/adapter.rs`
- `crates/runx-runtime/src/adapters/cli_tool.rs`
- `crates/runx-runtime/src/sandbox.rs`
- `crates/runx-runtime/src/journal.rs`
- `crates/runx-runtime/src/receipts.rs`
- `crates/runx-runtime/tests/hello_graph.rs`
- `crates/runx-runtime/tests/parity/**`
- `fixtures/runtime/hello-graph/**`

Invariants:
- TypeScript runner-local remains the behavioral reference until cutover specs
  replace consumers, but old TS local receipt JSON is not authoritative for the
  post-cutover harness receipt shape.
- The skeleton ports only `cli-tool`. Other adapters (`agent`, `catalog`,
  `a2a`, `mcp`) are their own specs.
- The skeleton produces harness receipts that pass `runx-receipts::verify`.
  Runtime parity tests compare against a checked-in post-cutover fixture with
  deterministic timestamps and receipt IDs.
- The skeleton enforces the Phase 1 process boundary: cwd resolution, env
  allowlisting, sandbox-profile admission, explicit rejection of unsupported
  hard-enforcement requests, and receipts that do not overclaim OS isolation.
  Platform helpers such as seatbelt, bubblewrap, and AppContainer are owned by
  later adapter-isolation specs.
- The skeleton exposes a sync facade for the CLI placeholder. Async/Tokio
  migration belongs to the adapter parity specs that need network IO.
- `runx-runtime` defaults to no adapter features; `cli-tool` is opt-in via
  `--features cli-tool`.

## Objectives

- Define the `Caller` trait (report, resolve, log) and adapter trait.
- Port the runner loop covering start, step, fanout, resume, terminal.
- Port process-boundary sandbox admission to Rust (`std::process::Command`,
  cwd resolution, env allowlist, unsupported hard-enforcement rejection).
- Port `cli-tool` adapter end to end.
- Run `oss/examples/hello-graph/graph.yaml` to a green receipt.
- Add parity fixtures that compare Rust runner output against the checked-in
  post-cutover runtime fixture for hello-graph.

## Scope

In scope:
- Runner loop, adapter trait, cli-tool adapter, sandbox, receipts emission,
  caller reporting.
- Hello-graph smoke test.
- Resume/replay primitives sufficient for `rust-runtime-fanout-parity` to
  build on.

Out of scope:
- MCP, agent, catalog, a2a adapters.
- Cloud connectivity (approval routing, registry client).
- CLI argument parsing or presentation. Runtime exposes a programmatic API
  callable from `runx-cli` once the launcher flips.
- Authoring helpers.

## Dependencies

- `rust-contracts-parity`, `rust-parser-parity`,
  `runx-contract-spine-hard-cutover`, `rust-receipts-parity`.

Sequencing:

- This spec can build an internal runtime skeleton earlier, but it cannot be
  used as a preservation dogfood or launcher-cutover gate until
  `rust-receipts-parity` targets post-cutover harness receipts.

## Open Questions

- Whether the runner exposes a sync facade or async-only. Default: async
  with a `blocking` helper used by `runx-cli` once it natively links.
- How platform-specific hard isolation composes with process-boundary sandbox
  admission (seatbelt, bubblewrap, AppContainer). Later adapter-isolation specs
  own those helpers.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode review of rust-runtime-skeleton. Prior blockers and findings N1/N2/N3 are resolved; N4 (hardcoded `RuntimeOptions::default().created_at`) is unchanged and remains a low non-blocking carryover. N1: `lib.rs:23-24` now gates `pub use runner::run_graph_file;` behind `#[cfg(feature = "cli-tool")]`, restoring the default-feature build. N3: `runner.rs:218` now calls `fanout_policies(graph)` (graph.rs:53-59), which maps `graph.fanout_groups` into `FanoutGroupPolicy`. N2: `fixtures/runtime/hello-graph/summary.json` now also pins `createdAt`, `sealDigest`, and `sandboxProfile`, and the parity test asserts those for parent + every child receipt — this satisfies the minimum validation path the prior review explicitly accepted. The substance gap (idempotency keys, signature payload, decision IDs, full canonicalized receipt tree) still leaves regressions outside those four fields unenforced, but that was the secondary option, not the minimum bar. No new regressions introduced by the fixes: tests gate themselves with `#![cfg(feature = "cli-tool")]`, sequential graphs see an empty policy map identical to prior behavior, and the receipt validators still run via `validate_harness_receipt` / `validate_receipt_tree` inside `receipts.rs`. No review self-mutation observed.

Attack log:
- `crates/runx-runtime/src/lib.rs feature-gate consistency (prior N1)`: Verify the unconditional re-export of run_graph_file has been gated behind #[cfg(feature = "cli-tool")] so the default-feature workspace build compiles -> clean (lib.rs:23-24 now reads `#[cfg(feature = "cli-tool")]\npub use runner::run_graph_file;` and lib.rs:25 re-exports only the always-available items `{GraphCheckpoint, GraphRun, Runtime, RuntimeOptions, StepRun}`. The free function in runner.rs:162-169 stays gated. Integration tests (tests/hello_graph.rs:1 and tests/parity.rs:1) both carry `#![cfg(feature = "cli-tool")]`, so a no-features `cargo check -p runx-runtime` no longer pulls in the missing symbol. CI's `cargo test --workspace` and `cargo clippy --workspace --all-targets` should now compile the crate with default features.)
- `crates/runx-runtime/tests/parity/hello_graph.rs + fixtures/runtime/hello-graph/summary.json (prior N2)`: Verify the fixture and parity test now pin the minimum receipt determinism fields (created_at, seal digest, sandbox profile) the prior validation accepted -> clean (summary.json now carries createdAt='2026-05-18T00:00:00Z', sealDigest='sha256:runtime-skeleton', sandboxProfile='process-boundary' alongside graphName/state/stepIds/stdout/graphReceiptId/childReceiptIds. The parity test asserts created_at, seal.digest, and harness.enforcement.sandbox.profile for both `run.receipt` and every child step receipt (tests/parity/hello_graph.rs:44-57). This matches the prior round's explicit minimum-validation path. The fuller canonical receipt-tree comparison (idempotency, signature payload, decision IDs) is still not pinned by parity, but that was the secondary option, not the accepted minimum.)
- `crates/runx-runtime/src/runner.rs fanout_policies plumbing (prior N3)`: Verify the empty BTreeMap::new() call has been replaced with a real policy map sourced from graph.fanout_groups -> clean (runner.rs:218 now reads `let fanout_policies = fanout_policies(graph);`, calling graph.rs:53-59 which maps every `(group_id, policy)` in `graph.fanout_groups` through `fanout_policy()` (graph.rs:128-137). The conversion threads strategy, min_success, on_branch_failure, threshold_gates, and conflict_gates from parser to core types (graph.rs:144-188). Hello-graph still gets an empty map because it declares no fanout groups, so sequential behavior is preserved. The fanout-parity spec inherits a real policy map to work with.)
- `crates/runx-runtime/src/runner.rs RuntimeOptions::default created_at (prior N4)`: Re-check whether a Clock injection point or non-hardcoded timestamp was added on the default path -> finding (runner.rs:32 still pins `created_at: "2026-05-18T00:00:00Z".to_owned()` in `Default for RuntimeOptions`. The cli-tool free function uses `RuntimeOptions::default()` (runner.rs:166), so default-constructed runs continue to produce receipts at that fixed timestamp. Carried forward as N4.)
- `crates/runx-runtime/src/runner.rs run_one_step branching (prior F1 regression check)`: Confirm step failure still dispatches StepFailed instead of StepSucceeded and the fix has not regressed under the cfg-gate refactor -> clean (runner.rs:294-305 branches on `run.output.succeeded()`. Success path: `succeed_step` issues SequentialGraphEvent::StepSucceeded and pushes run, then records completed_event. Failure path: `fail_step` issues SequentialGraphEvent::StepFailed with `output_error(run)`, calls `caller.log("step ... failed")`, and returns `RuntimeError::SkillFailed`. No double-counting, no skipped state transition.)
- `crates/runx-runtime/src/runner.rs resume/replay correctness (prior F4 regression check)`: Trace `run_graph_file_until_steps` -> `resume_graph_file` handoff for double-execution or step skipping after the cfg-gate refactor -> clean (GraphExecution::run() (runner.rs:207-235) snapshots `initial_step_count = self.runs.len()` and uses `reached_step_limit(initial, current, max_new_steps)` (runner.rs:370-372) so resume from a checkpoint with N runs accepts only `max_new_steps` more before returning. `from_checkpoint` (runner.rs:189-205) preserves `state`, `runs`, `journal` and rejects mismatched graph names with `RuntimeError::CheckpointGraphMismatch`. The integration test `hello_graph_resumes_from_checkpoint` (tests/hello_graph.rs:36-54) exercises the path and asserts both steps run.)
- `crates/runx-runtime/src/adapters/cli_tool.rs subprocess hygiene`: Look for stdin deadlocks, child orphaning on timeout, or unbounded output buffering -> clean (cli_tool.rs:64-79 always takes stdin and drops it (closing the write end so children that read stdin to EOF won't hang) even when input_mode != "stdin". Timeout path (cli_tool.rs:81-103) polls via try_wait every 10ms and kills the child on expiry before falling through to wait_with_output, so timed-out children are reaped. Output is truncated to 1 MiB via `truncate_utf8` (cli_tool.rs:105-108) using `String::from_utf8_lossy` to survive UTF-8 boundary splits. No deadlock or orphan risk in the skeleton scope.)
- `crates/runx-runtime/src/sandbox.rs env allowlisting (prior F5 regression check)`: Confirm env_clear + allowlist still blocks ambient secrets and that the sandbox-profile gate still rejects require_enforcement=true -> clean (cli_tool.rs:34 calls `.env_clear()` before `.envs(&sandbox.env)`. `allowed_base_env` (sandbox.rs:85-106) filters to PATH/SystemRoot/PATHEXT plus the optional `sandbox.env_allowlist`. `validate_sandbox` (sandbox.rs:108-125) rejects `require_enforcement = true` with `SandboxViolation`, and admits only readonly/workspace-write/network/unrestricted-local-dev profiles. Receipt enforcement reports `profile: process-boundary`, `network: declared-by-skill`, `filesystem: declared-by-skill` (receipts.rs:241-258), matching the actual posture rather than overclaiming OS isolation.)
- `.scafld/specs/active/rust-runtime-skeleton.md self-mutation`: Check whether the active spec or task-scoped files were mutated during review -> clean (Workspace classification reports baseline 'clean', task changes since approval 'none', and the ambient drift list (18 entries) does not include the active spec at .scafld/specs/active/rust-runtime-skeleton.md. No review-time mutation observed against the spec or task-scoped files.)
- `crates/runx-receipts ambient changes (tree.rs, finding.rs, harness_receipts.rs)`: Skim for task-scope overlap with the runtime skeleton's receipt emission -> skipped (Adjacent receipts-crate work is declared out of task scope per the workspace classification and was clean in the prior round; not in this verify pass's scope.)

Findings:
- [low/non-blocking] `N4` RuntimeOptions::default() still hardcodes created_at to 2026-05-18T00:00:00Z (prior round F8/N4 unchanged)
  - Location: `crates/runx-runtime/src/runner.rs:32`
  - Evidence: runner.rs:29-36 keeps `Default for RuntimeOptions` returning `created_at: "2026-05-18T00:00:00Z".to_owned()`. The feature-gated `run_graph_file` free function (runner.rs:163-168) uses `RuntimeOptions::default()`, so any default-constructed runtime produces receipts pinned to that timestamp. No `Clock` trait, no `Option<String>` opt-in, no chrono::Utc::now() path was added. The parity fixture (`fixtures/runtime/hello-graph/summary.json`) and parity test (`tests/parity/hello_graph.rs:44,51`) compare exactly this literal, so any future attempt to replace the hardcode with a real clock will fail parity until the test is rewritten.
  - Impact: Determinism is convenient for the parity gate but misleads downstream consumers (and the future `runx-cli` launcher) that read `receipt.created_at` from default-constructed runs — every receipt will be timestamped May 18, 2026. Same severity as the prior round; the spec amendment around the sync facade did not address this.
  - Validation: After the fix, default-constructed `RuntimeOptions::default()` returns a non-fixed timestamp (e.g., chrono::Utc::now()), and parity tests pass a `FixedClock("2026-05-18T00:00:00Z")` explicitly. `cargo test --features cli-tool -p runx-runtime` continues to pass.
