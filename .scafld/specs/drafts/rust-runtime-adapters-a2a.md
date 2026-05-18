---
spec_version: '2.0'
task_id: rust-runtime-adapters-a2a
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust runtime a2a adapter

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Third adapter; covers
agent-to-agent invocation paths.
Blockers: `rust-runtime-skeleton` complete, `rust-runtime-adapters-agent`
complete (a2a builds on agent semantics).
Allowed follow-up command: `scafld harden rust-runtime-adapters-a2a`
Latest runner update: none
Review gate: not_started

## Summary

Port the `a2a` adapter family to `runx-runtime`. Agent-to-agent paths
dispatch one skill's execution to another agent surface, propagating
scope, authority, and receipt linkage across the hop.

## Context

CWD: `.`

Packages:
- `@runxhq/adapters` (a2a subpath)
- `crates/runx-runtime`
- `crates/runx-contracts`

Current TypeScript sources:
- `packages/adapters/src/a2a/**`
- `packages/runtime-local/src/harness/a2a-fixture.ts`

Files impacted:
- `crates/runx-runtime/src/adapters/a2a.rs`
- `crates/runx-runtime/tests/a2a_parity.rs`
- `fixtures/runtime/adapters/a2a/**`

Invariants:
- Authority and scope propagation across a2a hops uses `runx-core::policy`
  primitives; a2a does not invent its own attenuation.
- Receipts include source-hop linkage (`source_refs` already in journal
  model).
- No live cross-network calls in fixtures.

## Objectives

- Port a2a dispatch end to end.
- Add fixture suite covering trusted, semi-trusted, and untrusted target
  hops with the right receipt linkage.

## Scope

In scope:
- A2A dispatch, propagation, receipts.

Out of scope:
- New cross-org trust models beyond what TS already implements.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-runtime-adapters-agent`.

## Open Questions

- Whether a2a continues using HTTP transport in Rust or migrates to a
  shared protocol library (e.g., reuse `rmcp` patterns). Defer to Phase 1.
