---
spec_version: '2.0'
task_id: rust-approval-gate-parity
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Rust approval gate parity

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Approval gates are the
governance surface customers notice; first-class spec rather than a runtime
sub-step.
Blockers: `rust-runtime-skeleton` complete, `rust-receipts-parity` complete.
Allowed follow-up command: `scafld harden rust-approval-gate-parity`
Latest runner update: none
Review gate: not_started

## Summary

Land end-to-end approval-gate parity on the Rust runtime. The TS shape
(`ApprovalGate { id, type, reason, summary }` plus `Caller.report` /
`Caller.resolve`) becomes the cross-language contract in
`runx-contracts::approval`; `runx-runtime` consumes it; receipts capture
the round-trip; a cloud-client crate or feature speaks the existing approval
routing HTTP contract.

This spec is the canary that proves Rust runtime can govern, not just
execute. It blocks any CLI cutover that touches mutation classes.

## Context

CWD: `.`

Packages:
- `@runxhq/core` (executor)
- `@runxhq/runtime-local` (runner-local approval, graph-governance)
- `@runxhq/contracts`
- `cloud/packages/api` (approval routes)
- `cloud/packages/db` (approval-routing.ts + policy_control migrations)
- `cloud/packages/agent-runner` (durable-step.ts pause/resume)
- `crates/runx-runtime`
- `crates/runx-contracts`
- (possible) `crates/runx-cloud-client`

Current TypeScript sources:
- `packages/core/src/executor/index.ts` (ApprovalGate type)
- `packages/runtime-local/src/runner-local/approval.ts`
- `packages/runtime-local/src/runner-local/graph-governance.ts`
- `cloud/packages/db/src/approval-routing.ts`
- `cloud/packages/db/migrations/0006_policy_control.sql`
- `cloud/packages/db/migrations/0007_policy_control_hardening.sql`
- `cloud/packages/agent-runner/src/durable-step.ts`

Files impacted:
- `crates/runx-contracts/src/approval.rs` (new)
- `crates/runx-runtime/src/approval.rs`
- `crates/runx-runtime/src/cloud_client.rs` (new, behind a feature)
- `crates/runx-receipts/src/approval_envelope.rs`
- `fixtures/approval/**`
- `cloud/packages/api/src/approval/openapi.ts` (publish stable HTTP shape)
- `scripts/generate-rust-approval-fixtures.ts`

Invariants:
- The TS approval contract does not silently change. Any clarification that
  the Rust port forces (enumerated gate types, payload schema) lands in TS
  first via a small clarification spec, not by Rust drift.
- Receipts capture every gate request, decision, actor, and gate hash.
- The cloud HTTP contract is documented before the Rust client consumes it.
- Approval routing decisions remain in TS (cloud/db) until a cloud cutover.
  The Rust client is read/write over a stable HTTP surface, not a reimpl.
- No approval bypasses: Rust runner must call the same gate evaluation paths
  as TS runner via shared `runx-core::policy` decisions.

## Objectives

- Define `runx-contracts::approval` types (gate, request envelope, resolution
  envelope, gate kind enum, actor identity).
- Implement `runx-runtime` runner-side: gate emission, caller reporting,
  resolution awaiting, decision-to-receipt wiring.
- Implement a cloud client (feature on `runx-runtime` or a new pure crate
  per open question 12.4 of `plans/rust-takeover.md`) covering the approval
  POST/GET/PUT routes.
- Add cross-language fixtures: sandbox-escalation gate, graph-step scope
  gate, destructive-action gate, denied gate, expired gate.
- Update receipts to carry approval round-trip envelopes.
- Document the cloud HTTP surface explicitly in `cloud/packages/api`.

## Scope

In scope:
- Contracts, runtime side, receipts envelope, cloud client.
- One local-only gate fixture and one cloud-routed gate fixture end to end.

Out of scope:
- Aster operator UI consumption of gates (separate spec under aster v1 reset).
- Approval routing logic changes (the cloud rules stay in TS).
- Replacing the TS runner-local approval path. Both runners co-exist until a
  TS sunset spec.

## Dependencies

- `rust-runtime-skeleton`, `rust-receipts-parity`, `rust-contracts-parity`.
- `cloud-http-contract-stabilization` (`.ai/specs/drafts/`) for the
  approval routing HTTP contract surface. This spec consumes a specific
  contract version produced there; it does not negotiate the contract
  ad-hoc.
- `rust-ts-interop-boundary` for the cross-language crossing reference.

## Open Questions

- Whether the cloud client is a `runx-runtime` feature or its own crate
  (`runx-cloud-client`). Defer until Phase 1 ingest measures the surface.
- Whether `actor` in the resolution envelope is a free string or an enum.
  TS today accepts a string; the Rust port may tighten with a TS adapter
  layer.
- Idempotency for approval requests: today the runner can retry; the spec
  picks an explicit semantics rather than implicit reliance on dedup.
