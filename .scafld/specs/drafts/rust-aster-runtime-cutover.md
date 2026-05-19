---
spec_version: '2.0'
task_id: rust-aster-runtime-cutover
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T02:35:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Rust aster runtime cutover

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Companion to
`plans/aster-v1-reset.md`; aster v1 builds against the Rust runtime from
its first commit.
Blockers: `rust-runtime-skeleton` complete, `rust-approval-gate-parity`
complete.
Allowed follow-up command: `scafld harden rust-aster-runtime-cutover`
Latest runner update: none
Review gate: not_started

## Summary

Preserve aster's existing hosted agent-step path through runx when the
underlying execution flips to the Rust runtime. Aster is not a blank
slate; substantial production code already exists across the cloud
tree, and `plans/aster-v1-reset.md` confirms a working hosted
agent-step path through runx is one of the "useful fragments" the reset
keeps.

The cutover dogfood is **preservation, not adoption**, parallel to the
nitrosend case. The aster v1 reset (governance hardening) is a separate
concern tracked in `plans/aster-v1-reset.md`; this spec only owns
runtime-execution preservation.

The actual aster shape today (verified, not assumed):

- UI: `cloud/packages/ui/src/Aster/` (`Aster.tsx`, `AsterBubble.tsx`,
  `useAsterGame.ts`, `useAsterTour.ts`, `util.ts`), plus
  `cloud/packages/ui/src/AsterTour/`, `cloud/packages/ui/src/AsterFeed/`
  (with `model.ts` and `model.test.ts`), `cloud/packages/ui/src/AsterSection/`,
  and adjacent `cloud/packages/ui/src/LiveFeed/`.
- Theme: `cloud/packages/tokens/src/themes/aster.css`.
- Owner profile data: `cloud/.data/runx-owner-profiles/aster.json`.
- Execution path: aster runs as a hosted agent through
  `cloud/packages/agent-runner` (`hosted-agent-adapter.ts`,
  `durable-step.ts`, `anthropic.ts`, `openai-compat.ts`, plus security
  and durable-step test suites).
- Selector and lane orchestration live in cloud TS; receipts route via
  `cloud/packages/receipts-store` and `cloud/packages/db/src/approval-routing.ts`.

What the v1 reset is *adding* (per `plans/aster-v1-reset.md`,
explicitly not present today): explicit scope grants, destructive-action
approval enforcement, durable Target / Opportunity / Priority /
ReflectionEntry objects, schema-backed control objects, runtime-enforced
verification before publication, honest prerelease scoping.

Implication for this cutover: aster's existing agent-step execution must
keep working on the Rust runtime, and the new governance contracts the
v1 reset adds (approval gates, scope grants, durable objects) must
consume `runx-contracts::approval` and `runx-core::policy` from day one.
Aster v1 reset and this cutover share the approval contract surface;
neither owns the other.

## Context

CWD: `.`

Packages:
- `cloud/packages/agent-runner`
- `cloud/packages/api`
- `cloud/packages/db`
- `cloud/packages/receipts-store`
- `cloud/packages/worker`
- `cloud/packages/ui` (UI surfaces for aster, not modified by this spec)
- `crates/runx-runtime`
- `crates/runx-contracts`

Current TypeScript sources:
- `cloud/packages/agent-runner/src/hosted-agent-adapter.ts`
- `cloud/packages/agent-runner/src/durable-step.ts`
- `cloud/packages/agent-runner/src/agent-runner.test.ts`
- `cloud/packages/agent-runner/src/agent-runner-security.test.ts`
- `cloud/packages/agent-runner/src/agent-runner-durable-step.test.ts`
- `cloud/packages/agent-runner/src/anthropic.ts`
- `cloud/packages/agent-runner/src/openai-compat.ts`
- `cloud/packages/db/src/approval-routing.ts`
- `cloud/packages/db/migrations/0006_policy_control.sql`
- `cloud/packages/db/migrations/0007_policy_control_hardening.sql`
- `cloud/packages/ui/src/Aster/*` (read-only for this spec)
- `cloud/packages/ui/src/AsterFeed/model.ts` (read-only for this spec)

Files impacted:
- `crates/runx-runtime/src/cloud_client.rs` (consumed by aster via the
  chosen binding)
