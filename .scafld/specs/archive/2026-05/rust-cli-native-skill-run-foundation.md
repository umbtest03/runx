---
spec_version: '2.0'
task_id: rust-cli-native-skill-run-foundation
created: '2026-05-20T09:40:11Z'
updated: '2026-05-20T10:17:03Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Rust CLI native skill run foundation

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-20T10:17:03Z
Review gate: pass

## Summary

Implement the minimal Rust-native `runx skill <skill-package>` foundation
needed by consumer dogfood without rushing the whole CLI launcher cutover.

This spec is a child slice of `rust-cli-rust-cutover`. It does not claim full
CLI release authority, native registry publication, native `resume`, or all
skill subcommands. It adds one clean execution shape:
`runx skill <skill-package> [--receipt-dir <dir>] [--run-id <id> --answers <file>] --json --non-interactive`.

The first call returns a stable caller-resolution report with `status:
needs_agent`, `run_id`, and `agent_act` requests and exits nonzero so callers
keep the agent loop open. The resumed call reruns the same `runx skill` command
with `--run-id` and `--answers`, then seals a `runx.harness_receipt.v1` through
runtime receipt code and returns `status: sealed`.

No legacy aliases are introduced. `runx skill run ...`, `--receipt`, and
camelCase `--receiptDir` fail closed.

## Context

Grounded facts at creation:

- nitrosend and Aster have both landed no-JS delegation guardrails and Rust
  harness dogfood, but both reported live caller-mediated lanes remain blocked
  until Rust exposes native `runx skill`.
- Existing Rust harness replay proves fixture determinism, but it cannot run a
  skill package `X.yaml` as the consumer bridge needs.
- The prior local implementation placed too much execution behavior in
  `crates/runx-cli/src/skill.rs`. That violates the runtime boundary: CLI may
  parse and render, but runtime owns manifest loading, resolution requests,
  receipt path resolution, and harness receipt sealing.
- `crates/runx-runtime/src/adapters/agent.rs` already owns the canonical
  `AgentActInvocation` envelope builder for managed agent execution; native
  skill pause/resume must reuse that logic rather than copy it.

## Invariants

- CLI only parses command-line shape and writes JSON/stdout/stderr.
- `runx-runtime` owns skill package loading, runner selection, caller request
  construction, answer ingestion, receipt path resolution, and receipt writes.
- Native skill runs use the same `agent_act` request envelope as the runtime
  managed-agent adapter.
- Resumed runs seal `runx.harness_receipt.v1` through
  `LocalReceiptStore`/runtime receipts, never through ad hoc CLI JSON writes.
- Act/decision payloads are provable only through the sealing harness receipt.
- Closure is not success-by-default: an answer carrying a non-closed
  `closure.disposition` must produce a receipt with that disposition.
- Top-level resumed status is `sealed`, not `success`; closure outcome is read
  from `closure.disposition` and the embedded harness receipt.
- Continuation flags are paired: `--run-id` without `--answers`, and `--answers`
  without `--run-id`, both fail closed.
- Unsupported skill source types fail closed. This slice intentionally covers
  `agent` and `agent-step`; graph-backed and tool-backed skill execution remain
  separate runtime slices.
- No compatibility shims or aliases: no `skill run`, no `--receipt`, no
  `--receiptDir`, no JS fallback inside this native shape.

## Objectives

- Add a runtime-owned `execute_skill_run` API for the minimal
  pause/resume/seal loop.
- Share the runtime agent invocation builder between managed-agent adapters and
  native skill pause requests.
- Add `runx skill <path>` launcher routing under `RUNX_RUST_CLI=1`.
- Keep `crates/runx-cli/src/skill.rs` thin and free of manifest parsing,
  runner selection, and receipt construction.
- Add runtime and CLI tests that prove pause output, resume sealing, receipt
  path resolution, legacy flag rejection, and launcher routing.
- Preserve consumer-facing skill names such as `issue-to-pr`; those are product
  surface names, not retired contract nouns.

## Scope

In scope:

