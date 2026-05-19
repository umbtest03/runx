# runx-runtime

Native Rust runtime skeleton for governed runx execution.

This crate owns impure execution boundaries: filesystem reads, subprocess
execution, sandbox preparation, caller reporting, and post-cutover harness
receipt emission. Pure parser, core, contract, and receipt crates remain
upstream.

Current slice:

- parses a local graph with `runx-parser`
- plans sequential/fanout transitions with `runx-core`
- runs `cli-tool` skills behind the `cli-tool` feature
- emits harness receipts and validates the parent receipt tree with
  `runx-receipts`

Adapter families remain feature gated:

- `cli-tool`
- `mcp`
- `a2a`
- `agent`
- `catalog`

The Cargo CLI launcher still delegates to the npm CLI until the later
launcher-cutover spec wires this runtime into `runx-cli`.

## Doctor

The native Rust doctor API is programmatic-only. `runx doctor` still routes
through the npm TypeScript CLI until a dedicated CLI cutover wires the command
surface, human renderer, and repair behavior to Rust.

This crate currently ports the read-only fixture-backed diagnostics:

- `runx.tool.manifest.removed_format`
- `runx.tool.fixture.missing`
- `runx.skill.fixture.missing`
- `runx.structure.file_budget.exceeded`
- `runx.structure.cross_package_reach_in`

Deferred doctor families remain owned by follow-up slices:

- `runx doctor --fix` repair writes
- diagnostic catalog, `--list-diagnostics`, and `--explain`
- official skills lock freshness
- tool manifest stale source and schema hashes
- packet index diagnostics
- graph packet path validation
- receipt proof health
- policy health
