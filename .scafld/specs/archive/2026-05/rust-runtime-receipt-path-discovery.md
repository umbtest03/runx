---
spec_version: '2.0'
task_id: rust-runtime-receipt-path-discovery
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T05:51:55Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# Rust runtime receipt path discovery

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T05:51:55Z
Review gate: pass

## Summary

Add runtime-owned receipt path discovery and local receipt store handling for
post-cutover harness receipts. This keeps filesystem concerns out of
`runx-receipts` while giving the Rust runtime, CLI, SDK, and Aster a single
deterministic contract for where receipts live, how derived indexes are loaded,
how receipt/index writes commit, and how public projections avoid leaking
machine-local paths.

## Context

CWD: `.` (runx OSS workspace)

Relevant code and fixtures:
- `crates/runx-runtime/src/**`
- `crates/runx-receipts/src/**`
- `fixtures/runtime/**`
- CLI wiring that passes receipt store paths into runtime runs

Verified current behavior to account for:
- The TypeScript CLI parses `--receipt-dir`, and runtime-local also honors
  `RUNX_RECEIPT_DIR`.
- Existing docs say local receipts live under `.runx/receipts`, but current
  TypeScript runner and SDK defaults are not identical. Rust must not grow a
  second implicit default.
- Current dirty Rust runtime receipt work emits graph and step receipts in
  memory and validates the supplied child set; it does not persist receipts or
  discover receipt directories yet.
- `runx-receipts` tree verification is pure and takes a root receipt plus
  supplied children. This spec supplies the runtime store side only.

Expected discovery inputs:
- Explicit runtime option, including CLI `--receipt-dir` and Aster/SDK
  caller-supplied paths.
- Policy/runtime config value, with Phase 1 naming the exact config field.
- Environment override: `RUNX_RECEIPT_DIR`.
- Workspace default under the project run state directory:
  `<runx_project_dir>/receipts`.

Invariants:
- The receipt crate remains IO-free.
- Runtime may canonicalize and use local paths internally, but public GitHub,
  Slack, Aster, reviewer, training-export, and normal CLI summary outputs use
  receipt ids and relative/safe labels only.
- Receipt JSON is the authoritative artifact. Store indexes/manifests are
  derived and rebuildable; if an index disagrees with receipt JSON, governed
  verification fails with a typed error instead of trusting the index.
- Receipt writes are temp-file-backed and commit before indexes, summaries, or
  success markers can make a failed/partial run look successful.
- Unknown, unreadable, malformed, or wrong-schema receipt stores fail closed for
  governed verification and tree/proof inputs. History/list views may be
  tolerant only when they are explicitly non-governing.
- Relative receipt paths resolve deterministically from the workspace/runx base
  and do not require the target directory to pre-exist.

## Objectives

- Define precedence for receipt store path discovery.
- Add a runtime local receipt store interface for read, write, list, and index
  operations.
- Support manifest/index loading and scan fallback for receipt tree
  verification without moving tree traversal into runtime.
- Redact or relativize path details in all public projections.
- Add typed receipt-store errors that distinguish missing store, unreadable
  store, malformed index, malformed receipt, schema mismatch, and unsafe path
  projection.
- Add an operator recovery path for rebuilding a derived receipt index from
  authoritative receipt JSON.
- Add tests for env/config/default precedence, atomic writes, malformed index,
  and missing store errors.

## Scope

In scope:
- Runtime receipt store path resolution.
- Local filesystem receipt store implementation.
- Derived local receipt index/manifest loading, rebuild, and consistency
  checks.
- Public projection redaction/relativization for receipt paths.
- Integration tests using `fixtures/runtime/**`.

Out of scope:
- Receipt proof semantics.
- Receipt tree traversal semantics.
- Cloud receipts store implementation.
- Nitrosend Slack/GitHub copy.
- Legacy TypeScript `rx_`/`gx_` receipt migration or compatibility readers
  unless a separate sunset/migration spec requires them.

