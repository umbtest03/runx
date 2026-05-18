---
spec_version: '2.0'
task_id: issue-to-pr-build-to-review
created: '2026-05-18T02:55:50Z'
updated: '2026-05-18T03:03:28Z'
status: completed
harden_status: not_run
size: small
risk_level: medium
---

# Use bounded build-to-review in issue-to-pr

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-18T03:03:28Z
Review gate: pass

## Summary

`issue-to-pr` currently performs two fixed `scafld build` calls before
running `scafld review`. That is brittle for native scafld 2.4 specs because
`scafld build` opens and completes one lifecycle phase at a time. The scafld
skill already documents a `build_to_review` command, but the runner does not
implement it and the issue-to-pr graph does not use it. This caused the live
Nitrosend issue-to-PR run to reach `scafld review` while the task still needed
another native build advance.

## Objectives

- Implement `build_to_review` in the scafld runx wrapper as a bounded loop over
  native `scafld build <task-id> --json`.
- Move the issue-to-pr graph from fixed build calls to `build_to_review` before
  invoking `scafld review`.
- Preserve native scafld JSON payloads and failure semantics; do not smooth over
  build failures.
- Keep the PR/story packaging context pointed at the final build-to-review
  result.

## Scope

- `skills/scafld/run.mjs`
- `packages/cli/skills/scafld/run.mjs`
- `skills/issue-to-pr/X.yaml`
- `tests/scafld-skill.test.ts`
- `tests/scafld-skill-parser.test.ts`
- `tests/scafld-issue-to-pr-parser.test.ts`
- `tests/issue-to-pr-graph.test.ts`

## Dependencies

- Native scafld 2.4+ JSON contracts for `build` and `status`.
- Existing runx graph execution and outbox tools.

## Assumptions

- `build_to_review` should stop only when a native build result or follow-up
  status reports `status: review` or `status: completed`.
- A native non-zero build exits the wrapper non-zero with the native payload
  preserved on stdout.
- `max_builds` defaults to 12 and is bounded to a positive integer.

## Touchpoints

- scafld wrapper command allowlist and argv handling.
- issue-to-pr graph step ids and downstream build result references.
- Parser and runtime tests that assert graph shape and lifecycle behavior.

## Risks

- A wrapper loop could hide a native lifecycle failure if it normalizes results;
  mitigate by preserving the native failed payload and exit code.
- Existing tests and packaging expect exact issue-to-pr step names; update them
  with the intentional hard cut.
- Local runx worktree contains unrelated dirty Rust/kernel work; stage only this
  issue-to-pr/scafld fix.

## Acceptance

Profile: standard

Validation:
- [x] `v1` test - scafld wrapper contract and build_to_review behavior.
  - Command: `pnpm vitest run tests/scafld-skill.test.ts tests/scafld-skill-parser.test.ts --config vitest.config.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-9
- [x] `v2` test - issue-to-pr graph contract and parser behavior.
  - Command: `pnpm vitest run tests/issue-to-pr-graph.test.ts tests/scafld-issue-to-pr-parser.test.ts --config vitest.config.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-10
- [x] `v3` check - no whitespace errors in changed files.
  - Command: `git diff --check -- skills/scafld/run.mjs packages/cli/skills/scafld/run.mjs skills/issue-to-pr/X.yaml tests/scafld-skill.test.ts tests/scafld-skill-parser.test.ts tests/scafld-issue-to-pr-parser.test.ts tests/issue-to-pr-graph.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-11

## Phase 1: Implementation

Status: completed
Dependencies: none

Objective: Make the scafld wrapper and issue-to-pr graph honor native

Changes:
- Implement `build_to_review` in both shipped scafld wrapper copies.
- Replace the two fixed issue-to-pr build steps with one bounded build-to-review step.
- Update all graph packaging references to the new build result.
- Add tests for multi-build advancement and non-zero build preservation.

Acceptance:
- [x] `ac1` command - Targeted tests pass.
  - Command: `pnpm vitest run tests/scafld-skill.test.ts tests/scafld-skill-parser.test.ts tests/issue-to-pr-graph.test.ts tests/scafld-issue-to-pr-parser.test.ts --config vitest.config.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6

## Rollback

- Revert the scoped wrapper, graph, and test changes. issue-to-pr would return
  to the older fixed two-build behavior.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: codex
Output: codex.output_file
Summary: No completion-blocking findings found. The scoped changes implement a bounded `build_to_review` wrapper path, update issue-to-pr to use it before review, keep downstream packaging pointed at `scafld-build.result`, and add focused tests for multi-build advancement and failure preservation. Acceptance evidence was treated as already executed per review instructions.

Attack log:
- `task scope`: Scoped change inventory -> clean (Reviewed task-scoped diff for `skills/scafld/run.mjs`, `packages/cli/skills/scafld/run.mjs`, `skills/issue-to-pr/X.yaml`, and the four scoped tests. Ambient drift was identified but not treated as task evidence.)
- `skills/scafld/run.mjs`: Bounded lifecycle loop -> clean (Inspected `runBuildToReview`: it caps native build attempts, invokes `scafld build <task-id> --json`, checks status between successful attempts, and exits once status is `review` or `completed`.)
- `skills/scafld/run.mjs`: Native failure preservation -> clean (Verified non-zero native build results return the native structured stdout and exit code without converting failures into wrapper success.)
- `skills/scafld/run.mjs`: JSON parsing behavior -> clean (Checked native JSON parsing, including the added final-object extraction for provider progress output. The extraction still preserves the final JSON envelope used by downstream steps.)
- `skills/issue-to-pr/X.yaml`: Issue-to-PR graph sequencing -> clean (Confirmed the two fixed build steps were replaced by one `scafld-build` step using `command: build_to_review`, followed by status, review, complete, final status, and handoff.)
- `skills/issue-to-pr/X.yaml`: Packaging references -> clean (Confirmed PR and source-thread story packaging both reference `scafld-build.result`, so they consume the final build-to-review result rather than a removed fixed build step.)
- `tests/scafld-skill.test.ts and issue-to-pr graph/parser tests`: Regression tests -> clean (Inspected tests asserting graph shape, parser contract, multi-build advancement, and native build failure preservation. Per provider instruction, recorded acceptance evidence was not rerun.)
- `configured invariants`: Convention and invariant check -> clean (Checked against AGENTS.md and CONVENTIONS.md for scoped diffs, no hardcoded secrets, no test-only production logic, no public API break beyond declared skill command addition, and no lifecycle state scraping from Markdown.)

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

- none

## Planning Log

- none
