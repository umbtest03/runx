---
spec_version: '2.0'
task_id: runx-verify-machine-surface-v1
created: '2026-06-10T05:08:42Z'
updated: '2026-06-10T05:42:22Z'
status: completed
harden_status: not_run
size: medium
risk_level: medium
---

# Machine-grade single-receipt verify surface

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-10T05:42:22Z
Review gate: pass

## Summary

Make `runx verify` consumable by machines, one receipt at a time. The hosted
receipt notary (spec `hosted-receipt-notary-v1` in the private root) verifies
edge-sealed receipts by invoking the runx binary, never by reimplementing
verification in TypeScript. That requires `runx verify` to accept a single
receipt from a file or stdin, emit a stable machine-readable JSON verdict, and
ship a conformance fixture corpus that any embedding surface can replay to
prove its verifier matches the CLI byte-for-byte.

This is the phase-1 "binary is the source of truth" guarantee: there is one
compiled verifier, and every surface that claims to verify a runx receipt
calls it.

## Objectives

- `runx verify --receipt <path>` and `runx verify --receipt -` (stdin) verify
  exactly one receipt document without requiring a receipt store directory.
- `--json` emits a stable verdict object: schema name, receipt id, digest
  validity, content-address validity, signature mode and outcome, findings
  (code/path/message), and a single top-level `valid` boolean.
- Exit codes are contractual: 0 valid, 1 invalid, 64 usage error.
- The verdict JSON shape gets a named schema id (e.g.
  `runx.verify_verdict.v1`); if runx-contracts schema emission is the
  established pattern for machine shapes, emit it there, otherwise lock the
  shape with fixture tests in the CLI crate.
- A conformance corpus of fixture receipts (valid, tampered body, tampered
  signature, unknown key, broken lineage, malformed JSON) lives in
  `oss/fixtures/receipt-verify/` with the expected verdict for each, and
  tests replay the corpus through both the CLI surface and the library API.
- Store-mode tree verification semantics are unchanged.

## Scope

In scope:

- `crates/runx-cli/src/verify.rs` single-receipt input, stdin support, verdict
  JSON output.
- `crates/runx-cli/src/launcher.rs` help text for the new flag.
- Conformance fixtures under `oss/fixtures/receipt-verify/` plus replay tests
  in runx-cli and runx-receipts.
- Docs note in `docs/security-authority-proof.md` (Offline Receipt
  Verification section).

Out of scope:

- Any hosted/notary code (separate private-root spec).
- Changes to receipt wire schemas or sealing.
- Changes to store-mode tree verification behavior.
- Networked verification or key distribution.

## Dependencies

- Offline verify command on main:
  [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)
- Pure verification in runx-receipts:
  [crates/runx-receipts/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify.rs)
- Existing safe-projection precedent in
  [crates/runx-receipts/tests/receipt_contracts.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/tests/receipt_contracts.rs)

## Grounding Evidence

- `runx verify` exists with store-dir mode, lineage-tree grouping, production
  signature verification via `RUNX_RECEIPT_VERIFY_KID` /
  `RUNX_RECEIPT_VERIFY_ED25519_PUBLIC_KEY_BASE64`, and non-zero exit on
  findings.
  [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)
- Verification primitives are pure and complete in runx-receipts:
  `verify_receipt`, `verify_receipt_proof`, `receipt_id_is_content_addressed`,
  and the `ReceiptFindingCode` vocabulary.
  [crates/runx-receipts/src/verify/finding.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify/finding.rs:10)
- Receipt issuer types already distinguish Local and Hosted issuers, which the
  notary counter-seal relies on downstream.
  [crates/runx-contracts/src/receipt.rs](/Users/kam/dev/runx/runx/oss/crates/runx-contracts/src/receipt.rs:282)
- Fixture receipts with valid/abnormal shapes already exist under
  [crates/runx-receipts/fixtures/contracts/harness-spine](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/fixtures/contracts/harness-spine)
  and can seed the conformance corpus.

## Assumptions

- The executor may be Codex; record evidence through
  `scafld build runx-verify-machine-surface-v1` after each phase and run
  `scafld review` with a real provider before `scafld complete`.
- Use `CARGO_TARGET_DIR=target/runx-verify-machine-surface` for all cargo
  commands to avoid contending with other agents' builds.
- Single-receipt mode verifies digest, content address, structure, and
  signature. Lineage findings that require sibling receipts are reported as
  an explicit `lineage_unverified` informational state, not failures, because
  a single document cannot prove tree membership. Store mode remains the tree
  authority.
- stdin/file input is size-capped (reject above ~10 MiB) with a usage-class
  error so the surface cannot be memory-bombed.
- The verdict shape is additive-stable: fields may be added, never repurposed;
  semantic changes require a new schema id version.

## Touchpoints

- [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)
- [crates/runx-cli/src/launcher.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/launcher.rs)
- [crates/runx-receipts/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify.rs)
- [fixtures/](/Users/kam/dev/runx/runx/oss/fixtures) (new `receipt-verify/` corpus)
- [docs/security-authority-proof.md](/Users/kam/dev/runx/runx/oss/docs/security-authority-proof.md)