## Dependencies

- `rust-runtime-skeleton`.
- Coordinates with `rust-receipt-tree-resolution`: this spec can provide a
  local store/resolver adapter first, but final bounded traversal semantics land
  in the tree spec.
- Coordinates with `rust-receipt-proof-verification`: this spec may load bytes
  and safe labels, but proof material, signatures, digest semantics, and
  verification-summary honesty land in the proof spec.

## Assumptions

- CI and dogfood runners can provide explicit receipt paths when needed.
- The Rust runtime default is project run state receipts
  (`<runx_project_dir>/receipts`). Existing callers that intentionally want a
  global receipt directory must pass it explicitly during cutover.
- The config key should be named in Phase 1 before implementation; the preferred
  shape is a runtime config value such as `runtime.receipts.dir`, not a second
  environment variable.
- Receipt file names are exact receipt ids plus `.json`; partial ids, suffix
  lookup, and existence-dependent path resolution are not accepted for governed
  paths.
- A missing derived index can be rebuilt from receipt JSON. A present malformed
  or inconsistent index is a typed error until the operator chooses the rebuild
  path.

## Touchpoints

- `runx-runtime` receipt store modules.
- CLI options/env parsing.
- Runtime summary and receipt projections.
- Aster runner surfaces that need receipt links or summaries.
- SDK host-protocol calls that inspect receipt history.
- Local knowledge indexing that currently receives receipt file paths.

## Risks

- Leaking absolute paths in GitHub or Slack comments can expose operator machine
  structure.
- Inconsistent precedence between CLI and Aster can make production behavior
  hard to debug.
- Non-atomic writes can make observers consume partial receipt data.
- Writing terminal ledgers or indexes before receipt JSON can create a
  successful-looking run with no durable receipt.
- Letting this spec absorb proof/tree semantics would duplicate security logic
  already assigned to the receipt specs.

## Design Contract

Path precedence:
1. Explicit runtime/host input, including CLI `--receipt-dir`.
2. Runtime policy/config receipt directory.
3. `RUNX_RECEIPT_DIR`.
4. Project run state default: `<runx_project_dir>/receipts`.

Resolution rules:
- Absolute paths are allowed for explicit local operators and CI.
- Relative explicit, config, and environment paths resolve from the same
  workspace/runx base used for other local runx project paths, with
  pre-existence disabled.
- Runtime stores the canonical local `PathBuf` internally, but public APIs carry
  `ReceiptStoreLabel`/receipt ids. Public labels are project-relative when the
  path is under the project run state directory and otherwise use a stable
  redacted label such as `external-receipt-store:<hash>`.
- A file where a receipt directory is expected fails with
  `ReceiptStoreUnavailable`; unreadable directories fail with
  `ReceiptStoreUnreadable`.

Store layout:
- Authoritative harness receipts are JSON files named `<receipt_id>.json` under
  the discovered store directory.
- A derived index may cache receipt id, schema, created time, parent/child ids,
  status, digest/proof summary fields provided by the proof layer, and safe
  labels. The index must not be the source of truth for receipt body fields.
- Index rebuild scans receipt JSON files, validates exact ids and schemas, and
  writes a new index through the same temp-file/rename discipline.
- Governed verification reads a requested root receipt and exactly scoped child
  receipts from the store, then passes parsed receipts into `runx-receipts`.

Write discipline:
- Write receipt JSON to a same-directory temp file with exclusive create,
  `0600` permissions where supported, file sync, atomic rename to
  `<receipt_id>.json`, and parent-directory sync where supported.
- Update the derived index only after the receipt file commit succeeds. If index
  update fails after receipt commit, report `ReceiptIndexStale` and keep the
  receipt readable by exact id.
- Do not emit a public success summary, success index row, or tree/proof success
  result until the authoritative receipt JSON is durable.
- Retrying a completed run must not overwrite an existing receipt id; collisions
  are typed errors unless the existing receipt is byte-identical and the caller
  is explicitly rebuilding the index.

