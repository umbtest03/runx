---
spec_version: '2.0'
task_id: rust-mcp-rmcp-cutover
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T12:12:00Z'
status: active
harden_status: not_run
size: large
risk_level: high
---

# rmcp client transport cutover for the MCP adapter

## Current State

Status: active
Current phase: stage-2 client transport repair complete
Next: review this Stage 1-2 slice, then execute `rust-mcp-rmcp-server-cutover`
for the server/deletion stages
Reason: the original file over-scoped one executable slice as the full
five-stage cutover. The code in this tree delivers the disjoint `mcp-rmcp`
feature and rmcp-backed process client transport. Server-loop migration,
rmcp-served wire parity, and deletion of hand-rolled protocol modules remain a
separate follow-up.
Blockers: none
Allowed follow-up command: `scafld review rust-mcp-rmcp-cutover`
Latest runner update: 2026-05-21T12:12:00Z
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
not run; this slice converts that to a package-scoped `runx-runtime` exception
while the staged `mcp-rmcp` feature exists and moves the process client
transport to rmcp.

This spec does not re-decide design. Where this file and
`rust-mcp-rmcp-adoption` differ, the predecessor wins. This file is the
executable decomposition of that plan's "Follow-up cutover plan" section.

## Summary

Deliver the first two independently compiling stages of the MCP rmcp cutover:
add a disjoint `mcp-rmcp` feature with an exact pinned rmcp dependency, then run
`ProcessMcpTransport` tool listing and calls through rmcp over stdio. This
slice deliberately does not claim the server loop, rmcp-served wire parity, or
deletion gate. Those stages are owned by `rust-mcp-rmcp-server-cutover`.

The runx-specific surfaces (skill execution under MCP, argument templating,
sandbox metadata, the `runx:` host-result projection, receipt sealing) are not
touched.

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

## Runner note: 2026-05-21T12:12:00Z

The review gate correctly caught two client-slice regressions in addition to
the over-broad Stage 3-5 claims. The client transport now records
receive-side Content-Length, size-limit, and JSON parse failures in the rmcp
transport error state before returning stream end to rmcp's `Transport`
interface, so downstream service errors can preserve the stable transport
message. The tokio child-process path now pipes and bounded-drains stderr like
the legacy client path instead of sending it to `/dev/null`.

New unit coverage under `--features mcp-rmcp --lib rmcp_transport_tests`
proves missing `Content-Length`, oversized body, and malformed JSON are
recorded as transport errors rather than clean EOF.

## Context

CWD: `.` (run cargo from `crates/`).

Packages:
- `crates/runx-runtime` (the `mcp` adapter modules and features)

Current sources (hand-rolled, still owned by the server/deletion follow-up):
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
- `crates/deny.toml` (scope the `rmcp` exception during this staged client
  path; remove the ban after `rust-mcp-rmcp-server-cutover` deletes the
  hand-rolled protocol path)
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
- The hand-rolled server/framing layer is not deleted by this spec. It is
  deleted only after the follow-up server cutover passes rmcp-served wire
  parity and its deletion gate.
- `cargo deny check licenses` stays clean; the rmcp tree (tokio, schemars,
  JSON-Schema helpers) must remain Apache-2.0 / MIT.

## Objectives

- Adopt `rmcp` for the process client transport behind a disjoint feature, with
  an exact pinned version.
- Preserve the existing `mcp` feature's recorded stdio wire contract while the
  staged `mcp-rmcp` client path is introduced.
- Preserve stable client error semantics for malformed JSON, missing
  `Content-Length`, oversized responses, timeout, and stderr draining.

## Scope

In scope:
- Stage 1 from `rust-mcp-rmcp-adoption`: feature flag, exact rmcp pin,
  dependency-policy exception, and mutual-exclusion build guard.
- Stage 2 from `rust-mcp-rmcp-adoption`: rmcp-backed process client transport
  for tool listing and calls; `FixtureMcpTransport` remains unchanged.
- Client-side parity repair for receive errors and stderr draining.

Out of scope:
- Server transport migration, rmcp-served wire parity, deletion of hand-rolled
  protocol modules, removal of the `mcp-rmcp` staging feature, and removal of
  the `deny.toml` rmcp ban. Owned by `rust-mcp-rmcp-server-cutover`.
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
3. Deferred to `rust-mcp-rmcp-server-cutover`: behind `mcp-rmcp`, swap the
   `serve_mcp_json_rpc` stdio loop for the rmcp server, wrapping
   `McpServerState` in an rmcp `ServerHandler`.
4. Deferred to `rust-mcp-rmcp-server-cutover`: diff the rmcp server's framed
   output against the recorded `*.responses.jsonl` baseline, holding to the
   predecessor's enumerated must-match / may-differ envelope.
5. Deferred to `rust-mcp-rmcp-server-cutover`: delete hand-rolled protocol
   code, make rmcp the only `mcp` path, and remove the `rmcp` ban from
   `deny.toml`.

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
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp-rmcp --lib rmcp_transport_tests -- --nocapture`
  passes.
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_server -- --nocapture`
  passes.
- [x] `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features cli-tool,mcp -- -D warnings`
  passes.
- [x] `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features mcp-rmcp -- -D warnings`
  passes.
- [x] From `crates/`, `cargo deny check bans` and `cargo deny check licenses`
  pass.

## Follow-up

- `rust-mcp-rmcp-server-cutover` owns the remaining server loop, rmcp-served
  wire parity, deletion-gate signal, and `deny.toml` ban removal.

## References

- [`rust-mcp-rmcp-adoption`](../archive/2026-05/rust-mcp-rmcp-adoption.md)
  (design, replacement map, wire-diff envelope, deletion gate)
- [`crates/deny.toml`](../../crates/deny.toml) (the `rmcp` ban to remove)
- [`plans/rust-takeover.md`](../../../plans/rust-takeover.md) §9 step 7
  ("MCP is last")
- [`oss/docs/rust-kernel-architecture.md`](../../docs/rust-kernel-architecture.md)
  §13 ("MCP is the hardest port")
- rmcp upstream: `https://github.com/modelcontextprotocol/rust-sdk`