- `crates/runx-runtime/src/skill_run.rs`
- Shared runtime agent invocation helper module
- `crates/runx-cli/src/skill.rs`
- Launcher routing and CLI tests for canonical native skill execution
- Runtime tests for pause/resume/receipt-path behavior
- A focused nitrosend smoke using the Rust binary after canonical `--run-id`
  adoption
- A focused Aster bridge update/smoke once it accepts the new canonical
  pause/resume shape

Out of scope:

- Full native release packaging and signer metadata
- Removing all JS fallback code from the launcher
- Native `runx resume`
- Hosted/cloud queue persistence for pending skill runs
- Graph-backed skill execution and native skill marketplace subcommands
- Legacy pre-cutover receipt shapes

## Dependencies

- `runx-contract-spine-hard-cutover` shape is the target: harness receipts,
  decision/act payloads, closure, no effect/outcome/result buckets.
- `rust-receipts-parity` supplies harness receipt verification and local
  receipt store behavior.
- `rust-runtime-skeleton` supplies receipt construction and runtime adapter
  boundaries.
- nitrosend and Aster consumer cutover branches now reject JS/npm delegation and
  therefore consume this slice before full launcher cutover.

## Risks

- A shortcut that writes receipts in CLI would pass consumer tests but violate
  the trusted-kernel boundary. The implementation must be rejected if CLI owns
  manifest loading or receipt sealing.
- A separate agent envelope builder would drift from `AgentAdapter`.
- A `--receipt` alias would preserve retired semantics and reopen compatibility
  ambiguity.
- Returning success for a failed/declined/blocked closure would recreate the
  old success-biased outcome bug.
- Treating this slice as full `rust-cli-rust-cutover` would hide remaining
  release and JS fallback blockers.

## Acceptance

Profile: standard plus consumer smoke.

Validation:

