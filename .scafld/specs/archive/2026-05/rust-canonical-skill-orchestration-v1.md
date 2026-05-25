---
spec_version: '2.0'
task_id: rust-canonical-skill-orchestration-v1
created: '2026-05-21T04:35:09Z'
updated: '2026-05-21T05:45:09Z'
status: completed
harden_status: not_run
size: large
risk_level: very_high
---

# Rust canonical skill orchestration v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T05:45:09Z
Review gate: pass

## Summary

Make Rust the canonical local skill orchestrator and make TypeScript a client
surface over Rust, not an alternate trusted runtime.

The end state is:

`TS may request. Rust admits. Rust orchestrates. Rust seals. TS displays.`

That means the native Rust CLI must run the accepted local workflows without
TypeScript installed. TypeScript can continue to provide npm packaging, SDKs,
composer/product UX, marketplace tooling, and compatibility tests, but it must
not be the only place where skill execution, graph execution, approval
resolution, retry/fanout/idempotency, receipt sealing, payment authority,
payment recovery, or harness replay is implemented.

This is a boundary/specification plan, not an immediate deletion spec. It
defines the proper code shape and the acceptance gates that must be satisfied
before `rust-ts-sunset-runtime-local` can complete safely.

## Boundary Contract

### Rust Owns

- Canonical CLI command parsing, help, exit codes, and JSON envelopes for local
  `runx` commands.
- Skill package loading from `SKILL.md` plus `X.yaml`, runner selection, input
  hydration, environment policy, and continuation via answer files.
- Graph execution: step planning, context edges, transitions, approval gates,
  retries, fanout, sync points, idempotency keys, and failure classification.
- Harness replay for standalone fixture files and inline skill/graph harness
  cases if those forms are advertised by `runx --help`.
- Adapters that affect trusted execution: `cli-tool`, `agent`, `a2a`, `mcp`,
  catalog/tool resolution, and offline deterministic fixture adapters.
- Receipt store, receipt sealing, receipt proof verification, history
  projection, and all redaction/public-message guarantees.
- Authority algebra and policy admission, including payment charge/pay/refund
  verbs, subset proofs, proof kinds, receipt-before-forward, recovery, and
  same-family constraints.
- Payment rail recovery state machines and deterministic offline mock/x402/
  Stripe SPT eventuality simulators used for dogfood.
- Public Rust APIs consumed by SDKs and wrappers for the above.

### TypeScript Owns

- npm package launcher and native-binary selection.
- Developer-facing wrappers that spawn the Rust binary or call stable Rust
  host-protocol/JSON APIs.
- TypeScript SDK and product/composer UX that requests local runs.
- Marketplace/package authoring tooling, scafld workflows, docs generation,
  and compatibility tests.
- Published TypeScript contract views that mirror `runx-contracts`.
- Cloud-hosted product services unless a separate cloud cutover spec moves
  them.

### TypeScript Must Not Own

- Any trusted admission decision.
- Any payment authority, spend, refund, dispute, recovery, or proof-sealing
  invariant.
- Any canonical receipt or history projection.
- Any only-implementation of local skill/graph/harness execution.
- Any hidden fallback for Rust CLI commands after the command is advertised as
  native.

## Current Boundary Violations

These are known violations to repair or explicitly retire:

- `README.md` says `@runxhq/runtime-local` owns local orchestration, caller
  interaction, sandbox preparation, receipt-write orchestration, harness
  execution, runtime SDK entrypoints, MCP client behavior, and tool catalogs.
  That contradicts the desired Rust canonical boundary.
- `docs/rust-kernel-architecture.md` still contains conformance-first language
  saying TypeScript remains authoritative. That is no longer true for local
  orchestration once this spec is accepted.
- `docs/api-surface.md` still publishes runtime-local as a local orchestration
  package rather than a sunset client/wrapper surface.
- `runx --help` advertises `runx harness <fixture.yaml|skill-dir|SKILL.md>`,
  but the Rust binary currently accepts fixture files only; `runx harness
  skills/stripe-pay --json` fails by trying to read the directory as YAML.
- Some native harness fixtures still depend on `node -e`; those are not valid
  standalone Rust CLI dogfood fixtures.
