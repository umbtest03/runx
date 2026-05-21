---
spec_version: '2.0'
task_id: rust-mcp-rmcp-server-cutover
created: '2026-05-21T12:12:00Z'
updated: '2026-05-21T12:12:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# rmcp server and deletion cutover for MCP

## Current State

Status: draft
Current phase: scoped from failed broad review
Next: harden this server/deletion slice before approval
Reason: `rust-mcp-rmcp-cutover` now owns only the completed Stage 1-2 client
transport slice. This draft owns the remaining server-loop migration,
rmcp-served wire parity, and deletion gate. It must not be executed blindly:
the current tree still depends on hand-rolled Content-Length framing for
`serve_mcp_json_rpc`, and rmcp 1.7.0's default async read/write transport is
newline-delimited rather than runx's recorded Content-Length stdio wire shape.
Blockers: choose the server transport strategy that preserves the recorded
stdio wire contract before deleting `framing.rs` or removing the `deny.toml`
rmcp exception.
Allowed follow-up command: `scafld harden rust-mcp-rmcp-server-cutover`
Latest runner update: 2026-05-21T12:12:00Z
Review gate: not_started

## Summary

Complete the remaining MCP rmcp cutover after the client transport slice:
implement an rmcp-backed server loop for `runx mcp serve`, prove its framed
stdout matches the recorded wire contract, then delete the temporary dual-path
protocol code and remove the package-scoped `rmcp` ban exception.

This is a clean cutover target, not a compatibility shim. Until the server
transport is proven byte-compatible, the existing `mcp` path remains the
authoritative server path and the staged `mcp-rmcp` feature remains a client
transport proof.

## Context

CWD: `.` from the OSS repo root.

Completed prerequisite:
- `rust-mcp-rmcp-cutover` Stage 1-2: disjoint `mcp-rmcp` feature, exact
  `rmcp = "=1.7.0"` pin, rmcp-backed `ProcessMcpTransport`, client error
  preservation, stderr draining, and deny/license gates.

Current hand-rolled server/protocol files:
- `crates/runx-runtime/src/adapters/mcp/server.rs`
- `crates/runx-runtime/src/adapters/mcp/framing.rs`
- `crates/runx-runtime/src/adapters/mcp/jsonrpc.rs`
- mcp-only client path in `crates/runx-runtime/src/adapters/mcp/transport.rs`

Runx-specific surfaces that must stay:
- `server_skill.rs`
- `templates.rs`
- `sandbox_metadata.rs`
- `adapter.rs`
- `McpServerTool`, `McpHostRunResult`, and sealed harness receipt projection

## Objectives

- Add an rmcp-backed server loop for `serve_mcp_json_rpc` behind the staged
  cutover path without changing runx tool behavior.
- Preserve the recorded Content-Length stdio wire contract for
  `basic-lifecycle` and `error-paths` fixtures, or explicitly record the
  predecessor-approved diff envelope with byte-level evidence.
- Keep malformed request, invalid header, oversized request, unknown method,
  tool error, needs-agent, denied, escalated, failed, and receipt-sealing
  behavior stable.
- Once rmcp-served wire parity passes, remove the hand-rolled protocol path,
  collapse `mcp-rmcp` into the canonical `mcp` feature, and remove the scoped
  `rmcp`/tokio wrapper exception from `crates/deny.toml`.

## Non-Goals

- No SSE or streamable HTTP MCP transport.
- No public reusable rmcp server trait unless the server cutover requires it.
- No compatibility alias between old and new feature names after the deletion
  gate. The end state is one `mcp` path.
- No change to harness receipts, skill execution, sandbox metadata, or
  argument templating.

## Design Constraints

- rmcp's built-in `AsyncRwTransport` is newline-delimited JSON. Runx's recorded
  MCP stdio wire contract is Content-Length framed. The server design must
  either use an rmcp transport that preserves Content-Length framing or provide
  a small, reviewed transport adapter with explicit wire-contract tests.
- The server cutover must not repeat the client-slice review defects:
  receive-side framing errors must be observable, and child stderr must be
  bounded-drained when a child process is used.
- Feature flags are temporary execution scaffolding only. The final code must
  not keep a permanent legacy path.

## Validation

- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp-rmcp --test mcp_server -- --nocapture`
  runs the rmcp server path and passes.
- A wire-contract test compares rmcp-served raw stdout bytes against
  `fixtures/runtime/adapters/mcp/wire-contract/basic-lifecycle.responses.jsonl`
  and `error-paths.responses.jsonl`.
- Existing `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_adapter --test mcp_server -- --nocapture`
  passes until the deletion commit, then equivalent canonical `mcp` tests pass
  after the feature collapse.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features mcp -- -D warnings`
  passes after the feature collapse.
- `cargo deny --manifest-path crates/Cargo.toml check bans licenses` passes
  with no scoped `rmcp` ban exception.
- `rg "^mod (framing|jsonrpc)" crates/runx-runtime/src/adapters/mcp.rs`
  returns no matches after deletion.

## Acceptance

- `runx mcp serve` is backed by rmcp for protocol dispatch.
- There is exactly one MCP feature path in `runx-runtime`.
- No hand-rolled JSON-RPC/framing modules remain unless the harden pass records
  a specific rmcp limitation and the owner explicitly narrows the deletion
  objective before build.
- The public wire-contract fixtures remain the source of truth for MCP stdio.

## References

- `.scafld/specs/active/rust-mcp-rmcp-cutover.md`
- `.scafld/specs/archive/2026-05/rust-mcp-rmcp-adoption.md`
- `crates/runx-runtime/src/adapters/mcp/server.rs`
- `fixtures/runtime/adapters/mcp/wire-contract/`
