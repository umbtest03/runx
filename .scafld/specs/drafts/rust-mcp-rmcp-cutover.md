---
spec_version: '2.0'
task_id: rust-mcp-rmcp-cutover
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T00:00:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# rmcp cutover for the MCP adapter

## Current State

Status: draft
Current phase: not started
Next: pin the reviewed `rmcp` release and land Stage 1 (feature flag, no behavior change)
Reason: the design-and-baseline predecessor
[`rust-mcp-rmcp-adoption`](../archive/2026-05/rust-mcp-rmcp-adoption.md)
completed and passed review. It recorded the stdio wire-contract baseline, the
replacement map, the staged plan, the wire-diff envelope, and the deletion
gate, then handed the actual migration to this spec. Its one hard blocker,
`rust-async-http-layer` (tokio/reqwest in the adapter tier), has since landed:
`runx-runtime` now builds `reqwest` + `tokio` behind the `async-http` feature
([`runx-runtime/Cargo.toml`](../../crates/runx-runtime/Cargo.toml)). The
sequencing constraint from `plans/rust-takeover.md` §9 ("MCP is last") is the
remaining reason this had not started; the customer-surface parity matrix work
ahead of it does not block authoring or Stage 1.
Blockers: none for authoring. Stage 1 needs the exact `rmcp` version pinned
(see Open Questions).
Review gate: not_started

## Why this exists

The MCP adapter at
[`crates/runx-runtime/src/adapters/mcp/`](../../crates/runx-runtime/src/adapters/mcp/)
hand-rolls the MCP protocol: Content-Length framing, JSON-RPC request/response
correlation, the stdio client transport, and the stdio server loop. The
predecessor spec established that the upstream `rmcp` crate should own that
protocol surface, that the runx-specific surfaces stay, and that the cutover
must preserve the recorded byte-shape contract. `rmcp` is still banned in
[`crates/deny.toml`](../../crates/deny.toml) precisely because this cutover has
not run. This spec runs it.

This spec does not re-decide design. Where this file and
`rust-mcp-rmcp-adoption` differ, the predecessor wins. This file is the
executable decomposition of that plan's "Follow-up cutover plan" section.

## Summary

Migrate the MCP adapter's protocol layer to the upstream `rmcp` crate behind a
disjoint `mcp-rmcp` feature, in five independently-compiling stages, preserving
the recorded stdio wire contract within the predecessor's enumerated diff
envelope, then delete the hand-rolled protocol modules and remove the
`deny.toml` ban. The runx-specific surfaces (skill execution under MCP,
argument templating, sandbox metadata, the `runx:` host-result projection,
receipt sealing) are not touched.

## Context

CWD: `.` (run cargo from `crates/`).

Packages:
- `crates/runx-runtime` (the `mcp` adapter modules and features)

Current sources (hand-rolled, to be replaced by rmcp):
- `crates/runx-runtime/src/adapters/mcp/framing.rs`
- `crates/runx-runtime/src/adapters/mcp/jsonrpc.rs`
- `crates/runx-runtime/src/adapters/mcp/transport.rs` (`ProcessMcpTransport`)
- `crates/runx-runtime/src/adapters/mcp/server.rs` (`serve_mcp_json_rpc`)

Current sources (runx-specific, must stay unchanged):
- `crates/runx-runtime/src/adapters/mcp/server_skill.rs`
- `crates/runx-runtime/src/adapters/mcp/templates.rs`
- `crates/runx-runtime/src/adapters/mcp/sandbox_metadata.rs`
- `crates/runx-runtime/src/adapters/mcp/adapter.rs` (`McpAdapter` trait impl)
- `crates/runx-runtime/src/adapters/mcp/transport.rs` (`FixtureMcpTransport`)

Files impacted:
- `crates/runx-runtime/Cargo.toml` (features, optional `rmcp` dep)
- `crates/Cargo.lock` (committed with the dependency review)
- `crates/deny.toml` (remove the `rmcp` ban after the cutover)
- `crates/runx-runtime/src/lib.rs` (mutual-exclusion `compile_error!`)

Baseline already in repo (reuse, do not rewrite):
- `fixtures/runtime/adapters/mcp/wire-contract/basic-lifecycle.{requests,responses}.jsonl`
- `fixtures/runtime/adapters/mcp/wire-contract/error-paths.{requests,responses}.jsonl`
- test `mcp_server_matches_recorded_stdio_wire_contract`
  (`cargo test -p runx-runtime --features mcp --test mcp_server`)

