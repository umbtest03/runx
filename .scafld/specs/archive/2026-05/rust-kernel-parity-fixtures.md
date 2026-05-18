---
spec_version: '2.0'
task_id: rust-kernel-parity-fixtures
created: '2026-05-15T12:51:06Z'
updated: '2026-05-17T15:27:32Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# Rust kernel parity fixtures

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-17T15:27:32Z
Review gate: pass

## Summary

Create the shared fixture contract that lets the TypeScript trusted kernel stay
the source of truth while Rust kernel parity work begins. This task does not
port behavior. It turns stable `state-machine` and `policy` decisions into
checked JSON fixtures that both TypeScript and Rust can consume.

The immediate goal is to make current TypeScript kernel development sharper:
future TS changes must update intentional parity fixtures instead of letting
governance behavior drift invisibly.

This spec depends on the architecture decisions in
`oss/docs/rust-kernel-architecture.md`, in particular the fixture schema,
serde conventions, decision model, and platform-sensitive behavior policy.
Do not start this spec without reading that doc.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- future `crates/runx-core`

Files impacted:
- `packages/core/src/state-machine/index.ts`
- `packages/core/src/policy/index.ts`
- `packages/core/src/policy/sandbox.ts`
- `packages/core/src/policy/authority-proof.ts`
- `packages/core/src/policy/public-work.ts`
- `tests/**/*.test.ts`
- `fixtures/kernel/**`
- `fixtures/kernel/schema/*.schema.json`
- `scripts/generate-kernel-parity-fixtures.ts`
- `scripts/gen-api-index.ts`
- `docs/rust-kernel-architecture.md`
- `docs/trusted-kernel-package-truth.md`
- `docs/api-surface.md`

Invariants:
- TypeScript remains the source of truth for the trusted kernel in this phase.
- Fixtures cover pure exported behavior only; no parser, receipts, runtime,
  filesystem, subprocess, network, MCP, or agent adapters.
- Fixture JSON must avoid environment-specific paths, timestamps, and other
  nondeterministic values.
- Fixture updates must be deliberate and reviewable.
- Every fixture file validates against a checked-in JSON Schema in
  `fixtures/kernel/schema/`.