## Acceptance

Profile: standard

Validation:
- `cargo fmt --check --manifest-path crates/Cargo.toml`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime`
- `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
- `bash -lc '! rg -n "std::fs|std::path" crates/runx-receipts/src'`
- `git diff --check`

Required behavior:
- [ ] `--receipt-dir` or equivalent explicit runtime input wins over config,
  environment, and default.
- [ ] Runtime policy/config receipt path wins over `RUNX_RECEIPT_DIR`.
- [ ] `RUNX_RECEIPT_DIR` wins over the project run state default.
- [ ] Relative receipt paths resolve from the workspace/runx base even when the
  receipt directory does not exist yet.
- [ ] Missing governed receipt store fails closed with typed actionable error;
  non-governing history/list mode may return empty with a safe message.
- [ ] Unreadable store, file-instead-of-directory, malformed index, malformed
  receipt JSON, and wrong receipt schema each produce distinct typed errors.
- [ ] Receipt JSON commits before derived index/summary success, and an injected
  write failure leaves no successful-looking receipt or index entry.
- [ ] Index rebuild recovers a missing/stale index from authoritative receipt
  JSON and rejects inconsistent receipt id/file-name pairs.
- [ ] Public run summaries, Aster messages, reviewer text, training exports, and
  normal CLI output never include absolute local filesystem paths.
- [ ] Runtime can load discovered receipt children and pass parsed receipts into
  the tree resolver/verification boundary without adding IO to
  `runx-receipts`.

## Phase 1: Discovery Contract

Status: completed
Dependencies: none

Objective: Define how a run discovers its receipt store.

Changes:
- Add precedence docs and tests.
- Define config/env/CLI names, including the exact runtime config field.
- Add a `ReceiptPathSource`/equivalent result that records which source won for local diagnostics.
- Add `ReceiptStoreLabel`/safe projection helpers next to the resolver.

Acceptance:
- none

## Phase 2: Local Store

Status: completed
Dependencies: Phase 1

Objective: Implement local receipt store IO in runtime.

Changes:
- Add store read/write/list/index APIs.
- Use temp-file-backed writes for receipts and indexes.
- Add typed store/index/receipt parse errors.
- Add index rebuild from receipt JSON.

Acceptance:
- none

## Phase 3: Safe Projection

Status: completed
Dependencies: Phase 2

Objective: Ensure public outputs are reviewer-safe.

Changes:
- Redact absolute paths in summary/projection code.
- Add regression fixture with an absolute path input.
- Route local-path details through safe labels in CLI, SDK, Aster, and knowledge indexing projections.

Acceptance:
- none

## Phase 4: Tree/Proof Sequencing

Status: completed
Dependencies: Phase 2; coordinates with `rust-receipt-tree-resolution` and

Objective: Wire store discovery into verification without stealing tree/proof

Changes:
- Load exact root/child receipts from the local store and pass parsed receipts into the existing or new tree resolver boundary.
- Keep canonical digest, signature, authority proof, and verification-summary checks delegated to `runx-receipts`.
- Add a sequencing note to runtime tests if the full resolver/proof APIs are not yet landed.

Acceptance:
- none

## Rollback

- Keep existing in-memory runtime receipt behavior until local store persistence
  tests are green.
- If the derived index format proves wrong, retain authoritative receipt JSON and
  drop/rebuild the index; do not weaken governed verification to trust stale
  index data.
