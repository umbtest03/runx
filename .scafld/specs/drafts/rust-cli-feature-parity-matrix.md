---
spec_version: '2.0'
task_id: rust-cli-feature-parity-matrix
created: '2026-05-15T13:16:55Z'
updated: '2026-05-15T13:24:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Rust CLI feature parity matrix

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld harden rust-cli-feature-parity-matrix`
Latest runner update: none
Review gate: not_started

## Summary

Define the one-to-one feature parity contract for any future Rust rebuild of
the runx CLI and OSS runtime surface. Kernel parity is useful, but it is not a
CLI rebuild. Before any Rust CLI/runtime cutover, every current command,
option, output contract, receipt behavior, runtime behavior, and adapter path
must have an oracle fixture or documented parity case against the current
TypeScript implementation.

This task creates the parity matrix and oracle harness only. It does not port
runtime behavior to Rust.

## Context

CWD: `.`

Packages:
- `@runxhq/cli`
- `@runxhq/core`
- `@runxhq/runtime-local`
- `@runxhq/adapters`
- `@runxhq/contracts`
- `crates/runx-cli`
- future `crates/runx-core`

Files impacted:
- `fixtures/cli-parity/**`
- `tests/cli-feature-parity.test.ts`
- `scripts/generate-cli-feature-parity.ts`
- `docs/trusted-kernel-package-truth.md`
- `docs/cli-exit-codes.md`
- `docs/api-surface.md`
- `README.md`
- `packages/cli/src/help.ts`
- `packages/cli/src/args.ts`
- `packages/cli/src/index.test.ts`
- `packages/runtime-local/src/**`
- `packages/adapters/src/**`

Invariants:
- TypeScript CLI and runtime remain the source of truth until a cutover spec
  explicitly changes the authoritative implementation.
- Rust `runx-cli` remains a launcher until the feature matrix is complete and a
  later native CLI spec passes it.
- Feature parity means no command, flag, exit code, JSON shape, receipt
  behavior, side effect, or documented workflow silently disappears.
- Live provider calls are not required for parity; provider paths must use
  deterministic mocks, fixtures, or local protocol servers.
- Human output parity may be semantic and snapshot-normalized; JSON and receipt
  contracts must be schema-exact.

Related docs:
- `docs/trusted-kernel-package-truth.md`
- `docs/cli-exit-codes.md`
- `docs/api-surface.md`
- `README.md`
- `plans/runx.md`

## Objectives

- Inventory every current CLI command, subcommand, flag, environment input,
  documented exit code, JSON response, and human-output promise.
- Add a checked-in CLI feature parity matrix under `fixtures/cli-parity`.
- Add TypeScript oracle tests that prove the current CLI satisfies the matrix.
- Add coverage for runtime behavior that is not visible from `--help`, including
  receipts, sandbox metadata, artifacts, history, resume/replay, and adapter
  side effects.
- Define the required Rust candidate test mode for later native CLI/runtime
  specs.
- Update the architecture doc so no Rust cutover can occur without this matrix.

## Scope

In scope:
- Current CLI commands from `runx --help`: `skill`, `evolve`, `resume`,
  `replay`, `diff`, `search`, `add`, `inspect`, `history`, `export-receipts`,
  `knowledge show`, `connect`, `config`, `new`, `init`, `harness`, `list`,
  `doctor`, `dev`, `mcp serve`, `tool search`, `tool inspect`, and
  `tool build`.
- Top-level aliases and admin forms such as `runx search`, `runx add`,
  `runx inspect`, and `runx skill <action>`.
- Exit codes `0`, `1`, `2`, and `64`.
- JSON output shapes, receipt schemas, ledger/artifact references, sandbox
  metadata, signature verification, and trainable receipt export behavior.
- Adapter paths: cli-tool, MCP, A2A, catalog, managed agent, and caller-mediated
  resolution using deterministic fixtures.
- Environment/config behavior: `RUNX_HOME`, `RUNX_CWD`, `RUNX_RECEIPT_DIR`,
  `RUNX_TOOL_ROOTS`, agent config env vars, sandbox enforcement env vars, and
  local config file behavior.

Out of scope:
- Implementing Rust equivalents.
- Changing command semantics.
- Live OpenAI, Anthropic, GitHub, registry, or OAuth integration calls.
- Hosted cloud parity.
- Performance targets beyond recording current baseline timings.

## Dependencies

- Current TypeScript fast suite is green.
- Existing Rust placeholder crate may exist, but this task does not depend on
  any Rust runtime behavior.
- `docs/trusted-kernel-package-truth.md` must state the no-cutover rule.

## Assumptions

- The current TypeScript CLI is the oracle even when behavior is imperfect; if
  the matrix exposes a bug, fix it explicitly before recording the new expected
  behavior.
- The matrix should start broad and shallow, then deepen high-risk commands with
  fixture cases.
- Commands that require external services should be represented by deterministic
  local stubs or mocked service implementations.

## Touchpoints

- CLI parser and help surface.
- CLI presentation and JSON output.
- Local skill and graph execution.
- Official skill resolution and local package resolution.
- Registry CE and remote registry behavior.
- Tool catalogs and bundled tools.
- MCP server/client protocol paths.
- Receipts, ledger, artifacts, history, inspect, replay, and diff.
- Sandbox admission and enforcement metadata.
- Managed agent and caller-mediated resolution.
- Developer commands: `dev`, `doctor`, `tool build`, and harness fixtures.

## Risks

- High: without a matrix, a Rust rebuild can appear successful while dropping
  low-frequency workflows such as `export-receipts`, `mcp serve`, `dev`, or
  receipt verification.
- Medium: snapshot tests can be too brittle if they include timestamps,
  absolute paths, receipt IDs, or platform-specific wording.
- Medium: the matrix can become stale unless `help.ts`, `args.ts`, and docs are
  checked against it.
- Low: creating the matrix may reveal existing undocumented behavior; those
  cases should be classified rather than hidden.

## Acceptance

Profile: strict

Definition of done:
- [ ] `dod1` `fixtures/cli-parity` contains a command/feature matrix covering
  every current CLI command and runtime surface listed in scope.
- [ ] `dod2` TypeScript oracle tests execute or validate every matrix entry.
- [ ] `dod3` The matrix distinguishes schema-exact JSON/receipt parity from
  semantic human-output parity.
- [ ] `dod4` The architecture doc states that no Rust CLI/runtime cutover is
  allowed without passing the matrix.
- [ ] `dod5` Future Rust candidate test requirements are documented but not
  required in this task.

Validation:
- [ ] `v1` test - CLI feature parity oracle passes.
  - Command: `pnpm exec vitest run --config vitest.config.ts tests/cli-feature-parity.test.ts`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v2` command - feature matrix check is clean.
  - Command: `pnpm exec tsx scripts/generate-cli-feature-parity.ts --check`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v3` command - all help commands are represented in the matrix.
  - Command: `pnpm exec tsx scripts/generate-cli-feature-parity.ts --check-help-coverage`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v4` command - architecture doc contains no-cutover parity rule.
  - Command: `rg -n 'one-to-one|feature parity|No npm-to-Rust CLI cutover|Rust Migration Rules' ../docs/trusted-kernel-package-truth.md`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v5` command - fast verification remains green.
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none

## Phase 1: Command inventory

Goal: Create a complete command/flag inventory from current code and docs.

Status: pending
Dependencies: none

Changes:
- `fixtures/cli-parity/README.md` (all, exclusive) - Document matrix fields,
  oracle expectations, and normalization rules.
- `fixtures/cli-parity/commands.json` (all, exclusive) - Inventory every
  command, alias, subcommand, required positional, flag, JSON support, expected
  exit codes, and side-effect class.
- `scripts/generate-cli-feature-parity.ts` (all, exclusive) - Generate or check
  matrix coverage against `packages/cli/src/help.ts`, `packages/cli/src/args.ts`,
  and docs.

Acceptance:
- [ ] `ac1_1` command - every help command is represented.
  - Command: `pnpm exec tsx scripts/generate-cli-feature-parity.ts --check-help-coverage`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac1_2` command - matrix names all documented exit codes.
  - Command: `rg -n '\"exitCodes\".*0|\"exitCodes\".*1|\"exitCodes\".*2|\"exitCodes\".*64|needs_resolution|usage' fixtures/cli-parity docs/cli-exit-codes.md`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Oracle fixtures

Goal: Add executable TypeScript oracle cases for representative behavior in
each command family.

Status: pending
Dependencies: Phase 1

Changes:
- `fixtures/cli-parity/cases/*.json` (all, exclusive) - Add normalized oracle
  cases for help, usage failure, skill run, graph run, harness, inspect,
  history, replay, diff, list, doctor, dev, tool, mcp, registry, config, init,
  new, knowledge, export-receipts, and resume/needs-resolution behavior.
- `tests/cli-feature-parity.test.ts` (all, exclusive) - Execute or validate
  oracle cases through the current TypeScript CLI with deterministic temp
  workspaces and normalization.

Acceptance:
- [ ] `ac2_1` test - oracle cases pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts tests/cli-feature-parity.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2_2` command - oracle cases avoid local machine paths and secrets.
  - Command: `! rg -n '/Users/|/private/|OPENAI_API_KEY|ANTHROPIC_API_KEY|RUNX_AGENT_API_KEY|sk-[A-Za-z0-9]' fixtures/cli-parity`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Runtime and adapter parity classes

Goal: Ensure non-help behavior is classified before any Rust runtime work.

Status: pending
Dependencies: Phase 2

Changes:
- `fixtures/cli-parity/runtime-surfaces.json` (all, exclusive) - Classify
  runtime behavior by owner: core, runtime-local, adapters, CLI presentation,
  host-adapters, and external service stubs.
- `fixtures/cli-parity/cases/*.json` (partial, shared) - Mark each case with
  surfaces it proves and gaps it intentionally does not prove.

Acceptance:
- [ ] `ac3_1` command - all adapter surfaces are represented.
  - Command: `rg -n 'cli-tool|MCP|A2A|catalog|managed agent|agent|tool-catalog|receipt|sandbox|ledger|artifact' fixtures/cli-parity`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 4: Cutover rule documentation

Goal: Make the 1:1 feature bar visible to future Rust rewrite tasks.

Status: pending
Dependencies: Phase 3

Changes:
- `../docs/trusted-kernel-package-truth.md` (partial, shared) - Add or refine
  Rust migration rules and no-cutover rule.
- `README.md` (partial, shared) - Add developer note only if it improves CLI
  contributor clarity.
- `crates/README.md` (partial, shared) - Explain that Rust CLI candidates must
  pass the feature matrix before replacing npm CLI behavior.

Acceptance:
- [ ] `ac4_1` command - docs contain no-cutover language.
  - Command: `rg -n 'one-to-one|feature parity|No npm-to-Rust CLI cutover|no Rust CLI/runtime cutover' ../docs README.md crates`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

Strategy: per_phase

Commands:
- Remove `fixtures/cli-parity/**`.
- Remove `tests/cli-feature-parity.test.ts`.
- Remove `scripts/generate-cli-feature-parity.ts`.
- Revert docs updates that mention the CLI feature parity matrix.

## Review

Status: not_started
Verdict: none
Timestamp: none
Review rounds: none
Reviewer mode: none
Reviewer session: none
Round status: none
Override applied: none
Override reason: none
Override confirmed at: none
Reviewed head: none
Reviewed dirty: none
Reviewed diff: none
Blocking count: none
Non-blocking count: none

Findings:
- none

Passes:
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
- cli
- feature-parity
- rewrite

## Origin

Source:
- user requested review of specs and architecture doc to ensure the rebuild is
  1:1 feature-wise.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- precedes: future-rust-native-cli
- related_to: rust-runx-cli-placeholder
- related_to: rust-kernel-parity-fixtures
- related_to: rust-parity-ci-governance

## Harden Rounds

- none

## Planning Log

- 2026-05-15T13:24:00Z: Drafted after reviewing Rust parity specs and the
  trusted-kernel architecture doc for 1:1 feature parity coverage.
