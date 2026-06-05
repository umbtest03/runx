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

## x402 receipt demo

```bash
./x402.sh
```

Without x402 environment variables this writes a deterministic mock transcript.
With a Runx-compatible operator signer and facilitator exported in the calling
shell, it performs a real testnet settlement through the Runx signer/facilitator
seam and verifies both receipts offline:

```bash
export RUNX_X402_DEMO_MODE=live
export RUNX_X402_FACILITATOR=https://...
export RUNX_X402_SIGNER=https://...
export RUNX_X402_CHAIN_ID=84532
export RUNX_X402_TOKEN_CONTRACT=0x...
export RUNX_X402_VERIFYING_CONTRACT=0x...
export RUNX_X402_FROM=0x...
export RUNX_X402_PAY_TO=0x...
./x402.sh
```

The signer endpoint receives the runx-bound EIP-712 template and returns only a
signature. runx never stores the wallet key.

This script is not, by itself, an upstream x402 protocol conformance test. It
proves the Runx receipt, authority, signer, and settlement-recording seam. Use the
upstream conformance process below to prove the standard HTTP 402 flow.

## Upstream x402 conformance process

Use this when you need to prove the x402 shape itself, not a runx-authored mock.
The source of truth is the upstream standard repository:

```bash
git clone https://github.com/x402-foundation/x402 /tmp/x402-upstream
cd /tmp/x402-upstream
git rev-parse HEAD
```

Install the upstream e2e runner from the official checkout:

```bash
cd /tmp/x402-upstream/e2e
pnpm install:all
```

From the Runx OSS checkout, preflight the exact upstream scenario:

```bash
pnpm x402:conformance
```

The preflight records the upstream commit SHA and prints missing environment
variables without reading or writing secrets. The minimal official scenario is
the TypeScript facilitator + Express resource server + fetch client, filtered to
Base Sepolia EVM, exact settlement, and `/exact/evm/eip3009`:

```bash
pnpm --dir /tmp/x402-upstream/e2e test \
  --testnet \
  --families=evm \
  --versions=2 \
  --schemes=exact \
  --clients=fetch \
  --servers=express \
  --facilitators=typescript \
  --endpoints=/exact/evm/eip3009 \
  --min \
  --output-json=/tmp/runx-x402-upstream-conformance/x402-upstream-e2e.json \
  --log=/tmp/runx-x402-upstream-conformance/x402-upstream-e2e.log
```

Run it through the Runx wrapper when dedicated funded test wallets are ready:

```bash
export X402_UPSTREAM_DIR=/tmp/x402-upstream
export RUNX_X402_CONFORMANCE_ARTIFACT_DIR=/tmp/runx-x402-upstream-conformance
export SERVER_EVM_ADDRESS=0x...
export CLIENT_EVM_PRIVATE_KEY=0x...
export FACILITATOR_EVM_PRIVATE_KEY=0x...
export SERVER_SVM_ADDRESS=...
export CLIENT_SVM_PRIVATE_KEY=...
export FACILITATOR_SVM_PRIVATE_KEY=...
node scripts/x402-upstream-conformance.mjs --run
```

The current upstream runner checks the SVM variables before applying the EVM-only
filter, so they are required even for this EVM-only scenario. Use dedicated
testnet wallets only; the upstream e2e runner may move funds between configured
wallets as part of normal setup/cleanup.

If you only need the narrower upstream SDK-level Base Sepolia settle check, use
the upstream EVM package integration test instead:

```bash
cd /tmp/x402-upstream/typescript/packages/mechanisms/evm
export CLIENT_PRIVATE_KEY=0x...
export FACILITATOR_PRIVATE_KEY=0x...
pnpm exec vitest run --config vitest.integration.config.ts test/integrations/exact-evm.test.ts
```

That is useful for rail mechanics, but it is not the full HTTP 402
client/server/facilitator conformance run.

Rules for a clean conformance run:

1. Do not patch or copy upstream protocol code into runx.
2. Record the upstream commit SHA beside the run output.
3. If an upstream example needs configuration, set environment variables only;
   do not commit secrets, private keys, generated wallets, or `.env` files.
4. Do not use the upstream `mock-facilitator` as settlement proof. It is a
   startup fallback and intentionally errors if `/verify` or `/settle` are called.
5. If you also need Runx receipt proof for the same rail, run `./x402.sh` with
   `RUNX_X402_DEMO_MODE=live` against a compatible signer/facilitator seam after
   the upstream conformance run succeeds.

The run is accepted only when:

- `node scripts/x402-upstream-conformance.mjs --run` succeeds from a clean
  upstream checkout at a recorded commit.
- The upstream output JSON records a successful TypeScript facilitator + Express
  server + fetch client scenario for `/exact/evm/eip3009`.
- If `./x402.sh` is also run, it reports `mode: live` and `operator_keyed: true`,
  and the settlement has a non-mock `tx_hash` / rail reference.
- Both `x402-settlement.receipt.json` and `x402-refusal.receipt.json` verify with
  `node examples/governed-spend/verify.mjs` when the Runx receipt demo is run.

If any of those fail, call it a local mock or conformance failure, not a real x402
test.

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
rails run through deterministic test supervisors by default. The optional x402 and
Stripe SPT scripts can call test networks/providers when operator-provided
credentials are present. The refusal needs no rail, which is the point.
