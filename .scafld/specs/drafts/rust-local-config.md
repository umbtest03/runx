---
spec_version: '2.0'
task_id: rust-local-config
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: low
---

# Rust local config

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Covers `runx config`.
Blockers: `rust-runtime-skeleton`.
Allowed follow-up command: `scafld harden rust-local-config`
Latest runner update: none
Review gate: not_started

## Summary

Port local config read/write (config-store, env-var overlay, profile
selection) to Rust. Today this lives in
`packages/cli/src/commands/config.ts` and `packages/cli/src/runx-state.ts`.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (config command, runx-state)
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/cli/src/commands/config.ts`
- `packages/cli/src/runx-state.ts`

Files impacted:
- `crates/runx-runtime/src/config/local.rs`
- `crates/runx-runtime/src/config/profile.rs`
- `fixtures/config/**`

Invariants:
- Config file paths match TS (XDG-friendly behavior on POSIX, AppData on
  Windows).
- Env-var precedence is identical.
- No secrets land in user-facing config files.

## Objectives

- Port config get/set/list.
- Port profile selection.
- Add a small fixture suite covering env overlay precedence.

## Scope

In scope:
- Local config surface.

Out of scope:
- Cloud-stored config (none today).
- Migration of legacy config locations beyond what TS does.

## Dependencies

- `rust-runtime-skeleton`.

## Open Questions

- None of consequence at draft time.
