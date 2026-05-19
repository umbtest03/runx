---
spec_version: '2.0'
task_id: runx-operational-policy-config
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T04:52:11Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Runx operational policy config

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T04:52:11Z
Review gate: pass

## Summary

Create a typed runx operational policy contract for repository routing,
ownership assignment, runner availability, Slack/source-thread routing, outcome
handling, dedupe behavior, and allowed automation actions. The policy must be
machine-validated, safe to display in Aster/admin surfaces, and explicit enough
that adopter repos like Nitrosend only supply thin product-specific mappings.

## Context

CWD: `.` (runx OSS workspace)

Current production learnings from Nitrosend:
- Issue intake can originate in Slack, Sentry, GitHub comments, or manual
  commands.
- The source issue/thread can differ from the target repo receiving the PR.
- Owner routing needs to be explicit and reviewable.
- Slack follow-ups must return to the original source thread, not channel root.
- Human review/merge gates must stay explicit for mutating code changes.
- Dedupe and outcome behavior need policy, not ad hoc script branches.

Candidate core surfaces:
- `skills/issue-intake/**`
- `skills/issue-to-pr/**`
- `skills/work-plan/**`
- `packages/cli/**`
- `crates/runx-runtime/**`
- Aster policy/admin read models

Hardening evidence from the current workspace:
- `packages/contracts/src/schemas/operational-policy.ts` already defines the
  `runx.operational_policy.v1` TypeBox schema, semantic lint findings, and a
  readback projection.
- `packages/cli/src/commands/policy.ts` reads policy files as JSON, calls the
  contracts helpers, and renders `policy inspect|lint` results.
- `packages/cli/src/args.ts`, `packages/cli/src/dispatch.ts`, and
  `packages/cli/src/help.ts` expose `runx policy inspect|lint <policy.json>`.
- `fixtures/operational-policy/nitrosend-like.json` exists as the current
  Nitrosend-like fixture, but the fixture `schema` literal must stay identical
  to the generated contract's `schema` and `schema_version` literals.

Invariants:
- Policy is not a secret store. Tokens, credentials, and private keys stay in
  runtime secrets.
- Unknown target repos, unknown runners, or unknown Slack routes fail closed.
- Policy distinguishes review-only automation, PR-producing automation, and any
  future merge-capable automation.
- Repo-local wrappers may narrow policy but should not reimplement core routing.
- Core accepts only `runx.operational_policy.v1` policy packets at cutover. No
  legacy aliases, dual-read parsers, or runtime compatibility shims.
- The initial checked-in policy format is JSON. YAML support would be a separate
  approved change because the current CLI policy command parses JSON only.
- Validation has two layers: policy-file semantic lint, and request-time
  admission for a concrete source, target repo, action, and runner.

## Objectives

- Define a versioned policy schema for routing, owners, runners, Slack/source
  threads, dedupe, and outcomes.
- Add validation errors that are actionable for operators.
- Provide a readback/projection suitable for Aster/admin surfaces.
- Add fixture policies for Nitrosend-like multi-repo routing and minimal
  single-repo routing.
- Add a CLI readback/lint gate (`runx policy inspect|lint`) so Aster,
  adopters, and CI can validate policy before enabling mutation.
- Document where adopter-specific config ends and runx core behavior begins.
- Add request-time admission helpers so `issue-intake`, `issue-to-pr`, and PR
  packaging reject unknown target repos or runners before any mutation boundary.

## Scope

In scope:
- Policy schema and parser.
- Validation for target repos, runner names, owner mappings, channel/thread
  routes, outcome settings, dedupe keys, and automation permissions.
- CLI/runtime helpers that consume the policy without duplicating parsing.
- Request-time policy admission for source, target repo, action, runner, dedupe,
  and source-thread requirements.
- Redacted policy readback from the CLI and contract package.
- Aster-facing safe projection shape.
- Tests and fixtures.

Out of scope:
- Secrets management implementation.
- Actual target-repo runner execution; owned by `runx-target-repo-runners`.
- Post-merge deploy observation; owned by `runx-post-merge-outcome-observer`.
- Nitrosend-specific copy, labels, and channel names beyond fixtures.
- YAML policy loading, legacy policy aliases, or runtime fallback parsing for
  adopter-specific policy files. One-off conversion scripts or fixtures are
  acceptable; compatibility paths in runx core are not.

