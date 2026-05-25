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

Status: implemented
Current phase: contract migration and boundary cleanup validated
Next: keep the boundary guard in the normal validation profile.
Reason: Credential envelope and authority-proof contracts now use
`provider_reference`, generated schemas require it, Rust and TypeScript contract
tests reject the legacy `connection_id` wire key, and fixtures use
`provider_reference`. The license-boundary manifest lists `connection_id` only
as a banned/guarded identifier and inventory term, not as an allowlist entry for
retained MIT source.
Blockers: none for this clean contract cutover.
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

This is a clean cutover. runx is still greenfields with no published external
consumers of the wire format, so the old `connection_id` key is renamed outright:
no serde alias, no dual-path, no compatibility shim.

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
- Remove the old `connection_id` wire key outright (no serde alias, no retained
  legacy field); fixtures and tests assert the new provider-opaque shape only.
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

- [x] `dod1` Public Rust contract types no longer expose a provider-shaped
  `connection_id` API as the preferred field name.
  - Command: `rg -n "connection_id|ConnectionId|connectionId" crates/runx-contracts crates/runx-core packages/contracts packages/cli schemas docs/license-boundary.manifest.json`
  - Expected kind: `reviewed_output`
  - Status: reviewed
  - Evidence: active credential contracts, generated schemas, OpenAPI runtime
    schemas, and CLI connect surfaces use `provider_reference`; remaining
    `connection_id` mentions are the negative contract test, this spec, docs
    inventory, and the boundary manifest's banned identifier.
- [x] `dod2` The old `connection_id` wire key is gone (no alias, no fallback); a
  test asserts the renamed provider-opaque field is the only accepted shape.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 schema generation/check passed after the
    `provider_reference` cutover.
- [x] `dod3` Authority-proof fixtures and schema validation match the chosen
  contract shape.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test schema_wire_compat emitted_schemas_are_wire_compatible_with_committed`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 runx-core policy fixtures and schema wire-compat test
    passed.
- [x] `dod4` `docs/license-boundary.manifest.json` no longer needs allowlist
  entries for `connection_id` in retained MIT source files.
  - Command: `node .scafld/scripts/check-license-edges.mjs --check manifest-complete && node .scafld/scripts/check-license-edges.mjs --check identifiers`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 license-boundary manifest completeness and identifier
    guards passed; `connection_id` remains only as a banned identifier.
- [x] `dod5` The license-boundary guard and runx-core policy tests pass.
  - Command: `node .scafld/scripts/check-license-edges.mjs --check manifest-complete && node .scafld/scripts/check-license-edges.mjs --check identifiers && cargo test --manifest-path crates/Cargo.toml -p runx-core policy`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 license-boundary guard and core policy tests passed.

Evidence (static, 2026-05-25):
- `crates/runx-contracts/src/policy_proof.rs` exposes
  `provider_reference` on `CredentialEnvelope` and authority-proof credential
  material.
- `schemas/credential-envelope.schema.json`,
  `schemas/authority-proof.schema.json`, and
  `packages/contracts/src/schemas/credentials.ts` use `provider_reference`.
- `packages/contracts/src/schemas/credentials.test.ts` and
  `crates/runx-contracts/tests/schema_wire_compat.rs` reject the legacy
  `connection_id` key.
- `docs/license-boundary.manifest.json` still contains `connection_id` only as a
  banned identifier and inventory search term; it is no longer an allowlisted
  retained MIT source field.

## Phase 1: Compatibility Design

Inventory every `connection_id` occurrence in retained MIT source and fixtures.
Record the exact renamed provider-opaque wire contract before implementation. No
alias path; this is a clean rename.

## Phase 2: Contract Migration

Implement the chosen field rename or versioned shape in Rust policy types,
authority-proof projection, fixtures, schema validation, and docs.

## Phase 3: Boundary Cleanup

Remove the `connection_id` allowlist entries and rerun the license-boundary guard
plus runx-core policy tests.

## Rollback

Revert the rename to restore the prior field name and re-run the boundary guard
and policy tests. There is no compatibility layer to unwind (clean cutover).

## Origin

Follow-up from `connect-auth-mit-boundary-v1` review on 2026-05-22: removing OSS
brokerage was completed, but the legacy envelope vocabulary still exposes
`connection_id` as a typed public field. That is a contract migration, not a
licensing-boundary move.
