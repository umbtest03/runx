# runx-contracts

Shared Rust contract types at runx JSON and protocol boundaries.

This crate owns stable serde shapes for host protocol, act assignment,
idempotency hashes, CLI JSON envelopes, receipts, registry/tool records, and
other public wire contracts as those surfaces gain fixture parity. It does not
execute skills, evaluate policy, perform IO, or replace the TypeScript
implementation.

The current surface is intentionally small:

- `json`: contract-owned JSON values backed by deterministic `BTreeMap`
  objects.
- `act::assignment`: typed act assignment envelopes and
  fixture-backed idempotency hashes.
- `host_protocol`: the serializable host result/state/event subset consumed by
  SDK v0. `AgentActInvocation` is typed, but its `envelope` remains an opaque
  contract JSON payload until `rust-resolution-payload-parity` owns the deeper
  agent-act payload model.

Deferred contract module homes:

- `cli`: Deferred contract module home for CLI JSON. Owned by
  `rust-contracts-cli-json-parity` after `rust-cli-feature-parity-matrix`
  produces the CLI JSON oracle.
- `receipts`: deferred to `rust-receipts-parity`.
- `registry`: deferred to `rust-registry-parity`.
- `tools`: deferred to `rust-tools-parity`.

contracts-first-ordering: SDK Phase 2 consumes `runx-contracts` for JSON,
act assignment, host protocol, and hashes. It must not duplicate these
types in `runx-sdk`.

Workspace fixtures under `fixtures/contracts` are not vendored into this crate.
The Cargo package `include allowlist` ships only `Cargo.toml`, `README.md`, and
`src/**/*.rs`, so fixture files are excluded from the packaged crate.

Deferred surfaces are declared only when their parity fixture owner exists.
That keeps this crate useful without turning it into a dumping ground for every
TypeScript interface.
