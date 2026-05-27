---
spec_version: '2.0'
task_id: runx-core-sequential-graph-split-v1
created: '2026-05-27T00:00:00Z'
updated: '2026-05-27T00:00:00Z'
status: completed
harden_status: passed
size: small
risk_level: medium
---

# runx core sequential graph split v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Review gate: pass

## Summary

`crates/runx-core/src/state_machine/sequential_graph.rs` carried graph state
creation, step-index construction, sequential planning, fanout-group planning,
step readiness, and transition application in one waived file. This spec split
those responsibilities into focused internal modules while preserving the public
state-machine API:

- `create_sequential_graph_state`
- `create_sequential_graph_step_index`
- `plan_sequential_graph_transition`
- `plan_sequential_graph_transition_indexed`
- `transition_sequential_graph`
- `apply_sequential_graph_event`

No serialized state-machine contracts, plan variant names, decision strings, or
public `runx_core::state_machine` exports were intentionally changed.

## Scope

- Keep the public `state_machine` exports stable.
- Move graph-state creation to `sequential_graph/state.rs`.
- Move step-index construction and lookup to `sequential_graph/index.rs`.
- Move top-level sequential planning to `sequential_graph/planning.rs`.
- Move fanout-group planning to `sequential_graph/fanout_group.rs`.
- Move context/retry readiness helpers to `sequential_graph/step_readiness.rs`.
- Move event application to `sequential_graph/transition.rs`.
- Remove the stale `large-file` waiver from `sequential_graph.rs`.
- Do not change fanout sync semantics or runtime execution behavior.

## Evidence

Commands run after implementation:

```sh
cargo fmt --manifest-path crates/Cargo.toml --all
CARGO_TARGET_DIR=/tmp/runx-authority-proof-split-target cargo test --manifest-path crates/Cargo.toml -p runx-core --lib state_machine --no-run
CARGO_TARGET_DIR=/tmp/runx-authority-proof-split-target cargo test --manifest-path crates/Cargo.toml -p runx-core --test state_machine_fixtures --no-run
CARGO_TARGET_DIR=/tmp/runx-authority-proof-split-target cargo test --manifest-path crates/Cargo.toml -p runx-core --test state_machine_proptest --no-run
CARGO_TARGET_DIR=/tmp/runx-authority-proof-split-target cargo clippy --manifest-path crates/Cargo.toml -p runx-core --all-targets -- -D warnings
rg -n "rust-style-allow: large-file" crates/runx-core/src/state_machine/sequential_graph.rs crates/runx-core/src/state_machine/sequential_graph
git diff --check -- .scafld/specs/archive/2026-05/runx-core-sequential-graph-split-v1.md crates/runx-core/src/state_machine/sequential_graph.rs crates/runx-core/src/state_machine/sequential_graph
```

All commands passed. Direct execution of the state-machine test binary stalled
before Rust test startup at the macOS loader, matching the same loader symptom
observed in other concurrent test binaries. This slice therefore used no-run
compilation for the state-machine fixture/proptest targets plus full
`runx-core --all-targets` clippy as its review evidence.

## Review Notes

- The split is internal to `runx-core`; runtime and kernel-eval callers keep
  using the same public `state_machine` module paths.
- Existing dirty files were present in CLI, runtime MCP, LangChain tests, and
  adapter fixtures. This spec did not touch them.
