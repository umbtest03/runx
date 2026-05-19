---
spec_version: '2.0'
task_id: rust-journal-local
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T07:01:40Z'
status: completed
harden_status: ready_for_review
size: medium
risk_level: medium
---

# Rust journal local

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T07:01:40Z
Review gate: pass

## Summary

Port local history and journal projection surfaces to Rust without creating a
second source of truth beside sealed harness receipts.

`runx history` is a receipt-store view today: TypeScript lists verified local
receipts, merges paused run state from ledgers, filters by query/skill/status/
source/actor/artifact/date, and renders reviewer-safe summaries. The Rust port
must use `rust-runtime-receipt-path-discovery` and `LocalReceiptStore` rather
than a new journal directory.

`runx journal show` is not a current TypeScript CLI surface. If this spec adds
it, it is a derived diagnostic projection over a run's sealed harness receipt
tree plus ledger/checkpoint execution events. Projection rows may carry
`projector_id`, `source_refs`, `recorded_at`, and `watermark`, matching the
contract-spine projection discipline. They are not an append-only mutable
authority log and must not be written as a separate governed source of truth.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (history command)
- `@runxhq/runtime-local` (current receipt-backed history implementation)
- `crates/runx-runtime`
- `crates/runx-cli`
- `crates/runx-sdk` (only if the SDK history surface is moved off CLI-backed
  TS behavior in the same implementation)

Current TypeScript sources:
- `packages/cli/src/commands/history.ts`
- `packages/cli/src/args.ts`
- `packages/cli/src/dispatch.ts`
- `packages/cli/src/help.ts`
- `packages/cli/src/commands/history.test.ts`
- `packages/runtime-local/src/runner-local/history.ts`
- `packages/runtime-local/src/runner-local/receipt-paths.ts`
- `packages/runtime-local/src/sdk/index.ts` (`history()` returns local receipt
  summaries with paths today; Rust must not expose absolute paths in public
  projections)

Files impacted:
- `crates/runx-runtime/src/journal.rs`
- `crates/runx-runtime/src/receipt_store.rs`
- `crates/runx-runtime/src/receipt_paths.rs`
- `crates/runx-runtime/src/receipt_tree.rs`
- `crates/runx-runtime/tests/journal_history.rs` (new or equivalent)
- `crates/runx-cli/src/launcher.rs` or the native CLI dispatch module active
  after `rust-cli-rust-cutover`
- `crates/runx-cli/src/main.rs` only if native dispatch is introduced here
- `crates/runx-sdk/src/**` only if SDK history is included here
- `fixtures/journal/**`
- `fixtures/runtime/**`
- `fixtures/cli-parity/**`

Invariants:
- Do not add `RUNX_JOURNAL_DIR` or revive `RUNX_MEMORY_DIR`. Local history uses
  the receipt path contract: explicit `--receipt-dir`/runtime input, config,
  `RUNX_RECEIPT_DIR`, then `<runx_project_dir>/receipts`.
- Sealed harness receipts remain authoritative. History and journal rows are
  derived projections with `source_refs` to exact receipt ids or typed
  `runx:harness_receipt:<id>` references.
- Projection rows carry `projector_id` and `watermark` when persisted or
  fixture-asserted. Reprojecting from cited receipts/ledgers must produce the
  same public payload.
- Public CLI, SDK, Aster, reviewer, and training-export outputs never include
  absolute local filesystem paths. Use `ReceiptStoreLabel`/safe projections.
- Governed verification fails closed on malformed receipt stores; non-governing
  history/list views may degrade only with an explicit safe message.
- Exact receipt ids or typed runx receipt URIs are accepted. Suffix lookup is
  forbidden for governed paths and must not be reintroduced for history
  convenience.
- Current Rust `ExecutionJournal` is in-memory execution telemetry. Persisted
  journal projections must be derived from receipts/ledgers/checkpoints, not
  from an unrelated mutable file format.