- [ ] `ac1_runtime_skill_run`
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test skill_run -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2_cli_skill_run`
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test skill -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac3_launcher_skill_shape`
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test launcher -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac4_style_and_lints`
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime -p runx-cli --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac5_no_legacy_native_skill_shape`
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test launcher rust_cli_signal -- --nocapture && cargo test --manifest-path crates/Cargo.toml -p runx-cli --test skill native_skill_rejects -- --nocapture && ! rg -n -- "runx resume|needs_resolution|work_item|runx\\.engagement" crates/runx-cli/src crates/runx-runtime/src/skill_run.rs crates/runx-runtime/tests/skill_run.rs`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac6_nitrosend_smoke`
  - Command: `RUNX_RUST_CLI=1 RUNX_RUST_HARNESS=1 RUNX_BIN=/Users/kam/dev/runx/runx/oss/crates/target/debug/runx RUNX_SKILLS_ROOT=/Users/kam/dev/runx/runx/oss/skills node --test scripts/issue-intake.test.mjs scripts/segment-from-prose.test.mjs`
  - CWD: `/Users/kam/dev/nitrosend`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac7_aster_bridge_smoke`
  - Command: `node --test scripts/runx-agent-bridge.test.mjs`
  - CWD: `/Users/kam/dev/runx/aster`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 1: Runtime-Owned Skill Run

Status: completed
Dependencies: `rust-runtime-skeleton`, `rust-receipts-parity`

Objective: Complete this phase.

Changes:
- Add `runx_runtime::SkillRunRequest`.
- Add `runx_runtime::execute_skill_run`.
- Add shared `agent_invocation` helper used by both native skill pause and managed-agent adapter.
- Resolve receipt paths through `resolve_receipt_path`.
- Write receipts through `LocalReceiptStore`.

Acceptance:
- none

## Phase 2: Thin CLI Surface

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- Add `runx_cli::skill` parser/presenter.
- Route canonical `runx skill <path>` under `RUNX_RUST_CLI=1`.
- Reject `runx skill run ...`, `--receipt`, and `--receiptDir`.
- Reject partial continuation: `--run-id` requires `--answers`, and `--answers` requires `--run-id`.
- Keep unsupported skill subcommands fail-closed under the native signal.

Acceptance:
- none

## Phase 3: Consumer Smoke

Status: completed
Dependencies: Phases 1 and 2

Objective: Complete this phase.

Changes:
- Verify nitrosend uses `--run-id` for resume and still passes the caller wrapper tests against the Rust binary.
- Verify Aster bridge accepts `needs_agent` and reruns `runx skill` with `--run-id` rather than `runx resume`.

Acceptance:
- none

## Rollback

- Remove `crates/runx-cli/src/skill.rs`, the launcher `RunSkill` branch,
  `crates/runx-runtime/src/skill_run.rs`, and the shared helper.
- Revert consumer bridge updates to their last committed no-JS guardrail state.
- Full `rust-cli-rust-cutover` remains blocked either way; this rollback does
  not affect release packaging.

## Review

Status: completed
Verdict: pass
Mode: verify
Summary: Human-reviewed override accepted: Claude harden blocker was fixed; Goodall read-only review blockers were fixed; Rust runtime/CLI tests, lints, nitrosend and Aster smokes, cutover script tests, and no-legacy greps passed.

Attack log:
- `review gate`: manual human audit -> clean (Claude harden blocker was fixed; Goodall read-only review blockers were fixed; Rust runtime/CLI tests, lints, nitrosend and Aster smokes, cutover script tests, and no-legacy greps passed.)

Findings:
- none

## Self Eval

- Harden round 1 blockers patched:
  - native skill seal now calls `step_receipt_with_disposition`, preserving
    `deferred`, `declined`, `blocked`, `failed`, `killed`, and `timed_out`;
  - resumed JSON returns `status: sealed` with explicit `closure.disposition`
    instead of top-level `success`;
  - separated and inline `--receipt`/`--receiptDir` are rejected;
  - partial `--run-id`/`--answers` continuation shapes are rejected;
  - native history no longer points users to retired `runx resume`.

## Deviations

- This spec deliberately does not implement native `runx resume`; the clean
  canonical resume shape for this slice is rerunning `runx skill <path>` with
  `--run-id` and `--answers`.

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-20T09:56:53Z
Ended: 2026-05-20T10:15:30Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Draft is architecturally on-target: runtime owns load/seal, CLI is thin parser/presenter, shared agent_invocation helper is in place, and ac1–ac5 commands point to real tests. One blocker: the Phase 1 invariant "non-closed closure disposition is preserved in the receipt" is not enforced — `seal_skill_answer` routes through `step_receipt`, whose `disposition()` collapses any non-success status to `ClosureDisposition::Failed`, so `deferred`/`declined`/`blocked`/`superseded`/`killed`/`timed_out` answers silently become `failed` in the receipt. The existing test only covers `failed`, hiding the bug. Also: ac5 lacks a `--receiptDir` rejection assertion (only `--receipt` is exercised), and the JSON envelope keeps `status: "success"` plus `execution.exit_code: 0` regardless of receipt disposition, which is the exact "success-by-default" risk the spec calls out.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/skill_run.rs:1
  - Result: passed
  - Evidence: Declared paths exist: crates/runx-runtime/src/skill_run.rs (351 lines), crates/runx-runtime/src/agent_invocation.rs (161 lines), crates/runx-cli/src/skill.rs (156 lines), crates/runx-runtime/tests/skill_run.rs (222 lines), crates/runx-cli/tests/skill.rs (152 lines). skill_run module is exported from runx-runtime/src/lib.rs:31,107 and registered in launcher RunSkill branch (crates/runx-cli/src/main.rs:47, crates/runx-cli/src/launcher.rs:31,233-236).
- command audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: ac1 binds to crates/runx-runtime/tests/skill_run.rs (five #[test] functions present). ac2 binds to crates/runx-cli/tests/skill.rs (six #[test] functions). ac3 binds to crates/runx-cli/tests/launcher.rs which includes rust_cli_signal_rejects_legacy_skill_run_alias, rust_cli_signal_routes_canonical_skill_run_to_native_plan, rust_cli_signal_rejects_legacy_skill_receipt_resume_flag, rust_cli_signal_rejects_legacy_skill_camelcase_receipt_dir_flag, and rust_cli_signal_rejects_partial_skill_continuation_shape. ac5 grep now targets active source for runx resume, needs_resolution, work_item, and runx.engagement.
- scope/migration audit
  - Grounded in: code:crates/runx-cli/tests/skill.rs:118
  - Result: passed
  - Evidence: `native_skill_rejects_legacy_camelcase_receipt_dir` now exercises both `--receiptDir` and `--receiptDir=...`; `rust_cli_signal_rejects_legacy_skill_camelcase_receipt_dir_flag` covers launcher routing. Inline `--receipt=...` and separated `--receipt` are also covered, and partial `--run-id`/`--answers` shapes are rejected by CLI tests.
- acceptance timing audit
  - Grounded in: spec_gap:phase-3-consumer-smoke
  - Result: passed
  - Evidence: ac1/ac2/ac3/ac5 are local cargo invocations runnable inside this slice. ac6 and ac7 are external repo smokes (CWD=/Users/kam/dev/nitrosend and /Users/kam/dev/runx/aster) gated behind Phase 3 which explicitly depends on Phases 1 and 2 (spec line 217-220). Phase 3 changes call out updating those external bridges as a prerequisite, so timing is consistent with the dependency graph.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback names the four artifacts introduced by this slice: crates/runx-cli/src/skill.rs, the launcher RunSkill branch, crates/runx-runtime/src/skill_run.rs, and the shared agent_invocation helper. Each exists and is reachable for removal; agent.rs and managed-agent paths reuse build_agent_act_invocation/agent_act_resolution_request, which would need a revert to inline copies if the helper is removed — that follow-on is implicit. Otherwise rollback is credible and does not affect release packaging as claimed.
- design challenge
  - Grounded in: code:crates/runx-runtime/src/skill_run.rs:248
  - Result: passed
  - Evidence: `seal_skill_answer` now calls the existing runtime `step_receipt_with_disposition` seam with the answer-derived `ClosureDisposition`, and runtime tests assert `declined` and `deferred` are preserved in both the receipt seal and contained act closure. The resumed JSON now returns `status: sealed` plus explicit `closure.disposition`, so callers cannot treat a non-closed closure as top-level success.

Issues:
- [critical/blocks approval] `harden-1` invariant_violation - Non-failed non-closed dispositions are silently rewritten to `failed` in the sealed receipt.
  - Status: fixed
  - Grounded in: code:crates/runx-runtime/src/skill_run.rs:248
  - Evidence: answer_disposition correctly distinguishes deferred/superseded/declined/blocked/failed/killed/timed_out (skill_run.rs:249-265). seal_skill_answer discards that distinction by routing through step_receipt, which uses receipts.rs:581 fn disposition(output) -> Closed | Failed. A consumer answer of {"closure":{"disposition":"deferred"}} ends up with receipt.seal.disposition == "failed" and reason_code == "process_failed". This directly contradicts Invariants (spec line 71-72) and Phase 1 acceptance ("non-closed closure disposition is preserved in the receipt") and instantiates the Risk on spec line 134-135.
  - Recommendation: In seal_skill_answer, call the existing pub(crate) step_receipt_with_disposition with the answer-derived ClosureDisposition (and a matching process_<disposition> reason_code/summary) instead of step_receipt. Add ac1 sub-cases that assert receipt.seal.disposition for at least `deferred` and `declined`, since `failed` happens to be the one disposition that survives the current mapping.
  - Question: Should this slice land the disposition-preserving seal call, or is the broader fix deferred to a follow-on receipt slice?
  - Recommended answer: Land it inside this slice — Phase 1 acceptance already claims the invariant and ac1 names it; deferring would mean approving a spec whose own acceptance text is unverifiable.
  - If unanswered: Treat as required for Phase 1 approval; downgrade only if the operator agrees to remove the invariant and Phase 1 acceptance bullet from the spec.
- [medium/advisory] `harden-2` test_coverage_gap - `--receiptDir` rejection is implemented but never exercised by tests.
  - Status: fixed
  - Grounded in: code:crates/runx-cli/src/skill.rs:40
  - Evidence: skill.rs:40-44 returns the error string `runx skill uses --receipt-dir; --receiptDir is not supported`, but no test in crates/runx-cli/tests/launcher.rs or crates/runx-cli/tests/skill.rs invokes that branch. ac5 grep only guards against `work_item`/`runx.engagement` strings and does not assert the camelCase rejection. Spec line 41 explicitly lists `--receiptDir` alongside `--receipt` as a must-fail-closed alias.
  - Recommendation: Add a launcher test mirroring rust_cli_signal_rejects_legacy_skill_receipt_resume_flag using `--receiptDir`, and an end-to-end test in tests/skill.rs analogous to native_skill_rejects_legacy_receipt_resume_flag.
  - If unanswered: Add the two assertions before approval; rejection branches without tests rot quickly.
- [medium/advisory] `harden-3` design_challenge - JSON envelope reports `status: "success"` and `execution.exit_code: 0` even when the receipt disposition is non-closed.
  - Status: fixed
  - Grounded in: code:crates/runx-runtime/src/skill_run.rs:292
  - Evidence: success_output unconditionally writes `status: success` and `execution.exit_code: I64(0)` (skill_run.rs:294,303). seal_skill_answer sets exit_code to None and InvocationStatus::Failure for non-closed dispositions, but the JSON envelope ignores that signal and only the embedded receipt carries the disposition. Consumer wrappers that branch on the top-level `status` field will treat a `failed`/`declined` closure as successful — the exact "success-by-default outcome bug" the spec lists as a risk (line 134-135).
  - Recommendation: Decide explicitly whether the top-level `status` mirrors the receipt disposition (e.g., success only when Closed, otherwise `needs_revision`/`failed`/`deferred`) or whether the JSON shape intentionally separates execution-completed from closure-outcome. Whichever is chosen, document it on the invariant and add a test that pins the chosen shape for at least one non-closed disposition. Today both nitrosend and Aster will see `success` for declined/blocked closures.
  - Question: Should the runtime envelope `status` reflect the closure disposition, or only whether the harness ran end-to-end?
  - Recommended answer: Map `status` from the receipt disposition (Closed → `success`, Deferred/Superseded → `paused` or `needs_revision`, the rest → `failed`) so the JSON shape and the receipt cannot disagree.
  - If unanswered: Add a spec invariant explicitly stating that JSON `status` reflects only end-to-end execution and that consumers must read receipt.seal.disposition for closure outcome — then add a regression test asserting `status: success` with `receipt.seal.disposition: failed`.
- [low/advisory] `harden-4` shape_consistency - First-call default run_id normalizes dots to dashes while the matching request id keeps dots; consumers may struggle to derive one from the other.
  - Status: accepted_risk
  - Grounded in: code:crates/runx-runtime/src/skill_run.rs:52
  - Evidence: agent_act_invocation_id returns e.g. `agent_step.issue-intake.output` (dots preserved). identifier_segment in skill_run.rs:334-338 replaces dots with dashes, so the default run_id becomes `run_agent_step-issue-intake-output` (asserted by the runtime test). The first-call JSON gives consumers a run_id whose textual relationship to requests[0].id is non-trivial, even though both ultimately resolve to the same logical entity.
  - Recommendation: Either keep the dot-preserving form for the default run_id (matching the request id with a `run_` prefix) or call out the normalization explicitly in the spec/consumer notes. The current asymmetry is harmless if consumers always echo `run_id` verbatim on resume, but it is worth pinning behavior in a test comment.
  - If unanswered: Leave as-is and pin the normalization with a comment in skill_run.rs and a sentence in the spec invariants.


## Planning Log

- 2026-05-20T10:25:00Z: Filled skeleton spec after nitrosend/Aster both
  reported native `runx skill` as the remaining Rust dogfood blocker.
