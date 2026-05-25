---
spec_version: '2.0'
task_id: x402-pay-ledger-projection-v1
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T13:15:00Z'
status: completed
harden_status: not_run
size: medium
risk_level: medium
---

# x402-pay ledger projection v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T13:15:00Z
Review gate: pass

## Summary

Add the P1.17 dogfood lane for x402 payment ledger projection evidence. The
projection must distinguish happy settlement accrual from governed refusal:

- happy settlement accrues exactly one bounded spend when a sealed rail proof is
  present;
- governed refusal records the refusal reason and zero accrued spend when
  authority, bounds, or proof validation blocks settlement.

This lane is evidence/projection only. It does not add live x402 rails, ledger
mutation commands, money movement, or public aliases.

## Current Surface Gap

The current native surface can read local run ledgers and produce journal/history
projections in Rust, while the old TypeScript `runtime-local` reflect projection
path remains a sunset surface. Payment ledger projection must therefore land in
`runx-runtime` and be exposed through the native CLI; adding new trusted
orchestration to `packages/runtime-local` would violate `docs/ts-interop-boundary.md`.
The existing ledger can record emitted artifacts, receipt links, step events,
and generic run history, but it does not expose a payment-specific projection
contract that folds x402 payment packets into an accrual/refusal view.

Boundary status:

- Rust API: `runx_runtime::payment::ledger::build_payment_ledger_projection`,
  accepting sealed graph/harness receipts plus typed child receipt evidence.
- Native file output: `write_payment_ledger_projection_artifact` writes a
  deterministic projection artifact under
  `<receipt-dir>/artifacts/payment-ledger/x402-pay/<receipt-id>.json`, keyed by
  `x402-pay:<source-receipt-id>`, without requiring TypeScript knowledge-store
  code.
- Event payload: the writer returns a `payment_ledger_projected` payload with
  the projection artifact id, projection artifact path, source receipt id,
  scenario id, profile, and disposition.
- Native ledger event: native harness completion appends a deterministic
  `payment_ledger_projected` run event after writing the projection artifact
  when `RUNX_RECEIPT_DIR` is configured.
- Contract: `runx.payment_ledger_projection.v1` has stable JSON fields for
  `scenario_id`, `disposition`, `accrual`, `refusal`, `evidence_refs`, and
  `source_receipt_id`.

The implementation should live beside the existing Rust journal/history
projection boundary, not in core policy and not in `packages/runtime-local`.
Suggested target paths:

- `crates/runx-runtime/src/payment/ledger.rs`
- `crates/runx-runtime/tests/payment/ledger_projection.rs`
- `crates/runx-cli/src/history.rs` or the native harness completion path for
  projection artifact discovery, whichever is the narrower integration point
- `crates/runx-cli/tests/x402_native_dogfood.rs` for TS-free dogfood assertions
- optional generated schema path:
  `schemas/payment-ledger-projection.schema.json`

## Scope And Touchpoints

In scope:

- `.scafld/specs/active/x402-pay-ledger-projection-v1.md`
- `fixtures/ledger-projections/x402-pay-ledger-*.json`
- A future Rust runtime projection module and tests under
  `crates/runx-runtime/`
- Existing x402 harness fixtures as read-only inputs:
  `fixtures/harness/x402-pay-paid-echo.yaml`,
  `fixtures/harness/x402-pay-negative-cap-exceeded.yaml`,
  `fixtures/harness/x402-pay-negative-ambiguous-bounds.yaml`, and
  `fixtures/harness/x402-pay-negative-proofless-rail.yaml`

Out of scope:

- `.scafld/specs/drafts/rust-nitrosend-dogfood.md`
- Shared x402 test rewrites unrelated to P1.17.
- Live x402, Stripe, MPP, refunds, disputes, or real money rails.
- New public `runx ledger`, `runx x402-pay`, `x402-charge`, or `x402-refund`
  commands.
- Changing the existing ledger chain schema.

## Projection Contract

The v1 projection is a derived evidence document:

- `schema_version`: `runx.payment_ledger_projection.v1`
- `payment_profile`: `x402-pay`
- `source_receipt_id`: graph receipt id used for the projection
- `disposition`: `settled` or `refused`
- `scenario_id`: P1 scenario id when known
- `accrual`: settled amount, currency, rail, idempotency key, and proof refs;
  zero amount and empty proof refs for refusal
- `refusal`: null for settled, otherwise reason code, refused stage, and
  `ledger_spend_recorded: false`
