---
spec_version: '2.0'
task_id: rust-mcp-rmcp-server-cutover
created: '2026-05-21T12:12:00Z'
updated: '2026-05-21T17:30:52Z'
status: completed
harden_status: passed
size: large
risk_level: high
---

# rmcp MCP cutover

## Current State

Status: completed
Current phase: final validation
Next: done
Reason: task completed
Blockers: none in the MCP write set.
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T16:42:54Z
Post-cutover dogfood update: 2026-05-21T17:30:52Z added an actual
`runx mcp serve` binary dogfood test, multi-call streaming stress coverage,
and malformed mid-session transport failure coverage. Dogfood found and fixed
two real runtime-boundary issues: the server used a blocking read inside an
async poll path, which starved live streaming sessions, and post-initialize
transport errors could be reported as clean shutdowns. The native CLI now
passes an explicit `RUNX_CWD` workspace boundary into MCP execution, and MCP
server skill refs are canonicalized before sandbox planning so relative CLI
refs cannot produce relative sandbox cwd artifacts.
Review gate: pass

## Summary

This spec executes the final MCP rmcp cutover.

The accepted design is:

- `rmcp` owns MCP protocol semantics: lifecycle, JSON-RPC method dispatch,
  tool listing, tool calls, protocol errors, and MCP content values.
- runx owns the runtime boundary around that protocol: Content-Length stdio
  framing, harness invocation, authority/proof projection, skill templates,
  sandbox metadata, stderr handling, and sealed receipt surfaces.
- `tokio` is allowed only inside the runtime adapter tier that hosts rmcp. It
  is not a pure-kernel dependency and is not exposed as a public runx API.
- There is exactly one runtime feature for MCP: `mcp`. The temporary
  `mcp-rmcp` staging feature is removed.
- Raw JSON object field ordering is not a product invariant. The stable MCP
  wire gate is exact Content-Length framing plus semantic JSON-RPC equivalence
  and stable tool/error/receipt payloads.

The old hand-rolled JSON-RPC server/client path has been deleted. The
remaining runx-specific transport module is the small reviewed
Content-Length adapter used by rmcp.

## Context

CWD: `.` from the OSS repo root.

Relevant runtime surfaces:

- `crates/runx-runtime/Cargo.toml`
- `crates/runx-runtime/src/adapters/mcp.rs`
- `crates/runx-runtime/src/adapters/mcp/framing.rs`
- `crates/runx-runtime/src/adapters/mcp/rmcp_content_length.rs`
- `crates/runx-runtime/src/adapters/mcp/server.rs`
- `crates/runx-runtime/src/adapters/mcp/transport.rs`
- `crates/runx-runtime/src/adapters/mcp/server_skill.rs`
- `crates/runx-runtime/src/adapters/mcp/templates.rs`
- `crates/runx-runtime/src/adapters/mcp/sandbox_metadata.rs`
- `crates/runx-runtime/tests/mcp_adapter.rs`
- `crates/runx-runtime/tests/mcp_server.rs`
- `fixtures/runtime/adapters/mcp/wire-contract/`

Removed runtime surface:

- `crates/runx-runtime/src/adapters/mcp/jsonrpc.rs`

Policy surfaces:

- `crates/deny.toml`
- `scripts/check-rust-crate-graph.mjs`

## Invariants

- No compatibility alias remains for `mcp-rmcp`. The canonical feature is
  `mcp`.
- No hand-rolled MCP JSON-RPC dispatcher remains. runx may keep a
  Content-Length transport adapter because framing is part of the runx stdio
  contract, not MCP method semantics.
- `rmcp` and `tokio` do not enter pure crates (`runx-contracts`, `runx-core`,
  `runx-parser`, `runx-receipts`) or `runx-cli`.
- The CLI remains a thin boundary for MCP serving. Protocol behavior lives in
  `runx-runtime`.
- MCP server responses may differ in harmless JSON object field order or omit
  default false fields. Tests must compare framed message boundaries exactly
  and compare JSON-RPC payloads semantically.
- Malformed transport frames fail as transport errors. Valid JSON-RPC requests
  with MCP-level errors return rmcp-owned JSON-RPC errors.
- Harness receipts, skill execution, sandbox metadata, and argument templating
  remain runx-owned behavior.

## Objectives

- Collapse the staged `mcp-rmcp` feature into the canonical `mcp` feature.
- Use rmcp for both process-client MCP calls and `runx mcp serve`.
- Keep runx's Content-Length stdio framing contract through
  `RmcpContentLengthTransport`.
- Delete hand-rolled JSON-RPC protocol code.
- Update fixture tests so they assert stable product semantics rather than
  JSON serializer field ordering.
- Update dependency guard rails so rmcp/tokio are explicit runtime-adapter
  exceptions, not silent workspace-wide drift.

## Non-Goals

- No SSE or streamable HTTP MCP transport.
- No public reusable rmcp server abstraction.
- No `mcp-rmcp` compatibility feature or alias.
- No compatibility shim for the deleted hand-rolled JSON-RPC path.
- No change to the public skill names or tool contract.

## Design

### Feature shape

`runx-runtime` exposes one MCP feature:

```toml
mcp = [
  "dep:rmcp",
  "dep:tokio",
  "tokio/process",
  "tokio/io-util",
  "tokio/sync",
  "tokio/rt-multi-thread"
]
```

The old staging feature is gone. The crate graph script enforces this exact
shape so a second MCP protocol path cannot reappear accidentally.

### Protocol ownership

