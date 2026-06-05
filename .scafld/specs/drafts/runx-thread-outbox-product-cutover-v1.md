---
spec_version: '2.0'
task_id: runx-thread-outbox-product-cutover-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T03:25:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# runx-thread-outbox-product-cutover-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: productize the shipped thread-outbox-provider front for GitHub mutation lanes
Blockers: operational action layer should define the generic action vocabulary
Allowed follow-up command: `scafld approve runx-thread-outbox-product-cutover-v1`
Latest runner update: none
Review gate: not_started

## Summary

The Rust thread-outbox-provider front exists and has a fixture graph, but the next
product step is to make it the clean mutation boundary for GitHub operational
lanes: review comments, PR notes, post-merge final outcome publication, and future
thread/outbox provider mutations. This should replace bespoke mutation paths, not
add another parallel surface.

## Objectives

- One provider-front path for thread/outbox fetch and push.
- GitHub mutation lanes route through the Rust front and seal receipts.
- Fixture examples stay as fixtures; product examples show the actual mutation
  boundary and refusal behavior.
- No provider HTTP client returns to pure contracts/core/parser crates.

## Scope

In scope:
- Existing thread-outbox provider examples and fixtures.
- Issue-to-pr, PR-review-note, and post-merge outcome publication wiring.
- Runtime dispatch and receipt sealing for the provider front.
- Docs that describe provider tokens as edge credentials, not skill manifest data.

Out of scope:
- A new GitHub SDK wrapper in core.
- Hosted queue/lifecycle and resident-kernel transport.
- A2A or generic provider marketplace work.

## Dependencies

- Archived `runx-thread-outbox-provider-front-v1` implementation.
- Operational action layer vocabulary for propose/admit/execute/publish/seal.
- Existing examples:
  `examples/thread-outbox-provider-graph`,
  `examples/post-merge-final-outcome-publisher`,
  `examples/github-mcp-hero`.

## Assumptions

- The front is already good enough for fixture dispatch; product cutover is about
  consumer migration, docs, and gates.

## Risks

- **Breaking GitHub dogfood.** Mitigation: keep issue-to-pr and PR-review-note
  harnesses green before deleting old paths.
- **Duplicate mutation surfaces.** Mitigation: add a guard that rejects direct
  provider mutation paths outside the thread-outbox-provider front.

## Acceptance

Profile: strict

Validation:
- GitHub review/comment/final-outcome publication routes through the front.
- Out-of-scope mutation is refused and sealed.
- No direct provider mutation path remains in OSS runtime or CLI code outside the
  sanctioned front.

## Phase 1: Consumer inventory and guard

Status: pending
Dependencies: runx-operational-action-layer-v1

Objective: identify and guard every GitHub/thread mutation consumer.

Changes:
- Inventory current direct mutation paths.
- Add a boundary check for direct provider mutation calls outside the front.

Acceptance:
- [ ] `p1_ac1` command - provider mutation inventory is clean
  - Command: `node scripts/check-thread-outbox-boundary.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Product cutover

Status: pending
Dependencies: Phase 1

Objective: migrate GitHub product lanes to the front.

Changes:
- Route issue-to-pr/pr-review/post-merge publisher paths through
  thread-outbox-provider.
- Delete or archive obsolete direct paths.
- Update examples/docs so fixture-only examples are not confused with product demos.

Acceptance:
- [ ] `p2_ac1` command - thread-outbox graph seals
  - Command: `runx harness examples/thread-outbox-provider-graph --json`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - GitHub dogfood preflight uses front
  - Command: `pnpm dogfood:github-issue-to-pr -- --preflight`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Cleanup and docs

Status: pending
Dependencies: Phase 2

Objective: leave one obvious mutation boundary.

Acceptance:
- [ ] `p3_ac1` command - full fast gate remains green
  - Command: `pnpm verify:fast && pnpm fixtures:harness:check`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Restore the previous consumer path and remove the boundary guard in the same
  revert. Do not leave two active mutation surfaces.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: codex

## Origin

Created by: Codex
Source: operator readiness queue

## Harden Rounds

- none

## Planning Log

- none
