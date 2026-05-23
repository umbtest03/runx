---
spec_version: '2.0'
task_id: runx-receipt-aplus-shape-v1
created: '2026-05-23T00:00:00Z'
updated: '2026-05-23T13:02:32Z'
status: completed
harden_status: needs_revision
size: large
risk_level: high
---

# runx receipt — A+ shape (the correct, shippable receipt)

> **BUILD AUTHORIZED 2026-05-23.** The operator approved this A+ shape for build (one
> full hardening round first, Gemini review as the final gate). It supersedes the
> in-flight `runx-receipt-clean-shape-v1` direction (the discredited lean/journal
> "butchered" shape). The S-tier and enlightened drafts remain build-gated.

## Tier

**A+** = the correct, fully-coherent receipt: one self-contained signed artifact
that serves every consumer, with no bug and no self-grading. It is the *buildable*
target. The S-tier (`runx-receipt-claim-graph-s-tier-v1`) and enlightened
(`runx-receipt-enlightened-north-star-v1`) specs sit above it and are explicitly
not build targets.

## The meaning of this tier — fidelity

A+ is the tier of **honest completeness**. Its whole meaning is that the receipt tells
the *whole truth about one governed run, to every consumer, once, without grading
itself*. It is where the receipt earns the word "receipt": a faithful record you can
act on, train on, bill on, and audit, with no consumer getting a degraded slice and no
field that flatters its own correctness.

The hard-won discipline encoded here is that **"clean" means "serves every consumer
faithfully," not "minimal."** The failure that produced this draft was treating
cleanliness as subtraction — stripping the reasoning out of the receipt until it was
small and useless. A+ rejects that: it removes only *duplication, incoherence, and
self-grading*, and keeps every load-bearing fact inline. Fidelity over elegance.
Nothing here is clever; everything here is true. That is the point of the tier.

## What a receipt is

A **Receipt is a signed, sealed run**: one governed reasoning episode recorded once
and signed at completion. `Receipt = signed(envelope, run)`. The envelope attests;
the `run` is the content. Everything semantic stays inline; only bulky execution
I/O is referenced and hydrated by projections. There is no journal and no per-act
exile — the lesson that produced this draft is that pushing the reasoning out of the
receipt destroyed its training and inspection value.

## The shape

The receipt is **flat**. There is no nested `run` object — "run" is the *conceptual*
name for the signed content (everything the digest covers), not a serialized field.
This matches the existing kernel, which digests the flat receipt body, and avoids
re-introducing a wrapper (the very smell we removed with `harness`).

```
runx.receipt.v1   (flat; "run" = the signed body = all fields except signature/digest/metadata)
  schema            "runx.receipt.v1"
  id                string                 # content-addressed: id = hash(canonical_body) under runx.receipt.c14n.v1
  created_at        date-time
  canonicalization  "runx.receipt.c14n.v1"
  issuer            { type: local|hosted|ci|verifier, kid, public_key_sha256 }
  signature         { alg: Ed25519, value }
  digest            sha256 over canonical_body = the receipt minus { signature, digest, metadata }
  idempotency       { intent_key, trigger_fingerprint, content_hash }   # cloud dedup + payment safety
  subject           { kind: skill|graph, ref, input_context?{source,preview,value_hash},
                      commitments[]{scope: input|output|stdout|stderr|error, algorithm, value, canonicalization} }
  authority         { actor_ref, grant_refs[], scope_refs[], authority_proof_refs[],
                      attenuation{parent_authority_ref?, subset_proof?}, mandate_ref?,
                      terms[]<full RBAC: principal/resource/family/verbs/capabilities/conditions/approvals/bounds(incl bounds.payment), expires_at, issued_by_ref, credential_ref>,
                      enforcement{profile_hash, redaction_refs[], setup_refs[], teardown_refs[]} }
  signals[]         <runx:signal: references — authenticity/trust/body live in the signal artifact>
  decisions[]       { decision_id, choice: open|continue|spawn_child|escalate|defer|close|decline|monitor,
                      inputs, proposed_intent, justification{summary, evidence_refs[]}, selected_act_id?, closure? }   # GOVERNANCE reasoning, inline, small
  acts[]            { id, form: observation|reply|review|verification|revision,
                      intent{purpose, legitimacy, success_criteria[{criterion_id, statement, required}]},
                      summary,
                      criterion_bindings[{criterion_id, status: verified|failed|pending|not_applicable|unknown, evidence_refs[], verification_refs[], summary?}],
                      by?{provider, model, prompt_version},          # runner provenance (agent acts)
                      source_refs[], target_refs[], artifact_refs[],
                      context_ref?,                                  # -> agent-context envelope / transcript (bulky AGENT reasoning + I/O), hydrated by projections
                      closure, revision?, verification? }
  seal              { disposition: closed|deferred|superseded|declined|blocked|failed|killed|timed_out,
                      reason_code, summary, closed_at, last_observed_at, criteria[]<rollup of act bindings> }
  lineage?          { parent?, previous?, children[], sync[]<fanout: group/strategy/decision/counts>, resume_ref? }
  metadata?         {}                          # read-aid, EXCLUDED from canonical_body / digest
```

### Three distinctions this shape gets right (and earlier drafts blurred)

