---
spec_version: '2.0'
task_id: runx-operational-action-layer-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T03:25:35Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# runx-operational-action-layer-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: product-spine lane after gates; converge operational intelligence on one action model
Blockers: none
Allowed follow-up command: `scafld approve runx-operational-action-layer-v1`
Latest runner update: none
Review gate: not_started

## Summary

Define and wire the operational action layer: the reusable path where operational
intelligence proposes an action, a policy admits or refuses it, a provider front
executes it, an outcome is published, and a receipt seals the decision. This is
the product spine for GitHub, Nitrosend, and future operator workflows. It should
not become provider-specific packet sprawl.

The key design choice: one generic operational action/proposal boundary, provider
details only at the front/publisher edge, and no widening of closed
`runx.operational_policy.v1` permissions.

## Objectives

- One generic action lifecycle: observe, propose, admit/refuse, execute, publish
  outcome, seal receipt.
- One contract surface for operational proposals/actions that composes with
  existing `Act`, `Decision`, `OperationalPolicy`, references, and receipts.
- Provider-specific execution stays in provider fronts: GitHub/thread-outbox,
  Nitrosend action layer, HTTP/OpenAPI, and future hosted adapters.
- Nitrosend dogfood proves the model with real operational intelligence flows
  without bypassing policy or receipts.

## Scope

In scope:
- Contracts/docs for the generic operational action lifecycle.
- Runtime/CLI path to carry admitted action packets through an existing provider
  front.
- Nitrosend dogfood shape: campaign/flow/audience/support operations become
  governed proposed actions with sealed outcomes where the connector is available.
- GitHub path alignment with issue-to-pr/pr-review/post-merge outcome publishers.

Out of scope:
- New provider families or product-specific schemas for every app.
- Auto-merge or silent mutation policy changes.
- Hosted dispatch/lifecycle. Hosted execution belongs to hosted ops.

## Dependencies

- Existing operational policy contracts and dogfood scripts:
  `scripts/dogfood-github-issue-to-pr.mjs`,
  `fixtures/operational-policy/**`,
  `docs/operational-intelligence.md` if present.
- Thread-outbox provider front for GitHub mutation publication.
- Nitrosend connector/tool layer for dogfood, but no secrets or live sends in
  default gates.

## Assumptions

- The action layer is generic enough to avoid provider-specific packet families.
- Connector-backed dogfood can validate the workflow shape without sending real
  marketing traffic by default.

## Risks

- **Schema overlap with Act/Decision.** Mitigation: document the boundary and reuse
  existing packets where they already fit.
- **Provider leakage into core contracts.** Mitigation: provider details live under
  references, metadata, or front-specific payloads at the edge.
- **Unsafe live actions.** Mitigation: default dogfood is dry-run/test-send only;
  live mutation requires explicit operator confirmation.

## Acceptance

Profile: strict

Validation:
- One generic operational action/proposal model is documented and wired.
- GitHub and Nitrosend use the same lifecycle vocabulary.
- Closed operational policy permissions remain closed; no `.v2`, aliases, or
  compatibility surfaces are added.
- Dry-run dogfood gates pass without live sends or provider mutations.

## Phase 1: Model and docs

Status: pending
Dependencies: none

Objective: lock the concept before wiring consumers.

Changes:
- Write the operational action lifecycle doc.
- Define the contract boundary against existing `Act`, `Decision`, and
  `OperationalPolicy`.
- State explicitly where provider-specific fields are allowed.

Acceptance:
- [ ] `p1_ac1` command - operational action docs exist and name the lifecycle
  - Command: `test -f docs/operational-action-layer.md && rg -n "observe|propose|admit|execute|publish|seal" docs/operational-action-layer.md`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Runtime/contract wiring

Status: pending
Dependencies: Phase 1

Objective: an admitted action can execute through a provider front and seal an
outcome.

Changes:
- Add or reuse the minimal contract types.
- Wire CLI/runtime handling without changing closed policy permissions.
- Add fixtures for admitted action, refused action, and sealed outcome.

Acceptance:
- [ ] `p2_ac1` command - contracts and fixtures validate
  - Command: `pnpm contracts:schemas:check && pnpm fixtures:contracts:check && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: GitHub and Nitrosend dogfood

Status: pending
Dependencies: Phase 2

Objective: prove two real operational domains use the same action layer.

Changes:
- Align GitHub issue-to-pr/pr-review/post-merge publication with the action
  lifecycle.
- Add Nitrosend dry-run/test-send operational dogfood path using the same model.

Acceptance:
- [ ] `p3_ac1` command - GitHub dogfood shape remains green
  - Command: `pnpm dogfood:github-issue-to-pr -- --preflight`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p3_ac2` manual - Nitrosend dry-run/test-send dogfood produces sealed action outcome
  - Expected kind: `manual`
  - Status: pending

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

## Harden Rounds

- none

## Planning Log

- none
