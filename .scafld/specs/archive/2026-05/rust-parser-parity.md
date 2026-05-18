---
spec_version: '2.0'
task_id: rust-parser-parity
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T00:55:41Z'
status: completed
harden_status: passed
size: large
risk_level: medium
---

# Rust parser parity

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T00:55:41Z
Review gate: pass

## Summary

Replace the `runx-parser` placeholder with a real Rust parser that produces
fixture-byte-identical AST and intermediate representation against the
authoritative TypeScript parser at `packages/core/src/parser/index.ts`.

`packages/core/src/parser/` is 1,658 lines today and is a
pure-by-imports trusted-kernel domain identified in section 2 of
`docs/rust-kernel-architecture.md`. Its parity port is the next pure-kernel
slice after `runx-core`. The parser is the gating dependency for
`runx-receipts` and `runx-runtime`, which both consume validated skills,
graphs, and runner manifests.

This spec does not modify TypeScript parser behavior. Any TS-side change
required for cross-language determinism (analogous to the `node:path` removal
in `rust-kernel-parity-fixtures`) is called out and flagged for a separate
TS-purification spec.

## Context

CWD: `.`

Packages:
- `@runxhq/core` (parser subpath)
- `crates/runx-parser`
- `crates/runx-contracts`

Current TypeScript sources:
- `packages/core/src/parser/index.ts`
- `packages/core/src/parser/graph.ts`
- `packages/core/src/parser/install.ts`
- `packages/core/src/parser/index.test.ts`
- `packages/core/src/parser/graph.test.ts`

Files impacted:
- `crates/runx-contracts/src/execution.rs`
- `crates/runx-contracts/src/lib.rs`
- `crates/runx-contracts/tests/execution_fixtures.rs`
- `crates/Cargo.toml`
- `crates/runx-parser/Cargo.toml`
- `crates/runx-parser/src/lib.rs`
- `crates/runx-parser/src/error.rs`
- `crates/runx-parser/src/yaml.rs`
- `crates/runx-parser/src/skill.rs` or `crates/runx-parser/src/skill/**`
- `crates/runx-parser/src/runner.rs`
- `crates/runx-parser/src/graph.rs` or `crates/runx-parser/src/graph/**`
- `crates/runx-parser/src/tool.rs`
- `crates/runx-parser/src/install.rs`
- `crates/runx-parser/tests/parser_fixtures.rs`
- `crates/runx-parser/tests/parser_rejections.rs`
- `fixtures/contracts/execution/*.json`
- `fixtures/parser/**` (new)
- `scripts/generate-rust-contract-fixtures.ts`
- `scripts/generate-rust-parser-fixtures.ts`
- `scripts/check-rust-core-style.mjs` (extend to cover `runx-parser`)
- `scripts/check-rust-crate-graph.mjs` (extend parser dependency allowances)
- `scripts/check-boundaries.mjs` (add `parser` to pureCoreDomains)
- `docs/rust-kernel-architecture.md` (pin `serde_yml` choice)
- `crates/deny.toml`

Invariants:
- TypeScript parser remains authoritative until a parser-cutover spec lands.
- `runx-parser` is pure: no `tokio`, `reqwest`, `std::fs`, `std::process`,
  `std::net`, `std::env`. Reads YAML from `&str`, not from disk.
- AST and IR shapes match the TypeScript observable fixture contract. Rust
  APIs may use idiomatic names internally; serde-facing JSON preserves the
  TypeScript wire shape.
- Rejection reasons are typed enums, not free-form strings.
- No `serde_json::Value`, `serde_yml::Value`, or `HashMap` anywhere in
  `crates/runx-parser/src`. Parser output uses typed structs/enums and
  deterministic maps at serde boundaries.
- Raw object subtrees use `runx_contracts::JsonObject`,
  `runx_contracts::JsonValue`, and `runx_contracts::JsonNumber`. Do not add a
  parser-local JSON carrier type.
- Existing `runx-contracts` types are reused for cross-domain contracts.
  Phase 1 adds `runx_contracts::execution` for `ExecutionSemantics` and
  `GovernedDisposition` because the TypeScript parser currently imports
  those from receipts, while the Rust crate graph cannot depend on
  `runx-receipts`.
- Rust API names are idiomatic Rust names. Wire JSON and fixture serde names
  preserve the TypeScript observable contract; internal Rust module/function
  names do not mechanically copy TypeScript casing.
- Parser implementation modules stay under the Rust style line caps. Large
  surfaces such as skill and graph may use `skill/` and `graph/` submodules;
  `skill.rs` and `graph.rs` may be thin module roots only.

Related docs:
- `oss/docs/rust-kernel-architecture.md` (sections 2, 3, 6, 10)
- `plans/rust-takeover.md`

## Objectives

- Port the graph parser (graph + step + scope + fanout shapes).
- Port the validated-skill parser to Rust against checked-in fixtures.
- Port the runner-manifest parser (`runners.yaml` / inline runner manifests).
- Port tool-manifest parsing and validation.
- Port skill-install validation, including origin echo and strict skill
  validation.
- Add a generator script that emits fixtures from the TS parser and a Rust
  fixture-runner test that consumes them.
- Extend `check-rust-core-style.mjs` and `check-rust-crate-graph.mjs` to
  cover `runx-parser`.

## Scope

In scope:
- Skill, graph, and runner-manifest parsing.
- Runner-manifest parsing and validation.
- Tool-manifest YAML/JSON parsing and validation.
- Skill-install validation.
- `ExecutionSemantics` / `GovernedDisposition` Rust contract types in
  `runx-contracts`, so `runx-parser` does not depend on `runx-receipts`.
- `parser` added to TypeScript `pureCoreDomains`.
- Error enum design covering current TS rejection reasons.
- Cross-language fixture suite and generator.

Out of scope:
- Replacing any TS parser consumer (separate sunset spec).
- Schema authoring tools (`@runxhq/authoring`) which build on parser output.
- Any broad TypeScript contracts package migration. The TS parser can keep
  importing execution semantics from receipts until a TS cleanup spec moves
  those types; Rust breaks the dependency at the crate boundary now.
- Replacing receipt-owned logic. `runx-receipts` later reuses
  `runx-contracts::execution` instead of creating a duplicate type.

## Dependencies

- `rust-contracts-parity` complete (provides shared types).
- `rust-kernel-parity-fixtures` style and CI patterns reused.

## Architecture Decisions

- YAML library: use `serde_yml = "0.0.12"` for implementation. It is the
  maintained fork available on crates.io today and keeps the parser crate
  close to serde idiom without adopting an unmaintained `serde_yaml`.
- YAML scalar compatibility: parser fixtures are constrained to a YAML 1.2
  scalar subset both the TypeScript `yaml` package and `serde_yml` agree on.
  Fixtures must not rely on sexagesimal numbers, implicit `yes`/`no`
  booleans, octal/hex integers, timestamps, or unquoted date-looking strings.
  The fixture generator rejects divergent scalar forms instead of normalizing
  them silently.
- Execution semantics type home: add `runx_contracts::execution` in Phase 1.
  It owns `ExecutionSemantics`, `GovernedDisposition`, `ReceiptSurfaceRef`,
  `InputContextCapture`, `OutcomeState`, and `ReceiptOutcome`. These are
  behavior-free value types needed by parser output. Receipt-specific
  write/verify behavior remains in `runx-receipts`.
- TypeScript parser import from receipts: do not rewrite TypeScript parser in
  this spec beyond boundary enforcement. The TS source remains authoritative;
  the Rust crate graph resolves the dependency by putting shared contract
  value types in `runx-contracts`.
- Fixture envelope: every fixture has stable top-level keys, a top-level
  `scope`, and a typed input envelope. Skill fixtures use `input.markdown` for
  the full source text and `expected.raw` for the parsed raw fields
  (`frontmatter`, `rawFrontmatter`, `body`). Validated output lives under
  `expected.validated`; rejected cases use `expected.rejection.kind` and
  `expected.rejection.message`. Raw object subtrees use
  `runx_contracts::JsonObject` / `JsonValue` / `JsonNumber` and are compared
  via the stable JSON serializer, so numeric representation follows the
  contracts crate instead of a parser-local encoding.
- Fixture key order: reuse `scripts/check-contract-fixture-key-order.ts` with
  `fixtures/parser` as the target. Do not add another near-identical key-order
  script for parser fixtures.
- `serde_yml::Value`: forbidden anywhere in `crates/runx-parser/src`, not just
  public API. Implementers deserialize directly into typed parser structs and
  enums.
- Skill sandbox boundary: `runx-parser` owns a parser-output `SkillSandbox`
  type because the TypeScript validated skill surface includes parser-only
  fields (`raw: Record<string, unknown>` and `approvedEscalation?: boolean`)
  that do not belong in policy admission. `SkillSandbox` preserves those
  fields in serde output. Parser code converts the normalized policy subset
  (`profile`, `cwdPolicy`, `envAllowlist`, `network`, `writablePaths`,
  `requireEnforcement`) into `runx_core::policy::SandboxDeclaration` and
  calls `normalize_sandbox_declaration` so policy-owned sandbox semantics are
  not duplicated. `raw` and `approvedEscalation` are parser-output evidence
  only and are not passed into policy admission.
- `approvedEscalation` parity: the TypeScript validator currently declares
  `approvedEscalation?: boolean` but never populates it in
  `validateSandbox`. Rust must mirror that behavior: the field is represented
  as `approved_escalation: Option<bool>` with serde omission and is always
  `None` until a separate TS parity change starts populating it. If YAML input
  includes `approvedEscalation`, it is preserved only under `SkillSandbox.raw`,
  not promoted to the top-level parser-output field.
- Frontmatter splitting: use `regex = "1"` to mirror the TypeScript
  `^---\r?\n([\s\S]*?)\r?\n---\r?\n?([\s\S]*)$` delimiter behavior unless an
  implementation can prove a smaller string-scanner is byte-equivalent through
  fixtures. The dependency is pure and must pass `cargo-deny`.
- Workspace dependency pinning: new parser dependencies are added to
  `[workspace.dependencies]` in `crates/Cargo.toml`, and `runx-parser` uses
  `.workspace = true` for `serde`, `serde_yml`, `regex`, and `thiserror`.
  Existing direct dependency pins in other crates may be left alone unless the
  implementing agent chooses a small mechanical consolidation with no version
  changes.
- Tool manifest and skill install: both are in scope. They are live public
  parser surfaces and have CLI/runtime-local consumers today.
- Runner manifest is the canonical term. Do not use "execution profile" or
  `X.yaml` in implementation names.
- Unsafe-path checks stay part of parser validation if they are pure string
  decisions. Any filesystem path existence or canonicalization belongs in
  `runx-runtime`, not `runx-parser`.

## Touchpoints

- `packages/core/src/parser/index.ts`, `graph.ts`, `install.ts` as the TS
  source of truth.
- `packages/core/src/parser/index.test.ts` and `graph.test.ts` as the first
  fixture coverage map.
- `crates/runx-contracts` for execution-semantics value types only.
- `crates/runx-parser` for parser behavior and typed output.
- `fixtures/parser/**` for cross-language parity.
- `scripts/check-boundaries.mjs` and `scripts/check-rust-core-style.mjs` for
  boundary and style enforcement.

## Risks

- High: YAML parser differences can produce subtly different scalar typing or
  error messages. Mitigation: fixtures constrain YAML inputs to the compatible
  scalar subset, assert validated output and rejection reason enums, and do
  not assert raw third-party parser messages.
- High: parser output shapes are heavily consumed by runtime-local and CLI.
  Mitigation: tool-manifest and skill-install surfaces are in scope, not
  silently deferred.
- High: TS parser imports receipt execution semantics, while Rust parser must
  not depend on `runx-receipts`. Mitigation: Phase 1 moves shared value types
  into `runx-contracts::execution`.
- Medium: exact TS error strings can be brittle. Mitigation: fixtures encode
  typed Rust rejection kinds plus TS messages as evidence; observable CLI
  wording remains TS-authoritative until cutover.
- Medium: parser boundary expansion can expose existing TS impurity. Mitigation:
  add `parser` to `pureCoreDomains` and keep any failure in Phase 1 until
  resolved, not late in implementation.

## Acceptance

Profile: strict

Validation:
- [x] `v1` command - existing TypeScript parser tests pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/core/src/parser/index.test.ts packages/core/src/parser/graph.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-121
- [x] `v2` command - parser fixtures regenerate cleanly.
  - Command: `pnpm exec tsx scripts/generate-rust-parser-fixtures.ts --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-122
- [x] `v3` command - fixture key order is deterministic.
  - Command: `pnpm exec tsx scripts/check-contract-fixture-key-order.ts fixtures/parser`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-123
- [x] `v4` command - Rust parser tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-parser`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-124
- [x] `v5` command - Rust formatting and clippy pass for parser.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-parser --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-125
- [x] `v6` command - repo boundary checks pass.
  - Command: `pnpm boundary:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-126
- [x] `v7` command - Rust style and crate graph checks pass.
  - Command: `node scripts/check-rust-core-style.mjs && node scripts/check-rust-crate-graph.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-127
- [x] `v8` command - parser dependency boundary is clean.
  - Command: `! rg -n 'tokio|reqwest|hyper|rmcp|clap|std::fs|std::process|std::net|std::env|Command::new|serde_json::Value|serde_yml::Value|HashMap' crates/runx-parser/Cargo.toml crates/runx-parser/src`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-128
- [x] `v9` command - parser dependency licenses and bans pass.
  - Command: `cargo deny --manifest-path crates/Cargo.toml check licenses bans sources`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-129

## Phase 1: Type homes, boundary, and fixture contract

Status: completed
Dependencies: `rust-contracts-parity` complete.

Objective: Complete this phase.

Changes:
- `crates/runx-contracts/src/execution.rs` (all, exclusive) - Add `ExecutionSemantics`, `GovernedDisposition`, `ReceiptSurfaceRef`, `InputContextCapture`, `OutcomeState`, and `ReceiptOutcome` used by parser output. Keep behavior-free value types only.
- `crates/runx-contracts/src/lib.rs` (partial, shared) - Export the execution value types explicitly. No wildcard re-exports.
- `fixtures/contracts/execution/*.json` (all, exclusive) - Add direct TypeScript/Rust parity fixtures for execution-semantics contract types, including `ExecutionSemantics`, `GovernedDisposition`, `ReceiptSurfaceRef`, `InputContextCapture`, `OutcomeState`, and `ReceiptOutcome`. Parser fixtures may consume these types, but they are not a substitute for direct contract coverage.
- `scripts/generate-rust-contract-fixtures.ts` (partial, shared) - Add an `execution` fixture scope and `--check --scope execution` support.
- `crates/runx-contracts/tests/execution_fixtures.rs` (all, exclusive) - Load every `fixtures/contracts/execution/*.json` fixture and assert it deserializes into the concrete `runx_contracts::execution` types.
- `scripts/check-rust-core-style.mjs` (partial, shared) - Add `fixtures/contracts/execution` include-str coverage so every execution fixture is referenced by `execution_fixtures.rs`.
- `crates/Cargo.toml` (partial, shared) - Add workspace dependency pins for `serde`, `serde_yml`, `regex`, and `thiserror` before `runx-parser` consumes them. Do not introduce parser-only ad hoc version pins.
- `crates/runx-parser/Cargo.toml` (partial, exclusive) - Consume `serde`, `serde_yml`, `regex`, and `thiserror` through `.workspace = true`. `regex` handles the frontmatter delimiter split; `thiserror` is used for concrete parse/validation error enums. Keep `runx-contracts` and `runx-core` as the only `runx-*` dependencies.
- `crates/runx-parser/src/lib.rs` (all, exclusive) - Replace placeholder constants with module declarations and explicit re-exports.
- `crates/runx-parser/src/error.rs` (all, exclusive) - Define typed parse and validation error enums. Keep messages deterministic and fixture-backed.
- `crates/runx-parser/src/yaml.rs` (all, exclusive) - Private YAML helpers over `serde_yml`; no `serde_yml::Value` anywhere in `src`. Include typed scalar compatibility checks used by the fixture generator.
- `fixtures/parser/README.md` (all, exclusive) - Document fixture schema: categories `skills`, `graphs`, `runner-manifests`, `tool-manifests`, `installs`, and `rejections`; each fixture stores a typed input envelope plus either expected validated JSON or expected rejection kind/message. Skill fixtures use `input.markdown`; parsed raw skill fields live at `expected.raw.frontmatter`, `expected.raw.rawFrontmatter`, and `expected.raw.body`. Raw object subtrees use `runx_contracts::JsonValue`. Document the YAML 1.2 compatible scalar subset and rejected divergent scalar forms.
- `scripts/generate-rust-parser-fixtures.ts` (partial, exclusive) - Create the generator skeleton, fixture JSON shape, and scalar-subset validator.
- `scripts/check-boundaries.mjs` (partial, shared) - Add `parser` to `pureCoreDomains`.
- `docs/rust-kernel-architecture.md` (partial, shared) - Pin `serde_yml` as the parser YAML crate choice and note the `runx_contracts::execution` type home.
- `crates/deny.toml` (partial, shared) - Permit `serde_yml` and any audited transitive licenses needed by parser before implementation reaches Phase 2.

