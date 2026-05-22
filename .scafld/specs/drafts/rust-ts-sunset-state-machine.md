---
spec_version: '2.0'
task_id: rust-ts-sunset-state-machine
created: '2026-05-18T00:00:00Z'
updated: '2026-05-22T01:34:22Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# TS sunset: state-machine

## Current State

Status: draft, blocked
Current phase: public export removed; deletion remains blocked by fixture oracle
Next: resolve the remaining fixture-oracle ownership and boundary deletion
enforcement before approval.
Reason: the deletion remains blocked, but the runtime-local importer prerequisite
slice is complete and the 2026-05-22 public-surface slice removed
`packages/core/package.json` `exports["./state-machine"]`, regenerated
`docs/api-surface.md`, updated `scripts/gen-api-index.ts`, and reframed
`tests/runtime-local-state-machine-bridge-parity.test.ts` so it no longer
imports `@runxhq/core/state-machine`. A fresh 2026-05-22 owned-file importer
scan now finds no public export dependency in the owned test/package/docs
surface, but still finds two fixture generator scripts importing
`packages/core/src/state-machine/index.js` as the TypeScript oracle.
`crates/runx-core` has state-machine fixture parity and runtime-local now calls
the Rust kernel bridge for state-machine operations, but final TS deletion still
needs fixture oracle ownership and boundary deletion enforcement.
Blockers:
- Fixture generation still imports `packages/core/src/state-machine/index.js`
  as the TypeScript oracle; fixture-generator ownership must be reassigned,
  rerouted, or retired before deletion.
- Rerouting `scripts/generate-kernel-parity-fixtures.ts` to the Rust kernel is
  not a safe local-only edit today: `buildKernelParityFixtures()` and
  `evaluateKernelFixtureInput()` are synchronous and are imported by
  `tests/kernel-parity-fixtures.test.ts`, `scripts/check-fixture-key-order.ts`,
  and `scripts/validate-kernel-fixture-schemas.ts`; the available Rust kernel
  bridge requires `RUNX_KERNEL_EVAL_BIN` or an explicit binary command.
- `scripts/check-boundaries.mjs` still needs a future removed-export assertion
  for `./state-machine` when that file is in scope.
- Do not add legacy or compatibility shapes to keep the TS import path alive;
  the remaining work must remove or reroute the final script references and
  enforce that the public export stays removed.
Allowed follow-up command: `scafld validate rust-ts-sunset-state-machine --json`
while blocked; do not run `scafld harden` for this draft.
Latest runner update: 2026-05-22T01:34:22Z - public export, generated API docs,
and the temporary public-export type-parity dependency were removed. Deletion
remains blocked by fixture-oracle ownership and boundary deletion enforcement;
runtime-local source importers and public package/docs surfaces are no longer
blockers.
Review gate: not_started

## Summary

Future deletion target: remove the TypeScript state-machine implementation
after all direct source consumers are no longer depending on it. The public
`@runxhq/core/state-machine` export has been removed without a compatibility
alias. The current codebase is not ready for implementation deletion yet:
`crates/runx-core` implements Rust state-machine parity against
`fixtures/kernel/state-machine/`, and runtime-local now calls that Rust kernel
boundary for graph execution. Final deletion is still blocked by fixture-oracle
and boundary-enforcement follow-up work.

This spec must remain blocked until the fixture generators no longer depend on
the TS implementation as their oracle and boundary checks enforce that the
removed package export stays removed.

## Completed Prerequisite Slice

This draft is not executable as a deletion spec today. The completed
runtime-local prerequisite slice removed the live runtime-local source imports
from `@runxhq/core/state-machine`.

Implemented shape:
- `packages/runtime-local/src/runner-local/kernel-bridge.ts` owns the explicit
  Rust kernel boundary for state-machine state, plan, transition, fanout sync,
  fanout decision key, and local admission calls.
- Runtime-local graph hydration, orchestration, fanout gates, governance,
  resume, run-step, run-fanout, terminal, paused, and package index paths use
  the kernel bridge surface instead of importing
  `@runxhq/core/state-machine`.
- Success transitions carry `admissionWitness` through the bridge, matching the
  Rust kernel requirement that success cannot be represented without an
  admission/receipt witness.
- Fixture generation remains on the TS oracle and is the next ownership
  decision before final deletion.

Focused regression validation for this slice:
- `pnpm exec vitest run packages/runtime-local/src/runner-local/kernel-bridge.test.ts`
- `pnpm exec tsc -p tsconfig.typecheck.json --noEmit`
- `rg -n "from ['\"]@runxhq/core/state-machine['\"]" packages/runtime-local/src --glob '!**/dist/**'`