1. **Governance decisions vs agent reasoning.** `decisions[]` is the *governance* why
   (admit / escalate / defer) — small, structural, inline. The *agent's* reasoning
   (chain-of-thought, tool calls, the rich transcript) is the bulky I/O behind
   `acts[].context_ref`, hydrated by the trainable/inspection projections. Both are
   preserved; they are different things and must not be conflated.

2. **The outcome verdict is an act, not a side contract.** The `outcome_resolution`
   peer contract was deliberately retired (`runx-retired-outcome-contract-sunset`,
   2026-05-20): post-run judgment is represented as a `review`/`verification` **act**,
   not a separate `outcome_resolution` packet. So A+ does NOT add a `resolution`
   artifact. A verdict known at seal time is a `review`/`verification` act in `acts[]`;
   a later verdict is a follow-up receipt carrying a `review` act with
   `lineage.previous`/`parent` pointing at the original. The supervised training label
   is therefore the criterion outcomes + `seal.disposition` + any review/verification
   act, all already in receipts.

3. **The receipt is the only signed artifact for the run.** It is content-addressed
   (`id = hash(canonical_body)`), so references to it are self-verifying. The transcript
   / agent-context behind `acts[].context_ref` is the only thing hydrated in; it is
   referenced, never embedded in the signed body.

## Verification is computed, never stored

The receipt carries facts + a signature, never a verdict. A verifier recomputes the
digest under `runx.receipt.c14n.v1`, checks the signature against
`issuer.public_key_sha256`, binds `seal.criteria` to `acts[].criterion_bindings`,
checks each `decision.selected_act_id` against the inline `acts[]` (no journal), and
validates `authority.attenuation` against `authority_proof_refs`. It RETURNS a
`ReceiptVerificationSummary`; that struct is never a field of the signed receipt. This
removes the old self-graded `seal.verification_summary`.

## Projections are a real, named layer

A single `projections` module (Rust + TS), pure functions over a sealed receipt
(joining the transcript and any linked review/verification receipt where needed),
never bespoke field-picking:

- `verify(receipt) -> ReceiptVerificationSummary`
- `trainable(receipt, transcript?) -> TrainableRow` — embeds the rich receipt and
  hydrates `context_ref`/`artifact_refs` into a complete training example (input ->
  governance decisions -> acts/intent/criteria -> outcome via `seal` + any
  review/verification act), computing verification on read.
- `history(receipt)`, `metering(receipt) -> {idempotency, disposition, closed_at}`,
  `payment_ledger(receipts)` (fold over spend acts + `terms.bounds.payment`),
  `lineage_graph(receipts)`.

## Use-case fit (must hold)

Verification, LLM training (trajectory + reward + negatives + provenance + integrity),
human inspection, cloud dedup/metering, payment ledger, suspended (`deferred` +
`resume_ref`), graph/fanout (`lineage.children/sync`), resume (`lineage.previous`),
delegation (`authority.attenuation`). Each is served from the one receipt plus the
joined sibling artifacts; none requires a bespoke slice.

## How we ensure the contracts do not diverge (the harness)

Every painful bug in the lead-up was contract divergence (Rust type vs TS `spine.ts`
vs published JSON Schema vs the runtime emitter). A+ ships the **detectors** that catch
divergence; it does NOT undertake the full source inversion:

1. **Cross-binding conformance oracle** — canonical example instances that the Rust
   emitter produces, the TS validator accepts, and the JSON Schema validates, with
   byte-identical canonicalization. This catches schema-vs-runtime divergence the
   instant it appears, even while Rust and TS remain hand-maintained copies.
2. **Emitter-validates-against-schema test** — what the runtime actually emits must
   validate against the published schema. (The single biggest miss historically.)
3. **Wired into the existing `contracts:schemas:check` gate** so drift fails CI.
4. **Deferred (NOT A+):** the single-source flip (one definition generates Rust + TS +
   JSON Schema) is owned by `rust-contract-pipeline-inversion`. A+ must not add a *new*
   hand-maintained copy, but it does not implement generation. (This removes the earlier
   self-contradiction between "generate all three" and "do not add a copy".)

## Explicitly NOT in A+ (belongs to higher tiers)

- The unified signed-claim envelope across receipt/signal/grant/authority-proof and a
  content-addressed store — see `runx-receipt-claim-graph-s-tier-v1`. (Only the one
  cheap move `id = hash(canonical_body)` is adopted in A+, as a Phase-1 item; the full
  claim-graph and the shared envelope are not.)
- The single-source contract generation flip — `rust-contract-pipeline-inversion`.
- Self-describing contracts, execution-is-proof, the recursive single type, the
  closed run->train loop — see `runx-receipt-enlightened-north-star-v1`.

## Tradeoffs / risks

- Hard cutover, version break (`runx.receipt.v1` / `runx.receipt.c14n.v1`); all
  fixtures + oracle regenerate together; partial application leaves the tree
  non-compiling by design.
- The outcome verdict as a `review`/`verification` act (not a side contract) means a
  *later* verdict is a follow-up receipt linked by `lineage`; the trainable projection
  must join it. Acceptance asserts the label is reachable.
- Bulky agent I/O behind `context_ref` means training completeness depends on the
  projection hydrating it AND upstream capture retaining it — acceptance must assert
  a rich row.

## Scope