## Dependencies

- Coordinates with `runx-target-repo-runners`.
- Coordinates with `runx-post-merge-outcome-observer`.
- Feeds `rust-nitrosend-dogfood` and `rust-aster-runtime-cutover`.

## Assumptions

- JSON is the initial policy format because the current CLI command parses JSON
  and the generated public schema artifact is JSON Schema.
- Existing adopter policy files are migration inputs, not runtime-compatible
  contracts. They are converted once to `runx.operational_policy.v1` or kept in
  repo-local wrappers until that repo cuts over.
- Aster can initially consume a static policy read model before a full admin UI
  editor exists.

## Touchpoints

- `packages/contracts/src/schemas/operational-policy.ts` (schema, semantic
  lint, request-admission types if kept in contracts).
- `schemas/operational-policy.schema.json` (generated schema artifact).
- `packages/contracts/src/schemas/operational-policy.test.ts` (positive,
  negative, readback, and admission tests).
- `fixtures/operational-policy/nitrosend-like.json`.
- `fixtures/operational-policy/minimal-single-repo.json` (new).
- `fixtures/operational-policy/invalid-*.json` (new negative fixtures).
- `packages/cli/src/commands/policy.ts`, `packages/cli/src/args.ts`,
  `packages/cli/src/dispatch.ts`, and `packages/cli/src/help.ts`.
- `skills/issue-intake/SKILL.md` and `skills/issue-intake/X.yaml`.
- `skills/issue-to-pr/SKILL.md` and `skills/issue-to-pr/X.yaml`.
- `packages/cli/tools/outbox/build_pull_request/src/index.ts` and
  `packages/cli/tools/outbox/build_feed_entry/src/index.ts`.
- `docs/developer-issue-inbox.md` and `docs/issue-to-pr.md`.

## Risks

- A permissive default could authorize automation in the wrong repo.
- Config drift between runx core and adopter wrappers could recreate the
  current duplication.
- Overfitting to Nitrosend would make the core policy less reusable.

## Acceptance

Profile: strict

Validation:
- `pnpm contracts:schemas:check`
- `pnpm test`
- `pnpm --filter @runxhq/cli run runx policy lint fixtures/operational-policy/nitrosend-like.json --json`
- `pnpm --filter @runxhq/cli run runx policy lint fixtures/operational-policy/minimal-single-repo.json --json`
- `cargo test --manifest-path crates/Cargo.toml`
- `git diff --check`

Required behavior:
- [ ] `schema` and `schema_version` both use the exact literal
  `runx.operational_policy.v1` in every positive fixture and generated artifact.
- [ ] Unknown target repo fails request-time policy admission before PR
  packaging or runner dispatch.
- [ ] Unknown runner fails policy-file semantic lint and request-time admission.
- [ ] Missing source-thread routing fails when Slack/GitHub follow-up publishing
  is enabled.
- [ ] Owner routing is explicit for each target or target class.
- [ ] Dedupe strategy is explicit for PR-producing flows.
- [ ] Outcome strategy is explicit for merged, closed-unmerged, failed verify,
  superseded, and provider-observation-missing states.
- [ ] Policy command exits nonzero for semantic findings and prints machine
  readable finding codes in `--json` mode.
- [ ] Policy projection redacts secrets and local paths.
- [ ] `runx policy inspect` returns redacted source locator counts, not raw
  Slack/Sentry/provider locators.
- [ ] `runx policy lint` exits non-zero for semantic policy failures.
- [ ] Nitrosend-like fixture can express existing API/App/workspace routing with
  no custom parser.
- [ ] Minimal single-repo fixture proves the schema is reusable without
  Nitrosend-only fields.
- [ ] No code path reads legacy adopter policy formats as a fallback.

## Phase 1: Schema

Status: completed
Dependencies: none

Objective: Make operational policy explicit and validated.

Changes:
- Add or finish the versioned JSON schema contract in `packages/contracts/src/schemas/operational-policy.ts`.
- Generate `schemas/operational-policy.schema.json` from the contract.
- Add typed validation findings for duplicate IDs, unknown runner references, owner-route mismatches, runner target mismatches, runner availability, scafld mismatch, missing source-thread policy, and unsafe outcome close rules.
- Add positive fixtures for Nitrosend-like routing and minimal single-repo routing.
- Add negative fixtures for unknown runner, owner-route mismatch, source-thread missing, no available runner for target action, invalid schema literal, and extra secret-like fields.

