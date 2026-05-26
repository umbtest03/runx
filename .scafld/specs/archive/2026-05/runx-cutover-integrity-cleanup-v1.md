---
spec_version: '2.0'
task_id: runx-cutover-integrity-cleanup-v1
created: '2026-05-26T00:00:00+10:00'
updated: '2026-05-26T01:50:48Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# runx-cutover-integrity-cleanup-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-26T01:50:48Z
Review gate: pass

## Summary

Finish the hard cutover cleanup without compatibility shims:

- Existing Postgres databases must get the new provider-reference table through
  a forward migration; editing bootstrap state is not enough.
- Hosted receipt admission must reject malformed proof, unsigned source-event
  receipts, path-shaped receipt body writes, and index/body id mismatches.
- Cloud auth and claim code must stop leaking Nango naming outside the Nango
  adapter boundary.
- Active plans must stop teaching `connection_id` and `/connections`.
- Rust supply-chain and clippy gates must return green.

## Scope

In scope:
- `cloud/packages/db/migrations` and migration tests.
- `cloud/packages/receipts-store` receipt admission/indexing.
- `cloud/packages/api` source-event receipt building, receipt provenance, native
  runtime outcome parsing, claim-session naming.
- `cloud/packages/auth` Nango adapter boundary and generic connect/revoke
  naming.
- Active `plans/` vocabulary references for provider references.
- OSS Rust clippy/license-policy fixes already present in the dirty tree.

Out of scope:
- A new runtime receipt shape.
- Runtime compatibility aliases.
- Replaying production provider data.
- Renaming product skill names such as `issue-to-pr`.

## Invariants

- No runtime compatibility path for old `connection_id`/`runx.harness_receipt`
  shapes.
- One-time migration references to old DB tables are allowed only to perform the
  hard cutover and must not expose runtime dual-read behavior.
- Nango-specific names are allowed in the concrete Nango adapter, raw provider
  wire tests, and provider API paths only.
- Receipt admission must fail closed when digest/signature binding is missing.
- Existing dirty files may be edited, but unrelated concurrent changes must not
  be reverted.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Existing DBs get `provider_references` through a numbered forward
  migration, not by relying on edited `0001_bootstrap.sql`.
- [x] `dod2` Hosted receipt stores verify canonical receipt body digest, reject
  unsigned/pending signatures, reject id/body mismatches, and keep digest-named
  body objects.
- [x] `dod3` Source-event receipts bind `digest` and `signature.value` to the
  canonical receipt body.
- [x] `dod4` Native runtime service output does not treat missing closure as
  success.
- [x] `dod5` Generic cloud auth/claim surfaces do not expose Nango naming.
- [x] `dod6` Active plans no longer use `connection_id` or `/connections` as the
  runx contract vocabulary.
- [x] `dod7` Rust supply-chain and clippy gates are green for the touched
  workspace.

Validation:
- [x] `v1` cloud receipt/auth focused tests
  - Command: `pnpm --dir cloud exec vitest run packages/receipts-store/src/index.test.ts packages/receipts-store/src/postgres.test.ts packages/auth/src/connect.test.ts packages/auth/src/connect-http.test.ts packages/auth/src/hosted-provider-credential.test.ts packages/api/src/claim-service.test.ts packages/api/src/claim-session-stores.test.ts packages/api/src/index.test.ts`
  - Expected kind: `exit_code_zero`
  - Result: passed, 8 files / 69 tests.
  - Status: passed
- [x] `v2` cloud typecheck
  - Command: `pnpm --dir cloud typecheck:server`
  - Expected kind: `exit_code_zero`
  - Result: passed.
  - Status: passed
- [x] `v3` DB migration test
  - Command: `pnpm --dir cloud exec vitest run packages/db/src/migrations.test.ts`
  - Expected kind: `exit_code_zero`
  - Result: passed, 1 file / 7 tests.
  - Status: passed