- `evidence_refs`: receipt, harness, and artifact refs sufficient to rerun or
  audit the projection

Required P1.17 distinctions:

- P1.1/P1.5 happy settlement: `disposition: settled`,
  `accrual.amount_minor > 0`, `rail_proof_refs` non-empty, and refusal null.
- P1.3 cap exceeded: `disposition: refused`, `reason_code: cap_exceeded`,
  `accrual.amount_minor: 0`, and no rail proof refs.
- P1.4 ambiguous bounds: `disposition: refused`,
  `reason_code: ambiguous_bounds`, `accrual.amount_minor: 0`, and no rail call.
- P1.12 proofless rail: `disposition: refused`,
  `reason_code: missing_rail_proof`, `accrual.amount_minor: 0`, and no paid echo.

## Planned Phases

Phase 1: projection boundary.
: Add the Rust runtime projection builder, contract type, and unit tests that
fold existing x402 ledger artifact envelopes into the v1 projection.

Phase 2: persistence boundary.
: Persist the native projection artifact under the configured receipt directory
and return the `payment_ledger_projected` payload.

Phase 2b: native ledger integration.
: Invoke the projection artifact writer from the native harness completion path
and append `payment_ledger_projected` to the run ledger.

Phase 3: dogfood evidence.
: Run the happy and refusal fixtures, assert projection differences against the
`fixtures/ledger-projections/x402-pay-ledger-*.json` examples, and update the
Phase 1 punch-list only after the executable projection evidence exists.

## Acceptance

Profile: strict

Validation:
- [x] `v1` scafld - Spec validates.
  - Command: `scafld validate x402-pay-ledger-projection-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-3
- [x] `v2` fixture shape - Golden projection examples are valid JSON.
  - Command: `node -e "const fs=require('node:fs'); for (const f of fs.readdirSync('fixtures/ledger-projections').filter(f => f.startsWith('x402-pay-ledger-') && f.endsWith('.json'))) JSON.parse(fs.readFileSync('fixtures/ledger-projections/'+f,'utf8'))"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-4
- [x] `v3` projection tests - Rust runtime projection distinguishes settlement from
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-5
- [x] `v4` x402 dogfood - Native x402 fixture lane remains green.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `v5` projection persistence writer - Runtime writer persists the
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment x402_projection_artifact_writer_persists_under_receipt_dir_and_returns_event_payload`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `v6` projection ledger integration - Native CLI harness run writes the
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_ledger_projection`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8
- [x] `v7` refusal projection ledger integration - Native CLI harness run
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_refusal_ledger_projection -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-9
- [x] `v8` full native x402 dogfood - Native x402 dogfood file remains green
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-10

## Remaining Native Integration

The Rust-native settled and governed-refusal persistence and event emission are
complete. Native artifact/ledger readback is covered by tests that parse the
projection artifact and JSONL `payment_ledger_projected` event from
`RUNX_RECEIPT_DIR`.

No `runx history` projection was added in this spec: the current native history
surface projects sealed receipts and pending runs, while payment-ledger events
are a domain projector artifact under the receipt directory. If product wants
`payment_ledger_projected` visible in `runx history`, file a focused history
projection spec rather than coupling it back into this payment projector.

Validation commands:

- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment`
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_ledger_projection`
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_refusal_ledger_projection`
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood`
- `scafld validate x402-pay-ledger-projection-v1 --json`

## Rollback

Strategy: per_phase

Commands:
- `rm -f .scafld/specs/active/x402-pay-ledger-projection-v1.md`
- `rm -rf fixtures/ledger-projections`
- Future implementation rollback:
  `git checkout HEAD -- crates/runx-runtime/src/payment/ledger.rs crates/runx-runtime/tests/payment/ledger_projection.rs crates/runx-cli/src/history.rs crates/runx-cli/tests/x402_native_dogfood.rs schemas/payment-ledger-projection.schema.json`
- Refusal dogfood fixture rollback:
  `rm -f fixtures/harness/x402-pay-ledger-governed-refusal.yaml fixtures/graphs/payment/x402-pay-ledger-governed-refusal.yaml`

## Harden Rounds

- none

## Planning Log

- 2026-05-21T00:00:00Z: Filed after P1.3, P1.4, and P1.12 refusal fixtures
  existed. The current repo has no Rust-native payment-specific ledger
  projection API, so this spec recorded the missing boundary and
  executable acceptance commands.
