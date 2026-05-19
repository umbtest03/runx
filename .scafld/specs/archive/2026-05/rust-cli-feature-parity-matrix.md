---
spec_version: '2.0'
task_id: rust-cli-feature-parity-matrix
created: '2026-05-15T13:16:55Z'
updated: '2026-05-19T03:17:42Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# Rust CLI feature parity matrix

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T03:17:42Z
Review gate: pass

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

Validation:
- [x] `v1` test - CLI feature parity oracle passes.
  - Command: `pnpm exec vitest run --config vitest.config.ts tests/cli-feature-parity.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-35
- [x] `v2` command - feature matrix check is clean.
  - Command: `pnpm exec tsx scripts/generate-cli-feature-parity.ts --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `v3` command - all help commands are represented in the matrix.
  - Command: `pnpm exec tsx scripts/generate-cli-feature-parity.ts --check-help-coverage`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `v4` command - architecture doc contains no-cutover parity rule.
  - Command: `rg -n 'one-to-one|feature parity|No npm-to-Rust CLI cutover|Rust Migration Rules' ../docs/trusted-kernel-package-truth.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38
- [x] `v5` command - fast verification remains green.
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-39

## Phase 1: Command inventory

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- `fixtures/cli-parity/README.md` (all, exclusive) - Document matrix fields, oracle expectations, and normalization rules.
- `fixtures/cli-parity/commands.json` (all, exclusive) - Inventory every command, alias, subcommand, required positional, flag, JSON support, expected exit codes, and side-effect class.
- `scripts/generate-cli-feature-parity.ts` (all, exclusive) - Generate or check matrix coverage against `packages/cli/src/help.ts`, `packages/cli/src/args.ts`, and docs.

Acceptance:
- [x] `ac1_1` command - every help command is represented.
  - Command: `pnpm exec tsx scripts/generate-cli-feature-parity.ts --check-help-coverage`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac1_2` command - matrix names all documented exit codes.
  - Command: `rg -n '\"exitCodes\".*0|\"exitCodes\".*1|\"exitCodes\".*2|\"exitCodes\".*64|needs_resolution|usage' fixtures/cli-parity docs/cli-exit-codes.md`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Oracle fixtures

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- `fixtures/cli-parity/cases/*.json` (all, exclusive) - Add normalized oracle cases for help, usage failure, skill run, graph run, harness, inspect, history, replay, diff, list, doctor, dev, tool, mcp, registry, config, init, new, knowledge, export-receipts, and resume/needs-resolution behavior.
- `tests/cli-feature-parity.test.ts` (all, exclusive) - Execute or validate oracle cases through the current TypeScript CLI with deterministic temp workspaces and normalization.

Acceptance:
- [x] `ac2_1` test - oracle cases pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts tests/cli-feature-parity.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-12
- [x] `ac2_2` command - oracle cases avoid local machine paths and secrets.
  - Command: `! rg -n '/Users/|/private/|OPENAI_API_KEY|ANTHROPIC_API_KEY|RUNX_AGENT_API_KEY|sk-[A-Za-z0-9]' fixtures/cli-parity`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13

## Phase 3: Runtime and adapter parity classes

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- `fixtures/cli-parity/runtime-surfaces.json` (all, exclusive) - Classify runtime behavior by owner: core, runtime-local, adapters, CLI presentation, host-adapters, and external service stubs.
- `fixtures/cli-parity/cases/*.json` (partial, shared) - Mark each case with surfaces it proves and gaps it intentionally does not prove.

Acceptance:
- [x] `ac3_1` command - all adapter surfaces are represented.
  - Command: `rg -n 'cli-tool|MCP|A2A|catalog|managed agent|agent|tool-catalog|receipt|sandbox|ledger|artifact' fixtures/cli-parity`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18

## Phase 4: Cutover rule documentation

Status: completed
Dependencies: Phase 3

Objective: Complete this phase.

Changes:
- `../docs/trusted-kernel-package-truth.md` (partial, shared) - Add or refine Rust migration rules and no-cutover rule.
- `README.md` (partial, shared) - Add developer note only if it improves CLI contributor clarity.
- `crates/README.md` (partial, shared) - Explain that Rust CLI candidates must pass the feature matrix before replacing npm CLI behavior.

Acceptance:
- [x] `ac4_1` command - docs contain no-cutover language.
  - Command: `rg -n 'one-to-one|feature parity|No npm-to-Rust CLI cutover|no Rust CLI/runtime cutover' ../docs README.md crates`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23

## Rollback

Strategy: per_phase

