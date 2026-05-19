---
spec_version: '2.0'
task_id: rust-policy-authority-proof-parity
created: '2026-05-17T00:00:00Z'
updated: '2026-05-19T04:31:41Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Rust policy authority-proof parity

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T04:31:41Z
Review gate: pass

## Summary

Port the remaining pure `@runxhq/core/policy` authority-proof and public-work
surface to Rust. This completes the current policy kernel parity slice without
changing the TypeScript source of truth, runtime adapters, provider calls, or
contract-vocabulary cutover.

This is a parity task, not a redesign task. The Rust implementation must match
the current TypeScript authority-proof and public-work behavior exactly enough
to pass checked-in TypeScript-oracle fixtures. The harness-spine cutover owns
future authority vocabulary changes; this spec only prevents Rust drift while
the existing policy surface still exists.

## Context

CWD: `.`

Authoritative TypeScript surfaces:
- `packages/core/src/policy/authority-proof.ts`
- `packages/core/src/policy/public-work.ts`
- `packages/core/src/policy/index.ts`
- `packages/core/src/policy/index.test.ts`
- `packages/contracts/src/schemas/credentials.ts`

Rust parity surfaces:
- `crates/runx-core/src/policy.rs`
- `crates/runx-core/src/policy/types.rs`
- `crates/runx-core/src/policy/connected_auth.rs`
- `crates/runx-core/src/policy/scope.rs`
- `crates/runx-core/tests/policy_fixtures.rs`
- `scripts/generate-kernel-parity-fixtures.ts`
- `fixtures/kernel/policy/*.json`

Existing state:
- Rust already ports local admission, retry, graph-scope, sandbox admission,
  connected-auth requirement parsing, grant matching, and scope matching.
- Rust does not expose `buildLocalScopeAdmission`,
  `buildAuthorityProofMetadata`, `buildAuthorityProof`,
  `validateCredentialBinding`, `evaluatePublicPullRequestCandidate`,
  `evaluatePublicCommentOpportunity`, or `normalizePublicWorkPolicy`.
- Kernel fixtures are already generated from TypeScript and evaluated in Rust.
  This spec extends that mechanism; it does not introduce a new oracle system.

## Objectives

- Add Rust public-work policy helpers matching TypeScript output shapes.
- Add Rust authority-proof helpers matching TypeScript output shapes.
- Add fixture kinds for those helpers to `generate-kernel-parity-fixtures.ts`
  and the Rust fixture runner.
- Regenerate fixtures from the TypeScript oracle and validate Rust against the
  same JSON.
- Keep naming and schema shape exactly aligned with current TS contracts until
  the separate harness-spine hard cutover replaces them.

## Scope

In scope:
- `packages/core/src/policy/authority-proof.ts`
- `packages/core/src/policy/public-work.ts`
- Re-exports from `packages/core/src/policy/index.ts` for authority-proof and
  public-work behavior.
- Shared fixture coverage against the TypeScript oracle before Rust behavior
  is accepted.

Out of scope:
- Runtime adapters, provider calls, filesystem, subprocess, MCP, A2A, and CLI
  cutover.
- Harness-spine vocabulary replacement (`authority_proof` to harness
  authority, schema discriminator cleanup, or receipt-shape changes).
- New credential material resolution. Rust receives material refs in fixture
  input and only proves/hash-validates them like TypeScript does.
- Network/package publishing.

## Dependencies

- `rust-kernel-parity-fixtures`
- `rust-policy-parity`
- `runx-contracts` carries any typed JSON contracts needed by the Rust port.

## Acceptance

Profile: strict

Validation:
- [x] `v1` command - TypeScript oracle fixtures are current.
  - Command: `pnpm tsx scripts/generate-kernel-parity-fixtures.ts --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `v2` command - kernel fixture schemas validate.
  - Command: `pnpm tsx scripts/validate-kernel-fixture-schemas.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `v3` command - Rust policy fixtures pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core --test policy_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16
- [x] `v4` command - Rust policy unit/proptest suite passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17
- [x] `v5` command - Rust formatting and clippy pass for `runx-core`.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-core --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18
- [x] `v6` command - Rust core style guard passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19

## Phases

### Phase 1 - Fixture oracle extension

Goal: make the TypeScript oracle express the missing surface before writing
Rust behavior.

Tasks:
- Extend `scripts/generate-kernel-parity-fixtures.ts` imports and evaluator
  for:
  - `policy.buildLocalScopeAdmission`
  - `policy.buildAuthorityProofMetadata`
  - `policy.validateCredentialBinding`
  - `policy.evaluatePublicPullRequestCandidate`
  - `policy.evaluatePublicCommentOpportunity`
  - `policy.normalizePublicWorkPolicy`
