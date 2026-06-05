---
spec_version: '2.0'
task_id: runx-demo-surface-prune-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T03:25:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# runx-demo-surface-prune-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: second readiness lane; keep only A-grade demos in the public window
Blockers: gate-hardening guard shape should be known first
Allowed follow-up command: `scafld approve runx-demo-surface-prune-v1`
Latest runner update: none
Review gate: not_started

## Summary

Prune and rank the example/demo surface so a fresh evaluator sees only polished,
runnable proof paths. Runx now has a strong set of featured demos:
`hello-world`, `http-graph`, `openapi-graph`, `github-mcp-hero`,
`governed-spend`, and the payment dogfood lanes. Other example-like material may
still be useful, but it should not sit in the same first-class window unless it
has a README/run command/gate and proves a current product surface.

This is not about deleting fixtures that protect contracts. It is about separating
featured demos, runnable examples, fixtures, and archived scaffolding with no
ambiguous leftovers.

## Objectives

- Define the public demo list in one place and make docs match it.
- Move fixture-only or protocol-only example material out of the featured demo
  lane, or promote it with a runnable script and gate.
- Ensure every `examples/*` directory has an explicit classification: featured,
  runnable-preview, fixture-support, or archived/deleted.
- Remove stale wrappers and stale docs that imply non-A-grade demos are featured.

## Scope

In scope:
- `examples/**`, `docs/demos.md`, `examples/README.md`, README references.
- `scripts/check-demos.mjs` and any new demo inventory/guard script.
- Thread-outbox and post-merge example directories only for classification; product
  wiring belongs to the thread-outbox spec.

Out of scope:
- Building new demos for payment, A2A, hosted, or gallery UX.
- Deleting contract fixtures that have test coverage value.

## Dependencies

- Gate hardening should define the demo inventory guard.
- Existing demos: `examples/github-mcp-hero/run.sh`, `examples/http-graph/run.sh`,
  `examples/openapi-graph/run.sh`, `examples/governed-spend/*`,
  `scripts/check-demos.mjs`.
- Current gap: `docs/demos.md` marks several demos as harness-gated, while
  `pnpm demos:check` only runs the payment receipt demos. This spec closes that
  mismatch by either expanding the gate or changing the public label.

## Assumptions

- A featured demo must run from a fresh checkout, emit or verify a receipt where
  applicable, and have a clear README path.
- Preview examples may stay if they are explicitly non-featured and runnable.

## Risks

- **Accidental fixture deletion.** Mitigation: classify first, delete only when no
  test or doc consumer exists.
- **Demo set becomes too narrow.** Mitigation: keep a runnable-preview section for
  real but secondary examples.

## Acceptance

Profile: strict

Validation:
- `docs/demos.md` and `examples/README.md` agree on the featured demo list.
- Every `examples/*` directory is classified or deleted.
- `pnpm demos:check`, `pnpm x402:dogfood:local`, and focused featured demo
  scripts pass.
- No stale "compatibility wrapper" or obsolete demo wording remains in featured
  docs unless the wrapper is intentionally retained with a deletion date.

## Phase 1: Inventory and classify examples

Status: pending
Dependencies: runx-readiness-gate-hardening-v1

Objective: create a single source of truth for demo classification.

Changes:
- Add or update a demo inventory guard.
- Classify each `examples/*` directory as featured, runnable-preview,
  fixture-support, or remove.
- Align `docs/demos.md`, `examples/README.md`, and root README links.

Acceptance:
- [ ] `p1_ac1` command - example inventory is complete
  - Command: `node scripts/check-demo-inventory.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Prune or promote non-featured material

Status: pending
Dependencies: Phase 1

Objective: no ambiguous example debris.

Changes:
- Delete truly orphaned example directories.
- Move fixture-support material under a fixture/support path if that is cleaner.
- Add `run.sh` and docs only for examples intentionally promoted to runnable-preview.

Acceptance:
- [ ] `p2_ac1` command - featured demos run
  - Command: `sh examples/github-mcp-hero/run.sh && sh examples/http-graph/run.sh && sh examples/openapi-graph/run.sh && crates/target/debug/runx harness examples/governed-spend/skills/overspend-refused --json && pnpm demos:check && pnpm x402:dogfood:local`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - full fast gate remains green
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Restore any deleted example directory only with its classification and gate.

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