## Context

CWD: `.` from the OSS repo root (`/Users/kam/dev/runx/runx/oss`).

Packages:
- `@runxhq/core`
- `crates/runx-core`
- `@runxhq/runtime-local`
- Fixture-generation scripts that currently import the TS state-machine source

Current Rust parity status:
- `crates/runx-core` currently implements state-machine parity against the
  checked-in TypeScript oracle fixtures under `fixtures/kernel/state-machine/`.
- `docs/trusted-kernel-package-truth.md` records Rust-owned state-machine kernel
  inputs, but fixture refreshes still depend on the TypeScript oracle until the
  generator ownership decision is made.
- Kernel parity is not deletion completeness. The public package export has
  been removed, but this draft must not delete the TS implementation until the
  fixture-oracle scripts are rerouted, reassigned, or retired.

Current TypeScript deletion targets:
- `packages/core/src/state-machine/**`
- Boundary-check enforcement that `packages/core/package.json` does not regain
  export key `./state-machine`
- `scripts/check-boundaries.mjs` from this CWD, not `oss/scripts/...`; remove
  `state-machine` from `pureCoreDomains` only when the TS domain is actually
  deleted and add/keep a boundary assertion that `./state-machine` is no longer
  exported.

Current live importers found in source by the 2026-05-22 scan after the public
surface slice:
- `scripts/generate-kernel-parity-fixtures.ts`
- `scripts/generate-rust-fanout-fixtures.ts`

Importer census command:

```bash
rg -n "from ['\"]@runxhq/core/state-machine['\"]|from ['\"].*state-machine/index\.js['\"]" packages/runtime-local/src tests scripts --glob '!**/dist/**' --glob '!node_modules' --glob '!target'
```

Observed result after the 2026-05-22 public-surface slice: 2 files and 2 import
statements, both direct TS source oracle imports.

Runtime-local direct public-export importers: none.

Non-runtime-local live importers:
- `scripts/generate-kernel-parity-fixtures.ts`
- `scripts/generate-rust-fanout-fixtures.ts`

The current importer scan matches the expected post-public-surface-slice shape:
runtime-local source and owned tests no longer import `@runxhq/core/state-machine`,
while the fixture oracle scripts still directly import the TS implementation.

Files impacted:
- `packages/core/src/state-machine/` (future deletion)
- `scripts/check-boundaries.mjs` (future removed-export enforcement)
- The two current fixture-oracle importers listed above, but only after their
  owning cutover or retirement spec has landed.

Invariants:
- No consumer regression. Deletion is allowed only after a fresh importer scan
  proves there are no live imports from `@runxhq/core/state-machine` and no
  direct imports of `packages/core/src/state-machine/index.js`.
- No false compatibility. Do not preserve `@runxhq/core/state-machine` with a
  shim, adapter, re-export, conditional fallback, or legacy shape.
- Rust runtime ownership must be explicit before deletion; fixture parity alone
  is insufficient.
- Receipts produced before and after deletion remain verifiable.

## Objectives

- Keep this draft honest about current blockers.
- Keep the removed public export target recorded:
  `packages/core/package.json` `exports["./state-machine"]`.
- Require a fresh source scan before any deletion attempt.
- Delete the TS state-machine implementation only after the fixture-oracle
  scripts are rerouted, reassigned, or retired.
- Update `scripts/check-boundaries.mjs` from the OSS repo root so removed
  package exports and deleted pure-core domains stay enforced.

## Scope

In scope:
- Future TS state-machine deletion plan.
- Public export removal evidence for `@runxhq/core/state-machine`.
- Boundary-check updates needed by that deletion.
- Importer verification and deletion gating.

Out of scope:
- Runtime-local graph orchestration migration itself.
- Fixture generator replacement or retirement implementation.
- Rust runtime authority/cutover implementation.
- Legacy import compatibility, package shims, or fallback adapters.

## Dependencies

- Completed prerequisite:
  `rust-ts-sunset-state-machine-runtime-local-importers`.
- A fixture-generator ownership decision: keep TS oracle until final deletion,
  move generation to a new owner, or retire the generator with an explicit
  fixture-freeze policy.
- `rust-cli-rust-cutover` only if CLI/runtime execution still reaches
  runtime-local graph orchestration when this spec is reconsidered.
- A fresh importer scan immediately before approval.

## Open Questions

- Which spec owns fixture generation after the TS oracle is deleted?
- Should `scripts/check-boundaries.mjs` gain a generic removed-export list so
  `./state-machine` removal is enforced the same way removed runtime-local
  subpaths are enforced today?
