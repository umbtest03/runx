---
spec_version: '2.0'
task_id: rust-approval-gate-parity
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T02:35:00Z'
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

Land end-to-end approval-gate parity on the Rust runtime. The current TS
shape (`ApprovalGate { id, reason, type?, summary? }` plus
`ResolutionRequest { id, kind: "approval", gate }`,
`ResolutionResponse { actor: "human" | "agent", payload: boolean }`, and
`Caller.report` / `Caller.resolve`) is the cross-language contract. Rust must
not invent a parallel compatibility shape. `runx-runtime` consumes the
contract; receipts capture the round-trip; a cloud-client crate or feature
speaks only a documented approval routing HTTP contract.

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
- `packages/contracts/src/schemas/agent-act.ts` (ApprovalGate contract)
- `packages/contracts/src/schemas/resolution.ts` (approval resolution request
  and response contracts)
- `packages/runtime-local/src/runner-local/approval.ts`
- `packages/runtime-local/src/runner-local/graph-governance.ts`
- `packages/contracts/src/openapi-runtime.ts` (current OSS OpenAPI fragments for
  hosted approval objects)
- `cloud/packages/db/src/approval-routing.ts` (not present in this OSS checkout;
  see build-readiness blockers)
- `cloud/packages/db/migrations/0006_policy_control.sql` (not present here)
- `cloud/packages/db/migrations/0007_policy_control_hardening.sql` (not present
  here)
- `cloud/packages/agent-runner/src/durable-step.ts` (not present here)
- `crates/runx-contracts/src/host_protocol.rs` (existing Rust
  `ApprovalGate`, `ResolutionRequest`, `ResolutionResponse`)
- `crates/runx-contracts/src/redaction.rs`
- `crates/runx-receipts/src/verify/proof.rs`
- `crates/runx-receipts/src/canonical.rs`

Files impacted:
- `crates/runx-contracts/src/approval.rs` (new only if it re-exports or wraps
  the existing host-protocol types without creating a second wire model)
- `crates/runx-contracts/src/host_protocol.rs`
- `crates/runx-runtime/src/approval.rs`
- `crates/runx-runtime/src/cloud_client.rs` (new, behind a feature)
- `crates/runx-receipts/src/approval_envelope.rs`
- `fixtures/approval/**`
- `packages/contracts/src/openapi-runtime.ts` or
  `cloud/packages/api/src/approval/openapi.ts` (publish the stable HTTP shape;
  the source must exist in the build checkout before implementation starts)
- `scripts/generate-rust-approval-fixtures.ts`

Invariants:
- The TS approval contract does not silently change. Any clarification that
  the Rust port forces (enumerated gate types, payload schema) lands in TS
  first via a small clarification spec, not by Rust drift.
- `ApprovalGate.type` and `ApprovalGate.summary` remain optional in Rust until
  TS makes them required. Rust serialization must omit absent optional fields,
  not emit nulls or empty objects to satisfy convenience tests.
- Approval resolution actors are exactly the TS enum values `human` and
  `agent`. Do not broaden to a free string in Rust or the cloud client.
- Approval resolution payload is boolean for approval requests. The Rust caller
  boundary must reject non-boolean payloads the same way
  `packages/core/src/executor/index.ts` does.
- Receipts capture every gate request, decision, actor, and gate hash.
- Approval receipt fixtures are proof-verifiable by
  `rust-receipt-proof-verification`; structural capture alone is not enough for
  this gate.
- Gate request, actor, boolean decision, route metadata, idempotency key hash,
  and gate hash are included in the canonical receipt body digest commitment so
  post-hoc mutation is detectable. The implementation must rely on
  `runx-receipts` canonical body proof logic rather than introducing a
  receipt-local digest algorithm.
- Approval summaries, route snapshots, receipt projection text, proof findings,
  and HTTP error bodies must pass the existing secret/path redaction bar before
  persistence or display. Raw local absolute paths, bearer tokens, API keys,
  material refs, raw tokens, and raw secrets must not appear in fixtures,
  receipts, logs, OpenAPI examples, or HTTP errors.
- The cloud HTTP contract is documented before the Rust client consumes it.
- Approval routing decisions remain in TS (cloud/db) until a cloud cutover.
  The Rust client is read/write over a stable HTTP surface, not a reimpl.
- No approval bypasses: Rust runner must call the same gate evaluation paths
  as TS runner via shared `runx-core::policy` decisions.
- No legacy/compat readers: do not accept alternate field spellings, null
  optional fields, stringly boolean decisions, or old actor names unless TS
  accepts them in the named source contract first.

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
- Update receipts to carry approval round-trip envelopes and proof-verifiable
  body commitments.
- Add at least one approval round-trip receipt fixture consumed by
  `rust-receipt-proof-verification`.
- Document the cloud HTTP surface explicitly in `cloud/packages/api`.
- Preserve the existing `runx-contracts::host_protocol` wire contract or move it
  through a single re-export path. Do not leave two public Rust approval models
  that can diverge.

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
- `rust-receipt-proof-verification` for approval receipt proof checks.
- `cloud-http-contract-stabilization` (`.ai/specs/drafts/`) for the
  approval routing HTTP contract surface. This spec consumes a specific
  contract version produced there; it does not negotiate the contract
  ad-hoc.
- `rust-ts-interop-boundary` for the cross-language crossing reference.

## Open Questions

- Whether the cloud client is a `runx-runtime` feature or its own crate
  (`runx-cloud-client`). Defer until Phase 1 ingest measures the surface.