## Risks

- Verdict shape churn would break the notary contract downstream. Mitigation:
  fixture-locked verdict tests and a named schema id from day one.
- Single-receipt mode could silently weaken tree semantics. Mitigation:
  explicit `lineage_unverified` reporting; store mode untouched.
- Parsing attacker-supplied receipts from stdin is an attack surface.
  Mitigation: size cap, existing serde strictness (`deny_unknown_fields` on
  receipt types), malformed-input fixtures in the corpus.

## Acceptance

Profile: strict

Validation:
- `cd crates && cargo fmt --check`
- `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli verify`
- `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli launcher`
- `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-receipts`
- `git diff --check`

## Phase 1: Single-Receipt Input And Verdict JSON

Status: completed
Dependencies: none

Objective: One receipt in, one stable verdict out.

Changes:
- Add `--receipt <path|->` to `runx verify`; mutually exclusive with a positional store receipt id.
- Implement the verdict JSON object with a named schema id; `--json` in single-receipt mode emits exactly one verdict document.
- Exit codes: 0 valid, 1 invalid, 64 usage.
- Enforce the input size cap with a usage-class error.
- Unit tests for valid, tampered, oversized, and malformed inputs.

Acceptance:
- [x] `ac1` command - CLI verify tests pass with single-receipt mode
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli verify`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac2` command - Launcher help/routing tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli launcher`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Conformance Corpus

Status: completed
Dependencies: phase1

Objective: Any embedding surface can prove it verifies exactly like the CLI.

Changes:
- Build `fixtures/receipt-verify/` with at least: valid production-signed, tampered body, tampered signature, unknown kid, broken lineage reference, malformed JSON, plus an expected-verdict JSON per fixture.
- Add a replay test in runx-cli running every corpus entry through the single-receipt surface, asserting the exact expected verdict.
- Add a runx-receipts test replaying the same corpus through the library API so the CLI and library can never drift.
- Document the corpus as the notary's conformance gate in `docs/security-authority-proof.md`.

Acceptance:
- [x] `ac3` command - Corpus replay passes through the CLI surface
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli verify`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-12
- [x] `ac4` command - Corpus replay passes through the library API
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-receipts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13

## Phase 3: Final Gate

Status: completed
Dependencies: phase2

Objective: Formatting and whitespace clean; no store-mode regression.

Changes:
- Run formatting and the focused test list.
- Confirm store-mode tree verification output is unchanged under existing tests.

Acceptance:
- [x] `ac5` command - Rust formatting is clean
  - Command: `cd crates && cargo fmt --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18
- [x] `ac6` command - Diff has no whitespace errors
  - Command: `git diff --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19

## Follow-Up Specs

- `hosted-receipt-notary-v1` (private root): consumes this surface as the
  notary's verifier; its build is blocked until this spec completes.

## Rollback

- Revert the new flag and corpus together; store-mode behavior is untouched so
  rollback cannot regress existing verification.
- A retired verdict schema id must never be reused with different semantics;
  bump the version instead.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Scope is implemented coherently: `runx verify` now accepts `--receipt <path|->`, mutually excludes it from store-mode flags, emits a `runx.verify_verdict.v1` JSON document with schema/receipt_id/digest/content_address/signature/lineage/findings/valid, enforces a 10 MiB usage-class cap, maps usage vs invalid vs ok to exit 64/1/0 in main.rs, and the conformance corpus under `oss/fixtures/receipt-verify/` (valid, tampered body, tampered signature, unknown kid, broken lineage, malformed JSON) is replayed byte-for-byte through both the CLI surface (`runx-cli` corpus test) and the library API (`runx-receipts` `receipt_verify_corpus.rs`), so the CLI and library cannot drift. Store-mode tree verification, signature mode resolution, and the help line are preserved and the docs section in `security-authority-proof.md` documents the corpus as the conformance gate. Acceptance criteria ac1–ac6 are recorded as passing. Three non-blocking quality findings noted around oversize path handling, the per-check signature status when the body digest cannot be recomputed, and brittleness of pinning a third-party parser error message in the malformed-json corpus. None block completion.

