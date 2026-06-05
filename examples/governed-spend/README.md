# governed-spend spend (demo)

A prompt-injected agent tries to overspend; runx refuses before any rail is
touched, identically across x402, MPP, and Stripe, and signs a receipt for every
outcome. The agent never holds the authority, so hijacking it changes nothing.

## Run it

```bash
./run.sh
```

No keys, no signup, no network. Needs the `runx` binary (a build under
`crates/target/`, `runx` on `PATH`, or `RUNX_BIN=/path/to/runx`) and `python3`.

The script prints three steps:

1. A bounded authority pays on `x402`, `mpp`, and `stripe`, each sealing a receipt.
2. The reserved child authority is capped at 100 minor; an injected agent tries to
   fulfill 125; runx blocks at the rail gate before any provider call.
3. A sealed refusal receipt: `disposition: refused`, `reason_code: cap_exceeded`,
   `rail_call_performed: false`.

## Over MCP

```bash
./mcp.sh
```

Serves `x402-pay`, `mpp-pay`, `stripe-pay`, and `overspend-refused` as MCP tools.
An agent calls a skill and gets the sealed receipt id, or the refusal, in one
round-trip.

## Recorded payments demo

```bash
node ../../scripts/payments-demo.mjs --record --receipt-dir /tmp/runx-payments-demo
node verify.mjs /tmp/runx-payments-demo/payments-demo-paid.receipt.json
node verify.mjs /tmp/runx-payments-demo/payments-demo-refusal.receipt.json
```

With `ANTHROPIC_API_KEY` and `RUNX_X402_SIGNER` present, the script records an
operator-keyed testnet transcript. Without those keys it writes a deterministic
mock transcript. In both modes the offline receipts are real signed artifacts:
one scoped x402 spend, then one over-run-cap refusal before money moves.

## Stripe SPT test-mode demo

```bash
./stripe-spt.sh
```

Without Stripe environment variables this writes a deterministic mock transcript.
With Stripe test-mode credentials exported in the calling shell, it performs a
real Stripe SPT test-mode charge and verifies both receipts offline:

```bash
export STRIPE_SECRET_KEY=sk_test_...
export STRIPE_WEBHOOK_SECRET=whsec_...
export RUNX_STRIPE_DEMO_MODE=live
./stripe-spt.sh
```

`STRIPE_TEST_KEY` is still accepted for older local setups. Live-mode keys are
refused; the script accepts only `sk_test_` or `rk_test_` keys and never writes
Stripe credentials to the receipt directory.

## Tweak it

In [`skills/overspend-refused/X.yaml`](skills/overspend-refused/X.yaml), raise the
reserved child authority's `max_per_call_units` from `100` to `125` and re-run: the
same agent now fulfills, because the spend is within its authority.

## What is real

The kernel, the quote/reserve/fulfill graph, the fail-closed authority subset proof,
the authority admission that refuses before any rail, and signed receipts are real and ship today. The
rails run through deterministic test supervisors by default. The optional Stripe
SPT script can call Stripe test mode when operator-provided test credentials are
present; x402 still requires the separate Base Sepolia rail build. The refusal
needs no rail, which is the point.
