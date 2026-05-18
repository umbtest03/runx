---
spec_version: '2.0'
task_id: rust-scaffold
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: low
---

# Rust scaffold

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Covers `runx init`
and `runx new`.
Blockers: `rust-runtime-skeleton`.
Allowed follow-up command: `scafld harden rust-scaffold`
Latest runner update: none
Review gate: not_started

## Summary

Port the workspace scaffold (`runx init`) and skill scaffold (`runx new`)
to Rust. Today these live in `packages/cli/src/commands/init.ts`,
`packages/cli/src/commands/new.ts`, and `packages/cli/src/scaffold.ts`,
backed by `packages/create-skill/`.

## Context

CWD: `.`

Packages:
- `@runxhq/cli`
- `@runxhq/create-skill`
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/cli/src/commands/init.ts`
- `packages/cli/src/commands/new.ts`
- `packages/cli/src/scaffold.ts`
- `packages/create-skill/**`

Files impacted:
- `crates/runx-runtime/src/scaffold/init.rs`
- `crates/runx-runtime/src/scaffold/new.rs`
- `crates/runx-runtime/src/scaffold/templates.rs`
- `fixtures/scaffold/**`

Invariants:
- Scaffolds emit byte-identical files to TS for the same inputs (templates
  vendored as compile-time `include_str!` or read from disk; pick one).
- Scaffolds never overwrite without explicit `--force`.

## Objectives

- Port `runx init` (workspace bootstrap).
- Port `runx new` (skill / chain bootstrap).
- Add a fixture suite covering each template.

## Scope

In scope:
- Init and new scaffolds.

Out of scope:
- `create-runx` npm package (separate distribution; consumers download the
  Rust binary indirectly).
- Authoring evolution flow (`runx evolve`).

## Dependencies

- `rust-runtime-skeleton`.

## Open Questions

- Whether templates ship vendored in the binary or read from a fetched
  template bundle. Default: vendored for v0.