Attack log:
- `spec compliance`: Re-run acceptance criteria against the verdict schema, exit codes, mutual exclusion, stdin path, and corpus replay (CLI + library) -> clean (All ac1-ac6 already recorded passing; --receipt + --receipt=- both wired; VerifyCliError::InvalidArgs -> 64, failed -> 1, ok -> 0 in main.rs::run_native_verify; schema 'runx.verify_verdict.v1' surfaced in both fixtures and lib re-exports.)
- `ambient drift`: Classify ambient drift outside task scope and confirm no overlap with verify/verdict surfaces -> clean (Heavy ambient deletions (runtime/payment/pay tests) belong to a separate cutover and never touch verify.rs, verdict.rs, the corpus, or the docs section under review.)
- `scope drift`: Diff declared task scope against task changes for undeclared additions -> clean (All task_changes paths fall under crates/runx-cli/src/verify.rs, crates/runx-cli/src/launcher.rs (help line only), crates/runx-receipts/src/verify*, docs/security-authority-proof.md, or fixtures/receipt-verify/. No store-mode shape changes.)
- `regression hunt`: Trace callers of run_verify_command(_with_stdin), help_text, and verify/lib re-exports for broken consumers -> clean (main.rs::run_native_verify is the sole caller and routes both stdin and non-stdin via run_verify_command_with_stdin; launcher.rs help text covers the new flag and is asserted by tests/launcher.rs::top_level_help_and_version_are_native; existing store-mode unit tests still exercise --receipt-dir without single-receipt entanglement.)
- `convention check`: Cross-check CLAUDE.md and CONVENTIONS guidance: trusted-kernel purity, error envelope, no test logic in production, public API stability, config_from_env, no legacy code -> clean (Verdict types live in runx-receipts (pure crate), no fs/net imports added there; CLI tests guarded by #[cfg(test)]; env keys reused from history module rather than re-defined; schema is versioned; no v2/compat aliases.)
- `dark patterns`: Hunt for subtle bugs: TOCTOU on path size cap, per-check status drift, mode/env coupling, fixture vs documented semantics, parser error text pinning, alphabetized vs canonical fixture ordering -> finding (Logged F1 (path-mode oversize buffers before cap when metadata fails), F2 (signature.status='valid' when signature path never ran), F3 (corpus pins serde_json's free-form error message). broken-lineage-reference fixture is intentionally documented as valid in single-receipt mode and the docs cover it, so noted but not raised.)

Findings:
- [low/non-blocking] `F1-path-oversize-fully-buffered` Path-mode --receipt reads the whole file before the size cap fires when fs::metadata fails
  - Location: `crates/runx-cli/src/verify.rs:326`
  - Evidence: read_single_receipt_input only enforces the cap pre-read when fs::metadata succeeds (`if let Ok(metadata) = fs::metadata(&path)`); on metadata error it falls through to fs::read(&path) which buffers the entire file into a Vec before the post-read `document.len() > SINGLE_RECEIPT_MAX_BYTES` check rejects it. Special files (FIFOs, sockets) or permission-restricted files that allow read but deny stat can bypass the pre-check.
  - Impact: A multi-GB receipt path could be fully buffered into memory before being rejected, allowing accidental OOM on hosts with limited RAM. The operator controls the input path, so this is not a hostile-input scenario, but the documented cap is not actually enforced on the read side.
  - Validation: Add a path-mode oversize unit test against a temp file > cap and confirm InvalidArgs without buffering the entire body.
- [low/non-blocking] `F2-signature-status-valid-when-not-evaluated` signature.status can report 'valid' even when the signature was never actually verified
  - Location: `crates/runx-receipts/src/verify/verdict.rs:179`
  - Evidence: signature_check classifies status as 'valid' whenever no SignatureXxx finding is present. In proof.rs::check_body_proof, when canonical_receipt_body_digest fails the verifier pushes SealDigestMismatch and returns before calling check_signature, so no signature finding is emitted and signature.status resolves to 'valid' even though signature verification never ran. The same is true for any future early-return added before the signature pass.
  - Impact: Per-check status drifts from per-check truth. The top-level `valid` flag remains correct (verification.valid + digest 'not_evaluated' force false), but a machine consumer that consults signature.status independently (e.g., a notary summarizing 'signature ok, digest broken') could mis-attribute. Unlikely in practice because canonical_receipt_body_digest only fails on a serializer breakdown.
  - Validation: Add a verdict unit test for a receipt whose body cannot canonicalize and assert signature.status == 'not_evaluated', then mirror that case in the corpus.
- [low/non-blocking] `F3-corpus-pins-serde-json-error-text` Malformed-json corpus fixture pins serde_json's free-form error message
  - Location: `fixtures/receipt-verify/malformed-json/expected.json:28`
  - Evidence: expected.json hard-codes `"message": "EOF while parsing a value at line 1 column 28"` and both the CLI replay test (run_verify_command + assert_eq! on the JSON value) and the library replay test (verify_receipt_document_verdict + assert_eq!) compare with strict equality. That string is serde_json's internal error formatting, not part of any public contract, and any serde_json patch update that re-words it would break both replay tests until the corpus is regenerated.
  - Impact: Maintenance brittleness for the byte-for-byte conformance contract: a routine dependency bump can fail the corpus, and embedding surfaces pinned to an older binary release will see the older message even though the corpus moved on. Not a correctness or security issue.
  - Validation: Bump serde_json across a minor patch range in a scratch branch and confirm the corpus tests still pass under the chosen normalization or matcher.

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-06-10: Authored as the dependency root of the phase-1 connector-hosting
  plan: the hosted notary verifies via the compiled runx binary, so the binary
  needs a machine-grade single-receipt surface and a conformance corpus first.