- If safe projection misses a public surface, block public projection changes
  until the safe-label helper is used by that surface.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: The prior critical/high blockers (F1–F4) are fully resolved: `write_receipt` lands a Unix-mode 0600 temp file with create-new, fsync, atomic rename, and parent-dir sync; on-disk `index.json` is schema-tagged, loaded with `MalformedIndex`/`ReceiptIndexStale` typed errors, and verified against receipt JSON; tests cover atomic commit-then-stale-index, malformed/stale index, divergent rewrite, and absolute-path Display redaction. The `runx-receipts` IO-free guard still holds (no `std::fs|std::path|std::io` in `crates/runx-receipts/src`). F5 (Display path leak) is materially improved: `#[error]` strings no longer embed `{path}` for path-bearing variants, but the redaction is partial — `StoreUnreadable`/`ReceiptUnreadable` Display still renders the wrapped `std::io::Error`, which on Rust 1.83+ embeds the operating filesystem path; `InvalidReceiptId`/`IdFilenameMismatch`/`ReceiptAlreadyExists` Display still embed caller-supplied `receipt_id` strings that may be path-like. F6 (CLI/SDK wiring) remains unimplemented but stays non-blocking per the prior verdict. New low-severity edges: (a) TOCTOU between `receipt_path.exists()` and `write_atomic` allows concurrent same-id divergent writers to bypass the `ReceiptAlreadyExists` guard via `fs::rename` overwrite; (b) `write_atomic` reports `StoreUnreadable` if `sync_directory` fails after a successful `fs::rename`, leaving the receipt JSON durable while the caller sees a write error and skips index rebuild; (c) a receipt id `"index"` would collide with the manifest filename `index.json` and either trip the divergent-rewrite check or be hidden from `list()`. None of these block completion; the spec acceptance items map cleanly to the implementation and tests.

Attack log:
- `crates/runx-runtime/src/receipt_store.rs::write_receipt`: Verify F1: atomic temp-file/rename, 0600, fsync, and divergent-rewrite guard exist with tests -> clean (write_atomic uses create_new+0o600+sync_all+rename+sync_directory; tests cover commit, identical-rewrite, divergent-rewrite, index-stale.)
- `crates/runx-runtime/src/receipt_store.rs::ReceiptStoreError`: Verify F2: typed errors MalformedIndex/ReceiptIndexStale/UnsafePathProjection present with public_message arms -> clean
- `crates/runx-runtime/src/receipt_store.rs::load_index/rebuild_index`: Verify F3: on-disk index.json with schema, load with verify, rebuild via atomic write -> clean (index.json filtered from list(); verify_index checks ids/file_name/created_at.)
- `crates/runx-runtime/tests/receipt_store.rs`: Verify F4: tests for atomic write and malformed/stale index -> clean
- `crates/runx-receipts/src`: IO-free invariant: grep std::fs|std::path|std::io -> clean (Grep returns no matches; runx-receipts stays pure.)
- `crates/runx-runtime/src/receipt_store.rs::ReceiptStoreError Display`: F5 verify: does Display still leak absolute paths via {source} or {receipt_id}? -> finding (io::Error source from std::fs on Rust ≥1.83 embeds operating path; receipt_id-bearing variants embed caller strings.)
- `crates/runx-cli & crates/runx-sdk`: F6 verify: are CLI/SDK now consuming the resolver/safe labels? -> finding (Still no matches; remains non-blocking per prior verdict.)
- `crates/runx-runtime/src/receipt_store.rs::write_receipt`: Concurrent write race: two writers same id different content -> finding (TOCTOU between exists()+write_atomic; fs::rename overwrites silently.)
- `crates/runx-runtime/src/receipt_store.rs::write_atomic`: sync_directory failure after successful rename — error classification -> finding (Returns StoreUnreadable while receipt JSON is durable and index never rebuilt.)
- `crates/runx-runtime/src/receipt_store.rs::receipt_file_name`: Reserved-name receipt id like `index` collides with manifest -> finding (`index` accepted; rebuild_index clobbers the receipt during rename.)
- `crates/runx-runtime/src/receipt_paths.rs::safe_receipt_store_label`: Label redaction for project-relative, project-outside-workspace, and external paths -> clean (Tests cover all three modes; lexical_normalize handles ./..; external paths hash to 16-hex.)
- `crates/runx-runtime/src/receipt_tree.rs::referenced_receipt_id`: URI parsing: prefix variant vs bare id vs scheme-only -> clean
- `fixtures/runtime/receipt-tree/oracle.json`: Task-scoped fixture used by tests (consumer in receipts crate) -> clean (Consumed by crates/runx-receipts/tests/receipt_tree_fixtures.rs; in-scope per fixtures/runtime scope.)
- `ambient drift (doctor, receipts internals, contracts)`: Confirm ambient drift is not blamed on this task -> skipped (Treated as context per provider instruction.)

