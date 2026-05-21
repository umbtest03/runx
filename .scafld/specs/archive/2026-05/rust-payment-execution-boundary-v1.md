---
spec_version: '2.0'
task_id: rust-payment-execution-boundary-v1
created: '2026-05-21T13:05:00Z'
updated: '2026-05-21T12:51:45Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Rust payment execution boundary cleanup

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T12:51:45Z
Review gate: pass

## Summary

Move the payment-specific execution bookkeeping out of the generic runner step
loop and into explicit payment-domain modules. The runner may still call one
small domain hook after a step receipt is sealed, but it must not parse
`payment_rail_packet` JSON paths or know mock-rail state names. Add typed
runtime payment packet readers shared by payment state persistence and x402
ledger projection, remove the production `MockRailMutation` naming leak, and
replace `x402-pay` receipt-id substring projection and scenario fallback with
explicit declared payment ledger fixture metadata.

This is a boundary cleanup, not a provider integration. No Stripe API,
PaymentIntent, webhook, SDK, or provider-specific settlement code may be added
to Rust core/runtime/CLI production sources.

## Context

CWD: `.` from the OSS repo root.

Packages:
- `crates/runx-runtime`

Current weak points:
- `crates/runx-runtime/src/execution/runner/steps.rs` performs generic step
  execution, then directly parses `payment_rail_packet.data.*` paths and writes
  payment state.
- `crates/runx-runtime/src/payment_ledger.rs` independently parses
  reservation, settlement, refusal, and paid-tool packet JSON paths, and gates
  x402 projection on `receipt.id.contains("x402-pay")` or
  `harness_ref.uri.contains("x402-pay")`.
- `crates/runx-runtime/src/payment_state.rs` exposes `MockRailMutation` and
  `mock_rail_mutations` as production state terminology. Because
  `mock_rail_mutations` is a persisted `deny_unknown_fields` state-document
  field, this is schema contamination, not a naming nit.
- `crates/runx-runtime/src/execution/runner/authority.rs` still opens
  `FileBackedPaymentStateStore` directly for admission and replay checks.

## Objectives

- Introduce typed payment packet readers for:
  `payment_reservation_packet`, `payment_rail_packet`,
  `payment_refusal_packet`, and `paid_echo_result`.
- Move payment state persistence from `execution/runner/steps.rs` into the
  payment-domain boundary.
- Remove production `MockRailMutation` naming in favor of generic
  `RailMutation` / `RailMutationStatus` state names.
- Replace x402 ledger projection activation with explicit
  `payment_ledger_profile: x402-pay` and `payment_ledger_scenario_id`
  fixture declarations. The runtime kernel must not infer projection
  eligibility or scenario labels from receipt IDs, harness URIs, or graph names.
- Wrap payment-state admission reads behind payment-domain query functions.
- Keep receipt-before-success enforcement based on typed authority/proof
  semantics: `AuthorityVerb::Spend` plus `ProofKind::PaymentRail`.
- Prove no Stripe provider API code entered Rust production sources.

## Scope

In scope:
- `crates/runx-runtime/src/payment_packets.rs` or equivalent typed packet
  reader module.
- `crates/runx-runtime/src/payment_state.rs` payment state type rename and
  domain persistence function.
- `crates/runx-runtime/src/payment_ledger.rs` reuse of typed packet readers
  where it extracts evidence from step outputs/stdout, plus removal of the
  `x402-pay` receipt substring predicate.
- `crates/runx-runtime/src/execution/runner/steps.rs` removal of payment
  packet parsing helpers and direct payment state writes.
- `crates/runx-runtime/src/execution/runner/authority.rs` replacement of
  direct store access with payment-domain query helpers.
- `crates/runx-runtime/src/execution/harness/runner.rs` explicit
  payment-ledger profile gate.
- `fixtures/harness/x402-pay-paid-echo.yaml` fixture metadata declaration for
  the x402 payment ledger profile and scenario id.
