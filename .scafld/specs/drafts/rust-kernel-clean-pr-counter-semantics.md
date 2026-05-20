---
spec_version: '2.0'
task_id: rust-kernel-clean-pr-counter-semantics
created: '2026-05-20T00:00:00Z'
updated: '2026-05-20T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: medium
---

# Rust kernel clean-PR counter semantics

## Current State

Status: draft
Current phase: none
Next: harden before approve if the promotion evidence path needs counter
semantics locked before `rust-kernel-blocking-promotion` runs.
Reason: `scripts/count-clean-kernel-prs.ts`, fixture data, and tests exist, but
the promotion path depends on audited semantics for advisory-start evidence,
qualifying PR classification, required passing evidence, and fixture-mode
operator evidence. This slice hardens those semantics without flipping CI.
Blockers: none for semantics hardening. Live GitHub evidence and CI promotion
remain blocked in `rust-kernel-blocking-promotion`.
Allowed follow-up command: `scafld harden rust-kernel-clean-pr-counter-semantics`
Latest runner update: none
Review gate: not_started

## Summary

Lock the semantics of the clean-kernel PR counter before its output is used as
promotion evidence. This is a focused pre-promotion audit slice. It may adjust
the counter, fixture, and tests so the rules are explicit and fail closed, but
it must not remove `continue-on-error: true` or declare Phase B active.

## Context

Grounded current facts:
- `scripts/count-clean-kernel-prs.ts` exists.
- `tests/count-clean-kernel-prs.test.ts` covers fixture-mode counting, rust-only
  and parser-only exclusion, missing evidence, advisory-start requirements, and
  minimum-count failure.
- `tests/fixtures/clean-kernel-prs.json` supplies local audited evidence.
- `rust-kernel-blocking-promotion` still owns live advisory-start evidence,
  five qualifying post-advisory PRs, and the CI flip.

## Scope

In scope:
- Precisely define which PR file changes count toward the five-clean-PR gate.
- Preserve fail-closed behavior when advisory-start evidence is missing.
- Require explicit passing evidence for counting PRs.
- Keep Rust-only, parser-only, missing-evidence, and outside-scope PRs
  non-counting.
- Add fixture tests for ambiguous cases that promotion reviewers are likely to
  challenge, such as mixed TypeScript kernel plus deliberate fixture refreshes.
- Record the final semantics in this spec for handoff to
  `rust-kernel-blocking-promotion`.

Out of scope:
- Live GitHub API integration unless harden explicitly narrows it to read-only
  evidence collection.
- Removing `continue-on-error: true` from CI.
- Changing `rust-kernel-blocking-promotion` evidence thresholds.
- Runtime, parser, receipt, SDK, or CLI cutover.

## Semantics To Lock

- Advisory start must be explicit from CLI input or audited fixture data; never
  infer it from file timestamps or git history.
- A counting PR must touch authoritative TypeScript state-machine or policy
  files, or must be an explicit deliberate kernel fixture refresh tied to that
  surface.
- Rust-only maintenance PRs remain advisory evidence but do not count toward the
  five-PR trigger.
- Parser-only PRs do not count toward the current five-PR trigger.
- Missing, skipped, failed, renamed, or ambiguous parity evidence makes the PR
  non-counting unless an audited fixture supplies explicit operator evidence.
- Mixed TypeScript kernel plus deliberate fixture refresh PRs must have an
  explicit classification and fixture test; they must not fall through as
  outside-scope by accident.

## Acceptance

Profile: strict
Self-eval threshold: 8
Review provider: external Claude; local review does not satisfy complete.
Harden required before approve: yes

Definition of done:
- [ ] `dod1` Counter classification rules are captured in code tests and this
  spec.
- [ ] `dod2` Ambiguous mixed kernel/fixture cases are either counted or rejected
  by an explicit, tested rule.
- [ ] `dod3` Missing advisory-start evidence and missing passing evidence fail
  closed.
- [ ] `dod4` CI remains advisory.
- [ ] `dod5` `rust-kernel-blocking-promotion` remains the only owner of the
  live five-PR evidence and CI flip.

Validation:
- [ ] `v1` command - counter tests pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts tests/count-clean-kernel-prs.test.ts`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: 60
  - Status: pending
- [ ] `v2` command - fixture-mode counter still passes at the audited local
  threshold.
  - Command: `pnpm exec tsx scripts/count-clean-kernel-prs.ts --fixture tests/fixtures/clean-kernel-prs.json --min 3`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: 60
  - Status: pending
- [ ] `v3` command - missing advisory-start evidence remains rejected.
  - Command: `pnpm exec tsx -e "import { analyzeCleanKernelPrs } from './scripts/count-clean-kernel-prs.ts'; try { analyzeCleanKernelPrs({ prs: [] }); process.exit(1); } catch (error) { process.exit(String(error instanceof Error ? error.message : error).includes('missing advisory start evidence') ? 0 : 1); }"`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: 30
  - Status: pending
- [ ] `v4` command - CI remains advisory after this slice.
  - Command: `rg -n 'Advisory Rust kernel parity|continue-on-error: true' .github/workflows/ci.yml`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 1: Semantics Audit

Goal: document and test the exact classification rules.

Status: pending
Dependencies: none

Expected changes:
- `tests/count-clean-kernel-prs.test.ts` adds any missing ambiguous-case tests.
- `tests/fixtures/clean-kernel-prs.json` adds only audited local evidence needed
  for those tests.
- `scripts/count-clean-kernel-prs.ts` changes only if the audited tests expose a
  semantic gap.

## Phase 2: Promotion Handoff

Goal: leave `rust-kernel-blocking-promotion` with an unambiguous counter
contract.

Status: pending
Dependencies: Phase 1

Expected changes:
- This spec records the final semantics and validation evidence.
- No CI promotion occurs in this phase.

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

## Metadata

Estimated effort hours: 3
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- trusted-kernel
- ci
- advisory
- promotion-evidence

## Origin

Source:
- split from obsolete `rust-kernel-port-orchestration` after observing that the
  clean-kernel counter exists but promotion evidence semantics still need to be
  trusted independently.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- follows: rust-kernel-port-orchestration
- hands_off_to: rust-kernel-blocking-promotion

## Harden Rounds

- none
