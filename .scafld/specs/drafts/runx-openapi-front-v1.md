---
spec_version: '2.0'
task_id: runx-openapi-front-v1
created: '2026-06-04T06:14:58Z'
updated: '2026-06-04T06:17:42Z'
status: draft
harden_status: in_progress
size: medium
risk_level: medium
---

# runx-openapi-front-v1

## Current State

Status: draft
Current phase: none
Next: harden
Reason: hardening round in progress
Blockers: none
Allowed follow-up command: `scafld harden runx-openapi-front-v1 --mark-passed`
Latest runner update: none
Review gate: not_started

## Summary

Add a governed OpenAPI front: ingest an external OpenAPI 3.x spec and expose its
operations as governed tools through the shipped external-adapter lane, so each
call is admitted under an authority, scoped to delivered credentials, sandboxed,
and sealed into a receipt. This is the concrete proof that the core runs from
specs other than MCP (governed-execution-layer.md item 11). It ships as an
`ExternalAdapter` implementation first and earns a first-class `SourceKind::OpenApi`
only if real usage warrants it; calling OUT to an OpenAPI endpoint is "runx as
client" and rides the external-adapter lane, never a new in-kernel HTTP client.

## Objectives

- Turn an arbitrary external OpenAPI 3.x document (file or URL) into a set of
  governed tools: one tool per selected operation, input schema derived from the
  operation params/body, output sealed into a `runx.receipt.v1`.
- Reuse the external-adapter front (`SourceKind::ExternalAdapter`) and the governed
  HTTP front (`runtime_http`) for the network leg; add no in-kernel provider client.
- For money/mutating operations, ride the governed-tool-call convention (admission
  ref in via args, `proof_ref`/receipt out), the same shape payments uses.
- Prove multi-spec end to end: a real public OpenAPI spec becomes governed tools
  with no core fork, with a refusal and a sealed receipt on screen.

## Scope

In scope:

- An external-adapter implementation that ingests an OpenAPI 3.x document,
  enumerates operations, validates request params/body against the spec, and maps
  each operation to a governed tool.
- The governed call over `runtime_http` (public-host default, private-network
  opt-in, secret-redacted), with `CredentialDelivery` for authenticated APIs.
- Graduate `oss/examples/openapi-tool` + `oss/examples/openapi-graph` from a
  checked-in fixture spec to a real-spec adapter, plus a harness case.
- A worked openapi-backed skill that reaches a maturity tier with a harness case.

Out of scope:

- A first-class `SourceKind::OpenApi` variant (revisit only if the external-adapter
  route proves demand; tracked, not built here).
- Hosted OAuth/connect brokerage for arbitrary providers (cloud/private
  dependency, not this spec). OSS demos use public/no-auth APIs or local
  descriptor delivery.
- Non-OpenAPI spec formats (gRPC, GraphQL) and OpenAPI features beyond the
  declared subset (3.x, JSON bodies, the operations the skill selects).
- Mutating operations before the governed-tool-call convention lands.

## Dependencies

- Shipped: external-adapter front (`SourceKind::ExternalAdapter`), governed HTTP
  front (`runtime_http` / `SourceKind::Http`), credential delivery.
- The governed-tool-call convention (admission-ref-in / proof-out) for
  money/mutation operations (governed-execution-layer.md item 5 / Wave 1 spike).
- Local credential descriptors for fixture-key APIs; hosted OAuth/connect
  brokerage remains a cloud/private dependency for live authenticated providers.

## Assumptions

- The external-adapter protocol (`runx.external_adapter.v1`) + manifest are the
  right seam (verified shipped + dogfooded via `examples/external-adapter-graph`).
- The existing `openapi-tool`/`openapi-graph` examples already prove the read path
  against a checked-in spec; this productizes ingestion of an arbitrary spec.
- Public, no-auth or local-descriptor fixture-key OpenAPI APIs are sufficient for
  the first OSS demo; live hosted OAuth providers wait on the cloud/private
  broker.

## Touchpoints

