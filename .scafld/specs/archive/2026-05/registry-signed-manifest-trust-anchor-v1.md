---
spec_version: '2.0'
task_id: registry-signed-manifest-trust-anchor-v1
created: '2026-05-25T17:51:35+10:00'
updated: '2026-05-26T03:48:19Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# registry-signed-manifest-trust-anchor-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-26T03:48:19Z
Review gate: pass

## Summary

Skill install must verify a digest from a trusted registry or publisher-signed
manifest, not a digest asserted by the downloaded candidate itself. This is a
clean cutover: no legacy self-asserted digest path remains for trusted installs.

## Scope

In scope:
- Registry manifest shape for skill digest, signer identity, key id, and
  signature.
- `runx-runtime` registry install verification.
- CLI install behavior and error messages.
- Fixtures for trusted, tampered, unsigned, and mismatched-manifest installs.
- Production trust-anchor runtime APIs are verifier-only: no exported signing
  seed, local signer, or compatibility alias can participate in trusted install
  acceptance.

Out of scope:
- Marketplace curation policy and trust-tier assignment.
- Remote key transparency infrastructure beyond the minimal trusted key set
  needed for this cutover.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Trusted install requires a registry/publisher-signed manifest.
- [x] `dod2` Candidate-supplied digests are never accepted as the trust anchor.
- [x] `dod3` Tampered content and mismatched manifest digests fail closed.
- [x] `dod4` CLI output clearly distinguishes unsigned, unknown-key, invalid
  signature, and digest mismatch failures.
- [x] `dod5` Production registry trust-anchor code exports verification only;
  local/private signing material is absent from trusted runtime and CLI install
  acceptance paths.

Validation:
- [x] `v1` runtime registry install tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test registry_install`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit 0
  - Status: passed
  - Evidence: 8 passed; covers trusted signed install, tampered content digest
    mismatch, unsigned manifest fail-closed, unknown key, invalid signature, and
    mismatched or missing manifest identity, plus delimiter injection in
    signed-manifest payload fields.
  - Source event: none
  - Last attempt: 2026-05-26T03:03:12Z
  - Checked at: 2026-05-26T03:03:12Z
- [x] `v2` CLI registry install tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test registry`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit 0
  - Status: passed
  - Evidence: 2 passed; CLI test proves typed install errors for
    unsigned_manifest, unknown_key, invalid_signature, and digest_mismatch, and
    proves install succeeds only after a trusted signed manifest fixture is
    present.
  - Source event: none
  - Last attempt: 2026-05-26T03:03:12Z
  - Checked at: 2026-05-26T03:03:12Z
- [x] `v3` self-asserted digest path removed
  - Command: `rg -n "candidate\\.digest|validate_candidate_digest" crates/runx-runtime/src crates/runx-cli/src`
  - Expected kind: `no_matches`
  - Timeout seconds: none
  - Result: no matches
  - Status: passed
  - Evidence: no trusted install path accepts a candidate-supplied digest as the
    trust anchor.
  - Source event: none
  - Last attempt: 2026-05-26T03:03:12Z
  - Checked at: 2026-05-26T03:03:12Z
- [x] `v4` production registry signing path removed
  - Command: `rg -n "RegistryManifestSigningKey|sign_registry_manifest|RUNX_REGISTRY_MANIFEST_SIGNING|RUNX_REGISTRY_MANIFEST_SIGNER|manifest_signing_key|registry manifest signing key|SIGNING_SEED" crates/runx-runtime/src/registry crates/runx-runtime/src/registry.rs crates/runx-cli/src/registry.rs`
  - Expected kind: `no_matches`
  - Timeout seconds: none
  - Result: no matches
  - Status: passed
  - Evidence: production registry trust-anchor/runtime/CLI source exports and
    consumes verifier-only trust keys; local signing seed APIs and CLI signing
    envs are absent.
  - Source event: none
  - Last attempt: 2026-05-26T03:03:12Z
  - Checked at: 2026-05-26T03:03:12Z

Additional validation:
- `CARGO_TARGET_DIR=/tmp/runx-oss-focused-target cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test registry`
  - Result: exit 0; 5 passed.
- `CARGO_TARGET_DIR=/tmp/runx-oss-focused-target cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test registry_client`
  - Result: exit 0; 15 passed.
- `RUNX_RUST_CLI_BIN=/tmp/runx-oss-focused-target/debug/runx RUNX_KERNEL_EVAL_BIN=/tmp/runx-oss-focused-target/debug/runx pnpm vitest run tests/skill-add.test.ts tests/skill-add-profile-metadata.test.ts`
  - Result: exit 0; 12 passed.
