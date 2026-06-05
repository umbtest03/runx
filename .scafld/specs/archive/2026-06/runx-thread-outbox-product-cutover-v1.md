---
spec_version: '2.0'
task_id: runx-thread-outbox-product-cutover-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T04:15:27Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# runx-thread-outbox-product-cutover-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: reconciled against shipped state; archived provider-front spec completed issue-to-pr cutover and post-merge publisher on the Rust front, and the operational action layer vocabulary is now recorded as completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-05T04:15:27Z
Review gate: not_required

## Summary

The Rust thread-outbox-provider front is already the product mutation boundary.
The completed archived spec
`.scafld/specs/archive/2026-06/runx-thread-outbox-provider-front-v1.md` records
fixture parity, graph dispatch, issue-to-pr provider-push cutover, deletion of
the obsolete TS outbox-push catalog tool, and post-merge final outcome
publication through the same front. This spec is a reconciliation guard to avoid
creating a second mutation surface.

## Objectives

- Shipped: one provider-front path for thread/outbox fetch and push.
- Shipped: issue-to-pr provider push and post-merge final outcome publisher route
  through the Rust front.
- Shipped: fixture graph and final outcome examples seal through the front.
- Preserve: no provider HTTP client returns to pure contracts/core/parser crates.

## Scope

In scope:
- Reconciliation only: verify shipped thread-outbox-provider product cutover.

Out of scope:
- A new GitHub SDK wrapper in core.
- Hosted queue/lifecycle and resident-kernel transport.
- A2A or generic provider marketplace work.
- Rebuilding the already-completed provider-front implementation.

## Dependencies

- Completed archived `runx-thread-outbox-provider-front-v1` implementation.
- Operational action layer vocabulary for propose/admit/execute/publish/seal.
- Existing examples:
  `examples/thread-outbox-provider-graph`,
  `examples/post-merge-final-outcome-publisher`,
  `examples/github-mcp-hero`.

## Assumptions

- The front is already productized; this spec prevents duplicate follow-up work.

## Risks

- **Duplicate mutation surfaces.** Mitigation: use the completed archived spec as
  source of truth and keep direct provider mutation guards in the readiness queue.

## Acceptance

Profile: strict

Validation:
- `examples/thread-outbox-provider-graph` seals through the front.
- `examples/post-merge-publish/final-outcome.yaml` seals through the same front.
- Current docs identify thread-outbox-provider as the provider mutation boundary.

## Phase 1: Consumer inventory and guard

Status: completed
Dependencies: runx-operational-action-layer-v1

Objective: identify and guard every GitHub/thread mutation consumer.

Changes:
- Verified current source and archived completion state.

Acceptance:
- [x] `p1_ac1` command - provider mutation inventory is clean
  - Command: `rg -n "thread-outbox-provider|post-merge-final|outbox-provider" crates packages scripts examples docs -g '!dist' -g '!target'`
  - Expected kind: `exit_code_zero`
  - Status: pass

## Phase 2: Product cutover

Status: completed
Dependencies: Phase 1

Objective: migrate GitHub product lanes to the front.

Changes:
- Verified archived phase-3 cutover evidence and current examples.

Acceptance:
- [x] `p2_ac1` command - thread-outbox graph seals
  - Command: `runx harness examples/thread-outbox-provider-graph --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
- [x] `p2_ac2` command - GitHub dogfood preflight uses front
  - Command: `pnpm dogfood:github-issue-to-pr -- --preflight`
  - Expected kind: `exit_code_zero`
  - Status: pass

## Phase 3: Cleanup and docs

Status: completed
Dependencies: Phase 2

Objective: leave one obvious mutation boundary.

Acceptance:
- [x] `p3_ac1` command - full fast gate remains green
  - Command: `pnpm verify:fast && pnpm fixtures:harness:check`
  - Expected kind: `exit_code_zero`
  - Status: pass

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
State cleanup: 2026-06-05T04:15:27Z verified no duplicate active/draft/archive record; reconciliation-only completion is the canonical state.

## Harden Rounds

- none

## Planning Log

- none
