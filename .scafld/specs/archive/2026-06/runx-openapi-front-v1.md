---
spec_version: '2.0'
task_id: runx-openapi-front-v1
created: '2026-06-04T06:14:58Z'
updated: '2026-06-04T22:15:25Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# runx-openapi-front-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-04T22:15:25Z
Review gate: pass

## Summary

Add a governed OpenAPI front through the shipped external-adapter lane: a checked-in
OpenAPI 3.x document is resolved into a governed graph-step tool call, the adapter
validates inputs against the operation parameters, performs the fixture HTTP GET,
and the runtime seals the result into a receipt tree. This is the concrete OSS
proof that the core runs from specs other than MCP without adding an in-kernel
provider client. Arbitrary hosted providers, OAuth, mutation, and a first-class
`SourceKind::OpenApi` stay out of scope.

## Objectives

- Resolve a checked-in OpenAPI 3.x document into a governed graph-step
  external-adapter call: selected operation, parameter validation, concrete HTTP
  request, structured output, and sealed `runx.receipt.v1`.
- Keep the kernel boundary clean: no `SourceKind::OpenApi`, no in-kernel provider
  client, and no new network surface beyond the supervised external-adapter lane.
- Prove the local end-to-end path: fixture server starts, OpenAPI `getPet` executes
  over HTTP, the harness seals, and the demo script rejects regressions.

## Scope

In scope:

- The checked-in `examples/openapi-tool/openapi.json` fixture spec, the
  `openapi-adapter.mjs` process adapter, and `examples/openapi-graph`.
- Operation lookup by `operationId`, required path/query parameter validation,
  adapter-owned fixture HTTP GET, and structured sealed result.
- A runnable demo script plus inline harness case.

Out of scope:

- A first-class `SourceKind::OpenApi` variant (revisit only if the external-adapter
  route proves demand; tracked, not built here).
- Arbitrary file/URL spec ingestion beyond the checked-in fixture document.
- Hosted OAuth/connect brokerage for arbitrary providers (cloud/private
  dependency, not this spec). OSS demos use public/no-auth APIs or local
  descriptor delivery.
- Non-OpenAPI spec formats (gRPC, GraphQL) and OpenAPI features beyond the
  declared subset (3.x, JSON bodies, the operations the skill selects).
- Mutating operations before the governed-tool-call convention lands.
- A first-class `SourceKind::OpenApi`; this front stays on `external-adapter`.

## Dependencies

- Shipped: external-adapter front (`SourceKind::ExternalAdapter`), governed HTTP
  front (`runtime_http` / `SourceKind::Http`), credential delivery.
- Local fixture HTTP server in `examples/openapi-graph/server.mjs`.

## Assumptions

- The external-adapter protocol (`runx.external_adapter.v1`) + manifest are the
  right seam (verified shipped + dogfooded via `examples/external-adapter-graph`).
- The existing `openapi-tool`/`openapi-graph` examples already prove the read path
  against a checked-in spec; this spec closes that local proof, not arbitrary
  hosted-provider product breadth.

## Touchpoints

- `oss/examples/openapi-tool`, `oss/examples/openapi-graph` (PoC to graduate).
- The external-adapter protocol + manifest (`runx.external_adapter.v1`).
- The external-adapter runtime process boundary and local fixture HTTP server.
- `runx-contracts` (tool input/output + `runx.receipt.v1`).
- The skill catalog + maturity tiers (a new openapi-backed skill).

## Risks

- **Spec sprawl.** OpenAPI specs are large and inconsistent. Mitigation: support a
  bounded subset (3.x, JSON bodies, declared operations) and fail closed on
  unsupported shapes rather than guessing.
- **Auth gap.** Live OAuth APIs need the cloud/private broker. Mitigation: keep
  this spec no-auth/local-fixture only; gate hosted OAuth APIs on a future spec.
- **Mutation safety.** A POST/PUT/DELETE operation moves state. Mitigation: gate
  mutating operations behind the governed-tool-call convention + an authority; do
  not expose mutation blindly.

## Acceptance

Profile: strict

Validation:
- The checked-in OpenAPI 3.x fixture spec is resolved into a governed
  external-adapter graph step; `getPet` executes against the local fixture server
  and seals a `runx.receipt.v1` receipt tree.
- No first-class `SourceKind::OpenApi` or in-kernel provider client is added; the
  final cutover guard stays green.
- `pnpm verify:fast`, `pnpm fixtures:harness:check`, and a focused all-features
  runtime nextest gate are green.

## Phase 1: External-adapter OpenAPI ingestion and governed tool exposure

Status: completed
Dependencies: external-adapter front, runtime_http, credential delivery

Objective: a runx external-adapter resolves the checked-in OpenAPI 3.x fixture

Changes:
- Keep the OpenAPI external-adapter on the process supervisor seam: operation lookup, parameter validation, fixture HTTP GET, sealed output.
- Keep `openapi-tool`/`openapi-graph` harness-gated and runnable via `examples/openapi-graph/run.sh`.

Acceptance:
- [x] `ac1` command - openapi skill runs a governed call and seals a receipt
  - Command: `sh examples/openapi-graph/run.sh`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16
- [x] `ac2` command - no new in-kernel provider client; gates green
  - Command: `pnpm verify:fast && cargo nextest run --manifest-path crates/Cargo.toml -p runx-runtime --all-features`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17

## Rollback

- Additive front. Remove the OpenAPI external-adapter + the graduated example and
  fixture graph; no contract churn, no SourceKind change.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-8
