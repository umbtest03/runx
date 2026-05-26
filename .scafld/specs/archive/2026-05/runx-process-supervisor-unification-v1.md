---
spec_version: '2.0'
task_id: runx-process-supervisor-unification-v1
created: '2026-05-27T00:00:00Z'
updated: '2026-05-26T22:49:21Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# runx process supervisor unification v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-26T22:49:21Z
Review gate: pass

## Summary

Unify process-group termination semantics so async MCP supervision does not
shell out to `/bin/kill` while the sync process supervisor uses Rust/rustix.
Process cleanup is part of the trust boundary. A child process group should be
terminated through one internal mechanism with the same TERM, grace, KILL, and
fallback behavior.

This spec is not the S-tier persistent-session work. It must not introduce MCP
session pooling, external-adapter pooling, or new protocol reset behavior.

## Scope

- `crates/runx-runtime/src/process/**`
- `crates/runx-runtime/src/adapters/mcp/transport.rs`
- `crates/runx-runtime/src/outbox_provider.rs`
- Focused process-supervision tests under `crates/runx-runtime/tests/**`

Out of scope:

- MCP server contract/type work currently dirty in another agent's lane.
- Persistent MCP sessions and spawn-count perf gates owned by S-tier.
- External adapter protocol reset/session pooling.

## Objectives

- Remove `/bin/kill` shell-out from MCP async process termination on Unix.
- Share signal vocabulary and process-group semantics between sync and async
  supervisors.
- Preserve non-Unix direct-child fallback.
- Add timeout/cleanup tests that fail if descendants outlive termination.

## Acceptance

- `! rg -n 'Command::new\\("/bin/kill"\\)' crates/runx-runtime/src --glob '*.rs'`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool,catalog,mcp --test mcp_adapter mcp_process_transport_times_out_and_terminates_child`
- `cargo check --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool,catalog,mcp`
- `rustfmt --check crates/runx-runtime/src/process.rs crates/runx-runtime/src/process/signal.rs crates/runx-runtime/src/adapters/mcp/transport.rs crates/runx-runtime/src/outbox_provider.rs`

## Phase 1: Overlap Check

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- Confirm no current dirty diff owns `adapters/mcp/transport.rs` or shared process supervisor files.
- If dirty overlap exists, keep this spec draft and do not execute.

Acceptance:
- none

## Phase 2: Shared Termination Mechanism

Status: completed
Dependencies: phase1

Objective: Complete this phase.

Changes:
- Extract a shared Rust signal helper usable by sync and async supervisors.
- Replace MCP `/bin/kill` shell-out with the shared helper.
- Keep non-Unix direct-child semantics unchanged.

Acceptance:
- none

## Phase 3: Tests And Guards

Status: completed
Dependencies: phase2

Objective: Complete this phase.

Changes:
- Add focused tests/guards for timeout, TERM/KILL fallback, and no shell-out.

Acceptance:
- none

## Review

Status: completed
Verdict: pass
Mode: verify
Summary: Human-reviewed override accepted: Reviewed scoped runtime process-supervisor diff. Acceptance passed: no /bin/kill shell-out under crates/runx-runtime/src, focused MCP adapter timeout test, runx-runtime cargo check with cli-tool/catalog/mcp, and rustfmt --check on touched files. Workspace-wide fmt is intentionally not claimed because concurrent execution-runner edits are dirty in another lane.

Attack log:
- `review gate`: manual human audit -> clean (Reviewed scoped runtime process-supervisor diff. Acceptance passed: no /bin/kill shell-out under crates/runx-runtime/src, focused MCP adapter timeout test, runx-runtime cargo check with cli-tool/catalog/mcp, and rustfmt --check on touched files. Workspace-wide fmt is intentionally not claimed because concurrent execution-runner edits are dirty in another lane.)

Findings:
- none