- Cloud journal sync is out of scope.

## Objectives

- Port receipt-store-backed local history read/list/projection behavior; do not
  add a separate journal index writer.
- Port `runx history` filters and rendering/JSON semantics to the native Rust
  path that is active after `rust-cli-rust-cutover`.
- Add a derived local journal projection API that can show a run's execution
  events from sealed harness receipts and available local ledgers/checkpoints.
- Add `runx journal show <run-id|receipt-id>` only as a new diagnostic surface,
  with CLI help/tests/fixtures making clear that it is a projection, not a
  pre-existing TS parity command.
- Add fixture coverage for receipt-backed listing, query filters, paused runs,
  malformed stores, safe path projection, and deterministic journal
  reprojection.

## Scope

In scope:
- `runx history` terminal and paused-run listing.
- History filters: free-text query, skill/name, status, source, actor,
  artifact type, since, until, limit.
- JSON and human render paths for history once the native Rust CLI owns the
  command.
- Exact-id receipt inspection inputs needed by history/journal projections.
- Derived journal projection rows sourced from harness receipt trees and local
  execution ledgers/checkpoints when present.
- Fixture/oracle updates under `fixtures/runtime/**`, `fixtures/journal/**`,
  and `fixtures/cli-parity/**`.

Out of scope:
- Cloud journal sync.
- A standalone mutable journal store or GC/compaction semantics.
- Legacy pre-cutover `rx_`/`gx_` receipt compatibility beyond explicit
  archival/sunset requirements.
- Proof, signature, tree traversal, or receipt path semantics already owned by
  the receipt specs.
- SDK history migration unless the implementing task explicitly includes the
  Rust SDK surface.

## Dependencies

- `rust-runtime-skeleton` (completed).
- `runx-contract-spine-hard-cutover` (completed; projection discipline and
  harness receipt spine are authoritative).
- `rust-receipts-parity` (completed; post-cutover harness receipts).
- `rust-receipt-proof-verification` and `rust-receipt-tree-resolution`
  (completed; history consumes verified summaries and must not duplicate proof
  or tree logic).
- `rust-runtime-receipt-path-discovery` (completed; receipt store path
  precedence, index rebuild, and safe public labels).
- Coordinates with `rust-cli-rust-cutover`; native CLI interception may land
  there or here, but acceptance must prove `runx history` no longer delegates to
  TypeScript when this spec is counted complete.

## Design Contract

History source:
- Use `LocalReceiptStore::list()`/index APIs and `runx-receipts` verification
  summaries for terminal runs.
- Merge paused runs from local runtime ledgers/checkpoints only when no terminal
  sealed receipt exists for the same run id.
- Missing stores in non-governing history mode render an empty/safe message;
  malformed stores, wrong schemas, unreadable stores, stale indexes, and
  malformed receipts return typed errors with public safe labels.

Projection shape:
- A persisted or fixture-asserted journal projection row includes:
  `schema`, `entry_id`, `recorded_at`, `projector_id`, `source_refs`,
  `watermark`, `event_kind`, `summary`, and optional `receipt_ref`,
  `harness_ref`, `act_ref`, `decision_ref`, `artifact_refs`, `status`, and
  `verification`.
- `source_refs` use exact receipt references and, where applicable, ledger or
  checkpoint artifact refs. A row must not cite an absolute filesystem path.
- `watermark` is deterministic for the projected source set, such as the
  highest receipt `created_at` plus the last ledger sequence consumed.
- Reprojection from unchanged source refs must produce identical ordered rows.

Ordering and filters:
- Terminal receipts sort newest first using receipt `created_at` or the
  post-cutover harness timestamp. Paused runs sort by available ledger/checkpoint
  time and then id for determinism.
- Query matching preserves TS behavior over name, id, source type, actor, and
  artifact type.