Acceptance:
- none

## Phase 2: Core Consumption

Status: completed
Dependencies: Phase 1

Objective: Make issue-intake and issue-to-PR flows consume policy through one

Changes:
- Add a shared request-admission helper that evaluates `(source_id, target_repo, action, runner_id)` against one validated policy.
- Reject unknown target repos, unknown runners, disabled/maintenance-only runners, source actions not allowed by policy, and missing source-thread routing before `outbox.build_pull_request` can produce a mutation-ready packet.
- Thread admitted policy context into `issue-intake`, `issue-to-pr`, PR package metadata, and outcome-story construction without duplicating JSON parsing.
- Keep adopter-specific owner names, Slack locators, Sentry projects, and labels as policy data, not hardcoded skill logic.

Acceptance:
- none

## Phase 3: Readback Surface

Status: completed
Dependencies: Phase 2

Objective: Expose policy safely to operators and Aster.

Changes:
- Add or finish the safe projection/readback command through `runx policy inspect|lint <policy.json>`.
- Document the admin-visible fields in `docs/developer-issue-inbox.md` and `docs/issue-to-pr.md`.
- Keep raw provider locators, absolute local paths, tokens, and private keys out of projection output and human CLI output.

Acceptance:
- none

## Rollback

- Revert only the files listed in Touchpoints for this spec.
- Remove generated `schemas/operational-policy.schema.json` if the source
  contract is reverted.
- For a bad checked-in policy, operators repair by editing the JSON policy and
  running `runx policy lint <policy.json> --json` until findings are empty.
- For a bad live route, operators can set runner `state` to `disabled` or remove
  the PR-producing action from the source/target policy, then relint and
  redeploy the policy file. This stops new mutation without adding code
  fallback.
- If an adopter migration fails, restore its last known working repo-local
  wrapper outside runx core and rerun the one-off conversion. Do not add
  runtime dual-read, alias, or compatibility parsing to runx.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: The operational policy contract, semantic lint, admission helper, CLI gate, fixtures, and skill wiring satisfy the task scope. Schema rejects extras (secret-field path), positive/negative fixtures behave as tested, admission denies unknown target/runner/source-thread paths, and the policy admission summary in build_pull_request avoids raw locators. The CLI inspect/lint command is wired through args/dispatch/help with --json support. Three small, non-blocking concerns: (1) `pr-review` is omitted from the source_thread_required lint despite the docs treating pr-review as a thread-publishing lane; (2) `runner_target_mismatch` and `mutation_without_scafld` semantic findings have no test or negative fixture coverage; (3) `admitOperationalPolicyRequest` blends whole-policy lint findings with request-specific findings, so a noisy policy taints any admission output. None block completion.

