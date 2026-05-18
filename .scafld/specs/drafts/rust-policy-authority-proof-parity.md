---
spec_version: '2.0'
task_id: rust-policy-authority-proof-parity
created: '2026-05-17T00:00:00Z'
updated: '2026-05-17T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# Rust policy authority-proof parity

## Current State

Status: draft
Current phase: none
Next: approve
Reason: placeholder created to anchor deferred parity scope
Blockers: `rust-policy-parity`
Allowed follow-up command: `scafld harden rust-policy-authority-proof-parity`
Latest runner update: none
Review gate: not_started

## Summary

Port the `@runxhq/core/policy` authority-proof and public-work re-export
surface to Rust after the initial policy parity spec lands. The first kernel
fixture phase intentionally excludes this surface because it depends on wider
contract and public-work semantics than local admission, sandbox, retry, and
graph-scope decisions.

## Scope

In scope:
- `packages/core/src/policy/authority-proof.ts`
- `packages/core/src/policy/public-work.ts`
- Re-exports from `packages/core/src/policy/index.ts` for authority-proof and
  public-work behavior.
- Shared fixture coverage against the TypeScript oracle before Rust behavior
  is accepted.

Out of scope:
- Runtime adapters, provider calls, filesystem, subprocess, MCP, A2A, and CLI
  cutover.

## Dependencies

- `rust-kernel-parity-fixtures`
- `rust-policy-parity`
- `runx-contracts` carries any typed JSON contracts needed by the Rust port.

## Acceptance

Profile: strict

Definition of done:
- [ ] Authority-proof fixtures exist and pass against the TypeScript oracle.
- [ ] Public-work fixtures exist and pass against the TypeScript oracle.
- [ ] Rust `runx-core` policy modules pass the same fixtures.
- [ ] Claude review passes before completion.

## Planning Log

- 2026-05-17T00:00:00Z: Created as the explicit follow-up anchor for
  authority-proof and public-work parity that `rust-kernel-parity-fixtures`
  defers.
