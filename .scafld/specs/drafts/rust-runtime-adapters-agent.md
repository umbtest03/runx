---
spec_version: '2.0'
task_id: rust-runtime-adapters-agent
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Rust runtime agent adapter

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Second adapter after
`cli-tool`; covers the agent execution path.
Blockers: `rust-runtime-skeleton` complete, `rust-approval-gate-parity`
complete (agent gates ride on approvals).
Allowed follow-up command: `scafld harden rust-runtime-adapters-agent`
Latest runner update: none
Review gate: not_started

## Summary

Port the `agent` adapter family to `runx-runtime` behind the
`features = ["agent"]` flag. Covers the host-protocol bridge to model
providers (anthropic-shaped today) including act assignment,
tool-call dispatch, and durable-step semantics.

## Context

CWD: `.`

Packages:
- `@runxhq/adapters` (agent subpath)
- `@runxhq/runtime-local` (sdk caller/host-protocol)
- `cloud/packages/agent-runner`
- `crates/runx-runtime`
- `crates/runx-contracts` (host-protocol)

Current TypeScript sources:
- `packages/adapters/src/agent/**`
- `packages/runtime-local/src/sdk/caller.ts`
- `packages/runtime-local/src/sdk/host-protocol.ts`
- `cloud/packages/agent-runner/src/anthropic.ts`
- `cloud/packages/agent-runner/src/openai-compat.ts`
- `cloud/packages/agent-runner/src/durable-step.ts`

Files impacted:
- `crates/runx-runtime/src/adapters/agent.rs`
- `crates/runx-runtime/src/adapters/agent/anthropic.rs`
- `crates/runx-runtime/src/adapters/agent/host_protocol.rs`
- `crates/runx-runtime/tests/agent_parity.rs`
- `fixtures/runtime/adapters/agent/**`

Invariants:
- Host protocol contract types come from `runx-contracts`; the adapter does
  not redeclare them.
- Capability execution semantics (idempotency hash, retry rules) match
  `runx-contracts::act_assignment` already ported.
- Tool calls are mocked in fixtures; no live provider hits.
- Durable-step semantics for hosted agent-runner remain TS until a separate
  cloud cutover.

## Objectives

- Port the local agent adapter (process the host protocol, dispatch tool
  calls back to `runx-runtime`, emit receipts).
- Provide a deterministic fixture harness for tool-call round-trips.
- Document the boundary between adapter and host-protocol contract.

## Scope

In scope:
- Local agent adapter behind a feature flag.
- Anthropic-shaped provider; OpenAI-compat as follow-up.

Out of scope:
- Hosted agent-runner durable-step replacement.
- Provider API client implementation choices (deferred until concrete need).

## Dependencies

- `rust-runtime-skeleton`.
- `rust-approval-gate-parity`.
- `rust-contracts-parity` (host protocol already covered).

## Open Questions

- Whether the adapter ships a tokio-based HTTP client (reqwest) or accepts
  an injected transport for testability. Default lean: injected transport
  for tests, real client behind a `live-http` test feature.
