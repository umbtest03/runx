---
spec_version: '2.0'
task_id: runx-post-merge-outcome-observer
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T02:30:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Runx post-merge outcome observer

## Current State

Status: draft
Current phase: none
Next: harden
Reason: issue-to-PR currently tells a good story through PR creation, but the
final outcome is still too dependent on repo-local glue. runx core needs an
observer for merge, close, deploy verification, final source-thread update, and
issue closure.
Blockers: `runx-operational-policy-config`; target/source context from
`runx-target-repo-runners` for cross-repo flows.
Allowed follow-up command: `scafld harden runx-post-merge-outcome-observer`
Latest runner update: none
Review gate: not_started

## Summary

Add a reusable post-merge outcome observer for runx issue-to-PR flows. It
observes PR merge/close state, runs policy-defined verification, records the
outcome in receipts/events, updates the source GitHub issue, posts the final
Slack/source-thread reply, and closes or marks the issue according to policy.

The observer does not auto-merge. Human merge remains the default final gate;
the observer publishes what happened after that gate.

## Context

CWD: `.` (runx OSS workspace)

Production story to support:
1. Intake creates a source issue from Slack/Sentry/GitHub.
2. runx triages and creates or links a target PR.
3. Human reviewer approves/merges or closes the PR.
4. runx observes the result.
5. runx runs verification appropriate to the target.
6. runx posts a concise final outcome to the original Slack thread and source
   GitHub issue.
7. runx closes or labels the issue when policy allows.

Candidate touchpoints:
- GitHub adapter/outbox event builders.
- `skills/issue-to-pr/**`
- `skills/work-plan/**`
- Runtime receipt/event model.
- Aster observer scheduling and status surfaces.

Invariants:
- Observer is idempotent by issue/PR/outcome key.
- Source thread metadata must be present before Slack publishing.
- Closed-unmerged, merged-unverified, merged-verified, failed-verification, and
  superseded states are distinct.
- Verification output is reviewer-safe and redacted.
- No hidden auto-merge path is introduced.
- Terminal observer output normalizes to `runx.issue_to_pr_outcome.v1` before
  any source issue closure or final source-thread publication.

## Objectives

- Define outcome event model for merged, closed-unmerged, superseded,
  verification-passed, and verification-failed.
- Define the `runx.issue_to_pr_outcome.v1` packet for provider outcome, PR
  state, human gate, verification, close policy, and source-thread target.
- Add provider observer for GitHub PR state changes.
- Add policy-driven verification hook.
- Publish final outcome to source GitHub issue and Slack/source thread.
- Add idempotency/dedupe for repeated webhook or scheduled observer runs.
- Add fixtures for merged verified, merged failed verify, closed unmerged,
  missing source thread, and repeated observer events.

## Scope

In scope:
- Core outcome observer contract.
- GitHub PR state observer.
- Policy-driven verification command/hook contract.
- Final issue and source-thread publishing.
- Issue close/label behavior when policy allows.
- Tests and fixtures.

Out of scope:
- Automatic PR merge.
- Provider-specific deployment integrations beyond a hook boundary.
- Slack listener/reaction intake.
- Nitrosend-only script details except as reference fixtures.

## Dependencies

- `runx-operational-policy-config`.
- `runx-target-repo-runners` for cross-repo source/target context.
- `rust-runtime-receipt-path-discovery` for outcome receipt storage.
- `rust-receipt-proof-verification` for proof-backed final receipts.

## Assumptions

- GitHub is the initial PR provider.
- Deploy verification can start as command/provider hook output with a stable
  contract before richer hosted integrations land.
- Source-thread publishing can use the same outbox/event model as earlier
  milestone comments.

## Touchpoints

- Provider adapter for PR state.
- Outbox/feed event builders.
- Runtime receipt/event summaries.
- Policy config.
- Aster observer scheduling/status.

## Risks

- Duplicate webhook deliveries can create noisy final comments.
- Missing source-thread metadata can cause root-channel Slack posts.
- Verification logs can leak secrets or local paths if not redacted.
- Closing issues before verification can hide unresolved bugs.

## Acceptance

Profile: strict

Validation:
- `pnpm test`
- `cargo test --manifest-path crates/Cargo.toml`
- outcome-observer fixture command
- `git diff --check`

Required behavior:
- [ ] Merged PR with passing verification posts one final source issue comment,
  one final source-thread reply, and closes/labels according to policy.
- [ ] Merged PR with failing verification posts a failure outcome and leaves the
  source issue open unless policy explicitly says otherwise.
- [ ] Closed-unmerged PR posts a distinct outcome and does not claim a fix
  shipped.
- [ ] Repeated observer event is idempotent.
- [ ] Missing source Slack thread fails Slack publish cleanly without posting to
  channel root.
- [ ] Final outcome includes issue link, PR link, merge sha when available,
  verification summary, and next human action.
- [ ] Final outcome validates against `runx.issue_to_pr_outcome.v1` before it
  is published or used to close the source issue.
- [ ] Final outcome excludes absolute local paths, raw env vars, secrets, and
  excessive logs.

## Phase 1: Outcome Model

Status: pending
Dependencies: `runx-operational-policy-config`

Objective: Define the events, states, and idempotency keys.

Changes:
- Add outcome state contract.
- Add `runx.issue_to_pr_outcome.v1` contract and semantic validation.
- Add idempotency key rules.
- Add policy validation for outcome actions.

Acceptance:
- [ ] Fixtures cover every outcome state.

## Phase 2: Observer

Status: pending
Dependencies: Phase 1

Objective: Observe provider PR state and run verification.

Changes:
- Add GitHub PR observer adapter.
- Add verification hook contract.
- Record outcome receipt/event.

Acceptance:
- [ ] Merged, closed, and repeated event fixtures produce correct outcomes.

## Phase 3: Publishing

Status: pending
Dependencies: Phase 2

Objective: Publish final outcome to the original source surfaces.

Changes:
- Publish source issue comment.
- Publish source Slack/source-thread reply only when thread metadata is present.
- Close/label source issue according to policy.

Acceptance:
- [ ] Source-thread fixture posts no root-channel messages.
- [ ] Final comment is concise but contains all review-gate state.

## Rollback

- Keep repo-local outcome scripts until core observer fixtures are green, then
  migrate adopters and remove duplicated observer logic. No compatibility alias
  remains after cutover.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- Target score: 9.5. Passing means humans get a complete issue-to-PR-to-merge
  story without watching multiple channels manually.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: make final outcome publishing a reusable runx capability

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-05-19: Expanded placeholder into post-merge outcome observer contract.
- 2026-05-19: Locked the terminal outcome packet to
  `runx.issue_to_pr_outcome.v1`; Rust, Aster, and repo wrappers must consume
  this shape rather than publishing bespoke terminal payloads.