- [x] `v4` active legacy vocabulary grep
  - Command: `rg -n "connection_id|connectionId|NangoConnection|nango_end_user_id|nangoClient|runx\\.harness_receipt|harness_receipt|closure_contract|act_closure" cloud/packages/auth/src cloud/packages/api/src cloud/packages/db/migrations plans oss/packages oss/crates --glob '!**/archive/**' --glob '!**/target/**' --glob '!**/node_modules/**'`
  - Expected kind: `reviewed_output`
  - Result: reviewed. Remaining hits are the one-time `0022` hard-cutover SQL,
    raw Nango provider-wire parsing/tests, and negative legacy contract tests.
  - Status: passed
  - Evidence: no runtime dual-read or generic cloud Nango naming remains.
- [x] `v5` Rust schema compatibility
  - Command: `cargo test --manifest-path oss/crates/Cargo.toml -p runx-contracts --test schema_wire_compat`
  - Expected kind: `exit_code_zero`
  - Result: passed, 3 tests.
  - Status: passed
- [x] `v6` Rust clippy
  - Command: `cargo clippy --manifest-path oss/crates/Cargo.toml --workspace --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Result: passed.
  - Status: passed
- [x] `v7` Rust supply-chain policy
  - Command: `cargo deny --manifest-path oss/crates/Cargo.toml check bans advisories licenses`
  - Expected kind: `exit_code_zero`
  - Result: passed: advisories ok, bans ok, licenses ok.
  - Status: passed

Additional validation:
- [x] `v8` provider-auth renamed integration tests
  - Command: `pnpm --dir cloud exec vitest run tests/provider-auth.test.ts tests/runx-connect-http-nango-webhook.test.ts`
  - Expected kind: `exit_code_zero`
  - Result: passed, 2 files / 6 tests.
  - Status: passed
- [x] `v9` post-review finding regression tests
  - Commands:
    - `rg -n "\\bNango\\b" cloud/packages/api/src/claim-service.ts`
    - `pnpm --dir cloud exec vitest run packages/auth/src/agent-keys.test.ts packages/db/src/migrations.test.ts`
    - `pnpm --dir cloud typecheck:server`
  - Expected kind: `reviewed_output`
  - Result: passed. The grep returns no matches, the two focused test files pass
    10 tests, and server typecheck passes.
  - Status: passed

## Review Gate

Run `scafld review runx-cutover-integrity-cleanup-v1 --provider claude` if
available. If not available, use `--provider command` with a read-only review
script that replays the validation commands and audits the changed files for the
invariants above. Do not complete on a local-only review.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-26T00:48:13Z
Ended: 2026-05-26T00:49:01Z

Checks:
- path audit
  - Grounded in: spec_gap:Scope
  - Result: passed
  - Evidence: Receipt bodies now use digest object names; this spec adds id/body
- command audit
  - Grounded in: spec_gap:Acceptance
  - Result: passed
  - Evidence: Acceptance lists focused cloud tests, DB migration tests, Rust
- scope/migration audit
  - Grounded in: spec_gap:Summary
  - Result: passed
  - Evidence: The migration runner skips already-applied filenames, so the DB
- acceptance timing audit
  - Grounded in: spec_gap:Acceptance
  - Result: passed
  - Evidence: Native runtime closure parsing is validated with focused cloud
- rollback/repair audit
  - Grounded in: spec_gap:Invariants
  - Result: passed
  - Evidence: The one-time migration can be reverted as a schema migration
- design challenge
  - Grounded in: spec_gap:Summary
  - Result: passed
  - Evidence: Source-event receipts must bind digest and signature to the

Issues:
- none

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Discover-mode read-only review of runx-cutover-integrity-cleanup-v1. Inspected all in-scope cloud packages (db/migrations, receipts-store, auth, api source-event/runtime-service code) plus plans. Migration 0022 forward-creates provider_references and drops the legacy connections table for both fresh and pre-existing DBs (matching tests in packages/db/src/migrations.test.ts at line 51 and line 157). Hosted receipt admission verifies canonical digest, rejects sig:pending/unsigned signatures, enforces id/body match, hashes receipt ids into digest-named object files, and rejects path-shaped writes (receipt-document.ts:127-133, 214-242; index.test.ts:54-69, 71-95, 97-109). Source-event receipts compute and bind both digest and signature.value (sig:${digest}) to the canonical body in source-events.ts:230-238. runtime-service-client.ts:283-298 fails the host when sealed payloads are missing a closure. Auth connect/revoke surface (connect.ts, grant-revocation.ts, provider-credential-broker.ts) is fully generic; the only remaining Nango references are inside hosted-provider-credential.ts (the concrete Nango adapter) and seed/catalog data (allowed by the invariants). Plans no longer mention connection_id or /connections. No completion blockers found.

Attack log:
- `cloud/packages/db/migrations/0022_provider_references_hard_cutover.sql`: Verify forward migration covers existing DBs with legacy connections table and is idempotent on fresh DBs via if-not-exists guards -> clean (Standalone-idempotent: creates provider_references + indexes if missing, copies rows from connections via on-conflict upsert, drops connections. shouldSkipMigrationSql in postgres.ts:2094-2099 short-circuits the SQL on fresh DBs where provider_references already exists and connections does not. Both fresh and legacy paths covered by migrations.test.ts (line 51 and 157).)
- `cloud/packages/receipts-store/src/receipt-document.ts`: Try to bypass digest/signature binding (sig:pending, unsigned, mismatched digest, wrong alg, empty/whitespace signature, upper-case UNSIGNED) -> clean (validateReceiptProofBinding rejects mismatched digest (line 221), non-Ed25519 alg (line 227), sig:pending/unsigned signatures (line 229), requires sig:${digest} when no trusted keys configured (line 239), and otherwise requires Ed25519 signature verification (line 234). requireText trims so whitespace-only signatures are rejected. UNSIGNED still fails the trailing equality check.)
- `cloud/packages/receipts-store/src/index.ts + postgres.ts (FileReceiptBodyStore, S3ReceiptBodyStore)`: Submit path-shaped or absolute receipt id and look for path traversal or arbitrary file write -> clean (hostedReceiptBodyObjectName sha256s the receipt id into a fixed `${hex}.json` filename, so any path-shaped id resolves to a hash filename inside the bodies/ root. Verified by index.test.ts:54-69 and postgres.test.ts:17-28. Body ref is system-controlled and persisted to receipts_index via parameterized inserts.)
- `cloud/packages/receipts-store/src/index.ts getReceipts`: Force an id/body mismatch where a manifest entry points to a body whose document id differs -> clean (assertHostedReceiptId (receipt-document.ts:135-142) is invoked on every body load (FileHostedReceiptStore line 130, PostgresHostedReceiptStore line 131); test index.test.ts:97-109 confirms mismatched bodies are skipped.)
- `cloud/packages/api/src/source-events.ts buildReceiptFromSourceEvent`: Look for digests/signatures that escape canonicalization or that leave sig:pending in persisted receipts -> clean (Draft uses sig:pending then computes canonicalReceiptBodyDigest(receiptDraft) and replaces signature.value with sig:${digest} before validateReceiptContract (lines 230-239). The persisted receipt always carries a matching digest/signature pair.)
- `cloud/packages/api/src/runtime-service-client.ts sealedOutcome`: Trigger sealed runtime output with missing closure or closure with non-closed disposition; check it is not treated as success -> clean (Lines 283-298 build a failedHost('native skill sealed payload is missing closure') when closure is missing; lines 299-314 fail when closure.disposition !== 'closed'. Both branches return kernelStatus 'failure' and an error, preventing accidental success.)
- `cloud/packages/auth/src + cloud/packages/api/src/claim-service.ts`: Grep for Nango naming leaks outside the concrete Nango adapter boundary -> clean (Remaining `Nango` hits are confined to hosted-provider-credential.ts (the concrete adapter, allowed) and api seed/catalog data (public-integrations.seed.ts plus a single `*.nango.dev` hostname matcher in public-integrations.ts). connect.ts, grant-revocation.ts, provider-credential-broker.ts, and claim-service.ts use generic provider/providerReference vocabulary. Validation step v9 already grepped claim-service.ts for `\bNango\b` and found none.)
- `plans/`: Grep for connection_id or /connections in active plans -> clean (No matches under plans/ for either token; only legacy adapter/test files and the one-time 0022 SQL still contain the legacy vocabulary.)

Findings:
- none

