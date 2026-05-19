---
spec_version: '2.0'
task_id: payment-authority-term-v1
created: '2026-05-20T00:00:00Z'
updated: '2026-05-20T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: high
---

# Payment authority term v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft for making payment a real runx authority term on the existing
harness spine.
Blockers: current harness spine schemas and Rust receipt/harness execution stay
authoritative.
Allowed follow-up command: `scafld harden payment-authority-term-v1`
Latest runner update: none
Review gate: not_started

## Summary

Define payment as an authority term, not a parallel durable payment object.
Payment challenge, reservation, rail execution, receipt proof, and ledger views
must be represented by existing spine concepts: Signal, Decision, Harness, Act,
receipt payload/reference, and projection.

Current repo-state caveat: Rust policy admission is still string-scope based
(`scopes`, `scope_family`, wildcard/prefix matching). This spec does not pretend
that model is enough for money movement. It introduces the payment authority
algebra and subset rules that Rust policy must enforce before any rail skill can
spend.

## Problem

Connected-auth scope admission can answer "does this skill scope fit this
grant?" It cannot answer "is this attempted payment a subset of the reserved
spend authority?" without typed amount, currency, merchant/rail, recipient,
purpose, expiry, idempotency, and receipt constraints.

The wrong shape would add durable `Payment`, `Reservation`, or `LedgerEntry`
objects beside Harness and Receipts. That would split auditability, retries, and
authority attenuation across two systems. Payment must instead be a domain term
inside the authority algebra, and durable state must remain the harness receipt
tree plus projections.

## Contract

A payment authority term is a regular spine `authorityTerm` with a payment
resource family and payment-specific bounds/conditions.

Term fields:
- `resource_family`: add `payment`.
- `verbs`: add or use payment verbs `reserve`, `spend`, `refund`, `void`,
  `verify`. `spend` is the only rail-mutating verb.
- `resource_ref`: identifies the merchant account, wallet/account handle, rail
  profile, or logical payment program. It must not contain wallet secrets.
- `principal_ref`: actor or child harness principal allowed to exercise the
  term.
- `bounds`: payment constraints carried in the existing bounds object plus a
  typed payment extension until the schema is promoted.
- `conditions`: require `decision_selected`, `within_budget`,
  `within_time_window`, and rail-proof criteria as applicable.
- `approvals`: human/system approvals that authorize the reservation or spend.
- `capabilities`: must include a spend capability token for `spend`; reserve
  terms may omit it.
- `expires_at`, `issued_by_ref`, `credential_ref`: same semantics as current
  authority terms. `credential_ref` may point to a secret-handle reference but
  never raw secret material.

Payment bounds:
- `amount_minor`: integer minor units.
- `currency`: ISO 4217 currency code.
- `recipient_ref`: payee, merchant, wallet, or invoice reference.
- `rail_ref`: rail/provider reference or allowlist.
- `merchant_ref`: optional merchant/account reference.
- `purpose`: stable business purpose.
- `idempotency_key`: stable key for rail retries.
- `single_use`: true for spend capability terms.
- `not_before` / `expires_at`: temporal spend window.

Subset / partial-order rules:
- Child term `resource_family` must equal parent `payment`.
- Child verbs must be a subset of parent verbs; `spend` cannot be derived from a
  parent that only has `reserve` or `verify`.
- Child amount must be less than or equal to the reserved amount and in the same
  currency.
- Child recipient, merchant, rail, and purpose must equal the parent or be a
  member of the parent's explicit allowlist. No wildcard recipient for `spend`.
- Child time window must be inside the parent time window.
- Child `idempotency_key` must equal the reservation key or be a deterministic
  child key bound to the reservation Decision and child Harness id.
- Child capabilities must be a subset of parent capabilities. A `spend`
  capability is single-use and cannot be copied into sibling child harnesses.
- Additional child conditions/approvals may narrow the term; removing parent
  conditions or approvals is widening and must fail subset proof.

## Spine Mapping