- Cloud client packaging remains open, but the HTTP contract source path is not:
  implementation cannot begin until the approval routes are present in this OSS
  checkout or this spec names an external pinned artifact with exact version,
  hash, and generated fixture provenance.

## Build Readiness Hardening

Verdict: blocked until the blockers below are resolved in the draft. Do not
approve this spec for build in its current form.

Blockers:
- **Cloud HTTP boundary is not buildable from this checkout.** The draft names
  `cloud/packages/*`, but this OSS tree has no `cloud/` directory. Current
  approval OpenAPI evidence in this checkout is limited to schema fragments in
  `packages/contracts/src/openapi-runtime.ts` (`HostedApprovalGate`,
  `ApprovalInboxItem`, `ApprovalInboxEnvelope`) and does not define concrete
  approval POST/GET/PUT routes, request bodies, response envelopes, auth
  headers, idempotency headers, retry semantics, or error envelopes. Before
  build, either add the source HTTP contract to the declared impacted files or
  depend on a pinned artifact with version and hash.
- **Contract shape is overspecified and risks Rust drift.** TS currently defines
  `ApprovalGate.type` and `summary` as optional
  (`packages/contracts/src/schemas/agent-act.ts`), and
  `ResolutionResponse.actor` as `human | agent`
  (`packages/contracts/src/schemas/resolution.ts`). The spec must require exact
  parity: no required gate type, no free-string actor, no null optional fields,
  and no secondary Rust approval wire shape.
- **Receipt proof integration is underspecified.** Metadata-only
  `approvalReceiptMetadata` in TS records `gate_id`, `gate_type`, `decision`,
  `reason`, and `summary`, but this spec requires proof-verifiable round-trip
  envelopes. Build must define where the approval request, route snapshot,
  actor, boolean decision, gate hash, idempotency key hash, redaction refs, and
  hash commitments live in the HarnessReceipt and must verify them through
  `runx-receipts` strict proof checks.
- **Idempotency is unresolved at the HTTP boundary.** Gate ids are stable enough
  for local caller lookup, but cloud retries need an explicit idempotency key
  contract. The Rust client must send a deterministic idempotency key derived
  from run id + request id + canonical gate hash, and the cloud route must
  return the same pending/resolved approval for duplicate submissions without
  creating duplicate inbox items or decision records.
- **Secret/path redaction is not yet part of the acceptance surface.** Approval
  summaries and route snapshots are arbitrary JSON and can carry local paths or
  provider material. Acceptance must include negative fixtures proving raw
  `/Users/...`, Windows home paths, bearer tokens, API keys, material refs, raw
  tokens, and raw secrets are redacted before receipt persistence, proof-status
  projection, logs, and HTTP error responses.
- **No-legacy rule needs to be explicit.** The build must reject compatibility
  shortcuts such as accepting `gate_type` in place of `type`, accepting
  `"true"`/`"false"` approval payload strings, accepting unknown actors, or
  silently ignoring extra fields on approval request/response envelopes.

Advisories:
- Prefer extending or re-exporting `crates/runx-contracts/src/host_protocol.rs`
  over adding `approval.rs` as an independent type home. A new module is fine
  only if `host_protocol` and public exports point at the same structs/enums.
- If `runx-cloud-client` becomes a new crate, keep it out of pure kernel crates
  and gate network dependencies with cargo feature checks. The runtime should
  have a local caller path that remains fully testable without network access.
- Fixture names should distinguish local approval, cloud pending, cloud
  resolved-approved, cloud resolved-denied, duplicate submit, duplicate resolve,
  expired approval, redacted summary, and tampered proof cases.

Required validation commands for the final build:

```sh
rg -n "export const approvalGateSchema|approvalResolutionRequestSchema|resolutionResponseActors" packages/contracts/src/schemas
rg -n "pub struct ApprovalGate|enum ResolutionRequest|struct ResolutionResponse|enum ResolutionResponseActor" crates/runx-contracts/src
rg -n "HostedApprovalGate|ApprovalInboxItem|ApprovalInboxEnvelope|approval" packages/contracts/src/openapi-runtime.ts cloud 2>/dev/null || true
test -d cloud || { echo "BLOCKER: cloud approval route source is absent from this checkout"; exit 1; }
pnpm test -- --runInBand packages/contracts/src/index.test.ts packages/core/src/executor/index.test.ts packages/runtime-local/src/runner-local/process-sandbox.test.ts
cargo test -p runx-contracts approval -- --nocapture
cargo test -p runx-contracts host_protocol -- --nocapture
cargo test -p runx-runtime approval -- --nocapture
cargo test -p runx-runtime sandbox -- --nocapture
cargo test -p runx-receipts proof -- --nocapture
cargo test -p runx-receipts approval -- --nocapture
pnpm exec tsx scripts/generate-rust-approval-fixtures.ts --check
pnpm boundary:check
pnpm rust:check
```

Additional proof/redaction validation required by this spec:

```sh
rg -n "approval.*gate_hash|idempotency_key_hash|redaction_refs|hash_commitments" fixtures/approval crates/runx-receipts crates/runx-runtime
! rg -n "/Users/|C:\\\\Users|bearer [A-Za-z0-9._:-]{6,}|sk-(proj-)?[A-Za-z0-9_-]{16,}|access[_-]?token|refresh[_-]?token|api[_-]?key|client[_-]?secret|material[_-]?ref|raw[_-]?(secret|token)" fixtures/approval crates/runx-runtime/src crates/runx-receipts/src
```