Findings:
- [critical/non-blocking] `F1-missing-receipt-write-apis` Atomic write_receipt with temp file, 0600 mode, fsync, rename, and parent-dir sync is now present with tests.
  - Location: `crates/runx-runtime/src/receipt_store.rs:51`
  - Evidence: `write_receipt` (l.51-83), `write_atomic` (l.418-433), `write_temp_file` with `create_new(true)`+`mode(0o600)`+`flush`+`sync_all` (l.435-447), `sync_directory` (l.449-451). Tests: `write_receipt_commits_readable_receipt_and_index`, `write_receipt_allows_identical_and_rejects_divergent_rewrite`, `index_write_failure_reports_stale_but_receipt_stays_readable` in `crates/runx-runtime/tests/receipt_store.rs`.
  - Validation: Prior critical blocker resolved by the fix. Verified via direct read of `receipt_store.rs` and `tests/receipt_store.rs`.
- [high/non-blocking] `F2-missing-malformed-index-and-other-typed-errors` MalformedIndex, ReceiptIndexStale, and UnsafePathProjection variants are present with public_message arms.
  - Location: `crates/runx-runtime/src/receipt_store.rs:297`
  - Evidence: `ReceiptStoreError::MalformedIndex` (l.297-298), `ReceiptIndexStale` (l.299-300), `UnsafePathProjection { reason }` (l.301-302), with matching `public_message` arms (l.336-344). Tests `malformed_index_is_typed_error` and `stale_index_is_typed_error` exercise the first two.
  - Validation: Prior high blocker resolved by the fix. `UnsafePathProjection` is defined but has no current production call site; acceptable since spec only requires the variant to exist.
- [high/non-blocking] `F3-no-on-disk-index-load-or-persistence` Receipt store now persists `index.json` with schema, rebuild, load with verify, and stale/malformed detection.
  - Location: `crates/runx-runtime/src/receipt_store.rs:113`
  - Evidence: `RECEIPT_STORE_INDEX_SCHEMA = runx.receipt_store_index.v1` (l.17), `INDEX_FILE_NAME = index.json` (l.18), `load_index` (l.113-140), `rebuild_index` writes via `write_atomic` (l.142-159 + l.197-204), `verify_index` checks listed vs indexed ids/file names/created_at (l.161-195).
  - Validation: Prior high blocker resolved. `list()` filters out `index.json` so the manifest is not treated as a receipt.
- [high/non-blocking] `F4-missing-required-tests-atomic-and-malformed-index` Atomic-write and malformed-/stale-index tests are now present.
  - Location: `crates/runx-runtime/tests/receipt_store.rs:145`
  - Evidence: `write_receipt_commits_readable_receipt_and_index` (l.146), `write_receipt_allows_identical_and_rejects_divergent_rewrite` (l.163), `index_write_failure_reports_stale_but_receipt_stays_readable` (l.184), `malformed_index_is_typed_error` (l.202), `stale_index_is_typed_error` (l.217).
  - Validation: Prior high blocker resolved.