- Focused runtime tests for payment state, payment ledger projection, payment
  execution, and Stripe SPT fixtures.

Out of scope:
- Stripe SDK/API/PaymentIntent/webhook implementation in Rust production code.
- Changing graph YAML shapes or checked-in skill names.
- Full replay rehydration for sealed idempotency keys. That remains owned by
  `x402-pay-idempotency-recovery-v1`.
- TypeScript runtime changes.

## Dependencies

- `authority-proof-kind-contract-v1` archived with typed payment rail proofs.
- `authority-core-surface-prune-v1` archived with typed step authority
  admission.
- `x402-pay-stripe-spt-dogfood-v1` archived with Rust-only Stripe SPT dogfood
  coverage.

## Assumptions

- The payment state file is local runtime state, not a public cross-version
  cloud protocol.
- Existing checked-in payment state tests can be migrated from mock names to
  generic rail names without compatibility aliases.
- Stripe remains implemented as skill/provider behavior outside Rust
  core/runtime production code.

## Touchpoints

- `crates/runx-runtime/src/lib.rs`
- `crates/runx-runtime/src/payment_packets.rs`
- `crates/runx-runtime/src/payment_state.rs`
- `crates/runx-runtime/src/payment_ledger.rs`
- `crates/runx-runtime/src/execution/runner/steps.rs`
- `crates/runx-runtime/src/execution/runner/authority.rs`
- `crates/runx-runtime/src/execution/harness/runner.rs`
- `fixtures/harness/x402-pay-paid-echo.yaml`
- `crates/runx-runtime/tests/payment_state.rs`
- `crates/runx-runtime/tests/payment_ledger_projection.rs`
- `crates/runx-runtime/tests/payment_execution.rs`
- `crates/runx-runtime/tests/stripe_spt_payment.rs`
- `crates/runx-cli/tests/x402_native_dogfood.rs`

## Risks

- Moving JSON extraction can accidentally weaken missing-field failures in
  ledger projection.
- Renaming rail mutation state can break tests or persisted local files.
  Because this is not a cloud protocol, the clean cutover is preferred over an
  alias. The state schema version must bump so stale v1 files fail clearly as
  unsupported rather than as corrupted JSON.
- x402 payment ledger projection must stay opt-in via declared fixture metadata;
  receipt IDs, harness URI strings, and graph-name substrings are not
  structural proof.
- The runner will still call a payment hook after sealing; this slice reduces
  domain leakage but does not fully introduce a generic event bus.

## Acceptance

Profile: strict

Validation:
- [x] `v1` runner boundary grep - Generic runner has no direct payment packet
  - Command: `! rg -n "payment_rail_packet|payment_reservation_packet|payment_refusal_packet|paid_echo_result|MockRail" crates/runx-runtime/src/execution/runner/steps.rs`
  - Expected kind: `no_matches`
  - Status: pass
  - Evidence: output was empty
  - Source event: entry-16
- [x] `v2` runtime mock naming grep - Production runtime state has no
  - Command: `! rg -n "MockRail|mock_rail" crates/runx-runtime/src`
  - Expected kind: `no_matches`
  - Status: pass
  - Evidence: output was empty
  - Source event: entry-17
- [x] `v3` Stripe production boundary grep - Rust production sources contain
  - Command: `! rg -n "stripe|STRIPE|PaymentIntent|payment_intent" crates/runx-core crates/runx-runtime/src crates/runx-cli/src`
  - Expected kind: `no_matches`
  - Status: pass
  - Evidence: output was empty
  - Source event: entry-18
- [x] `v3b` x402 projection gate grep - Runtime ledger projection has no
  - Command: `! rg -n "contains\\(\"x402-pay\"\\)|is_x402_payment_receipt" crates/runx-runtime/src/payment_ledger.rs`
  - Expected kind: `no_matches`
  - Status: pass
  - Evidence: output was empty
  - Source event: entry-19
