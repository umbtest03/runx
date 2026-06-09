---
spec_version: '2.0'
task_id: runx-cli-operator-ux-v1
created: '2026-06-09T15:36:59Z'
updated: '2026-06-09T16:09:26Z'
status: completed
harden_status: not_run
size: medium
risk_level: medium
---

# runx CLI operator UX overhaul

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-09T16:09:26Z
Review gate: pass

## Summary

Raise the `runx skill` operator path from "governed but sharp-edged" to a
strong 2026 CLI surface. The dogfood run proved the runtime spine works:
signing fails closed, agent-backed skills pause with structured
`needs_agent`, answer-file resume seals receipts, and graph/http skills can
fetch real provider evidence. It also exposed operator-facing defects that make
the product feel less capable than the runtime: documented `--input k=v` is
parsed incorrectly, local skills cannot be addressed by name, non-default
runners are not selectable from `runx skill`, exported Claude/Codex shims are
not friendly when invoked directly, and large graph output floods the terminal.

This spec fixes those CLI/operator affordances without changing the trusted
runtime model, receipt contract, policy gates, or skill package format.

## Objectives

- `runx skill <name>` resolves repo-local skill names such as
  `weather-forecast` to `skills/weather-forecast` when the current workspace
  contains that package.
- `runx skill ... --input key=value` and repeated `--input key=value` behave as
  advertised, while existing direct input flags such as `--location ...` remain
  the ergonomic shorthand.
- `runx skill ... --runner <name>` selects non-default runners through the
  existing runtime override path.
- Directly invoking an exported orchestrator shim either resolves to the source
  runx skill or fails with a human-readable explanation. It must never fail with
  a missing `X.yaml` path that makes the operator debug generated internals.
- Non-JSON terminal output for `runx skill` is concise and operator-first:
  status, skill, run id, receipt id, pending requests or key summary. Full JSON
  remains available through `--json`.
- Help text and tests describe the real 2026 UX surface.

## Scope

- In scope:
  - `crates/runx-cli` argument parsing, help text, skill ref resolution, and
    terminal projection for `runx skill`.
  - `crates/runx-runtime` request plumbing only where needed to expose already
    implemented runner selection through `LocalOrchestrator`.
  - Focused Rust tests for parser behavior, skill name resolution, runner
    selection, exported shim invocation, and terminal projection.
  - Local dogfood commands for `weather-forecast`, `brand-voice`, and
    `nws-weather-forecast`.
- Out of scope:
  - New receipt schema versions or compatibility aliases.
  - New skill package schema.
  - Network/provider behavior changes beyond exercising existing NWS read-only
    skill execution.
  - Claude/Codex app internals beyond generated shim invocation behavior.
  - Registry installation semantics.

## Dependencies

- Existing Rust runtime signing env remains required:
  `RUNX_RECEIPT_SIGN_KID`, `RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64`, and
  `RUNX_RECEIPT_SIGN_ISSUER_TYPE`.
- Existing `SkillRunOverrides.runner` in `crates/runx-runtime` is the canonical
  runner selection implementation; this work only exposes it to the CLI.

## Assumptions

- Operators expect CLI documentation to be literal. If help says
  `--input k=v`, that form must work.
- `--json` remains the machine contract and must continue printing the full
  current JSON payload.
- Human terminal output may be concise without changing sealed receipt content
  or the `--json` API.
- Bare skill names resolve only in clear local contexts. Ambiguous or missing
  refs fail closed with an actionable message.

## Touchpoints

- `crates/runx-cli/src/skill.rs`
- `crates/runx-cli/src/skill/inputs.rs`
- `crates/runx-cli/src/skill/output.rs`
- `crates/runx-cli/src/skill/parser.rs`
- `crates/runx-cli/src/skill/resolver.rs`
- `crates/runx-cli/src/launcher.rs`
- `crates/runx-cli/tests/skill.rs`
- `crates/runx-cli/tests/launcher.rs`
- `crates/runx-runtime/src/execution/orchestrator.rs`
- `crates/runx-runtime/tests/skill_run.rs`
- `docs/orchestrator-integrations.md`

## Risks

- Adding a `runner` field to runtime request construction can create compile
  churn across tests. Keep it optional and thread it through the existing
  override path.
- Concise non-JSON output must not hide failures or receipt identifiers.
- Exported shim source resolution uses generated metadata; invalid or stale
  source paths must fail closed rather than executing a wrong skill.
- Avoid turning skill-name resolution into a registry fallback. This is local
  operator convenience, not implicit remote install.

## Acceptance

Profile: standard