- `rg -n "expectedDigest = options\\.expectedDigest \\?\\?|candidate\\.origin\\.digest|validate_candidate_digest|candidate\\.digest" packages/runtime-local/src crates/runx-runtime/src crates/runx-cli/src`
  - Result: no matches.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode reread of the Rust trust-anchor cutover confirms the implementation is sound and all prior-review findings have been addressed. (1) verify_signed_manifest_anchor (crates/runx-runtime/src/registry/install.rs:210-243) requires a signed manifest, performs Ed25519 verification, validates identity (skill_id+version), enforces markdown digest match, and compares any expected_digest before any filesystem write. (2) validate_manifest_identity (install.rs:298-329) NOW returns ManifestIdentityMissing when candidate.skill_id/version is None, repairing the prior low-severity candidate-identity-bypass-when-none finding. (3) The TS SDK installLocalSkill at packages/runtime-local/src/runner-local/skill-install.ts:92-104 NOW requires options.expectedDigest and throws "Trusted skill install requires an expected digest" if absent, removing the self-asserted-digest fallback flagged in the prior ts-sdk-self-asserted-digest-residual finding (the only remaining `candidate.origin.digest` matches are in gitignored dist/ build output). (4) verify_registry_signed_manifest (trust_anchor.rs:52-85) enforces schema/algorithm/payload-term checks (rejects \n, \r, =, and NUL in skill_id/version/digest/profile_digest/signer_id/key_id), binds the signature to the full payload tuple, and rejects malformed keys/signatures. (5) build_registry_skill_version (local/build.rs:43) hardcodes `signed_manifest: None` so locally published records cannot bootstrap a trusted install. (6) The public registry module (crates/runx-runtime/src/registry.rs:32-37) re-exports only verifier-side trust-anchor types — no signer, key-pair, or seed surface is reachable. (7) CLI install path (crates/runx-cli/src/registry.rs:184-219) delegates `skill add` to the native runx binary via packages/cli/src/dispatch.ts:246-263, applies default trust keys first, and re-maps install errors into the typed unsigned_manifest/unknown_key/invalid_signature/digest_mismatch kinds. (8) End-to-end tests in crates/runx-runtime/tests/registry_install.rs (8 cases) and crates/runx-cli/tests/registry.rs (2 cases) cover trusted install, tampered content, unsigned candidate, identity mismatch, missing identity, unknown key, invalid signature, malformed payload, and prove install dir is never created on failure. v3 (`candidate.digest|validate_candidate_digest` in crates/) and v4 (signing-symbol grep) both re-confirmed empty. The previous critical workspace_mutation blocker was a procedural artifact of the prior review session; this verify pass used only Read/Grep/Glob, no Write/Edit, and Ambient Workspace Drift is empty.