- [x] `v3c` x402 fixture profile grep - x402 paid-echo fixture declares the
  - Command: `rg -n "payment_ledger_profile: x402-pay" fixtures/harness/x402-pay-paid-echo.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20
- [x] `v4` focused runtime tests - Payment state/projection/execution and
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_state --test payment_ledger_projection --test payment_execution --test stripe_spt_payment`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21
- [x] `v5` native x402 dogfood - Rust CLI x402 Stripe SPT fixture still runs
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_stripe_spt_happy_path_runs_without_typescript`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `v6` format and diff hygiene - Rust formatting and whitespace checks pass.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && git diff --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23

## Phase 1: Typed Packets

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- `crates/runx-runtime/src/payment_packets.rs` (all, exclusive) - New typed packet reader module.
- `crates/runx-runtime/src/lib.rs` (line-level) - Expose the module at the runtime crate boundary if tests need it.
- `crates/runx-runtime/src/payment_ledger.rs` (line-level) - Replace local packet readers with typed payment packet readers.
- `crates/runx-runtime/src/execution/harness/runner.rs` (line-level) - Gate payment-ledger projection on explicit fixture metadata.
- `fixtures/harness/x402-pay-paid-echo.yaml` (line-level) - Declare the x402 payment-ledger profile and scenario id.

Acceptance:
- none

## Phase 2: State Boundary

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `crates/runx-runtime/src/payment_state.rs` (line-level) - Add domain persistence input/function, add admission query helpers, bump local state schema version, and rename rail mutation state types, JSON field, duplicate error variant, and error-context strings.
- `crates/runx-runtime/src/execution/runner/steps.rs` (line-level) - Replace direct packet parsing and state writes with a small payment-domain hook.
- `crates/runx-runtime/src/execution/runner/authority.rs` (line-level) - Replace direct payment state store access with payment-domain query helpers.
- `crates/runx-runtime/tests/payment_state.rs` (line-level) - Update state terminology and assertions.

Acceptance:
- none

## Phase 3: Boundary Regression

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `crates/runx-runtime/tests/payment_execution.rs` (line-level) - Adjust only if names changed.
- `crates/runx-runtime/tests/stripe_spt_payment.rs` (line-level) - Adjust only if names changed.
- `crates/runx-cli/tests/x402_native_dogfood.rs` (line-level) - Adjust only if names changed.

Acceptance:
- none

## Rollback

Strategy: per_phase

Commands:
- `git checkout HEAD -- crates/runx-runtime/src/lib.rs crates/runx-runtime/src/payment_state.rs crates/runx-runtime/src/payment_ledger.rs crates/runx-runtime/src/execution/runner/steps.rs crates/runx-runtime/src/execution/runner/authority.rs crates/runx-runtime/src/execution/harness/runner.rs fixtures/harness/x402-pay-paid-echo.yaml crates/runx-runtime/tests/payment_state.rs crates/runx-runtime/tests/payment_ledger_projection.rs crates/runx-runtime/tests/payment_execution.rs crates/runx-runtime/tests/stripe_spt_payment.rs crates/runx-cli/tests/x402_native_dogfood.rs`
- `rm -f crates/runx-runtime/src/payment_packets.rs`

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode re-review of rust-payment-execution-boundary-v1. The previous review's only substantive finding (F1: residual `scenario_id_from_graph_name` substring fallback in `crates/runx-runtime/src/execution/harness/runner.rs`) is fully resolved: the substring matcher is gone, the projection gate uses `string_metadata(fixture, "payment_ledger_profile") == X402_PAY_PAYMENT_PROFILE`, and the scenario id is now read via `required_string_metadata(&fixture.metadata, "metadata.payment_ledger_scenario_id", "payment_ledger_scenario_id")`. The shipped fixture `fixtures/harness/x402-pay-paid-echo.yaml` declares both `payment_ledger_profile: x402-pay` and `payment_ledger_scenario_id: P1.5`, so the eligibility gate and scenario label are now purely typed metadata. The previous critical workspace_mutation blocker was a review-time meta-check on the prior provider and does not recur here — this pass performed read-only inspection only and the task workspace is unchanged. All six recorded acceptance commands (v1, v2, v3, v3b, v3c, v4, v5, v6) still align with current source: `steps.rs` only references payment via the typed hook `persist_payment_state_for_step` (no `payment_rail_packet`/`paid_echo_result`/`MockRail` substrings); `MockRail|mock_rail` is absent from `crates/runx-runtime/src` (all historical strings now appear only inside the spec markdown); `crates/runx-core`, `crates/runx-runtime/src`, and `crates/runx-cli/src` contain no `stripe|STRIPE|PaymentIntent|payment_intent`; `payment_ledger.rs` has no `contains("x402-pay")` or `is_x402_payment_receipt` predicates; admission reads in `authority.rs` route through `consumed_spend_capability_recorded` and `lookup_payment_idempotency_entry`; and `payment_state.rs` schema is bumped to `runx.payment_state.v2` with `UnsupportedSchemaVersion` returned for any mismatch. Receipt-before-success enforcement keys strictly on `AuthorityVerb::Spend` + typed `ProofKind::PaymentRail` via `is_payment_rail_proof_ref`. No new regressions, no Stripe SDK/API surface in Rust production code, no scope drift. Ambient workspace drift (canonical-json, maturity, sandbox.rs, registry edits) is unrelated to this task and is treated as context per the provider contract.

Attack log:
- `crates/runx-runtime/src/execution/harness/runner.rs#persist_payment_ledger_projection_if_configured`: verify_open_blocker_F1: confirm scenario_id_from_graph_name substring fallback has been removed and scenario_id is sourced from fixture metadata only -> clean (harness/runner.rs:684-727 — eligibility gate is `string_metadata(fixture, "payment_ledger_profile") == Some(X402_PAY_PAYMENT_PROFILE)` and scenario id is `required_string_metadata(&fixture.metadata, "metadata.payment_ledger_scenario_id", "payment_ledger_scenario_id")`. `scenario_id_from_graph_name` no longer exists anywhere under crates/ (rg returns no matches). The fixture `fixtures/harness/x402-pay-paid-echo.yaml` declares `payment_ledger_profile: x402-pay` (line 8) and `payment_ledger_scenario_id: P1.5` (line 9). Missing scenario_id now surfaces as `HarnessReplayError::InvalidReplayMetadata` rather than silently defaulting.)
- `verify_open_blocker_workspace_mutation`: confirm the previous review's workspace_mutation blocker does not recur — review is read-only this pass -> clean (This review performed only Read/Grep operations against in-scope files. Task-scope file hashes in the contract's task_changes section are unchanged from baseline. The prior workspace_mutation finding was a meta-check on the previous provider's behavior and is not a property of the work product.)
- `acceptance v1–v6 (recorded evidence)`: spec_compliance: re-read each acceptance command against current source/fixture and confirm the recorded pass result still holds -> clean (v1: rg of payment_rail_packet|payment_reservation_packet|payment_refusal_packet|paid_echo_result|MockRail in steps.rs → 0 matches. v2: rg of MockRail|mock_rail in crates/runx-runtime/src → 0 matches. v3: rg of stripe|STRIPE|PaymentIntent|payment_intent across crates/runx-core, crates/runx-runtime/src, crates/runx-cli/src → 0 matches. v3b: rg of contains("x402-pay")|is_x402_payment_receipt in payment_ledger.rs → 0 matches. v3c: `payment_ledger_profile: x402-pay` present at fixtures/harness/x402-pay-paid-echo.yaml:8. v4/v5/v6: recorded as exit 0 in the session ledger and not contradicted by source inspection.)
- `crates/runx-runtime/src/execution/runner/steps.rs`: regression_hunt: confirm the generic runner only calls a typed payment-domain hook and does not re-introduce packet JSON parsing or direct store access -> clean (steps.rs:19 imports only `PaymentStepStateInput, persist_payment_step_state`. The hook `persist_payment_state_for_step` (lines 83-111) consumes typed `StepAuthorityContext.payment` (rail, counterparty, amount_minor, currency, idempotency_key, spend_capability_ref) and forwards to the domain function. No JSON path strings, no FileBackedPaymentStateStore usage, no payment_rail_packet keys.)
- `crates/runx-runtime/src/execution/runner/authority.rs`: boundary_check: confirm admission no longer opens FileBackedPaymentStateStore directly and only consumes payment-domain query helpers -> clean (authority.rs:18-20 imports only `PaymentIdempotencyKey, consumed_spend_capability_recorded, lookup_payment_idempotency_entry`. `consumed_spend_capability_refs_for_admission` (line 91) and `block_unavailable_idempotency_replay` (line 108) both go through the domain helpers; no `FileBackedPaymentStateStore::open` calls remain in this file. Error context strings are generic ("reading payment state for admission", "reading payment state for replay lookup").)
- `crates/runx-runtime/src/payment_state.rs`: schema_migration_check: verify v2 bump and that v1 files fail clearly via UnsupportedSchemaVersion plus duplicate-error rename hygiene -> clean (PAYMENT_STATE_SCHEMA_VERSION = "runx.payment_state.v2" (line 16). `FileBackedPaymentStateStore::open` (line 187) returns `PaymentStateError::UnsupportedSchemaVersion { schema_version }` on mismatch. Persisted field is `rail_mutations` (line 117); error variant is `RailMutationAlreadyRecorded` (line 170); status enum is `RailMutationStatus { Partial, Fulfilled, Escalated }` (line 89). harden-1/2/3 advisories all landed.)
- `crates/runx-runtime/src/payment_ledger.rs`: convention_check: typed readers replace local JSON path parsing and x402 substring predicates are gone -> clean (Imports from `crate::payment_packets::{read_paid_tool_packet, read_payment_rail_packet, read_payment_refusal_packet, read_payment_reservation_packet}` (line 18). Eligibility gate inside `persist_x402_payment_ledger_projection_event` (line 362) is `graph_receipt.seal.disposition == ClosureDisposition::Closed && any(steps).has_payment_reservation_packet`. No receipt-id substring matching, no harness URI inference.)
- `crates/runx-runtime/src/payment_packets.rs`: dark_patterns: typed readers must fail loud on missing/invalid fields and handle nested refusal placement without silently defaulting -> clean (PaymentPacketError differentiates `MissingField{field}` vs `InvalidField{field}` with `&'static str` field paths. `required_u64` distinguishes "present but invalid" (returns InvalidField) from "absent" (returns MissingField). `read_payment_refusal_packet` falls back to nested `payment_reservation_packet.data.payment_refusal_packet` for the cap-exceeded fixture shape. Empty strings are treated as missing via `string_field` (line 237).)
- `fixtures/harness/x402-pay-paid-echo.yaml + sibling x402 fixtures`: ambient_check: confirm no other fixture silently relies on the removed eligibility inference -> clean (Only x402-pay-paid-echo.yaml declares `payment_ledger_profile: x402-pay`. Other x402-pay-* fixtures terminate with non-closed dispositions and would be skipped by the `ClosureDisposition::Closed + has_payment_reservation_packet` gate inside `persist_x402_payment_ledger_projection_event`. The Stripe SPT fixture has no projection metadata and its test does not assert a projection event.)
- `task vs ambient classification`: scope_drift: separate this task's changes from canonical-json / contracts maturity / sandbox / registry edits -> clean (Ambient drift listed in the dossier (canonical-json fixtures, packages/contracts, registry, sandbox.rs, runx-core/policy maturity, runx-contracts maturity) is unrelated to the payment-execution boundary slice and is not attributed to this task.)