- `scripts/dogfood-core-skills.mjs` is currently the top-level dogfood queue.
  It can remain as a wrapper, but it must not be the canonical proof that the
  Rust CLI works without TS.
- Many tests and package sources still import `@runxhq/runtime-local` or
  `@runxhq/adapters`; those importers block the runtime-local sunset and blur
  ownership.
- Recent payment/x402 Rust tests prove kernel behavior but do not by themselves
  expose every accepted payment dogfood scenario as a TS-free CLI workflow.

## Target Code Shape

### Rust Runtime

Create or consolidate a public orchestration boundary under
`crates/runx-runtime/src/execution/`:

- `orchestrator.rs`: canonical entrypoint for skill, graph, and harness
  execution.
- `skill_run.rs`: remains as the CLI skill surface but delegates execution
  semantics to the orchestrator rather than growing a second path.
- `runner/`: graph state machine integration, context forwarding, approval,
  retry, fanout, authority, and step execution.
- `harness/`: fixture loading, inline skill/graph harness case expansion,
  assertion, and receipt proof validation.
- `adapters/`: Rust implementations for `cli-tool`, `agent`, `a2a`, `mcp`,
  catalog/tool resolution, and deterministic fixture adapters.
- `payment/` or `policy/payment_runtime.rs`: provider-neutral payment rail
  contract and recovery eventuality model used by mock/x402/Stripe SPT offline
  scenarios.

The Rust orchestrator should expose stable request/result types:

- `SkillRunRequest`, `GraphRunRequest`, `HarnessRunRequest`.
- `RunContinuation` for answer-file/approval continuation.
- `RunResult` containing status, receipt refs, child receipt refs, pending
  requests, and public-safe diagnostics.
- `RunStatus` with canonical values aligned to CLI exit codes.

No TypeScript package type should be required to construct these values.

### Rust CLI

`crates/runx-cli` is the canonical local command surface:

- Help text must only advertise forms that are implemented and tested in Rust.
- `runx skill <skill-dir|SKILL.md>` must work without Node for agent-step
  request/answer flows and for Rust-native fixture adapters.
- `runx harness <fixture.yaml>` must remain Rust-native.
- If `runx harness <skill-dir|SKILL.md>` remains in help, it must expand the
  inline `X.yaml` harness cases natively. Otherwise remove that advertised form
  until implemented.
- `runx doctor`, `runx list`, `runx history`, `runx policy`, `runx kernel`,
  `runx config`, `runx tool`, `runx registry`, `runx mcp`, and `runx dev`
  must not shell out to Node for canonical local behavior.

### TypeScript Packages

TypeScript should route to Rust through stable boundaries:

- `packages/cli`: Node selector/package UX only; local execution commands
  delegate to the native binary or consume Rust JSON APIs.
- `packages/runtime-local`: no new features; slated for deletion after
  importers route through Rust.
- `packages/adapters`: no new trusted behavior; slated for deletion after Rust
  adapters cover callable surfaces.
- SDK packages: call stable Rust CLI/host-protocol contracts; do not re-host
  local execution semantics.
- Tests: compatibility tests may use TS, but every trusted local behavior must
  have a Rust test or a TS-free Rust CLI fixture.

## Scope And Touchpoints

In scope:

- `.scafld/specs/drafts/rust-canonical-skill-orchestration-v1.md`
- `.scafld/specs/drafts/rust-ts-sunset-runtime-local.md`
- `docs/ts-interop-boundary.md`
- `docs/rust-kernel-architecture.md`
- `docs/api-surface.md`
- `docs/cli-exit-codes.md`
- `README.md`
- `crates/runx-runtime/README.md`
- `crates/runx-runtime/src/execution/**`
- `crates/runx-runtime/src/adapters/**`
- `crates/runx-runtime/src/receipts/**`
- `crates/runx-runtime/src/journal/**`
- `crates/runx-cli/src/**`
- Rust CLI/runtime tests under `crates/runx-cli/tests/**` and
  `crates/runx-runtime/tests/**`
- Current payment/x402 specs that depend on dogfood proof:
  `x402-pay-paid-echo-composer-v1`,
  `x402-pay-stripe-spt-dogfood-v1`, and the mock-scenario punch list.
- TS wrapper tests only when they are changed to spawn/consume Rust, not to
  reimplement local orchestration.

