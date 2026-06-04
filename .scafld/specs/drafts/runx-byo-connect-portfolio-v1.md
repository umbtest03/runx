---
spec_version: '2.0'
task_id: runx-byo-connect-portfolio-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T09:09:45Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# runx-byo-connect-portfolio-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld approve runx-byo-connect-portfolio-v1`
Latest runner update: none
Review gate: not_started

Roadmap: Wave 2 (the local provider unlock) feeding Wave 3 (the non-GitHub
portfolio). The highest-leverage OSS work after the magnet + heroes.

## Summary

Close the OSS side of the BYO provider gap: a per-run local credential descriptor
must reach a graph-step `http` source as a scoped secret header, seal a receipt,
and record only non-secret credential-delivery evidence. Hosted OAuth brokerage,
grant issuance, and credential custody remain cloud/private per
`docs/licensing-boundary.md`; OSS proves consumption over the already-shipped HTTP
front and then uses that path to build the demand-shaped non-GitHub skill
portfolio (search / mail / calendar / db / browser).

## Objectives

- A locally supplied credential descriptor reaches a non-GitHub graph HTTP step
  via `${secret:NAME}`, with no argv secret and no raw secret in outputs, graph
  state, or sealed receipt metadata.
- A governed non-GitHub provider read seals a receipt and records a
  receipt-safe `CredentialDeliveryObservation`.
- The first portfolio skills over the http/external-adapter fronts (sql-analyst,
  inbox-and-calendar-exec, knowledge-router, deep-research-brief, lead-enrichment),
  each maturity-tiered with a harness case.

## Scope

In scope:
- Local `--credential` + `--secret-env` consumption through graph HTTP steps.
- A runnable non-GitHub HTTP provider example proving the descriptor -> header ->
  receipt path.
- The first ~5 non-GitHub skills over the shipped http front (and the OpenAPI front
  for spec-backed APIs), using local/fixture descriptors in OSS; harness +
  maturity tiering.

Out of scope:
- GitHub (already wired).
- Hosted OAuth brokerage, hosted connect-session UX, credential custody, grant
  issuance, and grant revocation (cloud/private).
- Deep per-provider polish / the full ~351-provider sprawl (start with high-demand).

## Dependencies

- SHIPPED: the HTTP front, credential delivery contracts, and local per-run
  credential descriptors.
- The OpenAPI front (Wave 2) for spec-backed providers.

## Assumptions

- The HTTP front already governs any REST provider once a credential is delivered
  (verified shipped: method+URL+headers, SSRF/private-net opt-in, `${secret:NAME}`
  headers). This spec proves graph-step delivery; hosted OAuth remains a separate
  cloud dependency, not an OSS runtime prerequisite.

## Touchpoints

- The HTTP front (`adapters/http.rs`), graph skill execution
  (`execution/skill_run.rs`), local credential provision tests, the BYO HTTP
  example, and the new portfolio skills + official lock + maturity tiers.

## Risks

- **Provider sprawl.** Mitigation: start with a few high-demand providers; the front
  generalizes, the demand does not.
- **Auth-scope correctness.** Mitigation: local descriptors carry explicit scopes;
  hosted OAuth scope negotiation stays cloud/private.

## Acceptance

Profile: strict

Validation:
- A credentialed local fixture read runs through the graph HTTP front; the response
  seals and the graph state/receipt metadata carry a non-secret credential
  observation.
- The first portfolio skills run under local/fixture descriptors, seal receipts,
  and are maturity-tiered + locked.
- `pnpm verify:fast` + the new harness cases green.

## Phase 1: Local credential descriptor + graph HTTP demo

Status: pending
Dependencies: HTTP front (shipped), local credential descriptors (shipped)

Objective: a locally supplied credential descriptor reaches a non-GitHub HTTP
graph step and seals a receipt without exposing secret material.

Changes:
- Thread `--credential` + `--secret-env` delivery through graph execution options.
- Verify `examples/byo-http-graph` + `examples/byo-http-tool` using
  `${secret:RUNX_EXAMPLE_CRM_TOKEN}` against a local non-GitHub HTTP fixture.

Acceptance:
- [ ] `ac1` command - non-GitHub local credential read seals
  - Command: `sh examples/byo-http-graph/run.sh`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: The first non-GitHub portfolio skills

Status: pending
Dependencies: Phase 1

Objective: the demand-shaped seeds run over the http/external-adapter fronts using
local/fixture descriptors in OSS.

Changes:
- Build sql-analyst, inbox-and-calendar-exec, knowledge-router,
  deep-research-brief, lead-enrichment; harness + maturity. Keep live hosted OAuth
  provider brokerage as a cloud/private dependency.

Acceptance:
- [ ] `ac2` command - portfolio skills run + are tiered
  - Command: `runx harness skills/<each-seed>/<case>.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Phase 1 is additive runtime/example work; revert the graph credential-delivery
  patch and example files if it regresses. Portfolio skills are additive +
  maturity-gated (alpha first). Hosted OAuth/connect changes are out of OSS scope.

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

- none

## Planning Log

- none
