---
spec_version: '2.0'
task_id: runx-parser-skill-decomposition-v1
created: '2026-05-27T00:00:00Z'
updated: '2026-05-27T00:00:00Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# runx parser skill decomposition v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Review gate: pass

## Summary

`crates/runx-parser/src/skill.rs` carried public skill types, markdown
frontmatter parsing, runner-definition support, catalog validation, harness
fixture parsing, source validation, sandbox validation, execution semantics,
and low-level field helpers in one file. That made the parser harder to
review and kept the old large-file waiver alive.

This spec executed a mechanical decomposition without changing the public
parser API or any serde wire shapes.

## Scope

- Keep `runx_parser::skill::*` and crate-root re-exports stable.
- Move public skill data shapes to `skill/types.rs`.
- Move markdown/frontmatter parsing and quality-profile extraction to
  `skill/markdown.rs`.
- Move runner-definition parsing support to `skill/runner_definition.rs`.
- Move catalog metadata parsing to `skill/catalog.rs`.
- Move harness fixture parsing to `skill/fixtures.rs`.
- Move skill governance validation to `skill/governance.rs`.
- Move source-kind validation to `skill/source.rs`.
- Move sandbox declaration normalization to `skill/sandbox.rs`.
- Move execution-semantics validation to `skill/execution_semantics.rs`.
- Do not change parser rejection messages, fixture semantics, serde names,
  or source-kind behavior.

## Evidence

Commands run after implementation:

```sh
cargo fmt --manifest-path crates/Cargo.toml --all
cargo test --manifest-path crates/Cargo.toml -p runx-parser
cargo check --manifest-path crates/Cargo.toml -p runx-runtime -p runx-cli
cargo clippy --manifest-path crates/Cargo.toml -p runx-parser --all-targets -- -D warnings
```

All commands passed.

## Review Notes

- The only pre-existing dirty file observed during execution was
  `crates/runx-cli/tests/native_no_ts.rs`; this spec did not touch it.
- The split is intentionally internal. External callers still import
  `parse_skill_markdown`, `validate_skill`, `validate_skill_source`,
  `SkillSource`, `SourceKind`, and related types from the same paths.
- `skill.rs` now owns orchestration and shared field helpers. Source,
  sandbox, governance, execution-semantics, catalog, fixture, markdown, and
  runner-definition details live in named modules.
- No `runx-parser/src/skill*.rs` file carries a large-file waiver after the
  split.