- `cloud/packages/agent-runner/src/runx-runtime-binding.ts` (new binding
  shim if subprocess-JSON is the chosen mode)
- `cloud/packages/api/src/approval/**` (versioned contract per
  `cloud-http-contract-stabilization`; consumed by aster's new gate code
  and by the Rust runtime)
- `fixtures/external/aster/agent-step/**` (new; deterministic snapshot
  of aster's hosted agent-step flow)
- `crates/runx-runtime/tests/external/aster_agent_step.rs` (new; runs
  the fixture against the Rust runtime)
- `scripts/generate-rust-aster-fixtures.ts` (new; TS oracle generator;
  retires when cloud agent-runner sunsets, which is not in this
  program)
- `docs/external-dogfoods.md` (extended with the aster section
  alongside nitrosend)

Invariants:
- Aster's existing production behavior is preserved. Same inputs produce
  byte-identical outputs (modulo timestamps and IDs) on TS and Rust.
- Aster never invents a parallel approval shape. All gates flow through
  `runx-contracts::approval`.
- Aster never reads receipts via private file paths. Receipts come through
  a documented surface (`runx-runtime` API, cloud receipts-store HTTP, or
  CLI JSON).
- Scope grants and durable target / opportunity / priority objects, once
  the v1 reset lands them, flow through `runx-runtime` state and
  `runx-core::policy`, not an aster-local reimplementation.
- Aster v1 reset's governance work and this cutover are independent
  tracks. Neither blocks the other except at the approval contract
  surface, which they share.
- TS agent-runner remains for non-aster hosted agent runners until a
  separate cloud cutover spec.

## Objectives

- Pick the aster to `runx-runtime` binding (in-process Rust dependency,
  subprocess JSON over `runx-cli`, or a `runx-runtime-service` daemon)
  with rationale in Phase 1 ingest.
- Capture a deterministic fixture suite from aster's current hosted
  agent-step flow.
- Run the fixture suite against `runx-runtime` and assert byte-identical
  outputs.
- Wire aster's gate code (whether already-present or added by the v1
  reset) through `rust-approval-gate-parity` contracts.
- Build the agent-runner-side shim that bridges aster's TS context to
  the Rust runtime, honoring the chosen binding.
- Surface runx operational policy readback in Aster so runner availability,
  allowed targets, source-thread routing, and outcome behavior are visible
  before execution.
- Use the same `runx.operational_policy.v1` semantic validator/readback that
  backs `runx policy inspect|lint`; Aster must not infer policy by scraping
  adopter config.
- Wire Aster-run issue-to-PR flows through the post-merge outcome observer so
  the final human merge/deploy result is a `runx.issue_to_pr_outcome.v1`
  packet, not a separate repo-local script.
- Soak the Rust binding side-by-side with the TS path on aster
  production before the launcher cutover.

## Scope

In scope:
- Binding decision and implementation.
- Fixture suite and parity test for aster's agent-step path.
- Approval contract integration for any aster gate behavior.
- Side-by-side soak before the launcher cutover.

Out of scope:
- Aster v1 reset governance work (owned by `plans/aster-v1-reset.md`).
- Aster's public UI, brand, feed curation, or selector logic changes.
- Replacing other hosted agent runners (those are their own cutover
  specs).
- Cloud-side approval routing logic changes (the cloud rules stay in
  TS).
- Moving aster's TS UI to Rust (not in this program at all).

## Dependencies

- `rust-runtime-skeleton`.
- `rust-approval-gate-parity`.
- `cloud-http-contract-stabilization` for any aster ↔ cloud HTTP surfaces
  the binding consumes.
- `rust-ts-interop-boundary` for the cross-language crossing reference.
- `runx-operational-policy-config` for policy/admin readback.
- `runx-target-repo-runners` for Aster-scheduled source-to-target PR flows.
- `runx-post-merge-outcome-observer` for final outcome observation and
  source-thread updates.
- `plans/aster-v1-reset.md` design pass.

## Open Questions

- Binding mode (open question 12.3 in `plans/rust-takeover.md`).
- Where the aster v1 governance code lands (which existing cloud package
  takes the new code, or whether a new package is created). Owned by
  `plans/aster-v1-reset.md`, not this spec.
