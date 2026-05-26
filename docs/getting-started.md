# Getting Started

This walkthrough proves the local runx path with one small skill. It uses the
checked-in `examples/hello-world` package so the commands stay tied to the repo.

## Prerequisites

- Rust 1.85 or newer for the native CLI path.
- Node.js 20 or newer for the checked-in `hello-world` runner command. No
  TypeScript install is required for the native CLI path.
- pnpm 10 or newer only when exercising the npm wrapper or TypeScript package
  tests.

Build the native CLI from the OSS workspace:

```bash
cd oss
cargo build --manifest-path crates/Cargo.toml -p runx-cli
```

## Run The Example

Run the skill directly through the CLI:

```bash
crates/target/debug/runx skill examples/hello-world \
  --message "hello from docs" \
  --non-interactive \
  --json
```

The JSON response should report `status: "sealed"` and include a receipt id.
The npm wrapper may be used for package-distribution checks, but it should
delegate to the same Rust binary behavior.

## Inspect The Receipt

Receipts are written under `.runx/receipts` unless `RUNX_RECEIPT_DIR` is set.
Use the id from the previous command as a history query:

```bash
crates/target/debug/runx history <receipt-id> --json
```

The history projection should show a `runx.receipt.v1` receipt. In
local development that receipt is structurally checked but not
production-verified, because the default deterministic pseudo signature is only
for stable local fixtures. It is still durable evidence that runx executed the
skill, recorded the input shape, and captured the output without relying on
prose claims.

## Production Receipt Signing

Local development uses deterministic pseudo signatures so fixtures stay stable.
For production-trusted receipts, configure an Ed25519 signing key before running
skills, graphs, harness replay, or MCP server calls:

```bash
export RUNX_RECEIPT_SIGN_KID="hosted-prod-key"
export RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64="<32-byte-ed25519-seed-base64>"
export RUNX_RECEIPT_SIGN_ISSUER_TYPE="hosted"
```

All three variables must be set together. `RUNX_RECEIPT_SIGN_ISSUER_TYPE` must
be `hosted` or `ci`; production receipts are never stamped as local issuers.
When configured, the runtime signs each receipt body digest with Ed25519 and
writes the matching public key hash in the issuer metadata. To have
`runx history` report those receipts as production-verified, provide the public
verification key to the same command:

```bash
export RUNX_RECEIPT_VERIFY_KID="hosted-prod-key"
export RUNX_RECEIPT_VERIFY_ED25519_PUBLIC_KEY_BASE64="<32-byte-ed25519-public-key-base64>"
crates/target/debug/runx history <receipt-id> --json
```

## Next

- Use `crates/target/debug/runx new docs-demo` for local standalone skill
  scaffolding.
- Use `npm create @runxhq/skill@latest docs-demo` when starting from npm.
- Compose the example into a graph with [Skill To Graph](./skill-to-graph.md).
- See [API Surface](./api-surface.md) for public package exports.
