---
spec_version: '2.0'
task_id: monolith-decomposition-v1
created: '2026-05-24T00:00:00Z'
updated: '2026-05-26T02:41:00Z'
status: draft
harden_status: not_run
size: large
risk_level: medium
---

# Decompose surviving god-files and retire large-file waivers

## Current State

Status: draft
Current phase: ready for scoped harden
Next: harden
Reason: refreshed on 2026-05-26 after the Rust rewrite. The original concrete
target list was stale, but the spec is not obsolete. The runtime-local TS
offenders still exist and are still large, but `rust-ts-sunset-runtime-local`
declares `packages/runtime-local/**` and `packages/adapters/**` deletion targets.
Do not spend decomposition effort inside those packages unless the sunset spec
later declares a surviving stable boundary. Current `runx doctor` file budgets
are only `packages/cli/src/index.ts` (62/1000 lines) and
`packages/cli/src/commands/doctor.ts` (873/950 lines), so there is no current
doctor file-budget offender. The surviving debt is now mostly Rust: 61 current
`rust-style-allow: large-file` waivers across surviving Rust crates.
Blockers: runtime-local/adapters deletion is blocked by its own sunset spec and
must not be solved here. Runtime Rust files are under active parallel work; each
implementation slice must rebase on the owning agent's edits and avoid broad
refactors. `runx-rust-95-release-readiness` is a dependency signal only; this
spec must not edit it.
Allowed follow-up command: `scafld harden monolith-decomposition-v1 --provider <provider>`
Review gate: not_started

## Summary

This remains a valid decomposition spec, but its execution target is now
surviving Rust/CLI/contract code rather than soon-to-be-deleted runtime-local
TypeScript. Split large surviving files into cohesive modules behind unchanged
public surfaces, and retire or re-justify each Rust large-file waiver. Runtime
behavior, public API, wire formats, fixtures, and test expectations stay
unchanged.

Do not archive this draft. Do not start by decomposing `packages/runtime-local/**`
or `packages/adapters/**`; those are deletion-bound and still blocked by
`rust-ts-sunset-runtime-local`.

## Current Evidence

Commands run on 2026-05-26:

```sh
wc -l packages/runtime-local/src/sdk/index.ts \
  packages/runtime-local/src/runner-local/graph-governance.ts \
  packages/runtime-local/src/runner-local/index.ts \
  packages/cli/src/index.test.ts \
  crates/runx-runtime/src/receipts/seal.rs \
  crates/runx-runtime/src/payment/ledger.rs \
  crates/runx-receipts/src/tree.rs
wc -l packages/cli/src/index.ts packages/cli/src/commands/doctor.ts
rg -l "rust-style-allow: large-file" crates --glob '*.rs' --glob '!**/target/**' | wc -l
rg -l "rust-style-allow: large-file" crates --glob '*.rs' --glob '!**/target/**' | xargs wc -l | sort -nr
rg --files crates packages scripts tests plugins --glob '*.rs' --glob '*.ts' \
  --glob '*.tsx' --glob '*.mts' --glob '*.cts' --glob '!**/dist/**' \
  --glob '!**/target/**' --glob '!node_modules/**' | xargs wc -l | \
  awk '$1 >= 1000 && $2 != "total" { print }' | sort -nr
rg -l "@runxhq/(runtime-local|adapters)|@runxhq/runtime-local|@runxhq/adapters|packages/(runtime-local|adapters)" \
  --glob '!node_modules/**' --glob '!dist/**' --glob '!.scafld/specs/**' | sort | wc -l
rg -l "@runxhq/core/marketplaces|\\.\\./marketplaces/index\\.js|packages/core/src/marketplaces|\\\"\\./marketplaces\\\"" \
  packages tests --glob '!**/dist/**' --glob '!node_modules' | sort
node scripts/check-rust-core-style.mjs
pnpm exec tsx packages/cli/src/index.ts doctor --json
```

Observed results:

