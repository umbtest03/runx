---
spec_version: '2.0'
task_id: monolith-decomposition-v1
created: '2026-05-24T00:00:00Z'
updated: '2026-05-24T00:00:00Z'
status: draft
harden_status: not_run
size: large
risk_level: medium
---

# Decompose god-files and retire the large-file waivers

## Current State

Status: draft
Current phase: planning
Next: harden
Reason: A+ roadmap step 4. Several files mix multiple responsibilities and exceed
their budgets; some Rust files carry explicit `// rust-style-allow: large-file`
waivers that are deferred decompositions. TS offenders: `runtime-local/src/sdk/
index.ts` (~1800 lines: SDK + ingestion + config + marketplace),
`runner-local/graph-governance.ts` (~1400), `runner-local/index.ts` (~1260),
`packages/cli/src/index.test.ts` (~2200). Rust waivers: `runx-runtime/src/
receipts/seal.rs`, `runx-runtime/src/payment/ledger.rs`, `runx-receipts/src/
tree.rs`.
Blockers: the runtime-local TS offenders overlap the `rust-ts-sunset-*` work; if
those modules are slated for deletion, decompose only what survives the sunset to
avoid churning soon-to-be-deleted code.

## Summary

Split each god-file into single-responsibility modules behind unchanged public
surfaces, and retire each `rust-style-allow: large-file` waiver by actually
splitting the file (or recording why it must stay). Behavior-preserving
refactor: no contract, wire, or test-behavior change.

## Objectives

- Each listed file drops below its budget by extracting cohesive submodules; the
  crate/package public API is unchanged.
- Every `rust-style-allow: large-file` waiver is either removed (file split) or
  re-justified with a concrete reason and a follow-up.
- The `runx doctor` monolith-budget diagnostic passes without per-file waivers
  for the decomposed files.

## Scope

In scope: the listed TS and Rust files and their internal module boundaries.

Out of scope: any behavior, contract, or public-API change; files owned by an
active sunset spec slated for deletion (decompose post-sunset instead).

## Dependencies

- Coordinate the runtime-local TS files with `rust-ts-sunset-runtime-local`
  (don't decompose code about to be deleted).

## Acceptance

- [ ] `dod1` Each listed file is below its monolith budget, split into cohesive
  modules, with unchanged public exports.
- [ ] `dod2` No `rust-style-allow: large-file` waiver remains on a decomposed
  file; any surviving waiver carries a concrete justification.
- [ ] `dod3` `verify:fast` + `cargo test --workspace` stay green; no behavior or
  wire change.

## Origin

A+ roadmap (2026-05-24), step 4. Surfaced by the structural review; the waivers
are self-documenting deferred decompositions.
