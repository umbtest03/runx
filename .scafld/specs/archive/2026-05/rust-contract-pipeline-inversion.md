---
spec_version: '2.0'
task_id: rust-contract-pipeline-inversion
created: '2026-05-21T23:10:00Z'
updated: '2026-05-25T03:11:04Z'
status: completed
harden_status: passed
size: large
risk_level: high
---

# Rust contract pipeline inversion

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-25T03:11:04Z
Review gate: pass

## Summary

Invert the contract source of truth from TypeScript (TypeBox) to Rust once the
Rust runtime is authoritative. Today the chain is one-directional with a gap:
TypeBox in `oss/packages/contracts/src/schemas/*.ts` is the oracle,
`oss/scripts/generate-contract-schemas.ts` generates `oss/schemas/*.json` under a
`--check` CI gate, and `oss/crates/runx-contracts` is a hand-written
reimplementation (~250 `pub struct`/`enum`, ~969 fields) policed only by example
fixtures. The end state: Rust contract types become the declarative source, emit
JSON Schema, and generate the published TypeScript types that surviving TS
consumers import. The hand-written TypeBox schemas are deleted.

This is a direction/sequencing spec, not a redesign. It does not change wire
shapes; it changes which representation is hand-authored and which is generated.

## Context

- The contract-spine duplication is a three-representation parity burden: JSON
  Schema (generated), TypeBox (authored), Rust structs (authored). Two are
  hand-maintained against each other.
- The mechanical mirroring is exactly what codegen removes: 54 `rename_all`
  attributes, 28 per-field `rename`, `Option<T>` + `skip_serializing_if` vs
  `Type.Optional`, `Vec<T>` + `#[serde(default)]` vs `Type.Array` defaulting.
- `rust-contract-schema-validation-gate` already lists pipeline inversion as
  explicitly out of scope; this spec owns it so the direction is not lost.
- Sequencing is load-bearing: inverting before Rust is authoritative would
  invert the parity tax onto the still-authoritative TS side. This must run
  after the sunset specs, not before.

## Objectives

- Choose the Rust-to-schema mechanism. A feasibility spike (2026-05-22) found
  vanilla `schemars` cannot reproduce the committed schemas (they are fully
  inlined — 1 `$ref` in 60 files — render enums as `anyOf` of `const`, and carry
  `x-runx-schema`/custom `$id`, none of which `schemars` emits), and `typify` is
  schema-to-Rust, the wrong direction. The realistic mechanism is a bespoke Rust
  schema emitter that controls shape directly. The invariant it must hold is
  **wire-compatibility, not byte-identity of the schema document** (see below).
- Express contract constraints at the Rust type level where the schemas do today
  only in JSON: model the ubiquitous `minLength:1` (2,382 occurrences) as a
  `NonEmptyString` newtype and the `const` discriminants as fixed/typed fields,
  so a contract type cannot be constructed in a shape its schema would reject.
- Flip the `--check` CI gate direction: Rust types generate JSON Schema; JSON
  Schema generates published TypeScript types for surviving consumers
  (`@runxhq/contracts`, `host-adapters`).
- Delete the hand-written TypeBox schemas once the generated TS types are the
  consumed artifact.
- Keep one canonicalization/fingerprint contract intact across the flip (depends
  on `canonical-json-fingerprint-contract-v1`).

## Scope

In scope:
- `runx-contracts` (and any contract types still living in `runx-core`, e.g.
  `AuthorityProof` if `rust-contract-schema-validation-gate` decides to relocate
  it) becoming schema-emitting.
- A generation + `--check` gate that replaces `generate-contract-schemas.ts`.
- Generating published TypeScript contract types for surviving TS consumers.

Out of scope:
- Changing any wire shape, casing, or optionality. Pure representation move.
- The runtime/CLI behavior. This is contracts-only.
- Sunsetting TS runtime packages (owned by `rust-ts-sunset-*`).

## Dependencies

- `rust-contract-schema-validation-gate` (Rust must demonstrably match the
  schemas first).
- `rust-ts-sunset-*` (TS must no longer be the authoritative runtime).
- `canonical-json-fingerprint-contract-v1` (the canonicalization byte contract
  must survive the flip unchanged).
- `ts-extension-survivorship-boundary` (defines which TS consumers still need
  generated contract types).