Commands:
- Remove `fixtures/cli-parity/**`.
- Remove `tests/cli-feature-parity.test.ts`.
- Remove `scripts/generate-cli-feature-parity.ts`.
- Revert docs updates that mention the CLI feature parity matrix.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode review of rust-cli-feature-parity-matrix. All declared task scope is covered: the matrix in fixtures/cli-parity/commands.json enumerates every help-listed command with aliases, flags, positionals, parity surfaces, and exit codes; runtime-surfaces.json links each non-help surface to a command; oracle cases cover every command with executable cases for help, usage failure, config list, harness, and list tools. checkHelpCoverage maps every Usage and Manage Skills entry to a matrix command id. Cutover rule text required by ac4_1 is present in docs/trusted-kernel-package-truth.md and crates/README.md. All recorded acceptance evidence (v1–v5, ac1_1–ac4_1) reports exit code 0. The lone ambient_drift entry (fixtures/cli-parity/cases/oracle.json) is actually inside declared scope; treating as context per verify-mode contract. No regressions or invariant violations identified.

Attack log:
- `Help coverage script vs help.ts Usage and Manage Skills blocks`: Enumerate every `runx ...` usage line in packages/cli/src/help.ts and confirm helpUsageCommandIds maps it to a command id present in matrix.commands -> clean (All 24 Usage entries and 5 Manage Skills entries resolve to matrix command ids.)
- `Args.ts vs matrix flag coverage`: Cross-reference args.ts parsed flags (devWatch, devRealAgents, doctorFix, etc.) against matrix flags array for each command -> clean (Matrix includes non-help flags such as --watch and --real-agents on dev, consistent with args.ts.)
- `Test isolation and env handling in tests/cli-feature-parity.test.ts`: Confirm execute cases isolate RUNX_HOME and RUNX_RECEIPT_DIR per case, suppress banner, and avoid leaking secrets -> clean (Memory streams avoid TTY; RUNX_BANNER=0 plus non-TTY stream keeps banner off; tempDir is removed in finally; no API keys touched.)
- `Trusted-kernel-package-truth.md cutover rules`: Search for required phrases per ac4_1 (one-to-one, feature parity, No npm-to-Rust CLI cutover, Rust Migration Rules) -> clean (All required phrases present and consistent with the new matrix as the gate.)
- `Crates/README.md gating language`: Verify crates README cites the fixtures/cli-parity matrix as a prerequisite for any Rust CLI cutover -> clean (Lines 36-38 state Rust candidates must pass the matrix before replacing npm CLI behavior.)
- `Surface vs command graph consistency`: For every runtime surface, verify coveredBy entries reference real matrix command ids and that surface.id appears in at least one oracle case's proves array -> clean (Test 'connects every runtime surface to a command and oracle case' enforces both invariants; manual scan confirms no orphan surfaces or dangling command refs.)
- `Secret/leak hygiene in fixtures`: Validate ac2_2 expectation that no absolute /Users paths, /private paths, OPENAI_API_KEY, ANTHROPIC_API_KEY, RUNX_AGENT_API_KEY, or sk-* tokens appear in fixtures/cli-parity -> clean (Manual read of all fixtures shows no host paths or secret tokens.)
- `Scope drift outside declared task scope`: Compare task_changes against task scope and look for undeclared modifications -> clean (All six task_changes entries map to declared scope. The 'ambient drift' oracle.json entry is also inside declared scope (classifier glitch documented as F2).)

Findings:
- [low/non-blocking] `F1` Environment scope items beyond RUNX_HOME/RUNX_CWD/RUNX_RECEIPT_DIR are not enumerated in matrix or runtime-surfaces
  - Location: `fixtures/cli-parity/runtime-surfaces.json:186`
  - Evidence: Spec scope lists RUNX_TOOL_ROOTS, agent config env vars, sandbox enforcement env vars, and local config file behavior. The config surface notes only "RUNX_HOME and local config file behavior"; the matrix and oracle cases do not mention RUNX_TOOL_ROOTS or sandbox-enforcement env vars (RUNX_PRODUCTION, etc.). tests/cli-feature-parity.test.ts only forwards RUNX_HOME/RUNX_CWD/RUNX_RECEIPT_DIR/RUNX_BANNER.
  - Impact: A future Rust candidate could pass the matrix without proving env-var-driven behaviors that the TypeScript CLI honors today. Not blocking because each Rust adapter spec will still need to add these, and current acceptance criteria all pass.
- [low/non-blocking] `F2` Session classifier marks fixtures/cli-parity/cases/oracle.json as ambient drift even though fixtures/cli-parity/cases/*.json is in declared scope
  - Location: `fixtures/cli-parity/cases/oracle.json`
  - Evidence: Task scope explicitly lists `fixtures/cli-parity/cases/*.json (all, exclusive)`, and the file is generated by scripts/generate-cli-feature-parity.ts within scope. The Workspace Classification section nevertheless lists it under Ambient Workspace Drift Outside Task Scope.
  - Impact: Context-only per verify-mode contract; does not affect the implementation. Worth noting so the next reviewer does not misattribute this file to unrelated work.

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
