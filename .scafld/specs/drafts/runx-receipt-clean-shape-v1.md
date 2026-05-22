---
spec_version: '2.0'
task_id: runx-receipt-clean-shape-v1
created: '2026-05-22T07:36:10Z'
updated: '2026-05-22T07:37:56Z'
status: draft
harden_status: needs_revision
size: large
risk_level: high
---

# runx-receipt-clean-shape-v1

## Current State

Status: draft
Current phase: none
Next: harden
Reason: hardening found draft contract issues
Blockers: check needs revision: path audit; check needs revision: command audit; check needs revision: scope/migration audit; check needs revision: acceptance timing audit; check needs revision: rollback/repair audit; check needs revision: design challenge; 5 approval-blocking issue(s)
Allowed follow-up command: `edit the draft, then run scafld harden runx-receipt-clean-shape-v1 --provider <provider>`
Latest runner update: none
Review gate: not_started

## Summary

Replace the `runx.harness_receipt.v1` contract with a single, clean
`runx.receipt.v1` shape and cut the entire Rust+TS+cloud surface over to it in one
coordinated move. This is a **clean cutover with zero back-compat**: no
`harness_receipt` aliases, no dual-shape support, no compat shims, no old-shape
glue. The old schema, Rust types, and fixtures are deleted, not kept alongside.

The current shape conflates three artifacts into one: durable proof, planner
deliberation, and per-act detail. The receipt nests a `harness` sub-object that
forces a mandatory duplicated `seal` (`verify.rs` checks `seal == harness.seal`),
inlines planner deliberation (`decisions[]`) that no consumer reads, and restates
the skill's declared contract in every act (`intent.success_criteria`) instead of
recording result bindings. The clean shape flattens the `harness` wrapper (which
removes the seal duplication by construction), references deliberation and per-act
detail instead of inlining them, and keeps the load-bearing proof inspectable.

## Context

Grounded in the committed schemas, a generated runtime receipt, and a consumer
sweep (2026-05-22):

- The contract is `runx.harness_receipt.v1` (`schemas/harness-receipt.schema.json`,
  ~14k inlined lines). The runtime emits it; cloud and the ledger read parts of it.
- `crates/runx-receipts/src/canonical.rs` commits a body digest over everything
  except `signature` and each seal's `digest`/`verification_summary`. So
  `decisions[]`, `acts[]`, `authority`, `enforcement`, `idempotency` are all in the
  signed body.
- `crates/runx-receipts/src/verify.rs` enforces `receipt.seal == receipt.harness.seal`
  (the duplication is mandatory only because of the nested `harness` object),
  validates `decisions[].selected_act_id` references a real act, validates act form
  consistency, binds seal criteria to `acts[].criterion_bindings`, and validates the
  authority attenuation against `authority-subset-proof`.
- Consumer sweep: cloud reads only `idempotency.{intent_key,trigger_fingerprint,content_hash}`
  (dedup) and top-level `seal.{disposition,closed_at}` (status). `decisions[]`
  deliberation content is read by nothing; it is only hashed plus the act-id
  integrity check.
- The payment authority bound (the magnet) lives in
  `authority.terms[].bounds.payment` (`currency`, `max_per_call_minor`,
  `max_per_run_minor`, `max_per_period_minor`, `single_use_spend`,
  `reservation_required`, `receipt_before_success`, `idempotency_required`). It is
  the provable bound and must stay fully inspectable, never collapsed to a hash.
- Composition targets already exist as separate schemas: `act-receipt`,
  `review-receipt-output`, `signal`, `authority-subset-proof`.

## The runx.receipt.v1 shape

Flat top-level keys, each answering one question; everything optional-when-empty:

- envelope: `schema`, `id`, `created_at`, `canonicalization`, `issuer`,
  `signature`, `digest`.
- `idempotency`: `{intent_key, trigger_fingerprint, content_hash}` (cloud dedup +
  payment safety; top-level).
- `subject`: `{kind: skill|graph, ref, commitments[]}` where each commitment is
  `{algorithm, value, canonicalization}` (input/output/stdout/stderr hashes; unifies
  the old `hash_commitments` + `enforcement.std*_hash`).
- `authority`: `actor_ref`, `grant_refs`, `scope_refs`, `attenuation`,
  `authority_proof_refs`, `mandate_ref`, `terms[]` (incl. `bounds.payment`),
  `enforcement: {profile_hash, redaction_refs}`. Fully inspectable.