Output: claude.mcp_submit_review
Summary: Reviewed the task-scoped changes to examples/openapi-graph/SKILL.md and run.sh, plus the supporting fixture (X.yaml, server.mjs, openapi.json, manifest.json) and the openapi-adapter.mjs / adapter-kit. The demo graduates the OpenAPI external-adapter PoC: a single-step graph routes an `external-adapter` source through the source-adapter registry, the adapter resolves `getPet` from a checked-in OpenAPI 3.1 spec, validates required parameters, performs an adapter-owned fixture HTTP GET, and the runtime seals a signed runx.receipt.v1. No `SourceKind::OpenApi` is introduced and no new in-kernel network surface is added, matching the declared scope and non-goals. run.sh closely parallels the sibling http-graph demo (same demo-only signing-key env pattern, mktemp receipt dir, background server + EXIT trap) and additionally adds a `kill -0` server-liveness check. Both acceptance criteria are recorded as passing. The runtime changes in crates/runx-runtime (skill_run.rs and its test) are classified ambient drift outside the declared task scope; I used them only as context and did not attribute them to this task. One low, non-blocking robustness observation: the run.sh verification asserts the GET executed by reading intermediate `.graph-state.json` checkpoint files rather than the sealed receipt content. No completion blockers found.

Attack log:
- `acceptance criteria ac1/ac2 vs spec objectives`: Spec Compliance: re-trace that the OpenAPI operation resolves, validates params, performs fixture GET, and seals runx.receipt.v1 without introducing SourceKind::OpenApi or in-kernel client -> clean (X.yaml single-step graph -> ../openapi-tool external-adapter -> openapi-adapter.mjs resolves getPet, validates required petId, fetches, seals. No SourceKind::OpenApi; routing via existing external-adapter registry handler.)
- `task scope vs workspace changes`: Scope Drift: confirm changes stay within declared scope (examples/openapi-graph SKILL.md, run.sh) -> clean (Task-scoped edits confined to SKILL.md and run.sh; both within explicit review scope.)
- `crates/runx-runtime/src/execution/skill_run.rs and tests/skill_run.rs`: Ambient Drift attribution: determine whether runtime registry routing is task work or separate -> clean (Classified ambient_drift; external-adapter graph routing supports the demo but is outside declared scope. Context only, not attributed or flagged as a finding.)
- `sibling examples and adapter-kit consumers`: Regression Hunt: trace whether the demo or shared adapter-kit affects other examples (http-graph, external-adapter-graph) -> clean (openapi-adapter.mjs imports shared adapter-kit/adapter.mjs without modifying it; sibling run.sh files unchanged.)
- `examples/openapi-graph/run.sh vs examples/http-graph/run.sh`: Convention Check: compare against established sibling demo pattern -> clean (Matches sibling: demo-only signing env defaults, mktemp receipt dir, background server + EXIT trap. Adds a kill -0 liveness check, an improvement.)
- `run.sh demo signing identity`: Hardcoded secret check -> clean (RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64 is an explicitly demo-only, env-overridable seed identical to the established http-graph pattern; not a real credential.)
- `openapi-adapter.mjs offline fallback and run.sh verification`: Dark Patterns: hunt for silent partial success, races, and fragile coupling -> finding (Offline fallback returns executed:false but harness asserts executed:true so misconfig is caught. Verification reads intermediate graph-state file instead of sealed receipt -> low non-blocking robustness finding.)

Findings:
- [low/non-blocking] `openapi-front-verify-coupling` run.sh asserts GET execution from intermediate graph-state checkpoint files rather than the sealed receipt
  - Location: `examples/openapi-graph/run.sh:49`
  - Evidence: The node verifier iterates `runs/*.graph-state.json` and inspects `checkpoint.steps[step_id==call].outputs` for executed/status_code/response. In skill_run.rs, write_graph_state is only called for non-succeeded checkpoints (execution/skill_run.rs:547,562); the terminal succeeded checkpoint writes receipts only. The graph-state file the demo reads exists solely because the step-at-a-time resume loop persists a non-terminal checkpoint after the call step runs. The sibling http-graph demo instead greps the sealed receipt content directly.
  - Impact: The demo's pass/fail is coupled to a transient intermediate checkpoint artifact rather than the governed sealed receipt. If runtime resume semantics ever seal a single-step graph without first persisting an intermediate state, the demo would report failure despite a correct sealed receipt. Currently works (acceptance passed).

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

Status: passed
Started: 2026-06-04T06:17:42Z
Ended: 2026-06-04T21:53:22Z

Checks:
- Path audit
  - Grounded in: code:examples/openapi-graph/X.yaml:9
  - Result: passed
  - Evidence: `examples/openapi-graph` has an inline harness; `examples/openapi-tool`
- Command audit
  - Grounded in: code:examples/openapi-graph/run.sh:28
  - Result: passed
  - Evidence: `sh examples/openapi-graph/run.sh` exited 0 and confirmed
- Scope/migration audit
  - Grounded in: code:examples/openapi-tool/SKILL.md:2
  - Result: passed
  - Evidence: Scope is the external-adapter OpenAPI front only; no
- Acceptance timing audit
  - Grounded in: spec_gap:Acceptance
  - Result: passed
  - Evidence: Placeholder harness command was replaced with concrete run script;
- Rollback/repair audit
  - Grounded in: spec_gap:Rollback
  - Result: passed
  - Evidence: The front is additive and remains on `external-adapter`; rollback
- Design challenge
  - Grounded in: spec_gap:Summary
  - Result: passed
  - Evidence: The spec was narrowed from arbitrary OpenAPI product breadth to the

Issues:
- none


## Planning Log

- none