Invariants:
- `mcp` (hand-rolled) and `mcp-rmcp` (rmcp-backed) are disjoint features.
  Enabling both is a build-time `compile_error!`.
- The runx-specific surfaces listed above are not modified by any stage.
- Every stage compiles and tests independently. No big-bang rewrite.
- The hand-rolled layer is deleted only after Stage 4 wire parity passes and
  the deletion gate is satisfied.
- `cargo deny check licenses` stays clean; the rmcp tree (tokio, schemars,
  JSON-Schema helpers) must remain Apache-2.0 / MIT.

## Objectives

- Adopt `rmcp` for MCP framing, JSON-RPC, the client transport, and the server
  loop, behind a disjoint feature, with an exact pinned version.
- Preserve the recorded stdio wire contract within the predecessor's diff
  envelope.
- Delete the hand-rolled protocol modules and remove the `deny.toml` ban once
  the deletion gate is met.

## Scope

In scope:
- The five stages from `rust-mcp-rmcp-adoption` "Follow-up cutover plan":
  feature flag (Stage 1), client transport (Stage 2), server transport
  (Stage 3), byte-exact wire-parity check (Stage 4), deletion plus deny.toml
  removal (Stage 5).
- The mutual-exclusion build guard and the dependency-review `Cargo.lock` diff.

Out of scope:
- rmcp HTTP transports (SSE / streamable HTTP). Stdio only; follow up if a
  consumer needs HTTP (predecessor Open Questions).
- Publishing a public reusable rmcp `ServerHandler` type
  (deferred to `runx-mcp-public-server-trait`).
- Any change to runx skill execution, templating, sandbox metadata, the
  `runx:` projection, or receipt sealing.

## Stages

Stages and their per-stage acceptance gates are defined in
`rust-mcp-rmcp-adoption` and not restated here. Execution order:

1. Pull `rmcp` behind `mcp-rmcp = ["dep:rmcp", "async-http"]` (no `"mcp"` in the
   list) with a `compile_error!` if both features are set. No behavior change.
2. Behind `#[cfg(feature = "mcp-rmcp")]`, swap `ProcessMcpTransport::call_tool`
   to the rmcp client. `FixtureMcpTransport` unchanged.
3. Behind `mcp-rmcp`, swap the `serve_mcp_json_rpc` stdio loop for the rmcp
   server, wrapping `McpServerState` in an rmcp `ServerHandler`.
4. Diff the rmcp server's framed output against the recorded `*.responses.jsonl`
   baseline, holding to the predecessor's enumerated must-match / may-differ
   envelope. Any diff outside the envelope is a regression.
5. Once Stage 4 passes and the deletion gate holds, delete
   `mcp/{framing,jsonrpc,transport,server}` protocol code, remove the
   `mcp-rmcp` feature so rmcp is the only `mcp` path, and remove the `rmcp`
   ban from `deny.toml`.

## Dependencies

- `rust-mcp-rmcp-adoption` (archived, completed; design + baseline source of
  truth).
- `rust-async-http-layer` (archived, landed; supplies the adapter-tier
  tokio/reqwest exception this spec consumes).

## Open Questions

- Exact `rmcp` version. The predecessor forbids fake literals and ranges and
  requires an exact pin verified against crates.io at authoring time. This must
  be filled with the current stable `rmcp` release as `=x.y.z` before Stage 1
  lands; it is intentionally not guessed here. Run `cargo update -p rmcp` and
  commit the `Cargo.lock` diff with the dependency review.
- Deletion-gate signal. The predecessor replaced the unverifiable external-soak
  gate with an in-repo attestation under
  `fixtures/runtime/adapters/mcp/rmcp-cutover/` OR an owner override OR
  default-on for one full minor with zero protocol-drift receipts. Pick the
  signal to use at Stage 5 time and record the query command or the override.

## References

- [`rust-mcp-rmcp-adoption`](../archive/2026-05/rust-mcp-rmcp-adoption.md)
  (design, replacement map, wire-diff envelope, deletion gate)
- [`crates/deny.toml`](../../crates/deny.toml) (the `rmcp` ban to remove)
- [`plans/rust-takeover.md`](../../../plans/rust-takeover.md) §9 step 7
  ("MCP is last")
- [`oss/docs/rust-kernel-architecture.md`](../../docs/rust-kernel-architecture.md)
  §13 ("MCP is the hardest port")
- rmcp upstream: `https://github.com/modelcontextprotocol/rust-sdk`
