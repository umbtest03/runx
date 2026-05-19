---
spec_version: '2.0'
task_id: runx-target-repo-runners
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T02:30:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Runx target repo runners

## Current State

Status: draft
Current phase: none
Next: harden
Reason: issue intake may originate from one repo/thread while the correct fix
belongs in another repo. Today that shape is mostly adopter script behavior;
runx core should own the reusable routing, runner, dedupe, and source-thread
linking contract.
Blockers: `runx-operational-policy-config` approved; target repos must be
explicitly allowed and scafld-ready.
Allowed follow-up command: `scafld harden runx-target-repo-runners`
Latest runner update: none
Review gate: not_started

## Summary

Add first-class target-repo runner support to runx issue-to-PR flows. A source
issue or Slack/Sentry intake can select a target repo, run the right governed
runner in that repo, create or update the PR there, and publish all milestone
links back to the original source issue/thread.

This preserves the security posture: automation can produce a PR, but human
review remains the merge gate unless a future policy explicitly enables a
separate merge workflow.

## Context

CWD: `.` (runx OSS workspace)

Production shape to generalize:
- Source thread: Slack/GitHub/Sentry-originated issue intake.
- Source issue: a GitHub issue used as durable runx source thread.
- Target repo: the actual codebase receiving the PR.
- Runner: a governed scafld/runx worker inside the target repo.
- Follow-up publishing: source Slack thread and source GitHub issue receive the
  issue link, triage result, PR link, review gate, and outcome.

Candidate touchpoints:
- `skills/issue-intake/**`
- `skills/issue-to-pr/**`
- `skills/work-plan/**`
- `packages/cli/**`
- `packages/cli/tools/outbox/**`
- `crates/runx-runtime/**`
- Aster runner scheduling/readback

Invariants:
- Target repo selection is policy-driven and fail-closed.
- Runner availability is explicit; no hidden fallback to the source repo.
- A target repo must be scafld-ready before mutating issue-to-PR work runs.
- Dedupe runs before creating a new PR.
- PR packaging records `metadata.dedupe.strategy`, `metadata.dedupe.key`, and
  `metadata.dedupe.result` so retries are auditable and provider pushers can
  reuse an existing PR path.
- Source issue/thread ids are carried through every milestone event.
- Public comments use repo names and URLs, not operator-local checkout paths.

## Objectives

- Define target repo runner model and source-to-target context contract.
- Add policy-backed target repo selection and runner availability checks.
- Add PR dedupe before creating a new branch/PR.
- Preserve source issue/thread metadata through runner execution and outbox
  publishing.
- Add fixtures for same-repo, cross-repo, no-runner, not-allowed, not-scafld,
  and duplicate-PR cases.

## Scope

In scope:
- Core issue-to-PR target repo runner contract.
- Policy-backed repository and runner selection.
- Dedupe query and reuse behavior.
- Source-thread/source-issue carry-through.
- Tests and fixtures.

Out of scope:
- Post-merge deployment observation; owned by
  `runx-post-merge-outcome-observer`.
- Slack-specific event listener implementation.
- Auto-merge behavior.
- Nitrosend-only wrapper script details except as fixture input.

## Dependencies

- `runx-operational-policy-config`.
- `rust-runtime-skeleton` for Rust runtime execution path, or current TS runner
  during staged migration.
- `rust-runtime-skill-execution` for Rust skill execution parity.
- `rust-nitrosend-dogfood` consumes this as the production proof point.

## Assumptions

- GitHub is the first provider target; the contract should not preclude other
  repository providers later.
- Target checkouts may be local worktrees, CI-provided paths, or Aster-managed
  sandboxes, but the runner contract must hide those details from public output.

## Touchpoints

- Issue intake contract payload.
- Issue-to-PR runner context.
- GitHub adapter/outbox event builders.
- Policy config loader.
- Dedupe provider query.
- Aster runner scheduling.

## Risks

- Running in the wrong repo is a high-impact failure.
- Dedupe bugs can spam reviewers with duplicate PRs.
- Losing source-thread metadata recreates noisy Slack root-channel posts.
- Hidden fallback behavior can bypass policy.

## Acceptance

Profile: strict

Validation:
- `pnpm test`
- `cargo test --manifest-path crates/Cargo.toml`
- target-runner fixture command
- `git diff --check`

Required behavior:
- [ ] Same-repo issue-to-PR still works through the target runner contract.
- [ ] Cross-repo issue-to-PR creates the PR in the configured target repo.
- [ ] Unknown target repo fails before checkout or mutation.
- [ ] Missing runner fails before checkout or mutation.
- [ ] Target repo without scafld readiness fails before mutation.
- [ ] Existing open PR for the dedupe key is reused and linked instead of
  creating another PR.
- [ ] Pull-request outbox metadata records whether the PR path was created or
  reused for the dedupe key.
- [ ] Source GitHub issue receives the target PR link.
- [ ] Source Slack thread metadata survives through all outbox events.
- [ ] Public output excludes local checkout paths and env-secret values.

## Phase 1: Contract

Status: pending
Dependencies: `runx-operational-policy-config`

Objective: Define the target runner context and fail-closed selection rules.

Changes:
- Add source/target context types.
- Add policy-backed target selection.
- Add scafld readiness and runner availability checks.

Acceptance:
- [ ] Fixtures cover allowed, denied, missing runner, and not-scafld targets.

## Phase 2: Runner Execution

Status: pending
Dependencies: Phase 1

Objective: Execute issue-to-PR work in the selected target repo.

Changes:
- Thread target repo context into runner invocation.
- Carry source issue/thread metadata through execution.
- Ensure public output uses URLs and repo names.

Acceptance:
- [ ] Cross-repo fixture produces target PR event and source issue/thread event.

## Phase 3: Dedupe

Status: pending
Dependencies: Phase 2

Objective: Avoid duplicate PRs for the same issue/fix.

Changes:
- Add dedupe key generation and provider lookup.
- Reuse/link existing PR when policy says so.
- Preserve the dedupe decision in the pull-request outbox entry metadata.

Acceptance:
- [ ] Duplicate fixture reuses the existing PR and produces no new branch.

## Rollback

- Keep current same-repo path available only until target runner parity is green.
  Remove duplicate old path during cutover; no legacy aliases or compatibility
  shims remain.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- Target score: 9.5. Passing means cross-repo issue-to-PR is a core runx
  capability with explicit policy and reviewable evidence.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: move reusable target repo execution out of adopter scripts

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-05-19: Expanded placeholder into target-repo runner contract after
  Nitrosend source-thread dogfood review.
- 2026-05-19: Locked the PR dedupe packet shape to outbox metadata so the Rust
  runner and provider pushers keep retry behavior observable.
