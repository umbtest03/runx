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

## Gates

Later phases compare against the Phase 1 baseline:

```bash
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads graph_planning,context_projection,output_projection --min-throughput-ratio 1.20
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads wide_fanout --min-throughput-ratio 2.00
pnpm --dir oss perf:runtime:check -- --baseline ../.scafld/perf/oss-runtime-throughput-baseline.json --workloads receipt_canonicalization,graph_receipt_sealing --min-throughput-ratio 1.50
```

The check command exits non-zero when any requested workload misses its declared
throughput ratio.
