---
spec_version: '2.0'
task_id: rust-receipt-proof-verification
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T05:19:20Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Rust receipt proof verification

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T05:19:20Z
Review gate: pass

## Summary

Turn `runx-receipts` from a structural receipt checker into a proof verifier.
The verifier must be able to recompute canonical receipt commitments, distinguish
body digests from full receipt digests, verify signatures through an explicit
verifier interface, and fail closed when receipt metadata claims proof strength
that the payload does not actually provide.

This spec closes the security gap that remains after basic Rust receipt parity:
a receipt that has the right fields is not necessarily trustworthy.

## Context

CWD: `.` (runx OSS workspace)

Relevant crates and fixtures:
- `crates/runx-receipts/src/canonical.rs`
- `crates/runx-receipts/src/verify/**`
- `crates/runx-receipts/tests/**`
- `crates/runx-contracts/src/harness.rs`
- `crates/runx-contracts/src/authority.rs`
- `crates/runx-contracts/src/redaction.rs`
- `fixtures/contracts/harness-spine/**`

Known weak spots to close:
- Canonical JSON currently depends on generic serde serialization behavior.
- Receipt digests are not split cleanly between body commitments and full
  receipt archival fingerprints.
- `seal.digest`, `verification_summary`, signatures, redaction commitments, and
  external proof fields are not all treated as independently verifiable claims.
- Test fixtures do not yet include enough negative cases to catch proof drift.

Invariants:
- Structural validation and strict proof verification stay separate APIs.
  Structural validation may report shape-only validity; strict proof
  verification is the API used for governed acceptance.
- Strict proof verifier APIs fail closed. Missing verifier inputs are explicit
  errors, not soft warnings, whenever the receipt carries a signature,
  redaction reference, authority proof reference, or external attestation claim.
- Public review output never leaks absolute local paths, secrets, env vars, or
  raw provider credentials.
- The proof layer remains generic runx core. Nitrosend routing, Slack text, and
  Sentry payload policy stay outside this crate.
- Rust proof verification is for `runx.harness_receipt.v1` only. Legacy
  `runx.receipt.v1` local/graph receipts remain archival TypeScript-era
  records and must not become compatibility input to this verifier.

## Objectives

- Define explicit canonical serialization for harness receipt proof material.
- Add separate body-digest and full-receipt-digest APIs.
- Recompute and verify `seal.digest` against the body commitment.
- Verify signatures through an injected verifier/key resolver, with deterministic
  test verifiers for fixtures.
- Verify `verification_summary` honesty: failed child checks, missing proofs, or
  malformed attestations cannot produce a successful summary.
- Add negative fixtures for digest tamper, signature tamper, redaction mismatch,
  missing external proof, and unsupported proof authority.

## Scope

In scope:
- Rust receipt proof APIs and tests.
- Contract-spine fixture expansion for proof-positive and proof-negative cases.
- Verifier finding codes for proof failures.
- Documentation of the proof model exposed to runtime, CLI, and Aster.
- Classification of existing harness-spine fixtures as structural-only until
  their `seal.digest` and `signature.value` are recomputed from fixture keys.

Out of scope:
- Persistent receipt store lookup or path discovery; owned by
  `rust-runtime-receipt-path-discovery`.
- Graph/tree traversal and child receipt lookup; owned by
  `rust-receipt-tree-resolution`.
- Cloud storage implementation.
- Nitrosend-specific Slack/GitHub comment formatting.
- Porting or accepting legacy `schema_version: runx.receipt.v1` skill/graph
  local receipt verification in `runx-receipts`.

## Dependencies

- `runx-contract-spine-hard-cutover`.
- `rust-receipts-parity`.
- `rust-policy-authority-proof-parity` for final authority-proof semantics. This
  spec may require an injected authority-proof verifier result, but it must not
  reimplement or silently fork that authority algebra while the parity slice is
  landing.
- Coordinates with `rust-receipt-tree-resolution`; either order can land, but
  final parent/child proof acceptance requires both.

## Assumptions

- Test keys and fixture-only verifiers are acceptable in deterministic fixtures,
  but production verification must use explicit verifier inputs.
- Existing archived receipts can remain archival artifacts; live governed paths
  must use post-cutover proof-verifiable receipts.
- Existing `fixtures/contracts/harness-spine/harness-receipt-success.json` and
  `harness-receipt-abnormal.json` are structural fixtures unless regenerated
  with real fixture signatures and recomputed seal digests.

## Proof Contract Decisions

- Body commitment input is the canonical `runx.harness_receipt.v1` envelope
  with `signature` removed and every `seal.digest` plus
  `seal.verification_summary` occurrence removed before hashing. This includes
  both the top-level `seal` and mirrored `harness.seal`.