- Original TS/Rust targets still exist at these sizes:
  `packages/runtime-local/src/sdk/index.ts` 1803,
  `packages/runtime-local/src/runner-local/graph-governance.ts` 1418,
  `packages/runtime-local/src/runner-local/index.ts` 1258,
  `packages/cli/src/index.test.ts` 2229,
  `crates/runx-runtime/src/receipts/seal.rs` 909,
  `crates/runx-runtime/src/payment/ledger.rs` 866, and
  `crates/runx-receipts/src/tree.rs` 1237.
- Current doctor file budgets are clean:
  `packages/cli/src/index.ts` is 62 lines against a 1000-line budget, and
  `packages/cli/src/commands/doctor.ts` is 873 lines against a 950-line budget.
- `pnpm exec tsx packages/cli/src/index.ts doctor --json` currently fails only
  on `runx.skill.lock.stale`, not on `runx.structure.file_budget.exceeded`.
- `node scripts/check-rust-core-style.mjs` currently fails on a separate
  long-function finding:
  `crates/runx-runtime/src/execution/runner/steps.rs:519 function has 62 lines`.
  It does not report unwaived Rust large-file failures because the large Rust
  files below carry waivers.
- Runtime-local/adapters exact references outside `.scafld/specs/**` and
  `dist/**` are still non-empty: 97 active files total, 73 outside
  `packages/runtime-local/**` and `packages/adapters/**`. This confirms package
  deletion is not complete and runtime-local decomposition would be churn.
- Marketplace deletion is also not complete. The current marketplace scan still
  lists `packages/cli/src/skill-refs.ts`, `packages/core/package.json`,
  `packages/core/src/marketplaces/index.ts`,
  `packages/runtime-local/src/runner-local/skill-install.ts`,
  `packages/runtime-local/src/sdk/index.ts`,
  `tests/skill-add-profile-metadata.test.ts`, and `tests/skill-add.test.ts`.

## Current Large Source Files

Files at or above 1000 lines in the current source/test/script tree:

| File | Lines | Disposition |
| --- | ---: | --- |
| `packages/contracts/src/schema-artifacts.ts` | 58540 | Surviving generated contract artifact; exclude from manual decomposition. |
| `crates/runx-contracts/tests/schema_wire_compat.rs` | 4284 | Surviving Rust test; split only with schema-wire test ownership. |
| `tests/thread-push-outbox-tool.test.ts` | 2377 | Surviving TS test; candidate test-fixture/helper split. |
| `crates/runx-runtime/src/execution/target_runner.rs` | 2344 | Surviving Rust waiver; priority decomposition target. |
| `packages/cli/src/index.test.ts` | 2229 | Surviving TS CLI test; still valid follow-up target, not a doctor-budget offender. |
| `scripts/generate-kernel-parity-fixtures.ts` | 1857 | Temporary oracle/generator surface; defer until kernel sunset owner confirms survival. |
| `packages/runtime-local/src/sdk/index.ts` | 1803 | Slated for runtime-local deletion; do not decompose here. |
| `crates/runx-runtime/tests/payment/execution.rs` | 1773 | Surviving Rust test; split after payment runtime owner stabilizes. |
| `crates/runx-parser/src/skill.rs` | 1609 | Surviving Rust waiver; priority decomposition target. |
| `crates/runx-runtime/tests/target_runner.rs` | 1546 | Surviving Rust test; split with target-runner implementation. |
| `crates/runx-runtime/tests/skill_run.rs` | 1507 | Surviving Rust test; split with skill-run implementation. |
| `crates/runx-runtime/src/execution/harness/runner.rs` | 1428 | Surviving Rust waiver; priority decomposition target. |
| `packages/runtime-local/src/runner-local/graph-governance.ts` | 1418 | Slated for runtime-local deletion; do not decompose here. |
| `crates/runx-runtime/src/post_merge_observer.rs` | 1322 | Surviving Rust waiver; priority decomposition target. |
| `crates/runx-runtime/src/adapters/external_adapter.rs` | 1259 | Surviving Rust waiver; priority decomposition target. |
| `packages/runtime-local/src/runner-local/index.ts` | 1258 | Slated for runtime-local deletion; do not decompose here. |
| `crates/runx-receipts/src/tree.rs` | 1237 | Surviving Rust waiver; priority decomposition target. |
| `crates/runx-runtime/src/sandbox.rs` | 1228 | Surviving Rust waiver; priority decomposition target. |
| `packages/contracts/src/index.test.ts` | 1204 | Surviving TS contract test; split only with contract test ownership. |
| `crates/runx-core/src/policy/payment_authority.rs` | 1192 | Surviving Rust waiver; priority decomposition target. |
| `packages/contracts/src/schemas/spine.ts` | 1165 | Surviving TS contract surface; split only if generated/schema contract stays unchanged. |
| `crates/runx-runtime/src/execution/skill_run.rs` | 1132 | Surviving Rust waiver; priority decomposition target. |
| `packages/contracts/src/openapi-public.ts` | 0 | Removed from OSS; hosted OpenAPI generation now lives in the cloud package. No decomposition action. |
| `crates/runx-runtime/src/execution/runner/steps.rs` | 1102 | Surviving Rust waiver; priority target and current long-function failure host. |
| `packages/runtime-local/src/runner-local/kernel-bridge.ts` | 1088 | Slated for runtime-local deletion; do not decompose here. |
| `crates/runx-contracts/src/post_merge_observer/plan.rs` | 1081 | Surviving Rust waiver; priority decomposition target. |
| `packages/core/src/artifacts/index.ts` | 1080 | `@runxhq/core` sunset-bound unless a surviving contract owner is declared. |
| `packages/core/src/parser/index.ts` | 1053 | Parser sunset-bound; defer to `rust-ts-sunset-parser`. |
| `crates/runx-cli/src/launcher.rs` | 1039 | Surviving Rust waiver; priority decomposition target. |
| `crates/runx-runtime/tests/post_merge_observer.rs` | 1036 | Surviving Rust test; split with observer implementation. |
| `crates/runx-runtime/tests/external_adapter.rs` | 1034 | Surviving Rust test; split with external-adapter implementation. |
| `crates/runx-runtime/src/journal.rs` | 1022 | Surviving Rust waiver; priority decomposition target. |
| `packages/runtime-local/src/runner-local/history.ts` | 1011 | Slated for runtime-local deletion; do not decompose here. |