Attack log:
- `crates/runx-runtime/src/registry/trust_anchor.rs::verify_registry_signed_manifest`: Schema/algorithm substitution, payload term delimiter injection (\n, \r, =, NUL), unknown key_id, malformed base64 key/signature, signature length tampering -> clean (Schema mismatch -> UnsupportedSchema (line 56-58); alg != ed25519 -> UnsupportedAlgorithm (line 59-61); delimiter injection in any payload term -> MalformedPayload (lines 122-133); unknown key_id -> UnknownKey (line 66); key.public_key.len()!=32 -> MalformedKey (line 67-69); signature.len()!=64 or missing base64: prefix -> MalformedSignature (lines 70-73, 135-140); signature is verified over the full (schema, skill_id, version, digest, profile_digest, signer_id, key_id) payload so cross-skill/cross-version manifest reuse fails (line 74-84).)
- `crates/runx-runtime/src/registry/install.rs::verify_signed_manifest_anchor + validate_manifest_identity + validate_candidate_profile_digest`: Unsigned, identity-mismatched, identity-None (bypass), content-tampered, profile-mismatched, profile-missing, digest-mismatched install candidates -> clean (verify_signed_manifest_anchor (lines 210-243) requires signed_manifest.is_some() (UnsignedManifest), runs ed25519 verification, calls validate_manifest_identity (which now errors when candidate.skill_id/version is None — lines 298-329), checks markdown sha256 against manifest.digest (DigestMismatch), and compares any expected_digest against the manifest digest. validate_candidate_profile_digest (lines 259-296) rejects all 4 (Some/Some mismatch, Some/None, None/Some, plus matching) combinations. install_local_skill (lines 124-160) calls validate_install_candidate before install_paths/prepare_install_write_plan/commit_install_write_plan, so no filesystem write happens before validation succeeds. CLI test asserts !install_dir.exists() on all 4 failure modes.)
- `crates/runx-cli/src/registry.rs::trusted_manifest_keys_from_env + InstallError mapping`: Env-key precedence override, orphan KEY_ID env var, error kind mapping -> clean (trusted_manifest_keys_from_env (lines 579-597) loads default keys first via default_trusted_registry_manifest_keys, only appends an env key when BOTH KEY and KEY_ID env vars are set, and returns usage_error for orphan KEY_ID. verify_registry_signed_manifest's `.find()` returns the first matching key_id, so env-injected key_id collisions with the default cannot override the default. InstallError mapping (lines 674-690) re-maps to stable kinds unsigned_manifest, unknown_key, invalid_signature, digest_mismatch; CLI test registry_install_reports_typed_trust_anchor_errors confirms all 4 kinds and that no install dir is created.)
- `crates/runx-runtime/src/registry/local/build.rs::build_registry_skill_version + crates/runx-runtime/src/registry.rs (public re-exports)`: Hunt for any local signer, key-pair generation, or compatibility alias that could mint a trusted-install signed_manifest -> clean (build_registry_skill_version (line 43) hardcodes `signed_manifest: None`; normalize_registry_skill_version (line 181) passes through payload.signed_manifest from the registry JSON only. The full v4 grep target list (RegistryManifestSigningKey, sign_registry_manifest, RUNX_REGISTRY_MANIFEST_SIGNING, RUNX_REGISTRY_MANIFEST_SIGNER, manifest_signing_key, registry manifest signing key, SIGNING_SEED) returned zero matches across crates. The public registry module re-exports only verifier-side types: TrustedRegistryManifestKey, default_trusted_registry_manifest_keys, verify_registry_signed_manifest, RegistryManifestKeyError, RegistryManifestVerificationFailure, REGISTRY_SIGNED_MANIFEST_SCHEMA, and env-key constants — no signer or seed surface.)
- `v3/v4 acceptance greps re-execution`: Confirm self-asserted-digest and signing-key residuals remain absent in production source -> clean (rg `candidate\.digest|validate_candidate_digest` across crates/runx-runtime/src + crates/runx-cli/src returned zero matches. rg of all v4 signing-related symbols returned zero matches. Only `candidate.origin.digest` matches in packages/{core,runtime-local}/dist/ which are gitignored build artifacts.)
- `packages/runtime-local/src/runner-local/skill-install.ts::installLocalSkill`: Re-check whether the TS SDK retains the self-asserted candidate.origin.digest fallback flagged in the prior review -> clean (Line 92-104 now reads `const expectedDigest = normalizeExpectedDigest(options.expectedDigest)` and throws when expectedDigest is undefined, removing the prior `options.expectedDigest ?? candidate.origin.digest` fallback. SDK addSkill (packages/runtime-local/src/sdk/index.ts:488-504) passes options.expectedDigest directly to installLocalSkill; the caller is now responsible for supplying a verified digest. ts-sdk-self-asserted-digest-residual is recorded as fixed.)
- `crates/runx-runtime/src/registry/install.rs::validate_manifest_identity`: Construct InstallCandidate with skill_id=None or version=None to bypass the cross-check against the signed manifest -> clean (Both None branches now return InstallError::ManifestIdentityMissing (lines 302-308, 315-320). The candidate-identity-bypass-when-none finding from the prior review is recorded as fixed; missing_manifest_identity_fails_closed test (registry_install.rs:86-102) confirms behavior.)
- `crates/runx-runtime/src/registry/install.rs::install_local_skill (write ordering)`: Filesystem side effects before validation completes -> clean (install_local_skill (lines 124-160) runs validate_install_candidate (which performs all signed-manifest checks) before install_paths/prepare_install_write_plan/commit_install_write_plan. prepare_install_write_plan only reads with read_optional (no writes); commit_install_write_plan is gated on a successful validation result. CLI test asserts !install_dir.exists() after every failure mode.)
- `workspace mutation guard`: Re-check whether the previous review's mutation blocker recurs in a read-only verify pass -> clean (This review used only Read, Grep, and Glob; no Write or Edit invocations. The previously-mutated scripts/check-boundaries.mjs is now part of the Task Changes Since Approval Baseline section (sha 29c5525012b5ed6c3820b99d6fc7468af1230eadbdcb396bcc30feda0d49d264) rather than mid-review drift, and Ambient Workspace Drift Outside Task Scope is empty. The prior workspace_mutation blocker is recorded with status=fixed/blocks_completion=false.)

Findings:
- [medium/non-blocking] `ts-sdk-self-asserted-digest-residual` TS SDK installLocalSkill no longer accepts candidate.origin.digest as the expected digest; the self-asserted fallback has been removed.
  - Location: `packages/runtime-local/src/runner-local/skill-install.ts:92`
  - Evidence: packages/runtime-local/src/runner-local/skill-install.ts:91-104 now reads `const expectedDigest = normalizeExpectedDigest(options.expectedDigest);` and immediately throws `Trusted skill install requires an expected digest for ${options.ref}; use the native runx registry install path for signed-manifest verification.` when expectedDigest is undefined, before any digest comparison. The prior `options.expectedDigest ?? candidate.origin.digest` fallback only persists in gitignored dist/ build output (packages/runtime-local/dist/src/runner-local/skill-install.js:10, packages/core/dist/src/runner-local/skill-install.js:11). The user-facing `runx skill add` path still routes through the Rust binary via packages/cli/src/dispatch.ts:246-263.
  - Impact: SDK embedders can no longer install skills with a digest asserted by the downloaded candidate. The trust responsibility is shifted to the caller, who must supply an expectedDigest from a verified source.
  - Validation: Source grep confirms no live `candidate.origin.digest` fallback in packages/*/src; only dist/ build output retains the old expression and dist/ is in .gitignore (line 14).
- [low/non-blocking] `candidate-identity-bypass-when-none` validate_manifest_identity now returns ManifestIdentityMissing when candidate.skill_id or candidate.version is None, closing the prior None-bypass path.
  - Location: `crates/runx-runtime/src/registry/install.rs:298`
  - Evidence: crates/runx-runtime/src/registry/install.rs:298-329 — `let Some(skill_id) = &candidate.skill_id else { return Err(InstallError::ManifestIdentityMissing { ref_name: candidate.r#ref.clone(), field: "skill_id" }); };` and the same for `version`. A new InstallError::ManifestIdentityMissing variant (lines 96-100) is wired through. The missing_manifest_identity_fails_closed test (crates/runx-runtime/tests/registry_install.rs:86-102) exercises the None case and asserts the install directory is not created.
  - Impact: Library callers can no longer skip the skill_id/version cross-check by constructing InstallCandidate with None identity fields. Combined with the still-valid signature-binds-content invariant, this closes the prior label-confusion risk surface.
  - Validation: Test missing_manifest_identity_fails_closed asserts InstallError::ManifestIdentityMissing with field="skill_id" and that no skills/ directory is created.
- [critical/non-blocking] `workspace_mutation` Prior review's workspace-mutation blocker did not recur in this read-only verify pass.
  - Location: `scripts/check-boundaries.mjs`
  - Evidence: This verify pass invoked only Read, Grep, and Glob — no Write or Edit. The previously mutated scripts/check-boundaries.mjs is now incorporated into the Task Changes Since Approval Baseline section (sha 29c5525012b5ed6c3820b99d6fc7468af1230eadbdcb396bcc30feda0d49d264), and the Ambient Workspace Drift Outside Task Scope section is empty. The underlying trust-anchor code (verify_registry_signed_manifest, install_local_skill, build_registry_skill_version, public registry re-exports, CLI install wiring) has been re-audited adversarially against the source.
  - Impact: The procedural concern (untrustworthy verdict due to mid-review drift) no longer applies. The verify pass produced a clean, evidence-grounded verdict on the trust-anchor implementation.
  - Validation: No tool calls in this review wrote to disk; the read-only contract was preserved.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-26T02:39:49Z
Ended: 2026-05-26T02:41:12Z

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/registry/trust_anchor.rs:42
  - Result: passed
  - Evidence: The runtime trust-anchor file currently owns both verification and
- command audit
  - Grounded in: spec_gap:validation
  - Result: passed
  - Evidence: Validation includes focused runtime install, CLI install, and
- scope/migration audit
  - Grounded in: code:crates/runx-cli/src/registry.rs:580
  - Result: passed
  - Evidence: CLI local publish currently reads process signing seed env vars;
- acceptance timing audit
  - Grounded in: code:crates/runx-runtime/tests/registry_install.rs:17
  - Result: passed
  - Evidence: Existing focused tests already exercise trusted install,
- rollback/repair audit
  - Grounded in: code:crates/runx-runtime/src/registry/install.rs:145
  - Result: passed
  - Evidence: Install verification runs before write planning/commit, so failed
- design challenge
  - Grounded in: code:crates/runx-runtime/src/registry/local/build.rs:85
  - Result: passed
  - Evidence: Local registry publish currently mints signed manifests from

Issues:
- none