Attack log:
- `operationalPolicySchema additionalProperties`: Inject root-level github_token via invalid-secret-field.json and confirm schema rejects + admittance path is gated by validateOperationalPolicyContract. -> clean (Root object and every nested object use additionalProperties:false; test 'rejects extra fields so secrets do not drift into policy' covers root path.)
- `operationalPolicySchema literals`: Try schema='runx.operational-policy.v1' (dash variant) and permissions.auto_merge=true to confirm literal types reject. -> clean (invalid-schema-literal.json and dedicated test both fail Value.Check; require_human_merge_gate is Literal(true) and auto_merge Literal(false).)
- `lintOperationalPolicyContract semantic findings`: Walk all four negative fixtures through the lint to ensure first finding code matches the expected RegExp in validateOperationalPolicySemantics. -> clean (Found ordering preserves unknown_runner, owner_route_target_mismatch, source_thread_required, target_action_without_runner as first-fired codes; tests pass.)
- `lintOperationalPolicyContract pr-review coverage`: Search for pr-review in the source_thread_required gate to see if it shares the same publishing requirement as issue-to-pr/pr-fix-up/merge-assist. -> finding (Logged op-policy-pr-review-thread-lint-gap.)
- `lintOperationalPolicyContract coverage tests`: Grep test file for runner_target_mismatch and mutation_without_scafld assertions. -> finding (Logged op-policy-lint-test-coverage-gap.)
- `admitOperationalPolicyRequest`: Trace selectRequestSource/selectRequestTarget/selectRequestRunner and confirm denial codes when source_id/target_repo/runner_id are missing or unknown, including the source-thread locator requirement. -> clean (Codes unknown_source, unknown_target_repo, unknown_runner, target_runner_not_allowed, source_thread_locator_required all fire; runner_required is added when no available runner serves the requested action.)
- `admitOperationalPolicyRequest finding scope`: Inspect whether lint findings bleed into request-time denials. -> finding (Logged op-policy-admission-finding-bleed.)
- `projectOperationalPolicyReadback secret/locator redaction`: Check that raw allowed_locators (e.g., slack://team/T123/channel/CBUGS) never appear in the readback projection used by runx policy inspect. -> clean (Projection emits locator_count only; readback test explicitly asserts JSON.stringify(readback) does not contain the raw Slack locator.)
- `outbox.build_pull_request admission path`: Confirm that when a policy is supplied with no source_thread_locator, packaging fails closed before mutation-ready outbox emission. -> clean (admitPolicyRequest throws on deny; summarizePolicyAdmission only persists redacted fields (policy_id, source_id, target_repo, runner_id, owner_route_id, owner_count, dedupe_strategy, outcome_close_mode, source_thread_required, mutate_target_repo, require_human_merge_gate).)
- `CLI runx policy inspect|lint dispatch`: Walk args.ts (isPolicy branch), dispatch.ts policy guard, handlePolicyCommand error paths, and help.ts usage to ensure all five surfaces stay aligned. -> clean (args declares policyAction/policyPath and effectiveInputs drops policy flags; dispatch returns 0/1 based on findings count; runCli try/catch converts schema/JSON errors to renderCliError; help/usage line 53 and example line 78 document inspect/lint.)
- `Domain boundary check`: Confirm operational-policy.ts only depends on TypeBox helpers in ../internal and does not reach into runtime/CLI surfaces. -> clean (Imports limited to ./internal.js; no runtime-local, adapters, or cloud imports.)
- `Schemas artifact generation`: Diff schemas/operational-policy.schema.json against operationalPolicySchema for additionalProperties, required fields, and literal constraints. -> clean (Generated artifact matches the TypeBox definition (additionalProperties:false at every object, const for schema/schema_version/auto_merge/require_human_merge_gate/missing_behavior, required arrays mirror Type.Optional usage).)

Findings:
- [low/non-blocking] `op-policy-pr-review-thread-lint-gap` Lint omits pr-review from the source_thread_required check even though pr-review publishes back to the source thread.
  - Location: `packages/contracts/src/schemas/operational-policy.ts:300`
  - Evidence: operational-policy.ts:300 hardcodes the PR-producing action list as [issue-to-pr, pr-fix-up, merge-assist]. docs/issue-to-pr.md describes pr-review as 'reads the source thread ... then publishes one concise review packet' and source-thread publishing is part of the security boundary. A policy that allows pr-review with source_thread.required=false or publish_mode=none currently passes lint.
  - Impact: An adopter could ship a policy that admits pr-review against a source whose thread cannot be recovered, defeating the source-thread fail_closed contract for the review lane.
  - Validation: Add a negative fixture or unit test with pr-review allowed + source_thread.required=false and assert source_thread_required is reported.
- [low/non-blocking] `op-policy-lint-test-coverage-gap` Two declared semantic findings have no negative fixture or unit test.
  - Location: `packages/contracts/src/schemas/operational-policy.test.ts`
  - Evidence: Grep for `runner_target_mismatch` and `mutation_without_scafld` in operational-policy.test.ts returns no matches, yet both codes are emitted by lintOperationalPolicyContract (operational-policy.ts lines 344-356 and 392-398). The task spec lists 'runner target mismatches' and 'scafld mismatch' as required typed validation findings.
  - Impact: Regressions to those branches would not be caught by the contract test suite, weakening the operator-actionability guarantee on the validation findings.
  - Validation: pnpm --dir oss test against packages/contracts shows the new cases fail before and pass after the lint branches are exercised.
- [low/non-blocking] `op-policy-admission-finding-bleed` admitOperationalPolicyRequest concatenates whole-policy lint findings with request-specific findings, so unrelated policy noise contaminates request-time denials.
  - Location: `packages/contracts/src/schemas/operational-policy.ts:421`
  - Evidence: operational-policy.ts:421 seeds the admission findings array with lintOperationalPolicyContract(policy). The 'denies maintenance-only runner' test (operational-policy.test.ts:387-408) relies on expect.arrayContaining because the same policy also raises target_action_without_runner for every target during lint.
  - Impact: Callers receive admission denials that bundle codes (target_action_without_runner, owner_route_target_mismatch, etc.) that are unrelated to the specific (source, target, runner, action) tuple they submitted, which is harder for operators to act on than a request-scoped denial.
  - Validation: Add a test where the policy is lint-clean and confirm admission.findings only ever contain request-scoped codes for that path.

## Self Eval

- Target score: 9.5. Passing means operators can understand and audit what runx
  is allowed to do before it does it.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: move reusable production routing/policy into runx core

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T04:06:54Z
Ended: 2026-05-19T04:14:46Z

Checks:
- path audit
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:173
  - Result: passed
  - Evidence: Exact contracts, generated schema, fixture, CLI, skill, outbox, and docs paths are named in Touchpoints.
- command audit
  - Grounded in: code:packages/cli/src/help.ts:53
  - Result: passed
  - Evidence: Acceptance uses the exposed policy command shape plus schema, test, Rust, and diff checks.
- scope/migration audit
  - Grounded in: code:.scafld/core/config.yaml:13
  - Result: passed
  - Evidence: Legacy aliases, dual-read parsers, and runtime compatibility shims are forbidden; old adopter policies are conversion inputs only.
- acceptance timing audit
  - Grounded in: code:package.json:19
  - Result: passed
  - Evidence: Phase 1 gates schema generation and fixture lint before Phase 2 admission and Phase 3 readback.
- rollback/repair audit
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:41
  - Result: passed
  - Evidence: Rollback is limited to listed touchpoints; repair is policy edit plus `runx policy lint`, runner disable, or action removal.
- design challenge
  - Grounded in: code:packages/cli/tools/outbox/build_pull_request/src/index.ts:88
  - Result: passed
  - Evidence: Unknown target repo is now request-time admission before PR packaging, not only schema lint.

Issues:
- [medium/advisory] `issue-1` high - Fixture schema literal drift would make a positive fixture fail the public schema contract.
  - Status: open
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:175
  - Evidence: The contract requires `schema` to equal `runx.operational_policy.v1`, so fixtures must use the same schema and schema_version literal.
  - Recommendation: Pin exact literals in required behavior and negative fixture coverage.
- [medium/advisory] `issue-2` high - Unknown target repo cannot be proven by policy-file lint alone.
  - Status: open
  - Grounded in: code:packages/cli/tools/outbox/build_pull_request/src/index.ts:88
  - Evidence: PR packaging derives `targetRepo` from input or thread URI, so a valid policy file can still be used with an unlisted request target.
  - Recommendation: Add request-time admission for source, target, action, and runner before PR packaging.
- [medium/advisory] `issue-3` medium - The original YAML/JSON assumption conflicted with the current CLI command path.
  - Status: open
  - Grounded in: code:packages/cli/src/commands/policy.ts:106
  - Evidence: `parseJson` calls `JSON.parse`; no YAML loader is used by the policy command.
  - Recommendation: Make JSON the initial format and put YAML behind a separate approved change.
- [medium/advisory] `issue-4` medium - Minimal single-repo fixture was required by the draft but is not present in current fixtures.
  - Status: open
  - Grounded in: code:fixtures/operational-policy/nitrosend-like.json:1
  - Evidence: The current fixture directory contains the Nitrosend-like fixture; reusable-policy acceptance also needs a non-Nitrosend shape.
  - Recommendation: Require `fixtures/operational-policy/minimal-single-repo.json` in Touchpoints, validation commands, and Phase 1 acceptance.


## Planning Log

- 2026-05-19: Expanded placeholder into policy-config contract after Nitrosend
  production dogfood review.
- 2026-05-19: Added the core contract target:
  `runx.operational_policy.v1`, redacted readback, semantic lint, and a CLI
  `policy inspect|lint` gate. Rust/Aster parity must use this contract surface
  rather than inventing an adopter-local policy shape.