- `acts[]`: `{id, form, summary, criteria[]: {criterion_id, status, evidence_refs,
  verification_refs}, detail_ref}`. `detail_ref` points to an `act-receipt` for the
  full intent/target/source/surface refs. The restated `intent` declaration is gone.
- `seal`: `{disposition, reason_code, summary, closed_at, criteria}`. One seal, no
  `harness.seal` twin.
- `lineage` (optional): `{parent, children[], sync[], setup_refs[], teardown_refs[],
  signal_refs[], journal_ref}`. `kind: graph` + `lineage` makes skill and graph one
  model. `journal_ref` commits the planner deliberation (former `decisions[]`) by
  reference instead of inlining it.

## Objectives

- Define `runx.receipt.v1` as the single receipt contract, flat (no `harness`
  wrapper), one seal, idempotency top-level, authority fully inspectable.
- Move planner deliberation (former `decisions[]`) into a referenced journal
  artifact committed by `journal_ref` hash; preserve the act-id integrity property
  through journal verification.
- Reference per-act detail via `act-receipt` (`detail_ref`) instead of inlining
  intent/target/source/surface refs.
- Cut the Rust contracts, the receipts kernel (canonical/verify/signing), the
  runtime emitter, and cloud reads over to `runx.receipt.v1` in one move.
- Delete the old shape entirely: `harness-receipt.schema.json`, the `HarnessReceipt`
  Rust type and `harness`/`harness.seal` nesting, and all `runx.harness_receipt.v1`
  fixtures and references. No alias, no dual-shape, no compat code remains.

## Scope

In scope:
- `crates/runx-contracts/src/*` receipt/act/authority/seal/idempotency types.
- `crates/runx-receipts/src/{canonical,verify,signing,tree}.rs` and the digest body.
- `crates/runx-runtime/src/receipts/seal.rs` and any emitter writing receipts,
  plus writing the planner journal artifact + `journal_ref`.
- The receipt schema(s) in the contract source and `schemas/*.json` regeneration.
- Cloud reads in `cloud/packages/{db,api,worker}` that touch `idempotency` and
  `seal` (confirm they still resolve against the flat shape).
- Fixtures under `fixtures/contracts/harness-spine/*` and the canonical-json oracle.

Out of scope (and explicitly forbidden):
- Any back-compat: `harness_receipt` aliases, dual-shape readers, compat shims, or
  old-shape glue code. None may remain after this task.
- The contract-pipeline source-of-truth flip (owned by
  `rust-contract-pipeline-inversion`); author the new shape in the current source.
- Redesigning `act-receipt`, `signal`, or `authority` semantics beyond referencing
  them; the spend-capability bounds shape is preserved as-is, only relocated under
  the flat `authority`.

## Dependencies

- `canonical-json-fingerprint-contract-v1` (the canonicalization byte contract; the
  new `runx.receipt.c14n.v1` rides on it).
- The act-model reconciliation: this spec settles the durable receipt shape; the
  schema and the runtime emitter currently disagree on `authority`/`enforcement`,
  and this cutover makes them agree on `runx.receipt.v1`.

## Touchpoints

- `crates/runx-contracts/src/` (receipt, act, authority, seal, idempotency)
- `crates/runx-receipts/src/{canonical,verify,signing,tree}.rs`
- `crates/runx-runtime/src/receipts/seal.rs`
- `schemas/harness-receipt.schema.json` (deleted), new `schemas/receipt.schema.json`
- `cloud/packages/db/src/*harness-receipt*`, `cloud/packages/api`, `cloud/packages/worker`
- `fixtures/contracts/harness-spine/*`, `fixtures/contracts/canonical-json/*`

## Risks

- Digest change: flattening, the single seal, and `decisions[]` -> `journal_ref`
  change what the body commits. This is a deliberate version break
  (`runx.receipt.v1` / `runx.receipt.c14n.v1`), not an in-place mutation. All
  fixtures and the oracle regenerate together.
- Coordinated cutover with no bridge: because there is no compat layer, the emitter,
  verifier, cloud, and fixtures must all land in the same change or receipts fail to
  round-trip. Sequence the build so the kernel and emitter flip together before the
  no-compat sweep asserts.