- Path semantics in fixtures are POSIX-only. Executable name normalization
  inputs that contain `\` are normalized as if the separator were `/`. See
  arch doc section 7.
- Serialized object-keyed maps (for example the fanout conflict gate
  `values` field at
  [packages/core/src/state-machine/index.ts:427](../../../packages/core/src/state-machine/index.ts#L427),
  the fanout branch `outputs` field, and any `Record<string, unknown>` in
  state-machine or policy outputs) are emitted with keys sorted
  lexicographically in fixture JSON. Rationale: V8 preserves insertion
  order for string keys, but the Rust port will use `BTreeMap` to keep
  iteration order deterministic across runs, language boundaries, and
  hash-randomization. Sorted-key serialization in fixtures is the canonical
  form both sides agree on.

Related docs:
- `docs/rust-kernel-architecture.md` (prerequisite reading)
- `docs/trusted-kernel-package-truth.md`
- `AGENTS.md`
- `plans/runx.md`

## Objectives

- Define the JSON fixture shape for kernel parity cases as a checked-in
  JSON Schema, not just prose.
- Replace the `node:path` import in `packages/core/src/policy/index.ts` with a
  small `posixBasename` helper. The trusted kernel must not import node-only
  modules; the arch doc requires it and Rust parity depends on it.
- Add deterministic fixture coverage for state-machine planning, single-step
  transitions, and fanout sync.
- Add deterministic fixture coverage for local admission, sandbox admission,
  retry admission, graph scope admission, and scope-narrowing semantics.
- Mark authority-proof and public-work re-exports as out of scope for this
  spec; record them as a follow-up.
- Add TypeScript tests that load the fixtures and assert current TS behavior.
- Add a fixture generation/check script so intentional fixture refreshes are
  explicit.
- Validate every fixture against its JSON Schema in CI.

## Scope

In scope:
- Stable pure behavior from `@runxhq/core/state-machine` direct exports:
  `createSingleStepState`, `transitionSingleStep`, `createSequentialGraphState`,
  `planSequentialGraphTransition`, `transitionSequentialGraph`,
  `evaluateFanoutSync`, `fanoutSyncDecisionKey`.
- Stable pure behavior from `@runxhq/core/policy` direct exports:
  `admitLocalSkill`, `admitRetryPolicy`, `admitGraphStepScopes`.
- Stable pure behavior from `@runxhq/core/policy/sandbox`:
  `normalizeSandboxDeclaration`, `sandboxRequiresApproval`, `admitSandbox`.
- Scope-narrowing fixtures exercised through `admitGraphStepScopes`; the
  production rule is the private `scopeAllows` helper in
  `packages/core/src/policy/index.ts`, and the checked-in `graph-scope-*`
  fixtures are the parity oracle for this phase.
- Fixture JSON Schema files in `fixtures/kernel/schema/`.
- TypeScript fixture validation tests.
- A small Rust-shaped sanity check: hand-port one fixture case through a
  throwaway parsing exercise to confirm the schema survives a Rust idiom
  before locking in the format. Discard after.

Out of scope:
- Rust implementation of the fixture runner (lives in
  `rust-state-machine-parity` and `rust-policy-parity`).
- `authority-proof` and `public-work` re-exports from `@runxhq/core/policy`.
  These are part of the trusted kernel surface but deferred to a follow-up
  spec (`rust-policy-authority-proof-parity`).
- Porting parser, YAML handling, receipts, signing, runtime-local, CLI, MCP,
  A2A, or provider adapters.
- Making Rust parity a blocking release gate.
- Changing existing kernel behavior except where a fixture exposes a clear bug
  and the fix is explicitly called out, plus the `node:path` removal noted in
  Objectives.

## Dependencies

- Current TS fast suite is green: `pnpm verify:fast`.
- Existing package boundary check continues to enforce pure policy and
  state-machine domains.

## Assumptions

- The first useful Rust migration artifact is a conformance contract, not Rust
  code.
- Fixtures should be small, named, and readable enough for code review.
- Fixture comparison should prefer semantic equality over byte equality where
  object key order is irrelevant.
- Fixture shape: `{ name, description?, input, expected }` where `expected` is
  a discriminated union `{ kind: "output", value } | { kind: "error", code,
  message? }`. See arch doc section 5 and the schema files.
- Fixtures cover the public TS export shape only, not internal helpers. If a
  helper needs a fixture, that helper should probably be a public export.

## Touchpoints

- State-machine fanout planning and sync decisions.
- Retry policy admission.
- Scope narrowing admission.
- Local skill admission and sandbox admission.
- Test utilities for loading fixture files.
- Documentation for the parity boundary.

## Risks

- Medium: overfitting fixtures to implementation internals instead of public
  exported behavior.
- Medium: fixture churn can slow TS development if made too broad too early.
- Medium: locking fixture format before any Rust port is attempted may force
  format rework in `rust-state-machine-parity`. Mitigated by the in-scope
  Rust-shaped sanity check.
- Low: a fixture can accidentally encode unstable object ordering or local
  paths.
- Low: the `node:path` removal could change behavior on Windows. Mitigated by
  POSIX-only semantics (arch doc section 7) and explicit tests of `\` input.

## Acceptance

Profile: standard

Validation:
- [x] `v1` test - TypeScript kernel fixture tests pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts tests/kernel-parity-fixtures.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26
- [x] `v2` command - fixture generation check is clean.
  - Command: `pnpm exec tsx scripts/generate-kernel-parity-fixtures.ts --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-27