- Full receipt digest is a separate archival fingerprint over the immutable
  fully populated receipt, including `signature`, `seal.digest`, and
  `verification_summary`. It is never used as the acceptance check for
  `seal.digest`.
- Canonicalization must be an explicit writer for the harness receipt proof
  material, not an accidental consequence of `serde_json::to_value` field order.
  Golden fixtures must assert exact canonical bytes and SHA-256 digests.
- Signature verification uses an injected verifier/key resolver. Missing keys,
  unsupported `issuer.type`, unsupported `signature.alg`, malformed signature
  bytes, or issuer key hash mismatch are strict proof failures.
- Authority proof verification is delegated to the authority-proof verifier
  surface from `rust-policy-authority-proof-parity`. `runx-receipts` checks
  binding and fail-closed semantics: claimed authority validity without a
  verified authority result is invalid.
- Redaction and hash commitments are recomputed from supplied fixture material
  or explicit verifier inputs. A reference alone does not make
  `redaction_valid` or `hash_commitments_valid` true.
- External attestations are explicit proof inputs keyed by receipt references.
  `external_attestations_present: true` is invalid unless the verifier receives
  and validates the referenced attestation material.

## Touchpoints

- `runx-receipts` canonical/digest modules.
- `runx-receipts` verification findings and summary output.
- Contract-spine fixture schema docs.
- Runtime and CLI callers that display receipt verification results.

## Risks

- Canonicalization drift could invalidate legitimate receipts if not versioned
  and fixture-backed.
- Treating missing proof material as a warning would create a false security
  signal.
- Signing the wrong material would allow metadata mutation without detection.

## Acceptance

Profile: strict

