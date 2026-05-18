---
spec_version: '2.0'
task_id: rust-runtime-skeleton
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T14:04:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Rust runtime skeleton

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. First impure crate
under the Rust takeover; the foundation every downstream surface depends on.
Sequenced after the contract spine and harness receipt parity when used as a
launcher-cutover gate.
Blockers: `rust-contracts-parity` complete, `rust-parser-parity` complete,
`rust-receipts-parity` complete.
Allowed follow-up command: `scafld harden rust-runtime-skeleton`
Latest runner update: none
Review gate: not_started

## Summary

Stand up `runx-runtime` as the first impure crate. Wire the local runner
loop, define the `Caller` and adapter traits, port the `cli-tool` adapter,
and execute `oss/examples/hello-graph/graph.yaml` end to end producing a
deterministic post-cutover harness receipt that matches the TS runner-local
canonical output.

The runtime crate owns side effects: filesystem, subprocess, network IO,
sandbox enforcement, MCP, and adapter concurrency. Pure crates feed it
contracts and decisions; runtime translates decisions into effects.

Section 13 of `docs/rust-kernel-architecture.md` reserves `runx-cli` as a
launcher until the runtime exists and one adapter is ported. This spec
satisfies that prerequisite. It does not flip the launcher; that is
`rust-cli-rust-cutover`.

## Context

CWD: `.`

Packages:
- `@runxhq/runtime-local`
- `@runxhq/adapters` (cli-tool adapter only)
- `@runxhq/core` (executor + state-machine + policy)
- `crates/runx-runtime`
- `crates/runx-contracts`
- `crates/runx-core`
- `crates/runx-parser`
- `crates/runx-receipts`

Current TypeScript sources:
- `packages/runtime-local/src/runner-local/index.ts`
- `packages/runtime-local/src/runner-local/orchestrator/`
- `packages/runtime-local/src/runner-local/execution-semantics.ts`
- `packages/runtime-local/src/runner-local/process-sandbox.ts`
- `packages/runtime-local/src/runner-local/caller-adapters.ts`
- `packages/adapters/src/cli-tool/*`

Files impacted:
- `crates/runx-runtime/Cargo.toml`
- `crates/runx-runtime/src/lib.rs`
- `crates/runx-runtime/src/runner.rs`
- `crates/runx-runtime/src/caller.rs`
- `crates/runx-runtime/src/adapter.rs`
- `crates/runx-runtime/src/adapters/cli_tool.rs`
- `crates/runx-runtime/src/sandbox.rs`
- `crates/runx-runtime/src/journal.rs`
- `crates/runx-runtime/src/receipts.rs`
- `crates/runx-runtime/tests/hello_graph.rs`
- `crates/runx-runtime/tests/parity/**`
- `fixtures/runtime/hello-graph/**`

Invariants:
- TypeScript runner-local remains authoritative until cutover specs replace
  consumers.
- The skeleton ports only `cli-tool`. Other adapters (`agent`, `catalog`,
  `a2a`, `mcp`) are their own specs.
- The skeleton produces harness receipts that pass `runx-receipts::verify`.
  Receipts in tests are byte-identical to post-cutover TS harness receipts for
  the same input modulo documented non-deterministic fields (timestamps, run
  ids), which use injected clocks/id sources for parity tests.
- Sandbox enforcement is real. The skeleton does not ship with "best effort"
  sandbox.
- Tokio is the runtime. No multi-runtime abstraction.
- `runx-runtime` defaults to no adapter features; `cli-tool` is opt-in via
  `--features cli-tool`.

## Objectives

- Define the `Caller` trait (report, resolve, log) and adapter trait.
- Port the runner loop covering start, step, fanout, resume, terminal.
- Port process sandbox enforcement to a Rust equivalent (likely
  `std::process::Command` + platform-specific helpers; isolation profile is
  Phase 1 ingest).
- Port `cli-tool` adapter end to end.
- Run `oss/examples/hello-graph/graph.yaml` to a green receipt.
- Add parity fixtures that compare Rust runner output against TS runner
  output for hello-graph.

## Scope

In scope:
- Runner loop, adapter trait, cli-tool adapter, sandbox, receipts emission,
  caller reporting.
- Hello-graph smoke test.
- Resume/replay primitives sufficient for `rust-runtime-fanout-parity` to
  build on.

Out of scope:
- MCP, agent, catalog, a2a adapters.
- Cloud connectivity (approval routing, registry client).
- CLI argument parsing or presentation. Runtime exposes a programmatic API
  callable from `runx-cli` once the launcher flips.
- Authoring helpers.

## Dependencies

- `rust-contracts-parity`, `rust-parser-parity`,
  `runx-contract-spine-hard-cutover`, `rust-receipts-parity`.

Sequencing:

- This spec can build an internal runtime skeleton earlier, but it cannot be
  used as a preservation dogfood or launcher-cutover gate until
  `rust-receipts-parity` targets post-cutover harness receipts.

## Open Questions

- Whether the runner exposes a sync facade or async-only. Default: async
  with a `blocking` helper used by `runx-cli` once it natively links.
- How sandbox enforcement composes with platform-specific tools (seatbelt,
  bubblewrap, AppContainer). Phase 1 ingest enumerates the TS implementation
  and picks the Rust mapping.
