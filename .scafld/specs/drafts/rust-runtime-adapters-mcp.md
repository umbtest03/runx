---
spec_version: '2.0'
task_id: rust-runtime-adapters-mcp
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: extra_large
risk_level: very_high
---

# Rust runtime MCP adapter

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. The hardest adapter
port; section 13 of `docs/rust-kernel-architecture.md` calls out rmcp +
tokio + sandbox + spawn semantics as the highest-risk cross-language
surface.
Blockers: `rust-runtime-skeleton`, `rust-runtime-adapters-agent`, and at
least one other adapter to validate the trait shape.
Allowed follow-up command: `scafld harden rust-runtime-adapters-mcp`
Latest runner update: none
Review gate: not_started

## Summary

Port the `mcp` adapter family to `runx-runtime` behind the
`features = ["mcp"]` flag. Covers MCP client (consume MCP servers as
tools) and MCP server (`runx mcp serve`) execution paths.

This is intentionally the LAST adapter ported. The rmcp ecosystem,
tokio integration, sandbox interaction, and spawn semantics combine more
moving parts than any other adapter. The earlier adapters validate the
runtime trait shape so MCP doesn't have to invent it.

## Context

CWD: `.`

Packages:
- `@runxhq/runtime-local` (mcp subpath)
- `@runxhq/adapters` (mcp subpath if present)
- `cloud/packages/mcp-hosted`
- `crates/runx-runtime`
- `crates/runx-contracts`

Current TypeScript sources:
- `packages/runtime-local/src/mcp/**`
- `packages/cli/src/commands/mcp.ts`
- `cloud/packages/mcp-hosted/src/**`

Files impacted:
- `crates/runx-runtime/src/adapters/mcp/client.rs`
- `crates/runx-runtime/src/adapters/mcp/server.rs`
- `crates/runx-runtime/src/adapters/mcp/sandbox.rs`
- `crates/runx-runtime/tests/mcp_parity.rs`
- `fixtures/runtime/adapters/mcp/**`

Invariants:
- MCP client uses rmcp (or chosen rust MCP library) per Phase 1 decision.
- Sandbox enforcement for MCP-spawned subprocesses matches the TS process-
  sandbox semantics; no weakening.
- MCP server respects the same scope-admission and approval-gate behavior
  the TS server does.
- No live network or external MCP server hits in fixtures.

## Objectives

- Pick the Rust MCP library in Phase 1 ingest (rmcp candidate).
- Port MCP client (consume MCP tools as skill steps).
- Port MCP server (`runx mcp serve`) including listing, invocation,
  approval round-trips on MCP-driven mutations.
- Add fixture suite for client and server flows.

## Scope

In scope:
- MCP client and server under the adapter trait.

Out of scope:
- Cloud-hosted MCP server (`cloud/packages/mcp-hosted`) cutover; that's a
  separate cloud cutover spec.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-runtime-adapters-agent`.
- `rust-approval-gate-parity`.
- At least one additional adapter complete so the trait shape is stable.

## Open Questions

- rmcp library maturity and licensing at port time. Defer to Phase 1.
- Whether MCP server spawns under the same process-sandbox primitives as
  cli-tool, or needs an MCP-specific isolation profile.
