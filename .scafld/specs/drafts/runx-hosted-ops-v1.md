---
spec_version: '2.0'
task_id: runx-hosted-ops-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-05T03:25:35Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# runx-hosted-ops-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: last readiness lane; hosted launch work starts only after OSS gates, demos, action layer, outbox, live rails, and release readiness are clean
Blockers: runx-release-readiness-v1 should land first
Allowed follow-up command: `scafld approve runx-hosted-ops-v1`
Latest runner update: none
Review gate: not_started

## Summary

Hosted ops is not a demo lane. It is the public hosted-launch lane after the OSS
surface is sealed: resident-kernel transport, process lifecycle, isolation,
secrets, backups, readiness, marketplace trust, issuer key publication, and cloud
operational UX. The local OSS kernel remains the source of truth; hosted wraps it
with production lifecycle and trust machinery.

## Objectives

- Resident-kernel transport behind the cloud-to-kernel bridge, with per-principal
  isolation and no spawn-per-run scaling bottleneck for steady hosted operation.
- Process lifecycle: pooling, crash recovery, graceful drain, observability, and
  bounded concurrency across many integrations.
- Deploy ops: secrets, backups, manifests, readiness probes, rollback, and
  operational runbooks that match the running processes.
- Marketplace trust: non-GitHub author verification, maturity graduation,
  moderation/abuse handling, and public issuer JWKS publication.
- Hosted UX only after the operational substrate exists.

## Scope

In scope:
- Cloud resident-kernel transport and lifecycle work.
- Hosted deploy/secrets/backups/readiness.
- Marketplace trust and issuer key publication.
- Hosted UX needed to operate the launch safely.

Out of scope:
- Local OSS cleanup, demo gallery work, A2A, new rails, and demo-specific polish.
- Any fallback that duplicates the OSS kernel semantics in TypeScript.

## Dependencies

- `runx-readiness-gate-hardening-v1`
- `runx-demo-surface-prune-v1`
- `runx-operational-action-layer-v1`
- `runx-thread-outbox-product-cutover-v1`
- `runx-live-rail-verification-v1`
- `runx-release-readiness-v1`

## Assumptions

- The existing hosted path can remain spawn-per-run until the resident transport
  is proven.
- Hosted code may be edited, but OSS kernel semantics remain authoritative.

## Risks

- **Cloud topology rebuild.** Mitigation: stage resident-kernel transport behind a
  feature gate and retain current hosted execution until cutover proof is green.
- **Secret handling drift.** Mitigation: use central secret management and add
  explicit tests/guards for no secret persistence in receipts/logs.
- **Marketplace trust gaps.** Mitigation: block public hosted launch until issuer
  keys, author verification, and moderation surfaces exist.

## Acceptance

Profile: strict

Validation:
- Hosted resident kernel serves multiple principals with isolation.
- Lifecycle tests prove crash recovery, drain, and concurrency limits.
- Deploy readiness checks cover secrets, backups, manifests, and rollback.
- Marketplace trust checks cover author verification, maturity, moderation, and
  JWKS publication.
- No cloud TypeScript path reimplements OSS kernel decisions.

## Phase 1: Resident-kernel transport

Status: pending
Dependencies: release readiness

Objective: hosted execution uses a resident kernel with per-principal isolation.

Acceptance:
- [ ] `p1_ac1` manual - resident kernel isolation and lifecycle proof
  - Expected kind: `manual`
  - Status: pending

## Phase 2: Deploy ops and lifecycle

Status: pending
Dependencies: Phase 1

Objective: production operations are explicit and tested.

Acceptance:
- [ ] `p2_ac1` manual - secrets/backups/manifests/readiness/rollback proof
  - Expected kind: `manual`
  - Status: pending

## Phase 3: Marketplace trust and hosted UX

Status: pending
Dependencies: Phase 2

Objective: public hosted launch is trust-ready.

Acceptance:
- [ ] `p3_ac1` manual - marketplace trust and JWKS publication proof
  - Expected kind: `manual`
  - Status: pending

## Rollback

- Keep spawn-per-run hosted execution available until resident-kernel transport is
  proven. Revert hosted transport/lifecycle changes together.

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
