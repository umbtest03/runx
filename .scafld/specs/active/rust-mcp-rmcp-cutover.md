---
spec_version: '2.0'
task_id: rust-mcp-rmcp-cutover
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T11:28:26Z'
status: review
harden_status: not_run
size: large
risk_level: high
---

# rmcp cutover for the MCP adapter

## Current State

Status: review
Current phase: final
Next: review
Reason: build completed; ready for review
Blockers: none
Allowed follow-up command: `scafld review rust-mcp-rmcp-cutover`
Latest runner update: 2026-05-21T11:28:26Z
Review gate: not_started

## Why this exists

The MCP adapter at
[`crates/runx-runtime/src/adapters/mcp/`](../../crates/runx-runtime/src/adapters/mcp/)
hand-rolls the MCP protocol: Content-Length framing, JSON-RPC request/response
correlation, the stdio client transport, and the stdio server loop. The
predecessor spec established that the upstream `rmcp` crate should own that
protocol surface, that the runx-specific surfaces stay, and that the cutover
must preserve the recorded byte-shape contract. `rmcp` started banned in
[`crates/deny.toml`](../../crates/deny.toml) precisely because this cutover had
not run; Stage 1 converts that to a package-scoped `runx-runtime` exception
while the staged feature exists. This spec runs it.

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

## Runner note: 2026-05-21T10:58:27Z

Stage 1 and Stage 2 are represented by a compile-gated rmcp client path:
`mcp-rmcp` is a disjoint feature that enables the exact pinned `rmcp = "=1.7.0"`
dependency via `async-http`; `mcp` plus `mcp-rmcp` fails with the intentional
mutual-exclusion compile error. `ProcessMcpTransport::list_tools` and
`ProcessMcpTransport::call_tool` use rmcp behind `mcp-rmcp`, while
`FixtureMcpTransport`, templates, sandbox metadata, and receipt projection stay
unchanged. The scoped dependency-policy exception remains package-bound to
`runx-runtime`; full removal of the rmcp ban is still reserved for Stage 5
after wire parity and the deletion gate.

Validation reached both sides of the staged client cutover:
`cargo check -p runx-runtime --features mcp-rmcp`, `cargo test -p
runx-runtime --features mcp-rmcp --test mcp_adapter`, `cargo test -p
runx-runtime --features mcp --test mcp_server
mcp_server_matches_recorded_stdio_wire_contract`, `cargo test -p runx-runtime
--features mcp --test mcp_adapter`, `cargo clippy -p runx-runtime
--all-targets --features mcp-rmcp -- -D warnings`, `cargo deny check bans`, and
`cargo deny check licenses` pass. `cargo check -p runx-runtime --features mcp,mcp-rmcp` fails
with the expected mutual-exclusion compile error.

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
- `crates/deny.toml` (scope the `rmcp` exception during the cutover; remove the
  ban after Stage 5)
- `crates/runx-runtime/src/lib.rs` and `src/adapters.rs` (feature exposure)
- `crates/runx-runtime/src/adapters/mcp.rs` (mutual-exclusion `compile_error!`)
- `crates/runx-runtime/src/adapters/mcp/{transport,jsonrpc}.rs` (client
  transport gating during Stage 2)

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

1. Pull `rmcp = "=1.7.0"` behind `mcp-rmcp = ["dep:rmcp", "async-http"]` (no
   `"mcp"` in the list) with a `compile_error!` if both features are set.
   **Done.**
2. Behind `#[cfg(feature = "mcp-rmcp")]`, swap `ProcessMcpTransport` tool
   listing and calls to the rmcp client. `FixtureMcpTransport` unchanged.
   **Done for the process client path.**
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

## Dependency pin

`rmcp = "=1.7.0"` (exact pin). Verified as the latest stable release on
crates.io on 2026-05-21 (`max_stable_version` 1.7.0). The predecessor was
written against a pre-1.0 `rmcp` and listed "pre-1.0 churn" as a risk; rmcp has
since reached a stable 1.x line, which removes that risk. Use
`default-features = false` with the `client` feature for the Stage 2 client
path, keep the tokio surface bounded to the runtime adapter tier, and keep
`cargo deny check licenses` clean. At Stage 1, run `cargo update -p rmcp`,
confirm the resolved version is still 1.7.0 (or bump the pin to the
then-current latest and re-review), and commit the `Cargo.lock` diff with the
dependency review.

## Validation

- [x] `cargo check --manifest-path crates/Cargo.toml -p runx-runtime --features mcp-rmcp`
  passes.
- [x] `cargo check --manifest-path crates/Cargo.toml -p runx-runtime --features mcp,mcp-rmcp`
  fails with the intentional mutual-exclusion `compile_error!`.
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test mcp_adapter --features mcp -- --nocapture`
  passes.
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test mcp_adapter --features mcp-rmcp -- --nocapture`
  passes.
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_server -- --nocapture`
  passes.
- [x] `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features cli-tool,mcp -- -D warnings`
  passes.
- [x] `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features mcp-rmcp -- -D warnings`
  passes.
- [x] From `crates/`, `cargo deny check bans` and `cargo deny check licenses`
  pass.

## Open Questions

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
