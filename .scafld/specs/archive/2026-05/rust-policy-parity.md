---
spec_version: '2.0'
task_id: rust-policy-parity
created: '2026-05-15T12:51:06Z'
updated: '2026-05-18T03:29:44Z'
status: completed
harden_status: failed
size: large
risk_level: medium
---

# Rust policy parity

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-18T03:29:44Z
Review gate: pass

## Summary

Port the pure policy admission behavior into `crates/runx-core` and prove it
against the shared fixture set. This is conformance work only. TypeScript
policy remains authoritative, and no runtime-local, adapter, MCP, receipt, or
CLI execution path should call Rust policy yet.

This spec depends on the architecture decisions in
`oss/docs/rust-kernel-architecture.md` and inherits the same conventions as
`rust-state-machine-parity`: decision enums (not `Result`), serde rules, MSRV,
std-default posture, boundary enforcement, and the Rust implementation quality
bar from section 18.

The `node:path` import in `packages/core/src/policy/index.ts` is replaced
during the fixtures spec; this spec assumes the helper is already in tree.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-core`

Files impacted:
- `crates/runx-core/src/lib.rs`
- `crates/runx-core/src/policy.rs`
- `crates/runx-core/src/policy/local.rs`
- `crates/runx-core/src/policy/retry.rs`
- `crates/runx-core/src/policy/graph_scope.rs`
- `crates/runx-core/src/policy/connected_auth.rs`
- `crates/runx-core/src/policy/interpreter.rs`
- `crates/runx-core/src/policy/types.rs`
- `crates/runx-core/src/policy/sandbox.rs`
- `crates/runx-core/src/policy/scope.rs`
- `crates/runx-core/src/policy/posix_basename.rs`
- `crates/runx-core/tests/policy_fixtures.rs`
- `crates/runx-core/tests/policy_proptest.rs`
- `scripts/check-rust-core-style.mjs`
- `fixtures/kernel/policy/*.json`
- `packages/core/src/policy/index.ts`
- `packages/core/src/policy/sandbox.ts`
- `packages/core/src/policy/index.test.ts`
- `packages/core/src/policy/scope-narrowing.test.ts`
- `docs/rust-kernel-architecture.md`

Invariants:
- TypeScript policy remains the source of truth.
- Parity means same observable behavior, fixture JSON, serde wire shape, and
  documented contracts. It does not require copying TypeScript names, module
  boundaries, or helper structure when a Rust-idiomatic shape is cleaner.
- Rust policy code is deterministic and side-effect free.
- Rust policy code must not evaluate local filesystem state, spawn processes,
  perform network IO, read environment variables, read system time, or own
  approval prompting.
- Rust policy must preserve runx authority vocabulary rather than introducing
  aliases.
- Path-like normalization uses an in-crate `posix_basename` helper. Rust must
  not use `std::path::Path` for executable name parsing, since its behavior
  diverges from the TS `posixBasename` helper on Windows.
- Rust policy code follows the architecture doc section 18 quality bar: small
  modules, explicit enums, private helpers by default, no public
  `serde_json::Value`, no `HashMap` at a serialized boundary, no wildcard
  re-exports, no macro-generated model code, and no dynamic error erasure.

Related docs:
- `docs/rust-kernel-architecture.md` (prerequisite reading)
- `docs/trusted-kernel-package-truth.md`
- `fixtures/kernel/README.md`
- `AGENTS.md`

## Objectives

- Port local skill admission decisions.
- Port sandbox declaration normalization and sandbox admission decisions.
- Port retry admission.
- Port graph scope admission and scope narrowing semantics.
- Port the executable name normalization helper (POSIX-only).
- Mirror the scope-narrowing rule in Rust as `pub(crate) fn scope_allows`
  inside `policy::scope`. It is shared by local connected-grant matching and
  graph-scope admission, but is never re-exported from `runx_core::policy`.
  The TypeScript helper is private (`scopeAllows` inside
  `packages/core/src/policy/index.ts`); no TS extraction is needed or planned.
- Test Rust policy against shared policy fixtures.
- Keep TypeScript policy tests and fixture tests green.
- Add `proptest` strategies covering admission inputs.
- Extend the Rust style guard so `crates/runx-core/tests/policy_fixtures.rs`
  must include every `fixtures/kernel/policy/*.json` fixture and no stale
  fixture names.

## Scope

In scope from `@runxhq/core/policy` direct exports:
- `admitLocalSkill` -> `admit_local_skill`
- `admitRetryPolicy` -> `admit_retry_policy`
- `admitGraphStepScopes` -> `admit_graph_step_scopes`

In scope from `@runxhq/core/policy/sandbox`:
- `normalizeSandboxDeclaration` -> `normalize_sandbox_declaration`
- `sandboxRequiresApproval` -> `sandbox_requires_approval`
- `admitSandbox` -> `admit_sandbox`

In scope, types: `LocalAdmissionSkill`, `LocalAdmissionOptions`,
`LocalExecutionPolicy`, `LocalAdmissionGrant`, `LocalAdmissionGrantStatus`,
`RetryAdmissionRequest`, `GraphScopeGrant`,
`GraphScopeAdmissionRequest`, `AdmissionDecision`,
`GraphScopeAdmissionDecision`, `SandboxProfile`, `SandboxDeclaration`,
`RequiredSandboxDeclaration`, `SandboxAdmissionDecision`.

In scope, helpers:
- `posix_basename` Rust mirror of the TS helper.
- Internal connected-auth requirement extraction and grant matching needed by
  `admit_local_skill`: `connectedAuthRequirement`, `findMatchingGrant`,
  `grantReferenceMatches`, and `hasGrantReference`. In Rust these live in a
  private `policy::connected_auth` module as `pub(crate)` helpers used by
  local admission. They are not re-exported from `runx_core::policy`; the
  public authority-proof surface remains deferred.
- Internal scope-narrowing logic (the `scopeAllows` helper inside
  `packages/core/src/policy/index.ts`, exercised by
  `scope-narrowing.test.ts`). Ported as `pub(crate) fn scope_allows` in the
  Rust `policy::scope` module. No TS-side extraction is in scope; the TS
  helper stays private.
- Strict inline interpreter detection from `detectInlineInterpreter`,
  including `env` unwrapping, Python-like command detection, shell `-c` flag
  detection, and Windows interpreter suffix normalization.

Explicitly out of scope (deferred to a follow-up `rust-policy-authority-proof-parity`
spec):
- Re-exports from `@runxhq/core/policy/authority-proof` (5 functions, 6 types).
- Re-exports from `@runxhq/core/policy/public-work` (4 functions, 5 types).
  Internal helpers required by local admission are in scope only as private
  Rust implementation details and do not make the authority-proof API public.

Also out of scope:
- Connected auth credential resolution.
- Runtime sandbox enforcement, bubblewrap planning, process execution, receipts,
  CLI presentation, MCP, A2A, agent/provider adapters.
- Replacing TypeScript policy in runtime-local.

## Dependencies

- `rust-contracts-bootstrap` completed and approved.
- `rust-kernel-parity-fixtures` completed and approved.
- `rust-state-machine-parity` completed and approved.
- No `regex` or `indexmap` dependency is introduced for this phase. The TS
  regexes in `detectInlineInterpreter`, `unwrapEnvCommand`, and
  `isPythonLike` are ported with small hand-written ASCII helpers.
  `sandbox.ts` path safety splitting (`value.split(/[\\\/]+/)`) is also
  ported with a hand-written segment walker, not `std::path::Path` and not
  `regex`. Ordered deduplication mirrors `unique()` with a `Vec<String>`
  output backed by a function-local `BTreeSet<String>` for O(log n)
  membership tracking. The `BTreeSet` never crosses a serialized boundary; the
  public, returned, and serialized type stays `Vec<String>`, so the
  architecture rule against `HashSet`/`BTreeSet` at serialized array
  boundaries still holds. Deduplicated arrays must preserve first-seen order,
  matching the TS `unique()` helper (`Array.from(new Set(values))`). The
  membership set is used only for `contains` checks and is never iterated to
  build returned arrays or reason strings.

## Assumptions

- `serde` and `serde_json` are enough for fixture compatibility (no
  `serde_with` or similar).
- Rust represents decisions as tagged enum variants with serde rename rules
  per arch doc section 5. JSON discriminator is `status` for admission
  decisions, mirroring TS.
- Variant names that map to string union values use kebab-case via
  `#[serde(rename_all = "kebab-case")]` on the enum and per-variant overrides
  only where TS uses an irregular form.
- Existing TypeScript policy tests remain the best source of edge cases.
- The `node:path` import has already been removed from the TS source by the
  fixtures spec. The Rust `posix_basename` is a literal mirror of that
  helper.

## Touchpoints

- Local admission defaults.
- Sandbox profiles and enforcement requirements.
- Mutating retry idempotency.
- Graph scope grants and wildcard scope behavior.
- Policy docs and fixture coverage.

## Risks

- High: policy behavior is the trust model. A silent parity gap weakens the
  security story even if Rust is not yet authoritative.
- Medium: TS policy historically imported `node:path`. Even after replacement
  by `posixBasename`, Rust must mirror exactly the same behavior. Tested
  explicitly in `posix_basename` unit tests.
- Medium: serde rename strategies for enums can produce subtle JSON shape
  differences from TS. Mitigated by the in-module serde round-trip tests in
  `crates/runx-core/src/policy/types.rs` required by Phase 1 (`ac1_5`).
- Low: policy fixtures may be too broad if every helper becomes public.
- Medium: fixture coverage drift would silently weaken parity. Mitigated by
  extending `scripts/check-rust-core-style.mjs` to verify every
  `fixtures/kernel/policy/*.json` fixture is referenced from
  `crates/runx-core/tests/policy_fixtures.rs`.
- Medium: `policy.rs` can become a large "god module" if admission,
  interpreter detection, scope, and sandbox logic are all in one file.
  Mitigated by keeping `policy.rs` as a root module only and splitting
  implementation into `policy/local.rs`, `policy/retry.rs`,
  `policy/graph_scope.rs`, `policy/interpreter.rs`, `policy/scope.rs`,
  `policy/sandbox.rs`, `policy/posix_basename.rs`, and `policy/types.rs`.

## Acceptance

Profile: strict

Validation:
- [x] `v1` command - Rust policy tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-33
- [x] `v2` command - Rust formatting and clippy pass.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-core --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-34
- [x] `v3` test - TypeScript policy tests pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/core/src/policy/index.test.ts packages/core/src/policy/scope-narrowing.test.ts tests/kernel-parity-fixtures.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-35
- [x] `v4` command - runx-core policy code does not use runtime APIs.
  - Command: `! rg -n 'std::fs|std::process|std::net|std::env|std::time::SystemTime|std::path::Path|tokio|reqwest|ureq|hyper|rmcp|Command::new' crates/runx-core/src crates/runx-core/tests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `v5` command - cargo-deny still passes.
  - Command: `cargo deny --manifest-path crates/Cargo.toml check bans licenses sources`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `v6` command - policy proptest run completes within the cap.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core --test policy_proptest`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38
- [x] `v7` command - Rust graph and style guards pass.
  - Command: `node scripts/check-rust-crate-graph.mjs && node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-39

## Phase 1: Policy data model

Status: completed
Dependencies: `rust-kernel-parity-fixtures` (hard: must complete first;

Objective: Complete this phase.

Changes:
- `crates/runx-core/src/lib.rs` (partial, shared) - Export the `policy` module.
- `crates/runx-core/src/policy.rs` (all, exclusive) - Module root only: declares only `types` and `posix_basename` in Phase 1. It performs named public re-exports for the in-scope types from `types`; `posix_basename` remains `pub(crate)` and is not re-exported from `runx_core::policy` because it is fixture/runtime scaffolding, not a TypeScript policy export. It must not declare Phase 2 modules before their files exist, and it must not carry admission, interpreter, scope, sandbox, or basename implementation logic.
- `crates/runx-core/src/policy/types.rs` (all, exclusive) - Policy enums, request/decision structs, serde derives per the conventions, and a small in-module serde round-trip test that exercises the tagged admission decision shape. Unknown-shaped JSON fields, including `LocalAdmissionSkill.auth`, use `Option<runx_contracts::JsonValue>` with `#[serde(skip_serializing_if = "Option::is_none")]`; no production `runx-core` type exposes `serde_json::Value`. `AdmissionDecision`, `SandboxAdmissionDecision`, and `GraphScopeAdmissionDecision` carry `reasons: Vec<String>`. Policy reason strings are formatted at call sites in `policy::local`, `policy::sandbox`, and `policy::graph_scope`, matching the existing `reason: String` precedent in `state_machine::fanout`; do not invent typed reason enums for fixture-baked interpolated strings. `LocalAdmissionGrant` and `GraphScopeGrant` use `#[serde(rename_all = "snake_case")]` because their fixture JSON fields are snake_case (`grant_id`, `scope_family`, `authority_kind`, `target_repo`, `target_locator`); all other policy structs use the default camelCase convention unless their TS wire shape proves otherwise. The in-module serde test must cover grant deserialization with snake_case targeting fields and `GraphScopeAdmissionDecision` serialization with camelCase fields such as `grantId`. `SandboxAdmissionDecision::ApprovalRequired` carries `#[serde(rename = "approval_required")]` because TS emits the snake_case string-union value `approval_required`, not kebab-case `approval-required`; the in-module serde test must cover this variant serialization. The same serde test must also cover `AdmissionDecision::Allow` and `AdmissionDecision::Deny` round-trips with `status: "allow"` / `status: "deny"` and a non-empty `reasons` array, so the most-used admission discriminator shape is verified in Phase 1. `GraphScopeAdmissionDecision.requestedScopes` and `grantedScopes` are plain `Vec<String>` fields with no `skip_serializing_if`, because fixtures require explicit empty arrays for empty requests and empty grants. Do not defensively omit empty scope arrays. `RequiredSandboxDeclaration` uses the default camelCase rename convention and `#[serde(skip_serializing_if = "Option::is_none")]` on `envAllowlist`, matching TS optional-field omission. `LocalAdmissionGrant.status` is `Option<LocalAdmissionGrantStatus>` with `#[serde(skip_serializing_if = "Option::is_none")]`. All optional fields on `LocalAdmissionGrant` (`status`, `scope_family`, `authority_kind`, `target_repo`, `target_locator`) carry `#[serde(skip_serializing_if = "Option::is_none")]`, mirroring the convention pinned for `LocalAdmissionSkill.auth`, `RequiredSandboxDeclaration.envAllowlist`, and `ConnectedAuthRequirement` optional fields. `LocalAdmissionGrantStatus` is a public enum with `Active` and `Revoked` variants under `#[serde(rename_all = "snake_case")]`; this Rust-only enum is not shared with the deferred `AuthorityProofGrant.status` until that follow-up spec decides whether sharing is useful. `LocalAdmissionSandbox` is not introduced as a Rust type. The Rust port uses a single `SandboxDeclaration` struct for both the standalone sandbox API and `LocalAdmissionSkill.source.sandbox: Option<SandboxDeclaration>`, because the two TS shapes are byte-identical and Rust should keep the cleaner data model.
- `crates/runx-core/src/policy/posix_basename.rs` (all, exclusive) - Rust mirror of the TS helper, with in-module unit tests.
- `scripts/check-rust-core-style.mjs` (partial, shared) - Add policy fixture coverage checks in the same style as the existing state-machine fixture coverage check. The guard must fail if any `fixtures/kernel/policy/*.json` file is not referenced by `crates/runx-core/tests/policy_fixtures.rs`, or if the Rust test includes a stale policy fixture filename. Phase 1 must add the checker function and register it even though the check may skip until `policy_fixtures.rs` exists in Phase 2.

Acceptance:
- [x] `ac1_1` command - policy module compiles.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-run`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac1_2` test - posix_basename Rust tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core posix_basename`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `ac1_3` test - TS policy tests still pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/core/src/policy/index.test.ts packages/core/src/policy/scope-narrowing.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8
- [x] `ac1_4` command - Rust style guard still passes.
  - Command: `rg -n 'checkPolicyFixtureCoverage|policy_fixtures.rs|fixtures/kernel/policy' scripts/check-rust-core-style.mjs && node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-9
- [x] `ac1_5` test - policy type serde round-trip tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy::types`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-10

## Phase 2: Admission parity

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `crates/runx-core/src/policy/sandbox.rs` (all, exclusive) - Pure sandbox normalization and admission. Path safety splitting is hand-written over `/` and `\` separators and does not use `std::path::Path` or `regex`.
- `crates/runx-core/src/policy/local.rs` (all, exclusive) - Pure local skill admission and strict inline-code checks. Defaults are literal parity values: `allowed_source_types = ["agent", "agent-step", "approval", "cli-tool", "mcp", "a2a", "catalog", "graph"]` and `max_timeout_seconds = 300`. `admit_local_skill` returns only `AdmissionDecision::Allow` or `AdmissionDecision::Deny`; when sandbox admission returns `ApprovalRequired`, local admission appends the sandbox reasons and returns deny, matching the TypeScript `admitLocalSkill` shape rather than exposing sandbox's `approval_required` status through the local-admission API.
- `crates/runx-core/src/policy/connected_auth.rs` (all, exclusive) - Private connected-auth requirement extraction and connected-grant matching used by `policy::local`: `connected_auth_requirement`, `find_matching_grant`, `grant_reference_matches`, and `has_grant_reference`. These helpers are `pub(crate)`, are never re-exported from `runx_core::policy`, and are internal scaffolding for local admission parity, not the public authority-proof API. `pub(crate) struct ConnectedAuthRequirement` lives in this module, uses `#[serde(rename_all = "snake_case")]` for targeting fields (`scope_family`, `authority_kind`, `target_repo`, `target_locator`), uses `#[serde(skip_serializing_if = "Option::is_none")]` on optional fields, and is never re-exported from `runx_core::policy`. `has_grant_reference` mirrors JavaScript truthiness: missing optional fields and present-but-empty strings are both falsey. Do not implement this as a naive `Option::is_some` check.
- `crates/runx-core/src/policy/retry.rs` (all, exclusive) - Pure retry admission.
- `crates/runx-core/src/policy/graph_scope.rs` (all, exclusive) - Graph-scope admission.
- `crates/runx-core/src/policy/interpreter.rs` (all, exclusive) - Inline interpreter detection used by strict local admission. No `regex` crate; port the TS regular expressions with small ASCII helper functions and unit tests for env-var assignment detection, Python-like command detection, shell `-c` flag detection, and Windows executable suffix stripping.
- `crates/runx-core/src/policy/scope.rs` (all, exclusive) - Scope-narrowing helper module. `scope_allows` is `pub(crate)` so local connected-grant matching and graph-scope admission can share the exact same rule while the helper stays outside the public API.
- `crates/runx-core/src/policy.rs` (partial, exclusive) - Widen the module root to declare and named-re-export Phase 2 modules.
- `crates/runx-core/tests/policy_fixtures.rs` (all, exclusive) - Load and assert all `fixtures/kernel/policy/*.json` cases by category.

Acceptance:
- [x] `ac2_1` command - Rust policy fixture tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core --test policy_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `ac2_2` command - TypeScript policy fixtures still pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/core/src/policy/index.test.ts packages/core/src/policy/scope-narrowing.test.ts tests/kernel-parity-fixtures.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16
- [x] `ac2_3` command - clippy is clean.
  - Command: `cargo clippy --manifest-path crates/Cargo.toml -p runx-core --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17
- [x] `ac2_4` command - Rust style guard still passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18
- [x] `ac2_5` command - policy fixture coverage is enforced.
  - Command: `node scripts/check-rust-core-style.mjs && rg -n 'checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs' scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19

## Phase 3: Property testing

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `crates/runx-core/tests/policy_proptest.rs` (all, exclusive) - Strategies for `LocalAdmissionOptions + LocalAdmissionSkill` pairs and `GraphScopeAdmissionRequest` inputs. Assertions: admitting the same request twice returns byte-identical serialized decisions; connected grant ordering is treated as semantically significant and the fixed-order first matching grant wins; graph-scope request deduplication is idempotent; repeated graph-scope admission for the same narrowed request returns byte-identical serialized decisions. Declare `#![proptest_config(ProptestConfig::with_cases(64))]` to match the existing state-machine convention and keep the 60-second cap meaningful as strategies grow.

Acceptance:
- [x] `ac3_1` command - policy proptest run completes within the cap.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core --test policy_proptest`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24

## Phase 4: Gap documentation

Status: completed
Dependencies: Phase 3

Objective: Complete this phase.

Changes:
- `docs/trusted-kernel-package-truth.md` (partial, shared) - Document that Rust policy is fixture-parity evidence only until an explicit binding/cutover spec changes runtime consumers. Add the exact status phrase `Rust policy parity status: fixture-evidence-only` so the acceptance gate proves this phase edited the doc instead of matching pre-existing text. Note that authority-proof and public-work re-exports are deferred to a follow-up spec.
- `docs/rust-kernel-architecture.md` (partial, shared) - Update section 14 (Placeholder publishing strategy) so the `runx-core` placeholder status reflects that the crate now contains policy parity in addition to state-machine parity. Include the exact phrase `runx-core policy parity is not runtime-authoritative` so the gate proves this phase edited the doc.
- `fixtures/kernel/README.md` (partial, shared) - Add policy fixture notes and include the exact phrase `Rust policy fixtures are policy parity evidence`. Also include a sentence containing `authority-proof and public-work re-exports are deferred to a follow-up spec` so `ac4_2` proves the fixture README carries the deferral note.

Acceptance:
- [x] `ac4_1` command - docs state Rust policy is not runtime-authoritative.
  - Command: `rg -n 'Rust policy parity status: fixture-evidence-only' docs/trusted-kernel-package-truth.md && rg -n 'Rust policy fixtures are policy parity evidence' fixtures/kernel/README.md && rg -n 'runx-core policy parity is not runtime-authoritative' docs/rust-kernel-architecture.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29
- [x] `ac4_2` command - deferred authority-proof scope is named in docs.
  - Command: `rg -n 'authority-proof and public-work.*follow-up|authority-proof.*public-work.*deferred' docs/trusted-kernel-package-truth.md && rg -n 'authority-proof and public-work.*follow-up|authority-proof.*public-work.*deferred' fixtures/kernel/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30

## Rollback

Strategy: per_phase

Commands:
- Remove `crates/runx-core/src/policy.rs`, `crates/runx-core/src/policy/`,
  `crates/runx-core/tests/policy_fixtures.rs`,
  `crates/runx-core/tests/policy_proptest.rs`.
- Revert the `pub mod policy;` export in `crates/runx-core/src/lib.rs`.
- Revert the policy fixture coverage additions in
  `scripts/check-rust-core-style.mjs`.
- Revert policy parity docs.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Rust policy parity port is faithful to the TypeScript oracle. Public types match TS shape, serde renames (snake_case grant fields, camelCase decisions, kebab-case status/profile/cwd enums, explicit approval_required rename) line up with fixture JSON. `admit_local_skill`, `admit_sandbox`, `admit_retry_policy`, `admit_graph_step_scopes`, and `normalize_sandbox_declaration` mirror TS branch-by-branch including reason strings, default lists (8 source types, 300s timeout, readonly/skill-directory sandbox defaults). `scope_allows` correctly enforces `prefix:*` only, `posix_basename` handles mixed/trailing separators, and `detect_inline_interpreter` covers env-unwrap, Windows suffix stripping, Python version matching, shell `-cX` lowercase-c flag detection, and cmd `/c`-lowercased trigger. Connected-auth requirement extraction matches TS truthiness for null/false/non-record auth and filters non-string scopes. Module visibility matches the spec: `scope_allows`, `posix_basename`, and the `connected_auth` helpers stay `pub(crate)`; authority-proof and public-work re-exports remain deferred. Style guard adds the new policy fixture coverage hook, all 22 policy fixtures are included, and `serde_json::Value` is confined to tests. Docs in `docs/trusted-kernel-package-truth.md`, `docs/rust-kernel-architecture.md`, and `fixtures/kernel/README.md` contain the exact required deferral phrases. Proptest config matches the state-machine convention with 64 cases and covers determinism for local, retry, and graph admission plus deduplication idempotence; connected-auth grant ordering is asserted where the selector is visible (in `connected_auth::tests`).

Attack log:
- `Fixture parity (22 policy fixtures)`: Read every fixture and trace its expected output through the Rust admit/normalize/requires_approval pipeline to confirm reason strings, ordering, and decision shape match TS byte-for-byte. -> clean (Verified local-admission allow/deny, sandbox normalize/admit, retry admit, graph-scope admit. Reason templates and snake/camel/kebab casing align with fixture JSON.)
- `Serde shape vs TS wire shape`: Spot-check every type in policy/types.rs for rename_all/skip_serializing_if/tag attributes against TS interfaces and the spec's pinned conventions (e.g., approval_required snake_case status, grant snake_case fields, GraphScopeAdmissionDecision empty arrays preserved). -> clean (All four required in-module serde tests (Allow/Deny round-trip, snake_case grant deserialization, camelCase graph decision with empty arrays, approval_required) are present and accurate.)
- `scope_allows / scopeAllows behavior`: Compare TS scopeAllows to Rust scope_allows on edge cases: `*` grant, `prefix:*` grant, `prefix*` (missing colon), `:*` degenerate, request widening like `repo:*` against exact grant, prefix substring like `repository:read` vs `repo:*`. -> clean (Rust's strip_suffix('*').filter(ends_with ':') is semantically identical to TS endsWith(':*'); prefix-substring denial and request-side wildcard denial both honored.)
- `Inline interpreter detection`: Walk every TS branch (node/nodejs/bun, deno eval, python-like, ruby/perl/lua, php, sh family `-cX` flag, pwsh, cmd) and compare argument trimming, lowering, suffix stripping, env unwrap, and trigger return casing. -> clean (is_shell_c_flag matches the lowercase-'c' literal in TS regex; cmd trigger is lowercased after find_exact_arg to match TS pre-lowercasing of inputs; is_python_like correctly rejects multi-dot versions and non-digit suffixes.)
- `posix_basename normalization`: Test trailing slashes, mixed separators, double slashes, empty results, root-only inputs against the TS replace+rsplit approach. -> clean (trim_end_matches plus rsplit yields the same last segment as TS replace(\\,/).replace(/+$,).slice(lastIndex+1) on all probed inputs.)
- `Connected-auth requirement extraction`: Probe TS isRecord / Boolean-coercion paths: auth=undefined/null/false -> None; auth=true/number/string/array -> {provider:unknown,scopes:[]}; auth.type in env|none|local -> None; auth.provider non-string falls back to auth.type; non-string scope entries filtered out. -> clean (match arms in connected_auth_requirement and helpers reproduce TS truthiness, including treating empty-string optional fields as falsy via truthy_string.)
- `Sandbox normalization and admission`: Check defaults when sandbox is None, network defaulting from profile=='network', writable_paths/network gating for readonly and network profiles, workspace-write `..` segment detection across both `/` and `\` separators, unrestricted-local-dev approval gate, escalation skip/approval flags. -> clean (is_unsafe_writable_path's split(['/','\\']) is equivalent to TS split(/[\\/]+/) for '..' segment containment; default normalization matches the sandbox-normalize-defaults fixture.)
- `Module visibility and re-exports`: Audit policy.rs to ensure authority-proof and public-work surfaces remain deferred and that helpers (scope_allows, posix_basename, connected_auth_*) are not exposed publicly. -> clean (Only the in-scope public functions/types are re-exported; ConnectedAuthRequirement, scope_allows, unique_strings, and posix_basename are pub(crate).)
- `Style guard and quality bar`: Inspect check-rust-core-style.mjs additions and confirm the policy fixture coverage check matches the state-machine pattern; verify no panic/unwrap/serde_json::Value/HashMap usage in crates/runx-core/src/policy; confirm file/function size limits hold. -> clean (checkPolicyFixtureCoverage mirrors state-machine coverage; serde_json::Value usage is restricted to tests/; all policy source files are under 350 lines and functions under 60 lines.)
- `Docs and fixture README`: Confirm the exact required phrases land in the three doc files and that the deferral note appears in fixtures/kernel/README.md. -> clean (trusted-kernel-package-truth.md: 'Rust policy parity status: fixture-evidence-only'; rust-kernel-architecture.md: 'runx-core policy parity is not runtime-authoritative'; fixtures/kernel/README.md: 'Rust policy fixtures are policy parity evidence' and 'authority-proof and public-work re-exports are deferred to a follow-up spec'.)
- `Proptest coverage and convention`: Confirm ProptestConfig::with_cases(64) is declared and that determinism + dedup idempotence + ordered-first-match properties exist where the selector is observable. -> clean (policy_proptest.rs declares the proptest_config and covers local, retry, graph determinism plus graph dedup idempotence; the first-matching-grant property lives in connected_auth::tests where it can read the grant id.)
- `Scope drift outside declared task scope`: Compare task-scope file list against detected changes for any out-of-scope edits. -> clean (All inspected changes (lib.rs, policy modules, fixtures references, docs, style guard, proptest, fixtures coverage) are within the declared scope. Ambient drift list was budget-omitted from the packet but no inspected files outside scope were observed.)

Findings:
- none

## Self Eval

Status: not_started
Completeness: none
Architecture fidelity: none
Spec alignment: none
Validation depth: none
Total: none
Second pass performed: none

Notes:
none

Improvements:
- none

## Deviations

- none

## Metadata

Estimated effort hours: 14
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- trusted-kernel
- policy
- parity

## Origin

Source:
- user requested phased scafld plans for Rust kernel parity.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- depends_on: rust-kernel-parity-fixtures
- related_to: rust-state-machine-parity

## Harden Rounds

### round-1

Status: failed
Started: 2026-05-17T16:45:37Z
Ended: 2026-05-17T16:45:37Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Spec is grounded in real architecture decisions and prior parity work, paths are sensible, and most invariants check out. Three substantive defects remain: (1) `scripts/check-rust-core-style.mjs` has no policy fixture coverage check, so the Phase 1/2/v7 `node scripts/check-rust-core-style.mjs` gate cannot enforce that `policy_fixtures.rs` includes every `fixtures/kernel/policy/*.json` - silent fixture drop is possible; (2) Phase 4 acceptance ripgrep gates already match strings in current repo (`docs/rust-kernel-architecture.md`, `docs/trusted-kernel-package-truth.md`, `crates/README.md`) and would pass without any new doc edits - vacuous gate; (3) several execution details are under-specified: `scope_allows` visibility crosses modules, `policy.rs` will likely exceed the 350-line style limit if it carries three admission functions plus interpreter detection, the interpreter regex port needs a deliberate regex-crate-vs-handrolled decision, and insertion-preserving deduplication (`IndexSet`) implies an undeclared `indexmap` dependency.

Checks:
- path audit
  - Grounded in: spec_gap:phases.phase1.changes
  - Result: failed
  - Evidence: Phase 1 plans 'policy.rs (all, exclusive) - Module root that re-exports submodules' while sandbox.rs and scope.rs are only created in Phase 2. policy.rs cannot 'pub mod sandbox; pub mod scope;' in Phase 1 without those files. The spec re-edits policy.rs in Phase 2 (partial), but Phase 1 must not declare modules whose files do not yet exist. The phase-1 file shape needs to be pinned (declare only types/posix_basename in Phase 1; add sandbox/scope mods in Phase 2).
- command audit
  - Grounded in: code:oss/crates/runx-core/Cargo.toml:18
  - Result: passed
  - Evidence: Existing dependencies (runx-contracts, serde, serde_json, proptest dev-dep) are consistent with `! rg 'std::fs|std::process|...|Command::new'` (v4). Architecture invariant 'no_std::path::Path' is matched by v4. proptest is std-only, fine for ac3_1.
- scope/migration audit
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:228
  - Result: failed
  - Evidence: Arch doc section 6 mandates 'Vec plus insertion-preserving deduplication (IndexSet or equivalent) for serialized arrays such as requestedScopes and grantedScopes'. The spec lists the affected fields (requestedScopes, grantedScopes) but never declares the implementation strategy or adds an `indexmap` dependency to crates/runx-core/Cargo.toml. Plain Vec-loop dedup is acceptable but should be pinned in the spec to avoid an unconsidered new dependency landing.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase3
  - Result: failed
  - Evidence: Phase 3 says 'rejection reasons are stable across equivalent inputs' without defining equivalence. With camelCase serde, optional fields with skip_serializing_if, and arrays whose deduplication preserves first-seen order, 'equivalent' could mean reordered scopes (which the API treats as distinct), or duplicated entries (which deduplicate). Without a definition, the property is untestable in a meaningful way.
- rollback/repair audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:7
  - Result: passed
  - Evidence: Rollback 'Remove crates/runx-core/src/policy.rs, crates/runx-core/src/policy/, crates/runx-core/tests/policy_*.rs' is credible because policy code starts as net-new files alongside existing state_machine. lib.rs needs the matching `pub mod policy;` line reverted, which the rollback section's 'Revert policy parity docs' phrase does not explicitly cover but is implied by 'per_phase' and standard git revert.
- design challenge
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-authority-proof-parity.md:11
  - Result: passed
  - Evidence: Deferring authority-proof and public-work to a follow-up spec is justified: those modules pull in @runxhq/contracts types (hashString, AuthorityProofContract) and would force a much wider port surface. The follow-up spec already exists as a draft and is named explicitly, satisfying the 'know-what-you-deferred' bar.

Questions:
- Should `scripts/check-rust-core-style.mjs` grow a `checkPolicyFixtureCoverage` mirror so ac1_4/ac2_4/v7 actually gate that every `fixtures/kernel/policy/*.json` is referenced by `crates/runx-core/tests/policy_fixtures.rs`?
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:141
  - Recommended answer: Yes. Add a `checkPolicyFixtureCoverage` function modeled on `checkStateMachineFixtureCoverage`, list `scripts/check-rust-core-style.mjs` as a Phase 2 change (partial, shared), and add an acceptance step that runs the script and asserts coverage. Without it, the spec's parity claim relies on author discipline, not enforcement.
  - If unanswered: Add the policy coverage check to the style script as part of Phase 2.
- How should the Phase 4 docs gates be tightened so they actually fail before the docs are updated?
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:1
  - Recommended answer: Replace ac4_1/ac4_2 with checks that target the specific files being edited (e.g., `rg 'rust-policy-parity|Rust policy is fixture-parity evidence only' docs/trusted-kernel-package-truth.md` and `rg 'policy fixture notes' fixtures/kernel/README.md`). The current patterns already match existing text in the arch doc and crates/README.md, so the gate is vacuous.
  - If unanswered: Tighten ac4_1/ac4_2 to look for new section-anchor strings unique to this spec's doc edits.
- What is the visibility of `scope_allows`, given it serves both graph-scope admission and connected-grant matching?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:182
  - Recommended answer: Make `scope_allows` `pub(crate)` inside `policy::scope`, and call it from both `policy::graph_scope` and the local-admission grant-matching module. Update the spec language ('private') accordingly.
  - If unanswered: Use `pub(crate) fn scope_allows` and document that the helper is shared across the policy module tree but never re-exported from the crate.
- How will inline-interpreter regexes (`^[A-Za-z_][A-Za-z0-9_]*=.*`, `^-[A-Za-z]*c[A-Za-z]*$`, `^python\d+(\.\d+)?$`, `\.(exe|cmd|bat)$`) be implemented in Rust - pull in the `regex` crate or hand-roll?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:259
  - Recommended answer: Pull in `regex` (already widely transitively present). Add it to `crates/runx-core/Cargo.toml`, run `cargo deny check`, and update the Phase 1 change list. Hand-rolling these patterns increases the surface area for subtle drift and obscures parity intent.
  - If unanswered: Add `regex` as a runx-core dependency in Phase 1 and re-run cargo-deny as part of ac1_4 or v5.
- What is the chosen insertion-preserving deduplication strategy for arrays like `requestedScopes` and `grantedScopes`, and does it require adding `indexmap`?
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:228
  - Recommended answer: Use a small private helper `unique_preserving_order(values: impl IntoIterator<Item = String>) -> Vec<String>` backed by a `Vec` plus an in-line `HashSet<&str>` for membership (HashSet is allowed inside function bodies as long as it never reaches a serialized boundary - verify the style script). Avoids new deps. If clarity is preferred, add `indexmap` to runx-core deps in Phase 2 and re-run cargo-deny.
  - If unanswered: Add an in-crate `unique_preserving_order` helper in `policy::scope`; no new dependency.
- Should `crates/runx-core/src/policy.rs` be split into per-admission submodules from the start to stay under the 350-line style ceiling?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:1
  - Recommended answer: Yes. Pre-plan `policy/local.rs` (admit_local_skill + connected-grant matching + interpreter detection), `policy/retry.rs` (admit_retry_policy), `policy/graph_scope.rs` (admit_graph_step_scopes), with `policy.rs` as a thin module root that re-exports the public surface and `types.rs`/`scope.rs`/`sandbox.rs`/`posix_basename.rs` siblings. Update Phase 1 and Phase 2 change lists accordingly.
  - If unanswered: Adopt the per-admission submodule layout to avoid relying on `rust-style-allow: large-file` escapes.
- Define 'equivalent inputs' for the Phase 3 stability property - what transformations are expected to leave rejection reasons unchanged?
  - Grounded in: spec_gap:phases.phase3
  - Recommended answer: Pin two properties explicitly: (a) idempotence - admitting the same request twice returns byte-identical decisions; (b) order-insensitivity of `connectedGrants` ordering for the chosen grant (the first matching grant wins, so document that order matters when multiple grants match). Drop the vague 'equivalent inputs' language.
  - If unanswered: Rewrite the Phase 3 description to two named properties: idempotence and order-insensitive grant lookup.
- Does Phase 1's `cargo test -p runx-core policy --no-fail-fast --no-run` need a follow-up that actually executes any Phase 1 test besides `posix_basename`?
  - Grounded in: spec_gap:phases.phase1.acceptance.ac1_1
  - Recommended answer: Add an in-module `#[cfg(test)]` serde round-trip test for the policy types in `policy::types` and execute it via `cargo test -p runx-core policy::types`. This catches enum rename collisions and tag-field mistakes in Phase 1 instead of waiting for Phase 2's fixture run.
  - If unanswered: Add a Phase 1 acceptance step `cargo test -p runx-core policy::types` once a small serde round-trip test exists in types.rs.

Design objections:
- `objection-1` high - Policy fixture coverage is not enforced by the style script the spec leans on.
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:141
  - Evidence: `check-rust-core-style.mjs` contains `checkStateMachineFixtureCoverage` (lines 141-167) which only walks the state-machine fixture directory and the state_machine_fixtures.rs test file. There is no analogous `checkPolicyFixtureCoverage`. ac1_4, ac2_4, and v7 all cite this script as the coverage guard, so a new policy fixture can be added without a Rust test and every gate still passes.
  - Recommendation: Add `checkPolicyFixtureCoverage` to `scripts/check-rust-core-style.mjs` (mirroring the state-machine logic over `fixtures/kernel/policy/` and `crates/runx-core/tests/policy_fixtures.rs`), list the script as a Phase 2 change (partial, shared), and add a dedicated acceptance step that runs it. Without this, the spec's parity claim is not enforced.
- `objection-2` medium - Phase 4 acceptance ripgrep patterns already match existing text and gate nothing.
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:1
  - Evidence: Confirmed by Grep: `policy parity|fixture-parity|not.*runtime-authoritative|TypeScript.*source of truth` matches in `docs/rust-kernel-architecture.md`, `docs/trusted-kernel-package-truth.md`, `crates/README.md`, and several archived/draft specs. `authority-proof|public-work|follow-up.*spec` matches in `docs/rust-kernel-architecture.md`, `docs/trusted-kernel-package-truth.md`, and `docs/security-authority-proof.md`. ac4_1 and ac4_2 therefore pass before Phase 4 work begins.
  - Recommendation: Replace the broad regexes with file-and-anchor-specific patterns, e.g. `rg 'Rust policy parity is fixture-parity evidence only' docs/trusted-kernel-package-truth.md` and `rg 'policy fixture notes' fixtures/kernel/README.md`, so the gates only pass when the targeted edits land.
- `objection-3` medium - `scope_allows` cannot be 'private' as specified; it spans two admission paths.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:182
  - Evidence: TS `scopeAllows` (line 360) is called from both `findMatchingGrant` (line 323, used by `admit_local_skill`) and `admitGraphStepScopes` (line 182). The spec places `scope_allows` in `policy::scope` as a 'private' helper but the Rust port needs to call it from both `policy::local` (or wherever local admission lives) and `policy::graph_scope`.
  - Recommendation: Declare `scope_allows` as `pub(crate)` and document that fact in the spec. Either keep it in `policy::scope` and import where needed, or hoist it into a shared `policy::matching` module.
- `objection-4` medium - Inline-interpreter regex port is unaddressed; dependency posture is unknown.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:259
  - Evidence: `detectInlineInterpreter` and `unwrapEnvCommand` rely on JS regexes (env-var, shell `-c` flag combinations, `python\d+`, `\.(exe|cmd|bat)$`). Rust does not have these in std; the `regex` crate is neither listed in `crates/runx-core/Cargo.toml` nor allowed/denied in `crates/deny.toml`. The spec says nothing about whether to add the crate or hand-roll.
  - Recommendation: Decide explicitly: add `regex = "1"` to runx-core, document it in the spec Change list, and re-run `cargo deny` as part of v5; alternatively, hand-roll the limited patterns and document the equivalence in `posix_basename.rs`-style unit tests. Either choice should be pinned in the spec.
- `objection-5` medium - Single-file `policy.rs` is likely to exceed the 350-line style ceiling.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:1
  - Evidence: TS `index.ts` is 403 lines for types + three admission functions + interpreter detection + grant matching. Rust port verbosity (serde derives, exhaustive `match`, enum reasons) typically grows by ~30%. The style script flags any file over 350 lines unless it carries `rust-style-allow: large-file`.
  - Recommendation: Pre-plan a per-admission sharded layout (`policy/local.rs`, `policy/retry.rs`, `policy/graph_scope.rs`, `policy/interpreter.rs`) and keep `policy.rs` as a thin module root. Update Phase 1 and Phase 2 change lists. Avoid relying on the large-file escape hatch.
- `objection-6` low - Phase 3 'stability across equivalent inputs' is vague.
  - Grounded in: spec_gap:phases.phase3
  - Evidence: The acceptance condition references rejection reasons being 'stable across equivalent inputs' without defining equivalence. Given that input arrays deduplicate first-seen order and grant lookup picks the first match, several plausible equivalence classes (reordered scopes, reordered grants) actually produce different decisions.
  - Recommendation: Restate the property as two named claims: (a) idempotence (same input twice yields byte-identical decisions); (b) explicit grant-order semantics (`connectedGrants` order matters and the first match wins). Drop the vague phrasing.

Recommended edits:
- Phase 1: Policy data model > Changes
  - Grounded in: spec_gap:phases.phase1.changes
  - Recommendation: Restrict Phase 1 `policy.rs` to declaring only `pub mod types;` and `pub mod posix_basename;` (plus their re-exports) so it compiles before sandbox.rs/scope.rs exist. Add an explicit note that Phase 2 widens policy.rs to declare `pub mod sandbox; pub mod scope; pub mod local; pub mod retry; pub mod graph_scope;`.
- Phase 1: Policy data model > Acceptance
  - Grounded in: spec_gap:phases.phase1.acceptance.ac1_1
  - Recommendation: Add a Phase 1 acceptance step that runs an in-module serde round-trip test for the policy types (e.g., `cargo test -p runx-core policy::types`). The current compile-only gate cannot catch tag/rename mistakes until Phase 2.
- Phase 2: Admission parity > Changes
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:141
  - Recommendation: Add `scripts/check-rust-core-style.mjs` (partial, shared) to the Phase 2 Changes with a new `checkPolicyFixtureCoverage` function that mirrors `checkStateMachineFixtureCoverage` over `fixtures/kernel/policy/` and `crates/runx-core/tests/policy_fixtures.rs`.
- Phase 2: Admission parity > Changes
  - Grounded in: code:oss/packages/core/src/policy/index.ts:1
  - Recommendation: Replace the single `crates/runx-core/src/policy.rs (partial, exclusive)` change with a sharded layout: `policy/local.rs`, `policy/retry.rs`, `policy/graph_scope.rs`, and `policy/interpreter.rs`, keeping `policy.rs` as a thin module root. This keeps each file under the 350-line style ceiling without an escape hatch.
- Phase 2: Admission parity > Changes
  - Grounded in: code:oss/packages/core/src/policy/index.ts:259
  - Recommendation: Pin the inline-interpreter implementation choice: either add `regex = "1"` to `crates/runx-core/Cargo.toml` (and re-run `cargo deny` under v5) or hand-roll equivalents. Document the chosen approach in the spec.
- Phase 3: Property testing
  - Grounded in: spec_gap:phases.phase3
  - Recommendation: Replace 'rejection reasons are stable across equivalent inputs' with two named properties: idempotence (same input yields byte-identical decision) and order-aware grant lookup (first matching `connectedGrants` entry wins). State that input arrays are not reordered for the property because order is semantically significant.
- Phase 4: Gap documentation > Acceptance
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:1
  - Recommendation: Replace ac4_1 and ac4_2 with file-and-anchor-specific ripgrep patterns that only match new content added by this spec (for example, `rg 'Rust policy parity is fixture-parity evidence' docs/trusted-kernel-package-truth.md`). The current patterns match pre-existing text in arch and trust docs.
- Context > Invariants
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:228
  - Recommendation: Add an invariant clarifying the chosen insertion-preserving deduplication implementation (in-crate `Vec`-based helper using `HashSet<&str>` only inside function bodies, or `indexmap::IndexSet` if a dependency is added), so the arch doc rule is not satisfied by accident.

### round-2

Status: failed
Started: 2026-05-18T01:20:40Z
Ended: 2026-05-18T01:20:40Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-1 revisions landed cleanly: the policy-fixture coverage guard, module-sharded layout, `pub(crate)` `scope_allows`, hand-rolled ASCII interpreter parsing, `Vec`+`BTreeSet` ordered dedup, Phase-1 types serde test, and non-vacuous Phase-4 doc anchors are all reflected. Two real defects remain: (1) `admitLocalSkill` depends on `connectedAuthRequirement` from `policy/authority-proof.ts`, and that helper is currently listed only inside the deferred follow-up set - so the in-scope `local-admission-allows-connected-wildcard-grant.json` and the `auth: { type: "nango", ... }` test paths cannot be admitted by the Rust port without porting an internal `connected_auth_requirement` helper; the spec does not pin this. (2) Phase 4's `ac4_2` ripgrep also asserts the `authority-proof.*public-work.*deferred|follow-up` phrase in `fixtures/kernel/README.md`, but the Phase 4 Changes block only promises the `Rust policy fixtures are policy parity evidence` phrase for that file - the gate will fail until the deferred-follow-up phrase is also added. A few smaller items (the regex split inside `isUnsafeWritablePath`, the `auth: unknown` Rust typing, the redundant `--no-run --no-fail-fast` flags in ac1_1) are worth pinning before approval.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:7
  - Result: passed
  - Evidence: All declared paths either exist as intentional future files (crates/runx-core/src/policy.rs, src/policy/*.rs, tests/policy_fixtures.rs, tests/policy_proptest.rs - none currently present in `crates/runx-core/`) or are real prerequisites that exist now: `crates/runx-core/src/lib.rs` (has only `serde_conventions` + `state_machine` today), `scripts/check-rust-core-style.mjs` (has `checkStateMachineFixtureCoverage` at line 141, no policy analogue yet), `packages/core/src/policy/index.ts` no longer imports `node:path` (confirmed by Grep - only `./sandbox.js`, `./authority-proof.js`, `./posix-basename.js`, `../util/array.js`), `packages/core/src/policy/posix-basename.ts` exists, `fixtures/kernel/policy/*.json` exists (16 policy fixtures present), `tests/kernel-parity-fixtures.test.ts` exists, `vitest.config.ts` exists, `crates/deny.toml` exists. Phase-1 change set restricting `policy.rs` to only declaring `types` and `posix_basename` matches what the state-machine root does today (`crates/runx-core/src/state_machine.rs:1-17` uses `mod fanout;`/`mod types;`/`pub use ...`).
- command audit
  - Grounded in: code:oss/crates/runx-core/Cargo.toml:18
  - Result: failed
  - Evidence: v4 ripgrep, v5 cargo-deny, v6 proptest run, v7 style scripts, and the per-phase `cargo test` lines are runnable against the current workspace once policy modules exist. However, `ac1_1` is `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-fail-fast --no-run`: `--no-run` only compiles, and `--no-fail-fast` is a no-op with `--no-run`. Functionally the same as `cargo build --tests -p runx-core`, but the redundant flag combination misleads a reader into thinking tests run. Either drop `--no-fail-fast` or make `ac1_1` actually execute (`cargo test -p runx-core policy --no-fail-fast`). Minor but should be cleaned up before `scafld approve`.
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/policy/index.ts:136
  - Result: failed
  - Evidence: Line 136 reads `const authRequirement = options.skipConnectedAuth ? undefined : connectedAuthRequirement(skill.auth);` - `admitLocalSkill` (in scope) calls `connectedAuthRequirement` (`packages/core/src/policy/authority-proof.ts:68`). The spec defers ALL `policy/authority-proof` re-exports to a follow-up (`rust-policy-authority-proof-parity`). The fixture `fixtures/kernel/policy/local-admission-allows-connected-wildcard-grant.json` exercises this exact path with `auth: { provider: 'github', scopes: ['repo:read'], type: 'nango' }`; without an internal Rust mirror of `connected_auth_requirement` (and `find_matching_grant`/`grant_reference_matches`/`has_grant_reference`) the fixture-parity gate cannot pass. The spec's `Scope` text lists `connectedAuthRequirement` only under the deferred section, but the in-scope `admitLocalSkill` cannot be implemented without it. The split is real but the spec does not say `connected_auth_requirement` will be ported as a `pub(crate)`/private internal helper inside `policy::local` (or a sibling module) without joining the public surface. Pin this before approval.
- acceptance timing audit
  - Grounded in: code:oss/fixtures/kernel/README.md:1
  - Result: failed
  - Evidence: Phase 4 `ac4_2` is `rg -n 'authority-proof and public-work.*follow-up|authority-proof.*public-work.*deferred' docs/trusted-kernel-package-truth.md && rg -n '...' fixtures/kernel/README.md` - the AND between the two `rg` invocations means both files must contain the deferred-follow-up phrase. The Phase 4 Change description for `fixtures/kernel/README.md` says: `Add policy fixture notes and include the exact phrase "Rust policy fixtures are policy parity evidence"`. It does NOT promise the deferred-follow-up phrase for that file. After Phase 4 implementation as written, `ac4_2` will still fail because nothing in the change list requires the second phrase to be added to `fixtures/kernel/README.md`. Either remove the second `rg` (drop the fixtures/README check), or expand the Phase 4 change description to explicitly include the deferred-follow-up phrase in `fixtures/kernel/README.md`.
- rollback/repair audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:7
  - Result: passed
  - Evidence: Rollback `Remove crates/runx-core/src/policy.rs, crates/runx-core/src/policy/, crates/runx-core/tests/policy_fixtures.rs, crates/runx-core/tests/policy_proptest.rs` is credible because none of those paths exist yet. `crates/runx-core/src/lib.rs` currently has `pub mod serde_conventions; pub mod state_machine;`. The rollback explicitly calls out reverting the `pub mod policy;` export and the script edits, which is enough to restore lib.rs to its current shape. Phase 4 doc edits in `docs/trusted-kernel-package-truth.md` and `fixtures/kernel/README.md` are reversible via `git checkout`. Per-phase rollback strategy matches the additive nature of the work.
- design challenge
  - Grounded in: code:oss/.scafld/specs/archive/2026-05/rust-state-machine-parity.md
  - Result: passed
  - Evidence: Right architectural move, not bandaid: this mirrors the already-archived `rust-state-machine-parity` plan one-for-one (private helpers, fixture-parity, proptest determinism, no runtime APIs). The arch doc decision in `docs/rust-kernel-architecture.md` section 7 (POSIX-only basename) and section 18 (Rust quality bar) explicitly anticipated a policy port with these constraints. The fixture set in `fixtures/kernel/policy/` already exists from the fixtures spec. Deferring authority-proof and public-work to `rust-policy-authority-proof-parity` keeps the surface bounded; that follow-up spec is real (drafts/rust-policy-authority-proof-parity.md). The future bloat risk is contained because Rust policy is not yet runtime-authoritative - Phase 4 documents that explicitly. Not over-engineered.

Questions:
- Where will the Rust port of `connectedAuthRequirement` live, given that `admit_local_skill` depends on it but the authority-proof re-exports are explicitly deferred to a follow-up spec?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:136
  - Recommended answer: Port `connectedAuthRequirement` as a `pub(crate) fn connected_auth_requirement(auth: &runx_contracts::JsonValue) -> Option<ConnectedAuthRequirement>` inside a new private `policy::connected_auth` module (or as a private helper inside `policy::local`). It must not be re-exported from `runx_core::policy`; the public surface for authority-proof stays deferred. Update the Phase 2 `policy/local.rs` change description to call this out, plus a `ConnectedAuthRequirement` struct in `policy::types` (also `pub(crate)`). The same applies to `findMatchingGrant`, `grantReferenceMatches`, `hasGrantReference` - all private helpers in the Rust local-admission module.
  - If unanswered: Add a `policy::connected_auth` private submodule containing `connected_auth_requirement` plus the supporting `find_matching_grant`/`grant_reference_matches`/`has_grant_reference` helpers, all `pub(crate)`. Note in the spec that none of these are re-exported from `runx_core::policy`.
- How is `auth: unknown` in `LocalAdmissionSkill` typed in Rust so it can carry arbitrary JSON shapes through the fixture runner without exposing `serde_json::Value` at a public boundary (forbidden by the style guard)?
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:22
  - Recommended answer: Use `runx_contracts::JsonValue` for the `auth` field on the Rust `LocalAdmissionSkill` struct (same convention as the state-machine fixture runner at `crates/runx-core/tests/state_machine_fixtures.rs:5`). The style guard's `serde_json::Value` ban does not apply to `runx_contracts::JsonValue`, which is the accepted deterministic wrapper. Mention this in the Phase 1 `types.rs` change description so the reviewer can verify the convention is mirrored.
  - If unanswered: Type `LocalAdmissionSkill.auth` and any other `unknown`-shaped fields as `Option<runx_contracts::JsonValue>` with `#[serde(skip_serializing_if = "Option::is_none")]`, matching the state-machine fixture runner convention.
- Will Phase 4 also add the `authority-proof and public-work` deferred-follow-up phrase to `fixtures/kernel/README.md`, given that `ac4_2` asserts that phrase in BOTH `docs/trusted-kernel-package-truth.md` AND `fixtures/kernel/README.md`?
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:461
  - Recommended answer: Yes - widen the Phase 4 Change description for `fixtures/kernel/README.md` to explicitly include both phrases: `Rust policy fixtures are policy parity evidence` AND a sentence containing `authority-proof and public-work re-exports are deferred to a follow-up spec`. Alternatively, split `ac4_2` so the trusted-kernel-package-truth.md and fixtures/kernel/README.md gates are independent and only require the deferred phrase in the doc that actually carries the deferral note. The current pairing creates a gate the change list does not satisfy.
  - If unanswered: Expand the Phase 4 Change description for `fixtures/kernel/README.md` to include both required phrases, so the `ac4_2` gate passes after Phase 4 work lands.
- How will the regex split inside `isUnsafeWritablePath` (`value.split(/[\\\/]+/).includes('..')` in `sandbox.ts:95`) be ported without adding the `regex` crate, given that the spec already commits to hand-rolled ASCII helpers for `detectInlineInterpreter`?
  - Grounded in: code:oss/packages/core/src/policy/sandbox.ts:94
  - Recommended answer: Hand-roll a small `path_segments_contain_parent(value: &str) -> bool` helper inside `policy::sandbox` that iterates byte-by-byte, treats `'/'` and `'\\'` as separators, and checks each non-empty segment for exact equality with `..`. Include in-module unit tests for `""`, `"a/.."`, `"a\\..\\b"`, `".."`, `"a..b"`. Add this detail to the Phase 2 `sandbox.rs` change description so the regex port is not silently delegated to author judgment.
  - If unanswered: Pin a hand-rolled segment-walking helper for `isUnsafeWritablePath`; do not introduce the `regex` crate.
- Should `ac1_1`'s redundant `--no-fail-fast --no-run` combination be cleaned up - either drop `--no-fail-fast` (since `--no-run` short-circuits running) or have ac1_1 actually run the policy tests instead of compile-only?
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:338
  - Recommended answer: Replace `ac1_1` with `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-run` (compile-only is the intent because `ac1_2` and `ac1_5` already run the in-module tests). Dropping `--no-fail-fast` makes the intent clearer and the command shorter.
  - If unanswered: Drop `--no-fail-fast` from ac1_1; keep `--no-run`.

Design objections:
- `objection-1` high - `admit_local_skill` depends on `connectedAuthRequirement`, which the spec only lists under deferred authority-proof re-exports.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:136
  - Evidence: `packages/core/src/policy/index.ts:136` calls `connectedAuthRequirement(skill.auth)` to extract a `ConnectedAuthRequirement`. That function lives in `packages/core/src/policy/authority-proof.ts:68` and is part of the explicitly deferred `policy/authority-proof` surface (spec lines 140-143). The in-scope policy fixture `fixtures/kernel/policy/local-admission-allows-connected-wildcard-grant.json` carries `auth: { provider: 'github', scopes: ['repo:read'], type: 'nango' }`, which can only be admitted if Rust ports the requirement-extraction logic. The spec never says how the Rust port resolves this: as a `pub(crate)` private helper inside `policy::local` / `policy::connected_auth`? Or by ignoring `auth` until the follow-up? Without explicit pinning, an implementer may inline a divergent shape, drop the `auth.type === 'nango'` recognition, or skip the connected-grant fixture entirely.
  - Recommendation: Add a Phase 2 change entry for a private `policy::connected_auth` module (or place the helpers inside `policy::local`) that ports `connectedAuthRequirement`, `findMatchingGrant`, `grantReferenceMatches`, and `hasGrantReference` as `pub(crate)` Rust functions. Pin the visibility (`pub(crate)`, never re-exported from `runx_core::policy`). Add a `ConnectedAuthRequirement` struct in `policy::types` (also `pub(crate)`). Note this is internal scaffolding for `admit_local_skill` parity; the public authority-proof surface remains deferred to `rust-policy-authority-proof-parity`.
- `objection-2` medium - `ac4_2` requires the deferred-follow-up phrase in `fixtures/kernel/README.md`, but the Phase 4 Change description for that file only promises a different phrase.
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:461
  - Evidence: `ac4_2` runs `rg -n 'authority-proof and public-work.*follow-up|authority-proof.*public-work.*deferred' docs/trusted-kernel-package-truth.md && rg -n '...' fixtures/kernel/README.md`. The AND means both rg invocations must succeed. The Phase 4 Change description for `fixtures/kernel/README.md` (line 452-453) only promises `Rust policy fixtures are policy parity evidence`; it does not promise the deferred-follow-up phrase. After Phase 4 implementation as written, `ac4_2` fails. Confirmed by Grep: the deferred phrase currently appears only in `.scafld/specs/drafts/rust-policy-parity.md` and the archived fixtures spec, not in `fixtures/kernel/README.md` itself.
  - Recommendation: Either (a) expand the Phase 4 Change description for `fixtures/kernel/README.md` to add both phrases - `Rust policy fixtures are policy parity evidence` AND a sentence containing `authority-proof and public-work re-exports are deferred to a follow-up spec` - or (b) drop the second `rg` from `ac4_2` and only assert the deferred phrase in `docs/trusted-kernel-package-truth.md` where Phase 4 explicitly promises it.
- `objection-3` medium - `isUnsafeWritablePath` uses a regex split, but the spec only commits to hand-rolled ASCII helpers for `detectInlineInterpreter`.
  - Grounded in: code:oss/packages/core/src/policy/sandbox.ts:94
  - Evidence: `sandbox.ts:94-96`: `function isUnsafeWritablePath(value) { return value.length === 0 || value.split(/[\\/]+/).includes('..'); }`. Spec Dependencies block (line 156-160) pins hand-rolled ASCII helpers for `detectInlineInterpreter`, `unwrapEnvCommand`, and `isPythonLike`, but does not list `isUnsafeWritablePath`. Without an explicit decision, the implementer might (a) add a one-off `regex` crate dep just for this, contradicting the dependency posture, (b) split on bytes manually but skip the empty-segment edge case, or (c) use `std::path::Path` segments (forbidden by invariant). Spec must pin this.
  - Recommendation: Add an `is_unsafe_writable_path(value: &str) -> bool` private helper inside `policy::sandbox` that walks bytes and treats `'/'` and `'\\'` as separators, with in-module unit tests for `''`, `'..'`, `'a/..'`, `'a\\..\\b'`, `'a..b'`. Mention this in the Phase 2 `sandbox.rs` change description.
- `objection-4` low - The Rust `LocalAdmissionSkill.auth` field's type is not pinned, but the style guard forbids `serde_json::Value` at the public boundary.
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:22
  - Evidence: `scripts/check-rust-core-style.mjs:22-25` denies `serde_json::Value` everywhere under `crates/*/src`. TS `LocalAdmissionSkill.auth: unknown` carries arbitrary JSON. The state-machine fixture runner uses `serde_json::Value` only inside the test file (`tests/state_machine_fixtures.rs:10`), which is allowed because the style guard only scans `crates/*/src`. But the policy `LocalAdmissionSkill` struct lives in `src/policy/types.rs`, where it must avoid `serde_json::Value`. `runx_contracts::JsonValue` is the documented escape hatch (`docs/rust-kernel-architecture.md:307`). The spec does not pin this typing choice.
  - Recommendation: In the Phase 1 `types.rs` change description, pin `LocalAdmissionSkill.auth` (and any other `unknown`-shaped field, such as `runtime`) as `Option<runx_contracts::JsonValue>` with `#[serde(skip_serializing_if = "Option::is_none")]`. Reference the state-machine convention and `docs/rust-kernel-architecture.md` section 10.
- `objection-5` low - `ac1_1`'s `--no-fail-fast --no-run` flags combine redundantly and obscure the intent.
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:338
  - Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-fail-fast --no-run`: `--no-run` compiles the test binary and stops, so `--no-fail-fast` (which only affects test execution) is a no-op. A reader inspecting the spec may misread this as actually running tests. The intent (per the surrounding text) is compile-only.
  - Recommendation: Replace `ac1_1` with `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-run` (drop `--no-fail-fast`). If actual execution is desired, drop `--no-run` and rely on the policy filter.

Recommended edits:
- Phase 2: Admission parity > Changes
  - Grounded in: code:oss/packages/core/src/policy/index.ts:136
  - Recommendation: Add a new change entry: `crates/runx-core/src/policy/connected_auth.rs` (all, exclusive) - private module hosting `connected_auth_requirement`, `find_matching_grant`, `grant_reference_matches`, `has_grant_reference` as `pub(crate)` helpers used by `policy::local`. None are re-exported from `runx_core::policy`. The corresponding `ConnectedAuthRequirement` struct goes in `policy::types` as `pub(crate)`. Note in the spec that this is internal scaffolding for `admit_local_skill` parity, distinct from the deferred authority-proof public surface.
- Phase 2: Admission parity > Changes (sandbox.rs)
  - Grounded in: code:oss/packages/core/src/policy/sandbox.ts:94
  - Recommendation: Extend the `sandbox.rs` change description to commit to a hand-rolled `is_unsafe_writable_path(value: &str) -> bool` helper that walks bytes treating `'/'` and `'\\'` as separators and checks for empty input or any segment equal to `..`. Include in-module unit tests for `''`, `'..'`, `'a/..'`, `'a\\..\\b'`, `'a..b'`. Explicitly say no `regex` dependency is added for this helper.
- Phase 1: Policy data model > Changes (types.rs)
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:22
  - Recommendation: In the `types.rs` change description, pin `LocalAdmissionSkill.auth` and any other `unknown`-shaped fields as `Option<runx_contracts::JsonValue>` with `#[serde(skip_serializing_if = "Option::is_none")]`. Reference the same convention used by the state-machine fixture runner.
- Phase 4: Gap documentation > Changes (fixtures/kernel/README.md)
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:461
  - Recommendation: Expand the `fixtures/kernel/README.md` change description to include BOTH required phrases: `Rust policy fixtures are policy parity evidence` AND a sentence containing `authority-proof and public-work re-exports are deferred to a follow-up spec`. Alternatively, split `ac4_2` so the deferred-follow-up phrase is only asserted in the doc that actually carries it.
- Phase 1: Policy data model > Acceptance (ac1_1)
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:338
  - Recommendation: Drop `--no-fail-fast` from `ac1_1` (it is a no-op with `--no-run`). Final command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-run`.

### round-3

Status: failed
Started: 2026-05-18T01:30:01Z
Ended: 2026-05-18T01:30:01Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 revisions all landed correctly: `connected_auth` private module is in Phase 2 changes, `LocalAdmissionSkill.auth` is pinned to `Option<runx_contracts::JsonValue>`, `is_unsafe_writable_path` commits to a hand-rolled segment walker, the `fixtures/kernel/README.md` change description now promises both required phrases, and `ac1_1` drops `--no-fail-fast`. Three new defects remain: (1) the Phase 4 doc edit description says "Update section 14 if any deferred decisions changed", but section 14 of `docs/rust-kernel-architecture.md` is "Placeholder publishing strategy"; deferred decisions live in section 16, so the section reference is wrong; (2) the spec's `types.rs` change description does not pin the serde rename strategy for `LocalAdmissionGrant` and `GraphScopeGrant`, whose fields are snake_case in the fixture JSON (`grant_id`, `scope_family`, `authority_kind`, `target_repo`, `target_locator`), while the surrounding `GraphScopeAdmissionDecision.grantId` uses camelCase - this asymmetry will silently break fixture parity if a Rust implementer follows the default camelCase convention from arch doc section 6; (3) the scope description states "Re-exports from `@runxhq/core/policy/authority-proof` (6 functions, 6 types)", but `packages/core/src/policy/index.ts` re-exports only 5 functions (`buildAuthorityProof`, `buildAuthorityProofMetadata`, `buildLocalScopeAdmission`, `connectedAuthRequirement`, `validateCredentialBinding`) and 6 types, so the count is off by one.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:7
  - Result: passed
  - Evidence: All declared paths are intentional future files (none of `crates/runx-core/src/policy.rs`, `src/policy/*.rs`, `tests/policy_fixtures.rs`, or `tests/policy_proptest.rs` currently exist - lib.rs has only `serde_conventions` and `state_machine`) or real prerequisites that exist now: `scripts/check-rust-core-style.mjs` (has `checkStateMachineFixtureCoverage` at line 141, no policy analogue), `packages/core/src/policy/index.ts` (no longer imports `node:path`, uses `./posix-basename.js` per line 5), `packages/core/src/policy/sandbox.ts` (with the `isUnsafeWritablePath` regex split at line 95), `packages/core/src/policy/authority-proof.ts` (with `connectedAuthRequirement` at line 68), `packages/core/src/policy/posix-basename.ts`, 20 `fixtures/kernel/policy/*.json` files, `docs/rust-kernel-architecture.md`, `docs/trusted-kernel-package-truth.md`, `fixtures/kernel/README.md`, `crates/runx-core/Cargo.toml` (no `regex` or `indexmap`), `crates/runx-contracts/src/lib.rs` (exposes `JsonValue`). The Phase 1 constraint that `policy.rs` declares only `types` and `posix_basename` matches the `state_machine.rs` pattern at `crates/runx-core/src/state_machine.rs:1-17`.
- command audit
  - Grounded in: code:oss/crates/runx-core/Cargo.toml:18
  - Result: passed
  - Evidence: Runnable surface checks out. `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-run` (ac1_1) compiles; `cargo test ... posix_basename` and `cargo test ... policy::types` (ac1_2/ac1_5) filter in-module tests; `cargo test ... --test policy_fixtures` and `--test policy_proptest` (ac2_1/ac3_1) work because integration tests live under `tests/`. v4 ripgrep `! rg 'std::fs|std::process|std::net|std::env|std::time::SystemTime|std::path::Path|tokio|reqwest|ureq|hyper|rmcp|Command::new' crates/runx-core/src crates/runx-core/tests` is consistent with the current `runx-core` deps (serde, serde_json, runx-contracts) and the no-runtime-API invariant. v5 cargo-deny runs against `crates/Cargo.toml`. v7 invokes the two style scripts that already exist. `proptest` is already a dev-dep at line 32.
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/policy/index.ts:46
  - Result: failed
  - Evidence: The fixture JSON in `fixtures/kernel/policy/local-admission-allows-connected-wildcard-grant.json:15-23` and `fixtures/kernel/policy/graph-scope-allows-exact-match.json:22-28` shows the input grant uses snake_case keys (`grant_id`, no `scope_family`/`authority_kind`/`target_repo`/`target_locator` here but the TS types support them as snake_case at `packages/core/src/policy/index.ts:46-54`). Meanwhile the output `GraphScopeAdmissionDecision` uses camelCase `grantId` (confirmed in `fixtures/kernel/policy/graph-scope-allows-exact-match.json:6`). The arch doc section 6 (`docs/rust-kernel-architecture.md:215-216`) pins the default serde struct convention as `#[serde(rename_all = "camelCase")]`. The spec's Phase 1 `types.rs` change description does not call out the snake_case exception for `LocalAdmissionGrant` and `GraphScopeGrant`. An implementer who follows the default camelCase would silently break fixture parity for connected-grant matching and graph-scope grant_id lookup. The `connected_auth` module also depends on the same `LocalAdmissionGrant` snake_case fields, so the convention question is shared across `policy::types`, `policy::connected_auth`, and `policy::graph_scope`.
- acceptance timing audit
  - Grounded in: code:oss/fixtures/kernel/README.md:1
  - Result: passed
  - Evidence: ac1_1 (`--no-run` only) is appropriately compile-only because the actual Phase 1 unit tests run via ac1_2 (`posix_basename`) and ac1_5 (`policy::types`). ac1_4 (`node scripts/check-rust-core-style.mjs`) passes in Phase 1 because the script's coverage check short-circuits when the test file does not yet exist (mirrors `checkStateMachineFixtureCoverage` lines 144-146: returns early if `testFile` is missing). Phase 2 fixture coverage check fires only once `tests/policy_fixtures.rs` is added. Phase 4 ac4_1 and ac4_2 use file-and-phrase-specific patterns that do not currently match anything in `docs/trusted-kernel-package-truth.md` or `fixtures/kernel/README.md` (verified by Grep returning no matches for `Rust policy parity status: fixture-evidence-only`, `Rust policy fixtures are policy parity evidence`, `authority-proof and public-work`, `authority-proof.*public-work.*deferred`); the gates are non-vacuous.
- rollback/repair audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: Rollback is credible because every Rust artifact (`crates/runx-core/src/policy.rs`, `src/policy/`, `tests/policy_fixtures.rs`, `tests/policy_proptest.rs`) is net-new and reversible by `rm`. `crates/runx-core/src/lib.rs` currently has only `pub mod serde_conventions;` and `pub mod state_machine;` so reverting the new `pub mod policy;` is a one-line change explicitly called out. Script and doc edits are reversible by `git checkout`. `per_phase` rollback matches the additive shape of the work.
- design challenge
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:476
  - Result: passed
  - Evidence: Right architectural move, not bandaid or future-bloat. The plan mirrors the already-archived `rust-state-machine-parity` shape: small modules, named re-exports, fixture parity, proptest determinism, no runtime APIs. Section 18 of the arch doc (line 476-499) sets the quality bar this spec defers to. The deferred authority-proof and public-work surface is bounded by a real follow-up draft at `.scafld/specs/drafts/rust-policy-authority-proof-parity.md`. The private `policy::connected_auth` carve-out is the minimum needed to admit the in-scope connected-grant fixture without widening the public authority-proof surface. Bounded blast radius: Rust policy is not runtime-authoritative (documented in Phase 4) and TypeScript remains the source of truth.

Questions:
- Should the Phase 4 `docs/rust-kernel-architecture.md` edit description reference section 16 instead of section 14, or be rewritten to describe the actual section-14 (Placeholder publishing strategy) edit?
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:428
  - Recommended answer: Change the Phase 4 description for `docs/rust-kernel-architecture.md` from 'Update section 14 if any deferred decisions changed during implementation' to 'Update section 14 (Placeholder publishing strategy) to reflect that runx-core now contains policy parity in addition to state-machine parity. Include the exact phrase `runx-core policy parity is not runtime-authoritative`.' Section 14 already carries the 'runx-core was reserved at 0.0.1 and now contains the first real Rust kernel surface: state-machine parity' sentence, which is the natural place to mention policy parity. Section 16 ('Open questions intentionally deferred') is unrelated to runtime-authoritativeness language.
  - If unanswered: Replace 'section 14' with 'section 14 (Placeholder publishing strategy)' so the implementer updates the placeholder-status paragraph that already mentions state-machine parity.
- Should the Phase 1 `types.rs` change description pin per-struct serde rename strategies so the snake_case grant fields do not get auto-renamed to camelCase by the default convention?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:46
  - Recommended answer: Yes. Add to the `types.rs` change description: `LocalAdmissionGrant` and `GraphScopeGrant` use `#[serde(rename_all = "snake_case")]` (their fields `grant_id`, `scope_family`, `authority_kind`, `target_repo`, `target_locator` are snake_case in fixture JSON), while `LocalAdmissionSkill`, `LocalAdmissionSandbox`, `LocalAdmissionOptions`, `RetryAdmissionRequest`, `GraphScopeAdmissionRequest`, `GraphScopeAdmissionDecision`, and `SandboxDeclaration` use the default `camelCase`. The serde round-trip test in `types.rs` should cover at least one grant deserialization with all snake_case targeting fields plus one decision serialization with `grantId`, so the asymmetric naming is caught at Phase 1 instead of waiting for Phase 2's fixture run.
  - If unanswered: Pin `LocalAdmissionGrant` and `GraphScopeGrant` to `#[serde(rename_all = "snake_case")]` and mention this in the Phase 1 types.rs change description with a serde round-trip test covering both snake_case grant input and camelCase decision output.
- Should the deferred authority-proof function count in the Scope section be corrected from 6 to 5 to match `packages/core/src/policy/index.ts`?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:371
  - Recommended answer: Yes. The `export { ... } from './authority-proof.js'` block at `packages/core/src/policy/index.ts:371-383` re-exports exactly 5 functions (`buildAuthorityProof`, `buildAuthorityProofMetadata`, `buildLocalScopeAdmission`, `connectedAuthRequirement`, `validateCredentialBinding`) and 6 types. Change `(6 functions, 6 types)` to `(5 functions, 6 types)` in the Scope section's deferred bullet. Minor but the count is wrong and a follow-up spec inheriting this scope would carry the wrong number.
  - If unanswered: Change the deferred authority-proof line to `(5 functions, 6 types)`.

Design objections:
- `objection-1` medium - Phase 4 says to update section 14 if deferred decisions changed, but section 14 is `Placeholder publishing strategy` (deferred decisions live in section 16).
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:428
  - Evidence: `docs/rust-kernel-architecture.md` section 14 is titled 'Placeholder publishing strategy' (line 428) and discusses crates.io reservation and publish order; the existing line 'runx-core was reserved at 0.0.1 and now contains the first real Rust kernel surface: state-machine parity. It remains conformance evidence only until a cutover spec replaces TypeScript consumers.' is the natural place to mention policy parity. Section 16 (line 458) is 'Open questions intentionally deferred', which carries the deferred-decisions content the spec's Phase 4 description seems to refer to. The mismatch can be read either way - either the section number is wrong, or the description ('if any deferred decisions changed') is wrong. The implementer may edit the wrong section, or skip the edit entirely because no 'deferred decisions' changed.
  - Recommendation: Rewrite the Phase 4 `docs/rust-kernel-architecture.md` change description to: 'Update section 14 (Placeholder publishing strategy) to add a sentence that `runx-core` now contains policy parity alongside state-machine parity, including the exact phrase `runx-core policy parity is not runtime-authoritative`.' Drop the conditional 'if any deferred decisions changed' phrasing. If the intent was section 16, change the number and rewrite the description to fit deferred-decisions content.
- `objection-2` medium - The spec does not pin serde rename strategy for `LocalAdmissionGrant`/`GraphScopeGrant`, whose fields are snake_case in fixture JSON.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:46
  - Evidence: TS `LocalAdmissionGrant` at `packages/core/src/policy/index.ts:46-54` uses snake_case fields (`grant_id`, `scope_family`, `authority_kind`, `target_repo`, `target_locator`), and `GraphScopeGrant.grant_id` at line 66 is also snake_case. The fixture `fixtures/kernel/policy/graph-scope-allows-exact-match.json:24` carries `"grant_id": "grant_1"` while the same fixture's decision uses camelCase `"grantId": "grant_1"` (line 6). Arch doc section 6 (`docs/rust-kernel-architecture.md:215-216`) pins `#[serde(rename_all = "camelCase")]` as the default struct convention. The Phase 1 `types.rs` change description (spec lines 339-345) does not call out the snake_case exception; an implementer following the default camelCase will silently break fixture parity for connected-grant matching and graph-scope grant_id lookup. The Phase 2 `policy::connected_auth` helpers depend on the same `LocalAdmissionGrant` snake_case fields, so the deserialization issue propagates.
  - Recommendation: Add to the Phase 1 `types.rs` change description: `LocalAdmissionGrant` and `GraphScopeGrant` use `#[serde(rename_all = "snake_case")]` because their fixture-JSON fields are snake_case; all other policy structs use the default `camelCase`. The serde round-trip test must cover both a grant deserialization (snake_case targeting fields) and a decision serialization (`grantId`, `grantedScopes`, `requestedScopes`, `stepId`) so the asymmetric naming is verified in Phase 1.
- `objection-3` low - Spec's deferred authority-proof scope says `(6 functions, 6 types)` but the index.ts re-export block has 5 functions.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:371
  - Evidence: `packages/core/src/policy/index.ts:371-383` re-exports five functions (`buildAuthorityProof`, `buildAuthorityProofMetadata`, `buildLocalScopeAdmission`, `connectedAuthRequirement`, `validateCredentialBinding`) and six types from `./authority-proof.js`. The spec's deferred-scope bullet records `(6 functions, 6 types)`, an off-by-one count. A follow-up `rust-policy-authority-proof-parity` spec inheriting this number would over-promise its surface.
  - Recommendation: Change the count in the Scope section's deferred bullet from `(6 functions, 6 types)` to `(5 functions, 6 types)`.

Recommended edits:
- Phase 4: Gap documentation > Changes (docs/rust-kernel-architecture.md)
  - Grounded in: code:oss/docs/rust-kernel-architecture.md:428
  - Recommendation: Replace 'Update section 14 if any deferred decisions changed during implementation. If edited, include the exact phrase `runx-core policy parity is not runtime-authoritative`.' with 'Update section 14 (Placeholder publishing strategy) so the `runx-core` placeholder status reflects that the crate now contains policy parity in addition to state-machine parity. Include the exact phrase `runx-core policy parity is not runtime-authoritative` so the gate proves this phase edited the doc.' Drop the conditional phrasing so the Phase 4 edit is mandatory, not optional.
- Phase 1: Policy data model > Changes (types.rs)
  - Grounded in: code:oss/packages/core/src/policy/index.ts:46
  - Recommendation: Extend the `policy/types.rs` change description to pin: `LocalAdmissionGrant` and `GraphScopeGrant` use `#[serde(rename_all = "snake_case")]` (fields `grant_id`, `scope_family`, `authority_kind`, `target_repo`, `target_locator` are snake_case in fixture JSON), while all other policy structs use the default `camelCase`. Extend the in-module serde round-trip test to cover both a grant deserialization with snake_case targeting fields and a `GraphScopeAdmissionDecision` serialization with `grantId`, so the asymmetric naming is verified in Phase 1 rather than discovered during the Phase 2 fixture run.
- Scope > Explicitly out of scope (deferred follow-up)
  - Grounded in: code:oss/packages/core/src/policy/index.ts:371
  - Recommendation: Change the count in the deferred bullet `Re-exports from @runxhq/core/policy/authority-proof (6 functions, 6 types)` to `(5 functions, 6 types)` to match the actual re-export block in `packages/core/src/policy/index.ts:371-383`.
- Acceptance > v6 / Phase 3 > ac3_1
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:454
  - Recommendation: Consider adding `--release` to `cargo test ... --test policy_proptest` once strategies grow, so the 60-second cap is realistic; alternatively, pin a `ProptestConfig::with_cases(N)` ceiling in the Phase 3 description so a future strategy expansion does not blow the timeout. Today both v6 and ac3_1 share the same 60s cap with no case-count guidance.

### round-4

Status: failed
Started: 2026-05-18T01:37:52Z
Ended: 2026-05-18T01:37:52Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-3 revisions landed cleanly: the section-14 arch-doc edit description is now mandatory and specific to placeholder publishing strategy, LocalAdmissionGrant/GraphScopeGrant pin `#[serde(rename_all = \"snake_case\")]` while decision outputs keep camelCase, and the deferred authority-proof re-export count is corrected to (5 functions, 6 types). Three defects remain. (1) ac4_1 and ac4_2 only check trusted-kernel-package-truth.md and fixtures/kernel/README.md; neither verifies the exact phrase `runx-core policy parity is not runtime-authoritative` lands in docs/rust-kernel-architecture.md (grep confirms it is absent today), so the section-14 edit Phase 4 promises is not gated. (2) The SandboxAdmissionDecision approval_required variant emits snake_case in fixtures (sandbox-requires-unrestricted-approval.json:9) but the kebab-case-with-overrides rule in the spec Assumptions block never pins the per-variant `#[serde(rename = \"approval_required\")]` override in the Phase 1 types.rs change description; the round-trip test will silently pass with `approval-required` until Phase 2's fixture run breaks. (3) Phase 3 reuses the 60s cap from v6 but does not pin a `ProptestConfig::with_cases(N)` ceiling like state_machine_proptest.rs:16, leaving the timeout fragile as four property assertions and broader strategies land.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:7
  - Result: passed
  - Evidence: All Rust artifact paths declared in the spec (crates/runx-core/src/policy.rs, src/policy/{types,posix_basename,local,retry,graph_scope,interpreter,scope,sandbox,connected_auth}.rs, tests/policy_fixtures.rs, tests/policy_proptest.rs) are intentional future files. None exist today; lib.rs only carries `pub mod serde_conventions; pub mod state_machine;`. All prerequisite paths exist: scripts/check-rust-core-style.mjs (line 141 has checkStateMachineFixtureCoverage), packages/core/src/policy/index.ts (node:path import already replaced; uses ./posix-basename.js per line 5), packages/core/src/policy/{sandbox,authority-proof,posix-basename}.ts, 20 fixtures/kernel/policy/*.json files, docs/{rust-kernel-architecture,trusted-kernel-package-truth}.md, fixtures/kernel/README.md, crates/runx-contracts/src/lib.rs exposing JsonValue (line 13).
- command audit
  - Grounded in: code:oss/crates/runx-core/Cargo.toml:18
  - Result: passed
  - Evidence: ac1_1 `cargo test --no-run` compiles; ac1_2/ac1_5 filter in-module tests; ac2_1/ac3_1 use integration test paths that match the spec's tests/ layout. v4 ripgrep `! rg 'std::fs|std::process|...|Command::new' crates/runx-core/{src,tests}` aligns with existing runx-core deps (serde, serde_json, runx-contracts; proptest dev-dep). v5 cargo-deny runs against crates/Cargo.toml. v7 invokes the two existing style scripts. The fixture-coverage short-circuit in checkStateMachineFixtureCoverage:144-146 (returns early if testFile is missing) is what allows the Phase 1 `ac1_4` to pass before policy_fixtures.rs exists, since the spec promises the same shape for the policy coverage check.
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/policy/index.ts:136
  - Result: passed
  - Evidence: Round 2 carve-out is correct: admit_local_skill depends on connectedAuthRequirement (index.ts:136). Spec Phase 2 now lists crates/runx-core/src/policy/connected_auth.rs as a private module with pub(crate) helpers (connected_auth_requirement, find_matching_grant, grant_reference_matches, has_grant_reference) used only by policy::local and never re-exported from runx_core::policy. The public authority-proof surface remains deferred to rust-policy-authority-proof-parity. Round 3 corrected the (5 functions, 6 types) count to match index.ts:371-383. Snake_case rename for LocalAdmissionGrant and GraphScopeGrant is now pinned in the Phase 1 types.rs change description.
- acceptance timing audit
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:494
  - Result: failed
  - Evidence: ac4_1 checks `Rust policy parity status: fixture-evidence-only` in docs/trusted-kernel-package-truth.md and `Rust policy fixtures are policy parity evidence` in fixtures/kernel/README.md. ac4_2 checks the deferred-follow-up phrase in the same two files. Neither acceptance command verifies the third Phase 4 doc edit: the Phase 4 Changes block (line 482-487) promises the exact phrase `runx-core policy parity is not runtime-authoritative` in docs/rust-kernel-architecture.md, but no rg/test in the spec asserts that phrase landed. Grep across docs/ confirms the phrase is absent today. An implementer can mark Phase 4 complete by editing the other two docs and silently skipping the arch doc edit, leaving the placeholder publishing strategy out of date.
- rollback/repair audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: All Rust artifacts are net-new (crates/runx-core/src/policy.rs, src/policy/, tests/policy_fixtures.rs, tests/policy_proptest.rs do not exist today; rm is sufficient). crates/runx-core/src/lib.rs currently has only `pub mod serde_conventions;` and `pub mod state_machine;`, so reverting the `pub mod policy;` export is a one-line change explicitly called out. Script and doc edits are git-revertable. Per-phase rollback strategy matches the additive shape of the work.
- design challenge
  - Grounded in: code:oss/.scafld/specs/archive/2026-05/rust-state-machine-parity.md
  - Result: passed
  - Evidence: Right architectural move, not bandaid or future-bloat. The plan mirrors archived rust-state-machine-parity (small modules, fixture parity, proptest determinism, no runtime APIs). Section 18 (line 476+) of the arch doc sets the quality bar this spec defers to. Deferred authority-proof and public-work surface is bounded by a real draft at .scafld/specs/drafts/rust-policy-authority-proof-parity.md. The private policy::connected_auth carve-out is the minimum needed for the in-scope connected-grant fixture without widening the public authority-proof surface. Rust policy is not runtime-authoritative (Phase 4 documents this) and TypeScript remains the source of truth.

Questions:
- Should ac4_1 be extended with a third rg invocation that asserts the exact phrase `runx-core policy parity is not runtime-authoritative` lands in `docs/rust-kernel-architecture.md`, so the Phase 4 arch-doc edit cannot be silently skipped?
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:494
  - Recommended answer: Yes. Change ac4_1 to `rg -n 'Rust policy parity status: fixture-evidence-only' docs/trusted-kernel-package-truth.md && rg -n 'Rust policy fixtures are policy parity evidence' fixtures/kernel/README.md && rg -n 'runx-core policy parity is not runtime-authoritative' docs/rust-kernel-architecture.md`. The Phase 4 Changes block already promises this phrase; the gate must verify it. Without it, the placeholder-publishing-strategy update is on author discipline, not enforcement. Grep confirms the phrase is currently absent from docs/.
  - If unanswered: Append the third rg to ac4_1 so all three Phase 4 doc edits are gated.
- Should the Phase 1 `types.rs` change description and its in-module serde round-trip test be widened to pin the `approval_required` per-variant rename for `SandboxAdmissionDecision`?
  - Grounded in: code:oss/fixtures/kernel/policy/sandbox-requires-unrestricted-approval.json:9
  - Recommended answer: Yes. The fixture emits `status: approval_required` (snake_case), but the spec's enum convention (Assumptions, lines 184-186) is kebab-case with per-variant override 'only where TS uses an irregular form'. The `approval_required` value IS the irregular form. Pin in the `types.rs` change description that `SandboxAdmissionDecision::ApprovalRequired` carries `#[serde(rename = "approval_required")]`, and extend the required serde round-trip test to include a `SandboxAdmissionDecision::ApprovalRequired` serialization. Otherwise an implementer following the default kebab-case rule will emit `approval-required` and silently break Phase 2's sandbox-requires-unrestricted-approval fixture.
  - If unanswered: Add an explicit `#[serde(rename = "approval_required")]` note to the types.rs change description and require the round-trip test to cover this variant.
- Should the Phase 3 description pin a `ProptestConfig::with_cases(N)` ceiling (matching the state-machine proptest's `with_cases(64)` at `state_machine_proptest.rs:16`) so the 60-second cap stays achievable as strategies expand?
  - Grounded in: code:oss/crates/runx-core/tests/state_machine_proptest.rs:16
  - Recommended answer: Yes. Add a sentence to the Phase 3 change description: `policy_proptest.rs declares ProptestConfig::with_cases(64)` to match the existing state-machine convention. Without this, the four named properties (idempotence on local admission, fixed-order grant matching, idempotent graph-scope dedup, idempotent narrowed graph-scope admission) could exceed 60s with the default 256 cases as strategies grow.
  - If unanswered: Pin `ProptestConfig::with_cases(64)` for policy_proptest.rs in the Phase 3 description.

Design objections:
- `objection-1` high - ac4_1 and ac4_2 do not verify the arch-doc edit Phase 4 promises.
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:494
  - Evidence: Phase 4 Changes (lines 482-487) commit the exact phrase `runx-core policy parity is not runtime-authoritative` to `docs/rust-kernel-architecture.md`. ac4_1 only checks `docs/trusted-kernel-package-truth.md` and `fixtures/kernel/README.md`; ac4_2 only checks the deferred-follow-up phrase in the same two files. No spec command verifies the arch doc was edited. Grep across docs/ confirms the phrase is currently absent. An implementer can mark Phase 4 complete by editing the other two docs and silently skipping section 14 of the arch doc, leaving the placeholder-publishing-strategy snapshot out of date.
  - Recommendation: Add a third rg to ac4_1: `rg -n 'runx-core policy parity is not runtime-authoritative' docs/rust-kernel-architecture.md`. Final ac4_1 becomes `rg -n 'Rust policy parity status: fixture-evidence-only' docs/trusted-kernel-package-truth.md && rg -n 'Rust policy fixtures are policy parity evidence' fixtures/kernel/README.md && rg -n 'runx-core policy parity is not runtime-authoritative' docs/rust-kernel-architecture.md`. This forces all three Phase 4 doc edits before the phase completes.
- `objection-2` medium - The `approval_required` variant rename on `SandboxAdmissionDecision` is not pinned in the Phase 1 types.rs description.
  - Grounded in: code:oss/fixtures/kernel/policy/sandbox-requires-unrestricted-approval.json:9
  - Evidence: The fixture emits `"status": "approval_required"` (snake_case). The spec Assumptions block (lines 184-186) says enum variants without payloads use `#[serde(rename_all = "kebab-case")]` with per-variant overrides 'only where TS uses an irregular form'. `approval_required` IS the irregular form. The Phase 1 types.rs Changes description (lines 339-352) covers LocalAdmissionGrant/GraphScopeGrant snake_case and GraphScopeAdmissionDecision camelCase, but never mentions the `approval_required` per-variant rename. A future implementer following the default kebab-case rule will emit `approval-required` and silently break the sandbox-requires-unrestricted-approval fixture when Phase 2 runs.
  - Recommendation: Extend the Phase 1 types.rs change description to: `SandboxAdmissionDecision::ApprovalRequired` carries `#[serde(rename = "approval_required")]` because the TS string-union form is snake_case (not kebab-case). Extend the required serde round-trip test to cover this variant serialization, so the asymmetric rename is caught in Phase 1 instead of waiting for Phase 2's fixture run.
- `objection-3` low - Phase 3 does not pin a proptest `with_cases(N)` ceiling, putting the 60s cap at risk.
  - Grounded in: code:oss/crates/runx-core/tests/state_machine_proptest.rs:16
  - Evidence: v6 and ac3_1 both cap at 60s. The existing state-machine proptest declares `ProptestConfig::with_cases(64)` (state_machine_proptest.rs:16). The new policy proptest carries four distinct property assertions (admission idempotence, fixed-order grant matching, graph-scope dedup idempotence, narrowed-request stability) over broader input strategies. Without a case-count cap, the default 256 cases can easily exceed 60s as strategies grow, producing flaky CI rather than the intended deterministic gate.
  - Recommendation: Pin `ProptestConfig::with_cases(64)` in the Phase 3 `tests/policy_proptest.rs` change description, matching the existing state-machine convention. Alternatively, raise the timeout cap with explicit case-count guidance. Round 3 already flagged this; round 4 should resolve it before approval.

Recommended edits:
- Phase 4: Gap documentation > Acceptance (ac4_1)
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:494
  - Recommendation: Append a third rg to ac4_1 so the arch-doc edit is gated: `rg -n 'Rust policy parity status: fixture-evidence-only' docs/trusted-kernel-package-truth.md && rg -n 'Rust policy fixtures are policy parity evidence' fixtures/kernel/README.md && rg -n 'runx-core policy parity is not runtime-authoritative' docs/rust-kernel-architecture.md`. Without this, the Phase 4 Changes promise for docs/rust-kernel-architecture.md (line 482-487) is not enforced.
- Phase 1: Policy data model > Changes (types.rs)
  - Grounded in: code:oss/fixtures/kernel/policy/sandbox-requires-unrestricted-approval.json:9
  - Recommendation: Extend the types.rs change description to pin: `SandboxAdmissionDecision::ApprovalRequired` carries `#[serde(rename = "approval_required")]` because TS emits the snake_case form `approval_required` rather than the kebab-case `approval-required`. The in-module serde round-trip test must cover a `SandboxAdmissionDecision::ApprovalRequired` serialization so this irregular-form override is verified in Phase 1, not discovered when Phase 2 runs sandbox-requires-unrestricted-approval.json.
- Phase 3: Property testing > Changes (policy_proptest.rs)
  - Grounded in: code:oss/crates/runx-core/tests/state_machine_proptest.rs:16
  - Recommendation: Add `ProptestConfig::with_cases(64)` to the policy_proptest description so the run stays within the 60s cap as strategies grow. Match the existing state-machine convention at state_machine_proptest.rs:16.

### round-5

Status: failed
Started: 2026-05-18T01:43:31Z
Ended: 2026-05-18T01:43:31Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-4 revisions all landed cleanly: ac4_1 now gates the arch-doc `runx-core policy parity is not runtime-authoritative` phrase, the `SandboxAdmissionDecision::ApprovalRequired` rename is pinned to `approval_required` with a required serde round-trip case, and Phase 3 commits to `ProptestConfig::with_cases(64)` matching the state-machine convention. Paths and prerequisites are verified (lib.rs only has `serde_conventions`+`state_machine` today, all `crates/runx-core/src/policy/**` are intentional future files, all 20 `fixtures/kernel/policy/*.json` exist, `runx_contracts::JsonValue` is real, `scripts/check-rust-core-style.mjs` has `checkStateMachineFixtureCoverage` at line 141 as the mirror target). The connected_auth carve-out correctly bridges in-scope `admit_local_skill` to the deferred `policy/authority-proof` public surface without widening it. Serde rename strategy is pinned per struct. Rollback is credible because every Rust artifact is net-new. Three low-severity gaps remain worth pinning before approval: (1) the `reasons` field type is not pinned for AdmissionDecision/SandboxAdmissionDecision/GraphScopeAdmissionDecision; arch doc section 5 says 'typed enums' but fixtures bake in interpolated strings like `"step 'deploy' requested scope(s) outside graph grant: deployments:write"` and the existing kernel precedent at `crates/runx-core/src/state_machine/fanout.rs:54` uses `format!(...)` String reasons \u2014 the Phase 1 types.rs description should pin `Vec<String>` so an implementer does not invent typed-reason enums that then need custom Serialize impls. (2) The Phase 1 dedup wording 'Vec<String> + BTreeSet<String> membership tracking' is technically fine (output is `Vec`, `BTreeSet` is internal only) but reads as a literal violation of arch doc section 6 / `fixtures/kernel/README.md:45` ('do not use HashSet or BTreeSet at serialized array boundaries'); a one-line scope clarifier would prevent confusion. (3) Phase 1 ac1_5 round-trip coverage explicitly names GraphScopeAdmissionDecision and SandboxAdmissionDecision::ApprovalRequired but not plain `AdmissionDecision`, the most-called shape (returned by both `admit_local_skill` and `admit_retry_policy`); a tag/discriminator error there would only surface in the Phase 2 fixture run.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: All Rust artifact paths (crates/runx-core/src/policy.rs, src/policy/{types,posix_basename,local,retry,graph_scope,interpreter,scope,sandbox,connected_auth}.rs, tests/policy_fixtures.rs, tests/policy_proptest.rs) are intentional future files. Confirmed by Glob on crates/runx-core/**: only state_machine + serde_conventions exist today. lib.rs at lines 1-7 has only `pub mod serde_conventions;` and `pub mod state_machine;`. Prerequisite paths all present: scripts/check-rust-core-style.mjs (checkStateMachineFixtureCoverage at line 141 short-circuits when the test file is missing - same shape the policy coverage check needs); packages/core/src/policy/{index,sandbox,authority-proof,posix-basename}.ts; 20 fixtures/kernel/policy/*.json including connected-wildcard-grant and sandbox-requires-unrestricted-approval; docs/{rust-kernel-architecture,trusted-kernel-package-truth}.md and fixtures/kernel/README.md; tests/kernel-parity-fixtures.test.ts; runx_contracts::JsonValue exposed at crates/runx-contracts/src/lib.rs:13. Phase 1 constraint that policy.rs declares only `types` and `posix_basename` matches the precedent at crates/runx-core/src/state_machine.rs:1-17 (`mod fanout; mod sequential_graph; mod single_step; mod types;` + `pub use ...`).
- command audit
  - Grounded in: code:oss/crates/runx-core/Cargo.toml:18
  - Result: passed
  - Evidence: All declared commands are runnable. ac1_1 `cargo test ... policy --no-run` compiles tests matching the `policy` filter without running them (round-2 fix dropped --no-fail-fast); ac1_2/ac1_5 use module-filter test runs that match in-module #[cfg(test)] suites; ac2_1/ac3_1 use --test names that map to integration tests under crates/runx-core/tests/; v4 ripgrep is consistent with the current runx-core deps (serde 1.0.228, serde_json 1.0.149, runx-contracts), with no std::process/std::fs/Command::new references today; v5 cargo-deny is real (crates/deny.toml exists); v7 invokes scripts/check-rust-crate-graph.mjs and scripts/check-rust-core-style.mjs which both exist. proptest is already a dev-dep at line 32. ac2_5 uses an rg over the script source so a no-op coverage function cannot satisfy the gate.
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/policy/index.ts:136
  - Result: passed
  - Evidence: Round-2 carve-out remains correct: index.ts:136 calls connectedAuthRequirement(skill.auth) from authority-proof.ts:68, which the spec defers at the PUBLIC surface but ports as `pub(crate)` helpers inside crates/runx-core/src/policy/connected_auth.rs (connected_auth_requirement, find_matching_grant, grant_reference_matches, has_grant_reference), explicitly not re-exported from `runx_core::policy`. Round-3 corrected the public-surface count to 5 functions, 6 types matching index.ts:371-383. Snake_case rename for LocalAdmissionGrant and GraphScopeGrant is now pinned in the Phase 1 types.rs change description (their fixture-JSON fields are snake_case at fixtures/kernel/policy/graph-scope-allows-exact-match.json:24 and local-admission-allows-connected-wildcard-grant.json:15-23). Decision shape is also pinned: spec Assumptions explicitly say 'Rust represents decisions as tagged enum variants ... JSON discriminator is `status` for admission decisions'.
- acceptance timing audit
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:144
  - Result: passed
  - Evidence: ac1_4 passes in Phase 1 because the planned policy coverage check will short-circuit when tests/policy_fixtures.rs does not yet exist (mirror of checkStateMachineFixtureCoverage:144-146 returning early if testFile is missing). ac2_5 asserts that the script TEXT references `checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs`, blocking no-op satisfaction. Phase 4 ac4_1 now requires all three exact phrases across docs/trusted-kernel-package-truth.md, fixtures/kernel/README.md, and docs/rust-kernel-architecture.md; Grep confirms none are present today, so the gates are non-vacuous. ac4_2 requires the deferred-follow-up phrase in both files, and both Phase 4 Change descriptions promise it. Phase 3 `ProptestConfig::with_cases(64)` keeps the 60s cap meaningful as four property assertions land.
- rollback/repair audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: Rollback is credible because every Rust artifact (crates/runx-core/src/policy.rs, src/policy/, tests/policy_fixtures.rs, tests/policy_proptest.rs) is net-new and reversible by rm. crates/runx-core/src/lib.rs currently has only `pub mod serde_conventions;` and `pub mod state_machine;`, so reverting the `pub mod policy;` export is a one-line change explicitly called out. Script and doc edits are reversible via git checkout. Per-phase rollback strategy matches the additive shape: each phase only adds new files and one-line lib.rs/script/doc edits, so a failed phase can be undone without disturbing prior phases.
- design challenge
  - Grounded in: code:oss/.scafld/specs/archive/2026-05/rust-state-machine-parity.md
  - Result: passed
  - Evidence: Right architectural move. The plan mirrors archived rust-state-machine-parity one-for-one (small modules, named re-exports from a thin policy.rs root, fixture parity, proptest determinism with `ProptestConfig::with_cases(64)`, no runtime APIs). Section 18 of docs/rust-kernel-architecture.md (lines 476-499) sets the quality bar this spec defers to. The deferred authority-proof and public-work surface is bounded by a real follow-up draft at .scafld/specs/drafts/rust-policy-authority-proof-parity.md. The private policy::connected_auth carve-out is the minimum needed to admit the in-scope connected-grant fixture without widening the deferred public authority-proof API. Future bloat risk is contained: Rust policy is not runtime-authoritative (Phase 4 documents this explicitly in three docs) and TypeScript remains the source of truth. Estimated 14h is consistent with the state-machine parity precedent at similar scope.

Questions:
- Should the Phase 1 `types.rs` change description explicitly pin `reasons: Vec<String>` for AdmissionDecision/SandboxAdmissionDecision/GraphScopeAdmissionDecision so an implementer following arch doc section 5 ('typed enums, not free-form strings') does not invent typed-reason enums that then need custom Serialize impls to reproduce fixture strings?
  - Grounded in: code:oss/fixtures/kernel/policy/graph-scope-denies-widening.json:11
  - Recommended answer: Yes. The fixture at line 11 bakes in `"step 'deploy' requested scope(s) outside graph grant: deployments:write"`, an interpolated string with the stepId and denied scope list. Other fixtures interpolate sandbox profile names, provider names, command names. Existing state-machine precedent at crates/runx-core/src/state_machine/fanout.rs:54 uses `format!(...)` to build a `reason: String` field. Pin in the Phase 1 types.rs description that `reasons` is `Vec<String>` for all three decision enums and that the strings are formatted at call sites in policy::local / policy::sandbox / policy::graph_scope. The arch doc section 5 'typed enums' guidance applies to discriminator-style rejection reasons; the fixture-baked formatted strings make `Vec<String>` the pragmatic choice that matches the existing kernel.
  - If unanswered: Pin `reasons: Vec<String>` in the Phase 1 types.rs change description and note that policy decision reasons are formatted at the call site (matching the fanout.rs precedent) rather than typed enums.
- Should the Phase 1 dedup wording explicitly say the BTreeSet membership tracker is function-local and never reaches a serialized boundary, so a reader does not flag it as a literal violation of arch doc section 6 / fixtures/kernel/README.md:45 ('do not use HashSet or BTreeSet at serialized array boundaries')?
  - Grounded in: code:oss/fixtures/kernel/README.md:45
  - Recommended answer: Yes. Extend the dedup sentence to: 'Ordered deduplication mirrors `unique()` with a `Vec<String>` output backed by a function-local `BTreeSet<String>` for O(log n) membership tracking; the `BTreeSet` never crosses a serialized boundary (the public/returned type stays `Vec<String>`), so the arch doc section 6 prohibition on `HashSet`/`BTreeSet` at serialized arrays still holds.' This kills the literal-reading ambiguity without changing the implementation choice.
  - If unanswered: Append a clause clarifying that `BTreeSet` is function-local and never reaches the serialized boundary.
- Should Phase 1 ac1_5 widen its serde round-trip test to also cover plain `AdmissionDecision` (used by both `admit_local_skill` and `admit_retry_policy`), not just `GraphScopeAdmissionDecision` and `SandboxAdmissionDecision::ApprovalRequired`?
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:339
  - Recommended answer: Yes. The Phase 1 types.rs description names round-trip coverage for the tagged GraphScope decision (camelCase grantId) and SandboxAdmissionDecision::ApprovalRequired (snake_case override), but does not require a round-trip case for plain `AdmissionDecision`. That type is the most-called decision shape (every `admit_local_skill` and `admit_retry_policy` call returns it) and a tag/rename mistake would silently slip until Phase 2's full fixture run. Add a one-line requirement: 'The in-module serde test must also cover one `AdmissionDecision::Allow` and one `AdmissionDecision::Deny` round-trip with `status: "allow"` / `status: "deny"` and a non-empty `reasons` array, so the discriminator and reasons array shape are verified in Phase 1.'
  - If unanswered: Extend the Phase 1 ac1_5 test scope to include AdmissionDecision allow/deny round-trips alongside the existing GraphScope and Sandbox cases.

Design objections:
- `objection-1` low - Reason field type is not pinned despite arch doc section 5 conflict with fixture format.
  - Grounded in: code:oss/crates/runx-core/src/state_machine/fanout.rs:54
  - Evidence: Fixtures bake in interpolated reason strings (e.g., fixtures/kernel/policy/graph-scope-denies-widening.json:11 contains "step 'deploy' requested scope(s) outside graph grant: deployments:write"; fixtures/kernel/policy/local-admission-allows-cli-tool.json's allow reason is the literal "local admission allowed"). Arch doc section 5 (`docs/rust-kernel-architecture.md:203-205`) says 'Rejection reasons are typed enums (AdmissionRejectionReason, SandboxRejectionReason, etc.), not free-form strings'. The existing state-machine precedent in `crates/runx-core/src/state_machine/fanout.rs:54` already uses `format!(...)` to build a `reason: String` field, so the kernel has relaxed the typed-enum rule for reasons. The spec's Phase 1 types.rs description does not pin the choice; an implementer following arch doc section 5 literally will invent typed reason enums and need custom Serialize impls to match the interpolated fixture strings, wasting work.
  - Recommendation: Pin in the Phase 1 types.rs change description: `AdmissionDecision`, `SandboxAdmissionDecision`, and `GraphScopeAdmissionDecision` carry `reasons: Vec<String>` (matching the `reason: String` precedent in `crates/runx-core/src/state_machine/fanout.rs:54`). Policy reason strings are formatted at the call site in `policy::local` / `policy::sandbox` / `policy::graph_scope` via `format!(...)`. Note that arch doc section 5's 'typed enums' guidance was relaxed in state-machine practice and that policy follows the same pragmatic shape.
- `objection-2` low - Phase 1 dedup wording literally reads as a BTreeSet usage that the arch doc and fixtures README forbid.
  - Grounded in: code:oss/fixtures/kernel/README.md:45
  - Evidence: Spec Dependencies block says 'Ordered deduplication mirrors `unique()` with `Vec<String>` plus `BTreeSet<String>` membership tracking, preserving first-seen order without adding `IndexSet`.' But fixtures/kernel/README.md:45 reads 'do not use `HashSet` or `BTreeSet` at serialized array boundaries' and docs/rust-kernel-architecture.md:227-230 echoes the same rule. The spec's choice is technically fine (the output is `Vec<String>`, the `BTreeSet` is internal scaffolding for O(log n) membership lookup), but a literal reading puts the spec in apparent violation of two prominent docs. A future reviewer or implementer may bounce the PR thinking the rule was broken.
  - Recommendation: Tighten the dedup sentence to make scope explicit: 'Ordered deduplication mirrors `unique()` with a `Vec<String>` output backed by a function-local `BTreeSet<String>` for O(log n) membership tracking. The `BTreeSet` is internal to the helper; the public/returned/serialized type stays `Vec<String>`, so the architecture rule about not exposing `HashSet`/`BTreeSet` at serialized array boundaries still holds.'
- `objection-3` low - Phase 1 round-trip test does not require coverage of plain `AdmissionDecision`.
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:339
  - Evidence: Phase 1 types.rs change description (lines 339-356) names round-trip coverage for `LocalAdmissionGrant` snake_case grant fields, `GraphScopeAdmissionDecision` camelCase `grantId`, and `SandboxAdmissionDecision::ApprovalRequired` snake_case rename. It does not require a round-trip case for plain `AdmissionDecision`, which is the most-called decision shape (returned by both `admit_local_skill` and `admit_retry_policy`). A tag/rename mistake on `AdmissionDecision` (e.g., wrong discriminator field name, missing kebab-case override on a variant) would slip through Phase 1 and only surface when the Phase 2 fixture runner deserializes the literal fixture JSON.
  - Recommendation: Add a one-line requirement to the Phase 1 types.rs change description: 'The in-module serde test must also cover one `AdmissionDecision::Allow` and one `AdmissionDecision::Deny` round-trip with `status: "allow"` / `status: "deny"` and a non-empty `reasons` array, so the discriminator and reasons shape are verified in Phase 1 instead of waiting for the Phase 2 fixture run.'

Recommended edits:
- Phase 1: Policy data model > Changes (types.rs)
  - Grounded in: code:oss/crates/runx-core/src/state_machine/fanout.rs:54
  - Recommendation: Pin reason typing: append to the types.rs change description that `AdmissionDecision`, `SandboxAdmissionDecision`, and `GraphScopeAdmissionDecision` carry `reasons: Vec<String>`. Policy reason strings are formatted at the call site (matching the existing `reason: String` precedent in `crates/runx-core/src/state_machine/fanout.rs`), not via typed reason enums. This forestalls an implementer trying to satisfy arch doc section 5 with typed enums that then need custom Serialize impls to reproduce fixture-baked interpolated strings.
- Dependencies / Phase 1 dedup wording
  - Grounded in: code:oss/fixtures/kernel/README.md:45
  - Recommendation: Tighten the dedup clause to: 'Ordered deduplication mirrors `unique()` with a `Vec<String>` output backed by a function-local `BTreeSet<String>` for O(log n) membership tracking. The `BTreeSet` is internal to the helper; the public/serialized type stays `Vec<String>`, so the architecture rule about not exposing `HashSet`/`BTreeSet` at serialized array boundaries still holds.' This kills the literal-reading ambiguity.
- Phase 1: Policy data model > Changes (types.rs) and Acceptance (ac1_5)
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:339
  - Recommendation: Extend the round-trip test scope to also cover `AdmissionDecision::Allow` and `AdmissionDecision::Deny` with discriminator `status` and non-empty `reasons`, so the most-called decision type's tag/rename shape is verified in Phase 1 rather than waiting for the Phase 2 fixture runner.

### round-6

Status: failed
Started: 2026-05-18T01:53:58Z
Ended: 2026-05-18T01:53:58Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-5 revisions all landed in the current draft: (1) AdmissionDecision/SandboxAdmissionDecision/GraphScopeAdmissionDecision carry `reasons: Vec<String>` with call-site `format!()` strings, matching the `state_machine::fanout::format!(...)` precedent at crates/runx-core/src/state_machine/fanout.rs:53; (2) the dedup wording now explicitly scopes the BTreeSet to a function-local membership tracker that never crosses a serialized boundary, killing the literal-reading conflict with fixtures/kernel/README.md:44-45; (3) the Phase 1 serde round-trip test now covers AdmissionDecision::Allow/Deny in addition to GraphScopeAdmissionDecision::Allow (camelCase grantId) and SandboxAdmissionDecision::ApprovalRequired (snake_case override). Path/command/scope/timing/rollback audits all pass against the live repo. Two minor follow-up items remain worth pinning before approval, both low-severity: (a) the spec never says where `ConnectedAuthRequirement` (the struct returned by the in-scope `connected_auth_requirement`) lives \u2014 `policy::types` as `pub(crate)` or co-located in `policy::connected_auth` \u2014 leaving the implementer to choose; (b) the in-scope types bullet (line 127-131) omits `RequiredSandboxDeclaration`, which is the return type of `normalize_sandbox_declaration`. Both are pinnable in one sentence each; neither is a blocker for approval, but they are real spec gaps.

Checks:
- path audit
  - Grounded in: code:crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: All Rust artifact paths (crates/runx-core/src/policy.rs, src/policy/{types,posix_basename,local,retry,graph_scope,interpreter,scope,sandbox,connected_auth}.rs, tests/policy_fixtures.rs, tests/policy_proptest.rs) are intentional future files. Glob on crates/runx-core/** confirms only state_machine/ and serde_conventions.rs exist today. lib.rs has `pub mod serde_conventions; pub mod state_machine;` only. Phase 1 plan to make policy.rs a thin module root declaring `types` and `posix_basename` mirrors crates/runx-core/src/state_machine.rs:1-17. Prerequisite paths verified: scripts/check-rust-core-style.mjs:141 (checkStateMachineFixtureCoverage short-circuits on missing testFile at lines 144-146), packages/core/src/policy/index.ts (no node:path import; uses ./posix-basename.js at line 5), packages/core/src/policy/{sandbox,authority-proof,posix-basename}.ts, 20 fixtures/kernel/policy/*.json, docs/{rust-kernel-architecture,trusted-kernel-package-truth}.md, fixtures/kernel/README.md (line 44-45 carries the HashSet/BTreeSet-at-serialized-boundary rule the spec now reconciles), runx_contracts::JsonValue.
- command audit
  - Grounded in: code:crates/runx-core/Cargo.toml:18
  - Result: passed
  - Evidence: Every acceptance command is runnable. ac1_1 `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-run` compiles tests matching the filter (--no-fail-fast already dropped per round-2). ac1_2/ac1_5 filter in-module test suites (`posix_basename`, `policy::types`). ac2_1/ac3_1 use --test names mapping to crates/runx-core/tests/ integration tests. v4 ripgrep `! rg 'std::fs|std::process|std::net|std::env|std::time::SystemTime|std::path::Path|tokio|reqwest|ureq|hyper|rmcp|Command::new' crates/runx-core/src crates/runx-core/tests` is consistent with current runx-core deps (serde 1.0.228, serde_json 1.0.149, runx-contracts). v5 cargo-deny is real (crates/deny.toml exists). v7 invokes scripts/check-rust-crate-graph.mjs and scripts/check-rust-core-style.mjs (both exist). proptest dev-dep at Cargo.toml:32. ac2_5 asserts the script TEXT references `checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs`, blocking a no-op stub from satisfying the gate.
- scope/migration audit
  - Grounded in: code:packages/core/src/policy/index.ts:136
  - Result: passed
  - Evidence: index.ts:136 calls `connectedAuthRequirement(skill.auth)` from authority-proof.ts:68. The spec defers the PUBLIC authority-proof surface (5 functions, 6 types per the corrected round-3 count) to rust-policy-authority-proof-parity, but ports the needed internal helpers (connected_auth_requirement, find_matching_grant, grant_reference_matches, has_grant_reference) as `pub(crate)` inside a new policy::connected_auth module not re-exported from runx_core::policy. The connected-grant fixture (local-admission-allows-connected-wildcard-grant.json:27-33 with auth.type='nango') and the graph-scope grant_id fixture (graph-scope-allows-exact-match.json:24 with snake_case `grant_id` input, line 6 with camelCase `grantId` output) drive the asymmetric serde rename strategy the spec now pins per struct. SandboxAdmissionDecision::ApprovalRequired's `#[serde(rename = "approval_required")]` override matches fixture sandbox-requires-unrestricted-approval.json:9 which emits snake_case `status: "approval_required"`.
- acceptance timing audit
  - Grounded in: code:scripts/check-rust-core-style.mjs:144
  - Result: passed
  - Evidence: Phase 1 ac1_4 (`node scripts/check-rust-core-style.mjs`) passes because the planned policy coverage check will short-circuit when tests/policy_fixtures.rs does not yet exist (mirror of checkStateMachineFixtureCoverage:144-146 returning early if testFile is missing). Phase 2 ac2_5 hard-asserts the script text references `checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs`, blocking accidental no-op. Phase 4 ac4_1 now requires three exact phrases across docs/trusted-kernel-package-truth.md, fixtures/kernel/README.md, and docs/rust-kernel-architecture.md; ac4_2 requires the deferred-follow-up phrase in two files. Grep across docs/ and fixtures/kernel/ confirms none of `Rust policy parity status: fixture-evidence-only`, `Rust policy fixtures are policy parity evidence`, `runx-core policy parity is not runtime-authoritative`, or `authority-proof.*public-work.*deferred|authority-proof and public-work.*follow-up` are present in target files today, so the gates are non-vacuous. Phase 3 `ProptestConfig::with_cases(64)` keeps the 60s cap meaningful as four assertions land.
- rollback/repair audit
  - Grounded in: code:crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: Rollback is credible because every Rust artifact (crates/runx-core/src/policy.rs, src/policy/, tests/policy_fixtures.rs, tests/policy_proptest.rs) is net-new and reversible by rm. lib.rs currently has only `pub mod serde_conventions;` and `pub mod state_machine;`, so reverting the new `pub mod policy;` is the explicitly-called-out one-line revert. Script and doc edits are git-revertable. Per-phase rollback matches the additive shape: each phase only adds files plus a one-line lib.rs/script/doc edit, so a failed phase can be undone without disturbing prior phases. Phase 4 doc edits are reversible because Phase 4 gates require exact phrases that don't currently exist anywhere in the target files.
- design challenge
  - Grounded in: code:.scafld/specs/archive/2026-05/rust-state-machine-parity.md
  - Result: passed
  - Evidence: Right architectural move, not bandaid. The plan mirrors archived rust-state-machine-parity one-for-one: small modules, named re-exports from a thin module root, fixture parity, proptest determinism with ProptestConfig::with_cases(64), no runtime APIs, hand-rolled ASCII helpers instead of new dependencies. Arch doc section 18 (lines 476-499) sets the quality bar this spec defers to; arch doc section 14 (Placeholder publishing strategy, line 428) is the correct edit target for the Phase 4 placeholder-status update. Deferred authority-proof and public-work surface is bounded by a real follow-up draft at .scafld/specs/drafts/rust-policy-authority-proof-parity.md. The private policy::connected_auth carve-out is the minimum needed to admit the in-scope connected-grant fixture without widening the deferred public authority-proof API. Bounded blast radius: Rust policy is not runtime-authoritative (Phase 4 documents this explicitly in three docs), TypeScript remains the source of truth, and rollback is rm-and-revert. Estimated 14h is consistent with state-machine parity precedent at similar scope.

Questions:
- Where should the `ConnectedAuthRequirement` struct live in the Rust port: in `policy::types` (alongside LocalAdmissionGrant et al.) as `pub(crate)`, or co-located in `policy::connected_auth` next to the function that returns it?
  - Grounded in: code:packages/core/src/policy/authority-proof.ts:12
  - Recommended answer: Co-locate `ConnectedAuthRequirement` in `policy::connected_auth` as a `pub(crate)` struct. It is internal scaffolding for `admit_local_skill` parity (never re-exported from `runx_core::policy`) and lives alongside `connected_auth_requirement`, `find_matching_grant`, `grant_reference_matches`, `has_grant_reference`. Add a one-sentence note to the Phase 2 `policy/connected_auth.rs` change description that the struct uses `#[serde(rename_all = "snake_case")]` (its `scope_family`, `authority_kind`, `target_repo`, `target_locator` fields are snake_case in the TS shape and on grants). This keeps `policy::types` focused on the publicly-named admission types and pins the visibility/colocation choice instead of leaving it to an implementer.
  - If unanswered: Pin `ConnectedAuthRequirement` as a `pub(crate)` struct inside `policy::connected_auth` with `#[serde(rename_all = "snake_case")]`; note it is internal-only and never re-exported.
- Should `RequiredSandboxDeclaration` (the return type of `normalize_sandbox_declaration`) be listed in the Scope > types bullet so the public surface is fully enumerated?
  - Grounded in: code:packages/core/src/policy/sandbox.ts:37
  - Recommended answer: Yes. `sandbox.ts:37-44` defines `RequiredSandboxDeclaration` as the canonical normalized shape returned by `normalizeSandboxDeclaration` (and the input to `admitSandbox`'s internal flow). The Rust `normalize_sandbox_declaration` must return a typed struct, not a tuple or `SandboxDeclaration` (which has all-optional fields). Add `RequiredSandboxDeclaration` to the in-scope types bullet at spec line 127-131. Phase 1 types.rs should declare it with `#[serde(rename_all = "camelCase")]` and `#[serde(skip_serializing_if = "Option::is_none")]` on `envAllowlist`.
  - If unanswered: Add `RequiredSandboxDeclaration` to the in-scope type list and pin its serde shape in the Phase 1 types.rs change description.

Design objections:
- `objection-1` low - `ConnectedAuthRequirement` struct location and visibility are not pinned.
  - Grounded in: code:packages/core/src/policy/authority-proof.ts:12
  - Evidence: The TS `ConnectedAuthRequirement` interface at `packages/core/src/policy/authority-proof.ts:12-19` is the return type of the in-scope `connectedAuthRequirement(auth)` helper. The spec lists `connected_auth_requirement`, `find_matching_grant`, `grant_reference_matches`, `has_grant_reference` as `pub(crate)` helpers inside `policy::connected_auth` (Phase 2 changes), but never says where `ConnectedAuthRequirement` itself lives or what its visibility/serde shape should be. Its fields (`scope_family`, `authority_kind`, `target_repo`, `target_locator`) are snake_case in TS and on grants; the per-struct snake_case rename rule the spec pins for `LocalAdmissionGrant`/`GraphScopeGrant` should extend to it too. Without an explicit location, an implementer may place it in `policy::types` (widening that file beyond the publicly-named admission types) or fabricate a `pub` export by accident.
  - Recommendation: Extend the Phase 2 `policy/connected_auth.rs` change description: `pub(crate) struct ConnectedAuthRequirement` lives in `policy::connected_auth`, uses `#[serde(rename_all = "snake_case")]` for the targeting fields, uses `#[serde(skip_serializing_if = "Option::is_none")]` on the optional fields, and is never re-exported from `runx_core::policy`. Mention this in the spec so the visibility and location are pinned, not author-judged.
- `objection-2` low - `RequiredSandboxDeclaration` is missing from the in-scope types list.
  - Grounded in: code:packages/core/src/policy/sandbox.ts:37
  - Evidence: `packages/core/src/policy/sandbox.ts:37-44` defines `RequiredSandboxDeclaration` as the canonical normalized shape returned by `normalizeSandboxDeclaration` and consumed by `admitSandbox`. The spec's in-scope types bullet (lines 127-131) lists `SandboxProfile`, `SandboxDeclaration`, `SandboxAdmissionDecision` but omits `RequiredSandboxDeclaration`. The Rust port needs a concrete return type for `normalize_sandbox_declaration`, almost certainly a public struct with camelCase serde and `skip_serializing_if` on `envAllowlist`, but the spec leaves whether it's public, internal, or simply a tuple to implementer judgment. Round-trip parity against fixtures sandbox-normalize-defaults.json depends on the chosen shape.
  - Recommendation: Add `RequiredSandboxDeclaration` to the in-scope types bullet at spec line 127-131 and pin in the Phase 1 types.rs change description that it uses `#[serde(rename_all = "camelCase")]` and `#[serde(skip_serializing_if = "Option::is_none")]` on `envAllowlist` to match the optional-undefined-omitted convention from `fixtures/kernel/README.md:47-49`.

Recommended edits:
- Phase 2: Admission parity > Changes (connected_auth.rs)
  - Grounded in: code:packages/core/src/policy/authority-proof.ts:12
  - Recommendation: Extend the Phase 2 `policy/connected_auth.rs` change description to include the location and shape of `ConnectedAuthRequirement`: `pub(crate) struct ConnectedAuthRequirement` lives in `policy::connected_auth` with `#[serde(rename_all = "snake_case")]` (its `scope_family`, `authority_kind`, `target_repo`, `target_locator` fields are snake_case) and `#[serde(skip_serializing_if = "Option::is_none")]` on the optional fields. It is never re-exported from `runx_core::policy`.
- Scope > In-scope types list (line 127-131) and Phase 1: Policy data model > Changes (types.rs)
  - Grounded in: code:packages/core/src/policy/sandbox.ts:37
  - Recommendation: Add `RequiredSandboxDeclaration` to the in-scope types bullet alongside `SandboxProfile`, `SandboxDeclaration`, `SandboxAdmissionDecision`. In the Phase 1 types.rs change description, pin that `RequiredSandboxDeclaration` uses the default camelCase rename and `#[serde(skip_serializing_if = "Option::is_none")]` on `envAllowlist`, matching the TS shape and the fixture undefined-omission convention from fixtures/kernel/README.md:47-49.

### round-7

Status: failed
Started: 2026-05-18T02:01:07Z
Ended: 2026-05-18T02:01:07Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-6 revisions all landed: ConnectedAuthRequirement is pinned as a `pub(crate)` struct co-located in `policy::connected_auth` with `#[serde(rename_all = "snake_case")]` + `skip_serializing_if`, and `RequiredSandboxDeclaration` is now in the in-scope types list with its camelCase + `skip_serializing_if` on `envAllowlist` shape pinned in Phase 1 types.rs. Path/command/scope/timing/rollback audits all pass against the live repo (lib.rs still has only `serde_conventions`+`state_machine`; all 20 `fixtures/kernel/policy/*.json` exist; `checkStateMachineFixtureCoverage` short-circuit pattern is the mirror target at scripts/check-rust-core-style.mjs:144; `runx_contracts::JsonValue` is real at crates/runx-contracts/src/lib.rs:13; state_machine.rs uses private `mod` + `pub use ...` named re-exports). Three low-severity gaps remain worth pinning before approval: (1) Phase 1 says `policy.rs` "performs named re-exports for those two modules" (`types` and `posix_basename`); under the state-machine precedent (`crates/runx-core/src/state_machine.rs:1-17` uses private `mod` + `pub use`) an implementer following the literal "named re-exports for ... posix_basename" wording would write `pub use posix_basename::posix_basename;`, widening the public `runx_core::policy` API beyond what TS exposes (TS does NOT re-export posix-basename from `policy/index.ts`). The spec should pin posix_basename as `pub(crate)`-only, mirroring the connected_auth/scope/interpreter visibility decisions already pinned. (2) `LocalAdmissionGrant.status` is `"active" | "revoked"` (index.ts:49) and the Rust port must invent an enum name and rename strategy; the spec lists `LocalAdmissionGrant` but never pins `LocalAdmissionGrantStatus` or its `#[serde(rename_all = "snake_case")]` (single-word values, but the rule should be explicit). (3) `LocalAdmissionSandbox` (index.ts:22) and `SandboxDeclaration` (sandbox.ts:3) are structurally identical except the inline `profile` literal union vs `SandboxProfile`; the spec lists both as in-scope types but never says whether the Rust port collapses them into one struct or keeps two byte-identical structs side by side. The Context invariant explicitly allows Rust to deviate from TS module/helper shape when cleaner; without pinning, the implementer's choice could either duplicate types or silently widen the public surface.

Checks:
- path audit
  - Grounded in: code:crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: All declared Rust artifact paths (crates/runx-core/src/policy.rs, src/policy/{types,posix_basename,local,retry,graph_scope,interpreter,scope,sandbox,connected_auth}.rs, tests/policy_fixtures.rs, tests/policy_proptest.rs) are intentional future files. Glob on crates/runx-core/** confirms only state_machine + serde_conventions exist today; lib.rs lines 1-7 carry only `pub mod serde_conventions;` and `pub mod state_machine;`. Phase 1's restriction that policy.rs declares only `types` and `posix_basename` mirrors the precedent at crates/runx-core/src/state_machine.rs:1-17. All prerequisite paths verified: scripts/check-rust-core-style.mjs:141 (checkStateMachineFixtureCoverage short-circuits on missing fixtureDirectory or testFile at lines 144-146), packages/core/src/policy/index.ts (no node:path; imports `./posix-basename.js` at line 5), packages/core/src/policy/{sandbox,authority-proof,posix-basename}.ts, 20 fixtures/kernel/policy/*.json files (including the 4 sandbox + 5 local-admission + 2 retry + 9 graph-scope cases the spec relies on), docs/{rust-kernel-architecture,trusted-kernel-package-truth}.md, fixtures/kernel/README.md, runx_contracts::JsonValue exposed at crates/runx-contracts/src/lib.rs:13.
- command audit
  - Grounded in: code:crates/runx-core/Cargo.toml:18
  - Result: passed
  - Evidence: Every acceptance command is runnable against the current workspace once the policy modules exist. ac1_1 `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-run` compiles tests without running them (round-2 cleanup applied). ac1_2 and ac1_5 (`cargo test -p runx-core posix_basename` / `policy::types`) use module-filter test runs that match in-module #[cfg(test)] suites. ac2_1/ac3_1 use --test names that map to integration tests under crates/runx-core/tests/. v4 ripgrep `! rg 'std::fs|std::process|std::net|std::env|std::time::SystemTime|std::path::Path|tokio|reqwest|ureq|hyper|rmcp|Command::new' crates/runx-core/{src,tests}` is consistent with current runx-core deps (serde 1.0.228, serde_json 1.0.149, runx-contracts; proptest dev-dep at Cargo.toml:32). v5 cargo-deny runs against crates/Cargo.toml. v7 invokes scripts/check-rust-crate-graph.mjs and scripts/check-rust-core-style.mjs (both exist). ac2_5 asserts the script TEXT references `checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs`, blocking a no-op stub from satisfying the gate.
- scope/migration audit
  - Grounded in: code:packages/core/src/policy/index.ts:136
  - Result: passed
  - Evidence: Round-2 carve-out remains correct: index.ts:136 calls `connectedAuthRequirement(skill.auth)` from authority-proof.ts:68. The spec defers the PUBLIC authority-proof surface (5 functions, 6 types, count corrected in round-3 to match index.ts:371-383) and the public-work surface (4 names, 5 types, verified via Grep against public-work.ts: PublicWorkPolicy/PublicPullRequestCandidateRequest/PublicCommentOpportunityRequest/PublicPolicyDecision/PublicCommentPolicyDecision types and DEFAULT_PUBLIC_WORK_POLICY/evaluatePublicPullRequestCandidate/evaluatePublicCommentOpportunity/normalizePublicWorkPolicy values) to rust-policy-authority-proof-parity. Internal helpers required by admit_local_skill are ported as `pub(crate)` inside a new policy::connected_auth module with `ConnectedAuthRequirement` co-located, not re-exported from runx_core::policy. The snake_case rename strategy on LocalAdmissionGrant/GraphScopeGrant matches fixture JSON at graph-scope-allows-exact-match.json:24 (`grant_id`) and local-admission-allows-connected-wildcard-grant.json:15-23, while GraphScopeAdmissionDecision keeps camelCase `grantId` at the same fixture's line 6. SandboxAdmissionDecision::ApprovalRequired's `#[serde(rename = "approval_required")]` override matches sandbox-requires-unrestricted-approval.json:9 which emits the snake_case status string `approval_required` (not kebab-case `approval-required`). Path-safety segment walking and inline-interpreter regex porting are both pinned to hand-rolled ASCII helpers in Dependencies block, so no `regex` or `indexmap` dep is added.
- acceptance timing audit
  - Grounded in: code:scripts/check-rust-core-style.mjs:144
  - Result: passed
  - Evidence: Phase 1 ac1_4 (`node scripts/check-rust-core-style.mjs`) passes because the planned policy coverage check mirrors checkStateMachineFixtureCoverage:144-146 (early return when fixtureDirectory or testFile is missing). Phase 2 ac2_5 hard-asserts via `rg` that the script SOURCE references `checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs`, so the gate cannot be silently no-op-satisfied. Phase 4 ac4_1 now requires three exact phrases across docs/trusted-kernel-package-truth.md (`Rust policy parity status: fixture-evidence-only`), fixtures/kernel/README.md (`Rust policy fixtures are policy parity evidence`), and docs/rust-kernel-architecture.md (`runx-core policy parity is not runtime-authoritative`); ac4_2 requires `authority-proof and public-work.*follow-up|authority-proof.*public-work.*deferred` in both trusted-kernel-package-truth.md and fixtures/kernel/README.md, and both Phase 4 Change descriptions promise it. Grep confirms none of the five anchor phrases are present in the target files today, so all gates are non-vacuous. Phase 3 commits to `ProptestConfig::with_cases(64)` matching state_machine_proptest.rs:16, keeping the 60s cap meaningful as four property assertions land.
- rollback/repair audit
  - Grounded in: code:crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: Rollback is credible because every Rust artifact (crates/runx-core/src/policy.rs, src/policy/, tests/policy_fixtures.rs, tests/policy_proptest.rs) is net-new and reversible by `rm`. lib.rs currently has only `pub mod serde_conventions;` and `pub mod state_machine;` (lines 1-7 in repo today), so reverting the new `pub mod policy;` export is the explicitly-called-out one-line revert. Script edits (`checkPolicyFixtureCoverage`) and doc edits (Phase 4 anchor phrases) are reversible via `git checkout`. Per-phase rollback matches the additive shape: each phase only adds new files plus a one-line lib.rs/script/doc edit, so a failed phase can be undone without disturbing prior phases. Phase 4 doc edits are reversible because all three Phase 4 gate phrases are absent from the target files today (confirmed by Grep), so partial-state recovery is well-defined.
- design challenge
  - Grounded in: code:.scafld/specs/archive/2026-05/rust-state-machine-parity.md
  - Result: passed
  - Evidence: Right architectural move, not bandaid or future-bloat. The plan mirrors archived rust-state-machine-parity one-for-one: small modules under a thin module root, named `pub use` re-exports (state_machine.rs:6-17 precedent), fixture parity, proptest determinism with `ProptestConfig::with_cases(64)`, no runtime APIs, hand-rolled ASCII helpers instead of new dependencies, `runx_contracts::JsonValue` for unknown-shaped fields. Arch doc section 18 (lines 476-499) sets the quality bar; section 14 (Placeholder publishing strategy, line 428) is the correct edit target for the Phase 4 placeholder-status update, which already carries the analogous state-machine-parity line at lines 438-440. The deferred authority-proof and public-work surface is bounded by a real follow-up draft at `.scafld/specs/drafts/rust-policy-authority-proof-parity.md`. The private `policy::connected_auth` carve-out is the minimum needed to admit the in-scope connected-grant fixture without widening the deferred public authority-proof API. Bounded blast radius: Rust policy is not runtime-authoritative (Phase 4 documents this explicitly in three docs), TypeScript remains the source of truth, rollback is rm-and-revert. Estimated 14h is consistent with the state-machine parity precedent at similar scope.

Questions:
- Should the Phase 1 `policy.rs` change description pin `posix_basename` as `pub(crate)`-only (not re-exported from `runx_core::policy`), so the literal reading of 'performs named re-exports for those two modules' does not lead an implementer to widen the public API with `pub use posix_basename::posix_basename;`?
  - Grounded in: code:crates/runx-core/src/state_machine.rs:1
  - Recommended answer: Yes. Update the Phase 1 `policy.rs` change description to: 'declares only `types` and `posix_basename` in Phase 1 and performs named `pub use` re-exports from `types` only; `posix_basename` is declared with private `mod posix_basename;` and the helper itself is `pub(crate) fn posix_basename(...)` consumed by `policy::interpreter` (Phase 2), matching how TS does NOT re-export `posix-basename` from `packages/core/src/policy/index.ts`.' This keeps the public surface aligned with TS, matches the visibility decisions already pinned for `policy::scope::scope_allows` and `policy::connected_auth::*`, and prevents an accidentally-public helper.
  - If unanswered: Pin `posix_basename` as `pub(crate)` consumed only by `policy::interpreter`; only `types` items are re-exported from `policy.rs`.
- Should the Phase 1 `types.rs` change description pin a name and serde rename strategy for the `LocalAdmissionGrant.status` field's enum (`'active' | 'revoked'`), so the implementer does not have to invent the type name or assume the rename rule?
  - Grounded in: code:packages/core/src/policy/index.ts:49
  - Recommended answer: Yes. Add to the Phase 1 `types.rs` change description: `LocalAdmissionGrant.status` is `Option<LocalAdmissionGrantStatus>` with `#[serde(skip_serializing_if = "Option::is_none")]` where `LocalAdmissionGrantStatus` is an enum with `Active` and `Revoked` variants and `#[serde(rename_all = "snake_case")]`. Although both values are single words and survive the default kebab-case rule, an explicit per-enum rename declaration prevents future misalignment if the TS union ever adds a multi-word state (e.g., `'pending_review'`).
  - If unanswered: Name the enum `LocalAdmissionGrantStatus` with `#[serde(rename_all = "snake_case")]` and `Active`/`Revoked` variants; field is `Option<LocalAdmissionGrantStatus>` with `skip_serializing_if`.
- Should the spec pin whether the Rust port keeps `LocalAdmissionSandbox` and `SandboxDeclaration` as two separate (structurally identical) types, or collapses them into a single `SandboxDeclaration` referenced from `LocalAdmissionSkill.source.sandbox`?
  - Grounded in: code:packages/core/src/policy/index.ts:22
  - Recommended answer: Collapse them. The TS shapes at `index.ts:22-29` and `sandbox.ts:3-10` differ only in the inline `profile` literal union vs the named `SandboxProfile`; the Rust port can use `Option<SandboxDeclaration>` on `LocalAdmissionSkill.source.sandbox` with no observable parity loss. The Context invariant (line 73-76) explicitly allows Rust to deviate from TS module/helper shape when cleaner. Add to the Phase 1 `types.rs` description: 'The Rust port uses a single `SandboxDeclaration` struct for both the standalone sandbox API and `LocalAdmissionSkill.source.sandbox`; `LocalAdmissionSandbox` is not introduced as a separate Rust type.' Update the in-scope types bullet (line 127-131) to drop `LocalAdmissionSandbox`.
  - If unanswered: Use a single `SandboxDeclaration` struct for both surfaces and drop `LocalAdmissionSandbox` from the in-scope types list.

Design objections:
- `objection-1` low - Phase 1 `policy.rs` wording leaves `posix_basename` visibility ambiguous; the literal reading widens the public API.
  - Grounded in: code:crates/runx-core/src/state_machine.rs:1
  - Evidence: Phase 1 (spec line 338-341) says `policy.rs` 'declares only `types` and `posix_basename` in Phase 1 and performs named re-exports for those two modules.' The state-machine precedent at `crates/runx-core/src/state_machine.rs:1-17` uses `mod fanout; ... pub use fanout::{...};`, meaning named `pub use` re-exports of items from each submodule. Applied literally to `posix_basename` (which has a single function), the implementer would write `pub use posix_basename::posix_basename;`, exposing the helper in the public `runx_core::policy` API. TypeScript explicitly does NOT re-export `posix-basename` from `packages/core/src/policy/index.ts` (the import at index.ts:5 is internal-only; the only consumer is `normalizeExecutableName` for interpreter detection). The spec also already pins `policy::scope::scope_allows`, `policy::connected_auth::*`, and `policy::interpreter` helpers as `pub(crate)`-only, so `posix_basename` should follow the same rule. Without an explicit visibility pin, the public API silently grows by one helper that has no fixture or external consumer.
  - Recommendation: Tighten the Phase 1 `policy.rs` change description to: 'declares only `types` and `posix_basename` in Phase 1 and performs named `pub use` re-exports of items from `types` only; `posix_basename` itself is `pub(crate) fn posix_basename(...)` inside a private `mod posix_basename;` and is not re-exported from `runx_core::policy`. This mirrors the TS convention where `posix-basename.ts` is internal scaffolding for `policy/index.ts` and not part of the package public surface.'
- `objection-2` low - `LocalAdmissionGrant.status` enum (the `'active' | 'revoked'` union) is not named or serde-pinned in the spec.
  - Grounded in: code:packages/core/src/policy/index.ts:49
  - Evidence: TS `LocalAdmissionGrant.status?: 'active' | 'revoked'` at `packages/core/src/policy/index.ts:49` requires a Rust enum that the spec never names or pins. The Phase 1 `types.rs` change description pins `#[serde(rename_all = "snake_case")]` on the `LocalAdmissionGrant` struct fields (lines 354-359) but does not extend that to the inline-string-union value type. An implementer will need to invent the enum name (`LocalAdmissionGrantStatus`? `GrantStatus`? `LocalGrantStatus`?), decide the rename strategy (default? `snake_case`? `kebab-case`?), and decide whether to share the type with the deferred `AuthorityProofGrant.status` (same shape at `policy/authority-proof.ts:25`). Both `'active'` and `'revoked'` are single-word values so multiple rename strategies happen to produce the same wire shape today, but a future addition like `'pending_review'` would diverge. The Phase 1 serde round-trip test would not catch this asymmetry because the status field is optional and untested for the variant case.
  - Recommendation: Add one sentence to the Phase 1 `types.rs` change description: '`LocalAdmissionGrant.status` is `Option<LocalAdmissionGrantStatus>` with `#[serde(skip_serializing_if = "Option::is_none")]` where `LocalAdmissionGrantStatus` is an enum with `Active` and `Revoked` variants under `#[serde(rename_all = "snake_case")]`. This type is not shared with the deferred `AuthorityProofGrant`; the follow-up `rust-policy-authority-proof-parity` spec decides whether to share or duplicate the enum.'
- `objection-3` low - `LocalAdmissionSandbox` and `SandboxDeclaration` are structurally identical; the spec lists both as in-scope types without pinning whether the Rust port unifies them.
  - Grounded in: code:packages/core/src/policy/index.ts:22
  - Evidence: TS `LocalAdmissionSandbox` at `packages/core/src/policy/index.ts:22-29` and `SandboxDeclaration` at `packages/core/src/policy/sandbox.ts:3-10` differ only in the inline literal union vs the named `SandboxProfile`; field set, casing, and optionality are identical. `LocalAdmissionSkill.source.sandbox` consumes the former; `admitSandbox(sandbox: SandboxDeclaration | undefined, ...)` consumes the latter. The Rust port can either (a) define two byte-identical structs (mirroring TS literally) or (b) collapse to a single `SandboxDeclaration` and reuse it on `LocalAdmissionSkill.source.sandbox`. The spec invariant at line 73-76 explicitly allows Rust to deviate from TS shape when cleaner, but the in-scope types list at line 127-131 lists BOTH `LocalAdmissionSandbox` and `SandboxDeclaration` without naming the deviation, so an implementer following the list literally will create two structs and waste serde derives, while an implementer following the invariant will collapse to one and create a delta between the type list and the implementation.
  - Recommendation: Pin the choice in the Phase 1 `types.rs` change description: 'The Rust port uses a single `SandboxDeclaration` struct for both the standalone sandbox API and `LocalAdmissionSkill.source.sandbox: Option<SandboxDeclaration>`; `LocalAdmissionSandbox` is not introduced as a separate Rust type because the TS shapes are byte-identical apart from the inline union vs `SandboxProfile`.' Update the in-scope types bullet at spec line 127-131 to drop `LocalAdmissionSandbox`, keeping only `SandboxDeclaration` / `SandboxProfile` / `RequiredSandboxDeclaration` / `SandboxAdmissionDecision`.

Recommended edits:
- Phase 1: Policy data model > Changes (policy.rs)
  - Grounded in: code:crates/runx-core/src/state_machine.rs:1
  - Recommendation: Replace 'declares only `types` and `posix_basename` in Phase 1 and performs named re-exports for those two modules' with 'declares only `types` and `posix_basename` in Phase 1; performs named `pub use` re-exports of items from `types` only. `posix_basename` is declared with private `mod posix_basename;` and the helper itself is `pub(crate) fn posix_basename(...)` so it is consumable by `policy::interpreter` (Phase 2) but never re-exported from `runx_core::policy`, mirroring how TS keeps `posix-basename.ts` internal to the policy package.'
- Phase 1: Policy data model > Changes (types.rs)
  - Grounded in: code:packages/core/src/policy/index.ts:49
  - Recommendation: Extend the `types.rs` change description with: '`LocalAdmissionGrant.status` is `Option<LocalAdmissionGrantStatus>` with `#[serde(skip_serializing_if = "Option::is_none")]`. `LocalAdmissionGrantStatus` is a `pub` enum with `Active` and `Revoked` variants under `#[serde(rename_all = "snake_case")]`. This Rust-only enum is not shared with the deferred `AuthorityProofGrant.status`; the follow-up spec decides whether to share.'
- Scope > In-scope types (line 127-131) and Phase 1 > Changes (types.rs)
  - Grounded in: code:packages/core/src/policy/index.ts:22
  - Recommendation: Drop `LocalAdmissionSandbox` from the in-scope types bullet at line 127-131 and add to the Phase 1 `types.rs` change description: 'The Rust port uses a single `SandboxDeclaration` struct for both the standalone sandbox API and `LocalAdmissionSkill.source.sandbox: Option<SandboxDeclaration>`; `LocalAdmissionSandbox` is not introduced as a separate Rust type. The TS duplicate exists for backward-compat module boundaries; Rust takes the cleaner shape under the spec's existing same-behavior-not-same-naming invariant (lines 73-76).'

### round-8

Status: failed
Started: 2026-05-18T02:10:16Z
Ended: 2026-05-18T02:10:16Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-7 revisions all landed cleanly: (1) `policy.rs` is explicitly restricted to publicly re-exporting types from the `types` module only, with `posix_basename` kept `pub(crate)` because TS does not re-export `posix-basename` from `policy/index.ts` either; (2) `LocalAdmissionGrant.status` is pinned to `Option<LocalAdmissionGrantStatus>` with `skip_serializing_if`, and `LocalAdmissionGrantStatus` is named explicitly as a public enum with `Active`/`Revoked` variants under `#[serde(rename_all = "snake_case")]`, with a note that the Rust enum is not pre-shared with the deferred `AuthorityProofGrant.status`; (3) `LocalAdmissionSandbox` is dropped from the in-scope types list and the Rust port uses a single `SandboxDeclaration` struct on both `admitSandbox` and `LocalAdmissionSkill.source.sandbox`. Path/command/scope/timing/rollback audits all pass against the live repo: lib.rs still carries only `serde_conventions` + `state_machine`; all 20 `fixtures/kernel/policy/*.json` files exist; `runx_contracts::JsonValue` is real at `crates/runx-contracts/src/lib.rs:13`; `checkStateMachineFixtureCoverage` at `scripts/check-rust-core-style.mjs:141` (short-circuits at lines 144-146 when test file is missing) is the proper mirror target; the `state_machine.rs` precedent of named `pub use` re-exports from private mods matches the Phase 1 `policy.rs` shape; the `policy::connected_auth` carve-out correctly bridges in-scope `admit_local_skill` to the deferred public authority-proof API without widening it; the `approval_required` snake_case override matches `sandbox-requires-unrestricted-approval.json:9`; the snake_case rename on `LocalAdmissionGrant`/`GraphScopeGrant` matches fixture inputs while the decision output `grantId` stays camelCase. Phase 4 anchor phrases are still absent from target docs, so all three Phase 4 gates remain non-vacuous. Two low-severity polish items remain worth pinning before approval, neither blocking but both easy one-sentence fixes: (a) the Risks section still references `serde_conventions.rs` for round-trip tests instead of the actual location `policy/types.rs`; (b) `skip_serializing_if = "Option::is_none"` is pinned on `LocalAdmissionGrant.status` but not on its other four optional fields (`scope_family`, `authority_kind`, `target_repo`, `target_locator`), creating an asymmetry with the convention already applied to `LocalAdmissionSkill.auth`, `RequiredSandboxDeclaration.envAllowlist`, and `ConnectedAuthRequirement` optional fields. Fix these two items and the spec is approval-ready.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: All Rust artifact paths declared in the spec (crates/runx-core/src/policy.rs, src/policy/{types,posix_basename,local,retry,graph_scope,interpreter,scope,sandbox,connected_auth}.rs, tests/policy_fixtures.rs, tests/policy_proptest.rs) are intentional future files. Glob on crates/runx-core/** confirms only state_machine subdir and serde_conventions.rs exist today; lib.rs lines 1-7 carry only `pub mod serde_conventions;` and `pub mod state_machine;`. All prerequisite paths verified: scripts/check-rust-core-style.mjs:141 (checkStateMachineFixtureCoverage short-circuits at lines 144-146 when fixtureDirectory or testFile is missing — the mirror target for the planned checkPolicyFixtureCoverage); packages/core/src/policy/{index,sandbox,authority-proof,posix-basename}.ts; 20 fixtures/kernel/policy/*.json including local-admission-allows-connected-wildcard-grant.json (auth.type='nango') and sandbox-requires-unrestricted-approval.json (status='approval_required'); docs/{rust-kernel-architecture,trusted-kernel-package-truth}.md; fixtures/kernel/README.md; runx_contracts::JsonValue at crates/runx-contracts/src/lib.rs:13. Phase 1 constraint that policy.rs declares only `types` and `posix_basename` mirrors the precedent at crates/runx-core/src/state_machine.rs:1-17.
- command audit
  - Grounded in: code:oss/crates/runx-core/Cargo.toml:18
  - Result: passed
  - Evidence: Every acceptance command is runnable. ac1_1 `cargo test --manifest-path crates/Cargo.toml -p runx-core policy --no-run` compiles tests matching the `policy` filter without running them (round-2 cleanup dropped --no-fail-fast). ac1_2 and ac1_5 use module-filter test runs that match in-module #[cfg(test)] suites. ac2_1/ac3_1 use --test names that map to integration tests under crates/runx-core/tests/. v4 ripgrep `! rg 'std::fs|std::process|std::net|std::env|std::time::SystemTime|std::path::Path|tokio|reqwest|ureq|hyper|rmcp|Command::new' crates/runx-core/{src,tests}` is consistent with current runx-core deps (serde 1.0.228, serde_json 1.0.149, runx-contracts at lines 19-21; proptest dev-dep at Cargo.toml:32). v5 cargo-deny is real (crates/deny.toml exists). v7 invokes scripts/check-rust-crate-graph.mjs and scripts/check-rust-core-style.mjs (both exist). ac2_5 hard-asserts that the script TEXT references `checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs`, blocking a no-op stub from satisfying the coverage gate.
- scope/migration audit
  - Grounded in: code:oss/packages/core/src/policy/index.ts:136
  - Result: passed
  - Evidence: Round-2 carve-out remains correct: index.ts:136 calls `connectedAuthRequirement(skill.auth)` from authority-proof.ts:68. The spec defers the PUBLIC authority-proof surface (5 functions, 6 types per index.ts:371-383) to rust-policy-authority-proof-parity, but ports the needed internal helpers (connected_auth_requirement, find_matching_grant, grant_reference_matches, has_grant_reference) as `pub(crate)` inside crates/runx-core/src/policy/connected_auth.rs with `ConnectedAuthRequirement` co-located (also `pub(crate)`, `#[serde(rename_all = "snake_case")]`), explicitly not re-exported from runx_core::policy. Snake_case rename strategy on LocalAdmissionGrant/GraphScopeGrant matches fixture JSON at graph-scope-allows-exact-match.json:24 (`grant_id`) and local-admission-allows-connected-wildcard-grant.json:14-23, while GraphScopeAdmissionDecision keeps camelCase `grantId` at the same fixture's line 6. SandboxAdmissionDecision::ApprovalRequired's `#[serde(rename = "approval_required")]` override matches sandbox-requires-unrestricted-approval.json:9. The collapsed SandboxDeclaration covers both standalone admitSandbox and LocalAdmissionSkill.source.sandbox per the 'same observable behavior, not same naming' invariant at line 73-76.
- acceptance timing audit
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:144
  - Result: passed
  - Evidence: Phase 1 ac1_4 passes because the planned policy coverage check will short-circuit when tests/policy_fixtures.rs does not yet exist (mirror of checkStateMachineFixtureCoverage:144-146). Phase 2 ac2_5 hard-asserts the script SOURCE references the coverage function and fixture paths. Phase 4 ac4_1 now requires three exact phrases across docs/trusted-kernel-package-truth.md, fixtures/kernel/README.md, and docs/rust-kernel-architecture.md; ac4_2 requires the deferred-follow-up phrase in two files, and both Phase 4 Change descriptions promise both phrases. Grep against docs/ and fixtures/kernel/ confirms none of the five anchor phrases are present in target files today, so all Phase 4 gates are non-vacuous. Phase 3 commits to `ProptestConfig::with_cases(64)` matching state_machine_proptest.rs:16, keeping the 60s cap meaningful as four property assertions land.
- rollback/repair audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:1
  - Result: passed
  - Evidence: Rollback is credible because every Rust artifact (crates/runx-core/src/policy.rs, src/policy/, tests/policy_fixtures.rs, tests/policy_proptest.rs) is net-new and reversible by `rm`. lib.rs currently has only `pub mod serde_conventions;` and `pub mod state_machine;`, so reverting the new `pub mod policy;` is the explicitly-called-out one-line revert. Script edits (`checkPolicyFixtureCoverage`) and Phase 4 doc edits (anchor phrases) are reversible via `git checkout`. Per-phase rollback matches the additive shape; each phase only adds files plus one-line lib.rs/script/doc edits, so a failed phase can be undone without disturbing prior phases. Phase 4 doc edits are reversible because all three Phase 4 gate phrases are absent from target files today, so partial-state recovery is well-defined.
- design challenge
  - Grounded in: code:oss/.scafld/specs/archive/2026-05/rust-state-machine-parity.md
  - Result: passed
  - Evidence: Right architectural move, not bandaid or future-bloat. The plan mirrors archived rust-state-machine-parity: small modules under a thin module root, named `pub use` re-exports, fixture parity, proptest determinism with `ProptestConfig::with_cases(64)`, no runtime APIs, hand-rolled ASCII helpers, `runx_contracts::JsonValue` for unknown-shaped fields. Arch doc section 18 sets the quality bar; section 14 is the correct edit target for the Phase 4 placeholder-status update. The deferred authority-proof and public-work surface is bounded by a real follow-up draft. The private `policy::connected_auth` carve-out is the minimum needed to admit the connected-grant fixture without widening the deferred public API. Bounded blast radius: Rust policy is not runtime-authoritative (Phase 4 documents this), TypeScript remains the source of truth, rollback is rm-and-revert. Estimated 14h is consistent with state-machine parity precedent.

Questions:
- Should the Risks section's enum-serde mitigation reference be updated to point at `policy/types.rs` instead of `serde_conventions.rs`, since the Phase 1 round-trip tests now live in `policy/types.rs`?
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:213
  - Recommended answer: Yes. Change the Risks block from 'Mitigated by round-trip tests in `serde_conventions.rs`' to 'Mitigated by the in-module serde round-trip tests in `crates/runx-core/src/policy/types.rs` required by Phase 1 (ac1_5).' This aligns the risk narrative with the actual test location and avoids future confusion if `serde_conventions.rs` is later removed or restructured.
  - If unanswered: Update the Risks line 211-213 mitigation reference to point at the Phase 1 in-module test in `policy/types.rs`.
- Should the Phase 1 `types.rs` change description enumerate `#[serde(skip_serializing_if = "Option::is_none")]` on `LocalAdmissionGrant`'s other optional fields (`scope_family`, `authority_kind`, `target_repo`, `target_locator`), not just `status`, to match the convention already pinned on `LocalAdmissionSkill.auth`, `RequiredSandboxDeclaration.envAllowlist`, and `ConnectedAuthRequirement` optional fields?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:45
  - Recommended answer: Yes. The fixture local-admission-allows-connected-wildcard-grant.json:15-23 carries `grant_id`, `provider`, `scopes`, `status` only; without `skip_serializing_if` on the other four optionals, a Rust round-trip would emit `"scope_family": null` etc. and break byte-identical re-serialization. These fields are input-only in current fixtures so deserialize-from-missing works, but pinning this now keeps the convention symmetric and prevents a future round-trip-asymmetry defect when more fixtures are added.
  - If unanswered: Extend the types.rs change description to apply `skip_serializing_if = "Option::is_none"` to all four other optional fields on `LocalAdmissionGrant`, not just `status`.

Design objections:
- `objection-1` low - Risks section references the stale `serde_conventions.rs` location for round-trip tests instead of `policy/types.rs`.
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:213
  - Evidence: Risks line 211-213 reads: 'serde rename strategies for enums can produce subtle JSON shape differences from TS. Mitigated by round-trip tests in `serde_conventions.rs`.' The Phase 1 `types.rs` change description (spec lines 345-385) explicitly places the in-module serde round-trip test in `policy/types.rs`, and ac1_5 runs `cargo test ... policy::types`. The `serde_conventions.rs` reference is a stale carry-over from earlier rounds. Not a correctness defect, but the risk narrative no longer matches the implementation plan and could confuse a future reader auditing test coverage.
  - Recommendation: Update Risks line 211-213 to: 'Mitigated by the in-module serde round-trip tests in `crates/runx-core/src/policy/types.rs` required by Phase 1 acceptance ac1_5.'
- `objection-2` low - `LocalAdmissionGrant`'s other optional fields are not pinned for `skip_serializing_if`, only `status` is.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:45
  - Evidence: TS `LocalAdmissionGrant` at `packages/core/src/policy/index.ts:45-54` carries four other optional fields (`scope_family`, `authority_kind`, `target_repo`, `target_locator`) in addition to `status`. The Phase 1 `types.rs` change description pins `skip_serializing_if = "Option::is_none"` on `LocalAdmissionGrant.status` (line 375-377), `LocalAdmissionSkill.auth` (line 348-350), `RequiredSandboxDeclaration.envAllowlist` (line 373-375), and `ConnectedAuthRequirement` optional fields (Phase 2 line 437-438), but does not enumerate it for the other four `LocalAdmissionGrant` optionals. The fixture `local-admission-allows-connected-wildcard-grant.json:15-23` omits those keys; without `skip_serializing_if` the Rust port would re-serialize them as `null`, breaking byte-identical roundtrip. The Phase 1 serde test only covers grant deserialization, so this would only surface if a later round-trip assertion is added. Low-severity because fixture inputs only require deserialization, not re-serialization.
  - Recommendation: Extend the Phase 1 `types.rs` change description: 'All optional fields on `LocalAdmissionGrant` (`status`, `scope_family`, `authority_kind`, `target_repo`, `target_locator`) carry `#[serde(skip_serializing_if = "Option::is_none")]`, mirroring the convention already pinned for `LocalAdmissionSkill.auth`, `RequiredSandboxDeclaration.envAllowlist`, and `ConnectedAuthRequirement`.'

Recommended edits:
- Risks
  - Grounded in: code:oss/.scafld/specs/drafts/rust-policy-parity.md:213
  - Recommendation: Update the enum-serde risk mitigation (line 211-213) from 'Mitigated by round-trip tests in `serde_conventions.rs`' to 'Mitigated by the in-module serde round-trip tests required in `crates/runx-core/src/policy/types.rs` (Phase 1 ac1_5).' The current text references a stale location; the round-trip tests now live alongside the types they cover.
- Phase 1: Policy data model > Changes (types.rs)
  - Grounded in: code:oss/packages/core/src/policy/index.ts:45
  - Recommendation: Append one sentence to the `types.rs` change description: 'All optional fields on `LocalAdmissionGrant` (`status`, `scope_family`, `authority_kind`, `target_repo`, `target_locator`) carry `#[serde(skip_serializing_if = "Option::is_none")]`, mirroring the convention already pinned for `LocalAdmissionSkill.auth`, `RequiredSandboxDeclaration.envAllowlist`, and `ConnectedAuthRequirement` optional fields.' This kills the asymmetry where only `status` is pinned.

### round-9

Status: failed
Started: 2026-05-18T02:18:47Z
Ended: 2026-05-18T02:18:47Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: The spec is well-grounded in the existing parity architecture and round-1 feedback was largely absorbed (fixture coverage script, anchor phrases, pub(crate) scope_allows, per-module layout, hand-rolled regex ports). Remaining concerns: (1) empty-array serialization for `requested_scopes`/`granted_scopes` on `GraphScopeAdmissionDecision` is implicitly assumed but not pinned — fixtures `graph-scope-allows-empty-request.json` and `graph-scope-denies-empty-grant.json` require `[]` to be emitted, so any defensive `skip_serializing_if = "Vec::is_empty"` would silently break parity; (2) insertion-preserving dedup remains unpinned despite `fixtures/kernel/README.md` mandating it and forbidding `HashSet`/`BTreeSet` at array boundaries — round-1 raised this and it is still open; (3) Phase 1 `ac1_4` (`node scripts/check-rust-core-style.mjs`) is a vacuous gate because the new `checkPolicyFixtureCoverage` will skip while `policy_fixtures.rs` doesn't yet exist — Phase 2 has an explicit `rg` for the function but Phase 1 does not; (4) the literal default `allowed_source_types` list and `max_timeout_seconds = 300` are not pinned in the spec, leaving room for drift; (5) the spec is silent about `hasGrantReference`'s JS-truthiness treatment of empty strings on optional targeting fields, which differs from naive Rust `Option::is_some`. Also worth a design note: `admit_local_skill` folds sandbox `ApprovalRequired` into a deny, losing the approval-required signal in the public `AdmissionDecision` shape.

Checks:
- path audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:7 and code:oss/packages/core/src/policy/index.ts:1
  - Result: passed
  - Evidence: lib.rs currently exposes serde_conventions + state_machine; all new policy paths (`crates/runx-core/src/policy.rs`, `policy/*.rs`, `tests/policy_fixtures.rs`, `tests/policy_proptest.rs`) are net-new and conflict-free. The companion TS sources cited in scope (policy/index.ts, sandbox.ts, posix-basename.ts, authority-proof.ts) exist. `fixtures/kernel/policy/*.json` contains 20 fixtures spanning local/sandbox/retry/graph-scope categories, matching the spec's fixture-coverage plan.
- command audit
  - Grounded in: code:oss/crates/runx-core/Cargo.toml:18 and code:oss/crates/Cargo.toml:11
  - Result: passed
  - Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-core ...` lines up with the existing workspace (members include runx-core, resolver = 3). `proptest = 1.11.0` is a dev-dependency already; Phase 3's `policy_proptest.rs` can reuse it. `cargo clippy ... -- -D warnings` is consistent with workspace clippy lints (workspace.lints.clippy denies unwrap_used/panic/dbg_macro/etc.). `pnpm exec vitest run --config vitest.config.ts ...` matches existing repo invocation style.
- scope/migration audit
  - Grounded in: code:oss/fixtures/kernel/README.md:42
  - Result: failed
  - Evidence: fixtures/kernel/README.md explicitly says 'Rust ports must use insertion-preserving deduplication for arrays such as requestedScopes, grantedScopes, stepIds, and contextFrom; do not use HashSet or BTreeSet at serialized array boundaries.' The spec lists graph-scope dedup as in-scope but does not pin the implementation strategy (Vec + in-function HashSet membership check, or a new `indexmap` dependency). Round-1 raised exactly this and the current draft still leaves it open.
- acceptance timing audit
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:141
  - Result: failed
  - Evidence: The existing `checkStateMachineFixtureCoverage` short-circuits when its test file is missing (`if (!(await exists(fixtureDirectory)) || !(await exists(testFile))) { return; }`). The new `checkPolicyFixtureCoverage` will follow the same pattern. In Phase 1, `crates/runx-core/tests/policy_fixtures.rs` does not yet exist (it is created in Phase 2), so Phase 1 ac1_4 (`node scripts/check-rust-core-style.mjs`) passes vacuously and does not prove the new policy-coverage code was even added to the script. Phase 2 ac2_5 covers this with an explicit `rg`, but Phase 1 has no equivalent.
- rollback/repair audit
  - Grounded in: code:oss/crates/runx-core/src/lib.rs:7
  - Result: passed
  - Evidence: Rollback removes net-new policy files (`crates/runx-core/src/policy.rs`, the `policy/` directory, both `tests/policy_*.rs`), reverts the `pub mod policy;` line in `lib.rs`, and reverts the script + docs edits. Each removal targets files that the spec itself introduces; existing `serde_conventions.rs` and `state_machine/` are untouched. The rollback is bounded and reversible.
- design challenge
  - Grounded in: code:oss/packages/core/src/policy/index.ts:127 and code:oss/fixtures/kernel/policy/graph-scope-allows-empty-request.json:12
  - Result: failed
  - Evidence: TS `admitLocalSkill` folds any non-allow sandbox decision (including `approval_required`) into the local-admission deny reasons, dropping the `approval_required` signal from the public `AdmissionDecision`. The spec mandates parity but never names this lossy conversion, so a Rust implementer could 'helpfully' surface `ApprovalRequired` on the outer decision and diverge. Separately, `graph-scope-allows-empty-request.json` requires `"requestedScopes": []` in the output, and `graph-scope-denies-empty-grant.json` requires `"grantedScopes": []`. The spec does not pin that the Rust `GraphScopeAdmissionDecision` MUST omit `skip_serializing_if = "Vec::is_empty"` on these fields, even though it pins the analogous rule for optional grant_id.

Questions:
- Should the spec explicitly forbid `skip_serializing_if = "Vec::is_empty"` on `GraphScopeAdmissionDecision.requested_scopes`/`granted_scopes` (and similarly for `LocalAdmissionGrant.scopes`)?
  - Grounded in: code:oss/fixtures/kernel/policy/graph-scope-denies-empty-grant.json:6
  - Recommended answer: Yes. Phase 1's `policy/types.rs` change should state that these `Vec<String>` fields use neither `skip_serializing_if` nor `default`, and the in-module serde test should round-trip an empty `requestedScopes` and an empty `grantedScopes` case so the rule is locked in.
  - If unanswered: Apply the no-skip rule to all `Vec<String>` fields on `GraphScopeAdmissionDecision`, `LocalAdmissionGrant`, and `RequiredSandboxDeclaration.writablePaths`, with serde round-trip tests pinning the behavior.
- What is the concrete implementation choice for insertion-preserving deduplication of `requestedScopes`/`grantedScopes` — in-function `HashSet` membership, or a new `indexmap` workspace dependency?
  - Grounded in: code:oss/fixtures/kernel/README.md:42
  - Recommended answer: Use a small private `unique_preserving_order(values: &[String]) -> Vec<String>` helper in `policy::scope` backed by a function-local `HashSet<&str>` for membership. HashSet is allowed because it never crosses a serde boundary. Avoids a new dep and keeps the style-script `HashMap` ban irrelevant (we never use HashMap, only HashSet inside a function).
  - If unanswered: Pin the in-function HashSet helper approach; do not add `indexmap`.
- Does Phase 1 need its own `rg` check on `scripts/check-rust-core-style.mjs` to prove the policy-coverage edit landed before Phase 2's test file exists?
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:141
  - Recommended answer: Yes. Add a Phase 1 acceptance criterion such as `rg -n 'checkPolicyFixtureCoverage|fixtures/kernel/policy' scripts/check-rust-core-style.mjs` so the Phase 1 gate fails closed if the script was not edited, even though the new function will short-circuit due to a missing test file.
  - If unanswered: Adopt the rg-based Phase 1 acceptance; ac1_4 as written is vacuous.
- Should `admit_local_skill`'s default `allowed_source_types` list be literally specified in the spec, or inherited by reading the TypeScript?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:102
  - Recommended answer: Pin it in the Phase 2 `policy/local.rs` change description, in order: `["agent", "agent-step", "approval", "cli-tool", "mcp", "a2a", "catalog", "graph"]`, plus `max_timeout_seconds = 300`. Drift on these constants would silently change parity without breaking the existing fixtures (which all use `cli-tool` or `mcp`).
  - If unanswered: Pin both literals in the Phase 2 change description.
- How should `has_grant_reference` handle deserialized empty-string targeting fields — treat `Some("")` as absent to match TS truthiness, or treat it as present?
  - Grounded in: code:oss/packages/core/src/policy/index.ts:351
  - Recommended answer: Treat `Some("")` as absent. TS uses `||` truthiness, so an empty-string `target_repo` is treated as missing. Encode this in a small private helper `is_targeted(field: &Option<String>) -> bool` that returns `false` for `None` and for `Some("")` alike, used by `has_grant_reference`. Add a unit test in `connected_auth.rs`.
  - If unanswered: Adopt the `is_targeted` helper with empty-string parity; document in Phase 2.
- Should Phase 4's `docs/trusted-kernel-package-truth.md` change also revise the existing 'does not yet implement policy parity' sentence so the doc is internally consistent after the edit?
  - Grounded in: code:oss/docs/trusted-kernel-package-truth.md:23
  - Recommended answer: Yes. Add a directive to the Phase 4 change description: 'Replace the existing "It is not an authoritative replacement for @runxhq/core/state-machine, and it does not yet implement policy parity." sentence with one that reads correctly with the new status phrase.'
  - If unanswered: Bundle the wording revision with the new status phrase so Phase 4 leaves the doc in a single coherent state.

Design objections:
- `objection-1` medium - Empty-array serialization rule for `requestedScopes`/`grantedScopes` is not pinned and two fixtures require `[]` to be emitted.
  - Grounded in: code:oss/fixtures/kernel/policy/graph-scope-allows-empty-request.json:12 and code:oss/fixtures/kernel/policy/graph-scope-denies-empty-grant.json:6
  - Evidence: graph-scope-allows-empty-request.json expects `"requestedScopes": []` in the decision; graph-scope-denies-empty-grant.json expects `"grantedScopes": []`. The spec pins `skip_serializing_if = "Option::is_none"` for optional fields (`grantId`, `LocalAdmissionSkill.auth`, `RequiredSandboxDeclaration.envAllowlist`, etc.) but never says 'do not apply `skip_serializing_if = "Vec::is_empty"` to these arrays'. A defensive implementer adding `Vec::is_empty` skip would break both fixtures.
  - Recommendation: Add an explicit invariant in Phase 1's `policy/types.rs` change description: `GraphScopeAdmissionDecision.requested_scopes` and `granted_scopes` are `Vec<String>` WITHOUT `skip_serializing_if`, so empty arrays serialize as `[]`. Add a matching assertion to the in-module serde test in Phase 1 covering an empty `requestedScopes`/`grantedScopes` round-trip.
- `objection-2` medium - Insertion-preserving deduplication for `requestedScopes`/`grantedScopes` remains unpinned despite a normative repo-wide rule.
  - Grounded in: code:oss/fixtures/kernel/README.md:42
  - Evidence: fixtures/kernel/README.md lines 42-45 say 'Rust ports must use insertion-preserving deduplication for arrays such as requestedScopes, grantedScopes, stepIds, and contextFrom; do not use HashSet or BTreeSet at serialized array boundaries.' The TS oracle uses `unique()` which preserves first-seen order. The spec lists these arrays as in-scope but never pins the Rust strategy (in-function HashSet membership over a Vec, or a new `indexmap` workspace dep). Round-1 explicitly raised this and the current draft has not closed it.
  - Recommendation: Pin the strategy in Phase 2 `policy/graph_scope.rs` change description: 'Deduplicates requested/granted scopes via a private `unique_preserving_order(&[String]) -> Vec<String>` helper backed by a local HashSet membership set (HashSet allowed inside function scope; never crosses a serde boundary).' Or, add `indexmap` to `crates/runx-core/Cargo.toml` and re-run `cargo deny check`. Either choice is fine; leaving it implicit is not.
- `objection-3` medium - Phase 1 `ac1_4` is a vacuous gate because `checkPolicyFixtureCoverage` will short-circuit before `policy_fixtures.rs` exists.
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:141
  - Evidence: The existing `checkStateMachineFixtureCoverage` skips its work via `if (!(await exists(fixtureDirectory)) || !(await exists(testFile))) { return; }`. The spec says the new policy check mirrors that pattern. Phase 1 doesn't create `crates/runx-core/tests/policy_fixtures.rs` (Phase 2 does), so the script will exit clean in Phase 1 regardless of whether the new policy-coverage code was actually added. The Phase 1 acceptance therefore does not verify the Phase 1 script edit.
  - Recommendation: Mirror Phase 2's ac2_5 in Phase 1: add an acceptance step like `rg -n 'checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs' scripts/check-rust-core-style.mjs` so Phase 1 proves the script was edited even when the test file is still absent.
- `objection-4` low - Default `allowed_source_types` list and `max_timeout_seconds = 300` for `admit_local_skill` are not literally pinned in the spec, inviting drift.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:102
  - Evidence: TS line 102: `allowedSourceTypes ?? ['agent', 'agent-step', 'approval', 'cli-tool', 'mcp', 'a2a', 'catalog', 'graph']` and `maxTimeoutSeconds ?? 300`. These are parity-critical constants, but the spec only names the functions, not the literal defaults. A Rust implementer who infers the defaults from the fixtures (which use camelCase keys, not the default list) could miss an entry.
  - Recommendation: In Phase 2's `policy/local.rs` change description, explicitly list the default `allowed_source_types` (the 8 strings, in the exact TS order) and the default `max_timeout_seconds = 300`, so any drift trips a code review rather than waiting for a fixture-less proptest run.
- `objection-5` low - `has_grant_reference` parity around empty-string targeting fields is not addressed.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:351
  - Evidence: TS `hasGrantReference` line 357: `Boolean(value.scope_family || value.authority_kind || value.target_repo || value.target_locator)` treats empty strings as absent (JS truthiness). Rust's `Option<String>` would deserialize `""` as `Some("")` and a naive `is_some()` check would treat it as present. The spec mandates `Option::is_none` skip-on-serialize, but doesn't address truthiness parity on the read side. Proptest in Phase 3 could surface this if its strategy emits empty strings.
  - Recommendation: Add a line to Phase 2's `policy/connected_auth.rs` change description: `has_grant_reference` and `grant_reference_matches` must treat `Some("")` as absent (mirroring TS `||` truthiness). Add one in-module unit test in `connected_auth.rs` that pins this behavior.
- `objection-6` low - `admit_local_skill` silently downgrades sandbox `ApprovalRequired` to `Deny` and the spec does not name this lossy conversion as required parity.
  - Grounded in: code:oss/packages/core/src/policy/index.ts:124
  - Evidence: TS lines 124-126: `if (sandboxDecision.status !== 'allow') { reasons.push(...sandboxDecision.reasons); }`. So when sandbox returns `approval_required`, the outer `AdmissionDecision` shape (which has only allow/deny) folds those reasons into a deny, dropping the `approval_required` signal. A Rust implementer reading the spec might 'fix' this by widening `AdmissionDecision` to include an `ApprovalRequired` variant and break parity.
  - Recommendation: Add a one-sentence parity note in Phase 2's `policy/local.rs` change description: 'Non-allow sandbox decisions (including ApprovalRequired) fold their reasons into the deny path. AdmissionDecision intentionally has only Allow/Deny; do not surface ApprovalRequired here.'
- `objection-7` low - Phase 4 adds new status phrases but does not require updating the existing 'does not yet implement policy parity' sentence, risking contradictory doc state.
  - Grounded in: code:oss/docs/trusted-kernel-package-truth.md:23
  - Evidence: Current `oss/docs/trusted-kernel-package-truth.md` line 23-25 reads 'crates/runx-core currently provides Rust state-machine parity only. It is not an authoritative replacement for @runxhq/core/state-machine, and it does not yet implement policy parity.' After Phase 4 lands the phrase 'Rust policy parity status: fixture-evidence-only', the surrounding sentence will still say 'does not yet implement policy parity', producing internally inconsistent docs.
  - Recommendation: Tighten Phase 4's `docs/trusted-kernel-package-truth.md` change description to explicitly revise the existing 'does not yet implement policy parity' wording while adding the new status phrase, so the doc states a single coherent position.

Recommended edits:
- phases.phase1.changes.crates/runx-core/src/policy/types.rs
  - Grounded in: code:oss/fixtures/kernel/policy/graph-scope-allows-empty-request.json:12
  - Recommendation: Add: 'GraphScopeAdmissionDecision.requested_scopes/granted_scopes and LocalAdmissionGrant.scopes are `Vec<String>` WITHOUT `skip_serializing_if`, so empty arrays serialize as `[]`. The in-module serde round-trip test covers an empty `requestedScopes` and an empty `grantedScopes` decision shape.'
- phases.phase1.acceptance
  - Grounded in: code:oss/scripts/check-rust-core-style.mjs:141
  - Recommendation: Add ac1_6: `rg -n 'checkPolicyFixtureCoverage|fixtures/kernel/policy|policy_fixtures.rs' scripts/check-rust-core-style.mjs`, mirroring Phase 2 ac2_5, so the Phase 1 script edit is verified before policy_fixtures.rs exists.
- phases.phase2.changes.crates/runx-core/src/policy/local.rs
  - Grounded in: code:oss/packages/core/src/policy/index.ts:102
  - Recommendation: Pin the literal default `allowed_source_types = ["agent", "agent-step", "approval", "cli-tool", "mcp", "a2a", "catalog", "graph"]` (in this order) and `max_timeout_seconds = 300` in the change description. Also add: 'Non-allow sandbox decisions (including ApprovalRequired) fold their reasons into the deny path; AdmissionDecision exposes only Allow/Deny.'
- phases.phase2.changes.crates/runx-core/src/policy/scope.rs
  - Grounded in: code:oss/fixtures/kernel/README.md:42
  - Recommendation: Add: 'Provides `pub(crate) fn unique_preserving_order(values: &[String]) -> Vec<String>` backed by a function-local `HashSet<&str>` for membership tracking. HashSet is permitted because it never crosses a serialized boundary. Used by `policy::graph_scope` for `requestedScopes`/`grantedScopes` deduplication so fixture key order matches the TypeScript oracle.'
- phases.phase2.changes.crates/runx-core/src/policy/connected_auth.rs
  - Grounded in: code:oss/packages/core/src/policy/index.ts:351
  - Recommendation: Add: 'Private helper `is_targeted(field: &Option<String>) -> bool` returns false for `None` and `Some("")` alike, mirroring TS `||` truthiness. `has_grant_reference` and `grant_reference_matches` go through this helper. Unit tests cover the empty-string case so an `""`-bearing JSON fixture would behave like an absent field.'
- phases.phase4.changes.docs/trusted-kernel-package-truth.md
  - Grounded in: code:oss/docs/trusted-kernel-package-truth.md:23
  - Recommendation: Tighten the change description to also revise the existing 'It is not an authoritative replacement for @runxhq/core/state-machine, and it does not yet implement policy parity.' sentence so the doc state is internally consistent after the edit, in addition to adding the `Rust policy parity status: fixture-evidence-only` phrase.


## Planning Log

- 2026-05-15T12:58:00Z: Drafted as third phase of Rust kernel parity.
- 2026-05-15T13:30:00Z: Revised after architectural review. Confirmed
  authority-proof and public-work re-exports as out of scope (deferred to
  follow-up). Added scope-narrowing module extraction step in Phase 1.
  Replaced wave-handing of `node:path` with `posix_basename` Rust helper that
  mirrors the TS posix-basename added by the fixtures spec. Added proptest
  phase. Now depends on `docs/rust-kernel-architecture.md`. Estimate bumped
  from 8h to 14h.
- 2026-05-16T00:00:00Z: Independent review correction. Dropped the
  scope-narrowing extraction step from Phase 1 (the TS `scopeAllows` helper
  is private and has no separate boundary; mirror it as `pub(crate)` inside
  the Rust policy module tree).
  Hardened the dependency on `rust-kernel-parity-fixtures` to a strict
  ordering for write access to `packages/core/src/policy/index.ts`. Dropped
  `--no-default-features` build step and the `no_std` posture references
  in line with the std-default decision in the arch doc.
- 2026-05-17T16:50:00Z: Claude harden round 1 required revisions. Added
  policy fixture coverage enforcement to the Rust style guard scope, made
  Phase 4 doc gates non-vacuous with exact new phrases, split policy
  implementation into small modules (`local`, `retry`, `graph_scope`,
  `interpreter`, `scope`, `sandbox`, `types`, `posix_basename`) with
  `policy.rs` as a thin root, changed `scope_allows` to `pub(crate)` but not
  public API, pinned hand-written ASCII interpreter parsing instead of adding
  `regex`, pinned `Vec` + `BTreeSet` ordered dedup instead of `indexmap`, and
  added Phase 1 serde round-trip tests for policy types.
- 2026-05-18T00:00:00Z: Claude harden round 2 required revisions. Added
  private `connected_auth` scaffolding for local admission parity without
  widening the deferred authority-proof public API, pinned unknown-shaped
  fields such as `LocalAdmissionSkill.auth` to `runx_contracts::JsonValue`,
  required hand-written sandbox path segment splitting, aligned Phase 4 README
  changes with its acceptance command, and simplified the Phase 1 compile-only
  cargo command. Also recorded the project rule that Rust parity means
  identical observable behavior and wire/fixture contracts, not blanket
  TypeScript-shaped naming or internals.
- 2026-05-18T00:15:00Z: Claude harden round 3 required revisions. Corrected
  the authority-proof deferred function count, made the section 14 architecture
  doc edit mandatory and specific to placeholder publishing status, and pinned
  snake_case serde exceptions for `LocalAdmissionGrant` and `GraphScopeGrant`
  while preserving camelCase for surrounding decision outputs such as
  `grantId`.
- 2026-05-18T00:30:00Z: Claude harden round 4 required revisions. Added an
  architecture-doc acceptance check for
  `runx-core policy parity is not runtime-authoritative`, pinned the
  `SandboxAdmissionDecision::ApprovalRequired` serde rename to
  `approval_required`, and required `ProptestConfig::with_cases(64)` for the
  policy property suite.
- 2026-05-18T00:45:00Z: Claude harden round 5 required revisions. Pinned
  policy decision `reasons` fields to `Vec<String>` with call-site formatted
  strings, clarified the `BTreeSet` dedupe helper is function-local and never
  serialized, and widened Phase 1 serde coverage to include
  `AdmissionDecision::Allow` and `AdmissionDecision::Deny` round-trips.
- 2026-05-18T01:00:00Z: Claude harden round 6 required revisions. Pinned
  `ConnectedAuthRequirement` as a `pub(crate)` internal struct co-located in
  `policy::connected_auth`, and added `RequiredSandboxDeclaration` to the
  in-scope type list with its camelCase/optional-field serde shape.
- 2026-05-18T01:15:00Z: Claude harden round 7 required revisions. Kept
  `posix_basename` private to the crate, pinned `LocalAdmissionGrantStatus`,
  and collapsed the duplicate TS sandbox shapes into the single
  `SandboxDeclaration` Rust type.
- 2026-05-18T02:20:00Z: Claude harden round 8 required revisions. Corrected
  the serde round-trip risk reference to `policy/types.rs` and pinned
  `skip_serializing_if = "Option::is_none"` across every optional
  `LocalAdmissionGrant` targeting field, not just `status`.
- 2026-05-18T02:35:00Z: Claude harden round 9 surfaced concrete polish items.
  Pinned explicit empty-array serialization for graph-scope decisions,
  insertion-preserving dedupe semantics, non-vacuous Phase 1 style-check
  registration, local-admission defaults, JS-truthiness for connected-grant
  references, and the intentional mapping of sandbox `approval_required` to a
  local-admission deny. Per user direction, do not continue harden cycling for
  additional low-severity polish once these contract holes are closed.