## Touchpoints

- `oss/crates/runx-contracts/src/*.rs`
- `oss/crates/runx-core/src/policy/types.rs` (if `AuthorityProof` relocates)
- `oss/scripts/generate-contract-schemas.ts` (replaced/inverted)
- `oss/packages/contracts/src/schemas/*.ts` (deleted once generated TS lands)
- `oss/schemas/*.json` (now a Rust-derived artifact)

## Risks

- Codegen drift: the Rust-emitted schema must accept/reject the same value domain
  as the current schemas, or downstream validators and cloud consumers break
  silently. Mitigate with a conformance-corpus wire-compatibility gate (not
  byte-equality) over the committed schemas before flipping.
- Schema-document consumers: a consumer that introspects schema *structure* or
  pins schema-document hashes (rather than validating data) could break on the
  idiomatic-shape change. Inventory these before the flip (dod6).
- Premature flip: running before the sunset specs lands the parity tax on the
  wrong side. Hard-gate on the dependencies above.
- Lost expressiveness: TypeBox constraints (formats, refinements) without a clean
  Rust equivalent must be inventoried; the bespoke emitter and the
  `NonEmptyString`/typed-discriminant model must cover the common cases
  (`minLength:1`, `const`) before commitment.

## Acceptance

- [ ] `dod1` The Rust-emitted JSON Schema is wire-compatible with every committed
  `oss/schemas/*.json` for the covered contract set: a conformance corpus
  validates identically (same accept/reject) against the prior and the
  Rust-emitted schema, and schema identity (`x-runx-schema`, `$id`, version) is
  preserved. The schema *document* shape may change to idiomatic JSON Schema;
  byte-identity is explicitly NOT required.
- [ ] `dod2` The `--check` gate is inverted: editing a Rust contract type and not
  regenerating fails CI; editing a hand-written TypeBox schema is no longer
  possible (files removed).
- [ ] `dod3` Surviving TS consumers (`@runxhq/contracts`, `host-adapters`) build
  against generated types, not hand-authored TypeBox.
- [ ] `dod4` `canonical-json-fingerprint-contract-v1` fixtures still pass
  unchanged across the flip.
- [ ] `dod5` No wire shape, casing, or optionality changed: serialized contract
  data is byte-identical across the flip and the canonical-json fixtures pass.
  (The schema document may change shape; only the validated value domain and the
  serialized wire bytes must be preserved.)
- [ ] `dod6` Before flipping, any consumer that structurally introspects schema
  documents or pins schema-document hashes (vs validating data) is inventoried;
  none break on the idiomatic-shape change.

## Phase 1: Emitter + Wire-Compatibility Drift Detector

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- Added `RunxSchema` derive coverage and a small bespoke schema emitter in
  `crates/runx-contracts`, including the schema artifact manifest and the
  `runx-contract-schemas` binary.
- Moved the remaining runtime-owned contract report shapes needed for schema
  emission into `runx-contracts` and re-exported them where runtime callers
  still need the public surface.
- Added `schema_wire_compat.rs` coverage over every committed schema artifact,
  including accept/reject corpora and `$id` / `x-runx-schema` identity checks.

