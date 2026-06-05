---
spec_version: '2.0'
task_id: runx-release-readiness-v1
created: '2026-06-05T03:25:35Z'
updated: '2026-06-05T03:25:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# runx-release-readiness-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: final OSS readiness lane before hosted/product launch work
Blockers: gate hardening and demo prune should land first
Allowed follow-up command: `scafld approve runx-release-readiness-v1`
Latest runner update: none
Review gate: not_started

## Summary

Prove that a fresh checkout and packaged artifacts are release-ready. This is the
boring but decisive lane: install, build, help, first skill, receipts, package
exports, release archives, docs links, and no stale spec/docs references. It does
not add new product behavior.

## Objectives

- Fresh checkout path works exactly as README says.
- Published package shape contains only intended files and exports.
- Release archive smoke works for the native binary.
- Demo verifier and receipt docs are coherent.
- Stale docs/spec references are either fixed or archived with clear status.

## Scope

In scope:
- README, docs/getting-started, docs/demos, docs/api-surface, package manifests.
- `scripts/check-cli-package-contract.mjs`,
  `scripts/check-rust-cli-release-artifacts.ts`, release workflow smoke checks.
- Fresh checkout scripts and docs link checks.

Out of scope:
- Hosted ops, live-funded rails, new demos.

## Dependencies

- Gate hardening and demo prune should define the canonical gate/demo list.
- Existing release workflow and package contract checks.

## Assumptions

- Native Rust CLI remains the trusted path; npm wrapper is a distribution/UX shim.
- Release readiness should be reproducible without private credentials.

## Risks

- **Local-only success, package failure.** Mitigation: test packed artifacts and
  archives, not only workspace commands.
- **Docs drift.** Mitigation: add link/command guards where cheap.

## Acceptance

Profile: strict

Validation:
- Fresh checkout path in README works.
- `runx --help`, first skill, harness, receipt verify, package contracts, and
  release archive smoke pass.
- No stale active/draft/archive status confusion in public docs.

## Phase 1: Fresh checkout smoke

Status: pending
Dependencies: runx-readiness-gate-hardening-v1

Objective: prove README commands.

Acceptance:
- [ ] `p1_ac1` command - fresh checkout build and first skill
  - Command: `pnpm install --frozen-lockfile && pnpm build && cargo build --manifest-path crates/Cargo.toml -p runx-cli && crates/target/debug/runx skill examples/hello-world --message "release smoke" --non-interactive --json`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Package and archive smoke

Status: pending
Dependencies: Phase 1

Objective: prove the shipped artifact shape.

Acceptance:
- [ ] `p2_ac1` command - package contracts pass
  - Command: `node scripts/check-cli-package-contract.mjs && pnpm authoring:check-package-contract`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - release artifact check passes
  - Command: `pnpm exec tsx scripts/check-rust-cli-release-artifacts.ts`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Docs and final gates

Status: pending
Dependencies: Phase 2

Objective: release docs and gates agree.

Acceptance:
- [ ] `p3_ac1` command - full release readiness gate
  - Command: `pnpm verify:fast && pnpm demos:check && pnpm x402:dogfood:local`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Revert docs/check changes together. Do not leave release-only scripts unwired.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: codex

## Origin

Created by: Codex
Source: operator readiness queue

## Harden Rounds

- none

## Planning Log

- none