- Lost proof by over-collapsing: `idempotency`, `authority.terms.bounds.payment`,
  attenuation/proof refs, and the hash commitments are durable proof. The shape keeps
  them; acceptance asserts they survive.
- Journal integrity: moving deliberation out must not weaken the `selected_act_id`
  integrity guarantee; the journal is committed by `journal_ref` and verified.

## Acceptance

Profile: strict

Definition of done:
- [ ] `dod1` `runx.receipt.v1` is the only receipt contract: flat top level (no
  `harness` wrapper), exactly one `seal`, `idempotency` top-level, `subject` with
  `kind: skill|graph` and `commitments[]`, `authority` carrying
  `terms[].bounds.payment` inspectably, `acts[]` with `criteria` bindings and
  `detail_ref`, optional `lineage` with `journal_ref`.
- [ ] `dod2` No back-compat remains: a repo scan finds no `harness_receipt`,
  `harness-receipt.schema.json`, `HarnessReceipt` type, `harness.seal`, or
  `runx.harness_receipt.v1` reference in `crates`, `cloud`, `schemas`, or
  `fixtures`. No alias or dual-shape reader exists.
- [ ] `dod3` The receipts kernel operates on the flat shape: the body digest commits
  `idempotency`/`subject`/`authority`/`acts`/`seal`/`lineage`; `verify.rs` no longer
  has a seal-equality check; the `selected_act_id` integrity property is verified via
  the journal.
- [ ] `dod4` The runtime emits `runx.receipt.v1`, writes the planner journal as a
  separate artifact, and records `lineage.journal_ref`. No seal is written twice.
- [ ] `dod5` Cloud dedup (`idempotency.*`) and status (`seal.disposition`,
  `seal.closed_at`) resolve against the flat shape with no field-path changes beyond
  the wrapper removal.
- [ ] `dod6` Harness-spine fixtures are regenerated under `runx.receipt.v1`, the
  canonical-json oracle covers `runx.receipt.c14n.v1`, and old
  `runx.harness_receipt.v1` fixtures are deleted.
- [ ] `dod7` The payment authority bound survives intact: a spend receipt still
  carries `authority.terms[].bounds.payment` with the cap/rail/single_use_spend
  fields, inspectable (not hashed away), and `idempotency` is present.