Findings:
- none

## Self Eval

Status: not_started
Completeness: none
Architecture fidelity: none
Spec alignment: none
Validation depth: none
Total: none
Second pass performed: none

Notes:
none

Improvements:
- none

## Deviations

- none

## Metadata

Estimated effort hours: 3
Actual effort hours: none
AI model: gpt-5
React cycles: none

Tags:
- rust
- payment
- execution-boundary
- x402

## Origin

Source:
- user asked whether the execution layer is clean and whether Stripe was
  incorrectly shoehorned into Rust; follow-up asked to spec and execute the
  cleanup with Claude harden and review.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- follow-up_to: x402-pay-idempotency-recovery-v1
- follow-up_to: x402-pay-ledger-projection-v1

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-21T12:18:34Z
Ended: 2026-05-21T12:18:34Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Boundary cleanup is well-scoped and executable. All declared paths exist (the only "missing" file, `payment_packets.rs`, is intentional Phase 1 output). Acceptance greps and `cargo test --test` filters resolve against real targets; the named CLI test exists at `crates/runx-cli/tests/x402_native_dogfood.rs:144`. Stripe production-source grep is already clean today. Architectural direction is sound: extracting typed packet readers and moving payment-state writes out of the generic runner is the right slice. Surfaced advisory issues: the write-boundary cleanup is asymmetric because `execution/runner/authority.rs` still opens `FileBackedPaymentStateStore` directly for admission lookups; the `MockRailMutationAlreadyRecorded` error variant and `mock_rail_mutations` JSON key require explicit handling during the rename; v3's grep pattern has minor redundancy. None block approval.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/payment_state.rs:1, code:crates/runx-runtime/src/payment_ledger.rs:1, code:crates/runx-runtime/src/execution/runner/steps.rs:1, code:crates/runx-runtime/tests/payment_state.rs:1, code:crates/runx-cli/tests/x402_native_dogfood.rs:144
  - Result: passed
  - Evidence: All touchpoint files exist except `crates/runx-runtime/src/payment_packets.rs`, which is intentionally produced in Phase 1 (declared as `all, exclusive` new module). The named CLI test function `native_x402_stripe_spt_happy_path_runs_without_typescript` is defined at line 144 of `x402_native_dogfood.rs`. `crates/Cargo.toml` exists and is the workspace manifest referenced by the acceptance commands.