- Challenge: a payment request is a `Signal` from an invoice, checkout event,
  internal request, or verified operator intent. It carries source/evidence refs
  and no rail secret.
- Reservation: the parent Harness records a `Decision` selecting the payment
  intent. The Decision artifact/metadata contains the reserved payment bounds,
  authority subset proof, and idempotency key.
- Fulfillment: the actual rail call runs in a child `Harness` with attenuated
  payment authority and one terminal `Act` for the rail operation.
- Rail proof: the rail response is stored as a receipt payload or referenced by
  a receipt ref. Sensitive rail data is redacted/committed by hash; the receipt
  ref is the proof surface.
- Ledger: account history, spend reports, and reconciliation are projections
  over Harness receipts and rail proof refs. They are rebuildable views, not
  independent durable payment truth.

## Correctness Rules

- Single-use spend capability: a `spend` term may be consumed once. The consumed
  capability is bound to the child Harness id, Act id, reservation Decision id,
  amount, currency, recipient, rail, and idempotency key.
- No wallet secret to rail skill: rail skills receive only scoped secret handles
  or provider session refs. Raw wallet keys, seed phrases, card PAN/CVV, and bank
  credentials never appear in Signal, Decision, Act, receipt payload, or logs.
- Reserve before rail: a child rail Harness cannot start unless the parent
  Harness has a selected Decision with payment bounds and a passing subset proof.
- Crash recovery by idempotency key: retries reuse the same idempotency key and
  reconcile the rail state before attempting another mutation. A recovered rail
  success seals the Harness from the existing rail proof.
- Receipt before success: runx must not report payment success until the child
  Harness receipt includes the rail proof ref/payload and verifies the Act
  criteria. A successful rail call with missing receipt proof is an incomplete
  Harness, not success.
- No parallel durable payment objects: reservation state, spend outcome, and
  proof live in Harness/Decision/Act/receipt data. Any ledger table or API is a
  projection with source receipt refs.

## Implementation Plan

1. Contracts: extend the spine schema in
   `packages/contracts/src/schemas/spine.ts` with `payment` resource family,
   payment verbs if needed, and typed payment bounds/conditions. Keep the shape
   embedded in authority terms; do not add a top-level payment schema.

2. Rust contracts/core: mirror the new enum values and typed payment authority
   structs in `crates/runx-contracts` and `crates/runx-core`. Add subset
   comparison tests for amount, currency, recipient, rail, purpose, time window,
   idempotency key, conditions, approvals, and single-use capabilities.

3. Rust policy: add payment authority admission beside current string-scope
   admission in `crates/runx-core/src/policy`. Existing connected-auth scope
   checks may still guard provider access, but rail mutation must require the
   payment subset proof.

4. Runtime harness: in `crates/runx-runtime`, require reserve-before-rail when a
   skill attempts a payment rail Act. Spawn the rail call as a child Harness with
   attenuated payment authority, record the reservation Decision metadata, and
   seal only after receipt proof exists.

5. Receipts/projections: in `crates/runx-receipts` and runtime receipt helpers,
   add fixture coverage for redacted rail proof refs and rebuildable ledger
   projections sourced from Harness receipts.

6. Fixtures/tests: add focused fixtures under the existing harness/runtime
   fixture tree for allowed spend, amount widening denial, recipient widening
   denial, duplicate idempotency recovery, missing receipt proof, and sibling
   reuse of a single-use spend capability.

## Out Of Scope

- Building a wallet, payment processor, reconciliation service, or accounting
  system.
- Persisting a durable payment domain model outside Harness receipts.
- Selecting a specific rail provider.
- General-purpose finance ledger semantics beyond receipt-backed projections.

## Open Questions

- Exact `payment` bounds schema location inside spine: extend
  `authorityBounds` directly or add a typed `payment` sub-object under bounds.
  The implementation should pick the smallest schema change that preserves
  strict validation.
- Whether reserve-only terms should use a new `reserve` verb or model
  reservation as `approve` plus payment conditions. This draft prefers explicit
  `reserve` because it makes the partial order clearer.