Validation:
- [ ] `v1` Scafld validates this spec.
  - Command: `scafld validate runx-receipt-clean-shape-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v2` Receipts kernel tests pass on the flat shape.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v3` Runtime receipt emission + sealing tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime receipt`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v4` No-compat sweep: zero old-shape references remain.
  - Command: `! rg -n 'harness_receipt|HarnessReceipt|harness-receipt|harness\.seal' crates cloud schemas fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v5` Canonical-json oracle covers the new receipt c14n.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-receipts canonical`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v6` Cloud receipt dedup/status tests pass against the flat shape.
  - Command: `pnpm --filter @runx/cloud-db test`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 1: Contract (ground level)

Status: pending
Dependencies: none

Define `runx.receipt.v1` in the contract source and emit `schemas/receipt.schema.json`
for the flat shape. Pin `runx.receipt.c14n.v1`. Delete `harness-receipt.schema.json`.
No Rust/runtime changes yet; this phase fixes the contract the rest builds against.

## Phase 2: Rust contract types

Status: pending
Dependencies: Phase 1

Replace the `HarnessReceipt` types in `runx-contracts` with the flat `Receipt`,
`Subject`, rich `Authority`, lean `Act` (+`detail_ref`), one `Seal`, and `Lineage`
(+`journal_ref`). Use `NonEmptyString` for ids/refs. Delete the old types; no alias.

## Phase 3: Receipts kernel

Status: pending
Dependencies: Phase 2

Rewrite `canonical.rs` (body excludes signature+digest; commits the flat fields),
`verify.rs` (drop the seal-equality check; move `selected_act_id` integrity to
journal verification; keep seal<->act criteria binding and attenuation checks), and
`signing.rs`/`tree.rs` for the flat shape.

## Phase 4: Runtime emitter + journal

Status: pending
Dependencies: Phase 3

Emit `runx.receipt.v1` from `seal.rs`; write the planner deliberation as a separate
journal artifact and record `lineage.journal_ref`; stop double-writing the seal.

## Phase 5: Cloud + fixtures + no-compat sweep

Status: pending
Dependencies: Phase 4

Confirm cloud dedup/status reads resolve against the flat shape (field paths only
lose the `harness.` prefix). Regenerate harness-spine fixtures under
`runx.receipt.v1`, extend the canonical-json oracle, delete old fixtures. Run the
no-compat sweep (`v4`) and fix any remaining old-shape reference until it is empty.

## Rollback

The cutover is a version break with no compat layer, so rollback is `git revert` of
the coordinated change, not a feature flag. There is no half-state to leave: either
`runx.receipt.v1` is the contract end-to-end, or the change is reverted whole.

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
Conversation 2026-05-22: receipt shape reconsidered from the ground up. The
`harness_receipt` shape conflates proof, deliberation, and per-act detail; the user
called for the ultimate clean shape and a clean cutover with zero compat. Shape
verified against the schemas, a generated receipt, and a consumer sweep before
spec'ing.

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-22T07:37:56Z
Ended: 2026-05-22T07:37:56Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Draft has the right diagnosis (collapsed harness/seal duplication, deliberation inlining, restated success_criteria) and a coherent target shape, but four blockers prevent approval: (1) the logical schema name `runx.receipt.v1` and the filename `schemas/receipt.schema.json` are already taken by the CLI run-summary receipt in `packages/contracts/src/schemas/receipt.ts` and `RUNX_LOGICAL_SCHEMAS.receipt`, so a no-compat cutover collides with an existing in-product contract; (2) the cutover crosses the documented `oss/` ↔ `cloud/` repo boundary (CLAUDE.md), yet acceptance/sweep commands and touchpoints assume a single working directory — `cloud/` is not a sibling of `crates/` from `oss/` and the `pnpm --filter @runx/cloud-db test` filter does not match any package (the package is `@runx/db`); (3) Phase 3 names a kernel file that does not exist (`crates/runx-receipts/src/signing.rs`); signing actually lives in `crates/runx-runtime/src/receipts/signing.rs`; (4) the no-compat assertion underestimates blast radius — 1,416 matches across 250 OSS files plus durable `ReferenceType::HarnessReceipt` / `HARNESS_RECEIPT_REF_PREFIX = "runx:harness_receipt:"` URIs in stored receipts that the spec doesn't address. Journal-ref verification semantics (load path, missing-journal failure mode, cross-trust-boundary reads) also need to be pinned before approval.

Checks:
- path audit
  - Grounded in: code:schemas/receipt.schema.json:1
  - Result: failed
  - Evidence: schemas/receipt.schema.json already exists and binds the logical name runx.receipt.v1 to the CLI run-summary shape ({run_id, command, status, started_at, root, steps}); packages/contracts/src/internal.ts:62 declares receipt: 'runx.receipt.v1' and packages/contracts/src/index.ts:698 maps 'receipt.schema.json' to receiptV1Schema. The spec's claim that it can write the new shape at this filename/name with no compat layer is unsupported. Also, Phase 3 names crates/runx-receipts/src/signing.rs which does not exist; signing lives at crates/runx-runtime/src/receipts/signing.rs (verified via Glob).
- command audit
  - Grounded in: code:cloud/packages/db/package.json:2
  - Result: failed
  - Evidence: Acceptance v6 runs `pnpm --filter @runx/cloud-db test`, but cloud/packages/db/package.json declares the package name as '@runx/db' (line 2). The filter matches zero packages and exits 0 silently, falsely satisfying the gate. Acceptance v4 (`! rg -n '...' crates cloud schemas fixtures`) is also unsafe from the OSS working directory: `cloud` is not a sibling of `crates` inside oss/ (cloud lives at /Users/kam/dev/runx/runx/cloud, sibling of oss/), so rg returns a non-zero exit for the missing path and the negation flips it to success — also a false pass.
- scope/migration audit
  - Grounded in: code:CLAUDE.md
  - Result: failed
  - Evidence: CLAUDE.md explicitly says `oss/` is for user-facing/product-agnostic code and `cloud/` is for hosted operations and 'do **not** belong in `oss/`'. The spec's touchpoints list cloud/packages/{db,api,worker} and fixtures/contracts/harness-spine/* and bundles them into one coordinated cutover. The 1,416 harness_receipt occurrences across 250 oss/ files (rg count) plus durable references — ReferenceType::HarnessReceipt (24 usages) and HARNESS_RECEIPT_REF_PREFIX='runx:harness_receipt:' at crates/runx-runtime/src/journal.rs:27 — extend the no-compat sweep into stored on-disk receipt URIs that the spec never enumerates.
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: failed
  - Evidence: v4 sweep depends on cross-repo presence of `cloud`; v6 names a non-existent pnpm filter (see command audit). v2 and v3 are reasonable but cannot pass until Phase 4 lands because runtime emitters in seal.rs construct HarnessReceipt today (crates/runx-runtime/src/receipts/seal.rs:115,228). dod3 says 'selected_act_id integrity is verified via the journal' but no acceptance command exercises that: verify.rs:194-207 (check_decisions) currently runs in-memory against harness.decisions; once decisions move out of the body, the spec doesn't say where verify reads the journal from, what happens if the journal is absent, or which test fixture proves the property.
- rollback/repair audit
  - Grounded in: spec_gap:Rollback
  - Result: failed
  - Evidence: Rollback is declared as `git revert` of 'the coordinated change'. With 1,416 OSS hits + cloud changes across two repositories (oss/ and cloud/, with cloud not in the OSS workspace), the 'one coordinated change' is implausible as a single commit. CLAUDE.md requires the OSS-published CLI not to depend on cloud, yet the cutover requires cloud reads to flip simultaneously, which crosses release boundaries. Rollback also doesn't address durable on-disk receipts under the local store (runx:harness_receipt: URIs) issued before the cutover — git revert restores code but not stored artifacts.
- design challenge
  - Grounded in: code:packages/contracts/src/internal.ts:62
  - Result: failed
  - Evidence: Architecturally the diagnosis is right: verify.rs:85-93 enforces seal==harness.seal solely because of the nested harness wrapper, and decisions[] is content-hashed but only used for the in-memory check_decisions act-id check (verify.rs:194-207). Flattening + journal_ref is sound. But the chosen schema name (runx.receipt.v1) is the third receipt named 'receipt' in this codebase (CLI run-summary, harness receipt, local-receipt schema_version='runx.receipt.v1' in packages/contracts/dist/src/schemas/local-receipt.d.ts:3). Without a distinct logical name (e.g., runx.governed_receipt.v1 or runx.harness_receipt.v2 with the harness wrapper removed), the cutover trades one ambiguity for another and breaks the existing CLI run receipt. This is fixable with a rename and a clear scope split: do the Rust kernel/runtime cutover in OSS first, treat cloud and durable-URI rename as separate, sequenced specs.

Issues:
- [critical/blocks approval] `harden-1` schema name collision - Proposed `runx.receipt.v1` / `schemas/receipt.schema.json` already names the CLI run-summary receipt.
  - Status: open
  - Grounded in: code:packages/contracts/src/internal.ts:62
  - Evidence: packages/contracts/src/internal.ts:62 (`receipt: 'runx.receipt.v1'`), packages/contracts/src/schemas/receipt.ts:14-30 (run_id/command/status/started_at/root/steps shape), packages/contracts/src/index.ts:698 (`'receipt.schema.json': receiptV1Schema`), and packages/contracts/dist/src/schemas/local-receipt.d.ts:3 (`schema_version: 'runx.receipt.v1'`) all bind that name to a different artifact. Spec dod1/Phase 1 mandate the new flat harness shape inherits exactly this name and filename with no compat layer.
  - Recommendation: Pick a distinct logical name (e.g., `runx.governed_receipt.v1`, `runx.execution_receipt.v1`, or `runx.harness_receipt.v2` with the wrapper removed) and a distinct filename. If the team really wants `runx.receipt.v1` to belong to the new shape, treat the CLI run-summary rename as an explicit prerequisite spec — not as silent collateral damage of this one.
  - Question: Which logical schema name and filename will the new flat shape take, given that `runx.receipt.v1` / `schemas/receipt.schema.json` already belong to the CLI run-summary?
  - Recommended answer: Adopt `runx.harness_receipt.v2` (same artifact, new shape) at `schemas/harness-receipt.schema.json` — preserves the CLI receipt and keeps cross-references searchable; the 'no harness_receipt' clause in dod2 then targets `v1` specifically.
  - If unanswered: Default to `runx.harness_receipt.v2` at the existing path and rewrite dod2 to forbid only the v1 logical name and HarnessReceipt Rust type.
- [critical/blocks approval] `harden-2` scope boundary - Cutover crosses the documented `oss/` / `cloud/` boundary and uses commands that cannot run from the OSS working directory.
  - Status: open
  - Grounded in: code:CLAUDE.md
  - Evidence: CLAUDE.md ('oss/ is for user-facing capabilities; cloud/ is for hosted operations; admin/control-plane features do not belong in oss/'). Spec touchpoints include cloud/packages/{db,api,worker}; acceptance v4 references the path `cloud` from oss/ (cloud is /Users/kam/dev/runx/runx/cloud, sibling of oss/); v6 uses `pnpm --filter @runx/cloud-db test` while cloud/packages/db/package.json:2 declares `@runx/db`.
  - Recommendation: Split into two specs: (a) OSS cutover (contracts/kernel/runtime/fixtures/oracle), with sweep paths restricted to crates schemas fixtures and tests inside oss/; (b) cloud cutover (cloud/packages/{db,api,worker}, the harness-receipts.ts dedupe path, the check-harness-data-cutover.mjs script) tracked as a coordinated downstream change. Document the temporal contract between them (versioned schema, atomic rollout, read-side tolerance window).
  - Question: Are cloud changes in-scope for this single spec, or do we split OSS and cloud into sequenced specs sharing the same version identifier?
  - Recommended answer: Split. Land the OSS cutover first under a new logical name, then cut cloud over in a sibling spec; co-version them so the cloud spec can assert against the same schema artifact.
  - If unanswered: Default to OSS-only here and open a follow-up `runx-receipt-cloud-cutover-v1` covering cloud/packages/{db,api,worker}, harness-routes.ts, and check-harness-data-cutover.mjs.
- [critical/blocks approval] `harden-3` acceptance command is a silent no-op - `pnpm --filter @runx/cloud-db test` matches no package and exits 0; the no-compat sweep on `cloud` errors then negates to success.
  - Status: open
  - Grounded in: code:cloud/packages/db/package.json:2
  - Evidence: cloud/packages/db/package.json:2 declares `"name": "@runx/db"`. v4's `! rg -n '...' crates cloud schemas fixtures` runs from oss/, where `cloud/` doesn't exist; rg exits non-zero for a missing path and `!` flips it to 0, falsely satisfying the gate.
  - Recommendation: Rewrite v4 to `! rg -n 'harness_receipt|HarnessReceipt|harness-receipt|harness\.seal' crates schemas fixtures packages` (paths that exist inside oss/, including packages/ for TS), and either drop v6 or, if the cloud cutover is kept in scope, change it to `pnpm --filter @runx/db test` and run it from `cloud/` (not `oss/`).
  - Question: What is the exact command set that will be run from `oss/` to prove no-compat, and how is the cloud side proven separately?
  - Recommended answer: OSS sweep: `! rg -n 'harness_receipt|HarnessReceipt|harness-receipt|harness\.seal' crates schemas fixtures packages tests` from oss/; cloud sweep + tests live in the cloud follow-up.
  - If unanswered: Default to the OSS-only sweep above and remove v6 from this spec.
- [high/blocks approval] `harden-4` phase touchpoint wrong - Phase 3 names `crates/runx-receipts/src/signing.rs` which does not exist; signing lives in the runtime.
  - Status: open
  - Grounded in: code:crates/runx-receipts/src
  - Evidence: Glob of crates/runx-receipts/src/*.rs returns only canonical.rs, lib.rs, tree.rs, verify.rs. Signing is at crates/runx-runtime/src/receipts/signing.rs.
  - Recommendation: Drop `signing.rs` from Phase 3 and either (a) add it to Phase 4 if the runtime signing path changes shape, or (b) leave runtime signing alone if the body-digest contract is the only change. Update touchpoints accordingly.
  - Question: Does the signing flow change beyond consuming the new body digest, and if so does the kernel grow a `signing.rs` or does Phase 4 own that change in the runtime?
  - Recommended answer: Phase 4 owns runtime signing edits; the kernel keeps canonical.rs/verify.rs/tree.rs only.
  - If unanswered: Default to Phase 4 owning runtime signing; remove the file from Phase 3.
- [high/blocks approval] `harden-5` missing migration of durable identifiers - Durable reference identifiers (`ReferenceType::HarnessReceipt`, `runx:harness_receipt:` URI prefix) are embedded in stored receipts and never enumerated.
  - Status: open
  - Grounded in: code:crates/runx-runtime/src/journal.rs:27
  - Evidence: crates/runx-runtime/src/journal.rs:27 (`HARNESS_RECEIPT_REF_PREFIX = 'runx:harness_receipt:'`); ReferenceType::HarnessReceipt has 24 usages across 11 files including verify.rs:1, tree.rs:8, post_merge_observer plan/runtime, target_runner.rs, receipts/seal.rs, plus tests. dod2 forbids any `harness_receipt` token in `crates`/`schemas`/`fixtures` after cutover.
  - Recommendation: Either keep the wire-level reference scheme stable (preserve `runx:harness_receipt:` and `ReferenceType::HarnessReceipt` as the lineage identifier even if the receipt's logical name changes) and narrow dod2 accordingly, or add an explicit migration step for local-store URIs and a rename of the enum variant + prefix, with a fixture that demonstrates pre-cutover stored receipts still verify (or are explicitly invalidated).
  - Question: Do we keep `ReferenceType::HarnessReceipt`/`runx:harness_receipt:` as the durable receipt-reference scheme (decoupled from the schema rename), or rename them and rebuild local stores?
  - Recommended answer: Keep the durable scheme stable; narrow dod2 to forbid `runx.harness_receipt.v1` (logical name) and `HarnessReceipt`-the-Rust-type for the old shape. Reference URIs and the lineage enum variant stay.
  - If unanswered: Default to keeping the durable reference scheme; document explicitly that the receipt schema rename does not touch lineage URIs.
- [high/advisory] `harden-6` journal verification semantics - Moving decisions[] to `journal_ref` removes an in-memory integrity check; the verifier's new contract is undefined.
  - Status: open
  - Grounded in: code:crates/runx-receipts/src/verify.rs:194
  - Evidence: verify.rs:194-207 walks `harness.decisions` against in-memory `acts[]`. Once decisions are committed only as a hash, verify needs the journal artifact to recompute the act-id check. The spec asserts the property is preserved 'through journal verification' without defining where the journal is loaded from, what happens when it is missing, or whether cloud read-side consumers must access it.
  - Recommendation: Add a paragraph under 'The runx.receipt.v1 shape' (or Phase 3) defining: (1) verifier interface for journal access (caller-provided loader vs. discovery via `journal_ref`); (2) verdict when the journal is unavailable (`unknown`/`partial` vs. `invalid`); (3) which consumers verify the journal vs. which just check the hash; (4) a fixture pair (receipt + journal) proving the property.
  - Question: What is the verifier's contract when `journal_ref` is present but the journal artifact is not provided?
  - Recommended answer: Verify treats journal-dependent checks as `unverified` (not `invalid`); receipt-shape and seal-binding checks still run; a dedicated `verify_with_journal(receipt, journal)` API computes the act-id integrity check and is the only path that produces a fully verified verdict.
  - If unanswered: Default to the recommended two-tier verifier and add a fixture under fixtures/contracts/harness-spine/journaled-acts.json plus a kernel test that exercises both paths.
- [medium/advisory] `harden-7` rollback credibility - Single `git revert` rollback is implausible given the cross-repo scope and durable on-disk artifacts.
  - Status: open
  - Grounded in: spec_gap:Rollback
  - Evidence: 1,416 OSS hits + cloud changes spanning two repos (oss/, cloud/); on-disk receipts written before the cutover carry `runx:harness_receipt:` URIs and the old body digest.
  - Recommendation: Tighten the rollback section to: (a) what's revertible by `git revert` (the OSS coordinated change), (b) what is not (durable stored receipts under the local receipt store and any cloud-side dedupe rows), (c) the operational stance for those — quarantine pre-cutover receipts and require re-emission, or keep them readable behind a tagged legacy verifier? This is a real choice the spec needs to make.
  - Question: How are pre-cutover stored receipts (local store + cloud dedupe rows) treated after rollback or after a forward roll?
  - Recommended answer: Forward: re-emit on next sealing; cloud-side keeps reads working by inspecting the schema version. Rollback: pre-cutover receipts already match the reverted code; only receipts emitted during the new-shape window need re-emission and are flagged in the journal.
  - If unanswered: Default to the above and add a paragraph to Rollback describing the durable-artifact stance.


## Planning Log

- none