- command audit
  - Grounded in: code:crates/Cargo.toml:1, code:crates/runx-runtime/Cargo.toml:18, code:crates/runx-cli/tests/x402_native_dogfood.rs:144
  - Result: passed
  - Evidence: Acceptance v4 uses `--test payment_state --test payment_ledger_projection --test payment_execution --test stripe_spt_payment` and all four files exist under `crates/runx-runtime/tests/`. The runtime Cargo.toml has no explicit `[[test]]` entries, so Cargo's default file-based test discovery applies. Acceptance v5 names a real test function. Greps in v1/v2 target real source paths. v3 grep paths (`crates/runx-core`, `crates/runx-runtime/src`, `crates/runx-cli/src`) all return zero matches today, confirming the assertion is achievable.
- scope/migration audit
  - Grounded in: code:crates/runx-runtime/src/execution/runner/authority.rs:18, code:crates/runx-runtime/src/payment_state.rs:70, code:crates/runx-runtime/src/error.rs:8
  - Result: passed
  - Evidence: Production `MockRail` references are confined to `payment_state.rs`, `execution/runner/steps.rs`, and `tests/payment_state.rs`. `error.rs` only imports `PaymentStateError` (no `MockRail` strings), so the rename does not silently expand scope. CLI sources contain zero `MockRail`/`mock_rail` matches. However, scope omits `execution/runner/authority.rs`, which also directly opens `FileBackedPaymentStateStore` for admission lookups — this is intentional per the spec's `runner may still call one small domain hook` framing but creates an asymmetric cleanup (writes leave, reads stay). Flagged as advisory issue.
