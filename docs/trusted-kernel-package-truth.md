# Trusted Kernel Package Truth

Status: accepted OSS addendum for the Rust parity track.

The repo-root `docs/trusted-kernel-package-truth.md` remains the broad package
authority document for the full runx repository. This OSS-local addendum
records the Rust parity boundary in the same docs tree as the Rust architecture
plan, so scafld specs that run from `oss/` have a stable local path.

## Rust Parity Rule

TypeScript remains the source of truth for trusted-kernel behavior until a
separate cutover spec changes a consumer and passes the relevant parity gate.
Rust crates may provide distribution, SDK, or fixture-parity implementations,
but they do not become authoritative by existing.

Local Rust kernel parity is checked with `pnpm rust:check`, which runs Cargo
formatting, clippy, workspace tests, crate graph/style guards, `cargo-deny`,
and the `runx-core` public API snapshot. In CI this remains advisory during
Phase A; it becomes blocking only through the `rust-kernel-blocking-promotion`
spec after five clean kernel-touching PRs.

Kernel parity fixtures live under `fixtures/kernel/`. They are generated from
the TypeScript implementation and act as conformance evidence for the Rust
port. Fixture refreshes must be deliberate: update the TypeScript oracle,
regenerate the fixture JSON, and review the semantic diff before accepting a
Rust behavior change.

`crates/runx-core` currently provides Rust state-machine parity and Rust
policy parity against the checked-in fixture set. Rust policy parity status: fixture-evidence-only.
It is not an authoritative replacement for `@runxhq/core/state-machine` or
`@runxhq/core/policy`, and no runtime-local, adapter, MCP, receipt, or CLI
execution path should call Rust policy until an explicit binding/cutover spec
changes ownership.

Policy executable-name normalization is host-independent for fixture parity:
backslashes are treated as path separators on every host. This keeps strict
CLI-tool inline-code admission consistent across POSIX and Windows runners;
for example, `C:\Tools\node.exe -e ...` normalizes to `node` and is denied
under the strict inline-code policy.

The initial pure-kernel Rust parity surface is:

- `@runxhq/core/state-machine`
- `@runxhq/core/policy`
- `@runxhq/core/policy/sandbox`

Parser, receipts, runtime, adapters, and CLI cutover are separate specs.
The authority-proof and public-work re-exports are deferred to a follow-up spec.
Full CLI/runtime cutover still requires the `fixtures/cli-parity` feature
matrix and one-to-one TypeScript oracle parity; kernel parity alone is not a
CLI or runtime cutover gate.