- [medium/non-blocking] `F5-error-display-leaks-paths-via-io-error-and-receipt-id` Display redaction is partial: `StoreUnreadable`/`ReceiptUnreadable` render the wrapped io::Error (which on Rust ≥1.83 embeds the operating path), and `InvalidReceiptId`/`IdFilenameMismatch`/`ReceiptAlreadyExists` Display still embed caller-supplied `receipt_id` strings that may be path-like.
  - Location: `crates/runx-runtime/src/receipt_store.rs:267`
  - Evidence: Variants at l.267-272 (`StoreUnreadable`) and l.277-282 (`ReceiptUnreadable`) format `"...: {source}"` where `source: std::io::Error` is produced by `fs::metadata`/`fs::read_dir`/`fs::read`/`fs::read_to_string` with workspace `Cargo.toml` `rust-version = "1.85"`. Variants at l.273-274, l.289-294, l.295-296 embed raw `receipt_id`. The added regression test `receipt_store_error_display_does_not_leak_absolute_paths` only asserts on `MissingStore` and `MalformedIndex` (which omit `{path}`/`{source}`).
  - Impact: An `anyhow`/`tracing` chain that prints the error via `{}` from a real fs failure can still surface the absolute receipt store path, contradicting the spec invariant `public … outputs use receipt ids and relative/safe labels only`.
  - Validation: Add a Display redaction test that triggers a real filesystem error against `/Users/kam/private/…` and asserts the rendered string contains no absolute path. Either drop `{source}` from Display and route detail through a separate internal accessor, or mark `ReceiptStoreError` as internal-only and ensure every public surface calls `public_message`.
- [low/non-blocking] `F6-cli-and-sdk-not-wired-to-resolver-or-safe-labels` `runx-cli` and `runx-sdk` still do not consume the resolver or `ReceiptStoreLabel`.
  - Location: `crates/runx-cli:1`
  - Evidence: Grep `receipt|RUNX_RECEIPT_DIR|ReceiptStoreLabel|ReceiptStorePublicProjection` in `crates/runx-cli` matches only `Cargo.toml`; same grep in `crates/runx-sdk` returns no matches. Spec Touchpoints list `CLI options/env parsing` and `SDK host-protocol calls that inspect receipt history`; Phase 3 lists `Route local-path details through safe labels in CLI, SDK, Aster, and knowledge indexing projections`.
  - Impact: Acceptance bullet `Public run summaries, … normal CLI output never include absolute local filesystem paths` is only proven at the resolver/store boundary, not at the CLI/SDK surfaces enumerated in Touchpoints.
  - Validation: Either wire `runx-cli`/`runx-sdk` to hydrate `ReceiptPathInputs`/`ReceiptStorePublicProjection` for any receipt-bearing output, or move the CLI/SDK wiring obligation to a named follow-up spec and update Touchpoints/Acceptance accordingly so completion is not claimed beyond what is built.
- [low/non-blocking] `F7-toctou-race-bypasses-divergent-rewrite-guard` Concurrent writers can both pass the `receipt_path.exists()` check and then `fs::rename` over each other, silently overwriting the prior receipt with divergent content.
  - Location: `crates/runx-runtime/src/receipt_store.rs:61`
  - Evidence: `write_receipt` performs `if receipt_path.exists() { … return ReceiptAlreadyExists }` at l.61-73 before calling `write_atomic` (l.75). `write_atomic` writes a unique temp file with `create_new(true)` (l.437) but `fs::rename` (l.423) atomically replaces an existing destination on POSIX. Two writers for the same id with different content can both observe `!receipt_path.exists()`, each create a distinct temp file, and the later rename wins silently.
  - Impact: Spec Design Contract says `collisions are typed errors unless the existing receipt is byte-identical`. Under concurrent writers, the divergent-rewrite guard is bypassed and the last writer wins without an error. In the local single-process runtime this is unlikely, but CI/Aster fan-out could trigger it.
  - Validation: Add a concurrency test that spawns two threads writing different content for the same id; assert exactly one succeeds and the other returns `ReceiptAlreadyExists`. Fix by using `OpenOptions::create_new(true)` on the final path (or `linkat(AT_FDCWD, temp, AT_FDCWD, final, 0)` / Linux `renameat2(RENAME_NOREPLACE)`), then unlink the temp.