- acceptance timing audit
  - Grounded in: spec_gap:phases
  - Result: passed
  - Evidence: v1/v2 (steps.rs and runtime grep for packet/mock names) become satisfiable only after Phase 2. v3 (Stripe production grep) is already clean today and only needs to stay clean. v4 depends on Phase 2 test rename. v5 depends on Phase 3 only if names changed. v6 runs at end. Phase ordering — Phase 1 (typed packets) → Phase 2 (state boundary + steps.rs migration) → Phase 3 (regression) — aligns with the acceptance gates.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Per-phase rollback is `git checkout HEAD -- <listed files>` plus `rm -f crates/runx-runtime/src/payment_packets.rs`. The file list covers every touchpoint that gets modified, and the new module is the only fresh artifact. Local payment-state.json files written under the renamed schema would fail to load against an older binary after rollback (deny_unknown_fields), but the spec explicitly accepts this as a clean cutover for local-only state.
- design challenge
  - Grounded in: code:crates/runx-runtime/src/execution/runner/steps.rs:88, code:crates/runx-runtime/src/execution/runner/authority.rs:97
  - Result: passed
  - Evidence: This is the right architectural move, not a bandaid: typed packet readers eliminate stringly-typed JSON path duplication between `steps.rs` and `payment_ledger.rs`, removing `MockRail` from production names removes a long-standing leak, and keeping receipt-before-success keyed on `AuthorityVerb::Spend` + `ProofKind::PaymentRail` preserves the invariant. The spec correctly defers a generic event bus as out-of-scope; payment is currently the only domain leaking into the runner, so a small typed hook is pragmatic. The remaining authority.rs read coupling is acknowledged but worth surfacing as a follow-up.

