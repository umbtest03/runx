---
spec_version: '2.0'
task_id: credential-envelope-opaque-reference-v1
created: '2026-05-22T03:18:00+10:00'
updated: '2026-05-22T03:18:00+10:00'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# Credential envelope opaque reference v1

## Current State

Status: draft
Current phase: planning
Next: harden
Reason: `connect-auth-mit-boundary-v1` removed OSS brokerage, but retained
legacy public `connection_id` wire fields as passive compatibility metadata.
This spec owns the separate contract migration to provider-opaque naming.
Blockers: `credential-broker-delivery-contract-v1` and downstream hosted
credential envelope consumers must confirm wire-compatibility requirements.
Allowed follow-up command: `scafld harden credential-envelope-opaque-reference-v1`
Latest runner update: 2026-05-22T03:18:00+10:00
Review gate: not_started

## Summary

Migrate the public credential envelope and authority-proof credential material
away from typed `connection_id` naming toward provider-opaque reference fields.
The MIT boundary already prevents OSS from brokering OAuth, calling Nango, or
constructing provider-specific `nango:<provider>:<connection_id>` locators. This
spec cleans up the remaining contract vocabulary so passive metadata does not
continue to encode a provider-shaped concept.

The migration must be wire-safe: either preserve backward-compatible serde
aliases during a planned transition or explicitly coordinate a versioned contract
break with the published contract consumers.

## Context

- `connect-auth-mit-boundary-v1` deliberately kept
  `CredentialEnvelope.connection_id` and the authority-proof projection as
  allowlisted legacy metadata rather than changing contract wire shape during a
  licensing-boundary refactor.
- The durable crossing is already `material_ref` plus the opaque
  `MaterialResolver` contract.
- The remaining problem is vocabulary and compatibility, not OAuth brokerage.

## Objectives

- Replace the provider-shaped public field name with provider-opaque naming such
  as `provider_reference`, `credential_reference`, or an equivalent agreed
  contract term.
- Keep or version the old `connection_id` wire key deliberately, with tests that
  prove old envelopes either still deserialize through an alias or fail only
  under an intentional version break.
- Update authority-proof metadata projection, fixtures, schemas, docs, and
  downstream tests consistently.
- Remove the `connection_id` allowlist entries from
  `docs/license-boundary.manifest.json` once the public contract migration lands.

## Scope

In scope:
- `crates/runx-core/src/policy/types.rs`
- `crates/runx-core/src/policy/authority_proof.rs`
- Credential envelope and authority-proof fixtures and schema validation.
- Boundary guard manifest cleanup after the migration.

Out of scope:
- Reintroducing hosted connect/OAuth brokerage to MIT crates.
- Changing the opaque `MaterialResolver` consumption contract.
- Cloud implementation work, except for coordinated fixture and compatibility
  evidence recorded in this spec.

## Acceptance

- [ ] `dod1` Public Rust contract types no longer expose a provider-shaped
  `connection_id` API as the preferred field name.
- [ ] `dod2` Backward compatibility for existing `connection_id` wire payloads is
  either proven with serde alias tests or explicitly rejected by a versioned
  contract-break decision.
- [ ] `dod3` Authority-proof fixtures and schema validation match the chosen
  contract shape.
- [ ] `dod4` `docs/license-boundary.manifest.json` no longer needs allowlist
  entries for `connection_id` in retained MIT source files.
- [ ] `dod5` The license-boundary guard and runx-core policy tests pass.

## Phase 1: Compatibility Design

Inventory every `connection_id` occurrence in retained MIT source and fixtures,
then choose alias-preserving migration versus versioned break. Record the exact
wire contract before implementation.

## Phase 2: Contract Migration

Implement the chosen field rename or versioned shape in Rust policy types,
authority-proof projection, fixtures, schema validation, and docs.

## Phase 3: Boundary Cleanup

Remove the `connection_id` allowlist entries and rerun the license-boundary guard
plus runx-core policy tests.

## Rollback

If downstream compatibility cannot be proven, keep the legacy fields as passive
metadata and leave the boundary allowlist in place with this spec blocked.

## Origin

Follow-up from `connect-auth-mit-boundary-v1` review on 2026-05-22: removing OSS
brokerage was completed, but the legacy envelope vocabulary still exposes
`connection_id` as a typed public field. That is a contract migration, not a
licensing-boundary move.