- Add fixture cases covering:
  - no connected auth requested
  - active grant admitted
  - structural denial before grant resolution
  - no matching grant denial
  - full authority-proof metadata with sandbox and approval
  - credential binding allow and key denial cases
  - public PR block reasons
  - cold external PR comment welcome-signal denial
  - trust-recovery comment lane denial
- Regenerate fixtures with `pnpm tsx scripts/generate-kernel-parity-fixtures.ts`.

Exit criteria:
- New fixtures are deterministic, schema-valid, and sorted with the existing
  generator.

### Phase 2 - Rust type and helper surface

Goal: add typed Rust policy APIs without widening runtime behavior.

Tasks:
- Add authority-proof/public-work input and output types to
  `crates/runx-core/src/policy/types.rs`.
- Add `authority_proof.rs` for connected-auth scope admission, authority-proof
  metadata construction, credential material proof, sandbox summary, and
  credential binding validation.
- Add `public_work.rs` for public PR/comment policy normalization and
  decisions.
- Re-export the public functions and types from `crates/runx-core/src/policy.rs`.
- Keep private helpers private unless a fixture requires them as public API.

Exit criteria:
- The new Rust API mirrors TS JSON output names and omits `None` fields in the
  same places TypeScript prunes `undefined`.

### Phase 3 - Rust fixture runner extension

Goal: prove Rust behavior against the TS-generated fixtures.

Tasks:
- Add new `PolicyInput` variants in `crates/runx-core/tests/policy_fixtures.rs`.
- Include the new fixture files in the `FIXTURES` table.
- Map each fixture kind to the new Rust helper and compare JSON directly.

Exit criteria:
- `cargo test --manifest-path crates/Cargo.toml -p runx-core --test policy_fixtures`
  passes.

### Phase 4 - Verification and review

Goal: leave the parity slice green and reviewable.

Tasks:
- Run all validation commands.
- Fix implementation defects, not fixture expectations, when Rust diverges from
  the TypeScript oracle.
- Run Claude review and complete only if there are no blockers.

Exit criteria:
- All validations pass.
- Review gate is pass.

## Invariants

- TypeScript remains authoritative for fixture generation until a separate
  cutover spec changes ownership.
- This spec must not introduce compatibility aliases or new `.v2` contracts.
- This spec must not edit the harness-spine vocabulary spec or rename current
  TS contract fields.
- Rust must reject accidental output drift by comparing whole JSON fixture
  outputs, not selected fields.
- Credential material must never expose raw secret material in expected output;
  only `material_ref_hash` is emitted.
- Public-work policy reason strings must match TypeScript exactly.

## Risks

- Medium: authority-proof output contains optional nested records, so Rust can
  diverge by serializing `null` where TypeScript prunes `undefined`. Mitigated
  with direct JSON fixture comparison and `skip_serializing_if`.
- Medium: fixture generation can become circular if expectations are edited by
  hand. Mitigated by using the TypeScript generator as the only fixture writer.
- Low: public-work terminology still includes `lane` because it is current TS
  policy surface. That naming is out of scope here and must not be silently
  changed during parity.

## Planning Log

- 2026-05-17T00:00:00Z: Created as the explicit follow-up anchor for
  authority-proof and public-work parity that `rust-kernel-parity-fixtures`
  defers.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T03:59:56Z
Ended: 2026-05-19T04:03:35Z

Checks:
- path audit
  - Grounded in: code:crates/runx-core/src/policy.rs:1
  - Result: passed
  - Evidence: The current Rust policy module is already split into local,
- command audit
  - Grounded in: code:crates/runx-core/tests/policy_fixtures.rs:149
  - Result: passed
  - Evidence: Existing policy parity is validated by deserializing
- scope/migration audit
  - Grounded in: code:packages/core/src/policy/authority-proof.ts:141
  - Result: passed
  - Evidence: The spec is explicitly parity-only for current
- acceptance timing audit
  - Grounded in: code:scripts/generate-kernel-parity-fixtures.ts:103
  - Result: passed
  - Evidence: Fixtures are generated from TypeScript first, then schema
- rollback/repair audit
  - Grounded in: code:crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: TypeScript remains authoritative and this slice only adds Rust
- design challenge
  - Grounded in: code:packages/core/src/policy/public-work.ts:17
  - Result: passed
  - Evidence: The public-work API still contains `lane`, which is not the

