# Payments Reference

Use this reference for funding, payouts, refunds, target changes, chargebacks,
settlement health, and payment reconciliation.

## Rule

Money state changes only after rail proof becomes a receipt/effect. UI state,
provider optimism, local API success, or agent narration is not settlement.

The ops desk spine is rail-neutral. It selects the governed payment family and
the configured rail adapter, then stops at the right approval or signature gate.
Rail-specific funding, wallet, webhook, dispute, and settlement details belong
in the rail adapter skill or the product-owned operator skill that owns that rail.

## Common Lanes

- `payment.quote`: read-only or proposal; no approval.
- `payment.reserve`: authority reservation or cap check; may require approval.
- `payment.fund`: money movement; approval or payer signature required.
- `payment.payout`: money movement; approval required.
- `payment.refund`: money movement; approval required.
- `payment.target_update`: rail configuration; approval required.
- `payment.reconcile`: read-only unless it creates corrections.
- `payment.dispute_response`: customer/provider communication; approval
  depends on whether it submits externally.

## Rail Adapter Contract

A payment rail adapter must make these fields explicit before settlement:

- operation: quote, reserve, fund, payout, refund, dispute, or reconcile;
- amount and currency;
- payer, payee, counterparty, or refund target;
- network, rail, account, asset, or processor path;
- cap, expiry, and idempotency key;
- approval or payer-signature requirement;
- settlement proof shape;
- readback source after settlement.

Do not infer cross-rail compatibility. A balance, address, token, processor
account, webhook, or credential on one rail does not imply readiness on another
rail. If a product supports multiple rails, each rail has its own adapter status,
target configuration, proof, and reconciliation readback.

## Operator Packet Requirements

For each payment proposal include:

- payer/payee refs, redacted when necessary;
- amount and currency;
- rail adapter and network/account path;
- quote, reservation, approval, or settlement refs;
- expiry or idempotency key;
- approval requirement;
- expected receipt/effect;
- reconciliation readback.

## Stop Conditions

- Missing amount, payee, rail adapter, quote, approval, signature, or
  idempotency key.
- Requested manual funded/paid/refunded marking without receipt-backed proof.
- Network, asset, account, or rail mismatch between quote and payer funds.
- Target update requested without explicit operator approval.
- Refund or payout amount not tied to the original settlement or policy.
- Payout requested for a claim, invoice, or obligation that has not reached the
  product's payable state.
- Rail-specific funding or recovery requested without loading the rail adapter
  or product runbook that owns that procedure.
