# runx-sdk

Placeholder crate for the Rust SDK.

The first SDK version is planned as a CLI-backed client over documented
`runx --json` output plus typed host-protocol and act-assignment helpers.
It does not execute skills natively and does not replace the TypeScript
runtime.

SDK v0 depends on `runx-contracts` only. It must not depend on `runx-core`;
that keeps the SDK shippable before kernel parity is complete.