Issues:
- [medium/advisory] `harden-1` scope_gap - Authority admission still opens `FileBackedPaymentStateStore` directly; only the write path moves to the payment domain.
  - Status: open
  - Grounded in: code:crates/runx-runtime/src/execution/runner/authority.rs:97, code:crates/runx-runtime/src/execution/runner/authority.rs:124
  - Evidence: `consumed_spend_capability_refs_for_admission` and `block_unavailable_idempotency_replay` in `runner/authority.rs` both call `FileBackedPaymentStateStore::open(&path)` to read consumed-capability and idempotency state during admission. The spec moves the persistence writes out of `steps.rs` and asserts via v2 grep that production runtime has no `MockRail|mock_rail` names, but it does not address the symmetric admission-read path. The cleanup therefore leaves the runner module reading payment state via a typed path that bypasses any payment-domain query function.
  - Recommendation: Either acknowledge the asymmetry explicitly in the spec's Risks section (and explain why the read coupling is acceptable for this slice), or extend Phase 2 to introduce a payment-domain query function (e.g., `payment_state::lookup_for_admission`) that `authority.rs` consumes instead of opening the store directly. The latter would let a future event-bus refactor relocate the read side without touching the runner.
  - Question: Is the admission-side direct read from `FileBackedPaymentStateStore` in `runner/authority.rs` intentionally out of scope, or should this slice also wrap it behind a payment-domain function?
  - Recommended answer: Out of scope for this slice; document the asymmetry in Risks and leave the read path to a follow-up that introduces a generic event/query boundary.
  - If unanswered: Treat the admission read as intentionally out of scope and add one line to Risks noting the asymmetric cleanup.