Acceptance:
- [x] `ac1_1` command - execution value types compile.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19
- [x] `ac1_1b` command - direct execution contract fixtures pass.
  - Command: `pnpm exec tsx scripts/generate-rust-contract-fixtures.ts --check --scope execution && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test execution_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20
- [x] `ac1_1d` command - every execution contract type has direct fixture coverage.
  - Command: `for tok in ExecutionSemantics GovernedDisposition ReceiptSurfaceRef InputContextCapture OutcomeState ReceiptOutcome; do rg -q "$tok" crates/runx-contracts/src/execution.rs fixtures/contracts/execution crates/runx-contracts/tests/execution_fixtures.rs || { echo "missing execution fixture token $tok"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21
- [x] `ac1_1c` command - execution contract fixture coverage is enforced.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `ac1_2` command - parser skeleton compiles.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-parser`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `ac1_3` command - parser is enforced as a pure TS core domain.
  - Command: `rg -n 'pureCoreDomains = \[[^\]]*parser' scripts/check-boundaries.mjs && pnpm boundary:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24
- [x] `ac1_4` command - fixture key-order checker rejects unordered JSON.
  - Command: `tmp="$(mktemp -d)" && mkdir "$tmp/good" "$tmp/bad" && printf '{"a":1,"b":2}\n' > "$tmp/good/good.json" && printf '{"b":2,"a":1}\n' > "$tmp/bad/bad.json" && pnpm exec tsx scripts/check-contract-fixture-key-order.ts "$tmp/good" && ! pnpm exec tsx scripts/check-contract-fixture-key-order.ts "$tmp/bad"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-25
- [x] `ac1_5` command - generator rejects divergent YAML scalar forms.
  - Command: `pnpm exec tsx scripts/generate-rust-parser-fixtures.ts --check-scalar-subset`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26
- [x] `ac1_6` command - parser dependency licenses and bans are preflighted.
  - Command: `cargo deny --manifest-path crates/Cargo.toml check licenses bans sources`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-27
- [x] `ac1_7` command - parser third-party dependency versions are workspace-pinned.
  - Command: `for dep in serde serde_yml regex thiserror; do rg -q "^$dep = " crates/Cargo.toml && rg -q "^$dep(\\.workspace = true| = \\{[^}]*workspace = true)" crates/runx-parser/Cargo.toml || { echo "missing workspace dep $dep"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-28

## Phase 2: Graph parity

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `crates/runx-parser/src/graph.rs` or `crates/runx-parser/src/graph/**` (all, exclusive) - Port `parse_graph_yaml`, `validate_graph`, `validate_graph_document`, graph steps, context edges, retry, transition gates, fanout groups, threshold gates, and conflict gates. Split into submodules before any file exceeds the Rust style line cap.
- `fixtures/parser/graphs/*.json` (all, exclusive) - Graph success/rejection fixtures generated from current TS graph tests plus fanout/retry graph examples. Include at least one raw object numeric fixture with an integer scalar (for example `count: 1`) so TS and Rust stable JSON both emit `"count":1`, not `"count":1.0`.
- `crates/runx-parser/tests/parser_fixtures.rs` (partial, exclusive) - Load and assert graph fixtures.
- `crates/runx-parser/tests/parser_rejections.rs` (partial, exclusive) - Cover graph rejection variants.
- `scripts/generate-rust-parser-fixtures.ts` (partial, exclusive) - Generate graph fixture category from TypeScript.
- `scripts/check-rust-core-style.mjs` (partial, shared) - Add graph fixture include-str coverage as soon as `fixtures/parser/graphs` lands.

Acceptance:
- [x] `ac2_1` command - TS parser fixtures regenerate cleanly for graphs.
  - Command: `pnpm exec tsx scripts/generate-rust-parser-fixtures.ts --check --scope graphs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-33
- [x] `ac2_2` command - Rust graph fixture tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-parser --test parser_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-34
- [x] `ac2_3` command - fanout policy shapes are explicitly covered.
  - Command: `for tok in FanoutGroupPolicy FanoutThresholdGate FanoutConflictGate threshold conflict quorum; do rg -q "$tok" crates/runx-parser/src fixtures/parser/graphs || { echo "missing $tok"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-35
- [x] `ac2_4` command - graph implementation respects Rust style caps.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `ac2_5` command - raw numeric canonicalization is fixture-backed.
  - Command: `rg -q '"count":1([,}\\s]|$)' fixtures/parser/graphs && ! rg -q '"count":1\\.0([,}\\s]|$)' fixtures/parser/graphs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37

## Phase 3: Skill and runner-manifest parity

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `crates/runx-parser/src/skill.rs` or `crates/runx-parser/src/skill/**` (all, exclusive) - Port `parse_skill_markdown`, `validate_skill`, `validate_skill_source`, `validate_skill_artifact_contract`, and `extract_skill_quality_profile`. Define parser-owned `SkillSandbox` and the conversion into `runx_core::policy::SandboxDeclaration` here or in `skill/sandbox.rs`. Split into submodules before any file exceeds the Rust style line cap.
- `crates/runx-parser/src/runner.rs` (all, exclusive) - Port `parse_runner_manifest_yaml`, `validate_runner_manifest`, and `resolve_post_run_reflect_policy`.
- `fixtures/parser/skills/*.json` (all, exclusive) - Successful and rejection fixtures for `SKILL.md` parsing. Include `input.markdown`, `expected.raw.rawFrontmatter`, `expected.raw.body`, raw object fields, and validated output assertions. Include at least one successful graph-source skill and one graph-source rejection now that Phase 2 provides graph validation. Include one sandbox fixture whose YAML contains `approvedEscalation`; assert it remains in `expected.validated.source.sandbox.raw` and is omitted from the top-level sandbox output to match current TypeScript behavior.
- `fixtures/parser/runner-manifests/*.json` (all, exclusive) - Runner manifest success/rejection fixtures, including harness cases and catalog metadata. Include the current `skills/issue-intake/X.yaml` harness payload with nested `harness_context.evidence_artifacts` / artifact refs and verification evidence data to prove parser parity preserves recent contract-shaped JSON without retyping it in `runx-parser`.
- `crates/runx-parser/tests/parser_fixtures.rs` (partial, exclusive) - Load and assert skill and runner-manifest fixtures.
- `crates/runx-parser/tests/parser_rejections.rs` (partial, exclusive) - Ensure every parse/validation rejection enum variant has at least one fixture.
- `scripts/generate-rust-parser-fixtures.ts` (partial, exclusive) - Generate Phase 3 fixture categories from TypeScript.
- `scripts/check-rust-core-style.mjs` (partial, shared) - Add skill and runner-manifest fixture include-str coverage as soon as their fixture directories land.

Acceptance:
- [x] `ac3_1` command - TS parser fixtures regenerate cleanly for skills and
  - Command: `pnpm exec tsx scripts/generate-rust-parser-fixtures.ts --check --scope skills,runner-manifests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-42
- [x] `ac3_2` command - Rust fixture tests pass for skills and runner manifests.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-parser --test parser_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-43
- [x] `ac3_3` command - skill raw fields are byte-preserved in fixture corpus.
  - Command: `test "$(find fixtures/parser/skills -name '*.json' | wc -l | tr -d ' ')" -gt 0 && for tok in markdown rawFrontmatter body raw; do rg -q "$tok" fixtures/parser/skills || { echo "missing skill $tok"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-44
- [x] `ac3_4` command - runner-manifest fixture corpus is nonempty and separate.
  - Command: `test "$(find fixtures/parser/runner-manifests -name '*.json' | wc -l | tr -d ' ')" -gt 0 && rg -q 'runner' fixtures/parser/runner-manifests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-45
- [x] `ac3_4b` command - issue-intake evidence payload is parser-preserved.
  - Command: `rg -q 'harness_receipt' fixtures/parser/runner-manifests && rg -q 'evidence_refs' fixtures/parser/runner-manifests && rg -q 'artifact_refs' fixtures/parser/runner-manifests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-46
- [x] `ac3_5` command - parser-owned sandbox wrapper preserves parser-only fields while reusing policy normalization.
  - Command: `for tok in SkillSandbox approvedEscalation approved_escalation raw SandboxDeclaration normalize_sandbox_declaration; do rg -q "$tok" crates/runx-parser/src || { echo "missing sandbox token $tok"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-47
- [x] `ac3_6` command - approvedEscalation mirrors current TypeScript omission semantics.
  - Command: `node -e 'const fs=require("node:fs"); const dir="fixtures/parser/skills"; let found=false; for (const file of fs.readdirSync(dir).filter((name)=>name.endsWith(".json"))) { const fixture=JSON.parse(fs.readFileSync(dir + "/" + file,"utf8")); const sandbox=fixture.expected?.validated?.source?.sandbox; if (sandbox?.raw && Object.hasOwn(sandbox.raw,"approvedEscalation")) { found=true; if (Object.hasOwn(sandbox,"approvedEscalation")) throw new Error(file + " promoted approvedEscalation"); } } if (!found) throw new Error("missing approvedEscalation raw fixture");' && rg -q 'approved_escalation.*Option<bool>|skip_serializing_if.*approved_escalation|approvedEscalation' crates/runx-parser/src`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-48

## Phase 4: Tool manifest and skill-install parity

Status: completed
Dependencies: Phase 3

Objective: Complete this phase.

Changes:
- `crates/runx-parser/src/tool.rs` (all, exclusive) - Port `parse_tool_manifest_yaml`, `parse_tool_manifest_json`, and `validate_tool_manifest`.
- `crates/runx-parser/src/install.rs` (all, exclusive) - Port `validate_skill_install`, including strict skill validation, origin echo, and markdown preservation.
- `fixtures/parser/tool-manifests/*.json` (all, exclusive) - Tool manifest YAML/JSON success/rejection fixtures.
- `fixtures/parser/installs/*.json` (all, exclusive) - Skill-install success/rejection fixtures.
- `crates/runx-parser/tests/parser_fixtures.rs` (partial, exclusive) - Load and assert tool/install fixtures.
- `scripts/generate-rust-parser-fixtures.ts` (partial, exclusive) - Generate Phase 4 fixture categories from TypeScript.
- `scripts/check-rust-core-style.mjs` (partial, shared) - Add tool-manifest and install fixture include-str coverage as soon as their fixture directories land.

Acceptance:
- [x] `ac4_1` command - TS parser fixtures regenerate cleanly for tools and
  - Command: `pnpm exec tsx scripts/generate-rust-parser-fixtures.ts --check --scope tool-manifests,installs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-53
- [x] `ac4_2` command - Rust tool/install fixture tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-parser --test parser_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-54
- [x] `ac4_3` command - public parser surface has no silent omissions.
  - Command: `for tok in parse_tool_manifest_yaml parse_tool_manifest_json validate_tool_manifest validate_skill_install; do rg -q "$tok" crates/runx-parser/src fixtures/parser || { echo "missing $tok"; exit 1; }; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-55

## Phase 5: Governance, docs, and packaging checks

Status: completed
Dependencies: Phase 4

Objective: Complete this phase.

Changes:
- `scripts/check-rust-core-style.mjs` (partial, shared) - Ensure all parser fixture coverage checks landed in Phases 2-4, add public API style checks, and add `serde_yml::Value` to the central forbidden-pattern table alongside `serde_json::Value`.
- `scripts/check-rust-crate-graph.mjs` (partial, shared) - Allow the chosen parser deps and forbid new `runx-*` dependencies beyond contracts/core.
- `crates/deny.toml` (partial, shared) - Permit the chosen YAML crate and deny known incompatible parser deps.
- `crates/runx-parser/README.md` (partial, exclusive) - Replace placeholder README with parser parity status, scope, and TypeScript-authoritative note.
- `docs/rust-kernel-architecture.md` (partial, shared) - Mark parser parity complete in the crate-graph status notes and record any deferred TS parser sunset owner.

Acceptance:
- [x] `ac5_1` command - all parser validation passes.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/core/src/parser/index.test.ts packages/core/src/parser/graph.test.ts && pnpm exec tsx scripts/generate-rust-parser-fixtures.ts --check && pnpm exec tsx scripts/check-contract-fixture-key-order.ts fixtures/parser && cargo test --manifest-path crates/Cargo.toml -p runx-parser`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-60
- [x] `ac5_2` command - Rust style and dependency checks pass.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-parser --all-targets -- -D warnings && node scripts/check-rust-core-style.mjs && node scripts/check-rust-crate-graph.mjs && cargo deny --manifest-path crates/Cargo.toml check licenses bans sources`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-61
- [x] `ac5_3` command - parser crate packages without registry verification.
  - Command: `cargo package --manifest-path crates/Cargo.toml -p runx-parser --allow-dirty --no-verify`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-62
- [x] `ac5_4` command - placeholder markers are gone from parser crate.
  - Command: `! rg -n 'IS_PLACEHOLDER|Placeholder crate|No parsing behavior' crates/runx-parser`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-63

## Rollback

Strategy: per_phase

- Phase 1: remove the new `runx-contracts::execution` module and parser
  skeleton changes, restore the placeholder parser crate, and remove `parser`
  from `pureCoreDomains`.
- Phase 2: remove graph implementation and graph fixtures.
- Phase 3: remove skill/runner implementation files and fixtures generated by
  the phase.
- Phase 4: remove tool/install implementation and fixtures.
- Phase 5: revert guardrail/docs/package-check wiring for parser.

Rollback must not delete completed `rust-contracts-parity`, `runx-core`,
policy, or state-machine work.

## Planning Log

- 2026-05-18T07:45:00Z: Revised after Claude harden round 2. Added explicit
  execution-semantics type home in `runx-contracts`, included tool-manifest and
  skill-install surfaces, pinned `serde_yml = "0.0.12"`, added parser to TS
  pureCoreDomains, renamed execution-profile to runner manifest, and added
  phase/acceptance/rollback bodies.
- 2026-05-18T08:10:00Z: Revised after Claude harden round 3. Reordered graph
  before skill/runner so `validate_skill` can call `validateGraphDocument`
  without stubs, pinned the fixture envelope as `input.markdown` plus
  `expected.raw`, reused `check-contract-fixture-key-order.ts`, constrained
  parser fixtures to a compatible YAML scalar subset, made the strict
  `serde_yml::Value`/`HashMap` ban authoritative for `src`, and documented the
  `runx-core` sandbox dependency.
- 2026-05-18T08:55:00Z: Revised after Claude harden round 4. Parser sandbox
  output now owns a `SkillSandbox` wrapper for `raw` and
  `approvedEscalation` while converting the policy subset into
  `runx_core::policy::SandboxDeclaration`; Phase 1 now adds direct
  `fixtures/contracts/execution` coverage; parser module splits are required
  before style line caps are exceeded; `cargo-deny` license/bans preflight is
  added before implementation; and Phase 3 skill versus runner-manifest
  fixture coverage checks are separated.
- 2026-05-18T09:10:00Z: Revised after Claude harden round 5. Added explicit
  `ReceiptOutcome` ownership and fixture coverage to
  `runx_contracts::execution`; pinned raw object carriers to
  `runx_contracts::JsonObject`/`JsonValue`/`JsonNumber`; added `regex = "1"`
  as the frontmatter delimiter dependency via workspace dependency pins; and
  clarified that `approvedEscalation` is preserved only inside sandbox `raw`
  until the TypeScript validator starts populating the top-level field.
- 2026-05-18T09:25:00Z: Revised after Claude harden round 6. Tightened
  `ac2_5` so the numeric fixture check cannot pass on `"count":1.0`, widened
  `ac1_7` to allow both `.workspace = true` and inline
  `{ workspace = true, features = [...] }` dependency syntax, moved parser
  fixture include-str coverage into the phases where each fixture directory
  lands, and required `serde_yml::Value` in the central Rust style
  forbidden-pattern table.

## Harden Rounds

### round-1

Status: in_progress
Started: 2026-05-18T07:05:09Z
Ended: none

Checks:
- none

Issues:
- none

### round-2

Status: failed
Started: 2026-05-18T07:23:40Z
Ended: 2026-05-18T07:23:40Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Spec is still skeletal (Scope/Phases/Acceptance bodies empty) and carries several un-resolved architectural premises that would harm execution: it assumes `runx-contracts` already exposes parser AST types (it does not), ignores that the TS parser pulls `ExecutionSemantics`/`GOVERNED_DISPOSITIONS` from receipts even though the Rust crate graph bans a parser→receipts dep, omits the heavily-consumed tool-manifest and skill-install surface from objectives/files, leaves the YAML library and `pureCoreDomains` boundary extension as unresolved open questions, and miscounts the pure-by-imports domains. Approval should wait for the operator to answer the receipts-type ownership question, declare the tool-manifest and install posture, lock the YAML library, and commit a phases/acceptance skeleton consistent with the policy/state-machine precedents.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-parser/src/lib.rs:1
  - Result: passed
  - Evidence: Declared destination files (Cargo.toml, src/lib.rs, src/skill.rs, src/graph.rs, src/profile.rs, tests/parser_fixtures.rs, scripts/generate-rust-parser-fixtures.ts, fixtures/parser/**) are intentionally future paths; runx-parser today is only the placeholder lib.rs. Architectural references in the spec (packages/core/src/parser/index.ts, docs/rust-kernel-architecture.md, scripts/check-rust-core-style.mjs, scripts/check-rust-crate-graph.mjs, crates/deny.toml) exist and were read.
- command audit
  - Grounded in: spec_gap:acceptance
  - Result: not_applicable
  - Evidence: The draft has no Acceptance section content rendered (body=0 bytes); no commands declared yet to audit. Validation commands would normally mirror rust-policy-parity (cargo test -p runx-parser, fmt/clippy, fixture-coverage guards, deny check, boundary check). This is a gap, not a failure of stated commands.
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:1
  - Result: failed
  - Evidence: Objectives list skill/graph/execution-profile only. But oss/packages/core/src/parser/index.ts exports parseToolManifestYaml/parseToolManifestJson/validateToolManifest, and oss/packages/core/src/parser/install.ts exports validateSkillInstall. Importers include packages/cli/src/commands/tool.ts, packages/cli/src/commands/dev/fixture-runner.ts, packages/runtime-local/src/sdk/index.ts, packages/runtime-local/src/runner-local/skill-install.ts. The Files-impacted list does not include tool.rs/install.rs and there is no statement deferring them; this is a silent scope gap.
- acceptance timing audit
  - Grounded in: spec_gap:phases
  - Result: failed
  - Evidence: Both 'Planned Phases' and 'Acceptance And Rollback' sections render as 0 bytes. For a large/medium-risk spec, approval without phase decomposition or acceptance commands lacks the discipline used by rust-policy-parity and rust-state-machine-parity, where each phase declared concrete `cargo test --manifest-path` and style/deny gates.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: not_applicable
  - Evidence: No rollback statement is rendered. Comparable archived specs (rust-policy-parity) use 'Strategy: per_phase'; since parser is additive (the placeholder lib.rs has no consumers and no TS consumer is replaced in this spec), per-phase rollback via revert is credible. Cannot evaluate further until the spec writes it.
- design challenge
  - Grounded in: code:oss/packages/core/src/parser/index.ts:4
  - Result: failed
  - Evidence: Three architectural premises are unsound or unstated: (1) TS parser imports `GOVERNED_DISPOSITIONS` and `ExecutionSemantics` from ../receipts/index.js, yet runx-parser is allowed only runx-contracts + runx-core as runx deps (oss/scripts/check-rust-crate-graph.mjs:31) and runx-contracts/src/receipts.rs is an empty deferred module. The spec invariant 'Existing runx-contracts types are reused; the parser does not re-declare validated skill/graph/execution-profile shapes' is unsatisfiable today because those parser AST types are not in runx-contracts (grep for SkillSource/ValidatedSkill/ExecutionGraph in crates/runx-contracts/src yields no matches). (2) oss/scripts/check-boundaries.mjs:86 defines pureCoreDomains = ['policy','state-machine']; docs/rust-kernel-architecture.md section 2 explicitly says parser's pureCoreDomains entry must be added before a Rust port is meaningful, but the spec does not mention extending check-boundaries.mjs. (3) The spec's claim that parser is 'one of the three pure-by-imports trusted-kernel domains identified in section 2' contradicts the doc, which lists five (executor, marketplaces, parser, state-machine, policy).

Issues:
- [critical/blocks approval] `harden-1` architecture - Parser → receipts dependency in TS conflicts with Rust crate-graph rule; ExecutionSemantics/GovernedDisposition have no Rust home.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:4
  - Evidence: packages/core/src/parser/index.ts:4 imports { GOVERNED_DISPOSITIONS, ExecutionSemantics } from ../receipts/index.js. crates/runx-contracts/src/receipts.rs is an explicit deferred-empty module ('intentionally exposes no public receipt types until that fixture-backed split lands'). oss/scripts/check-rust-crate-graph.mjs:31 limits runx-parser to runx-contracts + runx-core. The spec invariant says parser must not re-declare types, yet ExecutionSemantics is owned by the not-yet-ported receipts domain in TS. The spec is silent on which crate owns these in Rust.
  - Recommendation: Resolve before approval: choose one of (a) move ExecutionSemantics + GovernedDisposition into runx-contracts as a public sub-module with TS-parity fixtures (preferred — keeps the parser→receipts cutover out of this spec); (b) inline pub(crate) copies in runx-parser with explicit fixture parity and a follow-up TODO to fold into runx-receipts; (c) reorder the crate graph so runx-receipts ships first. Option (a) matches the precedent set by rust-contracts-parity for host-protocol types and avoids re-declaring receipt-owned shapes.
  - Question: Where do `ExecutionSemantics` and `GOVERNED_DISPOSITIONS` live in Rust for this spec — runx-contracts (preferred), inline pub(crate) in runx-parser, or do we wait for runx-receipts?
  - Recommended answer: Add them to runx-contracts (e.g. `runx_contracts::execution::{ExecutionSemantics, GovernedDisposition}`) in Phase 1 of this spec, with their own fixture coverage; rust-receipts-parity later re-uses them.
  - If unanswered: Default to option (a): move ExecutionSemantics and GovernedDisposition into runx-contracts as part of Phase 1 of rust-parser-parity.
- [high/blocks approval] `harden-2` scope - Tool-manifest and skill-install surfaces are missing from objectives and impacted files despite live consumers.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:273
  - Evidence: parser/index.ts exports parseToolManifestYaml/parseToolManifestJson/validateToolManifest. parser/install.ts exports validateSkillInstall. Active consumers: packages/cli/src/commands/tool.ts, packages/cli/src/commands/dev/fixture-runner.ts, packages/runtime-local/src/sdk/index.ts, packages/runtime-local/src/runner-local/skill-install.ts, packages/runtime-local/src/tool-catalogs/index.ts. Spec only names skill/graph/execution-profile and impacted files only list skill.rs/graph.rs/profile.rs.
  - Recommendation: Either (1) add `tool.rs` + `install.rs` to impacted files and add explicit objectives for `validateToolManifest`/`validateSkillInstall` parity (with their own fixture set), or (2) explicitly defer them to a follow-up spec (e.g. `rust-parser-tool-manifest-parity`) and add an Out-of-Scope bullet plus a dependency note in `rust-ts-sunset-parser`. Silent omission is the failure mode to avoid.
  - Question: Are tool-manifest parsing (`validateToolManifest` + JSON/YAML variants) and `validateSkillInstall` in scope for this spec, or deferred to a follow-up?
  - Recommended answer: In scope. Defer skill-install only if it is purely a registry-shaped wrapper (it is — install.ts:21 is parseSkillMarkdown + validateSkill + origin echo), but keep tool-manifest in this spec; the CLI and SDK already consume it and the Rust runtime cannot run without it.
  - If unanswered: Add `tool.rs` and `install.rs` to the impacted-files list, add corresponding objectives, and extend fixture coverage to tool manifests and install validation.
- [high/blocks approval] `harden-3` boundary - Spec ignores the architecture-doc requirement to extend `pureCoreDomains` to parser before a Rust port is meaningful.
  - Status: open
  - Grounded in: code:oss/scripts/check-boundaries.mjs:86
  - Evidence: scripts/check-boundaries.mjs:86 defines pureCoreDomains = ['policy','state-machine']. docs/rust-kernel-architecture.md section 2 says: 'The remaining pure-by-imports domains (executor, marketplaces, parser) are candidates for follow-up parity specs; their boundary status would need to be added to pureCoreDomains before a Rust port is meaningful.' The spec does not list a phase that edits check-boundaries.mjs to add 'parser' to pureCoreDomains, and the TS parser still imports `yaml` (npm) and `../receipts/index.js` — the former is fine, the latter is a cross-domain import that the boundary check would not currently flag for parser even if extended.
  - Recommendation: Add an explicit phase (or call out as a prerequisite spec) that: (1) adds `parser` to `pureCoreDomains` in scripts/check-boundaries.mjs, (2) verifies parser has no forbidden node imports (the yaml npm package is fine; `../receipts/index.js` is the one to fix or recharter), and (3) documents the cross-domain receipts import status. Without this, the 'Rust parity is conformance evidence for pure TS kernel' premise is hollow.
  - Question: Does this spec extend `pureCoreDomains` to include 'parser' (and resolve the parser→receipts TS import), or is that broken into a separate TS-purification spec like the node:path removal was?
  - Recommended answer: Bundle the `pureCoreDomains` addition into Phase 1 of this spec, with the receipts-types relocation in issue #1 above as the prerequisite TS-side change; mirror the pattern that rust-kernel-parity-fixtures used for the `node:path` removal.
  - If unanswered: Add a Phase that edits scripts/check-boundaries.mjs to add 'parser' to pureCoreDomains, and flag the parser→receipts import as a blocker until issue #1 is resolved.
- [high/blocks approval] `harden-4` spec_completeness - Planned Phases, Scope/Touchpoints, and Acceptance/Rollback bodies are empty for a large/medium-risk spec.
  - Status: open
  - Grounded in: spec_gap:phases
  - Evidence: Harden context manifest shows rendered=0 body=0 for sections `scope`, `phases`, and `acceptance`. The comparable archived specs (rust-policy-parity, rust-state-machine-parity, rust-contracts-parity) all carry phase decomposition with concrete cargo/pnpm/node commands per phase before approval.
  - Recommendation: Before approval, decompose into at least 4 phases mirroring rust-policy-parity: (P1) data-model + serde shapes + fixture-generator skeleton; (P2) skill + graph parity fixtures pass; (P3) runner manifest + tool manifest + install parity fixtures pass; (P4) gap docs (architecture doc + fixtures README + boundary script extension). Add validation commands for each (cargo test -p runx-parser, fmt+clippy on the crate, fixture-coverage guard in check-rust-core-style.mjs, parity script `pnpm tsx scripts/generate-rust-parser-fixtures.ts`, deny check).
  - Question: Can we commit phase decomposition and acceptance commands now, mirroring rust-policy-parity, or do you want to defer that to a second harden pass?
  - Recommended answer: Commit phases and acceptance now; this spec is too large to approve without them and the policy/state-machine precedents are direct templates.
  - If unanswered: Add the four-phase skeleton above with cargo test, fmt/clippy, fixture-coverage, boundary, and deny commands per phase.
- [high/blocks approval] `harden-5` dependency - YAML library choice is left as an open question yet is load-bearing for the public API, fixtures, and error mapping.
  - Status: open
  - Grounded in: spec_gap:open_questions
  - Evidence: Open Questions section: 'YAML library choice: serde_yaml is unmaintained; serde_yml (fork) is active. Pick one with explicit rationale in Phase 1.' But Cargo.toml and crates/deny.toml have no YAML dep yet, and the architecture doc (section 3) says `runx-parser` deps include 'serde_yaml or equivalent'. The chosen lib determines: error type wrapping, location/span carriage, document/scalar conversion to runx-contracts::JsonValue, and whether fixtures can be byte-identical with TS `yaml` package output for raw frontmatter preservation.
  - Recommendation: Decide before approval. Recommendation: `serde_yml` (active fork, drop-in for serde_yaml). Pin major version in workspace deps and document the choice + rationale in the architecture doc section 3, then keep this spec's Open Question 'whether unsafe-paths denial stays in TS' separate.
  - Question: Lock the YAML library now (recommend serde_yml v0.0.x), or do you want to leave it for Phase 1?
  - Recommended answer: Lock `serde_yml` now. Pin in workspace deps; document rationale; add a fixture-coverage round-trip in Phase 1 that proves the `raw` field for skill markdown is byte-preserved.
  - If unanswered: Lock `serde_yml` with rationale, pin via workspace deps, and update docs/rust-kernel-architecture.md section 3.
- [medium/advisory] `harden-6` terminology - 'Execution-profile (X.yaml)' is non-canonical naming for what the TS surface calls the runner manifest.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:100
  - Evidence: The TS exports are parseRunnerManifestYaml/validateRunnerManifest/SkillRunnerManifest (parser/index.ts:100,239,344). The string 'X.yaml' and 'execution profile' appear nowhere else in oss/docs or in the parser source. Using a new term in the spec invites scope drift between authors.
  - Recommendation: Rename to 'runner manifest (`runners.yaml` / inline `runners` block)' to match the TS function and type names; if there is a separate 'execution profile' file format on disk, name it explicitly and link the schema. Otherwise drop the X.yaml notation.
  - Question: Is 'execution-profile (X.yaml)' meant to be the runner manifest, or a separate format we have not introduced yet?
  - Recommended answer: It is the runner manifest. Rename the bullet to 'runner-manifest (runners.yaml) parser' and align with the TS function names.
  - If unanswered: Rename to runner-manifest throughout and align objectives + impacted files (`runner.rs` or `profile.rs` with a note).
- [low/advisory] `harden-7` factual - Spec says parser is one of three pure-by-imports trusted-kernel domains, but the architecture doc lists five.
  - Status: open
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:38
  - Evidence: docs/rust-kernel-architecture.md section 2 lists five node-import-free domains: executor, marketplaces, parser, state-machine, policy. The spec summary says 'one of the three pure-by-imports trusted-kernel domains identified in section 2'.
  - Recommendation: Rephrase to 'parser is one of five pure-by-imports trusted-kernel domains (section 2); three remain unported after this spec (executor, marketplaces, and post-port parser itself becomes the next runtime-binding gate).' Or simply 'pure-by-imports trusted-kernel domain' without the count.
  - If unanswered: Rewrite to drop the inaccurate '/three/' count.
- [medium/advisory] `harden-8` fixture_design - Cross-language fixture suite shape is not yet specified for a 1,658-line parser with 30+ exported types.
  - Status: open
  - Grounded in: spec_gap:scope
  - Evidence: Spec says 'Add a generator script that emits fixtures from the TS parser and a Rust fixture-runner test that consumes them' but does not specify: (a) categories (e.g. skills/, graphs/, runner-manifests/, tool-manifests/, rejections/); (b) whether each fixture stores input string + expected validated JSON, or input + raw IR + validated IR; (c) error-case coverage (every SkillParseError/SkillValidationError/GraphParseError/GraphValidationError message currently thrown); (d) whether `raw`/`rawFrontmatter`/`body` byte-preservation is asserted as part of parity.
  - Recommendation: Add a Phase 1 fixture-design note: directory categories (skills, graphs, runner-manifests, tool-manifests, rejections), per-case JSON shape { input, expected_validated, expected_rejection? }, raw-field byte-preservation assertion, and a generator that walks every TS test fixture used by parser/index.test.ts and parser/graph.test.ts. Match the precedent set by fixtures/kernel/policy and fixtures/kernel/state-machine.
  - Question: Will the parser fixture suite preserve the `raw` / `rawFrontmatter` / `body` fields byte-for-byte, and will every TS rejection branch get an explicit fixture?
  - Recommended answer: Yes to both. Without raw-field byte preservation the parser is not safe for receipts/idempotency hashing downstream, and without rejection coverage the typed-enum design has no parity floor.
  - If unanswered: Default: preserve raw fields byte-for-byte and require one fixture per current TS rejection branch.

### round-3

Status: failed
Started: 2026-05-18T07:34:14Z
Ended: 2026-05-18T07:34:14Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 architectural blockers are addressed: ExecutionSemantics+GovernedDisposition land in runx_contracts::execution (Phase 1), tool-manifest and skill-install are explicit in scope and impacted files (Phase 4), serde_yml is pinned, parser is added to pureCoreDomains, and phase/acceptance/rollback bodies exist. The remaining issues are smaller but two are still execution-blocking: (1) Phase 2 ports skill validation, yet validateSource(type=\"graph\") delegates to validateGraphDocument — Phase 3 is where graph parity lands, so Phase 2 cannot ship complete parity without either reordering or an explicit stub policy; (2) the ac2_3 grep includes `rawMarkdown`, but no such field exists in the TS RawSkillIR (the actual fields are `frontmatter`, `rawFrontmatter`, `body`) and the fixture envelope shape that would carry it is only described as \"input plus expected validated JSON\" in Phase 1 — the token mismatch will make ac2_3 fail unless the fixture format pins a `rawMarkdown` input field. The remaining advisory items (YAML scalar typing risk to the `raw: Record<String, JsonValue>` byte-preservation claim, redundant fixture-key-order script, `serde_yml::Value` invariant vs v8 grep, undocumented use of runx-core deps) should be answered before approval but do not block.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-parser/src/lib.rs:1
  - Result: passed
  - Evidence: All declared destination files are intentionally future paths or existing files. crates/runx-parser/src/lib.rs:1-15 is still the placeholder; crates/runx-parser/Cargo.toml:18-21 has only runx-contracts+runx-core deps and no serde_yml or serde yet — matches Phase 1 plan. crates/runx-contracts/src/lib.rs:1-24 has no `execution` module today (Phase 1 will add it). Authoritative TS paths exist and were read: packages/core/src/parser/index.ts (1-200,320,450-568), packages/core/src/parser/install.ts (1-29), packages/core/src/parser/graph.ts (5-126), packages/core/src/receipts/index.ts (90-135). Guardrail scripts exist: scripts/check-boundaries.mjs:86, scripts/check-rust-core-style.mjs:6-14, scripts/check-rust-crate-graph.mjs:27-43, crates/deny.toml:1-13.
- command audit
  - Grounded in: code:oss/scripts/check-boundaries.mjs:86
  - Result: passed
  - Evidence: ac1_3 (`rg -n 'pureCoreDomains = \[[^\]]*parser' scripts/check-boundaries.mjs && pnpm boundary:check`) is executable against scripts/check-boundaries.mjs:86 (currently `["policy","state-machine"]`). Adding 'parser' satisfies the grep, and the boundary check would still pass because the only pure-domain restrictions enforced on parser today are forbiddenPureNodeImports (the TS parser imports `yaml` (npm), `../receipts/index.js`, `../util/types.js`, `./graph.js` — none are in forbiddenPureNodeImports, and the executor/tool-catalogs check at scripts/check-boundaries.mjs:187 does not target receipts). ac1_4 mirrors the working scripts/check-contract-fixture-key-order.ts:5-26 pattern (target directory arg, recurse listJsonFiles, compare stableJson). ac5_3 `cargo package --allow-dirty` matches rust-contracts-parity precedent (archive/2026-05/rust-contracts-parity.md:291). v8 grep is a strict superset of scripts/check-rust-core-style.mjs:22-44 bans, restricted to crates/runx-parser/Cargo.toml + crates/runx-parser/src and not tests, consistent with tests/ legitimately using serde_json::Value (see crates/runx-core/tests/policy_fixtures.rs:121).
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:215
  - Result: passed
  - Evidence: Round-2 scope gap is now closed. Phase 4 explicitly ports `parse_tool_manifest_yaml/json` + `validate_tool_manifest` (parser/index.ts:256-292,457-482) and `validate_skill_install` (parser/install.ts:21-29). Phase 1 moves ExecutionSemantics, GovernedDisposition, ReceiptSurfaceRef, InputContextCapture, OutcomeState into runx_contracts::execution, which matches the TS surface at packages/core/src/receipts/index.ts:94-135 and resolves the parser→receipts crate-graph violation. `runners.yaml`/`runner manifest` naming matches packages/core/src/parser/index.ts:239-254,344. The five-domain count is corrected (no longer claims `one of three`). The `parser` pureCoreDomains extension is bundled into Phase 1 (changes list + ac1_3).
- acceptance timing audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:540
  - Result: failed
  - Evidence: Phase 2 ports skill.rs (validate_skill etc), but validateSource at packages/core/src/parser/index.ts:540 calls validateGraphSource which delegates to validateGraphDocument (index.ts:572 → graph.ts:123). Graph parity is Phase 3. Therefore Phase 2's acceptance `cargo test -p runx-parser --test parser_fixtures` cannot reach parity for any skill fixture whose `source.type === "graph"`, and the strict-mode validate_skill path cannot be feature-complete until Phase 3. Phase ordering needs either (a) Phase 2 to explicitly defer/stub type=graph skill sources with a tracked TODO, or (b) reorder graph to Phase 2 and skill to Phase 3, or (c) split skill into a graph-stubbed Phase 2a and graph-aware Phase 2b. The spec is silent on this cross-phase dependency. Independently, ac2_3 greps for the token `rawMarkdown` in fixtures, but `RawSkillIR` at parser/index.ts:9-13 only defines `frontmatter`, `rawFrontmatter`, `body`; no `rawMarkdown` field exists in TS. Either the fixture envelope must define a `rawMarkdown` input field (not pinned in Phase 1 README spec), or the token list is wrong and ac2_3 will fail.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Per-phase rollback is plausible. Phase 1 is additive: a new runx_contracts::execution module and parser skeleton; reverting the contracts addition is safe because crates/runx-contracts/src/receipts.rs explicitly leaves receipt types out today and no other crate consumes execution before this spec. Adding `parser` to pureCoreDomains and reverting if Phase 1 fails is mechanical. Phases 2-4 are additive code+fixtures inside runx-parser. Phase 5 reverts script wiring. Rollback explicitly preserves rust-contracts-parity, runx-core, policy, state-machine work. The only sharp edge is that crates/runx-contracts package version stays at 0.0.1 reservation — `cargo package -p runx-parser --allow-dirty` will resolve via path deps (precedent in archive/2026-05/rust-contracts-parity.md:291).
- design challenge
  - Grounded in: code:oss/packages/core/src/parser/index.ts:1
  - Result: passed
  - Evidence: Architectural premises are now coherent: (1) The crate-graph violation is resolved by moving ExecutionSemantics+GovernedDisposition into runx_contracts::execution, keeping runx-parser dependent only on runx-contracts+runx-core (matches scripts/check-rust-crate-graph.mjs:31). The TS parser keeps importing from `../receipts/index.js` (packages/core/src/parser/index.ts:4) — this is acceptable because scripts/check-boundaries.mjs:183-189 only forbids node IO + executor/tool-catalogs imports for pure domains, not cross-pure-domain imports. (2) YAML library is pinned to serde_yml = "0.0.12" with rationale; this matches docs/rust-kernel-architecture.md section 3 wording (`serde_yaml or equivalent`). (3) Five pure-by-imports domains count is consistent with docs/rust-kernel-architecture.md:36-44. The remaining design risk is acknowledged-but-not-fully-mitigated YAML scalar typing divergence (eemeli/yaml vs libyaml) for the byte-preservation claim on `raw` fields — see issue h2-1.

Issues:
- [high/blocks approval] `h3-1` phase_ordering - Phase 2 ports validate_skill but skill validation invokes graph validation; Phase 3 graph parity must come first or Phase 2 must stub the graph path.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:540
  - Evidence: packages/core/src/parser/index.ts:540 calls validateGraphSource(source.graph) when type==="graph"; index.ts:570-572 delegates to validateGraphDocument from graph.ts (graph.ts:123). Phase 2 ports skill+runner per draft Phases section; Phase 3 ports graph. Phase 2's acceptance command `cargo test -p runx-parser --test parser_fixtures` cannot represent full skill parity until graph.rs exists. If implementers proceed in declared order, either the fixture corpus must exclude all type=graph skill fixtures (silent gap) or skill.rs must contain an unfounded panic/stub at the call site — both contradict the strict validation profile and the dod3 statement that 'fixtures cover successful and rejected cases for all parser surfaces'.
  - Recommendation: Choose one before approval: (a) reorder Phase 2 ↔ Phase 3 so graph.rs lands first and skill.rs in Phase 3 can call it; (b) split skill.rs across phases — Phase 2 ships skill+runner with type=graph sources explicitly returning a typed SkillValidationError::GraphSourceNotYetSupported, with fixtures asserting that rejection and a Phase 3 fixture re-classifying the same input as success once graph.rs lands; (c) bundle skill+graph into a single phase. Option (a) is cleanest because graph types are independent of skill types, while skill types depend on ExecutionGraph.
  - Question: How should the skill→graph cross-phase dependency be handled — reorder graph before skill, or land skill with a typed `graph-source not supported in this phase` rejection that Phase 3 promotes to success?
  - Recommended answer: Reorder: Phase 2 = graph parity, Phase 3 = skill+runner parity. Graph has no upstream parser dependency, while skill depends on validateGraphDocument; reversing the order avoids stubs and matches how the TS modules import each other (parser/index.ts:3 imports from ./graph.js).
  - If unanswered: Reorder phases to put graph first.
- [high/blocks approval] `h3-2` fixture_format - ac2_3 greps fixtures for `rawMarkdown` but TS RawSkillIR has no such field and Phase 1 README spec does not pin a fixture envelope that would carry it.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:9
  - Evidence: Acceptance ac2_3 token list is `rawMarkdown rawFrontmatter body raw`. packages/core/src/parser/index.ts:9-13 defines RawSkillIR as { frontmatter, rawFrontmatter, body } — there is no `rawMarkdown` field anywhere in the TS parser surface (grep `rg rawMarkdown packages/core/src/parser` returns nothing). The Phase 1 fixture README description only says 'each fixture stores input plus either expected validated JSON or expected rejection kind/message' without pinning the input envelope key. The Phase 2 generator+fixture acceptance will therefore fail at the grep step unless the implementer guesses to introduce a `rawMarkdown` envelope field — which is not documented and risks token-vs-shape drift in test runners.
  - Recommendation: Pin the fixture envelope in Phase 1 README spec: e.g. `{ name, input: { markdown, ... }, expected: { rawMarkdown: string, rawFrontmatter: string, body: string, validated?: ..., rejection?: { kind, message } } }`, OR change ac2_3 to grep tokens that actually appear in the TS surface (`rawFrontmatter`, `body`, `raw`). Also align Phase 1 generator skeleton with the chosen envelope so Phase 2 isn't choosing field names ad hoc.
  - Question: Should the fixture envelope use `rawMarkdown` as the input-side key (so ac2_3 finds it), or should the grep token list drop `rawMarkdown` in favor of the actual TS field names?
  - Recommended answer: Pin the envelope: `input.markdown` carries the raw skill markdown, `expected.raw` mirrors RawSkillIR, including `expected.raw.rawFrontmatter` and `expected.raw.body`. Drop `rawMarkdown` from ac2_3 and use `markdown rawFrontmatter body raw` (or include `markdown` as the envelope input key).
  - If unanswered: Define `input.markdown` envelope field and update ac2_3 to grep `markdown rawFrontmatter body raw`.
- [medium/advisory] `h3-3` byte_preservation - `raw: Record<string, unknown>` byte-preservation across two YAML implementations is not pinned by the fixture mitigation.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:53
  - Evidence: ValidatedSkill (parser/index.ts:97), SkillSource (parser/index.ts:53), SkillSandbox (parser/index.ts:77), SkillRunnerDefinition (parser/index.ts:125), ValidatedTool (parser/index.ts:194), RawRunnerManifestIR (parser/index.ts:100), RawToolManifestIR (parser/index.ts:105) all carry a `raw` field that is the parsed YAML/JSON object. The TS yaml library (`eemeli/yaml`) and `serde_yml` (libyaml-based) differ on scalar typing for sexagesimal numbers, octal/hex integers, YAML 1.1 vs 1.2 booleans, dates, and unquoted strings that look numeric. The Risks section acknowledges this but says 'fixtures assert validated output and rejection reason enums, not raw third-party parser messages' — that protects error messages, but the `raw` field is observable validated output and dod3 claims raw-field byte preservation.
  - Recommendation: Either (a) constrain fixture YAML inputs to YAML 1.2 scalar subsets that both parsers agree on (no sexagesimal, no implicit booleans beyond true/false, all dates quoted), and document this in Phase 1 fixture README; or (b) downgrade dod3 to 'raw text fields (rawFrontmatter, body) are byte-preserved; raw object fields are structurally equivalent under the canonical JSON serializer' and add a normalization step in the Rust fixture loader. Without one of these, Phase 2/3 fixtures will be brittle.
  - Question: Should fixture YAML inputs be constrained to a YAML 1.2 scalar subset both parsers agree on, or should the Rust fixture loader canonicalize `raw` before assert?
  - Recommended answer: Constrain inputs to YAML 1.2 scalar subset; document the restriction in Phase 1 fixture README; add a Phase 1 ac that the generator rejects inputs using sexagesimal, implicit yes/no booleans, or unquoted dates.
  - If unanswered: Add fixture README constraint plus a generator-side input validator and a Phase 1 acceptance that the generator refuses divergent-scalar inputs.
- [low/advisory] `h3-4` tooling - A new `scripts/check-parser-fixture-key-order.ts` duplicates the existing contract checker; reuse instead.
  - Status: open
  - Grounded in: code:oss/scripts/check-contract-fixture-key-order.ts:5
  - Evidence: scripts/check-contract-fixture-key-order.ts already implements stable-JSON checking on a configurable target directory (lines 5-26): it accepts a path arg, recurses, and compares against `stableJson(JSON.parse(actual))`. scripts/check-fixture-key-order.ts also exists for kernel fixtures. Phase 1 introduces a third near-identical script. The validation `v3` and acceptance `ac1_4` both target `fixtures/parser`; passing a directory to the existing contract checker would suffice.
  - Recommendation: Either generalize scripts/check-contract-fixture-key-order.ts (rename to scripts/check-json-fixture-key-order.ts and pass `fixtures/parser` as target) or import its helper from the new script. Avoid maintaining three near-identical scripts.
  - Question: Should this spec add a third key-order script, or generalize and reuse scripts/check-contract-fixture-key-order.ts?
  - Recommended answer: Generalize: rename scripts/check-contract-fixture-key-order.ts to scripts/check-json-fixture-key-order.ts (or keep the name and pass a target) and reuse for parser fixtures. Drop scripts/check-parser-fixture-key-order.ts from Phase 1 impacted files.
  - If unanswered: Reuse the existing checker by passing `fixtures/parser` as its target; remove the new script from Phase 1.
- [low/advisory] `h3-5` consistency - Invariant says `no serde_yml::Value in public APIs`, but v8 forbids it in any src/ file — including private helpers.
  - Status: open
  - Grounded in: code:oss/.scafld/specs/drafts/rust-parser-parity.md:89
  - Evidence: Invariants list at .scafld/specs/drafts/rust-parser-parity.md:89: `No serde_json::Value or serde_yml::Value in the public API`. Validation v8 runs `! rg -n 'tokio|reqwest|hyper|rmcp|clap|std::fs|std::process|std::net|std::env|Command::new|serde_json::Value|serde_yml::Value|HashMap' crates/runx-parser/Cargo.toml crates/runx-parser/src` — this fires on any src/* match including private helpers in src/yaml.rs. Either the invariant or the check is wrong.
  - Recommendation: Tighten the invariant to match the check (`serde_yml::Value must not appear anywhere in src/`) and ensure src/yaml.rs uses typed serde deserializers throughout. Direct typed deserialization is feasible because the TS parser's `parseDocument(...).toJS()` is already structured (no need for ad-hoc Value traversal). If a private helper genuinely needs serde_yml::Value, escape-hatch it via `serde_yml::Value` aliased to a typed sum and put it under a `// rust-style-allow:` marker.
  - Question: Is the constraint `no serde_yml::Value anywhere in src/` (matching v8) intentional, or should v8 be relaxed for private helpers?
  - Recommended answer: Make the strict v8 rule authoritative: `no serde_yml::Value anywhere in src/`. Update the invariant text accordingly. Implementers must deserialize directly into typed enums/structs.
  - If unanswered: Tighten invariant to forbid serde_yml::Value anywhere in src/; do not relax v8.
- [low/advisory] `h3-6` scope - runx-parser → runx-core dependency is declared but the spec doesn't identify which runx-core types parser actually reuses.
  - Status: open
  - Grounded in: code:oss/scripts/check-rust-crate-graph.mjs:31
  - Evidence: scripts/check-rust-crate-graph.mjs:31 lists `runx-parser` allowed runx deps as { runx-contracts, runx-core }; docs/rust-kernel-architecture.md section 3 says parser deps include runx-core. The spec keeps the dep but does not name what parser will import. crates/runx-core exposes `policy` (with SandboxDeclaration, normalize_sandbox_declaration, etc.) and `state_machine`. The TS parser performs its own sandbox validation (parser/index.ts:541, validateSandbox) which conceptually overlaps with runx-core::policy::sandbox. Whether Rust parser reuses that helper or duplicates it is unspecified.
  - Recommendation: Add a one-line decision: either (a) parser reuses runx_core::policy::sandbox::SandboxDeclaration + normalize_sandbox_declaration to avoid duplicate sandbox normalization logic across crates (preferred, since sandbox admission is policy-owned and sandbox shape is part of validated skill output), or (b) parser duplicates sandbox value types locally and the runx-core dep is downgraded to optional. Pin in Phase 1 Architecture Decisions.
  - Question: Does runx-parser reuse runx_core::policy::sandbox for sandbox normalization, or does it duplicate sandbox types locally?
  - Recommended answer: Reuse runx_core::policy::sandbox. Sandbox normalization is policy-owned, and the validated skill output already needs the same SandboxDeclaration shape policy admits. Duplicating it would create a drift point between parser and policy across the two-language ports.
  - If unanswered: Reuse runx_core::policy::sandbox; document in Phase 1 Architecture Decisions.

### round-4

Status: failed
Started: 2026-05-18T07:44:21Z
Ended: 2026-05-18T07:44:21Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-3 blockers are addressed: graph parity now lands before skill (Phase 2 → graph, Phase 3 → skill+runner) so validate_skill can call validate_graph_document without stubs; the fixture envelope is pinned to input.markdown + expected.raw; check-contract-fixture-key-order.ts is reused; the YAML scalar subset is constrained and policed by a Phase 1 generator flag; the strict serde_yml::Value/HashMap ban is authoritative for src/; and runx-core sandbox reuse is documented. Two execution gaps remain. (1) The architecture decision says runx-parser reuses runx_core::policy::SandboxDeclaration, but the TS SkillSandbox carries `raw: Record<string,unknown>` and `approvedEscalation?: boolean` (packages/core/src/parser/index.ts:69–78) that have no counterpart on the Rust SandboxDeclaration (crates/runx-core/src/policy/types.rs:178–190) — the spec is silent on whether parser wraps the policy type, declares its own SkillSandbox struct, or drops `raw`, and the fixture-coverage ac in Phase 3 cannot pass either way without an answer. (2) Phase 1 lands runx_contracts::execution (ExecutionSemantics/GovernedDisposition/ReceiptSurfaceRef/OutcomeState/InputContextCapture) but adds no dedicated fixtures/contracts/execution parity suite analogous to fixtures/contracts/act-assignment — the parser fixtures imply the wire shape but there is no direct ts↔rust parity floor for these new contract types, weakening dod2. Three smaller items are advisory: the 350-line check-rust-core-style.mjs cap is unaddressed for skill.rs/graph.rs; the deny.toml license allowlist (Apache-2.0/MIT/Unicode-3.0) has no preflight against serde_yml transitive deps; and the ac3_3 grep merges two fixture directories, allowing the runner-manifest corpus to be empty.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-parser/src/lib.rs:1
  - Result: passed
  - Evidence: All declared destination files are intentionally future paths or already exist. crates/runx-parser/src/lib.rs:1-15 is still the IS_PLACEHOLDER stub; Cargo.toml:18-21 has only runx-contracts+runx-core deps and no serde_yml/serde — matches Phase 1 plan. crates/runx-contracts/src/lib.rs has no `execution` module today (lib.rs:1-24), and crates/runx-contracts/src/receipts.rs is the deferred-empty module that Phase 1 will sidestep. Authoritative TS paths verified: packages/core/src/parser/index.ts (1031 lines), packages/core/src/parser/graph.ts (598 lines), packages/core/src/parser/install.ts (29 lines). Guardrail scripts verified: scripts/check-boundaries.mjs:86 (pureCoreDomains = ['policy','state-machine']), scripts/check-rust-core-style.mjs:10 (already lists crates/runx-parser/src), scripts/check-rust-crate-graph.mjs:31 (runx-parser allowed deps = runx-contracts+runx-core), scripts/check-contract-fixture-key-order.ts:5-26 (accepts a path arg), crates/deny.toml:14-19 (Apache/MIT/Unicode-3.0 only).
- command audit
  - Grounded in: code:oss/scripts/check-contract-fixture-key-order.ts:6
  - Result: passed
  - Evidence: ac1_3 (`rg -n 'pureCoreDomains = \[[^\]]*parser' scripts/check-boundaries.mjs && pnpm boundary:check`) is executable: scripts/check-boundaries.mjs:86 currently reads `pureCoreDomains = ["policy", "state-machine"]`; adding `parser` matches the regex and the boundary check would still pass because parser today imports `yaml`, `../receipts/index.js`, `../util/types.js`, `./graph.js` — none are in forbiddenPureNodeImports and pure-domain restrictions at scripts/check-boundaries.mjs:183-189 only block executor/tool-catalogs. ac1_4 mktemp test exercises scripts/check-contract-fixture-key-order.ts:6 (path arg) and lines 22-23 (stableJson(JSON.parse(actual))) — passing `{"a":1,"b":2}\n` matches and `{"b":2,"a":1}\n` fails as expected. ac2_3 fanout tokens — FanoutGroupPolicy/FanoutThresholdGate/FanoutConflictGate/threshold/conflict/quorum all exist in packages/core/src/parser/graph.ts:20-44, so a Rust port preserving the wire contract will carry them. v8 grep is a strict superset of scripts/check-rust-core-style.mjs:16-49 bans and runs only against crates/runx-parser/Cargo.toml + src/ (so tests/ legitimately using serde_json::Value are not in scope, consistent with crates/runx-core/tests/policy_fixtures.rs:121).
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:215
  - Result: passed
  - Evidence: All public parser surfaces are in scope and named: parse_skill_markdown (Phase 3), validate_skill (Phase 3), validate_skill_source/validate_skill_artifact_contract (Phase 3), extract_skill_quality_profile (Phase 3), parse_runner_manifest_yaml/validate_runner_manifest/resolve_post_run_reflect_policy (Phase 3), parse_tool_manifest_yaml/parse_tool_manifest_json/validate_tool_manifest (Phase 4), validate_skill_install (Phase 4). These match the TS exports at packages/core/src/parser/index.ts:215-498 and packages/core/src/parser/install.ts:21. The crate-graph violation that round-2 raised is resolved by moving ExecutionSemantics+GovernedDisposition+ReceiptSurfaceRef+OutcomeState+InputContextCapture into runx_contracts::execution (Phase 1) — those types exist nowhere in crates/ today (`rg ExecutionSemantics|GovernedDisposition|InputContextCapture|OutcomeState|ReceiptSurfaceRef crates/` returns no matches), so the new module is additive and keeps runx-parser within its allowed-deps set in scripts/check-rust-crate-graph.mjs:31. Out-of-scope bullets correctly exclude consumer rewrites, @runxhq/authoring, and TS contracts package migration.
- acceptance timing audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:540
  - Result: passed
  - Evidence: Round-3 phase-ordering blocker (h3-1) is resolved: Phase 2 = graph parity, Phase 3 = skill+runner parity, Phase 4 = tool+install. validateSource at packages/core/src/parser/index.ts:540 delegates to validateGraphDocument (graph.ts:123) for type=`graph` sources, so skill parity (Phase 3) now has the dependency in place before it lands. Install in Phase 4 reuses parseSkillMarkdown+validateSkill via install.ts:22-23 — those land in Phase 3 first. Each phase has executable acceptance commands (cargo test -p runx-parser --test parser_fixtures per phase + generator --check --scope). Phase 5 ac5_4 (`! rg -n 'IS_PLACEHOLDER|Placeholder crate|No parsing behavior' crates/runx-parser`) correctly enforces removal of the current placeholder marker at crates/runx-parser/src/lib.rs:3-7.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Per-phase rollback is plausible because every phase is additive: Phase 1 adds a new runx_contracts::execution module + parser skeleton + boundary entry — revertible mechanically because no existing crate consumes execution (rg confirms zero references) and the placeholder lib.rs is restorable from VCS. Phases 2-4 add code+fixtures inside runx-parser only. Phase 5 reverts script/docs/deny.toml wiring. The rollback explicitly preserves rust-contracts-parity, runx-core, policy, and state-machine work. cargo package via path deps is precedented by archive/2026-05/rust-contracts-parity.md and confirmed by crates/Cargo.toml:14-17 declaring placeholder reservation versions. The single sharp edge is that adding `parser` to pureCoreDomains and then reverting requires careful state in scripts/check-boundaries.mjs:86 — but that is a one-line change.
- design challenge
  - Grounded in: code:oss/packages/core/src/parser/index.ts:69
  - Result: failed
  - Evidence: Two design gaps remain after round-3 fixes. (1) The Architecture Decisions section pins `parser reuses runx_core::policy::SandboxDeclaration + normalize_sandbox_declaration` (rust-parser-parity.md:174-179), but TS SkillSandbox carries fields the Rust SandboxDeclaration does not: packages/core/src/parser/index.ts:69-78 declares `approvedEscalation?: boolean` (line 76) and `raw: Record<string, unknown>` (line 77), while crates/runx-core/src/policy/types.rs:178-190 has neither. The TS validateSandbox builder (parser/index.ts:582-597) explicitly populates `raw: record` so consumers can recover the original YAML object. Phase 3 dod3 says raw text fields are byte-preserved and raw object fields are structurally equivalent — that contract is not satisfiable for the sandbox sub-tree unless the spec decides who owns the `raw` carrier (parser wrapper struct vs extending runx-core::SandboxDeclaration vs dropping `raw`). (2) Phase 1 introduces runx_contracts::execution with five new public types but the only Phase 1 acceptance is `cargo test -p runx-contracts` — no dedicated fixtures/contracts/execution/*.json parity suite analogous to fixtures/contracts/act-assignment (covered by check-rust-core-style.mjs:161-173). The parser fixture corpus implicitly exercises ExecutionSemantics via skill validated output, but the contract-types themselves have no direct ts↔rust JSON parity floor, weakening dod2 and breaking the precedent the round-2 mitigation relied on.

Issues:
- [high/blocks approval] `h4-1` design_gap - Sandbox `raw` and `approvedEscalation` have no home under the 'reuse runx_core::policy::SandboxDeclaration' decision.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:69
  - Evidence: TS SkillSandbox at packages/core/src/parser/index.ts:69-78 declares `approvedEscalation?: boolean` (line 76) and `raw: Record<string, unknown>` (line 77). validateSandbox at parser/index.ts:582-597 populates `raw: record` so downstream code can recover the original YAML object verbatim. crates/runx-core/src/policy/types.rs:178-190 (SandboxDeclaration) carries only profile, cwd_policy, env_allowlist, network, writable_paths, require_enforcement — no raw, no approvedEscalation. The spec's Architecture Decision (rust-parser-parity.md:174-179) says runx-parser reuses runx_core::policy::SandboxDeclaration directly, but that type cannot represent the parser's output. Phase 3 dod3 commits to structural parity for raw object fields, which the reuse-as-is plan cannot satisfy for the sandbox sub-tree.
  - Recommendation: Pick one and document it in the Architecture Decisions section before approval: (a) runx-parser declares a local `SkillSandbox` struct that composes `runx_core::policy::SandboxDeclaration` plus parser-only `raw` and `approved_escalation` fields (preferred — keeps policy's decision type pure and lets parser own its observable contract); (b) extend runx_core::policy::SandboxDeclaration with optional `raw`/`approved_escalation` (mixes decision shape with observable shape — not recommended); (c) explicitly drop `raw` from validated sandbox output in the Rust port and call that out as a TS-parity exception with a follow-up TS-side cleanup. Whichever option is chosen, add a Phase 3 acceptance asserting at least one fixture round-trips a non-empty sandbox `raw` object.
  - Question: Under the 'reuse runx_core::policy::SandboxDeclaration' decision, how do parser-only fields (`raw`, `approvedEscalation`) get carried in validated skill output — wrapper struct, core-type extension, or explicit TS-parity exception?
  - Recommended answer: Option (a): runx-parser owns a local SkillSandbox struct that composes runx_core::policy::SandboxDeclaration + parser-only `raw: BTreeMap<String, JsonValue>` + `approved_escalation: Option<bool>`. Keep policy's decision type pure; let parser carry observable wire fields. Add a Phase 3 fixture asserting `expected.validated.source.sandbox.raw` is preserved structurally.
  - If unanswered: Default to option (a): introduce runx-parser-local SkillSandbox wrapper carrying SandboxDeclaration + raw + approved_escalation, and add a Phase 3 acceptance fixture for sandbox.raw round-trip.
- [medium/advisory] `h4-2` spec_gap - runx_contracts::execution lands with no fixtures/contracts/execution parity suite, weakening dod2.
  - Status: open
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:161
  - Evidence: Phase 1 introduces ExecutionSemantics, GovernedDisposition, ReceiptSurfaceRef, OutcomeState, InputContextCapture in runx_contracts::execution. Phase 1's only acceptance is `cargo test -p runx-contracts` (ac1_1) — no parity fixture suite analogous to fixtures/contracts/act-assignment + fixtures/contracts/host-protocol, which scripts/check-rust-core-style.mjs:161-173 enforces include_str coverage for. The TS source at packages/core/src/receipts/index.ts:94-135 and packages/core/src/receipts/outcome-resolution.ts:9,22-27 is the only authoritative wire shape today; without a ts→rust parity fixture set, the only check that Rust serializes ExecutionSemantics the same way TS does is implicit (it appears in skill validated output, Phase 3). dod2 says runx_contracts::execution exists and is reused — but reuse without a direct parity floor invites silent drift, the exact failure mode the contracts crate precedent was set to prevent.
  - Recommendation: Add a Phase 1 acceptance step that creates `fixtures/contracts/execution/*.json` with representative ExecutionSemantics, GovernedDisposition, ReceiptSurfaceRef, OutcomeState, and InputContextCapture cases generated from the TS source, plus a `crates/runx-contracts/tests/execution_fixtures.rs` runner. Extend scripts/check-rust-core-style.mjs::checkContractFixtureCoverage to enforce include_str coverage for the new directory. This matches the pattern set by act_assignment_fixtures.rs and keeps the dod2 'reused' claim auditable.
  - Question: Should Phase 1 add a fixtures/contracts/execution parity suite + runner, or do we accept implicit coverage via parser skill fixtures in Phase 3?
  - Recommended answer: Add explicit fixtures + runner in Phase 1 — implicit coverage via skill fixtures only exercises the subset of ExecutionSemantics that skills actually use, and outcome_state/surface_refs/evidence_refs/input_context have edge cases skill fixtures will not cover.
  - If unanswered: Add fixtures/contracts/execution + execution_fixtures.rs + checkExecutionFixtureCoverage to Phase 1 and a corresponding ac1_6 acceptance command.
- [low/advisory] `h4-3` implementation_constraint - 350-line per-file cap is unaddressed for parser modules likely to exceed it (skill.rs in particular).
  - Status: open
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:116
  - Evidence: scripts/check-rust-core-style.mjs:116-124 fails any file in crates/runx-parser/src over 350 lines without a `rust-style-allow: large-file` marker. The TS parser is 1,658 lines total: packages/core/src/parser/index.ts is 1,031 lines (skill + source + runner + tool validators), graph.ts is 598 lines, install.ts is 29 lines. The spec splits Rust into skill.rs, runner.rs, graph.rs, tool.rs, install.rs, error.rs, yaml.rs — but skill+source validators alone will likely overflow 350 lines (the TS validateSource at parser/index.ts:510-568 plus validateSandbox + validateMcpServer + validateInputs + validate_skill_artifact_contract are already ~500 lines combined). Implementers will be forced to either split further (e.g. source.rs, sandbox.rs, inputs.rs as separate modules) or add `rust-style-allow: large-file` markers. The spec is silent on the split granularity.
  - Recommendation: Either (a) add a sentence in the Phase 3 Architecture/Changes section saying 'split skill.rs as needed (e.g. into skill.rs + source.rs + sandbox.rs) to stay within the 350-line cap; do not use rust-style-allow markers unless paired with a documented justification', or (b) pre-allow markers and list which files use them with rationale. Option (a) matches the runx-core layout under crates/runx-core/src/policy/ which uses many small files.
  - Question: Should the spec pre-commit to further module splits (e.g. source.rs, sandbox.rs separate from skill.rs) to stay under the 350-line cap, or pre-allow `rust-style-allow: large-file` markers with explicit rationale?
  - Recommended answer: Pre-commit to splits: Phase 3 should list skill.rs + source.rs + sandbox.rs + inputs.rs as separate modules, mirroring how crates/runx-core/src/policy/ is laid out. Avoid rust-style-allow markers unless a module is genuinely cohesive at >350 lines.
  - If unanswered: Add a Phase 3 note that skill.rs may split into skill.rs + source.rs + sandbox.rs + inputs.rs as needed to satisfy the 350-line cap without rust-style-allow markers.
- [low/advisory] `h4-4` license_audit - deny.toml license allowlist is restrictive; serde_yml transitive deps are not preflighted.
  - Status: open
  - Grounded in: code:oss/crates/deny.toml:14
  - Evidence: crates/deny.toml:14-19 allows only Apache-2.0, MIT, Unicode-3.0 with confidence-threshold 0.8. Phase 5 says it `permits the chosen YAML crate and denies known incompatible parser deps`, but does not preflight whether serde_yml = 0.0.12 and its transitive deps (libyaml-safer, unicode-ident, etc.) introduce a license outside the allowlist. cargo-deny will fail Phase 5 ac5_2 (`node scripts/check-rust-crate-graph.mjs` — note: this script does not invoke cargo-deny, but the broader CI may) and any cargo-deny gate. The spec does not name what licenses serde_yml's dep tree carries.
  - Recommendation: Add a Phase 1 prerequisite check that runs `cargo tree -p runx-parser --format '{p} {l}'` (or equivalent) and either confirms all transitive deps are Apache-2.0/MIT/Unicode-3.0 or extends crates/deny.toml's license allowlist with documented justifications. Do this in Phase 1 rather than Phase 5 so the YAML library choice can be re-litigated cheaply if a license blocker emerges.
  - Question: Has anyone confirmed serde_yml = 0.0.12 + its transitive deps stay within the Apache-2.0/MIT/Unicode-3.0 license allowlist in crates/deny.toml, or should Phase 1 add a license preflight?
  - Recommended answer: Add a Phase 1 preflight: run cargo tree and confirm transitive licenses fit the allowlist. If a transitive dep introduces e.g. BSD-3-Clause, either extend deny.toml with documented justification or reject the YAML library choice early.
  - If unanswered: Add a Phase 1 acceptance step that runs a cargo-deny license check against the parser dep tree and fails if any transitive dep is outside the allowlist.
- [low/advisory] `h4-5` test_quality - ac3_3 token grep merges skills+runner-manifests directories, so runner-manifests can be empty and still pass.
  - Status: open
  - Grounded in: spec_gap:phase3.acceptance
  - Evidence: Phase 3 ac3_3 is `for tok in markdown rawFrontmatter body raw; do rg -q "$tok" fixtures/parser/skills fixtures/parser/runner-manifests || { echo "missing $tok"; exit 1; }; done`. rg searches both directories together, so a single skill fixture containing all four tokens passes the check even if fixtures/parser/runner-manifests is empty. `markdown`, `rawFrontmatter`, and `body` are not even meaningful keys for runner-manifest fixtures (runner manifests are YAML, not markdown — RawRunnerManifestIR at parser/index.ts:100-103 only has `document` and `raw`). The grep gives the appearance of cross-category coverage but only enforces skill coverage.
  - Recommendation: Split ac3_3 into per-category token checks: one over fixtures/parser/skills requiring `markdown rawFrontmatter body raw`, and one over fixtures/parser/runner-manifests requiring `document raw`. Or replace the grep with a fixture-count acceptance: `[ "$(find fixtures/parser/runner-manifests -name '*.json' | wc -l)" -gt 0 ]` paired with the check-rust-core-style.mjs include_str coverage check.
  - Question: Should ac3_3 split into per-category grep checks (skills: markdown/rawFrontmatter/body/raw; runner-manifests: document/raw) and add a minimum-fixture-count assertion per category?
  - Recommended answer: Yes — split per-category. The current merged grep allows a silent empty runner-manifest corpus, which contradicts Phase 3's stated parity scope.
  - If unanswered: Replace ac3_3 with two separate grep checks (one per category) and a minimum-fixture-count guard for each, mirroring the precision used in ac2_3 for graphs.

### round-5

Status: failed
Started: 2026-05-18T07:56:50Z
Ended: 2026-05-18T07:56:50Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-5 closes the round-4 sandbox-wrapper and execution-contract-fixture blockers cleanly: SkillSandbox now composes runx_core::policy::SandboxDeclaration + parser-only raw + approvedEscalation (Phase 3 changes + ac3_5), Phase 1 lands a direct fixtures/contracts/execution parity suite (ac1_1b/ac1_1c) with cargo-deny preflight (ac1_6), ac3_3/ac3_4 split skill from runner-manifest grep coverage, and the architecture decisions section pins YAML 1.2 scalar subset, fixture envelope (input.markdown + expected.raw), reuse of check-contract-fixture-key-order.ts, and module-split discipline. Two execution issues should still be resolved before approval. (1) Phase 1's new runx_contracts::execution module enumerates "ExecutionSemantics, GovernedDisposition, ReceiptSurfaceRef, InputContextCapture, OutcomeState, and minimal outcome value shapes" but ReceiptOutcome — the concrete type carried by ExecutionSemantics.outcome (outcome-resolution.ts:22-27) — is not named, risking implementer omission or naming drift; the parity claim in dod2 needs an explicit type list and a per-type fixture, not "minimal outcome value shapes". (2) approvedEscalation parity is ambiguous: TS validateSandbox at parser/index.ts:582-597 never populates SkillSandbox.approvedEscalation from the YAML record (the field on the interface line 76 is always undefined in current TS output), but the Rust SkillSandbox wrapper "preserves those fields in serde output" per Architecture Decisions — implementers may legitimately read approvedEscalation from the YAML record to populate it, which would diverge from TS until a separate TS-parity spec lands. The remaining items are advisory but worth pinning before approval: structural parity for `raw: Record<string, unknown>` needs JSON-number canonicalization across yaml (TS) and serde_yml (Rust); the regex crate is implicitly required for parseSkillMarkdown's frontmatter split but not listed in Phase 1's dependency additions; and the Phase 1 generator skeleton commits to a fixture envelope but the README spec (input.markdown + expected.raw) does not yet pin which contracts::JsonValue carrier is used for the raw object subtree (vs a new type), creating an opportunity for divergence with the host-protocol fixture precedent.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-parser/src/lib.rs:1
  - Result: passed
  - Evidence: Verified all declared destination files. crates/runx-parser/src/lib.rs:1-15 is still the IS_PLACEHOLDER stub; Cargo.toml:18-21 has only runx-contracts+runx-core deps and no serde_yml — matches Phase 1 plan. crates/runx-contracts/src/lib.rs:1-23 exports act_assignment/host_protocol/json/receipts/registry/tools modules but has no `execution` module today; crates/runx-contracts/src/receipts.rs:1-5 is the deferred-empty receipt module Phase 1 sidesteps by routing ExecutionSemantics through a new runx_contracts::execution. crates/runx-core/src/policy/types.rs:178-202 defines SandboxDeclaration (writable_paths: Option<Vec<String>>) and RequiredSandboxDeclaration, confirming the wrapper-struct design fits. Authoritative TS paths verified: packages/core/src/parser/index.ts (1031 lines incl. validateSandbox at 582-597, parseSkillMarkdown at 215-237, validateExecutionSemantics at 627-641, validateOutcome at 643-654), packages/core/src/parser/graph.ts (598 lines incl. FanoutGroupPolicy/FanoutThresholdGate/FanoutConflictGate at 20-46), packages/core/src/parser/install.ts (29 lines, parseSkillMarkdown+validateSkill+origin echo). Guardrail scripts verified: scripts/check-boundaries.mjs:86 (pureCoreDomains = ['policy','state-machine']), scripts/check-rust-core-style.mjs:6-14 (rustRoots includes crates/runx-parser/src) and :161-173 (checkContractFixtureCoverage handles act-assignment + host-protocol; Phase 1 extends to execution), scripts/check-rust-crate-graph.mjs:31 (runx-parser allowed deps = runx-contracts+runx-core), scripts/check-contract-fixture-key-order.ts:5-26 (accepts a path arg, recurses, compares stableJson), crates/deny.toml:14-19 (allowlist Apache/MIT/Unicode-3.0).
- command audit
  - Grounded in: code:oss/scripts/check-contract-fixture-key-order.ts:6
  - Result: passed
  - Evidence: ac1_3 (rg -n 'pureCoreDomains = \[[^\]]*parser' scripts/check-boundaries.mjs && pnpm boundary:check) is executable: scripts/check-boundaries.mjs:86 is currently a single-line `pureCoreDomains = ['policy', 'state-machine']` and would match the regex with `'parser'` appended; boundary check at scripts/check-boundaries.mjs:183-189 only blocks forbidden node IO + executor/tool-catalogs imports for pure domains. The TS parser imports `yaml`, `../receipts/index.js`, `../util/types.js`, `./graph.js` — none of these are blocked, so ac1_3 passes after the edit. ac1_4 mktemp test exercises scripts/check-contract-fixture-key-order.ts:6 (path arg) and :22 (stableJson(JSON.parse(actual))) — passing `{"a":1,"b":2}\n` matches and `{"b":2,"a":1}\n` fails as expected. ac1_1b (`pnpm exec tsx scripts/generate-rust-contract-fixtures.ts --check --scope execution`) is consistent with scripts/generate-rust-contract-fixtures.ts:58-66 supporting `--scope` parsing and a closed set; Phase 1 must extend the closed set to accept `execution`. ac1_6 cargo-deny preflight requires Cargo.lock generation; precedent is rust-policy-parity. v8 grep is a strict superset of scripts/check-rust-core-style.mjs:16-49 bans and runs only against crates/runx-parser/Cargo.toml + src/, so tests/ legitimately using serde_json::Value are not in scope (consistent with crates/runx-core/tests/policy_fixtures.rs). ac5_4 grep tokens `IS_PLACEHOLDER|Placeholder crate|No parsing behavior` match crates/runx-parser/src/lib.rs:7 (IS_PLACEHOLDER) and :3 (No parsing behavior) literally; "Placeholder crate" is harmless even though not literally present (the assertion is `!` so absence is success).
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/receipts/outcome-resolution.ts:22
  - Result: failed
  - Evidence: Phase 1 changes list enumerates `ExecutionSemantics, GovernedDisposition, ReceiptSurfaceRef, InputContextCapture, OutcomeState, and minimal outcome value shapes used by parser output`. But ExecutionSemantics.outcome is typed `ReceiptOutcome` (outcome-resolution.ts:22-27: {code?, summary?, observed_at?, data?: Record<string, unknown>}), and validateOutcome at parser/index.ts:643-654 returns the same shape from skill validation. `Minimal outcome value shapes` is ambiguous — implementer may construct an ad-hoc type, omit fields, or rename. dod2 commits to `runx_contracts::execution exists, has direct fixtures/contracts/execution parity coverage, and is reused by runx-parser`. Reuse without an explicit list of types invites silent drift, the exact failure round-2 mitigated by moving these into the contracts crate. The fixture suite description in Phase 1 is also category-only (`ExecutionSemantics, GovernedDisposition, receipt surface refs, input-context capture, and outcome-state shapes`) without a `ReceiptOutcome` fixture row, so ac1_1b/ac1_1c can pass with implicit/partial coverage. All other parser surfaces (parse_tool_manifest_yaml/json/validate_tool_manifest/validate_skill_install/parse_runner_manifest_yaml/validate_runner_manifest/resolve_post_run_reflect_policy) are correctly enumerated across Phases 3-4 — only the receipts-type enumeration is incomplete.
- acceptance timing audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:582
  - Result: failed
  - Evidence: Phase ordering itself is correct (Phase 2 graph → Phase 3 skill+runner → Phase 4 tool+install) so validateSource at parser/index.ts:540 has its dependency satisfied. Each phase has concrete cargo test commands and the generator's --check --scope flag is used consistently. However, ac3_5 grep requires `approvedEscalation approved_escalation raw SandboxDeclaration normalize_sandbox_declaration` in crates/runx-parser/src — the tokens cover the wrapper struct surface but do NOT exercise whether the field is *populated*. TS validateSandbox at parser/index.ts:582-597 explicitly does not set approvedEscalation: it constructs `{ profile, cwdPolicy, envAllowlist, network, writablePaths, requireEnforcement, raw }` and never reads `record.approved_escalation` or any equivalent. The field on SkillSandbox (parser/index.ts:76) is therefore always `undefined` in current TS output. The spec's Architecture Decision says Rust SkillSandbox "preserves those fields in serde output" — but if Rust reads approvedEscalation from the sandbox YAML record, every fixture with `approved_escalation: true` will produce divergent expected.validated.source.sandbox.approvedEscalation between TS (undefined) and Rust (Some(true)). ac3_2 will fail. No phase acceptance pins the source-of-truth for approvedEscalation (read-from-YAML vs always-None-to-match-TS), and no fixture explicitly asserts the field is absent under current TS semantics.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Per-phase rollback is plausible because every phase is additive: Phase 1 adds runx_contracts::execution module + parser skeleton + generator skeleton + pureCoreDomains entry — fully revertible because no existing crate consumes execution (rg confirms zero references in crates/), the placeholder lib.rs is restorable from VCS, and the single-line pureCoreDomains addition is mechanical. Phases 2-4 are additive code+fixtures inside runx-parser only. Phase 5 reverts script/docs/deny.toml wiring. Rollback explicitly preserves rust-contracts-parity, runx-core, policy, and state-machine work. cargo package via path deps is precedented by archive/2026-05/rust-contracts-parity.md:291 and confirmed by crates/Cargo.toml:13-17 declaring placeholder reservation versions for runx-contracts/runx-core/runx-parser/runx-receipts. Adding parser to pureCoreDomains and reverting is a one-line change. The cargo-deny preflight ac1_6 is itself a check, not a state mutation, so it has no rollback cost.
- design challenge
  - Grounded in: code:oss/packages/core/src/parser/index.ts:53
  - Result: failed
  - Evidence: Three design ambiguities remain after round-4 fixes. (1) `raw: Record<string, unknown>` byte-preservation across YAML libraries is not pinned for the JSON-encoded structural-equivalence target. TS parseDocument(yaml).toJS() returns JS Numbers (single f64-typed); a YAML scalar `1` JSON-serializes as `1`. Rust serde_yml deserialized through typed maps (e.g., BTreeMap<String, JsonValue>) may preserve int vs float distinction in the Number variant; runx_contracts::JsonNumber at runx-contracts/src/json.rs offers both integer and float forms. Without explicit canonicalization in either the fixture writer (TS generator emits canonical JSON) or the Rust fixture loader (canonicalizes int-valued numbers), a fixture input `n: 1` may produce TS expected `"n":1` and Rust actual `"n":1.0` — even within the agreed YAML 1.2 scalar subset. (2) parseSkillMarkdown at parser/index.ts:216 uses the regex `/^---\r?\n([\s\S]*?)\r?\n---\r?\n?([\s\S]*)$/` to split frontmatter and body. Rust port requires either the `regex` crate or a hand-rolled byte split. Phase 1 Cargo.toml additions are `serde, serde_yml = "0.0.12", thiserror` — `regex` is implicit. If implementers add it, cargo-deny license/bans must accept its transitive deps; if they hand-roll, the spec should commit to that to prevent unsupervised dep growth. (3) The `raw` carrier type for skill/runner/tool/sandbox/etc. is not pinned: runx_contracts::JsonValue is the existing canonical carrier (used by host-protocol fixtures and runx-core::policy paths) but the spec does not require it. A new local Value type would proliferate parser-shaped JSON carriers and re-introduce the same drift host-protocol parity already solved.

Issues:
- [high/blocks approval] `h5-1` scope - Phase 1 enumerates contract types but omits ReceiptOutcome, which ExecutionSemantics.outcome references.
  - Status: open
  - Grounded in: code:oss/packages/core/src/receipts/outcome-resolution.ts:22
  - Evidence: Phase 1 changes list reads `ExecutionSemantics, GovernedDisposition, ReceiptSurfaceRef, InputContextCapture, OutcomeState, and minimal outcome value shapes used by parser output`. ExecutionSemantics at packages/core/src/receipts/index.ts:128-135 declares `outcome?: ReceiptOutcome` which is defined at packages/core/src/receipts/outcome-resolution.ts:22-27 as `{ code?, summary?, observed_at?, data?: Record<string, unknown> }`. parser/index.ts:643-654 validates this exact shape. The phrase `minimal outcome value shapes` does not name ReceiptOutcome, so an implementer may build an inline anonymous struct, omit fields, or rename it. The Phase 1 fixture description likewise lists category labels (`ExecutionSemantics, GovernedDisposition, receipt surface refs, input-context capture, and outcome-state shapes`) without a `ReceiptOutcome` fixture entry. dod2 commits to `direct fixtures/contracts/execution parity coverage` for `runx_contracts::execution` — implicit coverage via skill fixtures only exercises the subset of outcome shapes a skill actually carries.
  - Recommendation: Update Phase 1 changes for `crates/runx-contracts/src/execution.rs` to enumerate exactly: `ExecutionSemantics`, `GovernedDisposition`, `ReceiptSurfaceRef`, `InputContextCapture`, `OutcomeState`, and `ReceiptOutcome`. Add a Phase 1 acceptance assertion that fixtures/contracts/execution contains at least one fixture per type, mirroring scripts/check-rust-core-style.mjs:161-173 coverage gates. Recommended: add `for tok in ExecutionSemantics GovernedDisposition ReceiptSurfaceRef InputContextCapture OutcomeState ReceiptOutcome; do rg -q "$tok" fixtures/contracts/execution || { echo missing; exit 1; }; done` to a new ac1_1d.
  - Question: Should Phase 1's runx_contracts::execution module explicitly include ReceiptOutcome (matching the TS shape at outcome-resolution.ts:22-27), and should the fixture suite require one fixture per contract type?
  - Recommended answer: Yes. Add ReceiptOutcome to the enumerated type list in Phase 1 changes for crates/runx-contracts/src/execution.rs and add a per-type fixture coverage assertion. Anything less leaves the dod2 claim implicit.
  - If unanswered: Add `ReceiptOutcome` to the Phase 1 changes enumeration for crates/runx-contracts/src/execution.rs and append a per-type fixture grep gate to Phase 1 acceptance.
- [high/blocks approval] `h5-2` semantics - approvedEscalation parity is ambiguous: TS validateSandbox never populates the field, but the Rust SkillSandbox wrapper is told to preserve it.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:582
  - Evidence: packages/core/src/parser/index.ts:582-597 (validateSandbox) constructs the SkillSandbox literal with `{ profile, cwdPolicy, envAllowlist, network, writablePaths, requireEnforcement, raw }` — `approvedEscalation` is never read from `record` and never set. The interface at parser/index.ts:69-78 declares `approvedEscalation?: boolean` (line 76), so the field is structurally present but always `undefined` in current TS output. The spec's Architecture Decisions at .scafld/specs/drafts/rust-parser-parity.md:184-191 say the parser-owned Rust SkillSandbox `preserves those fields in serde output` (referring to both `raw` and `approvedEscalation`). If the Rust port reads `record.approved_escalation` to populate the field, any sandbox YAML with `approved_escalation: true` would produce divergent expected.validated.source.sandbox.approvedEscalation between TS (absent — `serde_yml` round-trip would emit nothing) and Rust (Some(true)). Phase 3 ac3_2 (cargo test --test parser_fixtures) would fail or — worse — pass with regenerated fixtures that silently diverge from TS observable output. ac3_5 grep checks the field name exists in src but does not pin populated-vs-always-None semantics.
  - Recommendation: Add a one-line clarification to Architecture Decisions: `approvedEscalation is always None in Rust SkillSandbox output for this spec, matching the current TS validateSandbox semantics (parser/index.ts:582-597 never populates it). A separate TS-parity spec will add YAML→approvedEscalation plumbing in TS first, after which Rust will populate from the YAML record.` Add a Phase 3 fixture asserting expected.validated.source.sandbox does not carry an approvedEscalation key for a sandbox input containing `approved_escalation: true`, so parity is enforced at fixture-runner time.
  - Question: Should Rust SkillSandbox.approvedEscalation be always-None to match current TS output, or should it read approved_escalation from the sandbox YAML record (and therefore diverge from TS until a TS-parity spec lands)?
  - Recommended answer: Always-None for Rust SkillSandbox in this spec — match current TS behavior exactly. The field stays on the struct for future TS-parity work, but is not populated from YAML until both languages agree.
  - If unanswered: Pin Rust SkillSandbox.approvedEscalation = None in Phase 3 architecture text; add a fixture with `approved_escalation: true` in input.sandbox and assert expected.validated.source.sandbox.approvedEscalation is absent.
- [medium/advisory] `h5-3` fixture_design - Raw object subtrees need a pinned numeric canonicalization so int-valued YAML scalars do not produce TS=`1` vs Rust=`1.0` JSON.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:53
  - Evidence: Multiple validated outputs carry a `raw: Record<string, unknown>` field at the YAML record level (parser/index.ts:53 for SkillSource, :77 for SkillSandbox, :125 for SkillRunnerDefinition, :194 for ValidatedTool; :100 for RawRunnerManifestIR.document; :105 for RawToolManifestIR.document; graph.ts:5-7 for RawGraphIR.document). TS parseDocument(...).toJS() returns native JS Numbers that JSON-serialize without trailing `.0` for integer values. Rust serde_yml deserialized through a typed JSON-value carrier (e.g., runx_contracts::JsonValue, which already has both Integer and Float variants at runx-contracts/src/json.rs) can preserve int vs float distinction; without canonicalization, a fixture input `count: 1` may serialize to `"count":1` in TS-generated expected.validated.raw and `"count":1.0` (or `"count":1` depending on variant pick) in Rust-runtime serialization. dod3 says raw object fields are structurally equivalent — but JSON encoders implement equivalence via byte comparison after canonicalization. The YAML 1.2 scalar subset constraint already excludes sexagesimal/implicit booleans but does not pin numeric canonicalization in the JSON encoder.
  - Recommendation: Pick one and pin it in Phase 1 Architecture Decisions: (a) reuse runx_contracts::JsonValue / JsonObject for every parser `raw` carrier, and require the JsonValue serializer to emit integers without trailing `.0` (the existing JsonNumber type at runx-contracts/src/json.rs already supports this — check the existing serialization layer for host-protocol fixtures); (b) declare a parser-local Value type and pin its numeric canonicalization rule; (c) tighten the YAML scalar subset to exclude all numeric scalars in raw subtrees. Option (a) reuses existing infrastructure and matches the host-protocol fixture precedent. Add a Phase 2 acceptance fixture: a graph step with `inputs: { count: 1 }` whose expected.validated.steps[0].inputs serializes to `{"count":1}` byte-identically in TS and Rust.
  - Question: Which carrier type should hold the `raw` object subtrees in Rust SkillSource/SkillSandbox/etc. — runx_contracts::JsonValue (existing canonical), a new parser-local Value, or something else?
  - Recommended answer: Reuse runx_contracts::JsonValue + JsonObject. They are the existing canonical JSON carriers used by host-protocol fixtures, and they already commit to deterministic key order + integer canonicalization. Avoid declaring a new parser-local Value type — it would re-introduce the drift host-protocol parity already solved.
  - If unanswered: Pin Phase 1 Architecture Decisions to `raw object subtrees use runx_contracts::JsonValue + JsonObject; integer-valued scalars serialize without trailing .0`; add a Phase 2 acceptance fixture asserting byte-identical numeric encoding across TS and Rust.
- [low/advisory] `h5-4` dependency - parseSkillMarkdown requires a regex; Phase 1 Cargo.toml additions do not list `regex` or commit to a hand-rolled split.
  - Status: open
  - Grounded in: code:oss/packages/core/src/parser/index.ts:216
  - Evidence: parser/index.ts:215-237 (parseSkillMarkdown) uses the regex `/^---\r?\n([\s\S]*?)\r?\n---\r?\n?([\s\S]*)$/` to split frontmatter from body. The Rust port needs either the `regex` crate or a hand-rolled byte-level scanner. Phase 1 Cargo.toml additions are `serde`, `serde_yml = "0.0.12"`, and `thiserror` (rust-parser-parity.md:316-319). If implementers add `regex` ad hoc, the cargo-deny license check at ac1_6 must accept regex's transitive deps (regex-automata, regex-syntax, memchr, aho-corasick — all MIT/Apache-2.0, so fine) — but the spec doesn't commit. If implementers hand-roll, that's also fine and avoids a new dep, but the Phase 1 changes list should say so.
  - Recommendation: Pick one in Phase 1 Architecture Decisions: (a) allow `regex` as a parser dep and add it to the Phase 1 Cargo.toml change list (and the deny.toml license preflight in ac1_6 will catch transitive license issues); (b) hand-roll a byte-level frontmatter scanner in src/skill/markdown.rs and forbid `regex` via scripts/check-rust-crate-graph.mjs. Recommended: (b) — the parse boundary is a 5-line scan of `^---\n…---\n` markers and adding regex for one regex is heavy.
  - Question: Should Phase 1 add `regex` to runx-parser Cargo.toml for parseSkillMarkdown, or commit to a hand-rolled byte-level frontmatter scanner?
  - Recommended answer: Hand-roll. The frontmatter split is a 5-line `^---\n…---\n?` byte scan; pulling in regex (and its transitive deps) is unwarranted for a pure-by-imports kernel crate.
  - If unanswered: Commit Phase 1 to a hand-rolled frontmatter scanner in `crates/runx-parser/src/skill/markdown.rs` (or `crates/runx-parser/src/markdown.rs`); add a Phase 1 acceptance grep ensuring `regex` is not in runx-parser/Cargo.toml.
- [low/advisory] `h5-5` consistency - Phase 1 adds serde/serde_yml/thiserror directly to runx-parser/Cargo.toml without considering workspace-level pinning.
  - Status: open
  - Grounded in: code:oss/crates/runx-core/Cargo.toml:20
  - Evidence: crates/runx-core/Cargo.toml:20-21 declares `serde = { version = "1.0.228", features = ["derive"] }` and `serde_json = "1.0.149"` per-crate. crates/runx-contracts/Cargo.toml:23-24 declares `serde = { version = "1.0.228", features = ["derive"] }` per-crate. crates/Cargo.toml:13-17 has only `[workspace.dependencies]` for runx-* crates, not for serde et al. Phase 1 plans to add `serde, serde_yml = "0.0.12", thiserror` to runx-parser/Cargo.toml directly — consistent with the existing per-crate pattern, but the second-third party version drift risk grows with each crate. If runx-receipts later wants the same version range, three Cargo.toml files must be touched.
  - Recommendation: Either (a) accept the existing per-crate pattern and explicitly pin runx-parser versions to match runx-core (`serde = "1.0.228"`, plus the new `serde_yml = "0.0.12"` and `thiserror`), and add a brief note in Architecture Decisions; or (b) hoist serde and serde_json to workspace.dependencies in this spec. Option (a) is the lowest-risk additive choice given that this spec is already large.
  - Question: Should Phase 1 keep the existing per-crate serde version pattern (1.0.228 to match runx-core), or hoist serde to workspace.dependencies?
  - Recommended answer: Keep per-crate. Match runx-core's `serde = "1.0.228"` and `serde_json = "1.0.149"` exactly. Hoist later in a non-parser-scoped cleanup spec.
  - If unanswered: Pin Phase 1 runx-parser/Cargo.toml to `serde = "1.0.228"` (matching runx-core) and treat workspace hoisting as out-of-scope.

### round-6

Status: failed
Started: 2026-05-18T08:08:29Z
Ended: 2026-05-18T08:08:29Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-5 blockers are cleanly resolved: Phase 1 enumerates ReceiptOutcome and Phase 3 pins approvedEscalation=None semantics with ac3_6 enforcing the fixture-level rule; runx_contracts::JsonObject/JsonValue/JsonNumber carry raw subtrees so the int-vs-float canonicalization rides on existing infrastructure (JsonNumber serialize at crates/runx-contracts/src/json.rs:44-61 collapses whole-f64 to integers); regex is pinned as a workspace dep with rationale in Architecture Decisions; and ac1_1d enforces per-type fixture coverage with cargo-deny preflight (ac1_6) before parser implementation. The remaining issue blocking approval is a real acceptance-check bug: ac2_5 runs `rg -q '"count":1' fixtures/parser/graphs`, which is a literal-substring search and matches inside `"count":1.0` (the exact failure mode the test purports to forbid). The check is grounded in spec line 435 (`"count":1`, not `"count":1.0`), but the chosen grep cannot distinguish the two — verified by re-running the same rg pattern against the spec body itself, where it matches both `"count":1` and `"count":1.0`. A correct test must terminate `1` with `[,}\s]` or use a `-P` lookahead. A secondary smaller issue is that ac1_7 strictly matches `^serde\.workspace = true`, which forbids the legitimate `serde = { workspace = true, features = [...] }` syntax that crates use when they need a feature beyond what the workspace declares; the parser likely only needs `derive` (already in runx-core's serde line) so this is unlikely to bite, but it pins implementer choice unnecessarily. Two advisory items: (a) Phase 2-4 ship parser fixtures without include_str! coverage gates in check-rust-core-style.mjs because the gate addition is deferred to Phase 5 (`scripts/check-rust-core-style.mjs (partial, shared) - Add parser fixture coverage checks`), leaving a window where stale references aren't caught; (b) the parser-side `serde_yml::Value` ban is enforced by v8 grep but not by the central disallowedPatterns table at scripts/check-rust-core-style.mjs:16-49 — Phase 5's wording 'forbidden dependency checks' doesn't explicitly name this pattern, which risks silent drift after the spec lands.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-parser/src/lib.rs:1
  - Result: passed
  - Evidence: Verified all declared destination files. crates/runx-parser/src/lib.rs:1-15 is still the IS_PLACEHOLDER stub (line 7) with message 'No parsing behavior is implemented yet' (line 3) — matches ac5_4 placeholder-marker removal check tokens. crates/runx-parser/Cargo.toml:18-21 lists only runx-contracts + runx-core workspace deps — Phase 1 adds serde/serde_yml/regex/thiserror via workspace inheritance. crates/Cargo.toml has [workspace.dependencies] (lines 13-17) but no serde/serde_yml/regex/thiserror entries yet — matches Phase 1 plan. crates/runx-contracts/src/lib.rs:1-23 has no execution module (Phase 1 adds it) and no wildcard re-exports. crates/runx-core/src/policy/types.rs:178-190 confirms SandboxDeclaration has no raw or approved_escalation fields (only profile/cwd_policy/env_allowlist/network/writable_paths/require_enforcement), justifying the parser-owned SkillSandbox wrapper. packages/core/src/parser/index.ts:9-13 (RawSkillIR), :69-78 (SkillSandbox with approvedEscalation), :582-597 (validateSandbox never populates approvedEscalation), :215-237 (parseSkillMarkdown regex split). packages/core/src/receipts/outcome-resolution.ts:22-27 confirms ReceiptOutcome shape. scripts/check-boundaries.mjs:86 currently `pureCoreDomains = ['policy', 'state-machine']`. scripts/check-rust-crate-graph.mjs:31 limits runx-parser deps to {runx-contracts, runx-core}. scripts/check-contract-fixture-key-order.ts:6-26 accepts a path arg. scripts/generate-rust-contract-fixtures.ts:58-67 has the scope validator that Phase 1 must extend with 'execution'. crates/deny.toml:14-19 allows Apache/MIT/Unicode-3.0 only.
- command audit
  - Grounded in: spec_line:461
  - Result: failed
  - Evidence: ac2_5 (`rg -q '"count":1' fixtures/parser/graphs`) is broken as a regression check. rg uses regex/literal-substring semantics; the pattern `"count":1` matches as a substring inside `"count":1.0`, `"count":11`, `"count":1.5`, etc. Verified by running `rg '"count":1\.0'` against the spec file itself (rust-parser-parity.md is the only file in the repo that currently contains `"count":1.0`, at line 435), and the same rg pattern `'"count":1'` matches that line. The acceptance therefore cannot catch the exact failure mode the spec text at line 434-435 says it must catch (`TS and Rust stable JSON both emit "count":1, not "count":1.0`). Other acceptance commands are sound: ac1_3 regex matches a pureCoreDomains line containing `parser`; ac1_4 mktemp positive/negative round-trip exercises scripts/check-contract-fixture-key-order.ts:22 stableJson semantics; ac1_1d per-type token grep works literally because ReceiptOutcome/ExecutionSemantics/etc. are unique CamelCase identifiers; ac3_6 inline node script correctly walks fixtures and asserts approvedEscalation promotion absence; ac1_6 cargo-deny preflight matches existing precedent; ac5_3 cargo-package on path-dep crates works per rust-contracts-parity precedent and crates/runx-contracts/version=0.0.1 reservation.
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:215
  - Result: passed
  - Evidence: All public parser surfaces are enumerated across phases and align with the TS exports: Phase 2 → parse_graph_yaml/validate_graph/validate_graph_document + fanout policies (graph.ts:20-46 confirms FanoutGroupPolicy/FanoutThresholdGate/FanoutConflictGate/threshold/conflict/quorum tokens for ac2_3); Phase 3 → parse_skill_markdown/validate_skill/validate_skill_source/validate_skill_artifact_contract/extract_skill_quality_profile (parser/index.ts:215-237,294,540), parse_runner_manifest_yaml/validate_runner_manifest/resolve_post_run_reflect_policy; Phase 4 → parse_tool_manifest_yaml/parse_tool_manifest_json/validate_tool_manifest (parser/index.ts:256-292) and validate_skill_install (install.ts:21-29). The Round-5 contracts enumeration is corrected: Phase 1 explicitly names ExecutionSemantics, GovernedDisposition, ReceiptSurfaceRef, InputContextCapture, OutcomeState, and ReceiptOutcome (matching outcome-resolution.ts:22-27 ReceiptOutcome shape). Out-of-scope bullets correctly exclude consumer rewrites, @runxhq/authoring, and broad TS contracts migration. crate-graph violation resolved: scripts/check-rust-crate-graph.mjs:31 allows {runx-contracts, runx-core} for runx-parser, and the new execution module sits in runx-contracts.
- acceptance timing audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:540
  - Result: passed
  - Evidence: Phase ordering is correct after Round-3 reorder: Phase 2 = graph parity → Phase 3 = skill+runner (skill validates source.type=graph via validateGraphDocument at parser/index.ts:540 → graph.ts:123; graph parity exists before skill needs it) → Phase 4 = tool+install (install.ts:22-23 reuses parseSkillMarkdown+validateSkill which land in Phase 3). approvedEscalation parity is now pinned: Architecture Decisions at spec:200-206 says approved_escalation is Option<bool> with serde omission and always None, mirroring TS validateSandbox:582-597 which never populates it. Phase 3 ac3_6 enforces the rule at fixture-runner time via an inline node script that walks fixtures/parser/skills, finds a fixture with sandbox.raw.approvedEscalation, and asserts sandbox.approvedEscalation is absent. Each phase has a concrete cargo test acceptance plus a TS-side generator `--check --scope` to keep the parity floor honest. Phase 1 cargo-deny preflight (ac1_6) runs after the deny.toml + parser Cargo.toml updates so it can validate the resolved dep tree.
- rollback/repair audit
  - Grounded in: spec_section:rollback
  - Result: passed
  - Evidence: Per-phase rollback is plausible because every phase is additive. Phase 1 adds runx_contracts::execution module + parser skeleton + fixture scaffolding + pureCoreDomains entry — fully revertible (rg confirms zero pre-existing references to ExecutionSemantics/GovernedDisposition/ReceiptSurfaceRef in crates/), the placeholder lib.rs is restorable from VCS at line 1-15, and the single-line pureCoreDomains addition at scripts/check-boundaries.mjs:86 is mechanical. Phases 2-4 are additive code+fixtures inside runx-parser only. Phase 5 reverts script/docs/deny.toml wiring. Rollback explicitly preserves rust-contracts-parity, runx-core, policy, and state-machine work. cargo package via path deps is precedented by archive/2026-05/rust-contracts-parity.md and confirmed by crates/Cargo.toml:13-17 declaring placeholder reservation versions. The single sharp edge is Phase 1 wires multiple shared touchpoints (boundaries.mjs, check-rust-core-style.mjs, deny.toml, generator) — partial rollback after partial implementation would need attention but the spec calls out the order. No state-mutating gate without revert path.
- design challenge
  - Grounded in: code:oss/crates/runx-contracts/src/json.rs:44
  - Result: passed
  - Evidence: Round-4/5 design gaps are resolved. (1) SkillSandbox carries parser-only raw + approved_escalation while composing runx_core::policy::SandboxDeclaration (Architecture Decisions spec:190-199), so policy's decision type stays pure and parser owns observable wire fields — ac3_5 + ac3_6 enforce. (2) Raw object canonicalization rides on existing runx_contracts::JsonNumber serialization at crates/runx-contracts/src/json.rs:44-61: serialize_whole_f64 collapses whole f64 values to i64 via serialize_i64, so a YAML scalar `count: 1` parsed as f64(1.0) emits as `1`, not `1.0`. The invariant at spec:96-98 pins parser to reuse runx_contracts::JsonObject/JsonValue/JsonNumber, so no parser-local Value type is introduced. (3) ExecutionSemantics.outcome's ReceiptOutcome (outcome-resolution.ts:22-27) is explicitly enumerated in Phase 1 and asserted by ac1_1d. (4) regex crate is pinned with rationale in Architecture Decisions spec:207-210 mirroring TS parseSkillMarkdown regex at parser/index.ts:216, with cargo-deny preflight in ac1_6. (5) approvedEscalation parity matches TS-always-undefined semantics (validateSandbox:582-597 never reads record.approved_escalation), pinned in Architecture Decisions and enforced by ac3_6.

Issues:
- [high/blocks approval] `h6-1` test_quality - ac2_5 grep `"count":1` is a literal substring and matches inside `"count":1.0`, defeating the exact regression it claims to forbid.
  - Status: open
  - Grounded in: spec_line:461
  - Evidence: Phase 2 spec text at lines 434-435 says: 'Include at least one raw object numeric fixture with an integer scalar (for example count: 1) so TS and Rust stable JSON both emit "count":1, not "count":1.0.' ac2_5 implements that as `rg -q '"count":1' fixtures/parser/graphs`. rg uses substring/regex semantics: the literal substring `"count":1` is present inside `"count":1.0`, `"count":11`, `"count":1.5`, etc. Verified by running rg with the same pattern against the spec file itself — the spec at line 435 contains the literal string `"count":1.0` and rg matches `"count":1` against it. Therefore the acceptance command passes even when a Rust regression emits `"count":1.0` instead of `"count":1`, which is precisely the canonicalization failure the test exists to detect. The underlying serializer at crates/runx-contracts/src/json.rs:52-54 already collapses whole f64 → i64, so the spec's defense-in-depth is well-founded — but the chosen acceptance command provides no defense at all.
  - Recommendation: Tighten ac2_5 to anchor the trailing character so `1` cannot be a prefix of `1.0` or `11`. Recommended: `rg -q '"count":1[,}\s]' fixtures/parser/graphs` (matches `"count":1,` or `"count":1}` or `"count":1 `). Alternatively, switch to `rg -P '"count":1(?=[,}\s])'` with PCRE2 lookahead. Add a paired negative check: `! rg -q '"count":1\.0' fixtures/parser/graphs` so any regression that emits a trailing `.0` fails the acceptance. Without one of these, ac2_5 is decorative.
  - Question: Should ac2_5 anchor the `1` with a terminator character (recommended: `"count":1[,}\s]`) and add a `! rg -q '"count":1\.0'` negative-assertion pair?
  - Recommended answer: Yes. Replace ac2_5 with two commands joined by &&: `rg -q '"count":1[,}\s]' fixtures/parser/graphs && ! rg -q '"count":1\.0' fixtures/parser/graphs`. The positive check confirms an integer fixture exists; the negative check confirms no fixture (anywhere in graphs) carries the broken `.0` form.
  - If unanswered: Replace ac2_5 with `rg -q '"count":1[,}\s]' fixtures/parser/graphs && ! rg -q '"count":1\.0' fixtures/parser/graphs`.
- [low/advisory] `h6-2` test_quality - ac1_7 grep `^$dep\.workspace = true` rejects the legitimate `serde = { workspace = true, features = [...] }` syntax crates use when they need a feature beyond what the workspace declares.
  - Status: open
  - Grounded in: spec_line:413
  - Evidence: ac1_7 enforces: `rg -q "^$dep\\.workspace = true" crates/runx-parser/Cargo.toml` for dep in serde/serde_yml/regex/thiserror. This matches only the bare `serde.workspace = true` form. If runx-parser ever needs a serde feature that isn't in the workspace.dependencies declaration (e.g. `rc` or `arbitrary_precision`), the canonical inheritance syntax is `serde = { workspace = true, features = ["derive", "rc"] }`. That line does not start with `serde.workspace = true` and ac1_7 fails. The parser likely only needs `derive` (already in runx-core's `serde = { version = "1.0.228", features = ["derive"] }`), so this is unlikely to bite in practice — but it pins implementer choice without justification.
  - Recommendation: Either (a) relax the grep to accept both inheritance forms: `rg -q "^$dep\\.workspace = true|^$dep = \\{[^}]*workspace = true" crates/runx-parser/Cargo.toml`, or (b) keep the strict form and document that the workspace.dependencies entry must already include every feature the parser needs (no per-crate feature extension). Option (a) is more permissive and matches Cargo idiom.
  - Question: Should ac1_7 accept both `serde.workspace = true` and `serde = { workspace = true, features = [...] }` syntaxes, or commit to bare-inheritance only?
  - Recommended answer: Accept both. Cargo treats them identically; pinning the implementer to bare-inheritance forces all feature decisions into workspace.dependencies, which is a fine default but doesn't need to be a hard constraint.
  - If unanswered: Relax ac1_7 to: `for dep in serde serde_yml regex thiserror; do rg -q "^$dep = " crates/Cargo.toml && rg -q "^$dep\\.workspace = true|^$dep = \\{[^}]*workspace = true" crates/runx-parser/Cargo.toml || { echo missing workspace dep $dep; exit 1; }; done`.
- [low/advisory] `h6-3` process_gap - Parser-fixture include_str! coverage gates land in Phase 5, leaving Phases 2-4 free to introduce stale or missing fixture references that no automated check would catch until the spec is almost complete.
  - Status: open
  - Grounded in: spec_phase:phase5
  - Evidence: Phase 5 changes list reads: `scripts/check-rust-core-style.mjs (partial, shared) - Add parser fixture coverage checks, public API style checks, forbidden dependency checks, and execution contract fixture coverage if not already added in Phase 1.` Phase 1 ac1_1c only enforces the contracts/execution coverage (already in Phase 1 changes). Phases 2-4 add fixtures under fixtures/parser/{graphs,skills,runner-manifests,tool-manifests,installs} and tests in crates/runx-parser/tests/parser_fixtures.rs + parser_rejections.rs, but checkContractFixtureCoverage at scripts/check-rust-core-style.mjs:161-173 does not yet enumerate those directories. A Phase 2 fixture could be added without being included in parser_fixtures.rs and Phase 2 acceptance ac2_2 would still pass (cargo test only runs include_str! fixtures that *are* referenced). The drift would only surface when Phase 5 adds the coverage gate and finds stale entries.
  - Recommendation: Move the parser fixture coverage extension to Phase 1's checkContractFixtureCoverage edit so all subsequent phases run under the gate. Alternatively, add a Phase 2 ac2_X and Phase 3 ac3_X explicitly running the coverage check on the per-phase fixture directories. The minimum viable fix is: Phase 2 changes add a coverage-extension for fixtures/parser/graphs to check-rust-core-style.mjs, Phase 3 extends to skills/runner-manifests, Phase 4 extends to tool-manifests/installs. Phase 5 then becomes a verify-only step.
  - Question: Should fixture-coverage gates be extended phase-by-phase (each phase wires its own directory into check-rust-core-style.mjs), or is deferring all parser-coverage to Phase 5 acceptable?
  - Recommended answer: Extend phase-by-phase. The contracts/act-assignment + contracts/host-protocol precedent (check-rust-core-style.mjs:161-173) shows the gate is cheap to add per directory, and the cost of a Phase 2 fixture being silently ignored until Phase 5 lands is paid in rework.
  - If unanswered: Add a 'extend checkContractFixtureCoverage to include fixtures/parser/<category>' line to the Phase 2/3/4 changes lists; reduce Phase 5's check-rust-core-style.mjs change to 'public API style checks + forbidden dependency checks'.
- [low/advisory] `h6-4` consistency - The `no serde_yml::Value in src/` rule is enforced only by v8 validation grep; the global disallowedPatterns table at check-rust-core-style.mjs:16-49 has no entry for serde_yml::Value, so a future Rust crate could reintroduce it without check-rust-core-style.mjs flagging.
  - Status: open
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:16
  - Evidence: scripts/check-rust-core-style.mjs:16-29 already denies `serde_json::Value` and `HashMap` globally for all rustRoots (which includes crates/runx-parser/src at line 10). There is no entry for `serde_yml::Value`. Validation v8 at spec:304-307 catches it via grep against runx-parser/Cargo.toml + src, but that command is a top-level validation, not a structural style check that other future crates would inherit. Phase 5's check-rust-core-style.mjs wording ('forbidden dependency checks') does not explicitly name this pattern. Once the spec lands, a developer porting another crate could add `serde_yml::Value` to its src/ and only the runx-parser-specific v8 check would even consider it.
  - Recommendation: In Phase 1 or Phase 5, add a new disallowedPatterns entry to scripts/check-rust-core-style.mjs:16-49: `{ pattern: /\bserde_yml::Value\b/u, reason: 'parser/YAML code must deserialize into typed structs, not yaml-Value' }`. This mirrors the existing serde_json::Value entry and applies globally. The v8 validation can stay as belt-and-braces.
  - Question: Should serde_yml::Value be added to the global disallowedPatterns table in check-rust-core-style.mjs (alongside serde_json::Value), or is v8 the only enforcement layer needed?
  - Recommended answer: Add it globally. The pattern parallels serde_json::Value exactly and the cost is one line.
  - If unanswered: Append a serde_yml::Value entry to disallowedPatterns in scripts/check-rust-core-style.mjs as part of Phase 1's existing edit.
- [low/advisory] `h6-5` test_quality - ac3_5 grep token `raw` is so common a substring that it always matches in any parser src; the check is effectively a no-op for verifying the sandbox raw carrier.
  - Status: open
  - Grounded in: spec_line:521
  - Evidence: ac3_5 is `for tok in SkillSandbox approvedEscalation approved_escalation raw SandboxDeclaration normalize_sandbox_declaration; do rg -q "$tok" crates/runx-parser/src || { echo missing sandbox token $tok; exit 1; }; done`. The token `raw` is a 3-character substring that will match in any module containing rawFrontmatter, raw_text, raw_bytes, raw_yaml, etc. Even a parser implementation that omitted the sandbox raw field entirely would pass the `raw` check because rawFrontmatter (from parseSkillMarkdown) would be present elsewhere. The other tokens (SkillSandbox, approvedEscalation, approved_escalation, SandboxDeclaration, normalize_sandbox_declaration) are unique enough to be meaningful checks.
  - Recommendation: Replace `raw` with a more specific token, e.g. `pub raw:` or `raw: JsonObject` or `raw: BTreeMap` — the actual sandbox-raw-field declaration. Recommended: `for tok in SkillSandbox approvedEscalation approved_escalation SandboxDeclaration normalize_sandbox_declaration 'pub raw:'; do ...; done`. The `pub raw:` token forces the implementation to actually declare a public raw field rather than just mentioning `raw` somewhere.
  - Question: Should ac3_5 drop the bare `raw` token and add a more targeted check like `'pub raw:'` or `raw: JsonObject`?
  - Recommended answer: Yes. The bare `raw` token contributes nothing; replace with `'pub raw:'` (or whatever the agreed sandbox raw field syntax is). Already-meaningful tokens like SkillSandbox + approvedEscalation + approved_escalation + SandboxDeclaration + normalize_sandbox_declaration carry the load.
  - If unanswered: Drop the `raw` token from ac3_5 and add `'pub raw:'` as a replacement, keeping the other five tokens.

### round-7

Status: passed
Started: 2026-05-18T08:17:48Z
Ended: 2026-05-18T08:17:48Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-7 closes the remaining round-6 blockers cleanly. ac2_5 now uses anchored character-class regex pair (`"count":1[,}\s]` positive, `"count":1\.0` negative) that correctly fails on `.0` regressions; ac1_7 accepts both `serde.workspace = true` and `serde = { workspace = true, features = [...] }` inheritance forms; per-phase fixture-coverage gates land in Phases 2/3/4 (not deferred to Phase 5); Phase 5 explicitly adds `serde_yml::Value` to the central disallowedPatterns table at scripts/check-rust-core-style.mjs:16-49. Architecture decisions are consistent: SkillSandbox composes runx_core::policy::SandboxDeclaration plus parser-only `raw`+`approved_escalation`; raw object subtrees route through `runx_contracts::JsonObject`/`JsonValue`/`JsonNumber` whose existing `serialize_whole_f64` at crates/runx-contracts/src/json.rs:52-54 collapses `1.0` to `1`; ReceiptOutcome is explicitly enumerated alongside ExecutionSemantics/GovernedDisposition/ReceiptSurfaceRef/InputContextCapture/OutcomeState; Phase 1 cargo-deny preflight (ac1_6) gates parser deps before Phase 2 implementation; ac3_6 inline node script pins approvedEscalation always-None semantics at fixture-runner time. The remaining issues are advisory: (a) ac3_5 and ac3_3 still include the bare `raw` token, which is a 3-character substring matching inside `rawFrontmatter`/`raw_text`/etc. and contributes no signal — round-6 h6-5 flagged this and the spec did not act; (b) Rust SandboxDeclaration.writable_paths is `Option<Vec<String>>` with `skip_serializing_if = "Option::is_none"` but TS validateSandbox always emits `writablePaths: []` (parser/index.ts:593) — implementers must always populate `Some(vec![])` or override serde at the wrapper to keep fixture parity; (c) ac1_1d coverage check is satisfied by token presence anywhere in {execution.rs, fixtures dir, test file} rather than requiring per-type fixture files. None block approval since fixture-runner tests would catch (b) at execution time and (a)/(c) are belt-and-braces over already-strong checks.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-parser/src/lib.rs:1
  - Result: passed
  - Evidence: All declared destination files are intentionally future paths or already exist and were verified. crates/runx-parser/src/lib.rs:1-15 is still the IS_PLACEHOLDER stub (line 7 IS_PLACEHOLDER, line 3 'No parsing behavior'). crates/runx-parser/Cargo.toml:18-21 has only runx-contracts+runx-core workspace deps with no serde/serde_yml/regex/thiserror yet — Phase 1 plan adds them via workspace inheritance. crates/Cargo.toml has [workspace.dependencies] at lines 13-17 with only runx-* path crates, no serde et al — matches Phase 1 plan. crates/runx-contracts/src/lib.rs:1-23 exports act_assignment/host_protocol/json/receipts/registry/tools but has no `execution` module today; Phase 1 adds it. crates/runx-contracts/src/json.rs:7-18 (JsonObject=BTreeMap<String,JsonValue>, JsonValue enum) and :44-61 (JsonNumber Serialize with serialize_whole_f64 collapsing whole f64 to i64) confirm the raw-carrier reuse claim. crates/runx-core/src/policy/types.rs:176-190 confirms SandboxDeclaration has `#[serde(rename_all="camelCase")]` so flattened wire shape matches TS camelCase; types.rs:187 confirms `writable_paths: Option<Vec<String>>` with skip_serializing_if — see design challenge for shape risk. Authoritative TS paths verified: packages/core/src/parser/index.ts:1-100 (RawSkillIR/SkillSource/SkillSandbox/ValidatedSkill shapes), :582-597 (validateSandbox never reads/populates approvedEscalation), :627-654 (validateExecutionSemantics+validateOutcome), packages/core/src/parser/install.ts:21-29 (validateSkillInstall = parseSkillMarkdown + validateSkill + origin echo), packages/core/src/parser/graph.ts:1-20 (RawGraphIR + FanoutSyncStrategy tokens). packages/core/src/receipts/index.ts:94-135 confirms GOVERNED_DISPOSITIONS+ReceiptSurfaceRef+InputContextCapture+ExecutionSemantics shapes; outcome-resolution.ts:22-27 is ReceiptOutcome. Guardrail scripts verified: scripts/check-boundaries.mjs:86 (pureCoreDomains=['policy','state-machine']), :183-189 (only blocks forbiddenPureNodeImports + executor/tool-catalogs cross-domain for pure domains — parser imports `yaml`, `../receipts/index.js`, `../util/types.js` are not blocked), scripts/check-rust-core-style.mjs:6-14 (rustRoots includes crates/runx-parser/src), :16-49 (disallowedPatterns has serde_json::Value + HashMap but not serde_yml::Value yet — Phase 5 adds it), :161-173 (checkContractFixtureCoverage gates act-assignment + host-protocol). scripts/check-rust-crate-graph.mjs:31 (runx-parser allowed deps = {runx-contracts, runx-core}), :39 (runx-parser required deps). scripts/check-contract-fixture-key-order.ts:6-26 (path arg + stableJson). scripts/generate-rust-contract-fixtures.ts:58-67 (scope validator that Phase 1 must extend to accept 'execution'). crates/deny.toml:14-19 (allowlist Apache-2.0/MIT/Unicode-3.0).
- command audit
  - Grounded in: spec_line:461
  - Result: passed
  - Evidence: Acceptance commands are well-formed and executable. ac2_5 round-6 blocker is fixed: `rg -q '"count":1([,}\s]|$)' fixtures/parser/graphs && ! rg -q '"count":1\.0([,}\s]|$)' fixtures/parser/graphs` — the character class `[,}\s]|$` correctly anchors the trailing terminator and rejects substring matches inside `1.0`/`1.5`/`11`, and the paired negative assertion ensures no fixture carries the broken `.0` form. ac1_7 round-6 advisory is fixed: `rg -q "^$dep(\.workspace = true| = \{[^}]*workspace = true)" crates/runx-parser/Cargo.toml` accepts both bare-inheritance and inline-feature-extension forms. ac1_3 (`rg -n 'pureCoreDomains = \[[^\]]*parser' scripts/check-boundaries.mjs && pnpm boundary:check`) is executable against scripts/check-boundaries.mjs:86 (currently `['policy','state-machine']`) — adding 'parser' satisfies the regex and parser's TS imports (`yaml`, `../receipts/index.js`, `../util/types.js`, `./graph.js`) are all allowed by the boundary rules at scripts/check-boundaries.mjs:183-189. ac1_4 mktemp test correctly exercises scripts/check-contract-fixture-key-order.ts:6,22 — script's `path.resolve(workspaceRoot, targetArg)` handles absolute mktemp paths correctly (absolute components in path.resolve override prior segments), and `{"a":1,"b":2}\n` matches stableJson output while `{"b":2,"a":1}\n` fails as expected. ac1_1b (`pnpm exec tsx scripts/generate-rust-contract-fixtures.ts --check --scope execution`) is consistent with scripts/generate-rust-contract-fixtures.ts:58-67 supporting --scope parsing; Phase 1 must extend the closed set to accept 'execution'. ac1_1d per-type token grep works literally because ExecutionSemantics/GovernedDisposition/ReceiptSurfaceRef/InputContextCapture/OutcomeState/ReceiptOutcome are unique CamelCase identifiers. ac3_6 inline node script logic is correct: walks fixture dir, finds fixture with sandbox.raw.approvedEscalation, asserts sandbox.approvedEscalation is absent — the spec's architecture decision pinned camelCase `approvedEscalation` as the raw record key, matching `Object.hasOwn(sandbox.raw, 'approvedEscalation')`. ac1_6 cargo-deny preflight matches existing precedent (rust-contracts-parity). ac5_3 cargo-package on path-dep crates works per archive/2026-05/rust-contracts-parity.md precedent. ac5_4 grep tokens `IS_PLACEHOLDER|Placeholder crate|No parsing behavior` match crates/runx-parser/src/lib.rs:7 (IS_PLACEHOLDER) and :3 ('No parsing behavior is implemented yet') literally. v8 dependency-boundary grep is a strict superset of scripts/check-rust-core-style.mjs:16-49 bans and runs only against Cargo.toml + src/ (tests/ legitimately using serde_json::Value are out of scope, consistent with crates/runx-core/tests/policy_fixtures.rs).
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:215
  - Result: passed
  - Evidence: All public parser surfaces are explicitly enumerated and aligned with TS exports. Phase 2 → parse_graph_yaml/validate_graph/validate_graph_document + FanoutGroupPolicy/FanoutThresholdGate/FanoutConflictGate (graph.ts:1-46 confirms shapes; ac2_3 tokens match). Phase 3 → parse_skill_markdown (parser/index.ts:215-237), validate_skill, validate_skill_source, validate_skill_artifact_contract, extract_skill_quality_profile, parse_runner_manifest_yaml, validate_runner_manifest, resolve_post_run_reflect_policy + parser-owned SkillSandbox composing runx_core::policy::SandboxDeclaration (Architecture Decisions spec:190-199; ac3_5 enforces tokens; ac3_6 enforces approvedEscalation always-None semantics). Phase 4 → parse_tool_manifest_yaml, parse_tool_manifest_json, validate_tool_manifest (parser/index.ts:256-292), validate_skill_install (install.ts:21-29). Round-2 crate-graph violation resolved: ExecutionSemantics+GovernedDisposition+ReceiptSurfaceRef+InputContextCapture+OutcomeState+ReceiptOutcome land in runx_contracts::execution (Phase 1) with per-type token coverage (ac1_1d) — `rg ExecutionSemantics|GovernedDisposition|InputContextCapture|OutcomeState|ReceiptSurfaceRef crates/` returns no matches today, so the new module is additive and runx-parser stays within scripts/check-rust-crate-graph.mjs:31 allowed-deps set. Out-of-scope bullets correctly defer consumer rewrites, @runxhq/authoring, broad TS contracts migration, and receipt-owned logic. The `parser` pureCoreDomains addition is bundled into Phase 1 (scripts/check-boundaries.mjs change + ac1_3). Raw object subtrees pin to runx_contracts::JsonObject/JsonValue/JsonNumber (invariants spec:96-98) — no new parser-local Value type proliferates.
- acceptance timing audit
  - Grounded in: code:oss/packages/core/src/parser/index.ts:540
  - Result: passed
  - Evidence: Phase ordering matches dependency direction: Phase 1 (contracts execution + parser skeleton + boundary entry + scalar-subset generator + cargo-deny preflight) → Phase 2 (graph parity) → Phase 3 (skill+runner; validateSource at parser/index.ts:540 delegates to validateGraphDocument at graph.ts:123, satisfied by Phase 2) → Phase 4 (tool+install; install.ts:22-23 reuses parseSkillMarkdown+validateSkill from Phase 3) → Phase 5 (governance/docs). Each phase has concrete cargo test acceptance per phase plus a TS-side `--check --scope` to keep parity floor honest. Fixture-coverage gates land per-phase rather than being deferred — Phase 2/3/4 each extend scripts/check-rust-core-style.mjs as their fixture directories land, closing round-6 h6-3. cargo-deny preflight ac1_6 runs after deny.toml + parser Cargo.toml updates in Phase 1, so the resolved dep tree is validated before Phase 2 implementation. approvedEscalation parity is pinned by Architecture Decisions spec:200-206 (always None until TS-parity spec lands) and enforced at fixture-runner time by ac3_6 inline node script. ExecutionSemantics direct parity floor is enforced by Phase 1 ac1_1b (fixtures/contracts/execution) + ac1_1c (check-rust-core-style.mjs include_str coverage) + ac1_1d (per-type token grep). The fix to ac2_5 (h6-1) eliminates the literal-substring escape that previously made the `1` vs `1.0` regression undetectable.
- rollback/repair audit
  - Grounded in: spec_section:rollback
  - Result: passed
  - Evidence: Per-phase rollback is plausible because every phase is additive. Phase 1 adds runx_contracts::execution module + parser skeleton + scalar-subset generator + pureCoreDomains entry + cargo-deny preflight — fully revertible because (a) `rg ExecutionSemantics|GovernedDisposition|ReceiptSurfaceRef|InputContextCapture|OutcomeState|ReceiptOutcome crates/` confirms zero pre-existing references, (b) crates/runx-parser/src/lib.rs:1-15 placeholder is restorable from VCS, (c) the single-line pureCoreDomains addition at scripts/check-boundaries.mjs:86 is mechanical, (d) ac1_6 cargo-deny is a check not a state mutation so has no rollback cost. Phases 2-4 are additive code+fixtures inside runx-parser only; partial revert is mechanical because fixture directories and crate src are exclusive paths. Phase 5 reverts script/docs/deny.toml wiring. Rollback explicitly preserves rust-contracts-parity, runx-core, policy, and state-machine work. cargo package via path deps is precedented by archive/2026-05/rust-contracts-parity.md and confirmed by crates/Cargo.toml:13-17 placeholder reservation versions. Phase 1's multi-touchpoint surface (boundaries.mjs + check-rust-core-style.mjs + deny.toml + generator + execution.rs + execution_fixtures.rs + fixtures/contracts/execution + runx-parser/Cargo.toml + lib.rs + error.rs + yaml.rs + fixtures/parser/README.md + docs/rust-kernel-architecture.md) is the one sharp edge but the rollback statement is per-phase and the touched files are enumerated in changes.
- design challenge
  - Grounded in: code:oss/crates/runx-core/src/policy/types.rs:187
  - Result: passed
  - Evidence: All round-2 through round-6 design gaps are resolved. (1) SkillSandbox composes runx_core::policy::SandboxDeclaration + parser-only raw + approved_escalation (Architecture Decisions spec:190-199 + ac3_5 + ac3_6); SandboxDeclaration's `#[serde(rename_all="camelCase")]` at crates/runx-core/src/policy/types.rs:177 means flattening into SkillSandbox preserves the TS camelCase wire shape for cwdPolicy/envAllowlist/network/writablePaths/requireEnforcement. (2) Raw object canonicalization rides on existing JsonNumber serialization at crates/runx-contracts/src/json.rs:52-54 where `serialize_whole_f64` collapses whole f64 to i64 via serialize_i64 — so a YAML scalar `count: 1` parsed as f64(1.0) emits as `1`, not `1.0`; the invariant at spec:96-98 pins parser to reuse runx_contracts::JsonObject/JsonValue/JsonNumber, blocking new local Value type proliferation. (3) ExecutionSemantics.outcome's ReceiptOutcome (outcome-resolution.ts:22-27) is explicitly enumerated in Phase 1 changes for execution.rs and asserted by ac1_1d. (4) regex crate is pinned with rationale in Architecture Decisions spec:207-210 mirroring TS parseSkillMarkdown regex at parser/index.ts:216, with cargo-deny preflight in ac1_6. (5) approvedEscalation parity matches TS-always-undefined semantics (validateSandbox:582-597 never reads record.approved_escalation), pinned in Architecture Decisions and enforced by ac3_6. (6) YAML scalar subset constraint is policed by Phase 1 generator's `--check-scalar-subset` flag (ac1_5). One residual implementation-discipline concern (see issues): writable_paths shape mismatch between TS `writablePaths: readonly string[]` (always emitted, defaults to []) and reused Rust SandboxDeclaration.writable_paths: Option<Vec<String>> with skip_serializing_if (crates/runx-core/src/policy/types.rs:187) — implementers must always populate Some(vec![]) to match TS wire shape, otherwise fixture parity fails at Phase 3 ac3_2. Fixture-runner tests will catch this at execution time, so it does not block approval.

Issues:
- [low/advisory] `harden-1` test_quality - ac3_5 still includes the bare `raw` token, which matches inside `rawFrontmatter`/`raw_text` and contributes no signal (round-6 h6-5 carryover).
  - Status: open
  - Grounded in: spec_line:521
  - Evidence: ac3_5 is `for tok in SkillSandbox approvedEscalation approved_escalation raw SandboxDeclaration normalize_sandbox_declaration; do rg -q "$tok" crates/runx-parser/src ...`. The 3-character substring `raw` matches inside rawFrontmatter (from parseSkillMarkdown at parser/index.ts:215-237), raw_text, raw_bytes, raw_yaml — every parser src/ file with a rawFrontmatter reference passes the `raw` check even if no sandbox `raw` field exists. The other tokens (SkillSandbox/approvedEscalation/approved_escalation/SandboxDeclaration/normalize_sandbox_declaration) are unique enough to carry the load. Round-6 h6-5 flagged this exact issue and the spec did not act on it. ac3_3 has the identical problem: `for tok in markdown rawFrontmatter body raw; do rg -q "$tok" fixtures/parser/skills ...` — `raw` matches inside rawFrontmatter.
  - Recommendation: Replace the bare `raw` token with `'pub raw:'` (the actual sandbox field declaration) in ac3_5, and replace it with `'"raw":'` (the JSON envelope key) in ac3_3. Both more specific tokens force real coverage rather than incidental rawFrontmatter matches. Keep the other tokens unchanged.
  - Question: Should ac3_5 drop `raw` in favor of `'pub raw:'`, and should ac3_3 drop `raw` in favor of `'"raw":'` to avoid rawFrontmatter substring noise?
  - Recommended answer: Yes. Replace `raw` in ac3_5 with `'pub raw:'` (the Rust struct field declaration). Replace `raw` in ac3_3 with `'"raw":'` (the JSON envelope key). Five round-6 advisory issues were addressed; this one was missed.
  - If unanswered: Update ac3_5 token list to `SkillSandbox approvedEscalation approved_escalation 'pub raw:' SandboxDeclaration normalize_sandbox_declaration` and ac3_3 to `markdown rawFrontmatter body '"raw":'`.
- [low/advisory] `harden-2` design_gap - Reused runx_core::policy::SandboxDeclaration.writable_paths is Option<Vec<String>> with skip_serializing_if; TS always emits writablePaths: [], so implementers must always populate Some(vec![]) or fixture parity breaks.
  - Status: open
  - Grounded in: code:oss/crates/runx-core/src/policy/types.rs:187
  - Evidence: crates/runx-core/src/policy/types.rs:187 declares `#[serde(skip_serializing_if = "Option::is_none")] pub writable_paths: Option<Vec<String>>` on SandboxDeclaration. crates/runx-core/src/policy/sandbox.rs:31 confirms the conversion uses `unwrap_or_default()` to handle None. TS validateSandbox at packages/core/src/parser/index.ts:593 always emits `writablePaths: optionalStringArray(...) ?? []` — the field is always present in TS wire output, defaulting to `[]`. If the Rust SkillSandbox wrapper flattens SandboxDeclaration via `#[serde(flatten)]` and the parser populates `writable_paths: None` when YAML omits writable_paths, the resulting JSON would omit `writablePaths` entirely — diverging from TS `"writablePaths":[]`. Phase 3 ac3_2 (cargo test --test parser_fixtures) would catch this at fixture-runner time, but the spec does not pin the implementation discipline. The Architecture Decisions section at spec:190-199 says parser converts the policy subset into runx_core::policy::SandboxDeclaration but doesn't specify writable_paths defaulting.
  - Recommendation: Add one line to Architecture Decisions: `When converting the YAML sandbox subset into runx_core::policy::SandboxDeclaration, the parser must always populate writable_paths: Some(vec![]) when the YAML omits writable_paths, so the Rust SkillSandbox serialization matches TS validateSandbox's always-emitted writablePaths: []`. Alternatively, add a Phase 3 acceptance assertion that at least one sandbox fixture has YAML omitting writable_paths and `expected.validated.source.sandbox.writablePaths == []` (not absent).
  - Question: Should the spec pin writable_paths = Some(vec![]) default when YAML omits the field, or rely on fixture-runner discovery?
  - Recommended answer: Pin it. Add a one-line Architecture Decisions note and a Phase 3 fixture that exercises a sandbox YAML with no writable_paths key, asserting expected.validated.source.sandbox.writablePaths == []. Fixture-runner discovery is fine but documenting the rule prevents a regression chase later.
  - If unanswered: Add to Architecture Decisions: `SkillSandbox always emits writablePaths as an array (possibly empty), matching TS validateSandbox. Implementers populate the wrapped SandboxDeclaration.writable_paths = Some(vec![]) when YAML omits the key.` Add a fixture covering the empty-default case in Phase 3.
- [low/advisory] `harden-3` test_quality - ac1_1d satisfies its check if every contract type appears as a token anywhere in {execution.rs, fixtures dir, test file}; a single fixture covering all six types would pass, not per-type fixture coverage.
  - Status: open
  - Grounded in: spec_line:385
  - Evidence: ac1_1d: `for tok in ExecutionSemantics GovernedDisposition ReceiptSurfaceRef InputContextCapture OutcomeState ReceiptOutcome; do rg -q "$tok" crates/runx-contracts/src/execution.rs fixtures/contracts/execution crates/runx-contracts/tests/execution_fixtures.rs || ...; done`. rg searches all three locations together — the token check passes as long as the CamelCase identifier appears anywhere across the union. Since execution.rs will declare all six types as `pub struct`/`pub enum`, all six tokens will match in execution.rs alone, regardless of whether each type has a dedicated fixture. A test file that does `include_str!("../../../fixtures/contracts/execution/all-in-one.json")` and references all six types in a single Rust assert passes the check. The dod2 commitment to `direct fixtures/contracts/execution parity coverage` for each runx_contracts::execution type is enforced only by Phase 1's check-rust-core-style.mjs include_str coverage (ac1_1c), which checks that every fixture file is referenced, not that every type has a fixture.
  - Recommendation: Strengthen ac1_1d to require per-type fixture files: change the grep to scope each token to `fixtures/contracts/execution` only (drop execution.rs and test file from the search), so a missing fixture for OutcomeState fails the check. Alternatively, add a separate ac1_1e enforcing `find fixtures/contracts/execution -name '*<token>*.json' | grep -q .` for each token. The current breadth allows silent under-coverage.
  - Question: Should ac1_1d limit the grep scope to fixtures/contracts/execution so per-type fixture coverage is enforced, or is the looser any-of-three-locations check sufficient?
  - Recommended answer: Limit the grep scope. Change ac1_1d to `for tok in ...; do rg -q "$tok" fixtures/contracts/execution || ...; done`. Per-type fixture coverage is the dod2 commitment and the check should match it exactly.
  - If unanswered: Tighten ac1_1d to grep only fixtures/contracts/execution for each token, dropping execution.rs and the test file from the search paths.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: command:deterministic-source-check
Output: command.stdout
Summary: Deterministic command review verified the previously reported parser parity findings against current source and tests. The strict runx message, lenient validation API, raw-only approvedEscalation behavior, cached frontmatter regex, SkillMcpServer re-export, parser fixture coverage, and sandbox normalization tests are present. External Claude review found no implementation blockers; the remaining failure was the mutation guard on a stale/inconsistent review snapshot.

Attack log:
- `skill validation surface`: Check current Rust source for ValidateSkillMode and validate_skill_with_options parity with TS strict/lenient behavior -> clean (ValidateSkillMode and validate_skill_with_options are exported and used by validate_skill default strict mode.)
- `strict runx rejection`: Check current Rust source and tests for the TS-equivalent runx non-object error -> clean (The source emits runx must be an object when present. and parser_rejections.rs covers it.)
- `sandbox approved escalation`: Check current Rust source and tests keep approvedEscalation raw-only until TS emits it -> clean (SkillSandbox.approved_escalation is set to None and parser_sandbox.rs covers raw-only preservation.)
- `frontmatter parser hot path`: Check current Rust source caches the static regex instead of recompiling per parse -> clean (parse_skill_markdown uses OnceLock<Result<Regex, String>>.)
- `public parser exports`: Check lib.rs re-exports SkillMcpServer with the other skill public types -> clean (SkillMcpServer appears in the public re-export block.)
- `fixture and normalization coverage`: Check current tests cover lenient validation and approvedEscalation raw-only behavior after the fixture expansion -> clean (parser_sandbox.rs includes lenient_skill_validation_ignores_non_object_runx_metadata and approved_escalation_stays_raw_only.)

Findings:
- none

