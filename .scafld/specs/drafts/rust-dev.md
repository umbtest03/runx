---
spec_version: '2.0'
task_id: rust-dev
created: '2026-05-18T00:00:00Z'
updated: '2026-05-21T01:13:58Z'
status: draft
harden_status: in_progress
size: medium
risk_level: medium
---

# Rust dev

## Current State

Status: draft
Current phase: native skill/graph fixture execution plus CLI presentation parity slice implemented
Next: CLI watch cutover decision
Reason: a narrow Rust runtime slice now exists for dev fixture discovery,
deterministic tool fixture execution, polling watch debounce, presentation, and
dev-mode receipt metadata tagging. `target.kind: skill` and `target.kind:
graph` fixtures now execute through the Rust harness replay path and validate
against the dev fixture expectation engine. Repo-integration skill fixtures bind
workspace cwd through `RUNX_CWD` instead of process-global cwd mutation. The
Rust CLI dev JSON path now pretty-prints like the TS CLI, and the native dev
terminal presentation uses the same no-color status glyphs as the TS
presentation. This is not complete `runx dev` parity yet.
Blockers: the Rust CLI dev command is owned by the CLI cutover worker; the TS
command currently parses `--watch` but does not run a watch loop, so CLI-level
watch parity still needs an owning cutover decision before Rust should expose a
long-running watch loop.
Allowed follow-up command: make the explicit CLI watch decision, then wire the
chosen Rust behavior and rerun runtime and CLI dev validation; do not mark
passed until the remaining blockers are closed.
Latest runner update: 2026-05-21T01:13:58Z
Review gate: not_started

## Summary

Port `runx dev` to Rust. Dev mode runs a skill or chain in an iterative
loop with file watch, fast-feedback receipts, and harness wiring. Today
this lives in `packages/cli/src/commands/dev/` and consumes runner-local
plus harness primitives.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (dev command tree)
- `@runxhq/runtime-local` (runner-local, harness)
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/cli/src/commands/dev/**`
- `packages/cli/src/commands/dev.ts`
- `packages/runtime-local/src/harness/runner.ts`

Files impacted:
- `crates/runx-runtime/src/dev/watch.rs`
- `crates/runx-runtime/src/dev/loop.rs`
- `crates/runx-runtime/src/dev/presentation.rs`
- `fixtures/dev/**`

Invariants:
- File watch debounce and ignore patterns match TS.
- Dev mode never silently consumes secrets; reuses connect grants.
- Receipts emitted in dev are clearly tagged as dev-mode in metadata.

## Objectives

- Port dev mode loop with file watch.
- Match presentation (terminal output) to TS via snapshot tests.

## Scope

In scope:
- Dev loop, file watch, presentation.

Out of scope:
- New dev features beyond TS.

## Dependencies

- `rust-runtime-skeleton` (archived completed; review gate pass).
- `rust-harness` (archived completed; harden passed and review gate pass).

## Open Questions

- File watch library choice (notify, watchexec). Defer to Phase 1.

## Harden Rounds

### round-1

Status: in_progress
Started: 2026-05-20T10:34:14Z
Ended: none

Checks:
- `cargo fmt --package runx-runtime` from `crates`: passed.
- `cargo test -p runx-runtime --test dev -- --nocapture` from `crates`: passed
  with 5 tests.
- `cargo check -p runx-runtime` from `crates`: passed.
- `cargo fmt --manifest-path crates/Cargo.toml --package runx-runtime`: passed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  passed with 5 tests.
- `cargo check --manifest-path crates/Cargo.toml -p runx-runtime`: passed.
- `cargo fmt --manifest-path crates/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  passed with 5 tests in the default feature set.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test dev -- --nocapture`:
  passed with 7 tests, including deterministic native skill/graph fixtures and
  repo-integration workspace cwd binding.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --tests`:
  passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets`:
  passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --all-targets`:
  passed.
- `git diff --check`: passed.
- `cargo fmt --manifest-path crates/Cargo.toml --package runx-cli --package runx-runtime`:
  passed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  passed with 5 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli dev_json_stdout_is_pretty_printed_like_ts_cli -- --nocapture`:
  passed with the focused CLI dev JSON unit test.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test dev -- --nocapture`:
  passed with 7 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli dev_ -- --nocapture`:
  passed with the dev JSON unit test plus the existing dev launcher routing
  tests.
- `cargo fmt --manifest-path crates/Cargo.toml --all -- --check`: passed.
- `git diff --check -- crates/runx-cli/src/dev.rs crates/runx-runtime/src/dev/presentation.rs crates/runx-runtime/tests/dev.rs .scafld/specs/drafts/rust-dev.md`:
  passed.
- Earlier broad filtered check `cargo test -p runx-runtime dev -- --nocapture`
  passed the new 5 dev tests and filtered the rest; initial invocation from repo
  root failed because the Cargo workspace lives under `crates/`.

Issues:
- Runtime slice implemented under `crates/runx-runtime/src/dev/**` with
  deterministic tool fixture execution only.
- Deterministic native skill/graph dev fixture execution is implemented through
  the Rust harness replay path with stable fixture output projection.
- Native skill/graph repo-integration fixtures bind workspace cwd through
  `RUNX_CWD` without process-global cwd mutation.
- CLI dev routing and watch cutover intentionally untouched; the TS command
  parses `--watch` without running a watch loop, so Rust still fails closed on
  that flag until the product behavior is explicit.
- CLI dev JSON and no-color terminal presentation parity tightened in the Rust
  CLI/runtime.
