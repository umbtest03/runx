# Runtime Throughput

This note defines the OSS runtime performance contract for the
`oss-runtime-throughput-architecture-v1` scafld task. The target is throughput
on runx-controlled overhead: graph planning, context and output projection,
fanout synchronization, receipt sealing, receipt store maintenance, and thin
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
- TypeScript packages remain compatibility bridges. Parser/kernel bridges now
  share one bounded native process helper, but Rust remains the behavior owner.
  MCP keeps its protocol-specific Content-Length session handling rather than
  being forced through the one-shot parser/kernel helper.

The shared Rust process supervisor is intentionally private to
`runx-runtime`. It owns only process lifecycle mechanics: environment/cwd
application, stdin writing, bounded stdout/stderr capture, timeout signaling,
process-group cleanup, duration, and sandbox cleanup paths. Adapter-specific
policy, redaction, protocol parsing, and receipt projection stay in their
adapter modules.

## Limits

The 2x gate applies to deterministic runx-controlled overhead. It does not
claim a 2x end-to-end speedup when wall time is dominated by external models,
remote APIs, user subprocess work, package manager startup, or operating system
sandbox setup. Subprocess pooling is not enabled for arbitrary user commands;
pooling is only appropriate for protocol-safe sessions after sandbox,
credential, and cleanup isolation are proven.

## Gates

Later phases compare against the Phase 1 baseline:

```bash
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads graph_planning,context_projection,output_projection --min-throughput-ratio 1.20
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads wide_fanout --min-throughput-ratio 2.00
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads receipt_canonicalization,graph_receipt_sealing --min-throughput-ratio 1.50
```

The check command exits non-zero when any requested workload misses its declared
throughput ratio.
