# runx Rust Crates

This workspace contains Rust packages for runx distribution, SDKs, contracts,
and future portable runtime work.

The npm package `@runxhq/cli` remains the authoritative CLI implementation.
The first Cargo package is `runx-cli`, which installs a native `runx` launcher
that delegates to the latest npm CLI by default.

## Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo package --workspace --allow-dirty
node ../scripts/check-rust-crate-graph.mjs
node ../scripts/check-rust-core-style.mjs
```

## Layout

- `runx-cli`: Cargo-installable launcher binary named `runx`.
- `runx-contracts`: shared JSON boundary types today; host protocol and receipt
  contract parity follow in later specs.
- `runx-core`: pure state-machine parity today; policy parity follows next.
- `runx-parser`: placeholder for skill, graph, and tool parser parity.
- `runx-receipts`: placeholder for receipt model and verification parity.
- `runx-runtime`: placeholder for the future native runtime, including adapter
  features such as `cli-tool`, `mcp`, `a2a`, `agent`, and `catalog`.
- `runx-sdk`: placeholder for the CLI-backed Rust SDK. SDK v0 depends on
  `runx-contracts`, not `runx-core`.

Placeholder crates must not claim native feature parity. TypeScript remains
authoritative until each crate has its own fixture-backed parity spec.
The placeholder library crates are crates.io reservation releases at `0.0.1`;
`runx-core` also remains at `0.0.1` while it accumulates parity surfaces.
`runx-cli` is live at `0.1.0` because it installs the usable `runx` launcher.
These releases claim names first; real behavior still requires
fixture-backed implementation specs.

The runtime crate defaults to no adapter features. Adapter families are opt-in
features: `cli-tool`, `mcp`, `a2a`, `agent`, and `catalog`.

Commit the single workspace lockfile at `crates/Cargo.lock`; this workspace
contains a binary and publishable libraries.

Rust skill authoring helpers stay inside `runx-cli` or `runx-sdk` until there
is a concrete library use case for a separate authoring crate.
