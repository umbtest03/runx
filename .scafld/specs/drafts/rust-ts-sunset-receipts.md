---
spec_version: '2.0'
task_id: rust-ts-sunset-receipts
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T06:12:36Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# TS sunset: receipts

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Fifth TS sunset,
reframed around post-cutover harness receipts.
Blockers: `rust-ts-sunset-executor` complete,
`runx-contract-spine-hard-cutover` complete, `rust-receipts-parity` complete
against post-cutover harness receipts, and the proof/tree/path receipt specs
complete.
Allowed follow-up command: `scafld harden rust-ts-sunset-receipts`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/core/src/receipts/`. The live receipt contract after the hard
cutover is a sealed harness receipt, not the retired TS `LocalSkillReceipt` or
`LocalGraphReceipt` shape. A skill receipt is a sealed leaf harness receipt. A
graph receipt is a sealed parent harness receipt that links child harness
receipts and verifies their refs through `runx-receipts`.

No production caller may continue importing, emitting, reading, adapting, or
aliasing the old TS receipt model. Verification and public projections come
from `runx-receipts` over first-class sealed harness nodes.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `@runxhq/contracts`
- `crates/runx-receipts`
- `crates/runx-runtime`
- `cloud/packages/receipts-store` (must already consume post-cutover harness
  receipt contracts or be on its own sunset path before this spec starts)

Current TypeScript sources:
- `packages/core/src/receipts/**` (to be deleted)

Files impacted:
- `packages/core/src/receipts/` (deleted)
- `packages/core/src/index.ts`
- tests and fixtures that still import retired receipt helpers or assert old
  receipt shapes

Receipt model:
- Receipt nodes are first-class sealed harness receipts with the canonical
  post-cutover schema from `runx-contract-spine-hard-cutover`.
- A skill receipt is a sealed leaf harness: no child harness receipt refs, one
  governed skill execution boundary, contained acts/decisions/proofs, and a
  proof-verifiable body/full digest.
- A graph receipt is a sealed parent harness receipt: it links child harness
  receipt refs, binds child digest/proof expectations, and verifies recursive
  integrity through `rust-receipt-tree-resolution`.
- Public receipt references point at harness receipts and contained act or
  decision ids. They do not point at retired `rx_`/`gx_` local receipt files or
  suffix-based child lookup.

## Invariants

- Live governed paths use only post-cutover harness receipts.
- Existing pre-cutover receipts on disk are either migrated, archived, or read
  through an explicitly offline archival verifier. They are not accepted by
  live runtime, CLI, cloud, or receipt-store paths after this sunset.
- TypeScript receipt deletion cannot proceed while any live caller depends on
  old receipt field names, old digest semantics, suffix child lookup,
  repo-local receipt path discovery, `LocalSkillReceipt`, or
  `LocalGraphReceipt`.
- The Rust receipt stack owns canonical serialization, body/full digest
  computation, proof verification, redaction checks, parent/child tree
  resolution, and safe path discovery.
- Fixture history must preserve the reviewer/security oracle without keeping
  retired live reader behavior. Old fixtures can be archived as historical
  artifacts only if they are excluded from active parity and runtime
  acceptance.
- The final fixture catalogue covers at least denied approval, approval
  round-trip, leaf skill receipt, graph fanout parent receipt, replay, child
  harness, verification form, abnormal seal, digest tamper, signature tamper,
  redaction mismatch, missing child, malformed child ref, and external
  proof-missing cases.

## Sequencing

- `runx-contract-spine-hard-cutover` is the source of truth for the hard
  cutover receipt shape: sealed harness receipts, contained acts, contained
  decisions, authority, signals, refs, fingerprints, redaction refs, and proof
  bindings.
- `rust-receipts-parity` must target that post-cutover harness receipt shape.
  It must not use pre-cutover `LocalSkillReceipt` or `LocalGraphReceipt` bytes
  as the parity target unless a prior phase explicitly names an offline
  archival migration fixture and keeps it outside live acceptance.
- Byte-identical means byte-identical within one fixed post-cutover receipt
  shape: canonical harness receipt JSON, stable key order, normalized fixture
  ids/timestamps where applicable, deterministic child ordering where order is
  semantic, and exact hash inputs shared with `runx-receipts`.
- Byte-identical does not mean retired TS local receipt bytes equal Rust
  harness receipt bytes across the hard cutover. The cutover intentionally
  changes the contract shape.
- This sunset starts only after every live producer and consumer has already
  moved to the post-cutover harness receipt contract. If any live caller still
  needs retired TS receipts, this spec pauses and that caller gets a migration
  or archival-read spec first.

## No-Legacy Rule

- Do not keep compatibility readers, write shims, field aliases, export
  aliases, deprecated wrappers, or dual emission for
  `LocalSkillReceipt`/`LocalGraphReceipt`.
- Do not accept old receipt field names such as `skill_execution`,
  `graph_execution`, `skill_name`, `graph_name`, `source_type`, `owner`,
  `childReceipts`, or repo-local path discovery fields in live receipt
  readers. Canonical child harness receipt refs remain allowed.
- Do not add `v2` schema ids, `runx.*.v2` compatibility contracts, or
  version-switched readers to bridge old and new receipt models. The hard
  cutover uses the canonical harness receipt contract and removes superseded
  names instead of aliasing them.
- Do not preserve public exports whose only purpose is to keep old TS import
  paths compiling. Callers must import the post-cutover contract or
  `runx-receipts` verifier APIs directly.

## Objectives

- Enumerate every importer of `packages/core/src/receipts/**` and retire or
  migrate it before deletion.
- Prove live receipt producers emit sealed harness receipts for both leaf
  skills and parent graphs.
- Prove `runx-receipts` covers proof, tree, redaction, and runtime path
  discovery responsibilities through the dedicated Rust specs.
- Preserve fixture coverage for security/reviewer behavior without making old
  TS receipt shapes a live compatibility target.
- Delete the TS receipts implementation and remove the `@runxhq/core` export
  surface for retired receipt APIs.

## Scope

In scope:
- Deleting the TS receipts implementation from `@runxhq/core`.
- Removing retired receipt exports from `packages/core/src/index.ts`.
- Updating tests/fixtures that still assert retired TS receipt fields so active
  assertions target sealed harness receipts.
- Adding static guards that prevent reintroducing live imports or aliases for
  retired receipt types.

Out of scope:
- Cloud receipts-store internal changes. Cloud must already be migrated or
  tracked by its own blocking sunset spec.
- Changing the canonical harness receipt contract.
- Adding an archival verifier for pre-cutover receipts. If required, that is a
  separate offline-only spec with explicit non-live routing.
- Renaming public harness receipt schemas or introducing a second receipt
  version.

## Dependencies

- `rust-ts-sunset-executor` complete.
- `runx-contract-spine-hard-cutover` complete and reviewed; canonical harness
  receipt contract is frozen for this sunset.
- `rust-harness` complete enough to supply canonical post-cutover harness
  receipt replay fixtures for leaf skills and graph parents.
- `rust-receipts-parity` complete against post-cutover harness receipts, not
  retired TS local receipt bytes.
- `rust-receipt-proof-verification` complete for body/full digest checks,
  signature/proof verification, tamper failures, abnormal seals, redaction
  mismatches, and external proof-missing cases.
- `rust-receipt-tree-resolution` complete for parent graph harness receipts,
  child harness receipt refs, missing child failures, malformed refs, and
  recursive digest/proof recomputation.
- `rust-runtime-receipt-path-discovery` complete for runtime-owned receipt
  discovery and public projection paths, with no repo-local legacy lookup.
- `rust-approval-gate-parity` complete for approval request/decision receipt
  round-trip evidence.
- Cloud receipts-store migration or sunset decision recorded before build:
  either it consumes harness receipt contracts, or a blocking cloud spec owns
  the remaining retired receipt dependency.

## Planned Phases

Phase 1: importer and shape audit.
- Search active source, tests, fixtures, scripts, and cloud touchpoints for
  `packages/core/src/receipts`, `LocalSkillReceipt`, `LocalGraphReceipt`,
  retired receipt field names, `rx_`/`gx_` live lookup, and suffix child lookup.
- Classify each hit as already migrated, active blocker, fixture archive, or
  false positive.
- Record the cloud receipts-store state and block this spec if any live cloud
  path still requires retired receipt types.

Phase 2: receipt fixture and verifier handoff.
- Confirm leaf skill and graph parent harness receipt fixtures exist and
  validate through `runx-receipts`.
- Confirm approval round-trip, graph fanout, replay, child harness, abnormal
  seal, digest tamper, signature tamper, redaction mismatch, malformed child
  ref, and external proof-missing cases are covered by Rust validation.
- Move or rewrite any active TS fixture expectations that still assert retired
  receipt fields.

Phase 3: delete TS receipt implementation.
- Delete `packages/core/src/receipts/**`.
- Remove retired exports from `packages/core/src/index.ts`.
- Delete or update tests that exercised only retired TS receipt implementation
  details.
- Keep active tests focused on public harness receipt behavior and Rust
  verifier acceptance.

Phase 4: no-legacy guardrails.
- Add or extend static checks so live source cannot import retired receipt
  paths, reference retired receipt type names, or introduce `runx.*.v2`
  receipt schema ids.
- Ensure fixtures under active runtime/replay paths reject retired receipt
  fields instead of silently ignoring them.
- Verify generated package surfaces no longer expose retired receipt names.

Phase 5: final validation and review evidence.
- Run the full validation command set below.
- Attach importer audit evidence, fixture catalogue evidence, and no-legacy
  scan output to this spec's completion notes.
- Confirm all changed files are limited to this sunset's implementation scope.

## Acceptance Criteria

- No active source imports `packages/core/src/receipts/**` or imports retired
  receipt names from `@runxhq/core`.
- `packages/core/src/receipts/` is deleted, and `packages/core/src/index.ts`
  no longer exports receipt APIs that model `LocalSkillReceipt` or
  `LocalGraphReceipt`.
- Live skill execution emits and verifies a sealed leaf harness receipt.
- Live graph execution emits and verifies a sealed parent harness receipt that
  links child harness receipt refs and fails closed on missing, malformed, or
  digest-mismatched children.
- Receipt equality tests compare canonical post-cutover harness receipt bytes
  within the fixed harness receipt shape. They do not compare Rust output to
  retired TS `LocalSkillReceipt` or `LocalGraphReceipt` bytes.
- `runx-receipts` proof verification, not structural JSON comparison alone,
  validates body digest, full digest, signatures/proofs, abnormal seals,
  redaction refs, and external proof requirements.
- Active fixtures contain no retired receipt expectation fields under receipt
  assertions.
- Pre-cutover receipt fixtures, if retained, live only in an explicitly named
  archive path and are not loaded by live runtime, CLI, cloud, or Rust parity
  validation.
- No compatibility readers, aliases, deprecated exports, dual-emission paths,
  or `v2` receipt schema ids are introduced.
- Cloud receipts-store has no live dependency on old TS receipt field names,
  old digest semantics, suffix child lookup, or repo-local path discovery
  before deletion lands.

## Validation Commands

```sh
test ! -d packages/core/src/receipts
SCAN_ROOTS="apps bindings crates packages fixtures schemas scripts tests tools"
! rg -n "packages/core/src/receipts|LocalSkillReceipt|LocalGraphReceipt" $SCAN_ROOTS
! rg -n "skill_execution|graph_execution|skill_name|graph_name|source_type|childReceipts|receiptPath|receipt_path" $SCAN_ROOTS
! rg -n "runx\\.[a-z0-9_.-]+\\.v2|receipt.*compat|compat.*receipt|legacy.*receipt|alias.*receipt" $SCAN_ROOTS
rg -n "runx\\.harness_receipt\\.v1|HarnessReceipt|harness receipt" $SCAN_ROOTS
cargo test --manifest-path crates/Cargo.toml -p runx-receipts
cargo test --manifest-path crates/Cargo.toml -p runx-runtime receipt
cargo test --manifest-path crates/Cargo.toml -p runx-runtime harness
pnpm test -- packages/core
pnpm test -- packages/runtime-local/src/harness
pnpm exec tsx scripts/generate-rust-harness-fixtures.ts --check
pnpm boundary:check
pnpm rust:check
```

Cloud-specific validation is required before completion if the sibling cloud
checkout exists:

```sh
test ! -d ../cloud || ! rg -n "LocalSkillReceipt|LocalGraphReceipt|skill_execution|graph_execution|source_type|childReceipts|receipt_path|receiptPath" ../cloud
test ! -d ../cloud || pnpm --dir ../cloud test -- receipts harness
```

## Rollback And Repair

- Before Phase 3, rollback is to keep the TS receipt implementation in place
  and continue blocking this sunset on the importer or fixture that still
  requires it.
- After Phase 3, rollback is a normal revert of this sunset's deletion commit
  only if a live importer was missed. Do not add compatibility aliases or dual
  readers as rollback.
- If a fixture was upgraded incorrectly, repair the canonical harness receipt
  fixture and Rust verifier expectation together. Do not restore retired
  receipt fields to active fixtures.
- If cloud still needs retired receipt types, pause this spec and complete the
  cloud migration/sunset first.
- If byte comparison fails because a fixture crosses the hard cutover, remove
  that comparison from live acceptance or explicitly sequence it as an
  offline archival migration fixture. Do not redefine byte-identical to span
  two receipt contract shapes.

## Open Questions

- Whether cloud receipts-store gets its own Rust port before this sunset.
  If yes, that is an additional cloud spec; if no, the cloud-side must keep a
  contract-typed post-cutover harness receipt view via `runx-contracts`.
- Whether an offline archival verifier for pre-cutover local receipts is needed
  for customer support. Default: no live compatibility path; archival support,
  if required, is separate and cannot block deletion once live callers are
  migrated.
