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

The history projection should show a verified `runx.harness_receipt.v1` receipt.
That receipt is the durable evidence that runx executed the skill, recorded the
input shape, and captured the output without relying on prose claims.

## Next

- Use `crates/target/debug/runx new docs-demo` for local standalone skill
  scaffolding.
- Use `npm create @runxhq/skill@latest docs-demo` when starting from npm.
- Compose the example into a graph with [Skill To Graph](./skill-to-graph.md).
- See [API Surface](./api-surface.md) for public package exports.
