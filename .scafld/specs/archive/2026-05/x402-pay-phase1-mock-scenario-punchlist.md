---
spec_version: '2.0'
task_id: x402-pay-phase1-mock-scenario-punchlist
created: '2026-05-21T00:46:25Z'
updated: '2026-05-21T09:46:38Z'
status: completed
harden_status: not_run
size: small
risk_level: low
---

# x402-pay Phase 1 mock scenario punch list

This file is append-only. Rows stay here after closure with their status and
closing evidence updated by a follow-up spec.

Native Rust CLI coverage added 2026-05-21:

- P1.1 happy path remains covered by `fixtures/harness/x402-pay-approval.yaml`.
- P1.5 approval denial is now covered by
  `fixtures/harness/x402-pay-approval-denied.yaml`.
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test
  x402_native_dogfood` proves both fixtures through the native Rust CLI with no
  Node, pnpm, tsx, or TypeScript package dependency.

Naming boundary: every row in this punch list belongs to canonical `x402-pay`
mock payment coverage. Charge and refund names are separate profile/flow
families, not `x402-pay` aliases and not Phase 1 executable payment surfaces.

| Scenario | Status | Observed Behavior | Expected Behavior | Concrete Blocker | Follow-up |
| --- | --- | --- | --- | --- | --- |
| P1.2 | Closed | Native negative fixture `fixtures/harness/x402-pay-negative-malformed-challenge.yaml` now blocks after `quote` only. | Malformed challenge is rejected with a governed error and no reserve, settlement, or ledger spend entry. | Closed by `x402-pay-phase1-negative-fixtures-v1`; no reserve child receipt is emitted. | Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`. |
| P1.3 | Closed | Native negative fixture `fixtures/harness/x402-pay-negative-cap-exceeded.yaml` fails authority admission before rail fulfillment. | Cap-exceeded reserve declines before any rail call and records a refused intent. | Closed by `x402-pay-phase1-negative-fixtures-v1`; stderr reports spend capability binding denial and no mock rail material is emitted. | Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`. |
| P1.4 | Closed | Native negative fixture `fixtures/harness/x402-pay-negative-ambiguous-bounds.yaml` blocks after `reserve` refusal. | Undefined currency or range bounds produce governed refusal and no rail call. | Closed by `x402-pay-phase1-negative-fixtures-v1`; no approval or fulfill child receipt is emitted. | Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`. |
| P1.7 | Open | Current harness uses deterministic receipt ids but does not replay the same idempotency key through a second settlement attempt. | Replay returns the recovered receipt without issuing a second mock spend. | Missing mock rail spend counter or persisted idempotency lookup exposed through CLI/harness. | Add idempotency replay fixture once mock rail state is observable. |
| P1.8 | Closed | Native negative fixture `fixtures/harness/x402-pay-negative-authority-broader-child.yaml` now fails at authority admission with `child payment authority is not a subset of parent authority`. | Broader child `AuthorityTerm` is rejected before mock execution. | Closed by `x402-pay-authority-cli-admission-v1`; the crafted reservation keeps the spend binding valid while widening the child authority cap above the parent. | Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`. |
| P1.9 | Open | No current CLI fixture consumes and then reuses the same spend capability ref. | Second use of a single-use spend capability is rejected by core. | Missing spend-capability consumption ledger visible to current harness. | Add single-use capability replay fixture after capability state is persisted. |
| P1.10 | Open | Happy-path receipts are persisted before test observation, but no fixture can delay receipt persistence after rail success. | Caller cannot observe success until the rail proof receipt is durably sealed. | Missing delayed receipt-store fault injection in CLI/harness. | Add receipt-store delay/failure fixture at runtime boundary. |
| P1.11 | Open | `pay-recover` has a happy-path recovery inspection harness, but no current CLI fixture simulates a mock rail crash after partial mutation. | Recovery queries by idempotency key after crash and classifies the state as sealed or escalated. | Missing crash/partial-mutation fixture and persisted rail state to recover against. | Add mock rail crash fixture once idempotency state is observable. |
| P1.12 | Closed | Native negative fixture `fixtures/harness/x402-pay-negative-proofless-rail.yaml` returns rail success without `rail_proof` and fails before paid echo. | Rail success without required proof fields is refused as non-sealable. | Closed by `x402-pay-phase1-negative-fixtures-v1`; stderr reports missing rail proof and no echo child receipt is emitted. | Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`. |
| P1.13 | Open | Current reserve fixture is single-run and single-threaded. | Concurrent reserves under one policy use atomic budget arithmetic. | Missing policy budget store and concurrent CLI/harness driver. | Add budget store fixture and concurrent reserve test. |
| P1.14 | Closed | Native negative fixture `fixtures/harness/x402-pay-negative-quote-drift.yaml` now fails authority admission before rail fulfillment. | Spend above reserved bounds is rejected before mock execution. | Closed by `x402-pay-quote-drift-v1`; the reservation keeps the child authority subset-valid at the quoted 125 minor-unit bound while drifting the spend binding to 175. | Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`. |
| P1.17 | Open | The graph ledger records run events and receipt links, not payment accrual/refusal projections. | Ledger projection distinguishes P1.1 accrual from P1.3/P1.4 refused entries. | Missing payment-specific ledger projection file and refusal scenarios. | Add payment ledger projection after refusal fixtures exist. |

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T09:46:38Z
Review gate: pass

## Review

Status: completed
Verdict: pass
Mode: verify
Summary: Human-reviewed override accepted: implemented and validated in commit 2ceb010; x402-pay mock approval fixtures and punch-list coverage assertions passed

Attack log:
- `review gate`: manual human audit -> clean (implemented and validated in commit 2ceb010; x402-pay mock approval fixtures and punch-list coverage assertions passed)

Findings:
- none