Out of scope:

- Cloud-hosted service runtime migration.
- Adding legacy aliases or compatibility command names.
- Live-money Stripe or crypto behavior.
- Keeping a TypeScript runtime-local facade as a long-term compatibility layer.
- Introducing a second Rust execution path that bypasses the orchestrator.

## Planned Phases

Phase 0: boundary inventory and docs repair.
: Update docs to say Rust is canonical for local orchestration, TypeScript is a
client/wrapper, and `runtime-local`/`adapters` are sunset surfaces. Add an
importer census and command-surface truth table.

Phase 1: help truth and TS-free native smoke.
: Make `runx --help` match implemented Rust behavior exactly. Add TS-free smoke
tests for `runx doctor`, `runx list`, `runx history`, `runx skill`, and
`runx harness` using the native binary with no Node/pnpm/tsx dependency.

Phase 2: canonical Rust orchestrator surface.
: Consolidate skill/graph/harness execution behind one Rust orchestrator API.
Remove duplicated execution semantics between `skill_run`, graph runner, and
harness replay. Make answer-file continuation, approval, retries, and receipt
writing flow through this API.

Phase 3: inline harness and official fixture cutover.
: Either implement native `runx harness <skill-dir|SKILL.md>` for `X.yaml`
inline harness cases or remove the advertised form. Replace official local
dogfood fixtures that use `node -e` with Rust-native fixtures or checked-in
CLI-tool scripts that are not Node-dependent for core proof.

Phase 4: payment/x402 dogfood promotion.
: Promote payment mock, paid-echo, x402, and Stripe SPT offline eventualities
from in-memory Rust tests into CLI-runnable Rust fixtures. Keep TS composer and
dogfood scripts as wrappers only.

Phase 5: TypeScript importer reroute.
: Route `packages/cli`, IDE, langchain, and remaining tests through the Rust
binary/JSON/host-protocol boundary. Remove `@runxhq/runtime-local` and
`@runxhq/adapters` imports from all non-sunset package sources.

Phase 6: runtime-local/adapters deletion readiness.
: Re-run the `rust-ts-sunset-runtime-local` importer census. Delete the
packages only after Rust CLI/runtime acceptance is green and docs/API surfaces
no longer publish runtime-local as an execution owner.

## Acceptance

Profile: strict

Validation:
- [x] `dod1` command - Docs state one clear boundary: Rust is canonical for
  - Command: `! rg -n '@runxhq/runtime-local.*owns local orchestration|runtime-local.*local runtime: orchestration|TypeScript remains authoritative' README.md docs crates/runx-runtime/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19
- [x] `dod2` command - Native `runx --help` advertises only
  - Command: `crates/target/debug/runx --help && cargo test --quiet --manifest-path crates/Cargo.toml -p runx-cli --test launcher`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20
- [x] `dod3` command - The Rust CLI has a TS-free smoke suite that runs
  - Command: `env -i PATH="$PATH" crates/target/debug/runx doctor --json && env -i PATH="$PATH" crates/target/debug/runx list skills --json && env -i PATH="$PATH" crates/target/debug/runx harness fixtures/harness/payment-approval-graph.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21
- [x] `dod4` command - All trusted local orchestration flows enter through the
  - Command: `rg -n 'pub (struct|enum) (SkillRunRequest|GraphRunRequest|HarnessRunRequest|RunContinuation|RunResult|RunStatus)' crates/runx-runtime/src/execution && cargo test --quiet --manifest-path crates/Cargo.toml -p runx-runtime --tests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `dod5` command - Official payment/x402 dogfood eventualities have
  - Command: `cargo test --quiet --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood && cargo test --quiet --manifest-path crates/Cargo.toml -p runx-runtime --test payment && ! rg -n 'command: node|node -e|tsx|pnpm' fixtures/harness fixtures/graphs fixtures/skills`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `dod6` command - `packages/cli/src/**` has no `@runxhq/runtime-local` or
  - Command: `! rg -n '@runxhq/(runtime-local|adapters)' packages/cli/src --glob '!**/*.test.ts'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24