Issues:
- none

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Reviewed the Rust authority-proof / public-work parity port against the TypeScript oracle. Acceptance evidence is recorded as passing for all six gates (fixture regen --check, schema validation, policy_fixtures.rs, cargo test, fmt/clippy, style check). No completion-blocking issues found. The port mirrors the TS surface accurately on every shape covered by the regenerated fixtures. I identified four parity edge cases that the fixtures do not exercise: (1) `normalize_values` falls back to defaults for explicitly-empty arrays where TS preserves the empty result, (2) the Rust `has_version_number` token split swallows hyphenated tokens like `"abc-1.2"` that the TS regex `\bv?\d+\.\d+` would still match, (3) `summarize_authority_sandbox` does not trim declaration-sourced strings while TS's `nonEmptyString` does, (4) Rust emits empty `{}` sub-objects (filesystem/network/runtime) where TS `pruneUndefined` removes them. None impact the recorded acceptance evidence; surfacing them so the harness-spine cutover can either add fixtures or accept the divergence explicitly.

Attack log:
- `acceptance_evidence`: Spec-compliance: confirm recorded acceptance gates (fixture regen --check, schema validation, policy_fixtures.rs, cargo test policy, fmt/clippy, style check) match the spec exit_code_zero contract -> clean (All six v1-v6 gates report pass; treating acceptance evidence as authoritative per provider instruction.)
- `ambient_drift`: Scope-drift: separate task-relevant changes (crates/runx-core/src/policy/{authority_proof.rs,public_work.rs,types.rs,policy.rs}, tests/policy_fixtures.rs, fixtures/kernel/policy/{authority-*,public-work-*}.json, generate-kernel-parity-fixtures.ts) from ambient drift (contracts, runtime fanout, docs, outbox tool, etc.) -> clean (Task-scoped diff is consistent with the spec; ambient changes are unrelated.)
- `crates/runx-core/src/policy/authority_proof.rs`: Parity audit vs packages/core/src/policy/authority-proof.ts for buildAuthorityProof, validateCredentialBinding, buildLocalScopeAdmission, summarizeAuthoritySandbox, credentialMaterialProof, redaction shape, scope ordering, and grant-reference reasoning -> finding (Found F3 (declaration values not trimmed) and F4 (empty sub-objects not pruned). Reason ordering, redaction shape, requested/credential_material outputs match.)
- `crates/runx-core/src/policy/public_work.rs`: Parity audit vs packages/core/src/policy/public-work.ts for normalize semantics, dependency-update detection regex, welcome-signal logic, and trust-recovery handling -> finding (Found F1 (normalize_values empty-array fallback) and F2 (has_version_number hyphen tokenization). Welcome-signal short-circuit, comment-without-welcome-signal predicate, and trust-recovery enumeration mirror TS.)
- `crates/runx-core/src/policy/connected_auth.rs`: Parity audit of connectedAuthRequirement, scope_allows wildcards, grant-reference matching, unique_strings ordering, and authority_kind validation -> clean (Reservation logic, falsy-string handling (truthy_string), and prefix-wildcard semantics match TS, including the `:*` edge case asserted in unit tests.)
- `crates/runx-core/tests/policy_fixtures.rs`: Convention/regression check: every new fixture is listed via include_str! (style check enforces coverage) and dispatch arms cover every policy.* kind in generate-kernel-parity-fixtures.ts -> clean (All new public-work and authority-proof fixture kinds are wired into the PolicyInput enum and FIXTURES list; style check's checkPolicyFixtureCoverage validates the include set.)
- `fixtures/kernel/schema/policy.schema.json`: Schema parity: confirm new fixture kinds (buildLocalScopeAdmission, buildAuthorityProofMetadata, validateCredentialBinding, evaluatePublicPullRequestCandidate, evaluatePublicCommentOpportunity, normalizePublicWorkPolicy) are admitted -> clean (Each new kind is present as a const branch within the input oneOf.)
- `scripts/generate-kernel-parity-fixtures.ts`: Generator coverage: new policy.* cases dispatch to the correct TS oracle helpers with the expected payload shape -> clean (dispatch arms (evaluateKernelFixtureInputUnchecked) hit normalizePublicWorkPolicy / evaluatePublicPullRequestCandidate / evaluatePublicCommentOpportunity / buildAuthorityProofMetadata / validateCredentialBinding / buildLocalScopeAdmission with the matching fixture-case shapes.)

Findings:
- [medium/non-blocking] `F1-normalize-empty-array-fallback` normalize_values applies fallback to explicitly-empty arrays, diverging from TS which preserves []
  - Location: `crates/runx-core/src/policy/public_work.rs:239`
  - Evidence: Rust `normalize_values` returns `fallback.to_vec()` whenever `values.is_empty()` (line 240-242). TS `normalizeValues` in packages/core/src/policy/public-work.ts:152-156 only falls back when `Array.isArray(values)` is false (i.e. the field is undefined); an explicit empty array maps to []. A TS caller can opt out of defaults by passing `{ blocked_author_patterns: [] }` and the resulting policy will block nothing; the Rust port will instead reinstate the full 6-pattern default. Trim-then-filter inputs like `["   "]` collapse to [] inside both paths, hitting the same divergence by a second route.
  - Impact: Any TS caller that sets a public-work field to `[]` to disable a category of blocking will see different normalized output between TS and Rust. The fixtures do not cover the empty-array case, so the test gate is silent on this divergence.
  - Validation: Add a fixture `public-work-normalizes-empty-arrays` that calls `policy.normalizePublicWorkPolicy` with each blocked_* field set to []; regenerate from the TS oracle and confirm Rust matches.
- [low/non-blocking] `F2-version-number-hyphen-token` has_version_number tokenizes on hyphens differently than the TS regex \bv?\d+\.\d+
  - Location: `crates/runx-core/src/policy/public_work.rs:221`
  - Evidence: Rust `has_version_number` splits on chars that are neither alphanumeric nor `.` nor `-` (line 223). For the title `"upgrade abc-1.2"` the token list becomes ["upgrade", "abc-1.2"]; stripping a leading `v` leaves "abc-1.2" whose first dot-segment is "abc-1", failing `digits()`. TS regex `\bv?\d+\.\d+` matches `1.2` at the word boundary between `-` and `1` and would mark this as a dependency-update title.
  - Impact: PR titles of the form `<word>-N.M` (rare but possible for monorepo path/version mashups) are classified differently. Both `has_update_verb` and `has_version_number` must return true to add the `dependency_update_pull_request` reason, so the divergence only surfaces when both conditions otherwise hold.
  - Validation: Add a fixture exercising `evaluatePublicPullRequestCandidate` with `title: 'upgrade abc-1.2'` and confirm Rust matches the TS output.
- [low/non-blocking] `F3-sandbox-declaration-not-trimmed` summarize_authority_sandbox uses declaration strings verbatim while TS trims them via nonEmptyString
  - Location: `crates/runx-core/src/policy/authority_proof.rs:358`
  - Evidence: In packages/core/src/policy/authority-proof.ts:314-336 every declaration-sourced string flows through `nonEmptyString` which trims and rejects whitespace. The Rust equivalent (authority_proof.rs:364-394) only trims values pulled from the metadata record (`string_field` trims) but consumes `declaration.profile`, `declaration.cwd_policy`, etc. unchanged via `.clone()`. A declaration like `{ profile: '  workspace-write  ' }` would surface unchanged in the Rust authority proof's `sandbox.profile`, where TS would emit `'workspace-write'`.
  - Impact: Authored declarations with stray whitespace produce different `authority_proof.sandbox.profile` / `cwd_policy` values between the TS oracle and Rust, breaking receipt parity for that subset of inputs. Fixtures use clean strings, so this is not caught.
  - Validation: Add a fixture whose `sandboxDeclaration.profile` includes leading/trailing whitespace and confirm Rust's output trims it like TS.
- [low/non-blocking] `F4-empty-sandbox-subobjects-not-pruned` Rust emits empty sandbox sub-objects ({}) where TS pruneUndefined removes them
  - Location: `crates/runx-core/src/policy/authority_proof.rs:410`
  - Evidence: TS `pruneUndefined` (authority-proof.ts:450-474) drops any record key whose value reduces to `{}`. The Rust `summarize_filesystem` and `summarize_runtime` always return `Some(struct)` when the metadata field is present, even if all inner values are None/empty. With `#[serde(skip_serializing_if = "Option::is_none")]` the serialized result becomes `"filesystem": {}` (or similar) where TS would have omitted the key entirely. The same applies to `summarize_network` when only `declaration.network` is present but everything else is missing.
  - Impact: Inputs whose sandbox metadata has present-but-empty `filesystem`/`runtime`/`network` blocks produce divergent authority-proof JSON between Rust and TS. The fixtures only cover the fully-populated case (authority-proof-metadata-full.json), so this slips past the gate.
  - Validation: Add a fixture with `sandboxMetadata.filesystem: {}` and confirm Rust output omits `filesystem` like TS.