The rmcp crate owns:

- MCP initialization lifecycle
- JSON-RPC request/response dispatch
- tool list and tool call semantics
- protocol error construction
- MCP content objects

Runx owns:

- Content-Length framed stdio transport
- command/process spawning
- bounded stderr handling
- skill execution and harness projection
- receipt and sandbox metadata payloads
- test fixtures that represent the runx product contract

### Server lifecycle

`serve_mcp_json_rpc` delegates to an rmcp server over
`RmcpContentLengthTransport`.

The server path uses a channel-backed async reader over owned stdio handles, so
`runx mcp serve` can stream responses without blocking the Tokio executor or
waiting for stdin EOF. The adapter uses a small hidden multi-thread Tokio
runtime so the rmcp server task and stdin reader can make progress during real
binary dogfood sessions.

### Client lifecycle

`ProcessMcpTransport` starts the configured MCP process, connects rmcp over
Content-Length transport, initializes the lifecycle, drains stderr, performs
the requested call, and shuts down the child boundary.

If invoked from an existing tokio runtime, the blocking runtime bridge runs on
a separate OS thread so nested runtime panics do not leak into skill execution.

### Fixture semantics

The wire fixtures stay authoritative for:

- valid Content-Length frame parsing
- request lifecycle shape
- JSON-RPC ids, methods, and stable result/error semantics
- runx tool payload fields and receipt projection

They are not authoritative for raw JSON object field order. Fixture tests
normalize harmless rmcp serializer differences such as absent/default
`isError: false` while still rejecting semantic drift.

## Acceptance

- `crates/runx-runtime/Cargo.toml` has no `mcp-rmcp` feature.
- `crates/runx-runtime/src/adapters/mcp/jsonrpc.rs` is deleted.
- `runx mcp serve` is backed by rmcp protocol dispatch.
- `ProcessMcpTransport` is backed by rmcp protocol dispatch.
- The shared Content-Length transport tests pass under `--features mcp`.
- MCP server and adapter tests pass under `--features mcp`.
- A native `runx mcp serve` binary dogfood test streams initialize,
  `tools/list`, multiple `tools/call` requests, and verifies sealed harness
  receipt files.
- A native `runx mcp serve` malformed mid-session frame test fails closed and
  reports the recorded transport diagnostic.
- `scripts/check-rust-crate-graph.mjs` passes and prevents rmcp/tokio from
  entering pure crates or `runx-cli`.
- `cargo clippy -p runx-runtime --all-targets --features mcp -- -D warnings`
  passes.
- `cargo deny --manifest-path crates/Cargo.toml check bans licenses` passes.
- `scafld validate rust-mcp-rmcp-server-cutover --json` passes.

## Validation Evidence

Validation completed during the cutover:

- `cargo fmt --manifest-path crates/Cargo.toml --all -- --check` passed.
- `node scripts/check-rust-crate-graph.mjs` passed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_server -- --nocapture`
  passed 13 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_adapter -- --nocapture`
  passed 11 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --lib rmcp_transport_tests -- --nocapture`
  passed 4 transport tests.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features mcp -- -D warnings`
  passed.
- `cargo deny --manifest-path crates/Cargo.toml check` passed advisories,
  bans, licenses, and sources.
- `scafld validate rust-mcp-rmcp-server-cutover --json` passed.
- `git diff --check` passed.

Post-cutover dogfood/stress validation completed after the initial cutover:

- `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test mcp_dogfood -- --nocapture`
  passed 2 tests. This spawns the native `runx` binary, speaks Content-Length
  MCP over stdin/stdout, streams six real `tools/call` invocations through the
  `mcp-echo` skill, verifies completed structured content, and checks each
  written receipt has schema `runx.harness_receipt.v1`, sealed harness state,
  and a seal.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_server -- --nocapture`
  passed 15 tests, including a 96-call single-session streaming stress test
  and a mid-session malformed-frame diagnostic test.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_adapter -- --nocapture`
  passed 11 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --lib rmcp_transport_tests -- --nocapture`
  passed 4 transport tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-core policy -- --nocapture`
  passed the policy unit, fixture, and proptest-filtered surfaces after the
  authority wildcard call-site drift was repaired.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features mcp -- -D warnings`
  passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-cli --all-targets -- -D warnings`
  passed.
- `cargo deny --manifest-path crates/Cargo.toml check` passed advisories,
  bans, licenses, and sources.
- `node scripts/check-rust-crate-graph.mjs` passed.
- `cargo fmt --manifest-path crates/Cargo.toml --all -- --check` passed.
- `git diff --check` passed.

## References

- `.scafld/specs/archive/2026-05/rust-mcp-rmcp-cutover.md`
- `.scafld/specs/archive/2026-05/rust-mcp-rmcp-adoption.md`
- `crates/runx-runtime/src/adapters/mcp/server.rs`
- `crates/runx-runtime/src/adapters/mcp/transport.rs`
- `crates/runx-runtime/src/adapters/mcp/rmcp_content_length.rs`
- `fixtures/runtime/adapters/mcp/wire-contract/`

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Human-reviewed override accepted: Claude review found two low MCP notes; both were fixed and revalidated. The remaining provider blocker was ambient concurrent workspace mutation, which is not an MCP completion blocker for this task.

Attack log:
- `review gate`: manual human audit -> clean (Claude review found two low MCP notes; both were fixed and revalidated. The remaining provider blocker was ambient concurrent workspace mutation, which is not an MCP completion blocker for this task.)

Findings:
- none
