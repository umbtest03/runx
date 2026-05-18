---
spec_version: '2.0'
task_id: rust-receipts-parity
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T14:04:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Rust harness receipt parity

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`; reframed after
`runx-contract-spine-hard-cutover` so receipts are sealed harness nodes.
Blockers: `runx-contract-spine-hard-cutover`, `rust-parser-parity` complete
(receipts reference validated skill shapes), and `rust-approval-gate-parity`
scoped (receipts must capture approval round-trips).
Allowed follow-up command: `scafld harden rust-receipts-parity`
Latest runner update: none
Review gate: not_started

## Summary

Replace the `runx-receipts` placeholder with a real Rust receipts crate
covering the post-cutover harness receipt model, deterministic hashing, and
verification rules. After the hard cutover, a receipt is a sealed harness node:
a leaf harness receipt corresponds to a skill hop, and a parent harness receipt
links child harness receipts into graph proof.

`packages/core/src/receipts/` is authoritative only until the contract spine
hard cutover. This spec does not port retired pre-cutover receipt shapes as the
lasting Rust target.

Receipts are the trust substrate for runx. Every appeal to "governed
execution" collapses without verifiable receipts on the Rust runtime path.
This spec ships the model + verification for harness receipts; receipt
production from `runx-runtime` is owned by the runtime adapter specs.

## Context

CWD: `.`

Packages:
- `@runxhq/core` (receipts subpath)
- `crates/runx-receipts`
- `crates/runx-contracts`
- `cloud/packages/receipts-store`

Current TypeScript sources:
- `packages/core/src/receipts/index.ts`
- `packages/core/src/receipts/*` (subfiles enumerated in Phase 1)
- `packages/core/src/receipts/index.test.ts`

Files impacted:
- `crates/runx-receipts/Cargo.toml`
- `crates/runx-receipts/src/lib.rs`
- `crates/runx-receipts/src/model.rs`
- `crates/runx-receipts/src/local.rs`
- `crates/runx-receipts/src/graph.rs`
- `crates/runx-receipts/src/hashing.rs`
- `crates/runx-receipts/src/verify.rs`
- `crates/runx-receipts/tests/receipt_fixtures.rs`
- `fixtures/receipts/**` (new)
- `scripts/generate-rust-receipt-fixtures.ts`

Invariants:
- TypeScript receipts remain authoritative only until the contract spine hard
  cutover replaces the receipt shape.
- Rust parity targets the ratified harness receipt shape when this spec runs
  after the hard cutover.
- Byte-for-byte parity is measured within one contract shape. It is not a
  promise that retired receipt JSON remains byte-identical after the hard
  cutover.
- Receipts are append-only; verification rules cannot weaken without a
  governance spec.
- Receipt hashing matches `runx-contracts::hash_stable` semantics already
  ported.
- No raw secrets in receipts. Scrub helpers stay parity-equivalent to TS.
- Receipt JSON keys are sorted deterministically; `BTreeMap` only.
- Approval round-trip envelopes (gate id, gate hash, decision, actor) are
  first-class receipt fields; their shape is defined in
  `rust-approval-gate-parity` and consumed here.
- Harness receipt verification includes signature validity, hash commitments,
  authority attenuation, criterion binding, redaction commitments, child
  receipt linkage, abnormal seal validity, and external attestations present.

## Objectives

- Port harness receipt, leaf receipt, and graph/parent receipt supporting
  types from the post-cutover contract spine.
- Port verification rules: hash matches, append-only constraints,
  schema validation, authority attenuation, criterion binding, redaction
  verification, child receipt linkage, and abnormal seal validation.
- Port the receipt-path discovery logic that the orchestrator uses to
  resolve receipt directories.
- Add cross-language fixtures covering: success receipts, denied receipts,
  approval-round-trip receipts, graph fanout receipts, replay receipts,
  child-harness receipts, verification-form receipts, and abnormal seals.
- Extend style/graph checks to cover `runx-receipts`.

## Scope

In scope:
- Receipt model parity and verification parity.
- Hashing helpers tied to the receipt contract.
- Approval-round-trip receipt envelope.
- Harness receipt canonicalization.
- Child receipt recursive verification.
- Act-ref resolution through harness receipt plus contained act id.

Out of scope:
- Cloud receipts-store HTTP shape (separate spec when cloud consumes Rust).
- Receipt search / export UX (`runx history`, `runx export-receipts`) which
  remain TS until their own ports.
- Legacy local receipt shape preservation after the hard cutover.

## Dependencies

- `runx-contract-spine-hard-cutover` approved.
- `rust-contracts-parity` complete.
- `rust-parser-parity` complete (validated skill / graph types referenced).
- `rust-approval-gate-parity` scoped (envelope fields available).

Sequencing:

- Preferred order: contract spine hard cutover first, then this spec ports the
  final harness receipt shape.
- If Rust receipt parity lands before the hard cutover, it is temporary parity
  for the old TS receipt implementation only and must be replaced by this spec
  before the launcher flip.
- `rust-runtime-skeleton`, `rust-ts-sunset-receipts`, and
  `rust-runtime-skill-execution` consume this spec after it targets the
  post-cutover harness receipt shape.

## Open Questions

- Whether `runx-receipts` adopts a small no-std-friendly subset for embedded
  verification consumers, or stays `std`-only per architecture doc section 8.
- Whether receipt path discovery belongs in `runx-receipts` (pure) or
  `runx-runtime` (impure). Current TS lives at runner-local; lean toward
  runtime.
- Which external attestations are mandatory for pass/fail verification versus
  "present but not independently recomputable".
