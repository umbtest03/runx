---
spec_version: '2.0'
task_id: runx-operational-action-layer-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T04:15:27Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# runx-operational-action-layer-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: reconciled against shipped state; `runx.operational_proposal.v1`, docs, fixtures, and Nitrosend cross-repo proof already landed in the completed operational-intelligence spec
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-05T03:25:35Z
Review gate: not_required

## Summary

Define and verify the operational action layer: the reusable path where
operational intelligence proposes an action, policy admits or refuses it, a
provider front executes it, an outcome is published, and a receipt seals the
decision.

This spec is complete by reconciliation. The shipped tree already has the
generic public proposal packet, docs, fixtures, schema validation, and Nitrosend
cross-repo proof. The completed parent record is
`.scafld/specs/archive/2026-06/runx-operational-intelligence-action-layer-v1.md`.
This file exists only to keep the readiness queue ordered and to prevent a
duplicate action-layer implementation.

## Objectives

- Shipped: one generic action lifecycle documented in
  `docs/operational-intelligence.md`.
- Shipped: one contract surface, `runx.operational_proposal.v1`, with TS and Rust
  schema validation.
- Shipped: provider-specific details stay at the edge; invalid provider-specific
  and product-specific top-level fields have negative fixtures.
- Shipped: Nitrosend dogfood proof is tracked in the completed cross-repo spec.

## Scope

In scope:
- Reconciliation only: verify and record the shipped operational action layer.

Out of scope:
- New provider families or product-specific schemas for every app.
- Auto-merge or silent mutation policy changes.
- Hosted dispatch/lifecycle. Hosted execution belongs to hosted ops.
- Rebuilding the already-completed Nitrosend integration.

## Dependencies

- `.scafld/specs/archive/2026-06/runx-operational-intelligence-action-layer-v1.md`
- `docs/operational-intelligence.md`
- `packages/contracts/src/schemas/operational-proposal.ts`
- `crates/runx-contracts/src/operational_proposal.rs`
- `fixtures/contracts/operational-proposal/**`

## Assumptions

- The existing completed cross-repo spec is the source of truth for Nitrosend
  dogfood details.

## Risks

- **Duplicate action-layer build.** Mitigation: this spec is reconciliation-only
  and points at the completed parent/child specs and shipped artifacts.

## Acceptance

Profile: strict

Validation:
- `docs/operational-intelligence.md` names the action lifecycle.
- `runx.operational_proposal.v1` is exported and fixture-validated in TS and Rust.
- Completed cross-repo Nitrosend proof remains the dogfood source of truth.
- No new implementation path is introduced by this spec.

## Phase 1: Model and docs

Status: completed
Dependencies: none

Objective: lock the concept before wiring consumers.

Changes:
- Verified the shipped lifecycle doc and contract boundary.

Acceptance:
- [x] `p1_ac1` command - operational action docs exist and name the lifecycle
  - Command: `test -f docs/operational-intelligence.md && rg -n "source/context/signal/decision/proposal/action/outcome|proposal|provider mutation|receipt" docs/operational-intelligence.md`
  - Expected kind: `exit_code_zero`
  - Status: pass

## Phase 2: Runtime/contract wiring

Status: completed
Dependencies: Phase 1

Objective: an admitted action can execute through a provider front and seal an
outcome.

Changes:
- Verified the shipped operational proposal schema, fixtures, and Rust exports.

Acceptance:
- [x] `p2_ac1` command - contracts and fixtures validate
  - Command: `pnpm contracts:schemas:check && pnpm fixtures:contracts:check && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features`
  - Expected kind: `exit_code_zero`
  - Status: pass

## Phase 3: GitHub and Nitrosend dogfood

Status: completed
Dependencies: Phase 2

Objective: prove two real operational domains use the same action layer.

Changes:
- Recorded the existing completed cross-repo Nitrosend dogfood spec as the proof
  source; no new Runx implementation was needed.

Acceptance:
- [x] `p3_ac1` command - GitHub dogfood shape remains green
  - Command: `pnpm dogfood:github-issue-to-pr -- --preflight`
  - Expected kind: `exit_code_zero`
  - Status: pass
- [x] `p3_ac2` manual - Nitrosend dry-run/test-send dogfood produces sealed action outcome
  - Expected kind: `manual`
  - Status: pass

## Rollback

- Revert contracts, fixtures, docs, and consumer wiring together. Do not leave a
  half-promoted schema or provider-specific fallback path.

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