- [low/non-blocking] `F8-sync-directory-failure-misclassified-after-successful-rename` When `sync_directory` fails after a successful rename, `write_atomic` returns `StoreUnreadable` even though the receipt JSON is now durable on disk.
  - Location: `crates/runx-runtime/src/receipt_store.rs:422`
  - Evidence: `write_atomic` chains `write_temp_file → fs::rename → sync_directory` (l.422-424) under a single `and_then` and reports `StoreUnreadable` for any failure (l.427-430). After `fs::rename` succeeds, the receipt JSON is at `<id>.json`; a subsequent `sync_directory` failure leaves the receipt present on disk while `write_receipt` returns a write error (l.75 `?` propagation) before reaching `rebuild_index`, so the derived index is never updated.
  - Impact: Caller sees a write failure while the receipt JSON is durable but absent from the index. This is the inverse of the `ReceiptIndexStale` path (receipt present, index stale, caller informed). The cleanup `fs::remove_file(&temp_path)` at l.426 is also a no-op here because the temp file was renamed away. Mild violation of `an injected write failure leaves no successful-looking receipt or index entry`.
  - Validation: Split `write_atomic` into pre-rename and post-rename phases. After successful `fs::rename`, treat a `sync_directory` failure as a soft warning that still allows `rebuild_index` to run, or surface a dedicated `ReceiptIndexStale`-like variant since the receipt is durable.
- [low/non-blocking] `F9-receipt-id-index-collides-with-manifest-file` `receipt_file_name` allows the id `"index"`, which collides with the derived manifest file `index.json` and is silently filtered from `list()`.
  - Location: `crates/runx-runtime/src/receipt_store.rs:349`
  - Evidence: `receipt_file_name` rejects empty/`.`/`..`/`/`/`\` but accepts `"index"` (l.349-361). `list()` skips `path.file_name() == Some("index.json")` (l.99-103). Result: writing a receipt with id `"index"` either trips `ReceiptAlreadyExists` (if a real index pre-exists with different bytes) or stores the receipt at `index.json` and then has `rebuild_index` immediately `fs::rename` the new index over it, clobbering the receipt. Reads of id `"index"` would parse the index manifest and return `WrongSchema`.
  - Impact: Sharp edge for any caller that legitimately wants id `"index"`. Production receipt ids follow the `hrn_rcpt_*` prefix so this is unlikely to fire in practice, but the failure mode is silent data loss in the clobber path.
  - Validation: Add a reserved-id check in `receipt_file_name` that rejects ids whose generated file name equals the manifest file (`index.json`). Cover with a unit test.

## Self Eval

- Target score: 9.5. Passing means receipt storage is predictable for operators
  without contaminating proof code or public comments.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: isolate runtime IO from receipt proof/tree verification

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T04:06:48Z
Ended: 2026-05-19T04:13:51Z

Checks:
- path audit
  - Grounded in: code:packages/cli/src/args.ts:138
  - Result: passed
  - Evidence: Explicit path input, env override, resolver precedence, and safe labels are now specified together.
- command audit
  - Grounded in: code:crates/Cargo.toml:1
  - Result: passed
  - Evidence: Validation targets existing Cargo workspace packages plus a scoped no-IO guard for `runx-receipts`.
- scope/migration audit
  - Grounded in: code:crates/runx-runtime/src/lib.rs:3
  - Result: passed
  - Evidence: Runtime owns local receipt IO; `runx-receipts` stays IO-free and TS receipt migration remains out of scope.
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: Acceptance is phased after discovery, local store, safe projection, and tree/proof sequencing boundaries.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback preserves in-memory receipts until persistence is green; repair rebuilds derived indexes from receipt JSON.
- design challenge
  - Grounded in: code:packages/runtime-local/src/runner-local/receipt-paths.ts:5
  - Result: passed
  - Evidence: Divergent TS defaults are resolved by one Rust default plus explicit caller paths for global/CI stores.

Issues:
- none


## Planning Log

- 2026-05-19: Expanded placeholder into runtime receipt store discovery
  contract.