- `oss/examples/openapi-tool`, `oss/examples/openapi-graph` (PoC to graduate).
- The external-adapter protocol + manifest (`runx.external_adapter.v1`).
- `runtime_http` (the governed network leg) and `CredentialDelivery`.
- `runx-contracts` (tool input/output + `runx.receipt.v1`).
- The skill catalog + maturity tiers (a new openapi-backed skill).

## Risks

- **Spec sprawl.** OpenAPI specs are large and inconsistent. Mitigation: support a
  bounded subset (3.x, JSON bodies, declared operations) and fail closed on
  unsupported shapes rather than guessing.
- **Auth gap.** Live OAuth APIs need the cloud/private broker. Mitigation: scope
  the first front + demo to public/no-auth or local-descriptor fixture-key APIs;
  gate hosted OAuth APIs on the cloud unlock.
- **Mutation safety.** A POST/PUT/DELETE operation moves state. Mitigation: gate
  mutating operations behind the governed-tool-call convention + an authority; do
  not expose mutation blindly.

## Acceptance

Profile: strict

Validation:
- A real external OpenAPI 3.x spec is turned into governed tools; a scoped GET
  operation seals a `runx.receipt.v1`, and an out-of-scope/over-authority call is
  refused with a sealed denial receipt.
- The network leg rides `runtime_http` (no new in-kernel provider client; the
  in-kernel-reqwest guard stays green); private-network access stays opt-in.
- `pnpm verify:fast`, `pnpm fixtures:harness:check`, and
  `cargo nextest run --workspace --all-features` are green.

## Phase 1: External-adapter OpenAPI ingestion and governed tool exposure

Status: pending
Dependencies: external-adapter front, runtime_http, credential delivery

Objective: a runx external-adapter ingests an OpenAPI 3.x spec and exposes its
selected operations as governed, sealed tool calls; the example/skill runs end to
end with a refusal and a sealed receipt.

Changes:
- Implement the OpenAPI external-adapter: spec ingestion, operation enumeration,
  param/body validation, governed call via `runtime_http`, sealed output.
- Graduate the `openapi-tool`/`openapi-graph` examples to ingest a real spec and
  add a harness case.
- Wire credential delivery for the no-auth/local-descriptor fixture-key path;
  leave hosted OAuth providers behind the cloud/private broker.

Acceptance:
- [ ] `ac1` command - openapi skill runs a governed call and seals a receipt
  - Command: `runx harness examples/openapi-graph/<harness-case>.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2` command - no new in-kernel provider client; gates green
  - Command: `pnpm verify:fast && cargo nextest run --manifest-path crates/Cargo.toml --workspace --all-features`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: First-class SourceKind::OpenApi (only if earned)

Status: pending
Dependencies: Phase 1 + demonstrated demand

Objective: promote OpenAPI from an external-adapter implementation to a
first-class `SourceKind` only if usage shows it deserves kernel-native dispatch.

Changes:
- Add `SourceKind::OpenApi` + its runtime adapter, reusing the Phase 1 ingestion.

Acceptance:
- [ ] `ac3` command - openapi source type parses + dispatches (if pursued)
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-parser`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Additive front. Remove the OpenAPI external-adapter + the graduated example and
  revert the examples to the fixture spec; no contract churn, no SourceKind change
  if Phase 2 is not pursued.

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

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: in_progress
Started: 2026-06-04T06:17:42Z
Ended: none

Checks:
- Path audit
  - Grounded in: 
  - Result: 
  - Evidence: 
- Command audit
  - Grounded in: 
  - Result: 
  - Evidence: 
- Scope/migration audit
  - Grounded in: 
  - Result: 
  - Evidence: 
- Acceptance timing audit
  - Grounded in: 
  - Result: 
  - Evidence: 
- Rollback/repair audit
  - Grounded in: 
  - Result: 
  - Evidence: 
- Design challenge
  - Grounded in: 
  - Result: 
  - Evidence: 

Issues:
- none


## Planning Log

- none