In scope (OSS, from `oss/`): `crates/runx-contracts/src/*` (the flat `Receipt` type,
content-addressed id); `crates/runx-receipts/src/{canonical,verify,tree}.rs`
(digest over the canonical body, computed verification, no journal); `crates/runx-runtime/src/receipts/*`,
the emitter, `payment_ledger.rs`; the `projections` layer (verification, trainable,
history, metering); `packages/contracts/src/schemas/*` + `schemas/*.json`; fixtures +
the conformance oracle. In scope (cloud, from `cloud/`): `cloud/packages/{db,api,worker}`
reads of `idempotency`/`seal` and the route surface. Out of scope / forbidden: any
back-compat (`harness_receipt`/journal aliases, dual-shape, shims); the full S-tier
claim-graph and enlightened dissolution (separate gated drafts); redesigning
`signal`/`authority` semantics beyond relocation.

## Acceptance

Profile: strict

Definition of done:
- [ ] `dod1` `runx.receipt.v1` is **flat** (no nested `run`/`harness` object; "run" is
  the conceptual signed body): top-level `idempotency`, `subject`(+`input_context`+scoped
  `commitments`), `authority`(terms + `bounds.payment` inspectable), `signals[]`,
  `decisions[]` (inline governance reasoning), `acts[]` (inline `intent`+`success_criteria`
  +`criterion_bindings`+`by`+`context_ref`), one `seal` (no `verification_summary`),
  optional `lineage`. No journal. `digest` covers the canonical body minus
  {signature, digest, metadata}.
- [ ] `dod2` No resurrected `outcome_resolution`: the post-run verdict is a
  `review`/`verification` **act** (in `acts[]`, or in a follow-up receipt linked by
  `lineage`), per `runx-retired-outcome-contract-sunset`. No `runx.resolution.v1` or
  `outcome_resolution` contract is introduced.
- [ ] `dod3` Verification is computed: `verify` returns a `ReceiptVerificationSummary`
  (signature/digest/criteria-binding/selected_act_id-inline/attenuation); no
  `verification_summary` field exists on the signed receipt; no journal exists.
- [ ] `dod4` The trainable projection is rich + hydrating: a row embeds the run and
  joins `acts[].context_ref` + `artifact_refs`; a projected row contains `intent.purpose`,
  `success_criteria` statements, decision justifications, and criterion outcomes (not ids),
  and computes verification on read.
- [ ] `dod5` Projections are one named pure layer (verify, trainable, history, metering,
  payment_ledger, lineage) over a sealed receipt; no bespoke field-picking exporter.
- [ ] `dod6` Contract-coherence harness (A+ scope = *detectors*, not the full source
  inversion): a cross-binding **conformance oracle** (canonical instances the Rust
  emitter produces, the TS validator accepts, and the JSON Schema validates, byte-
  identical) and an **emitter-validates-against-schema** test, both green and wired into
  the existing `contracts:schemas:check` gate. Full single-source generation
  (Rust→TS→JSON) is explicitly deferred to `rust-contract-pipeline-inversion`; A+ must
  not add a *new* hand-maintained copy.
- [ ] `dod7` Content-addressed id adopted: `receipt.id = hash(canonical_body)` under
  `runx.receipt.c14n.v1` (a Phase-1 item). The shared-envelope S-tier move is NOT in
  A+ scope (it depended on the dropped `resolution`).
- [ ] `dod8` Run-summary renamed to `runx.run-summary.v1` (with `receipt_ref`); the
  governance receipt owns `runx.receipt.v1`.
- [ ] `dod9` Durable identity: `ReferenceType::Receipt` + `runx:receipt:` prefix;
  `payment_ledger.rs` projects over it; `authority.terms[].bounds.payment` inspectable.
- [ ] `dod10` Hard cutover: no `harness_receipt`/`HarnessReceipt`/`harness-receipt`/
  `harness.seal`/`runx.harness_receipt.v1`/`runx:harness_receipt:`/`ReceiptJournal`/
  `journal_ref`/`verify_with_journal` token anywhere in OSS or cloud. No alias, no shim.

