---
spec_version: '2.0'
task_id: rust-journal-local
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust journal local

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Covers `runx journal
show` and `runx history` plus the local journal index used by the runner.
Blockers: `rust-runtime-skeleton`, `rust-receipts-parity`.
Allowed follow-up command: `scafld harden rust-journal-local`
Latest runner update: none
Review gate: not_started

## Summary

Port the local journal index and history surface (`runx journal show`,
`runx history`). The post-2026-04-22 cutover model uses unified `entries[]`
with `recorded_at`, `source_refs`, `projector_id`, and `watermark`. The
Rust port matches that shape and writes via `RUNX_JOURNAL_DIR`.

## Context

CWD: `.`

Packages:
- `@runxhq/memory` (carrying the `entries[]` local journal model)
- `@runxhq/cli` (history command)
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/cli/src/commands/history.ts`
- `packages/memory/**` (local journal index)

Files impacted:
- `crates/runx-runtime/src/journal/local.rs`
- `crates/runx-runtime/src/journal/index.rs`
- `fixtures/journal/**`

Invariants:
- Env: `RUNX_JOURNAL_DIR` (not `RUNX_MEMORY_DIR`).
- Entries are append-only with `recorded_at` timestamps.
- Source refs and watermark semantics match TS.
- The journal is local-only; cloud sync is a separate concern.

## Objectives

- Port local journal index read/write.
- Port `runx journal show` and `runx history` surfaces.
- Add a fixture suite covering append, query, and projection.

## Scope

In scope:
- Local journal index, history surface.

Out of scope:
- Cloud journal sync.
- Journal compaction / GC semantics beyond what TS does.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-receipts-parity` (journal entries reference receipts).

## Open Questions

- Whether the journal index file format gets a version bump as part of the
  port. Default: no; preserve byte-for-byte to keep TS-readable.