- [x] `dod7` command - `rust-ts-sunset-runtime-local` is unblocked by a
  - Command: `scafld validate rust-ts-sunset-runtime-local --json && ! rg -n '@runxhq/runtime-local.*owns local orchestration|runtime-local.*local runtime: orchestration|TypeScript remains authoritative' README.md docs crates/runx-runtime/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-25
- [x] `v1` command - This spec validates.
  - Command: `scafld validate rust-canonical-skill-orchestration-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26
- [x] `v2` command - Native Rust CLI works without TS runtime.
  - Command: `env -i PATH="$PATH" crates/target/debug/runx doctor --json && env -i PATH="$PATH" crates/target/debug/runx list skills --json && env -i PATH="$PATH" crates/target/debug/runx harness fixtures/harness/payment-approval-graph.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-27
- [x] `v3` command - Every advertised `runx harness` form works or is not
  - Command: `crates/target/debug/runx --help && cargo test --quiet --manifest-path crates/Cargo.toml -p runx-cli --test launcher`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-28
- [x] `v4` command - Canonical local execution tests pass.
  - Command: `cargo test --quiet --manifest-path crates/Cargo.toml -p runx-runtime --tests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29
- [x] `v5` command - CLI source no longer imports runtime
  - Command: `! rg -n '@runxhq/(runtime-local|adapters)' packages/cli/src --glob '!**/*.test.ts'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30
- [x] `v6` command - Trusted dogfood fixtures do not require Node for core
  - Command: `! rg -n 'command: node|node -e|tsx|pnpm' fixtures/harness fixtures/graphs fixtures/skills`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-31
- [x] `v7` command - Docs no longer name runtime-local as canonical local
  - Command: `! rg -n '@runxhq/runtime-local.*owns local orchestration|runtime-local.*local runtime: orchestration|TypeScript remains authoritative' README.md docs crates/runx-runtime/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-32
- [x] `v8` command - Native x402 mock dogfood runs through the Rust CLI without
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-33

## Sequencing Rules

- Do not delete `packages/runtime-local` or `packages/adapters` until Phases 0
  through 5 pass.
- Do not add new TypeScript execution behavior for local skills, graphs,
  harnesses, or payments. New TS code must call Rust or remain purely
  presentational.
- Do not add compatibility aliases. Clean cutover means one canonical command
  spelling and one trusted execution owner.
- Do not count a TS wrapper test as proof unless the same behavior is proven by
  Rust tests or a TS-free Rust CLI fixture.
- If a Rust command cannot support a form currently advertised in help, fix the
  command or remove the advertised form before claiming cutover progress.

## Rollback

Strategy: per_phase

Commands:
- Phase 0 docs rollback: `git checkout HEAD -- README.md docs/ts-interop-boundary.md docs/rust-kernel-architecture.md docs/api-surface.md docs/cli-exit-codes.md crates/runx-runtime/README.md`
- Runtime rollback: revert the specific `crates/runx-runtime/src/execution/**`,
  `crates/runx-runtime/src/adapters/**`, and `crates/runx-cli/src/**` files
  touched by the phase.
- TS routing rollback: restore the previous wrapper/importer files only if the
  Rust command is also removed from help and the runtime-local sunset spec is
  marked blocked again.

## Harden Rounds

- none

## Planning Log