- 2026-05-21T00:00:00Z: Added the Rust-native pure projection builder and
  focused runtime tests for the happy settlement and governed refusal golden
  fixtures. Persistence remains a follow-up boundary.
- 2026-05-21T00:00:00Z: Added the narrow Rust-native projection artifact writer
  and runtime persistence test. Full native ledger event append remains blocked
  on wiring the writer into harness completion and the run ledger writer.
- 2026-05-21T00:00:00Z: Wired native harness completion to write the projection
  artifact and append `payment_ledger_projected` when `RUNX_RECEIPT_DIR` is
  configured. Added native CLI dogfood coverage proving the artifact and event
  are produced without TypeScript.
- 2026-05-21T13:07:05Z: Added a dedicated governed-refusal dogfood fixture and
  allowed the metadata-gated payment projector to persist blocked sealed graph
  receipts when reservation/refusal evidence is present. Added runtime and
  native CLI tests proving zero-accrual refusal artifacts and JSONL events are
  written/read back without TypeScript.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Reviewed the x402 payment ledger projection lane within declared scope. The projection module cleanly distinguishes settled accrual (rail proof + reservation match) from governed refusal (zero accrual, refusal record), the artifact writer derives a deterministic per-receipt path under `artifacts/payment-ledger/x402-pay/`, and the JSONL ledger event append checks semantic identity (`source_receipt_id` + `projection_artifact_id`) before either no-op-ing or returning `LedgerEventConflict`. The persistence trigger in `harness/runner.rs` is double-gated on the `RUNX_RECEIPT_DIR` env var and the `payment_ledger_profile=x402-pay` metadata, so it cannot bleed into unrelated harnesses. Golden fixtures match the Rust serialization for both happy P1.5 and refused P1.3 cases; refusal evidence is correctly read from the nested `payment_reservation_packet.data.payment_refusal_packet` path, and `ledger_spend_recorded` defaults to false when absent from skill output. Receipt-id and run-id path segments are validated for alphanumeric/-/_ chars, preventing path traversal via fixture names. No completion blockers, no scope drift beyond the metadata field additions implied by the spec contract.

Attack log:
- `crates/runx-runtime/src/payment/ledger.rs build_payment_ledger_projection`: Differentiate settled vs refused dispositions; verify zero-accrual on refusal and rail-proof matching on settlement -> clean (Refusal path forces amount_minor=0 and empty rail_proof_refs via refused_accrual(); settlement path validates reservation/settlement parity, child receipt linkage, and verification ref ProofKind::PaymentRail + locator==idempotency_key.)
- `crates/runx-runtime/src/payment/ledger.rs append_payment_ledger_projected_event`: Idempotency and conflict semantics under repeated persist calls; path traversal via run_id -> clean (validate_run_ledger_id rejects empty/non-[A-Za-z0-9_-]; same payload identity returns existing path; divergent payload with matching source_receipt_id+projection_artifact_id returns LedgerEventConflict. No file lock, but acceptable for dogfood/evidence boundary.)
- `crates/runx-runtime/src/execution/harness/runner.rs persist_payment_ledger_projection_if_configured`: Trace caller gating; verify projection is opt-in via env+metadata and does not leak into non-x402 fixtures -> clean (Returns early when RUNX_RECEIPT_DIR is unset or payment_ledger_profile metadata != x402-pay. scenario_id is required via required_string_metadata, propagating a clean RuntimeError on misconfiguration.)
- `fixtures/harness/x402-pay-paid-echo.yaml metadata addition`: Ambient regression on existing consumers of the fixture -> clean (Only consumers are harness_fixtures schema check (metadata is unconstrained) and existing x402 dogfood tests that don't assert metadata; new fields are additive.)
- `fixtures/ledger-projections/*.json golden fixtures`: Field-order / serde alignment between Rust struct order and golden JSON -> clean (Both golden files serialize fields in declaration order (schema_version, payment_profile, scenario_id, source_receipt_id, disposition, accrual, refusal, evidence_refs) matching #[derive(Serialize)] on PaymentLedgerProjection; refusal is null for settled, populated for refused.)
- `crates/runx-cli/tests/x402_native_dogfood.rs native_x402_refusal_ledger_projection`: Verify refusal CLI lane actually exits success and writes both artifact and JSONL -> clean (policy_denied disposition still produces stdout receipt + success exit; persist runs before assert_expectations sealing fan-out succeeds; reserve skill emits payment_refusal_packet nested under reservation packet, which read_payment_refusal_packet handles.)

Findings:
- none