## Current Rust Large-File Waivers

All current `rust-style-allow: large-file` entries are in Rust crates that
survive the TS sunset unless a later crate-specific spec says otherwise. Files
below 350 lines still need a cleanup decision: remove the stale waiver if the
file is now under budget, or keep a concrete reason only if the waiver is still
serving an active style exception.

| File | Lines | Survival | Required action |
| --- | ---: | --- | --- |
| `crates/runx-runtime/src/execution/target_runner.rs` | 2344 | Surviving Rust code | Split or re-justify. |
| `crates/runx-parser/src/skill.rs` | 1609 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/execution/harness/runner.rs` | 1428 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/post_merge_observer.rs` | 1322 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/adapters/external_adapter.rs` | 1259 | Surviving Rust code | Split or re-justify. |
| `crates/runx-receipts/src/tree.rs` | 1237 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/sandbox.rs` | 1228 | Surviving Rust code | Split or re-justify. |
| `crates/runx-core/src/policy/payment_authority.rs` | 1192 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/execution/skill_run.rs` | 1132 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/execution/runner/steps.rs` | 1102 | Surviving Rust code | Split or re-justify. |
| `crates/runx-contracts/src/post_merge_observer/plan.rs` | 1081 | Surviving Rust code | Split or re-justify. |
| `crates/runx-cli/src/launcher.rs` | 1039 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/journal.rs` | 1022 | Surviving Rust code | Split or re-justify. |
| `crates/runx-contracts/src/operational_policy/evaluate.rs` | 989 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/receipts/seal.rs` | 909 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/doctor.rs` | 909 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/payment/ledger.rs` | 866 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/dev/loop.rs` | 835 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/payment/state.rs` | 744 | Surviving Rust code | Split or re-justify. |
| `crates/runx-contracts/src/target_runner/plan.rs` | 726 | Surviving Rust code | Split or re-justify. |
| `crates/runx-cli/src/registry.rs` | 711 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/config.rs` | 675 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/runtime_http.rs` | 674 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/list.rs` | 665 | Surviving Rust code | Split or re-justify. |
| `crates/runx-contracts/src/schema.rs` | 653 | Surviving Rust code | Split or re-justify. |
| `crates/runx-core/src/policy/authority_proof.rs` | 648 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/execution/runner/authority.rs` | 628 | Surviving Rust code | Split or re-justify. |
| `crates/runx-contracts/src/operational_policy.rs` | 621 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/payment/supervisor.rs` | 616 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/execution/runner/execution.rs` | 613 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/receipts/store.rs` | 605 | Surviving Rust code | Split or re-justify. |
| `crates/runx-cli/src/history.rs` | 596 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/outbox_provider.rs` | 586 | Surviving Rust code | Split or re-justify. |
| `crates/runx-contracts/src/tools.rs` | 584 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/registry/install.rs` | 575 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/adapters/a2a.rs` | 550 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/credentials.rs` | 545 | Surviving Rust code | Split or re-justify. |
| `crates/runx-core/src/policy/types.rs` | 540 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/adapters/mcp/server_skill.rs` | 534 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/registry/local/build.rs` | 528 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/adapters/mcp/server.rs` | 523 | Surviving Rust code | Split or re-justify. |
| `crates/runx-core/src/kernel_eval.rs` | 496 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/registry/local.rs` | 493 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/registry/payload.rs` | 482 | Surviving Rust code | Split or re-justify. |
| `crates/runx-core/src/state_machine/sequential_graph.rs` | 478 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/scaffold/templates.rs` | 461 | Surviving Rust code | Split or re-justify. |
| `crates/runx-cli/src/config.rs` | 435 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/adapters/mcp/transport.rs` | 431 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/execution/runner.rs` | 423 | Surviving Rust code | Split or re-justify. |
| `crates/runx-core/src/state_machine/fanout.rs` | 418 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/dev/skill.rs` | 407 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/registry/local/trust.rs` | 397 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/execution/harness/fixtures.rs` | 378 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/dev/tool.rs` | 373 | Surviving Rust code | Split or re-justify. |
| `crates/runx-cli/src/tool.rs` | 373 | Surviving Rust code | Split or re-justify. |
| `crates/runx-core/src/state_machine/types.rs` | 367 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/bin/runx-harness-fixture-oracles.rs` | 366 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/adapters/cli_tool.rs` | 353 | Surviving Rust code | Split or re-justify. |
| `crates/runx-runtime/src/adapters/agent.rs` | 339 | Surviving Rust code | Remove stale waiver or record why it remains. |
| `crates/runx-sdk/src/client.rs` | 303 | Surviving Rust code | Remove stale waiver or record why it remains. |
| `crates/runx-cli/src/main.rs` | 177 | Surviving Rust code | Remove stale waiver or record why it remains. |

## Objectives

- Decompose surviving production Rust files above 1000 lines first, preserving
  public module exports and wire behavior.
- Retire stale Rust large-file waivers on files below 350 lines, or replace them
  with concrete, current justifications if a waiver remains intentional.
- Split large surviving tests only after the owning implementation slice is
  stable, keeping assertion semantics and fixture bytes unchanged.
- Keep `runx doctor` file budgets clean and avoid adding runtime-local or
  adapters compatibility shims.

## Scope

In scope:
- Surviving Rust files listed in `Current Rust Large-File Waivers`.
- Surviving TS/contract/CLI tests and helpers listed in `Current Large Source
  Files`, when the owning domain is not scheduled for deletion.
- Removing stale `rust-style-allow: large-file` comments from files already
  below the Rust style budget.
- Behavior-preserving module extraction and test-helper extraction.

Out of scope:
- Any behavior, contract, public API, fixture, schema, or wire-format change.
- `packages/runtime-local/**` and `packages/adapters/**` decomposition; those
  packages are deletion-bound under `rust-ts-sunset-runtime-local`.
- `@runxhq/core` parser/artifacts/marketplace decomposition while their sunset
  owner has not declared a surviving boundary.
- Generated contract artifacts such as `packages/contracts/src/schema-artifacts.ts`.
- `runx-rust-95-release-readiness`; this spec must not edit it.
- Large cross-crate refactors that are not needed to cut a single file boundary.

## Dependencies

- `rust-ts-sunset-runtime-local` remains the deletion owner for
  `packages/runtime-local/**` and `packages/adapters/**`.
- `rust-ts-sunset-marketplaces` and `rust-ts-sunset-parser` remain deletion or
  blocker owners for their TS surfaces.
- Any implementation slice touching files already edited by another agent must
  re-read the file and preserve their changes before patching.

## Execution Plan

1. Refresh inventory before each harden/build run:
   - Re-run the line-count, waiver, doctor, and sunset reference commands above.
   - Fail the slice if a target moved under another active spec or is
     deletion-bound.
2. Remove stale low-risk waivers:
   - Start with files already below 350 lines:
     `crates/runx-cli/src/main.rs`, `crates/runx-sdk/src/client.rs`, and
     `crates/runx-runtime/src/adapters/agent.rs`.
   - If the file still has a current style reason, update the reason and record
     the follow-up; otherwise remove the waiver and run
     `node scripts/check-rust-core-style.mjs`.
3. Decompose the highest-risk surviving Rust production files in small batches:
   - Batch A: `crates/runx-runtime/src/execution/target_runner.rs` with
     `crates/runx-runtime/tests/target_runner.rs`.
   - Batch B: `crates/runx-parser/src/skill.rs` with parser fixture tests.
   - Batch C: `crates/runx-runtime/src/execution/harness/runner.rs` with
     harness fixture tests.
   - Batch D: `crates/runx-runtime/src/adapters/external_adapter.rs` with
     external-adapter tests.
   - Batch E: receipts/payment cluster:
     `crates/runx-receipts/src/tree.rs`,
     `crates/runx-runtime/src/receipts/seal.rs`, and
     `crates/runx-runtime/src/payment/ledger.rs`.
   Each batch must keep public exports unchanged and avoid touching unrelated
   runtime modules.
4. Address medium waivers by ownership cluster:
   - CLI command parsing/rendering: `crates/runx-cli/src/{launcher,registry,history,config,tool}.rs`.
   - Runtime registry/config/list/journal/doctor surfaces.
   - Contracts schema/planning surfaces.
   - Core policy/state-machine bridge surfaces.
5. Split surviving test monoliths after implementation boundaries settle:
   - `packages/cli/src/index.test.ts` by command or diagnostic family.
   - Rust runtime tests by implementation module.
   - Do not split generated artifacts or runtime-local tests that will be
     deleted with their package.
6. Validation for each batch:
   - `node scripts/check-rust-core-style.mjs`.
   - The focused `cargo test --manifest-path crates/Cargo.toml -p <crate> ...`
     suite for touched Rust crates.
   - `pnpm exec tsx packages/cli/src/index.ts doctor --json`; acceptable only if
     any failure is unrelated to file budgets and recorded.
   - `git diff --check -- <touched-files>`.

## Acceptance

- [ ] `dod1` The runtime-local and adapters TS files are explicitly excluded
  unless their sunset owner later declares a surviving stable boundary.
- [ ] `dod2` Every touched surviving Rust file either drops below 350 lines with
  its large-file waiver removed, or retains a concrete current justification and
  a named follow-up.
- [ ] `dod3` Surviving large test files are split only behind unchanged test
  semantics and fixture bytes.
- [ ] `dod4` `runx doctor` reports no `runx.structure.file_budget.exceeded`
  diagnostics for the current doctor budget list.
- [ ] `dod5` Rust style and focused crate/package tests pass for every touched
  batch, with any unrelated pre-existing failure recorded.
- [ ] `dod6` No public API, contract schema, fixture output, or runtime behavior
  changes as part of decomposition.

## Origin

A+ roadmap (2026-05-24), step 4. Surfaced by the structural review; the waivers
are self-documenting deferred decompositions.