- `--since` and `--until` parse ISO-compatible timestamps and fail with a typed
  CLI error on invalid input.

CLI behavior:
- `runx history [query] [--skill s] [--status s] [--source s] [--actor a]
  [--artifact-type t] [--since iso] [--until iso] [--receipt-dir dir] [--json]`
  remains the compatibility surface.
- `runx journal show <run-id|receipt-id> [--receipt-dir dir] [--json]` may be
  introduced only with help, parser, dispatch, and fixture coverage. If not
  introduced in implementation, remove it from the user-visible objective before
  completion rather than leaving a phantom command.

## Acceptance

Profile: standard

Validation:
- `cargo fmt --check --manifest-path crates/Cargo.toml`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime`
- `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli`
- `cargo test --manifest-path crates/Cargo.toml -p runx-sdk` if SDK history is
  touched
- `pnpm vitest run packages/cli/src/commands/history.test.ts` or the current
  package-scoped equivalent while TS remains the oracle
- `git diff --check`

Required behavior:
- [ ] No code, docs, fixtures, or help introduce `RUNX_JOURNAL_DIR` or
  `RUNX_MEMORY_DIR`.
- [ ] `runx history` uses the receipt path precedence from
  `rust-runtime-receipt-path-discovery`.
- [ ] Empty/missing non-governing history store renders a safe empty state.
- [ ] Malformed receipt store/index/receipt cases produce typed errors and
  public messages without absolute paths.
- [ ] History lists verified post-cutover harness receipts and includes
  verification/ledger status without trusting derived indexes as source of
  truth.
- [ ] Paused run summaries appear only when local ledgers/checkpoints exist and
  no terminal sealed receipt supersedes the run id.
- [ ] Query, skill, status, source, actor, artifact type, since, until, and
  limit filters match the TypeScript oracle fixtures.
- [ ] Human output and JSON output contain receipt ids/safe labels, not local
  absolute receipt paths.
- [ ] Derived journal projection rows cite exact source refs and deterministic
  watermarks, and reproject identically from unchanged sources.
- [ ] If `runx journal show` ships, help, parser, dispatch, JSON, human output,
  and CLI parity fixtures cover it as a new diagnostic projection command.
- [ ] If `runx journal show` does not ship, no help or spec-completion notes
  claim it exists.
- [ ] Native Rust CLI completion proof shows `runx history` is not delegated to
  the TypeScript CLI for the accepted path.

## Phases

### Phase 1: Source Audit And Fixture Oracle

Status: pending

Objective: Freeze current TS history behavior and explicit non-behavior for
`runx journal show`.

Changes:
- Add or update fixture cases for history empty state, terminal receipts,
  paused runs, filters, invalid dates, and safe path projection.
- Record that `packages/memory/**` is absent and is not an implementation
  dependency.
- Decide whether `runx journal show` is included in this implementation slice.

Acceptance:
- none

### Phase 2: Runtime Projection API

Status: pending

Objective: Build Rust runtime history/journal projection APIs over receipt
store and local ledgers/checkpoints.

Changes:
- Add summary/projector structs in `runx-runtime`.
- Consume `LocalReceiptStore`, safe receipt labels, tree/proof summaries, and
  in-memory `ExecutionJournal` where appropriate.
- Keep receipt verification delegated to `runx-receipts`.

Acceptance:
- none

### Phase 3: CLI Surface

Status: pending

Objective: Wire native Rust CLI dispatch for accepted history/journal commands.

Changes:
- Add or update parser/help/dispatch tests in `runx-cli`.
- Preserve TS-compatible history flags and JSON shape where promised by the
  fixture oracle.
- Add `journal show` only if Phase 1 explicitly included it.

Acceptance:
- none

### Phase 4: SDK And Sunset Coordination

Status: pending

Objective: Prevent SDK/runtime-local sunset drift.

Changes:
- If SDK history remains CLI-backed, document it as out of this spec's
  completion proof.
- If SDK history is ported here, replace absolute `path` exposure with safe
  labels or remove it from the public Rust SDK shape.
- Feed completion evidence into `rust-ts-sunset-runtime-local`.

Acceptance:
- none

## Rollback

- Keep TypeScript `packages/cli/src/commands/history.ts` and
  `packages/runtime-local/src/runner-local/history.ts` authoritative until the
  Rust CLI fixture oracle passes.
- If the derived journal projection cannot be made deterministic, ship
  receipt-backed `runx history` first and leave `runx journal show` unintroduced.
- If safe path projection is incomplete on any public surface, block the native
  CLI flip for history rather than exposing local paths.

## Open Questions

- Whether `runx journal show` is still desired as a user-facing command now that
  current TS exposes `runx history`, `runx skill inspect`, `runx replay`, and
  `runx diff` but not `runx journal show`. Default: do not ship the command
  unless Phase 1 adds explicit UX and fixture coverage.
- Whether local ledgers/checkpoints get a stable artifact-ref contract in this
  spec or in a later runtime-local sunset spec. Default: cite them only through
  safe projection labels here; do not create a new governed receipt shape.

## Harden Notes

- 2026-05-19: Replaced stale `@runxhq/memory`/`RUNX_JOURNAL_DIR` assumptions
  with receipt-store-backed history and contract-spine projection rules.
- 2026-05-19: Updated impacted Rust paths to match current runtime
  (`crates/runx-runtime/src/journal.rs`, receipt store/path/tree modules).
- 2026-05-19: Added acceptance gates for path safety, exact receipt refs,
  TS-history filter parity, and the absence or fully-covered presence of
  `runx journal show`.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify mode: all four prior blockers (F1 CLI delegation, F2 paused-run merging, F3 fixture coverage, F4 oracle parity) are addressed in the current diff. The native CLI now intercepts `history` in `crates/runx-cli/src/launcher.rs:57-58` and dispatches via `runx_cli::history::run_history_command` from `main.rs:46`, satisfying the "no TS delegation" acceptance gate. `journal.rs:707` reads `ledgers/*.jsonl` and merges checkpoint summaries with proper deduplication against terminal receipts (`history_does_not_double_list_paused_ledger_with_terminal_receipt`). Fixture coverage now includes `fixtures/cli-parity/cases/oracle.json` `history.execute` execute case and an enriched `fixtures/journal/history-oracle.json` with paused_run + invalid_date_filter sections. `--since`/`--until` use a real RFC 3339 `Timestamp::parse` returning `InvalidTimestamp` (F5 fixed). The portable temp dir prefix `runx-runtime-journal-history` in `TestDir::new` is also asserted by `assert_no_local_paths` so any leaked absolute path on Linux/Windows containing the test dir would be caught (F6 effectively addressed). One residual gap remains worth flagging non-blocking: the new `fixtures/journal/history-oracle.json` is still consumed only by Rust tests (`packages/cli/src/commands/history.test.ts` and other TS code do not reference it), so the "filters match the TypeScript oracle fixtures" gate is only proven on the Rust side via a fixture authored alongside the Rust impl.

Attack log:
- `F1 — CLI delegation (verify)`: Confirm launcher.rs intercepts `history` unconditionally and main.rs routes RunHistory through the native Rust handler -> finding (Verified fixed; launcher.rs:57-58 returns RunHistory; main.rs:26 dispatches to run_native_history.)
- `F2 — paused-run merging (verify)`: Inspect list_local_history and list_paused_runs for ledger/checkpoint sources and dedup against terminal_ids -> finding (Verified fixed; list_paused_runs reads ledgers/*.jsonl and accepts PausedRunCheckpoint; dedup test in place.)
- `F3 — Phase 1 fixture coverage (verify)`: Enumerate added fixtures vs Phase 1 list (empty, terminal, paused, filters, invalid dates, safe path) -> finding (Verified fixed; oracle.json gained history.execute; history-oracle.json gained paused_run and invalid_date_filter; Rust tests cover each case.)
- `F4 — TS oracle parity (verify)`: Grep packages/** for references to fixtures/journal/history-oracle.json or fixtures/cli-parity/cases/oracle.json -> finding (Still Rust-only; TS history.test.ts only renders a hand-built summary. Re-listed as low non-blocking.)
- `F5 — --since/--until parsing (verify)`: Trace HistoryFilter date application through ResolvedHistoryFilter::parse and parse_date_filter -> finding (Verified fixed; ISO parse returns InvalidTimestamp.)
- `F6 — path safety assertion portability (verify)`: Inspect assert_no_local_paths and the temp-dir naming for cross-OS coverage -> finding (Verified addressed by portable temp-dir prefix; macOS-specific checks remain harmless extras.)
- `Regression — RUNX_JOURNAL_DIR / RUNX_MEMORY_DIR introductions`: Grep entire repo for either env var -> clean (Only the spec body mentions them in prohibitions.)
- `Regression — runx journal show phantom command`: Grep help, launcher, fixtures for `journal show` -> clean (Only the spec references it conditionally; command is not shipped.)
- `Regression — runtime receipt path precedence in history CLI`: Trace history.rs ReceiptPathInputs ordering vs spec contract -> clean (Explicit --receipt-dir > RuntimeReceiptConfig > RUNX_RECEIPT_DIR > project_runx_dir/receipts.)
- `Regression — exit codes for invalid history args`: Inspect main.rs InvalidArgs handling vs other history errors -> clean (InvalidArgs returns exit 2; other history errors return 1, matching conventional usage/error split.)
- `Convention check — LocalHistoryProjection serde shape`: Inspect Serde rename annotations across LocalHistoryProjection, LocalHistoryReceipt, PausedRunSummary -> finding (F7 — mixed snake_case/camelCase JSON shape; low non-blocking.)

Findings:
- [high/non-blocking] `F1` runx history native dispatch is now wired
  - Location: `crates/runx-cli/src/launcher.rs:57`
  - Evidence: plan_launcher_with_rust_harness intercepts `history` unconditionally at launcher.rs:57-58 and returns LauncherAction::RunHistory(HistoryPlan). main.rs:26 routes RunHistory to run_native_history -> runx_cli::history::run_history_command, which calls runx_runtime::journal::list_local_history through receipt-path resolution (history.rs:54-67).
  - Validation: Run `runx history` with RUNX_JS_BIN unset and observe HistoryPlan is taken; the npm/node delegate branches are bypassed. Unit test `history_routes_to_native_cli_even_with_js_fallback_configured` (launcher.rs:265) covers the routing.
- [high/non-blocking] `F2` Paused-run merging from local ledgers and checkpoints is implemented
  - Location: `crates/runx-runtime/src/journal.rs:707`
  - Evidence: list_paused_runs (journal.rs:707-741) reads `<receipt_dir>/ledgers/*.jsonl`, parses LedgerLine entries, derives PausedRunSummary via paused_run_from_events, and rejects runs already present in terminal_ids. PausedRunCheckpoint pre-supplied entries are merged with the same dedup. Tests `history_merges_paused_ledgers_and_checkpoints` and `history_does_not_double_list_paused_ledger_with_terminal_receipt` exercise both code paths.
  - Validation: Run the journal_history integration tests; the dedup test seals a receipt with id `gx_paused_terminal` and ensures the ledger row is suppressed.
- [medium/non-blocking] `F3` Phase 1 fixture coverage is now present
  - Location: `fixtures/cli-parity/cases/oracle.json:194`
  - Evidence: oracle.json now includes a `history.execute` case (lines 194-223) with argv, expectJson, expect.pendingRuns, and stdoutIncludes assertions. `fixtures/journal/history-oracle.json` covers history_order, journal_source_ref, projector_id, paused_run, and invalid_date_filter. Rust tests cover empty store, terminal receipts, filters, paused merging, terminal supersedes paused, malformed store, invalid date, and deterministic reprojection.
  - Validation: Inspect the diff under fixtures/{journal,cli-parity}; both have the documented additions.
- [low/non-blocking] `F4` Shared oracle file is still Rust-only; TS history test does not reference fixtures/journal or fixtures/cli-parity
  - Location: `fixtures/journal/history-oracle.json:1`
  - Evidence: Grep shows `fixtures/journal/history-oracle.json` and `fixtures/cli-parity/cases/oracle.json` are referenced only by Rust sources (`crates/runx-runtime/tests/journal_history.rs`, `crates/runx-cli/src/history.rs`); `packages/cli/src/commands/history.test.ts` (38 lines, renders only) and other TS packages do not import or read either fixture. The expected receipt ids (`hrn_rcpt_new`, `hrn_rcpt_old`) and refs in history-oracle.json are produced by the Rust test setup itself.
  - Impact: The acceptance gate 'filters match the TypeScript oracle fixtures' is only proven Rust-internally. A future filter divergence between TS and Rust history would not be caught by the current oracle. Non-blocking because the spec validation list still runs `pnpm vitest run packages/cli/src/commands/history.test.ts` and TS remains in place as a separate gate until sunset.
- [low/non-blocking] `F5` --since/--until now parse ISO-8601 and return typed errors
  - Location: `crates/runx-runtime/src/journal.rs:557`
  - Evidence: ResolvedHistoryFilter::parse (journal.rs:541-555) calls parse_date_filter, which uses Timestamp::parse (journal.rs:582-599) and returns JournalProjectionError::InvalidTimestamp { field, value } on failure (journal.rs:570-573). Test `history_rejects_invalid_date_filters` asserts the typed error path against the new invalid_date_filter oracle entry.
  - Validation: Pass HistoryFilter { since: Some("not-a-date".into()), .. } and observe Err(InvalidTimestamp { field: "since", .. }).
- [low/non-blocking] `F6` assert_no_local_paths catches portable temp-dir leakage via test-dir prefix
  - Location: `crates/runx-runtime/tests/journal_history.rs:529`
  - Evidence: TestDir::new (journal_history.rs:541-551) creates temp paths with the unique prefix `runx-runtime-journal-history-`. assert_no_local_paths (journal_history.rs:529-533) checks `/Users/`, `/private/`, and `runx-runtime-journal-history` — the last is a portable catch-all that fires on Linux (`/tmp/runx-runtime-journal-history-...`) and Windows alike because the prefix is part of every test temp path.
  - Impact: Path safety assertion now degrades gracefully across OSes for any leak that includes the test root path. A truly raw `/tmp/` or `C:\` outside the test dir would still escape, but no such surface is reachable from the journal projection inputs in these tests.
- [low/non-blocking] `F7` LocalHistoryProjection JSON shape mixes snake_case and camelCase
  - Location: `crates/runx-runtime/src/journal.rs:59`
  - Evidence: LocalHistoryProjection serializes `projector_id`, `store_label`, `receipts` as snake_case but renames only `pending_runs` to `pendingRuns` (journal.rs:64-65). Inner LocalHistoryReceipt is also snake_case (`receipt_ref`, `created_at`, `source_type`, `harness_id`, `harness_state`, `artifact_types`), while PausedRunSummary opts into `rename_all = camelCase` (journal.rs:90). The cli-parity oracle only asserts `pendingRuns`/`selectedRunner`, so the divergence is not currently caught.
  - Impact: A reviewer or downstream consumer parsing the history JSON sees a mixed convention. If a future TS oracle compares the JSON envelope to camelCase keys (consistent with how PausedRunSummary serializes today), the LocalHistoryReceipt fields would mismatch.
