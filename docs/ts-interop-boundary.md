# TypeScript interop boundary

This document records the surviving TypeScript and Python package boundary for
the Rust takeover. It is the package-disposition source of truth for OSS
packages during the dual-tree window.

## Current boundary after takeover

Rust is canonical for local skill and graph execution, harness and dogfood
execution, receipt sealing and verification, history, policy and registry
configuration, authority admission, payment authority, and adapter
orchestration. TypeScript remains a client, packaging, product UX, docs, and
compatibility-test surface unless a separate spec gives it ownership of a
non-local service boundary.

The package CLI is a distribution and UX shim, not a second runtime. A usable
installation must be able to execute the Rust `runx` binary without TypeScript
source, tsx, or a local workspace. If the wrapper cannot find a supported
native binary, it should fail closed with installation guidance rather than
falling back to a TypeScript implementation of local behavior.

Three crossings exist between the surviving TypeScript surface and the Rust
runtime. Each crossing has one contract surface that owns the wire shape.

1. **CLI JSON.** Anything that shells `runx`, including package launchers, the
   LangChain bridge, `runx-py`, user scripts, and CI workflows. The Rust CLI
   owns behavior; the contract is each command's JSON output shape, exit
   codes, and human-output stability.
2. **Published TypeScript contracts.** Anything that imports from
   `@runxhq/contracts`, including host adapters, cloud packages, and external
   TypeScript consumers. The contract is the TypeScript shape that mirrors
   `runx-contracts` Rust types, with fixture cross-validation.
3. **Cloud HTTP contracts.** Anything where the Rust runtime calls cloud, such
   as approval routing, connect/auth, registry, and receipts-store. The
   contract is versioned, documented HTTP endpoints.

No fourth boundary is added without updating this document. No published
TypeScript package is silently broken: each package disposition is named here.
Stable-boundary edits to consume new `runx-contracts` versions are normal
maintenance. Sunset means deletion only through the named sunset spec.

## OSS package dispositions

| Package | Disposition |
| --- | --- |
| `@runxhq/adapters` | Sunset with `rust-ts-sunset-runtime-local`; adapter logic moves into `runx-runtime::adapters`. |
| `@runxhq/authoring` | Stays unchanged until the authoring DX plan decides whether to port to Rust or move into scafld. |
| `@runxhq/cli` | Stays as a platform-aware npm launcher that resolves and execs the Rust binary. It must remain useful from an installed package without TypeScript sources. |
| `@runxhq/contracts` | Stays as the published TypeScript view of `runx-contracts`, maintained with fixture cross-validation. |
| `@runxhq/core` | Sunset across the named Rust TS sunset specs for state-machine, policy, parser, executor, receipts, registry, marketplaces, and final shell cleanup. |
| `@runxhq/create-skill` | Stays as a thin npm bootstrapper that wraps `runx new` through the CLI. |
| `@runxhq/host-adapters` | Stays as thin host response adapters over the runx host protocol, retargeted to `@runxhq/contracts` types. |
| `@runxhq/langchain` | Stays as an optional LangChain bridge that shells the `runx` CLI for governed skill and tool invocation. |
| `@runxhq/runtime-local` | Sunset with `rust-ts-sunset-runtime-local`; runner, sandbox, harness, MCP, SDK caller, and host-protocol execution move to Rust. No new trusted local orchestration starts here. |
| `runx-py` | Stays as a thin Python client over `runx` CLI JSON output. |

Cloud packages remain TypeScript. The Rust runtime consumes cloud through the
cloud HTTP contracts; cloud cutovers are separate future specs. Local registry
and policy configuration remains Rust-owned when exercised by the native CLI.

## Test ownership

Language-owned unit tests stay with their implementation. Contract and parity
tests use durable fixtures under `fixtures/`. End-to-end tests should spawn the
`runx` binary and assert stdout, exit codes, and JSON instead of importing
TypeScript internals. Trusted local graph, harness, receipt, authority,
registry/policy config, and payment behavior needs Rust coverage or a TS-free
Rust CLI fixture before wrapper tests can count as proof.

The TypeScript oracle is temporary. Once a TypeScript domain sunsets, no new
fixtures should be derived from that domain's TypeScript implementation.