Validation:
- `cd crates && cargo test -p runx-cli 'skill::'`
- `cd crates && cargo test -p runx-cli launcher`
- `cd crates && cargo test -p runx-runtime skill_run`
- `pnpm exec vitest run packages/cli/src/index.test.ts --config vitest.fast.config.ts`
- `pnpm fixtures:cli-parity:check`
- `pnpm fixtures:cli-help:check`
- `pnpm rust:style`
- Dogfood signed local execution:
  - `runx skill weather-forecast --input location="Sydney, AU" ... --json`
    pauses with structured inputs, then resumes and seals with an answer file.
  - `runx skill skills/nws-weather-forecast --runner forecast --office LWX
    --grid-x 97 --grid-y 71` prints concise terminal output without dumping the
    full NWS JSON unless `--json` is supplied.
  - `runx skill ~/.claude/skills/weather-forecast ...` resolves to the exported
    source skill or fails with an explicit generated-shim message.

## Phase 1: CLI Skill Inputs and Runner Selection

Status: completed
Dependencies: none

Objective: Make `runx skill` parse the documented input and runner selection

Changes:
- Add `runner: Option<String>` to `SkillPlan`.
- Parse `--runner <name>` and `--runner=<name>` for skill runs.
- Parse repeated `--input key=value` and `--input key value` into real input keys; reject malformed `--input`.
- Preserve existing direct `--field value` / `--field=value` input flags.
- Expose runner selection through `LocalOrchestrator` and use it to populate the existing `SkillRunOverrides.runner` path without adding duplicate CLI execution logic.
- Update help text and launcher tests.

Acceptance:
- [x] `ac1` command - CLI parser and launcher tests
  - Command: `cd crates && cargo test -p runx-cli 'skill::' && cargo test -p runx-cli launcher`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-cli skill --all-features -- --test-threads=1` and `cargo test --manifest-path crates/Cargo.toml -p runx-cli launcher --all-features -- --test-threads=1` passed.
- [x] `ac2` command - Runtime skill-run tests
  - Command: `cd crates && cargo test -p runx-runtime skill_run`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime skill_run --all-features -- --test-threads=1` passed.

## Phase 2: Skill Ref Resolution and Exported Shim UX

Status: completed
Dependencies: Phase 1

Objective: Let operators address local skills naturally and make exported shim
invocation comprehensible.

Changes:
- Resolve bare skill refs by checking local `skills/<name>` before treating the
  value as an invalid path.
- Resolve `SKILL.md` paths and package directories consistently.
- Detect generated exported shim packages without an `X.yaml`; if they carry a
  `runx-export:* source=...` marker and the source package is valid, execute the
  source package.
- If the generated source is missing/invalid, fail with an explicit message that
  says the exported shim should be re-exported.
- Add focused tests with temp skills and generated shim fixtures.

Acceptance:
- [x] `ac3` command - Skill ref resolution tests
  - Command: `cd crates && cargo test -p runx-cli 'skill::'`
  - Expected kind: `exit_code_zero`
  - Status: pass

## Phase 3: Operator Projection and Docs

Status: completed
Dependencies: Phase 2

Objective: Make terminal output readable for humans while preserving the JSON
contract.

Changes:
- Keep `--json` unchanged.
- For non-JSON `runx skill`, print concise status-oriented output:
  `status`, `skill`, `run_id`, `receipt_id`, pending request ids, and summary
  fields when present.
- Avoid dumping large provider payloads in non-JSON output.
- Update orchestrator integration docs with the new CLI examples and the
  recommended operator flow.

Acceptance:
- [x] `ac4` command - Non-JSON projection test
  - Command: `cd crates && cargo test -p runx-cli 'skill::'`
  - Expected kind: `exit_code_zero`
  - Status: pass
- [x] `ac5` command - Style check
  - Command: `pnpm rust:style`
  - Expected kind: `exit_code_zero`
  - Status: pass

## Phase 4: Dogfood

Status: completed
Dependencies: Phase 3

Objective: Prove the operator flow works against shipped skills.

Changes:
- Run local signed dogfood for:
  - `weather-forecast` by bare name with `--input`.
  - `brand-voice` by bare name with direct flags.
  - `nws-weather-forecast` with explicit `--runner forecast`.
  - exported Claude `weather-forecast` shim path.

Acceptance:
- [x] `ac6` command - Dogfood commands complete as expected
  - Command: `RUNX_RECEIPT_SIGN_KID=runx-demo-key RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64=QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI= RUNX_RECEIPT_SIGN_ISSUER_TYPE=hosted <dogfood commands>`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: bare `weather-forecast` and `brand-voice` paused with structured `needs_agent`; bare `taste-profile` printed concise pending output; exported Codex `taste-profile` resolved to the source skill; `nws-weather-forecast --runner forecast` sealed with an isolated receipt dir.

## Rollback

- Revert CLI/parser/runtime request plumbing changes. Existing path-based
  `runx skill skills/<name>` execution and `--json` payloads should remain
  recoverable because the runtime execution model is not replaced.

## Review

Status: completed
Verdict: pass

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- none