Acceptance:
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test schema_wire_compat`
  passes.
- The Rust artifact manifest and committed `schemas/*.schema.json` set are the
  same set.

## Phase 2: Flip The Gate

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- Replaced the TypeScript TypeBox generator entrypoint with a wrapper around
  the Rust `runx-contract-schemas` binary.
- Extended the generator command to write the generated TypeScript schema
  artifact module consumed by `@runxhq/contracts`.
- Regenerated all committed `schemas/*.schema.json` documents from Rust.

Acceptance:
- `pnpm contracts:schemas:check` fails when either committed schema JSON or the
  generated TS schema artifact module is stale.
- Editing a Rust contract type without regenerating leaves the check red.

## Phase 3: Delete TypeBox

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- Removed the direct TypeBox dependency from `@runxhq/contracts`.
- Repointed `runxContractSchemas` and the public generated artifact export to
  the Rust-generated schema artifact map.
- Switched contract validation to Ajv 2020 over JSON Schema documents instead
  of TypeBox `Value.Check`.
- Recorded the schema consumer inventory in
  `docs/contract-schema-consumer-inventory.md`.

Acceptance:
- `rg '@sinclair/typebox' packages/contracts/src packages/contracts/package.json`
  returns no matches.
- `pnpm typecheck`, `pnpm contracts:schemas:check`, and `pnpm verify:fast`
  pass.
- The consumer inventory records no schema-document hash pins that break on
  idiomatic Rust-emitted schema documents.

## Rollback

The flip is gated on byte-equality; if generated schemas diverge from committed
schemas at any phase, do not flip. Phase 1 is non-authoritative and safe to land
independently as a drift detector even if the full inversion is deferred.

## Origin

User architecture review on 2026-05-21: the TS-first contract spine forces a
permanent hand-mirroring tax onto Rust (~250 types, ~969 fields) and the Rust
side is currently policed only by example fixtures. Once Rust is authoritative,
the source of truth should invert. Captured as the one cross-language abstraction
item not already owned by an existing spec.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-25T01:35:29Z
Ended: 2026-05-25T01:37:55Z

Checks:
- path audit
  - Grounded in: code:crates/runx-contracts/src/schema_artifacts.rs:34
  - Result: passed
  - Evidence: The published artifact list is now centralized in Rust and covers
- command audit
  - Grounded in: code:scripts/generate-contract-schemas.ts:1
  - Result: passed
  - Evidence: `pnpm contracts:schemas:check` now shells through the Rust
- scope/migration audit
  - Grounded in: code:crates/runx-contracts/tests/schema_wire_compat.rs:97
  - Result: passed
  - Evidence: The Rust wire-compat gate covers every committed schema artifact;
- acceptance timing audit
  - Grounded in: code:schemas/tool-manifest.schema.json:1
  - Result: passed
  - Evidence: The schema flip was run after Rust contract coverage reached
- rollback/repair audit
  - Grounded in: code:scripts/generate-contract-schemas.ts:12
  - Result: passed
  - Evidence: Rollback is a single generator path revert plus regenerated
- design challenge
  - Grounded in: code:crates/runx-contracts/src/host_protocol.rs:125
  - Result: passed
  - Evidence: The last missing non-tool schema was not papered over; the

Issues:
- none


## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Reviewed the Rust→JSON Schema inversion across the emitter primitives (`schema.rs`), derive macro (Option/serde semantics), 59-entry artifact manifest, wire-compat test, generator binary, TS consumer surface (`internal.ts` Ajv2020 wrapper + `index.ts` exports), and ambient runtime drift in `agent_invocation.rs`. Sampled new emitters (operational_policy, ledger, aster, act_assignment) and confirmed required-vs-required-nullable Option modelling matches the committed schemas, NonEmptyString conversions in runtime callers are safe (debug_assert + non-empty fallbacks), and the new `schema_generator_check.rs` test closes the prior F2 orphan-detection gap in `--check` mode. No completion blockers found. Two LOW residuals (write-mode orphan cleanup and unmigrated `runxAuxiliarySchemas`) are reported as non-blocking follow-ups, consistent with the prior review's F2/F3 framing.

Attack log:
- `crates/runx-contracts/src/schema.rs`: Check emitter primitives for $id/x-runx-schema injection, Identity::Runx auto-discriminant, nullable/any_of_with_identity correctness, NonEmptyString/IsoDateTime invariants. -> clean (Identity::Runx inserts schema discriminant via entry().or_insert_with(const_string(logical)); newtypes emit minLength:1 / iso pattern with debug_assert in From<String>.)
- `crates/runx-contracts-derive/src/lib.rs`: Probe Option/serde semantics: omittable vs required-nullable, flatten/tag/untagged handling, unit-only enum identity emission, deny_unknown_fields propagation. -> clean (omittable = optional && (has_skip || field_default || container_default); required = !(omittable || default); nullable = optional && !omittable. Matches committed schema shapes for sampled contracts.)
- `crates/runx-contracts/src/schema_artifacts.rs + bin/runx-contract-schemas.rs`: Reconcile 59-entry manifest against on-disk schemas; verify orphan handling in both check and write modes. -> finding (Orphan detection only runs in --check mode (new schema_generator_check.rs test confirms). Write-mode orphans persist and propagate via TS readdirSync — filed R1.)
- `crates/runx-contracts/tests/schema_wire_compat.rs`: Assess whether 59 Covered entries provide meaningful drift signal now that committed schemas are Rust-emitted. -> clean (Identity ($id + x-runx-schema) + accept/reject corpus parity holds. Tautology risk acknowledged in prior review's F3 framing; not a new blocker.)
- `crates/runx-contracts/src/{operational_policy,ledger,aster,act_assignment}.rs`: Verify new emitters model Option<T> without skip_serializing_if as required-but-nullable to match committed schema required[] + anyOf-null. -> clean (operational_policy.rs uses hand-rolled emitter with id_schema pattern. ledger.rs previous_hash/step_id/parent_artifact_id/receipt_id all bare Option<NonEmptyString> → match committed required[]+nullable. aster.rs Selection.decision_ref etc. same pattern. act/assignment.rs uses skip_serializing_if consistently for truly omittable fields; bare String chosen for idempotency-hash parity per source comment.)
- `crates/runx-runtime/src/agent_invocation.rs`: Look for ambient drift where runtime callers feed possibly-empty strings into NonEmptyString::from (debug_assert) or build contract structs with invariant violations. -> clean (skill_name fallback to 'skill'/'agent-step' guarantees non-empty input before From<String>; debug_assert won't fire under realistic inputs.)
- `packages/contracts/src/{index,internal}.ts`: Inspect Ajv2020 wrapper for silent schema substitution, normalization stripping that could mask shape drift, and dual-source exports. -> finding (schemaWithGeneratedArtifact silently overrides by $id match; normalizeSchemaForAjv strips nested $id/$schema. runxAuxiliarySchemas still TS-built despite same $ids in Rust manifest — filed R2.)
- `.scafld/specs/active/rust-contract-pipeline-inversion.md`: Cross-check prior review F1/F2/F3 to ensure new findings are not duplicates and that F2 (orphan detection) was actually closed. -> clean (F2 closed in --check path by new test; R1 is the residual write-mode subset, R2 continues F3 specifically for the auxiliary surface.)

Findings:
- [low/non-blocking] `R1-write-mode-orphan-bundling` Write mode of runx-contract-schemas leaves orphan .schema.json files on disk, and the TS wrapper bundles them into schema-artifacts.ts via readdirSync.
  - Location: `crates/runx-contracts/src/bin/runx-contract-schemas.rs`
  - Evidence: crates/runx-contracts/src/bin/runx-contract-schemas.rs guards orphan detection behind `if options.check { ... } else { Vec::new() }`. scripts/generate-contract-schemas.ts calls renderSchemaArtifactsSource(schemasDir) which uses readdirSync(sourceDir).filter(name => name.endsWith('.schema.json')) — any stale file from a removed/renamed contract is silently picked up locally and only fails in CI via --check.
  - Impact: Local developer iteration can produce a packages/contracts bundle that diverges from the Rust manifest until CI runs. Risk is bounded because CI --check rejects orphans.
- [low/non-blocking] `R2-auxiliary-schemas-dual-source` runxAuxiliarySchemas (registryBinding, reviewReceiptOutput) still ships TS-authored schemas even though the same $ids exist in the Rust manifest.
  - Location: `packages/contracts/src/index.ts`
  - Evidence: packages/contracts/src/index.ts exports runxAuxiliarySchemas built from registryBindingSchema and reviewReceiptOutputSchema (TS sources), while packages/contracts/src/internal.ts.schemaWithGeneratedArtifact would silently substitute the Rust artifact for any matching $id at Ajv compile time. Tests assert $id equality but not structural parity between the TS auxiliary export and the Rust artifact.
  - Impact: Two sources of truth for the same $id increase the chance of subtle drift between what TS consumers import directly and what Ajv actually compiles. Continuation of prior review's F3 for the auxiliary surface.

## Post-Completion Cleanup

Status: completed
Date: 2026-05-25

The two low, non-blocking review residuals were closed after the scafld complete
transition:

- `R1-write-mode-orphan-bundling`: `runx-contract-schemas` now removes orphan
  `*.schema.json` files in write mode, and
  `schema_generator_check.rs` covers the cleanup path.
- `R2-auxiliary-schemas-dual-source`: `runxAuxiliarySchemas` now points to the
  Rust-generated schema artifacts for `registry-binding.schema.json` and
  `review-receipt-output.schema.json`; the public index test asserts artifact
  identity and `$id` parity against the TS helper exports.