Validation:
- [ ] `v1` Scafld validates this spec.
  - Command: `scafld validate runx-receipt-aplus-shape-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v2` Receipts kernel tests pass on the flat run shape.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v3` Runtime emission + sealing tests pass (full suite).
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v4` Contract types tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v5` OSS no-compat + no-journal sweep is empty.
  - Command: `! rg -n 'harness_receipt|HarnessReceipt|harness-receipt|harness\.seal|runx\.harness_receipt\.v1|runx:harness_receipt:|ReceiptJournal|journal_ref|verify_with_journal' crates schemas fixtures packages`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v6` Contract schemas in sync + contracts test green.
  - Command: `pnpm contracts:schemas:check && npx vitest run packages/contracts/src/index.test.ts --config vitest.fast.config.ts`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v7` Trainable projection hydrates a rich training row (intent/criteria/justification/outcome, not ids).
  - Command: `npx vitest run packages/cli/src/trainable-receipts`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v8` Cross-binding conformance oracle + emitter-validates-against-schema.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-receipts conformance`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `v9` Cloud no-compat sweep empty + cloud typecheck + api/db tests (robust: only rg
  exit 1 = no matches passes; exit 0 = found and exit 2 = error both fail).
  - Command: `cd ../cloud && rg -n 'harness_receipt|HarnessReceipt|harness-receipt|harness\.seal|runx\.harness_receipt\.v1|runx:harness_receipt:' packages; rc=$?; if [ "$rc" -ne 1 ]; then echo "sweep failed (rc=$rc)"; exit 1; fi; pnpm typecheck && npx vitest run packages/api packages/db`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phases

- **Phase 1 — contracts (ground):** the flat `runx.receipt.v1` definition in the contract
  source (no nested `run`/`harness`); **content-addressed id** (`id = hash(canonical_body)`);
  run-summary rename confirmed (`runx.run-summary.v1`); emit `schemas/receipt.schema.json`;
  pin `runx.receipt.c14n.v1`. No `Resolution` contract (verdict is a `review`/`verification`
  act). No Rust/runtime changes yet.
- **Phase 2 — Rust contract types:** flat `Receipt` (+`Subject`, `Lineage`); reuse rich
  `Act`/`Decision`/`Authority`/`Seal` sub-types; delete `Harness*` and any `ReceiptJournal`.
- **Phase 3 — receipts kernel:** `canonical.rs` digests the canonical body (minus
  signature/digest/metadata); `verify.rs` returns the computed `ReceiptVerificationSummary`,
  checks `selected_act_id` inline, no seal-equality, no journal; `tree.rs`;
  `ReferenceType::Receipt` + `runx:receipt:`.
- **Phase 4 — runtime emitter:** emit flat `runx.receipt.v1` with rich inline `decisions`/
  `acts` (populate real `intent`/`success_criteria`/`criterion_bindings`/`context_ref`), one
  seal, no journal write; content-address the id.
- **Phase 5 — projections + cloud:** the named `projections` layer (verify, trainable
  hydrating, history, metering, payment_ledger, lineage); cloud reads against the flat shape;
  route `/v1/receipts`.
- **Phase 6 — fixtures + conformance + sweep:** regenerate fixtures + canonical-json oracle;
  add the cross-binding **conformance oracle** + **emitter-validates-against-schema** test
  (the contract-coherence detectors); run the no-compat + no-journal sweep (OSS + cloud) to
  empty.

## Rollback

Version break (`runx.receipt.v1` / `runx.receipt.c14n.v1`) with no compat layer; rollback is
`git revert` of the coordinated change, not a flag. Pre-cutover stored receipts/ledger are
re-seeded from source, not migrated. Partial application leaves the tree non-compiling by
design; sequence so kernel + emitter flip before the sweep asserts.

## Origin

Conversation 2026-05-22/23: the lean cutover butchered the trainable receipt by
exiling reasoning to a journal. This A+ draft is the corrected shape (rich inline run,
verification computed, resolution as a sibling artifact, projections as a real layer,
a structural contract-coherence harness). Build only on explicit confirmation.

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-23T13:02:32Z
Review gate: pass

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-22T14:33:19Z
Ended: 2026-05-22T14:33:19Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: The A+ draft is conceptually serious and largely the right architectural direction — flat, rich-inline, computed verification, named projections — but ships three structural ambiguities that block a safe approval: (1) the spec asserts `digest = sha256 over canonical(run)` while the existing kernel digests the flat receipt (no nested `run`), and never says whether `run` becomes a Rust struct or a virtual canonical subset; (2) DoD6's "single source generates Rust+TS+JSON Schema with --check gate and conformance oracle" is contradicted in-spec by "A+ at minimum must not add a fourth hand-copy" — Phase 6 cannot be implemented from the spec as written; (3) `idempotency` is present and cloud-load-bearing on today's Rust `Receipt` but is absent from the A+ shape diagram entirely. Several high-value advisory issues also surface: the `resolution` sibling pivot resurrects a contract retired three days ago (`runx-retired-outcome-contract-sunset` completed 2026-05-20); DoD7 layers two further structural changes (content-addressed id, shared envelope) onto an already-large cutover without phase placement; `v9`'s `cd ../cloud && ... ; test $? -eq 1 && ...` chains a cross-repo command that hides rg exit-code 2 (bad regex/missing dir) as a pseudo-success; and the in-flight `runx-receipt-clean-shape-v1` sits in `active/` with a failed review gate, which collides with approving the supersessor. None of these are reasons to abandon the A+ shape — they are the questions a one-coordinated-cutover spec must settle before build authorization.

Checks:
- path audit
  - Grounded in: code:crates/runx-contracts/src/harness.rs:303
  - Result: passed
  - Evidence: Verified declared touchpoints exist and are coherent with the body: `crates/runx-contracts/src/harness.rs` defines the flat `Receipt` (line 303) with `resolution: Option<Resolution>` (line 320); `crates/runx-receipts/src/{canonical,verify,tree}.rs` exist; `crates/runx-runtime/src/receipts/{seal,store,tree,signing,paths}.rs` exist; `crates/runx-runtime/src/{journal,payment_ledger}.rs` exist; `packages/cli/src/trainable-receipts.ts` and `.test.ts` exist; `packages/contracts/src/schemas/receipt.ts` exists; `schemas/receipt.schema.json` exists. The only path the spec invokes that does not yet exist is the `projections` module (referenced abstractly, not asserted as a current file).
- command audit
  - Grounded in: spec:Acceptance validation v5/v9
  - Result: failed
  - Evidence: `v5` uses `! rg -n ... crates schemas fixtures packages` with `expected_kind: exit_code_zero`. The `!` prefix flips rg's exit, so empty-match → 0 (the intended path), but rg-exit=2 (e.g. bad regex, missing dir) also → 1 → negated → 0, masking a real failure as success. Worse, `v9` chains `cd ../cloud && rg -n ... packages; test $? -eq 1 && pnpm typecheck && npx vitest run packages/api packages/db`: rg-exit=2 satisfies `test $? -eq 1` → false → typecheck and tests are skipped silently; only rg-exit=1 actually advances. `cd ../cloud` also assumes a sibling-repo layout that the OSS spec cannot guarantee from `oss/`. Sweep should use `! rg -q ...` with an explicit error-distinguishing form (e.g. capture exit and assert == 1), and cloud gating should not live inside an OSS acceptance command.
- scope/migration audit
  - Grounded in: spec:Phases + spec:DoD6 + code:packages/cli/src/trainable-receipts.ts:33,86
  - Result: failed
  - Evidence: Three migration tensions: (a) DoD6 says 'one source generates Rust + TS + JSON Schema; --check gate; cross-binding conformance oracle; emitter-validates-against-schema test' yet the same section explicitly says 'tracked by rust-contract-pipeline-inversion. A+ at minimum must not add a fourth hand-copy' — Phase 6 cannot be both 'don't add a fourth copy' and 'implement the generator and oracle'. (b) `packages/contracts/src/schemas/receipt.ts` is still encoded against the journal/detail_ref design (`detail_ref` line 125, `signal_refs` line 157, `journal_ref` line 159) while Rust `Receipt` is the rich-inline design — DoD10 demands those tokens be absent, so the schema rewrite is implicitly inside Phase 1/4 but never named. (c) The active spec `runx-receipt-clean-shape-v1` is in `.scafld/specs/active/` with `Review gate: fail`; the A+ spec is meant to supersede it but the lifecycle hand-off (archive clean-shape, lift active to A+) is not stated.
- acceptance timing audit
  - Grounded in: spec:Phases + spec:Validation v6/v7/v8
  - Result: failed
  - Evidence: `v6` (`pnpm contracts:schemas:check && npx vitest run packages/contracts/src/index.test.ts`) cannot pass until Phase 1 lands AND `packages/contracts/src/schemas/receipt.ts` is rewritten to the rich-inline shape (currently has journal_ref/detail_ref). `v7` (`npx vitest run packages/cli/src/trainable-receipts`) runs the existing field-picker test — today's projection at `packages/cli/src/trainable-receipts.ts:69` is `subject_ref/disposition/act_ids/runners/...` (passes shape, not richness). DoD4 demands the row contain `intent.purpose`, `success_criteria` statements, decision justifications, and *hydrated* criterion outcomes — but `v7`'s only assertion is `exit_code_zero`, so the existing thin test would pass without ever proving the hydration. `v8` (`cargo test ... -p runx-receipts conformance`) names a not-yet-existing test target. Each validation should declare the phase that authorizes it and the contentful assertion (e.g. fixture-row equality), not just exit code.
- rollback/repair audit
  - Grounded in: spec:Rollback
  - Result: passed
  - Evidence: The rollback plan is honest about scope: `git revert` of the coordinated change, no compat flag, pre-cutover stored receipts/ledger re-seeded from source, partial application leaves the tree non-compiling by design with sequencing 'kernel + emitter flip before the sweep asserts'. That is a credible rollback for a hard cutover and matches the in-flight code state. The unanswered repair edge — what happens to a locally-stored `deferred` receipt whose owning runtime then upgrades to A+ — is real but the spec already declares it out of scope ('re-seeded from source'). Acceptable for this profile; worth noting in the build phase notes, not blocking.
- design challenge
  - Grounded in: spec:'The shape' + code:crates/runx-receipts/src/canonical.rs:38 + archive:runx-retired-outcome-contract-sunset
  - Result: failed
  - Evidence: Three load-bearing design moves are under-specified: (1) The shape says `Receipt = signed(envelope, run)` and `digest = sha256 over canonical(run)`, but `crates/runx-receipts/src/canonical.rs:38` digests the flat receipt minus signature/digest/metadata — there is no `run` wrapper in the type. The spec never says whether Phase 2 introduces a nested `Run` struct (breaking every consumer's field path) or whether the digest commits a virtual subset (keeping flat field paths but changing the canonicalization contract). This is the single biggest implementer ambiguity. (2) `idempotency` is on today's `Receipt` (harness.rs:151, top-level) and is the cloud dedup magnet; the A+ shape diagram lists envelope + run with subject/authority/signals/decisions/acts/seal — `idempotency` is absent. Either it is implicitly retained at envelope-or-receipt level (unstated) or it is removed (cloud regression). (3) `runx.resolution.v1` as a sibling artifact: the spec presents this as the obvious fix because 'cannot sign a not-yet-existent verdict' — but the archived spec `runx-retired-outcome-contract-sunset` (completed 2026-05-20) deliberately deleted exactly this concept ('issue-to-pr-outcome and outcome-resolution schema modules ... removed') and chose to represent post-merge observation inline as `runx.harness_receipt.v1` nodes. The A+ pivot reverses that decision three days later without naming the prior retirement. This may be the right move (it cleanly fixes the sign-before-verdict problem), but the spec must acknowledge the reversal and say what changed between 2026-05-20 and today, or the team will repeat the same lifecycle. Architecturally, the rich-inline-flat-receipt with computed verification and named projections IS the right move — none of these issues argue against the tier choice; they argue the spec is not yet executable as one coordinated cutover.

Issues:
- [critical/blocks approval] `harden-1` design_ambiguity - `run` wrapper vs virtual canonical subset is unspecified
  - Status: open
  - Grounded in: spec:'The shape' + code:crates/runx-receipts/src/canonical.rs:38
  - Evidence: The shape says `Receipt = signed(envelope, run)` and `digest = sha256 over canonical(run)`. Today, `crates/runx-contracts/src/harness.rs:303` is a flat `Receipt` with no nested `run`, and `crates/runx-receipts/src/canonical.rs:38` digests the full receipt minus signature/digest/metadata. The spec never tells the implementer whether Phase 2 introduces a nested `Run` struct (changes every consumer's JSON field path) or whether the digest commits a virtual subset (keeps flat paths, changes the c14n contract).
  - Recommendation: Pick one in the spec before approval: either (a) define `Run` as a nested struct (`Receipt { envelope..., run: Run }`) and update all examples + fixtures + schema paths accordingly, or (b) keep flat fields and define `canonical(run)` as the named subset of receipt fields excluded from the digest, then list the excluded set explicitly. Whichever choice, name it in Phase 1 and Phase 3 so the kernel + emitter ship the same digest contract.
  - Question: Is `run` a nested Rust struct in the receipt, or a virtual subset of flat top-level fields that the canonicalizer commits?
  - Recommended answer: Keep the type flat (matches today's Rust + verifier + all fixtures), and define `canonical(run)` as the canonical subset excluding `schema`, `id`, `created_at`, `canonicalization`, `issuer`, `signature`, `digest`, `metadata`. Document the excluded set in Phase 1 so the spec, kernel, and schema agree on what `run` means.
  - If unanswered: Treat `run` as a virtual subset, not a nested struct (flat shape preserved); spec must list excluded fields.
- [critical/blocks approval] `harden-2` scope_contradiction - DoD6 contract-coherence harness contradicts the 'don't add a fourth hand-copy' guidance
  - Status: open
  - Grounded in: spec:DoD6 + spec:'How we ensure the contracts do not diverge'
  - Evidence: DoD6: 'Contract-coherence harness: one source generates Rust + TS + JSON Schema; a --check gate fails on drift; a cross-binding conformance oracle round-trips; an emitter-validates-against-schema test asserts the runtime's output validates.' Same section: 'Direction: Rust contracts as source, generate TS + JSON Schema; tracked by `rust-contract-pipeline-inversion`. A+ at minimum must not add a fourth hand-copy.' Either A+ ships the generator (subsumes the pipeline-inversion task, which is a large separate spec) or it does not (and DoD6's --check gate is unbuildable). The spec cannot be both.
  - Recommendation: Split DoD6 into two DoDs with explicit ownership: (i) emitter-validates-against-schema test + cross-binding conformance oracle as the ONLY contract harness inside A+ (small, lands in Phase 6, catches divergence at the boundary the current tree can express); (ii) full single-source generator + --check gate explicitly delegated to `rust-contract-pipeline-inversion`. The minimum guarantee in A+ is then 'no new hand-copy and an oracle test', not 'generated artifacts checked in'.
  - Question: Is A+ committing to the full generator + --check gate, or only to the conformance oracle + emitter-validates-against-schema test, with the generator delegated to `rust-contract-pipeline-inversion`?
  - Recommended answer: Only the conformance oracle + emitter-validates-against-schema test in A+. The full generator is `rust-contract-pipeline-inversion`'s job. DoD6 should be rewritten to assert only what Phase 6 can land here.
  - If unanswered: Treat A+ DoD6 as the oracle + emitter-validates test only; the --check generator gate is out of scope for this spec.
- [critical/blocks approval] `harden-3` shape_omission - `idempotency` is missing from the A+ shape but is on today's `Receipt` and cloud depends on it
  - Status: open
  - Grounded in: spec:'The shape' + code:crates/runx-contracts/src/harness.rs:151
  - Evidence: `crates/runx-contracts/src/harness.rs:151` defines `ReceiptIdempotency { intent_key, trigger_fingerprint, content_hash }` and `Receipt` carries it at line 311 as a required top-level field; the predecessor clean-shape spec calls out cloud dedup as the only thing cloud reads (`idempotency.{intent_key,trigger_fingerprint,content_hash}` + `seal.{disposition,closed_at}`). The A+ shape diagram (lines 53-91 of the draft) lists envelope + run with subject/authority/signals/decisions/acts/seal — `idempotency` is absent. The spec cannot silently drop a cloud-load-bearing field while claiming 'use-case fit must hold: ... cloud dedup/metering, payment ledger'.
  - Recommendation: Add `idempotency` to the shape diagram explicitly. Decide whether it lives on the envelope (excluded from the run digest) or inside `run.subject` (inside the digest). The cleanest place is `run.subject.idempotency` because the dedup keys are facts about the subject, and that keeps them signed. Acceptance DoD1 should then list it alongside the other run fields.
  - Question: Where does `idempotency` live in the A+ shape — envelope, run.subject, or top-level of run?
  - Recommended answer: Keep `idempotency` as a top-level field of `run` (signed, alongside subject/authority/decisions/acts/seal). It is a fact about this run, so it belongs inside the signed body, not the envelope.
  - If unanswered: Spec must add `idempotency` to `run` before approval; default position: top-level of `run`.
- [high/advisory] `harden-4` design_reversal - Resolution-as-sibling resurrects a contract retired three days ago without naming the reversal
  - Status: open
  - Grounded in: archive:runx-retired-outcome-contract-sunset + spec:'resolution is NOT a receipt field'
  - Evidence: `.scafld/specs/archive/2026-05/runx-retired-outcome-contract-sunset.md` (status: completed, updated 2026-05-20) explicitly deleted `outcome-resolution` schema modules and chose 'post-merge observation is represented by sealed runx.harness_receipt.v1 nodes with contained observation, verification, reply, and revision acts, not by a side runx.issue_to_pr_outcome.v1 packet or outcome_resolution contract.' The A+ spec (Origin says 'Conversation 2026-05-22/23') reverses this exactly: `runx.resolution.v1` as a separate signed artifact referencing the receipt. The justification ('cannot sign a not-yet-existent verdict') is sound, but the spec must name the reversal so the next agent does not delete it again.
  - Recommendation: Add a short subsection 'Why resolution is a sibling artifact (2026-05-20 reversal)' that cites the retirement, names the failure mode the inline-resolution-with-empty-verdict-at-seal pattern caused (or would have caused), and pins the decision. Without this, the next sweep will look at the new contract, see the archive, and propose deleting it again.
  - Question: Should the spec explicitly cite and reverse the 2026-05-20 retirement so the decision is durable?
  - Recommended answer: Yes. Add the reversal section and a one-line note in DoD2 pointing at the archived spec.
  - If unanswered: Spec ships with an unmarked reversal of a three-day-old completed decision; high probability the next agent re-deletes it.
- [high/advisory] `harden-5` layered_scope - DoD7 layers two further structural changes (content-addressed id, shared envelope) onto the cutover with no phase placement
  - Status: open
  - Grounded in: spec:DoD7
  - Evidence: DoD7: 'Cheap S-tier moves adopted: `receipt.id = hash(canonical(run))` (content-addressed) and one shared signed envelope for receipt + resolution.' Content-addressing the id is a structural identity change — today `id` is a string field assigned by the emitter (no derivation logic exists in `crates/runx-runtime/src/receipts/`). The 'shared signed envelope' is a new contract type that does not exist in `RUNX_LOGICAL_SCHEMAS` (only `receipt`, `runSummary`, ... — see `packages/contracts/src/internal.ts:58`). Neither item is placed in a phase, and neither has its own DoD. They are not 'cheap' relative to the rest of the cutover.
  - Recommendation: Either (a) place both in Phase 1 with their own sub-DoDs (`dod7a` content-addressed id with the derivation function + a fixture asserting `id == hash(canonical(run))`; `dod7b` shared envelope contract with its schema entry in `RUNX_LOGICAL_SCHEMAS`), or (b) drop DoD7 from A+ and move it to the S-tier spec where it belongs. Do not ship them as a one-line bonus DoD.
  - Question: Are content-addressed id and shared envelope first-class A+ phase work, or do they belong to the S-tier draft?
  - Recommended answer: Move them to S-tier. A+ is already a large coordinated cutover; adding identity derivation + a new envelope contract increases the per-phase blast radius without paying off here. The S-tier draft (`runx-receipt-claim-graph-s-tier-v1`) is the right place for both.
  - If unanswered: Drop DoD7 from A+; track in the S-tier draft.
- [medium/advisory] `harden-6` process_collision - Active sibling spec `runx-receipt-clean-shape-v1` has not been archived/superseded
  - Status: open
  - Grounded in: file:.scafld/specs/active/runx-receipt-clean-shape-v1.md:6-23
  - Evidence: `.scafld/specs/active/runx-receipt-clean-shape-v1.md` is `status: review, harden_status: in_progress, Review gate: fail` (lines 6-23). The A+ draft asserts it 'supersedes the in-flight runx-receipt-clean-shape-v1 direction'. Two active specs proposing different receipt shapes will confuse the runner and any future harden round. The body of clean-shape also has `resolution` inline (line 142), which is incompatible with A+ DoD2.
  - Recommendation: Before approving A+: run `scafld complete runx-receipt-clean-shape-v1` with a supersedes note, or `scafld archive` with explicit status `superseded_by: runx-receipt-aplus-shape-v1`. State the supersession path inside the A+ spec ('Supersedes: runx-receipt-clean-shape-v1') so the lifecycle is auditable.
  - Question: How are we retiring clean-shape — completion with note, archive-as-superseded, or letting it expire?
  - Recommended answer: Archive clean-shape as `superseded_by: runx-receipt-aplus-shape-v1` before `scafld approve` runs on A+, and add a `Supersedes:` line to A+'s frontmatter or 'Origin' section.
  - If unanswered: Archive clean-shape as superseded; add Supersedes header to A+.
- [medium/advisory] `harden-7` command_brittleness - Sweep commands mask rg error-exit as success and the cloud command crosses repo boundary
  - Status: open
  - Grounded in: spec:Validation v5 + spec:Validation v9
  - Evidence: `v5`: `! rg -n 'harness_receipt|...' crates schemas fixtures packages` with `expected_kind: exit_code_zero`. rg-exit=2 (bad regex / missing dir) becomes 1 after negation, hiding a real failure as success. `v9`: `cd ../cloud && rg -n 'harness_receipt|...' packages; test $? -eq 1 && pnpm typecheck && npx vitest run packages/api packages/db` — the `; test $? -eq 1 &&` chain treats rg-exit=2 as not-1 → false → typecheck and tests silently skipped. Additionally `cd ../cloud` assumes a sibling layout the OSS spec cannot guarantee.
  - Recommendation: Replace the `!` form with an explicit pattern: capture rg's exit (`out=$(rg ... ; echo $?)`) and assert `exit_status == 1`. Move the cloud sweep + typecheck + tests into the cloud repo's own validation spec, referenced from A+'s 'Dependencies' (`cloud-receipt-cutover-v1` or similar). Keep OSS acceptance commands inside the OSS tree.
  - Question: Should the cloud sweep + cloud test runs live inside a separate cloud spec, or stay bundled here?
  - Recommended answer: Move them to a sibling cloud spec referenced as a dependency. Keep `v9` in A+ only as a contract-shape assertion that does not require running cloud tests from the OSS working dir.
  - If unanswered: Rewrite v5/v9 to assert explicit rg exit==1 and drop the cross-repo `cd ../cloud` chain.
- [medium/advisory] `harden-8` test_assertion_strength - DoD4 demands a rich hydrated trainable row but `v7` only asserts exit_code_zero against today's field-picker
  - Status: open
  - Grounded in: code:packages/cli/src/trainable-receipts.ts:69-89 + spec:DoD4 + spec:Validation v7
  - Evidence: Today `packages/cli/src/trainable-receipts.ts:69-89` is a thin projection returning `subject_ref, disposition, reason_code, actor_ref, act_ids, runners, child_receipt_refs, signal_refs, journal_ref, receipt`. DoD4 says a projected row must contain 'intent.purpose, success_criteria statements, decision justifications, and criterion outcomes (not ids)' and must 'join acts[].context_ref + artifact_refs' to hydrate the rich row. `v7` runs `npx vitest run packages/cli/src/trainable-receipts` with `expected_kind: exit_code_zero` — the existing test continues to pass against the shallow shape and never proves the hydration happened. The shape-vs-test gap is what discredited the predecessor spec.
  - Recommendation: Strengthen `v7` to a contentful assertion: a checked-in fixture receipt + a checked-in expected trainable row whose JSON must equal the projection output. The fixture must include an act with `context_ref` and `artifact_refs` so the test fails if the projection returns the unhydrated row.
  - Question: Will Phase 5 add a fixture-based golden test for the trainable projection, or rely on the existing exit-code test?
  - Recommended answer: Add a golden-fixture test in Phase 5; promote `v7` from exit_code_zero to a contentful equality assertion via that fixture.
  - If unanswered: Add a fixture+expected-row golden in Phase 5 and rewrite v7 to compare JSON, not just run.


## Review

Status: completed
Verdict: pass
Mode: discover
Provider: gemini:gemini-3-flash-preview
Output: gemini.mcp_submit_review
Summary: The 'runx-receipt-aplus-shape-v1' task has been successfully implemented with high technical integrity. The core receipt and harness schemas are robust and well-mirrored across Rust and TypeScript. The 'runx-receipts' crate provides correct canonicalization and thorough verification logic, including content-addressing and signature validation. The 'runx-runtime' integration correctly seals receipts during execution, supporting both skill and graph runs. Payment ledger projections and target runner executions have been updated to utilize the new receipt shape without regressions. Comprehensive conformance tests in both Rust and TypeScript ensure the coherence of the receipt bindings across the project. Local runtime placeholders for timestamps and signatures are consistent with established project patterns for determinism.

Attack log:
- `schemas/, crates/runx-contracts/, packages/contracts/`: Verify that the new receipt.schema.json and harness.schema.json match the implemented Rust and TypeScript structs. -> clean
- `crates/runx-receipts/src/canonical.rs`: Verify canonicalization logic in crates/runx-receipts/src/canonical.rs ensures deterministic hashing and excludes non-committing fields. -> clean
- `crates/runx-runtime/src/receipts/seal.rs, crates/runx-runtime/src/execution/skill_run.rs`: Trace receipt generation and sealing in crates/runx-runtime to ensure content-addressed IDs and pseudo-signatures are correctly applied. -> clean
- `crates/runx-runtime/src/execution/target_runner.rs, crates/runx-runtime/src/payment_ledger.rs`: Check for regressions in target_runner.rs and payment_ledger.rs during the transition to the A+ receipt shape. -> clean
- `crates/runx-runtime/src/time.rs, crates/runx-runtime/src/receipts/seal.rs`: Audit for 'Dark Patterns' like hardcoded timestamps or weak idempotency keys. (Found intentional local determinism helpers). -> clean
- `crates/runx-receipts/src/verify.rs, crates/runx-receipts/src/verify/proof.rs`: Verify that the new runx-receipts crate correctly implements structural and cryptographic verification. -> clean

Findings:
- none