- [x] `v3` command - fast verification remains green (includes boundary check).
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-28
- [x] `v4` command - every fixture validates against its JSON Schema.
  - Command: `pnpm exec tsx scripts/validate-kernel-fixture-schemas.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29
- [x] `v5` command - no `node:path` import remains in `packages/core/src/policy`.
  - Command: `! rg -n "from ['\"]node:path['\"]" packages/core/src/policy`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30

## Phase 1: Fixture schema and posix path helper

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- `fixtures/kernel/schema/fixture.schema.json` (all, exclusive) - Generic fixture envelope schema: `{ name, description?, input, expected }` with `expected` as a discriminated union on `kind`.
- `fixtures/kernel/schema/state-machine.schema.json` (all, exclusive) - Concrete schemas for state-machine input/output value types, referenced from the envelope.
- `fixtures/kernel/schema/policy.schema.json` (all, exclusive) - Concrete schemas for policy input/output value types.
- `fixtures/kernel/README.md` (all, exclusive) - Document fixture categories, naming, determinism rules, schema references, and update workflow.
- `packages/core/src/policy/posix-basename.ts` (all, exclusive) - In-tree helper that mirrors `path.basename` with POSIX-only semantics.
- `packages/core/src/policy/index.ts` (partial, shared) - Replace `import path from "node:path"` and `path.basename(...)` calls with the new helper.
- `packages/core/src/policy/posix-basename.test.ts` (all, exclusive) - Cover POSIX inputs, mixed-separator inputs, and inputs with trailing slashes.
- `tests/kernel-parity-fixtures.test.ts` (all, exclusive) - Add fixture loader that validates each loaded fixture against its schema before dispatching.

Acceptance:
- [x] `ac1_1` command - no `node:path` import in core policy.
  - Command: `! rg -n "from ['\"]node:path['\"]" packages/core/src/policy`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac1_2` test - posix-basename tests pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/core/src/policy/posix-basename.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `ac1_3` command - schema files exist and are valid JSON.
  - Command: `node -e "for(const f of require('fs').readdirSync('fixtures/kernel/schema')){JSON.parse(require('fs').readFileSync('fixtures/kernel/schema/'+f,'utf8'))}"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8

## Phase 2: Fixture coverage

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `fixtures/kernel/state-machine/*.json` (all, exclusive) - Fixture cases for single-step transitions, sequential planning, sequential transitions, fanout sync decisions, and decision-key derivation.
- `fixtures/kernel/policy/*.json` (all, exclusive) - Fixture cases for local admission, sandbox normalization, sandbox admission, retry admission, graph scope admission, and scope narrowing.
- `fixtures/kernel/runner/*.json` (all, exclusive) - Fixture-runner ingestion cases that pin fail-closed error envelopes for malformed-but-schema-shaped inputs.
- `tests/kernel-parity-fixtures.test.ts` (partial, exclusive) - Extend dispatch table to cover all fixture categories.
- `scripts/check-fixture-key-order.ts` (all, exclusive) - Enforce canonical sorted-key JSON for every generated fixture file.

Acceptance:
- [x] `ac2_1` command - fixtures avoid local paths and home-dir aliases.
  - Command: `! rg -n '/Users/|/home/|/private/|/tmp/sourcey|kam' fixtures/kernel`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13
- [x] `ac2_2` test - TypeScript can load and execute all fixtures.
  - Command: `pnpm exec vitest run --config vitest.config.ts tests/kernel-parity-fixtures.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `ac2_3` command - every fixture validates against its schema.
  - Command: `pnpm exec tsx scripts/validate-kernel-fixture-schemas.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `ac2_4` command - object-keyed map fields are sorted in fixture JSON.
  - Command: `pnpm exec tsx scripts/check-fixture-key-order.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16

## Phase 3: Generator and check mode

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `scripts/generate-kernel-parity-fixtures.ts` (all, exclusive) - Regenerate fixtures from TS execution and fail in `--check` mode when generated output differs from checked-in fixtures.
- `scripts/validate-kernel-fixture-schemas.ts` (all, exclusive) - Standalone schema validator for CI.
- `package.json` (partial, shared) - Add script aliases that match existing script conventions.
- `docs/trusted-kernel-package-truth.md` (partial, shared) - Document the parity fixture boundary and TS-first migration rule.
- `docs/api-surface.md` (partial, shared) - Note fixtures as conformance evidence for stable kernel exports.

Acceptance:
- [x] `ac3_1` command - check mode passes after generation.
  - Command: `pnpm exec tsx scripts/generate-kernel-parity-fixtures.ts --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21
- [x] `ac3_2` command - fixture script appears in docs or package scripts.
  - Command: `rg -n 'generate-kernel-parity-fixtures|kernel parity' fixtures/kernel README.md docs package.json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `ac3_3` command - docs name the parity boundary.
  - Command: `rg -n 'kernel parity|TypeScript remains the source of truth|fixtures/kernel' docs fixtures/kernel`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23

## Rollback

Strategy: per_phase

Commands:
- Revert the `node:path` removal in `packages/core/src/policy/index.ts` and
  delete `packages/core/src/policy/posix-basename.ts` plus its test.
- Remove `fixtures/kernel/**`, `tests/kernel-parity-fixtures.test.ts`,
  `scripts/generate-kernel-parity-fixtures.ts`, and
  `scripts/validate-kernel-fixture-schemas.ts`.
- Revert any docs/package script references added by this spec.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Second discover-mode pass on rust-kernel-parity-fixtures. Re-verified all in-scope deliverables against the approved spec and found no new completion blockers. posixBasename correctly normalizes backslashes + trailing slashes and is the sole replacement for node:path in @runxhq/core/policy; boundary check enforces the pure-kernel domain invariant; 33 fixtures (15 state-machine, 17 policy decision, 1 runner) match the TypeScript oracle for createSingleStepState, transitionSingleStep, createSequentialGraphState, planSequentialGraphTransition (linear + fanout + resolved-gate variants), transitionSequentialGraph, evaluateFanoutSync (proceed/halt/pause/escalate), fanoutSyncDecisionKey, admitLocalSkill (cli-tool/unsupported/inline-env/inline-windows-path/connected-grant), admitRetryPolicy, admitGraphStepScopes (exact/wildcard-narrowing/empty-request/widening/empty-grant/partial/prefix-request/dedupe/no-grant-id), normalizeSandboxDeclaration, sandboxRequiresApproval, and admitSandbox. The custom JSON-Schema subset validator is defensive (Object.hasOwn, prototype-safe schema-ref lookup, unsupported-keyword rejection, oneOf branch reporting), additionalProperties:false + kind-const discriminators eliminate cross-branch ambiguity, and the runner-prefix and isRunnerKernelFixture predicates are mutually enforced. CI gate is enforced via scripts/verify-fast.mjs invoking fixtures:kernel:validate, fixtures:kernel:check, fixtures:kernel:keys before test:fast; .github/workflows/ci.yml runs pnpm verify:fast. The three prior non-blocking findings (f1 runner message literal, f2 $schema keyword overload, f3 runner routing by name prefix) remain accurate and non-blocking.

Attack log:
- `packages/core/src/policy/posix-basename.ts`: Behavioural drift from node:path basename: trailing slashes, root path, mixed separators, bare command name, Windows drive letter -> clean (replace(/\\/gu, '/').replace(/\/+$/u, '') then slice after lastIndexOf('/'). '/' returns '', 'node' returns 'node', '/usr/bin/node' returns 'node', 'C:\\Tools\\node.exe' returns 'node.exe', 'C:\\Tools/bin/bash/' returns 'bash'. Tests cover these cases.)
- `packages/core/src/policy/index.ts`: Residual node-only import in pure policy domain after refactor; transitive imports through sandbox/authority-proof -> clean (grep over packages/core/src/policy shows zero node:* imports. scripts/check-boundaries.mjs lists policy in pureCoreDomains and forbids node:fs, node:path, node:child_process, node:crypto, etc. authority-proof.ts imports only ../util/hash.js and contracts (pure).)
- `scripts/generate-kernel-parity-fixtures.ts validateJsonSchemaValue`: Prototype pollution via __proto__/toString in schema or value; unsupported keyword silently passing; oneOf giving false negatives -> clean (Object.hasOwn for all property lookups; kernelFixtureSchemaFile uses Object.hasOwn; supportedJsonSchemaKeywords whitelist with explicit rejection (test asserts 'enum' is rejected and 'toString' schema ref is rejected); oneOf branch summary reports all branch failures with first-error path/message.)
- `scripts/generate-kernel-parity-fixtures.ts evaluateKernelFixtureInputUnchecked dispatch`: Missing input.kind branch or unhandled kind silently passing through -> clean (Switch covers all 13 declared kinds (7 state-machine, 6 policy). Default branch throws and is wrapped in KernelFixtureEvaluationError; runner fixture pins this envelope.)
- `scripts/generate-kernel-parity-fixtures.ts --check mode`: Stale fixture file remains under fixtures/kernel/ that no generator case produces -> clean (After regen, iterates collectKernelFixtureFiles() and throws for any path not in expectedFiles set. Diff also detects content drift via per-file readFile+string compare.)
- `scripts/check-fixture-key-order.ts`: Object-keyed map serialization order divergence between V8 insertion order and Rust BTreeMap -> clean (Re-serializes each fixture through stableFixtureJson (sorts keys at every object boundary, drops undefined) and string-compares to checked-in content; fails closed. README documents that arrays preserve first-seen insertion order and only object keys are sorted.)
- `fixtures/kernel/schema/*.schema.json`: Schema admits malformed inputs: missing kind, extra fields, wrong shape, wrong $schema for the kind -> clean (Envelope requires {$schema, name, input, expected} with name kebab-case pattern; concrete schemas pin $schema via const, use additionalProperties:false + oneOf discriminated on input.kind. Cross-schema mismatches (policy fixture with state-machine $schema) fail because the loaded concrete schema's oneOf has no matching kind branch.)
- `tests/kernel-parity-fixtures.test.ts loader`: Test silently skips fixtures or accepts undefined-equals-undefined for error fixtures without enforcing throw -> clean (Asserts fixtureFiles.length > 0, asserts filename === fixture.name, runs full schema validation, dispatches both output and error fixtures, and for error fixtures forces a thrown variable + toMatchObject on code/message. Also pins relative path to runner/ prefix matching isRunnerKernelFixture.)
- `scripts/verify-fast.mjs and .github/workflows/ci.yml`: Fixture validation could be skipped in CI or run after the failing test, masking drift -> clean (verify-fast.mjs invokes fixtures:kernel:validate, fixtures:kernel:check, fixtures:kernel:keys before test:fast; CI executes pnpm verify:fast on every push and PR.)
- `fixture canonicalization vs runner ingestion`: Rust runner satisfies envelope parity but emits different wrapper message, or routes a runner fixture by accident -> skipped (Already recorded as non-blocking findings f1 (runner-message-contract-ambiguity) and f3 (runner-routing-by-name-prefix) in the prior review pass; no new evidence to add.)

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

Estimated effort hours: 8
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- trusted-kernel
- parity
- fixtures

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
- none

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-17T13:06:31Z
Ended: 2026-05-17T13:07:09Z

Checks:
- path audit
  - Grounded in: code:packages/core/src/policy/index.ts:3
  - Result: passed
  - Evidence: Phase 1 owns the only TS kernel code change needed before
- command audit
  - Grounded in: code:package.json:16
  - Result: passed
  - Evidence: Acceptance commands use the existing workspace toolchain
- scope/migration audit
  - Grounded in: code:scripts/check-boundaries.mjs:84
  - Result: passed
  - Evidence: The spec targets exactly the pure core domains enforced by
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: Phase 1 validates schema existence and POSIX basename behavior
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback is local and reversible: delete the new fixture
- design challenge
  - Grounded in: code:packages/core/src/state-machine/index.ts:427
  - Result: passed
  - Evidence: The spec captures the known determinism hazard from fanout

Questions:
- none


## Planning Log

- 2026-05-15T12:58:00Z: Drafted as first phase of Rust kernel parity.
- 2026-05-15T13:30:00Z: Revised after architectural review. Added JSON Schema
  files as a deliverable, expanded scope to include single-step transitions
  and scope-narrowing, marked authority-proof and public-work as deferred,
  added in-tree `posixBasename` helper to remove the `node:path` import,
  restructured into schema/coverage/generator phases. Estimate bumped from
  4h to 8h. Now depends on `docs/rust-kernel-architecture.md`.
- 2026-05-16T00:00:00Z: Independent review pass. Added an invariant
  requiring sorted-key serialization of object-keyed map fields (fanout
  conflict gate `values`, fanout `outputs`, any `Record<string, unknown>`)
  to keep `HashMap` vs `BTreeMap` semantics from diverging across language
  boundaries. Added `ac2_4` to enforce it via a key-order checker script.
  Updated arch doc section refs after renumbering (path-sensitive behavior
  is now section 7; decision model is section 5).
