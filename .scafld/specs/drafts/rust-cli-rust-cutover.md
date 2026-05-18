---
spec_version: '2.0'
task_id: rust-cli-rust-cutover
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: extra_large
risk_level: very_high
---

# Rust CLI cutover

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. The launcher flip.
The single most user-visible event in the rust takeover.
Blockers: `rust-cli-feature-parity-matrix` green across every row,
`rust-aster-runtime-cutover` complete, every CLI-surface port complete.
Allowed follow-up command: `scafld harden rust-cli-rust-cutover`
Latest runner update: none
Review gate: not_started

## Summary

Flip the npm `@runxhq/cli` package from a Node CLI to a thin platform-aware
launcher that downloads and executes the bundled Rust binary. Today
`crates/runx-cli/src/launcher.rs` already implements a launcher that can
`npm exec` the TS package or invoke a local `node` against a `js-bin`
path. This spec replaces "delegate to TS via npm exec" with "execute the
bundled Rust binary for this platform".

The Rust binary becomes the authoritative `runx` invocation. The npm
package's only job is to download the right platform binary and exec it
(the esbuild / biome / turbo pattern).

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (npm package)
- `crates/runx-cli` (native binary)
- `crates/runx-runtime`
- `cloud/packages/api` (release distribution; see existing
  `registry-release-distribution-hardening` draft)

Current TypeScript sources:
- `packages/cli/src/index.ts`
- `packages/cli/src/dispatch.ts`
- `crates/runx-cli/src/launcher.rs` (today: delegate-to-TS launcher)
- `crates/runx-cli/src/main.rs`

Files impacted:
- `crates/runx-cli/src/main.rs` (full native CLI)
- `crates/runx-cli/src/commands/**` (one module per command)
- `crates/runx-cli/src/presentation/**`
- `packages/cli/bin/runx.js` (shrinks to: detect platform, download binary,
  exec; or even-thinner postinstall download)
- `packages/cli/package.json` (optional platform-specific dependencies for
  the binary, or postinstall script)
- `scripts/release-rust-cli.ts` (new: package the Rust binary per platform,
  publish to the binary CDN, bump npm)
- `crates/runx-cli/build.rs` (if needed for embedding version metadata)

Invariants:
- The npm package still installs `runx` on every platform that TS supports
  today: macOS (x86_64, aarch64), Linux (x86_64, aarch64), Windows
  (x86_64). Other platforms must be explicitly documented as unsupported
  before this cutover lands.
- Every command currently in `dispatch.ts` (see `plans/rust-takeover.md`
  section 4 matrix) has a Rust implementation that passes its parity row
  in `rust-cli-feature-parity-matrix`.
- Existing scripts and workflows that invoke `runx <subcommand>` continue
  to work unchanged. The launcher does not introduce a new argument shape.
- The launcher exits with the same exit codes the TS CLI did. The current
  `docs/cli-exit-codes.md` is the contract.
- Receipts emitted before and after the cutover are interchangeable; users
  can replay either.
- Rollback path: a single npm publish reverts the launcher to npm-exec the
  TS CLI. The TS CLI is not deleted in this spec; it stays publishable as
  a separate package for rollback for at least one minor cycle.

## Objectives

- Build the full native Rust CLI on top of `runx-runtime`.
- Replace the launcher's "npm exec" path with "download + exec bundled
  Rust binary per platform".
- Publish the binary distribution pipeline.
- Run the full parity matrix as the cutover gate.
- Document rollback.

## Scope

In scope:
- Native CLI implementation for every command in the parity matrix.
- Binary distribution pipeline.
- Launcher logic to fetch and exec the binary.
- Release notes and rollback documentation.

Out of scope:
- Deleting the TS CLI package (deferred to a TS sunset spec).
- Adding new commands not present in TS.
- New platforms beyond what TS supports today.

## Dependencies

- `rust-cli-feature-parity-matrix` green.
- `rust-aster-runtime-cutover` complete.
- All CLI-surface specs complete (skill execution, registry client,
  connect, config, journal, scaffold, tool catalogs, doctor, dev, harness,
  resume, replay, history, knowledge show, mcp serve, evolve disposition).
- Binary distribution infrastructure: signing, CDN, version pinning.

## Open Questions

- Whether the launcher keeps a `--js-fallback` flag after the cutover
  (open question 12.1 of `plans/rust-takeover.md`). Default lean: no.
- Whether `knowledge show` ports or retires before this cutover.
- Binary signing scheme (Apple notarization, Authenticode, sigstore).
  Defer to Phase 1 ingest.