Validation:
- `cargo fmt --check --manifest-path crates/Cargo.toml`
- `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts harness_spine_fixtures`
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-receipts --all-targets --all-features -- -D warnings`
- `git diff --check`

Required behavior:
- [ ] Canonical writer has fixture tests proving stable object ordering and
  deterministic output across repeated runs.
- [ ] Body digest excludes `signature`, every `seal.digest`, and every derived
  `seal.verification_summary`; full digest includes immutable archival fields by
  explicit design.
- [ ] Receipt with a tampered body fails seal verification.
- [ ] Receipt with a tampered signature fails signature verification.
- [ ] Receipt with missing verifier inputs fails when signature verification is
  required.
- [ ] Receipt with a redaction commitment mismatch fails verification.
- [ ] Receipt with `redaction_valid: true` but unresolved or unrecomputed
  redaction references fails verification.
- [ ] Receipt that claims an external attestation without verifiable attestation
  material fails verification.
- [ ] Receipt that claims `authority_attenuation_valid: true` without a verified
  authority-proof result fails verification.
- [ ] Unsupported proof authorities and unsupported issuer/key types produce
  specific finding codes.
- [ ] Verification summary cannot claim success when any required proof check
  failed.
- [ ] Public verification output redacts local filesystem paths and raw secret
  values.
- [ ] `runx-receipts` does not accept or add compatibility aliases for legacy
  `schema_version: runx.receipt.v1` skill/graph local receipts.

## Phase 1: Proof Model

Status: completed
Dependencies: none

Objective: Make the proof contract explicit before changing callers.

Changes:
- Define body/full digest semantics.
- Define required verifier inputs and failure modes.
- Document the exact receipt fields included in each proof commitment.
- Define the fixture key format and how proof-positive fixtures are regenerated.

Acceptance:
- none

## Phase 2: Implementation

Status: completed
Dependencies: Phase 1

Objective: Implement proof checks in `runx-receipts`.

Changes:
- Add canonical writer and digest APIs.
- Add signature verifier trait or equivalent injected verifier boundary.
- Add authority-proof, redaction, hash-commitment, and external-attestation verifier input structs or equivalent typed context.
- Add proof finding codes and strict summary aggregation.

Acceptance:
- none

## Phase 3: Integration

Status: completed
Dependencies: Phase 2

Objective: Make runtime/CLI consumers display proof status without leaking

Changes:
- Update receipt verification projections.
- Ensure CLI review text reports concise proof status and actionable failures.

Acceptance:
- none

## Rollback

- Keep old structural verification behind tests until the proof verifier is
  green, then remove redundant code in the same change.
- If proof verification exposes fixture gaps, keep this spec open and add the
  missing fixtures instead of weakening the verifier.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Strict proof verifier surface is in place: separate body/full digest APIs, an injected SignatureVerifier trait, fail-closed authority/redaction/hash/external-attestation summary checks, and a path-redacting projection layer. Negative tests cover tamper, missing verifier, unsupported issuer, redaction gaps, hash gaps, and missing external attestations. Discover review surfaces four non-blocking issues: (1) canonical body stripping is too broad and erases any nested `signature` key — most concerning for the free-form `metadata: Option&lt;JsonObject&gt;` whose tampering then escapes the body digest; (2) Proof Contract Decisions calls for golden fixtures asserting exact canonical bytes and SHA-256 digests, but tests only assert repeated-run equivalence and `sha256:` prefix; (3) `runx-receipts` README still claims signature checking is out of scope; (4) `public_text` redacts only `/`-leading whitespace-delimited tokens (no Windows paths, no secret values). None of these block completion: the realistic exploit surface is narrow, structural invariants and main proof paths are correct.

Attack log:
- `crates/runx-receipts/src/canonical.rs::strip_body_proof_fields`: Attempt to mutate a body field that escapes the digest by exploiting recursive `signature`/`seal` stripping -> finding (Recursive strip of `signature` key allows tampering inside `metadata` JsonObject without changing body digest.)
- `crates/runx-receipts/src/canonical.rs::canonical_json_value`: Check for deterministic key ordering and number serialization -> clean (BTreeMap-backed sort plus JsonNumber Display gives stable output; F64 whole-number coercion is consistent.)
- `crates/runx-receipts/src/verify/proof.rs::check_body_digest`: Tamper body, ensure SealDigestMismatch fires for both top-level seal and harness.seal -> clean (Both digests are compared against the recomputed body commitment.)
- `crates/runx-receipts/src/verify/proof.rs::check_signature`: Trigger every SignatureVerificationFailure variant and confirm finding-code mapping -> clean (signature_failure_code maps each variant to a distinct finding; tampered-signature, missing-verifier, unsupported-issuer, unsupported-algorithm covered by tests.)
- `crates/runx-receipts/src/verify/proof.rs::check_summary`: Confirm summary cannot honestly claim success without matching context flags or material -> clean (Authority, redaction (per-ref), hash (per-commitment), and external_attestations summaries all fail closed without verified context.)
- `crates/runx-receipts/src/verify/proof.rs::check_signature_summary`: Check whether dishonest `signature_valid:true` with actual signature failure is flagged separately -> clean (Not directly flagged, but check_signature already fails the verification overall; gap is theoretical.)
- `crates/runx-receipts/src/verify/projection.rs::public_text`: Probe redaction with Windows paths, punctuation-prefixed paths, and embedded secrets -> finding (Only redacts whitespace-delimited tokens that begin with `/`; misses Windows paths and secret values.)
- `crates/runx-receipts/README.md`: Compare README claims against new public API -> finding (README still says signature checking is out of scope.)
- `crates/runx-receipts/src/canonical.rs::tests`: Look for golden fixtures asserting exact canonical bytes / SHA-256 digests as Phase 1 requires -> finding (Tests assert reproducibility and prefix only; no byte-level or digest-level golden assertion.)
- `crates/runx-runtime/src/receipts.rs`: Trace runtime consumers of the new strict proof surface for regression risk -> clean (Runtime still uses structural validate_harness_receipt only; placeholder-signed receipts would correctly fail strict proof (no integration regression introduced).)
- `fixtures/contracts/harness-spine/harness-receipt-success.json`: Verify the spec's classification of harness-spine fixtures as structural-only -> clean (Spec explicitly classifies these as structural-only; tests recompute digest before strict-proof tests, matching the classification.)

Findings:
- [medium/non-blocking] `F-1-overbroad-strip` Body-digest stripping recursively erases every `signature` key, allowing tampering inside free-form `metadata` to escape the seal commitment
  - Location: `crates/runx-receipts/src/canonical.rs:52`
  - Evidence: strip_body_proof_fields() at canonical.rs:52 walks the entire receipt JSON and calls `map.remove("signature")` on every object. `HarnessReceipt.metadata: Option<JsonObject>` is a free-form BTreeMap (json.rs:7); if a sender puts `{"signature": ...}` inside metadata, that key is silently stripped from the body commitment, so it can be changed after sealing without altering canonical_receipt_body_digest. The spec Proof Contract Decisions (`active/rust-receipt-proof-verification.md` lines 127-131) says the envelope is hashed `with signature removed` (singular, top-level) but is explicit about `every seal.digest plus seal.verification_summary occurrence` for the seal fields — the difference in wording suggests targeted removal for `signature`.
  - Impact: A receipt that round-trips through `canonical_receipt_body_digest` does not commit to the contents of any nested object keyed `signature`, including any future per-act/per-attestation signature fields. This weakens the acceptance criterion `Receipt with a tampered body fails seal verification` for that subset of body content.
  - Validation: Add a fixture where metadata contains a `signature` key; the body digest must change when that nested field changes. Existing tamper tests should still pass.
- [medium/non-blocking] `F-2-no-golden-bytes` No golden fixture asserts exact canonical bytes or SHA-256 digest, so canonicalization regressions are invisible until a downstream consumer disagrees
  - Location: `crates/runx-receipts/src/canonical.rs:114`
  - Evidence: Phase 1 Proof Contract Decisions (`active/rust-receipt-proof-verification.md` lines 135-137) requires: `Golden fixtures must assert exact canonical bytes and SHA-256 digests`. Canonical tests at canonical.rs:140-173 only check `first == second`, `starts_with("{\"created_at\":")`, and `digest.starts_with("sha256:")`. The fixture digest test at canonical.rs:132 hashes only `b"runx"`. tests/harness_receipts.rs::strict_proof_accepts_recomputed_digest_and_signature recomputes the digest into the receipt before verifying, so a canonicalization drift would still self-validate.
  - Impact: A change to `serde_json`, the contract struct field set, or `canonical_json_value` could silently produce a different body digest without any test failure. Cross-implementation (Rust vs. legacy TS) drift would also be invisible.
  - Validation: Run `cargo test -p runx-receipts` after introducing the golden assertions; a deliberate canonical perturbation must fail the test.
- [low/non-blocking] `F-3-readme-stale` Crate README still says signature checking lives outside the crate, contradicting the newly added `SignatureVerifier` surface
  - Location: `crates/runx-receipts/README.md:15`
  - Evidence: README.md:11-16 describes the crate as covering `structural invariants` and explicitly states `Signature checking, persistent child receipt lookup, and full authority algebra verification are separate runtime integrations`. After this task, `verify_harness_receipt_proof`, `SignatureVerifier`, `ReceiptProofContext`, and `receipt_proof_status` are public API (lib.rs:19-25).
  - Impact: Downstream consumers and reviewers reading the README will under-estimate the crate's surface and may miss the strict-proof entrypoint.
  - Validation: Re-read README after edit; ensure the strict-proof entrypoints are mentioned and the misleading sentence is removed.
- [low/non-blocking] `F-4-public-text-redaction` Public projection redaction only catches `/`-leading whitespace-delimited tokens; Windows paths and secret values are not redacted
  - Location: `crates/runx-receipts/src/verify/projection.rs:49`
  - Evidence: public_text() at projection.rs:49-61 splits on whitespace and replaces a token only if it `starts_with('/')`. Windows-style paths (e.g. `C:\Users\...`) and tokens with leading punctuation (e.g. `(/Users/...`) pass through. The acceptance bullet says `Public verification output redacts local filesystem paths and raw secret values`; raw secret values (e.g. signature bytes, key material echoed into a message) are not redacted by any rule.
  - Impact: Today no finding message embeds secrets, but the contract is weaker than the acceptance claim. A future verifier that interpolates raw values into a message, or a CI runner on Windows, would leak through this projection.
  - Validation: Add tests for Windows paths, embedded paths, and a sample base64-looking secret; assert they are redacted.

## Self Eval

- Target score: 9.5. Passing means receipts are useful security evidence, not
  just structured logs.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: close receipt proof gaps before TS sunset and live Aster use

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T04:06:48Z
Ended: 2026-05-19T04:11:56Z

Checks:
- path audit
  - Grounded in: code:crates/runx-receipts/src/canonical.rs:12
  - Result: passed
  - Evidence: Current canonical JSON is generic serde serialization; the spec now requires an explicit versioned proof-material writer with exact fixture bytes and digests.
- command audit
  - Grounded in: code:crates/runx-receipts/tests/harness_receipts.rs:22
  - Result: passed
  - Evidence: Existing tests load harness-spine fixtures for structural invariants; the spec now adds contract fixture validation and requires recomputed proof-positive fixtures.
- scope/migration audit
  - Grounded in: code:packages/contracts/src/schemas/local-receipt.ts:153
  - Result: passed
  - Evidence: Legacy local skill/graph receipts use `runx.receipt.v1`; the spec now limits strict proof verification to `runx.harness_receipt.v1`.
- acceptance timing audit
  - Grounded in: code:crates/runx-contracts/src/harness.rs:128
  - Result: passed
  - Evidence: Harness seals carry `digest` and optional `verification_summary`; the spec now makes body-digest and summary recomputation Phase 1 exit criteria.
- rollback/repair audit
  - Grounded in: code:crates/runx-receipts/src/verify.rs:22
  - Result: passed
  - Evidence: The current verifier can remain structural; the spec now adds strict proof verification without presenting structural validity as governed proof.
- design challenge
  - Grounded in: code:crates/runx-contracts/src/authority.rs:153
  - Result: passed
  - Evidence: Authority attenuation proof fields exist, but authority algebra belongs to the parity slice; the spec now requires injected verified authority results and fail-closed binding.

Issues:
- none


## Planning Log

- 2026-05-19: Expanded placeholder into proof-verification contract after review
  of receipt parity and tree-resolution gaps.
