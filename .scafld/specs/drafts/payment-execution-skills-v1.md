---
spec_version: '2.0'
task_id: payment-execution-skills-v1
created: '2026-05-20T00:00:00Z'
updated: '2026-05-20T12:55:31Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# Payment execution skills v1

## Current State

Status: completed
Current phase: final
Next: harden
Reason: first-party payment skill skeletons and X.yaml profiles now exercise
the payment authority term through quote, reserve, approval, deterministic mock
rail fulfillment, and recovery inspection without claiming live runtime payment
behavior.
Allowed follow-up command: `scafld harden payment-execution-skills-v1`
Latest runner update: 2026-05-20T12:55:31Z
Review gate: pass

## Summary

Payment execution in runx is a governed graph over the existing spine. The
first-party skills make the flow legible to humans and registry tooling, but
they do not own payment truth. Core owns the spend decision, reservation,
idempotency lookup, authority subset proof, recovery path, and receipt-before-
success invariant. Rail skills run only below that gate with attenuated payment
authority and a single-use spend capability.

## Skill Set

`pay-quote`
: Turns a `payment_required` signal, MCP payment challenge, invoice request, or
operator intent into a quote packet plus requested payment authority bounds.
Settlement-agnostic; non-mutating; receives no rail secrets.

`pay-reserve`
: Selects or declines the payment intent. The output is a Decision-shaped
reservation packet containing payment bounds, idempotency key, approval status,
and the child authority term that may be passed to a settlement step.
Settlement-agnostic; does not settle.

`pay-recover`
: Reconciles an idempotency key after crash, timeout, retry, or ambiguous
settlement state. Must query by idempotency key before any repeat mutation and
returns a recovered proof, a safe retry recommendation, or an escalation.

`stripe-pay`, `mpp-pay`, `mock-pay`
: Settlement-pinned graph marquees. Each composes quote, reserve, optional
approval, and the named settlement family, then hands off to recover on
ambiguity. Each receives an already-reserved child authority term and a
single-use spend capability ref. Each returns a settlement proof payload/ref
suitable for sealing into the child harness receipt.

`crypto-pay`
: Reserved placeholder for on-chain settlement. Documented for naming
continuity so the slot is not reused later. Not exposed in the registry; no
SKILL.md, no X.yaml profile, and no harness case in this iteration.

`x402-pay`
: The unpinned graph marquee. Same composition as the settlement-pinned
marquees, but the settlement family is selected from policy and the inbound
challenge at runtime. This is the first "seamless agent payments" surface: a
paid tool call can enter as one request and leave as a sealed payment receipt
without hiding the governance steps.

## Spine Mapping

- Challenge input is `SignalType::PaymentRequired`.
- Quote output is evidence and requested authority, not spend.
- Reservation is a selected `Decision` and authority subset proof metadata.
- Rail fulfillment is a child `Harness` with one terminal `Act`.
- Rail proof is receipt payload/reference with sensitive fields redacted.
- Ledger/reporting remains a projection over sealed receipts.

## Core-Owned Rules

- Core compares child and parent `AuthorityTerm` values with the payment
authority partial order before settlement starts.
- Core reserves budget atomically by idempotency key before settlement runs.
- Core derives or validates the single-use spend capability for `spend`.
- Core performs idempotency recovery before retrying a mutating settlement
call.
- Core refuses success until the child harness receipt carries the settlement
proof.

## Skill-Owned Rules

- A quote skill may normalize protocol-specific challenge fields and recommend
payment bounds, but it cannot authorize spend.
- A reserve skill may present the human-readable decision record, but it cannot
mint broader authority than core admits.
- A settlement marquee may adapt one payment protocol/provider, but it cannot
set caps, read raw wallet secrets, or decide approval.
- A recovery skill may inspect idempotency state and settlement proof refs, but
it cannot hide an ambiguous spend as success.

## Initial Settlement Families

- `mock-pay`: deterministic local settlement for harnesses, demos, and contract
tests.
- `stripe-pay`: Stripe session/payment token settlement family.
- `mpp-pay`: multi-party payment protocol settlement family.
- `crypto-pay`: on-chain settlement family. Reserved placeholder, not exposed
or harnessed in this iteration.

`x402-pay` is the unpinned graph marquee that selects one of the above at
runtime from policy and the inbound challenge.

These names are first-party skill packages, not hardcoded core concepts. Core
only sees payment authority terms, idempotency keys, child harnesses, and
receipt proof refs.

## Acceptance Criteria

- Each first-party payment skill except the `crypto-pay` placeholder has a
human-readable `SKILL.md`.
- Each first-party payment skill except the `crypto-pay` placeholder has an
`X.yaml` profile with concrete inputs, outputs, artifacts, and harness cases.
- Graph profiles (`x402-pay`, `stripe-pay`, `mpp-pay`, `mock-pay`) make the
authority transition visible: quote -> reserve -> optional approval ->
settlement.
- Settlement profiles declare payment authority metadata under `runx` and
never declare raw secret inputs.
- Existing profile parsing validates all new payment X.yaml files.
- The `crypto-pay` slot is documented but neither installable nor harnessed in
this iteration.
- No runtime or CLI payment behavior is claimed until the runtime harness owns
reserve-before-settlement and receipt-before-success enforcement.