- [low/advisory] `harden-2` scope_completeness - Rename surface is broader than the listed types: it must also cover the `MockRailMutationAlreadyRecorded` error variant, the `mock_rail_mutations` JSON field, and the `recording mock payment rail mutation` error-context string.
  - Status: open
  - Grounded in: code:crates/runx-runtime/src/payment_state.rs:154, code:crates/runx-runtime/src/payment_state.rs:101, code:crates/runx-runtime/src/execution/runner/steps.rs:197
  - Evidence: `PaymentStateError::MockRailMutationAlreadyRecorded` (line 154) and the `PaymentStateDocument.mock_rail_mutations` field (line 101) both contain `MockRail|mock_rail` substrings and will fail v2 grep if not renamed. `steps.rs:197` passes the literal string `"recording mock payment rail mutation"` to `RuntimeError::payment_state` — also caught by v2. Phase 2's change line for `payment_state.rs` says only `rename rail mutation state types`, which under-specifies these adjacent surfaces.
  - Recommendation: Expand the Phase 2 description for `payment_state.rs` to enumerate: (a) struct/enum rename (`MockRailMutation` → `RailMutation`, `MockRailMutationStatus` → `RailMutationStatus`), (b) error variant rename (`MockRailMutationAlreadyRecorded` → `RailMutationAlreadyRecorded`), (c) JSON document field rename (`mock_rail_mutations` → `rail_mutations`), and (d) update the error-context string in `steps.rs`. v2 grep will already enforce these, but listing them avoids a mid-build surprise.
- [medium/advisory] `harden-3` migration_risk - Renaming the `mock_rail_mutations` document field without bumping `PAYMENT_STATE_SCHEMA_VERSION` leaves stale on-disk files unreadable with the existing v1 version tag.
  - Status: open
  - Grounded in: code:crates/runx-runtime/src/payment_state.rs:10, code:crates/runx-runtime/src/payment_state.rs:96
  - Evidence: `PAYMENT_STATE_SCHEMA_VERSION = "runx.payment_state.v1"` is paired with `#[serde(deny_unknown_fields)]` on `PaymentStateDocument`. After the rename, any pre-existing `payment-state.json` with the old key will deserialize as a parse error tagged as schema v1, which conflates a field rename with a corruption signal. The spec acknowledges this as a clean-cutover risk but does not direct whether to bump to `runx.payment_state.v2` or keep v1.
  - Recommendation: Decide and record one of: (1) bump to `runx.payment_state.v2` so the failure mode is `UnsupportedSchemaVersion` (a clear, recoverable signal), or (2) explicitly keep v1 and accept that stale files must be deleted (document the recovery step in the Rollback or Risks section). Bumping the version is the lower-friction option since this is local-only state.
  - Question: Should the schema version bump to `runx.payment_state.v2` so existing on-disk files surface as `UnsupportedSchemaVersion` rather than a parse error?
  - Recommended answer: Yes — bump to `v2`. The clean cutover is cheaper than litigating a parse error against a renamed field.
  - If unanswered: Bump `PAYMENT_STATE_SCHEMA_VERSION` to `runx.payment_state.v2` and add a one-line note in Risks.
- [low/advisory] `harden-4` validation_hygiene - v3 grep alternation contains redundant patterns (`stripe-spt` is a substring of `stripe`).
  - Status: open
  - Grounded in: spec_gap:acceptance.v3
  - Evidence: The v3 pattern `stripe|stripe-spt|STRIPE|PaymentIntent|payment_intent` is case-sensitive ripgrep. `stripe-spt` is unreachable because any string containing it already contains `stripe`. `STRIPE` provides legitimate uppercase coverage. The redundancy is harmless but reads as if uppercase + a special-cased substring were both meaningful checks.
  - Recommendation: Trim to `stripe|STRIPE|PaymentIntent|payment_intent`, or use `-i` and drop the explicit `STRIPE`. Cosmetic only.


## Planning Log

- 2026-05-21T13:05:00Z: Drafted from execution-layer architecture review after
  live Stripe SPT dogfood.
- 2026-05-21T13:30:00Z: Folded operator review into active scope: remove
  x402 substring projection gate, treat `mock_rail_mutations` as schema
  contamination, keep replay fail-closed while wrapping admission reads.
