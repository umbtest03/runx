# Runtime Throughput

This note defines the OSS runtime performance contract for the runtime cutover
tasks. The target is throughput on runx-controlled overhead: graph planning,
context and output projection, fanout synchronization, receipt sealing, receipt
store maintenance, MCP session framing, native CLI launch overhead, and thin
TypeScript bridge framing. It does not claim speedups for external LLMs,
network APIs, or user subprocess work.

## Baseline

Capture the local baseline before hot-path changes:

```bash
pnpm --dir oss perf:runtime:capture -- --output ../.scafld/perf/oss-runtime-throughput-baseline.json
```

The capture script runs the Rust Criterion benches and records a JSON document
using schema `runx.oss_runtime_throughput.v1`. The JSON stores workload
throughput in iterations per second plus the Criterion mean in nanoseconds.

## Benchmarks

Rust runtime workloads live in
`crates/runx-runtime/benches/graph_throughput.rs`:

- `graph_planning`
- `context_projection`
- `output_projection`
- `wide_fanout`
- `graph_receipt_sealing`
- `receipt_store_append`
- `receipt_store_index`

Receipt canonicalization workloads live in
`crates/runx-receipts/benches/receipt_canonicalization.rs`:

- `receipt_canonicalization`
- `receipt_body_json`
- `receipt_full_json`

The capture script also records `ts_bridge_framing`, a bounded Node framing
microbenchmark for the TypeScript bridge surface.

S-tier protocol/session workloads are orchestrated by
`scripts/runtime-throughput.mjs` because they are process/protocol overhead
rather than Criterion benches:

- `mcp_session_start`
- `mcp_session_reuse`
- `native_cli_launch`

The MCP rows are measured through the Rust `runx-mcp-session-probe` binary, which
invokes `McpAdapter<ProcessMcpTransport>` and reports the transport spawn
counter. These rows include `spawn_count`. The MCP reuse and native launch gates
require `spawn_count <= 1` and no p99 regression above the declared budget. MCP
is the only pooled protocol lane in the S-tier cutover. External adapters remain
one-shot until a reset-capable wire contract and negative isolation tests exist.

Process/protocol rows are measured from release binaries built in
`crates/target/runx-perf/release`. The perf harness intentionally does not reuse
`crates/target/debug/runx`, because that binary may be stale or built from a
different local checkout state. Each capture asks Cargo to refresh those release
probe binaries before measuring so an existing perf artifact cannot silently
stand in for the current checkout. The native launch row performs one unmeasured
warm-up launch before collecting samples so p99 gates track steady local launch
overhead rather than first-touch page-cache noise.

## Fanout Execution

Fanout remains serial by default. Set `RUNX_MAX_FANOUT_CONCURRENCY` in
`RuntimeOptions.env` or the process environment to opt into bounded parallel
fanout. The runtime only parallelizes isolated, non-mutating skill branches when
the adapter explicitly provides a sendable fanout clone; native run steps,
tool-resolution paths, host-resolution paths, payment-authority inputs, and
custom adapters without the capability stay serial.

## Runtime Boundaries

The hot-path runtime changes keep ownership narrow:

- `runx-core` remains the pure decision layer for graph planning, fanout sync,
  retry, scope admission, credential binding, and authority proof metadata.
- `runx-runtime` owns mutable execution indexes, fanout scheduling, subprocess
  supervision, receipt linking, receipt store indexing, and journal projection.
- `runx-receipts` owns canonical byte output, body/full digesting, proof
  verification, and receipt tree resolution.
- TypeScript packages remain generated contracts, host/client wrappers,
  authoring tools, and cloud/product code. Deleted executor packages do not
  remain as runtime bridges.
- MCP keeps protocol-specific Content-Length session handling with explicit
  session safety rules. The pool is keyed by server command, args, cwd, and
  sandboxed environment; plans with cleanup paths remain one-shot. Arbitrary
  CLI/user subprocesses and external adapters are not pooled.

The shared Rust process supervisor is intentionally private to
`runx-runtime`. It owns only process lifecycle mechanics: environment/cwd
application, stdin writing, bounded stdout/stderr capture, timeout signaling,
process-group cleanup, duration, and sandbox cleanup paths. Adapter-specific
policy, redaction, protocol parsing, and receipt projection stay in their
adapter modules.

## Limits

The 2x gate applies to deterministic runx-controlled graph/projection
overhead. Receipt canonicalization and store maintenance use a 1.75x
throughput gate plus allocation and growth-shape budgets. Session gates track
spawn count and p99 regression. These gates do not claim an end-to-end speedup
when wall time is dominated by external models, remote APIs, user subprocess
work, package manager startup, or operating system sandbox setup.

## Gates

Later phases compare against the Phase 1 baseline:

```bash
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads graph_planning,context_projection,output_projection --min-throughput-ratio 1.20
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads wide_fanout --min-throughput-ratio 2.00
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads receipt_canonicalization,graph_receipt_sealing --min-throughput-ratio 1.50
```

The check command exits non-zero when any requested workload misses its declared
throughput ratio.

The S-tier final gate captures all runtime-owned workloads into
`.scafld/perf/oss-runtime-s-tier-final.json` and compares them against
`.scafld/perf/oss-runtime-s-tier-baseline.json`:

```bash
pnpm --dir oss perf:runtime:capture -- --output ../.scafld/perf/oss-runtime-s-tier-final.json --workloads graph_planning,context_projection,output_projection,wide_fanout,receipt_canonicalization,graph_receipt_sealing,receipt_store_append,receipt_store_index,mcp_session_start,mcp_session_reuse,native_cli_launch
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-s-tier-baseline.json --candidate ../.scafld/perf/oss-runtime-s-tier-final.json --workloads graph_planning,context_projection,output_projection,wide_fanout --min-throughput-ratio 2.00 --max-p99-regression 1.10
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-s-tier-baseline.json --candidate ../.scafld/perf/oss-runtime-s-tier-final.json --workloads receipt_canonicalization,graph_receipt_sealing,receipt_store_append,receipt_store_index --min-throughput-ratio 1.75 --max-growth-exponent 1.10 --max-allocation-regression 1.10
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-s-tier-baseline.json --candidate ../.scafld/perf/oss-runtime-s-tier-final.json --workloads mcp_session_reuse,native_cli_launch --max-spawn-count 1 --max-p99-regression 1.10
```
