---
spec_version: '2.0'
task_id: rust-doctor
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: low
---

# Rust doctor

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Low-risk warm-up port
that exercises the runtime crate API surface against a non-trivial command.
Blockers: `rust-runtime-skeleton`.
Allowed follow-up command: `scafld harden rust-doctor`
Latest runner update: none
Review gate: not_started

## Summary

Port `runx doctor` to Rust. Doctor runs a set of environmental checks
(node version, git, scafld, network, sandbox capability, config validity).
It is the natural warm-up port: the surface is well-defined, the side
effects are read-only, and the JSON output is small.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (doctor command, doctor-structure, doctor-types)
- `crates/runx-runtime`
- `crates/runx-contracts` (doctor schema in
  `packages/contracts/src/schemas/doctor.ts`)

Current TypeScript sources:
- `packages/cli/src/commands/doctor.ts`
- `packages/cli/src/commands/doctor-structure.ts`
- `packages/cli/src/commands/doctor-types.ts`
- `packages/contracts/src/schemas/doctor.ts`

Files impacted:
- `crates/runx-runtime/src/doctor/checks.rs`
- `crates/runx-runtime/src/doctor/run.rs`
- `fixtures/doctor/**`

Invariants:
- Doctor is read-only; no side effects beyond probing the environment.
- JSON output is schema-exact against the contract.
- Check ordering is deterministic.

## Objectives

- Port doctor checks and runner.
- Match JSON output byte-for-byte.

## Scope

In scope:
- All current doctor checks.

Out of scope:
- New checks beyond what TS implements.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-contracts-parity`.

## Open Questions

- None of consequence at draft time.
