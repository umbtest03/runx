---
spec_version: '2.0'
task_id: rust-registry-client
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust registry client

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Covers `runx skill
search / add / inspect / publish` and `runx list`.
Blockers: `rust-runtime-skeleton`, `rust-contracts-parity` (registry types
present).
Allowed follow-up command: `scafld harden rust-registry-client`
Latest runner update: none
Review gate: not_started

## Summary

Port the registry client (skill search, add, inspect, publish, and list)
to a Rust crate. Today this lives in TS across the CLI dispatch and the
runner's `registry-resolver.ts`. The Rust client speaks the same HTTP
contract published by `cloud/packages/api` and consumes
`runx-contracts::registry` types.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (skill / search / add / publish / list commands)
- `@runxhq/runtime-local` (registry-resolver)
- `crates/runx-runtime` (or new `crates/runx-registry-client`)
- `crates/runx-contracts` (registry shapes)
- `cloud/packages/api` (registry routes)

Current TypeScript sources:
- `packages/cli/src/dispatch.ts` (skill / search / add / publish / list)
- `packages/runtime-local/src/runner-local/registry-resolver.ts`
- `packages/runtime-local/src/runner-local/skill-install.ts`

Files impacted:
- `crates/runx-runtime/src/registry/client.rs` (or new crate)
- `crates/runx-runtime/src/registry/resolver.rs`
- `crates/runx-runtime/src/registry/install.rs`
- `fixtures/registry/**`

Invariants:
- HTTP contract version is owned by `cloud-http-contract-stabilization`;
  this spec consumes a specific version, it does not negotiate ad-hoc.
- Trust tiers (`first_party`, `verified`, `community`) round-trip identically.
- Registry namespace ownership rules are not duplicated; the client
  consumes server decisions.
- Skill install is idempotent; receipts capture the install action when
  invoked from a chain.

## Objectives

- Port registry client (search, get, list, publish).
- Port registry resolver used by the runner.
- Port skill install flow.
- Add fixture suite for each surface.

## Scope

In scope:
- Client, resolver, install.

Out of scope:
- Cloud-side registry logic.
- Registry signing / attestation hierarchy (already covered by
  `registry-release-distribution-hardening` draft).

## Dependencies

- `rust-runtime-skeleton`.
- `rust-contracts-parity`.
- `cloud-http-contract-stabilization` for the registry HTTP surface.
- `rust-ts-interop-boundary` for the cross-language crossing reference.

## Open Questions

- Whether registry client is a feature on `runx-runtime` or a separate
  crate. Lean: feature; revisit if registry surface becomes large.