- 2026-05-21T04:35:09Z: Filed after boundary review found that skill
  orchestration remains partially TypeScript-owned, docs still contradict the
  Rust canonical target, and the Rust CLI must be useful without TypeScript
  installed.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode re-review of rust-canonical-skill-orchestration-v1 confirms the prior open blocker (workspace_mutation) is cleared: the workspace baseline before this review is clean, the 13 task-scoped changes since approval are stable, and no review-side mutation occurred during this pass. Spot-checks of acceptance evidence remain green: packages/cli/src has zero @runxhq/runtime-local|adapters imports (dod6/v5), packages/cli/src/import-boundary.test.ts pins ALLOWED_RUNTIME_IMPORTERS to an empty Map and walks every .ts under cliSourceRoot, fixtures/{harness,graphs,skills} are free of node/tsx/pnpm directives (dod5/v6), and packages/cli/src/help.ts only advertises runx forms that are implemented in the native binary. dispatch.ts funnels harness/dev/tool search+inspect/skill add+publish/registry/history-JSON through writeNativeRunx → runNativeRunx → spawn of crates/target/.../runx, with binary resolution preferring env (RUNX_RUST_CLI_BIN/RUNX_RUST_REGISTRY_BIN), then debug/release locations, then PATH "runx", and a guarded SIGTERM→1s→SIGKILL timeout race with shared `settled`/clearTimers. The prior F1 (dead TS dev-fixture pipeline in packages/cli/src/commands/dev/*) and F4 (replay/diff/skill inspect handlers that throw 'not implemented yet') persist intentionally — they are accepted-risk per the spec's sequencing rule that deletion of packages/runtime-local and packages/adapters defers to rust-ts-sunset-runtime-local. One new minor observation: packages/cli/src/dispatch.ts:55 imports handleInspectCommand but never references it — unused import, no behavior impact. No new completion blockers, no scope drift, no ambient drift, no review_self_mutation.

Attack log:
- `Prior workspace_mutation blocker`: Compare current Workspace Baseline Before Review (0 dirty paths) and Ambient drift (0) against prior review's mutated-during-review file list (crates/runx-receipts/src/tree.rs et al). Confirm those files are now classified as stable task_changes and that this review introduces no new mutations. -> clean (Baseline clean; task_changes stable across this read-only pass; the prior review_self_mutation condition is no longer present.)
- `Acceptance evidence dod1..dod7 and v1..v8`: Spot-check the recorded acceptance commands against current workspace state: docs greps, packages/cli/src import boundary, fixtures node/tsx/pnpm absence, help/launcher consistency. -> clean (rg -n '@runxhq/(runtime-local|adapters)' packages/cli/src --glob '!**/*.test.ts' returns no files; fixtures/{harness,graphs,skills} are free of node/tsx/pnpm; help.ts only advertises forms backed by native runx.)
- `packages/cli/src/import-boundary.test.ts`: Trace listTypeScriptFiles recursion and extractRuntimeImportSpecifiers regex against static and side-effect import patterns under packages/cli/src. -> clean (Walks every .ts file under cliSourceRoot, normalizes specifiers to runtimeLocalPackage/adaptersPackage and pins ALLOWED_RUNTIME_IMPORTERS to an empty Map. Matches both `from "x"` and bare `import "x";` patterns.)
- `packages/cli/src/native-runx.ts spawnNativeRunx`: Audit binary resolution, env merge, SIGTERM→SIGKILL timeout race, and stream encoding for hangs or unhandled rejections. -> clean (resolveNativeRunxBinary prefers RUNX_RUST_CLI_BIN/RUNX_RUST_REGISTRY_BIN then debug/release before falling back to 'runx' on PATH. `settled` flag guards both timer paths, killTimer is cleared in close/error handlers, child output is utf8-decoded before resolve.)
- `dispatch.ts retired-command paths (replay/diff/skill inspect)`: Confirm that handlers throw rather than silently swallow on retired commands, and that no production caller invokes them. -> finding (F4 — handleInspectRunCommand/handleReplaySeedCommand/handleDiffCommand still throw 'not implemented yet'; help.ts removed advertised forms, dispatch.ts still wires them. Accepted-risk per spec sequencing rules.)
- `packages/cli/src/commands/dev/* reachability`: Trace dispatch routes from `parsed.command === 'dev'` through writeNativeRunx; grep for callers of handleDevCommand/runDevFixture/runSkillFixture. -> finding (F1 — dispatch.ts:138-147 routes dev to native runx; handleDevCommand/runDevFixture/runSkillFixture remain in packages/cli/src/commands/dev/* with no production caller. Accepted-risk pending sunset.)
- `packages/cli/src/dispatch.ts imports`: Cross-reference each named import in dispatch.ts against in-file usage sites. -> finding (F5 — handleInspectCommand is imported (dispatch.ts:55) but never invoked anywhere in dispatch.ts; only handleInspectRunCommand is wired. Minor lint debt.)
- `Scope drift vs task_scope`: Map the 13 task_changes against the spec's Scope And Touchpoints and the system classifier's ambient_drift signal. -> clean (Classifier reports Ambient drift = 0. crates/runx-receipts/src/tree.rs and the receipt-tree oracle fixture are receipt-proof-verification adjuncts to the canonical Rust receipt sealing surface declared in the spec ('Receipt store, receipt sealing, receipt proof verification, history projection'). No ambient drift to attribute.)

Findings:
- [critical/non-blocking] `workspace_mutation` Prior workspace_mutation blocker is resolved.
  - Location: `crates/runx-receipts/src/tree.rs`
  - Evidence: Workspace Baseline Before Review reports 0 dirty paths; the 13 task_changes since approval baseline (including crates/runx-receipts/src/tree.rs, crates/runx-receipts/tests/receipt_tree_fixtures.rs, fixtures/runtime/receipt-tree/oracle.json) are accounted for as task-scoped (Ambient drift = 0) and remained stable across this read-only review. No new modifications were introduced by this review session.
  - Impact: Completion gate can advance: the read-only review contract held this pass.
  - Validation: scafld status --json shows baseline_dirty=0 and ambient_drift=0; running scafld review again from current state should not flag review_self_mutation.
- [low/non-blocking] `F1-dead-dev-fixture-pipeline` Unreachable TS dev-fixture pipeline still ships in packages/cli/src; deferred to runtime-local sunset.
  - Location: `packages/cli/src/commands/dev.ts:37`
  - Evidence: dispatch.ts:138-147 routes command === 'dev' directly through writeNativeRunx → runNativeRunx, so handleDevCommand (packages/cli/src/commands/dev.ts:37), runDevFixture (packages/cli/src/commands/dev/fixture-runner.ts:27), and the skill-fixture.ts stub that always returns failedFixture (line 35-41) are no longer invoked from any production path. Grep across packages/cli/src shows handleDevCommand/runDevFixture/runSkillFixture only referenced inside the commands/dev/* tree itself.
  - Impact: Leaves dead trusted-execution-shaped TS in OSS CLI source; no runtime correctness regression. Spec sequencing places this deletion in rust-ts-sunset-runtime-local, so it is accepted risk here.
  - Validation: After deletion in the sunset spec, rg -n 'handleDevCommand|runDevFixture|runSkillFixture' packages/cli/src returns zero matches and pnpm --dir oss typecheck/test continue to pass.
- [low/non-blocking] `F4-divergent-dispatch-wires-retired-commands` dispatch.ts still routes replay/diff/skill inspect to handlers that throw 'not implemented yet' even though help.ts no longer advertises them.
  - Location: `packages/cli/src/dispatch.ts:319`
  - Evidence: packages/cli/src/help.ts:36-63 omits evolve/replay/diff/skill inspect/export-receipts/knowledge show from the published usage, but dispatch.ts:319-364 still wires handleInspectRunCommand, handleReplaySeedCommand, handleDiffCommand which throw 'native receipt inspection is not implemented yet', 'native replay is not implemented yet', and 'native run diff is not implemented yet' (packages/cli/src/commands/history.ts:107,143,152). args.ts still parses replay/diff/skill inspect/evolve as routable commands.
  - Impact: A programmatic runCli consumer calling a retired command receives a thrown Error instead of a user-friendly usage hint. No production caller or in-repo test exercises this path today, so no observable regression.
  - Validation: After cleanup, runCli with a retired command should return a non-zero exit with a clear stderr usage message; rg -n 'handleReplaySeedCommand|handleDiffCommand|handleInspectRunCommand' packages/cli/src should return zero matches or only history.ts definitions intentionally retained.
- [low/non-blocking] `F5-unused-import-in-dispatch` packages/cli/src/dispatch.ts imports handleInspectCommand but never references it.
  - Location: `packages/cli/src/dispatch.ts:55`
  - Evidence: dispatch.ts:52-62 imports handleInspectCommand alongside handleInspectRunCommand/handleDiffCommand/handleReplaySeedCommand/etc. A workspace-wide grep for handleInspectCommand outside its definition in packages/cli/src/commands/history.ts:101 shows only the dispatch.ts:55 import — there is no call site. The handler itself unconditionally throws so the import is doubly dead.
  - Impact: Adds noise and a fragile reference; would be caught by no-unused-imports lint if enabled. No behavior impact.
  - Validation: After removal, rg -n 'handleInspectCommand' packages/cli/src returns only the history.ts definition (or zero if also removed), and pnpm --dir oss typecheck/test pass.
