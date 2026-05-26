---
spec_version: '2.0'
task_id: runx-source-command-contracts-v1
created: '2026-05-26T12:12:06Z'
updated: '2026-05-26T12:27:59Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# Runx source command contracts

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-26T12:27:59Z
Review gate: pass

## Summary

Add a small, reusable runx core source-command layer for developer issue
inbox entrypoints. The layer normalizes Slack, GitHub, Sentry, file/API/manual
source references into provider-neutral runx command inputs, derives the stable
thread/source locators needed by existing `runx.signal.v1`,
`runx.operational_policy.v1`, thread story, and target-runner contracts, and
renders concise provider-safe command responses.

This is a child slice of `runx-rust-95-release-readiness`. It should reduce the
amount of bespoke Nitrosend wrapper code without moving Nitrosend policy into
runx core and without creating a second public schema source that can drift
from the Rust contract spine.

## Objectives

- Provide a reusable `@runxhq/core/source` module for source locator parsing,
  request normalization, dedupe key construction, and command response
  projection.
- Support the source kinds already proven by live Nitrosend dogfood:
  GitHub issue/PR URLs or locators, Slack permalinks/thread locators, Sentry
  issue/event URLs or locators, local/manual/file/API sources, and unsupported
  source diagnostics.
- Preserve target repository ownership when a GitHub source URL names a repo;
  default repos must not overwrite a concrete URL repo.
- Make unsupported or blocked command paths explicit and safe for Slack or
  other chat surfaces: no raw provider JSON, no secrets, no local absolute
  paths, and no false "dispatched" claim.
- Keep provider mutation, live Slack/Sentry channel policy, owner routing,
  credentials, and deployment configuration outside core.
- Document how consuming repos should map source events into runx source
  commands before `issue-intake`, `issue-to-pr`, `pr-review`, or
  `merge-assist`.

## Scope

- In scope:
  - `packages/core/src/source/**`
  - `packages/core/src/index.ts`
  - `packages/core/package.json`
  - focused core tests for source normalization and safe response rendering
  - docs that clarify where source-command normalization sits relative to
    `runx.signal.v1`, operational policy, thread story, and target runners
- Out of scope:
  - product-specific Nitrosend Slack/Sentry filters, owner maps, channel names,
    GitHub Projects, or deployment settings
  - provider mutations such as Slack replies, GitHub issues, PRs, comments, or
    post-merge publication
  - changing Rust-generated JSON schemas, generated contract artifacts, or the
    active Rust target-runner/post-merge contracts
  - automatic PR merge
  - backwards-compatible aliases for old or unclear public names

## Dependencies

- Active umbrella spec: `.scafld/specs/active/runx-rust-95-release-readiness.md`.
- Existing core helpers:
  - `packages/core/src/knowledge/thread-story.ts`
  - `packages/core/src/knowledge/outbox.ts`
  - `packages/contracts/src/schemas/operational-policy.ts`
  - `crates/runx-contracts/src/signal.rs`
  - `crates/runx-contracts/src/target_runner.rs`
- Node 20+, pnpm workspace dependencies, and the current TypeScript test
  toolchain.

## Assumptions

- The source-command layer is a typed helper and adapter boundary, not a new
  stable machine packet. Existing Rust-owned contracts remain the public schema
  source of truth.
- Product adapters can provide extra provider context after this layer, but
  they should do so through evidence artifacts, thread context, signal refs, or
  operational policy inputs rather than custom hidden fields.
- Slack thread hydration and Sentry payload hydration are adapter concerns.
  Core can normalize locators and report missing capability; it must not fetch
  from providers.
- A concrete GitHub URL is stronger evidence than an ambient/default repo.
- Safe command responses are intended for busy developer channels, so they must
  be compact, actionable, and sanitized by default.

## Touchpoints

- `packages/core/package.json`
- `packages/core/src/index.ts`
- `packages/core/src/knowledge/index.ts`
- `packages/core/src/source/index.ts`
- `packages/core/src/source/index.test.ts`
- `docs/developer-issue-inbox.md`
- `docs/issue-to-pr.md`
- `docs/thread-story-contract.md`
- `.scafld/specs/active/runx-rust-95-release-readiness.md` for alignment only;
  do not edit it unless this child spec discovers a real contradiction.

## Risks

- Contract drift: adding a schema-like packet in TypeScript would conflict with
  the Rust contract spine. Keep this as helper logic and typed shapes only.
- Security leak: raw provider errors, local filesystem paths, or token-shaped
  strings could be echoed into Slack/GitHub. Sanitize response text and tests.
- False authority: source normalization must not imply permission to mutate a
  repo. Mutation still requires operational-policy admission and the target
  runner boundary.
- Over-centralization: runx core should not learn Nitrosend channel names,
  Sentry project ids, owner assignment, or deployment details.
- Duplicate work: the layer should compose with existing thread story,
  operational policy, and target-runner contracts instead of reimplementing
  them.

## Acceptance

Profile: standard

Validation:
- `pnpm exec vitest run --config vitest.fast.config.ts packages/core/src/source/index.test.ts packages/core/src/knowledge/index.test.ts`
- `pnpm typecheck`
- `git diff --check`

Completion bar:
- `@runxhq/core/source` is exported as a public subpath.
- GitHub, Slack, Sentry, local/manual/file/API, and unsupported inputs normalize
  into predictable source-command projections.
- GitHub URL parsing preserves the concrete owner/repo and issue/PR number.
- Slack permalink parsing produces a canonical thread locator without requiring
  a provider fetch.
- Sentry URL parsing produces a source locator and marks hydration as adapter
  owned.
- Safe response rendering strips local absolute paths, token-shaped values, and
  raw provider JSON while retaining a clear blocker and next action.
- Docs explain that this layer feeds `issue-intake`/`issue-to-pr` but does not
  own product policy or provider mutation.

## Phase 1: Core Source Command Module

Status: completed
Dependencies: none

Objective: Add the reusable typed source-command helpers and focused tests.

Changes:
- Create `packages/core/src/source/index.ts` with:
- Parse provider-native references without provider fetches: `github://owner/repo/pulls/N` style locators
- Export the module from `packages/core/package.json` and the root core index.
- Add tests for supported parsing, default repo precedence, unsupported source diagnostics, dedupe stability, and safe response redaction.

Acceptance:
- [x] `p1_ac1` command - Source module tests
  - Command: `pnpm exec vitest run --config vitest.fast.config.ts packages/core/src/source/index.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `p1_ac2` command - Core source typecheck slice
  - Command: `pnpm typecheck`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Integration Docs And Contract Alignment

Status: completed
Dependencies: phase1

Objective: Document the boundary and prove the new module composes with

Changes:
- Update developer issue inbox and issue-to-PR docs to place `@runxhq/core/source` before `issue-intake`, operational-policy admission, target-runner planning, and provider publishing.
- Update thread-story docs to state that source-command normalization supplies locators and safe command summaries, while the story/outbox layer remains the reviewer projection.
- Export existing thread-story helpers from `@runxhq/core/knowledge` if the docs/barrel export are misaligned, keeping the helper implementation in its existing module.
- Add or extend a core knowledge/source test only if needed to prove the source command output feeds existing thread story helpers without leaking raw local paths or provider payloads.
- Verify that the active Rust readiness spec remains aligned and that this work did not edit generated contract artifacts or Rust schema definitions.

Acceptance:
- [x] `p2_ac1` command - Source plus thread-story regression tests
  - Command: `pnpm exec vitest run --config vitest.fast.config.ts packages/core/src/source/index.test.ts packages/core/src/knowledge/index.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18
- [x] `p2_ac2` command - Typecheck
  - Command: `pnpm typecheck`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19
- [x] `p2_ac3` command - Whitespace safety
  - Command: `git diff --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20

## Rollback

- Remove `packages/core/src/source/**`, remove the `@runxhq/core/source`
  package export, and revert the docs. No migrations, provider state, or
  generated schemas are touched.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Discover-mode review of runx-source-command-contracts-v1 found no completion blockers. The new `@runxhq/core/source` module parses GitHub/Slack/Sentry/file/api/manual references without provider fetches, preserves concrete GitHub repo over defaultTargetRepo in `normalizeRunxSourceCommand`, builds stable dedupe keys via canonicalJsonStringify+sha256Prefixed, and the sanitizer redacts `(SENTRY_AUTH_TOKEN|GITHUB_TOKEN|GH_TOKEN|SLACK_BOT_TOKEN)=...`, ghp/gho/ghu/ghs/ghr_/github_pat_/xox*/nskey_*, Bearer tokens, and `/Users|/private/tmp|/tmp|/home|C:\` local paths plus structured-error payloads before they hit chat surfaces. Root `src/index.ts` only re-exports the new source module and `@runxhq/contracts` helpers; no trusted-kernel boundary violations (no imports of runtime-local, adapters, CLI, or cloud). Docs (`developer-issue-inbox.md`, `issue-to-pr.md`, `thread-story-contract.md`) place source-command normalization before `issue-intake`/operational-policy admission and explicitly retain hydration, channel policy, and credentials outside core. Ambient drift in `crates/runx-runtime/**` and `crates/runx-core/src/state_machine/**` is unrelated to this child task and tracked as context only.

Attack log:
- `packages/core/src/source/index.ts parseGithubSource/parseGithubPath`: Confirm `github://owner/repo/pulls/N` URL parsing yields the same canonical `github://owner/repo/pulls/N` as `https://github.com/owner/repo/pulls/N` and that defaultTargetRepo cannot overwrite a concrete URL repo (spec invariant). Traced url.protocol branch and `targetRepo = source.targetRepo ?? normalizeRepoSlug(options.defaultTargetRepo)`. -> clean (concrete repo wins; both URL and locator forms produce identical canonical locator and dedupe key (verified by test `builds stable source dedupe keys from canonical locators`).)
- `packages/core/src/source/index.ts sanitizeRunxCommandText`: Probe redaction coverage for the chat-surface invariants: ghp_/gho_/ghu_/ghs_/ghr_, github_pat_, xox[abprs]-, nskey_(live|test)_, Bearer <token>, env-style GITHUB_TOKEN=..., and `/Users|/private/tmp|/tmp|/home|C:\` local paths, plus structured-JSON payloads via `summarizeStructuredError`. Verified replacement order so token-prefixed env assignments are redacted before the ghp/gho regex would consume them and before the local-path regex would expose `=/Users/...`. -> clean (Order is safe; tested by `sanitizes command responses for chat surfaces` and the outbox-entry test which asserts the final JSON contains neither `/Users/kam` nor `ghp_` and does contain `[local path]`. Gaps (e.g., generic JWTs, GitHub Enterprise hosts) are out of scope for v1.)
- `packages/core/src/index.ts and packages/core/package.json`: Convention/boundary check: ensure `@runxhq/core` root and the new `./source` subpath export the module without importing `@runxhq/runtime-local`, adapters, CLI, or cloud code (AGENTS.md trusted-kernel rule). Grepped imports in `src/source/index.ts` (only `@runxhq/contracts`) and confirmed subpath wiring in `package.json` `exports`. -> clean (Root `src/index.ts` does `export * from "./source/index.js"` alongside `corePackage`; subpath `./source` resolves to `dist/src/source/index.{d.ts,js}` consistent with sibling domains.)
- `packages/core/src/source/index.test.ts integration with `@runxhq/core/knowledge``: Regression hunt: confirm source-command outputs feed `buildThreadStoryMessageOutboxEntry` and `readOutboxEntryControl` without leaking local paths or provider tokens through the thread-story barrel (knowledge module re-export alignment requested by spec). -> clean (Both helpers are re-exported from `packages/core/src/knowledge/index.ts`; outbox JSON assertion in the new test guards against `/Users/kam` and `ghp_` leakage end-to-end.)
- `docs/{developer-issue-inbox,issue-to-pr,thread-story-contract}.md and `.scafld/specs/active/runx-rust-95-release-readiness.md``: Scope-drift check: docs must place `@runxhq/core/source` before `issue-intake`/operational-policy admission/target-runner planning, and the active Rust readiness spec must not be edited or have generated contract artifacts touched. -> clean (Docs reference `@runxhq/core/source` ahead of intake/admission and explicitly keep hydration, credentials, and provider mutation outside core; no edits to Rust schema or generated contract artifacts observed in task changes.)

Findings:
- none

## Self Eval

- Target score: 9.0+ for this bounded core slice. A lower score means either
  the module leaked product policy into core, failed to sanitize command
  responses, or introduced schema drift.

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-26T12:13:48Z
Ended: 2026-05-26T12:15:01Z

Checks:
- path audit
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:35
  - Result: passed
  - Evidence: Existing core story helpers already carry thread, source, issue,
- command audit
  - Grounded in: code:package.json:33
  - Result: passed
  - Evidence: The declared validation commands resolve in the workspace:
- scope/migration audit
  - Grounded in: code:docs/issue-to-pr.md:19
  - Result: passed
  - Evidence: The existing issue-to-PR docs split reusable runx machinery from
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: Phase 1 acceptance is runnable immediately after the source module
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback is file-local: remove the new `source` module/export and
- design challenge
  - Grounded in: code:crates/runx-contracts/src/signal.rs:60
  - Result: passed
  - Evidence: The main design risk is TS/Rust contract drift. The spec explicitly

Issues:
- none


## Planning Log

- 2026-05-26: Inspected existing runx core knowledge helpers, thread-story
  docs, issue-to-PR docs, operational-policy schema, Rust signal contract, and
  target-runner source context. The best shape is a provider-neutral source
  command helper that feeds existing contracts, not a new provider pusher or a
  Nitrosend policy module.
- 2026-05-26: Kept Rust schema changes out of scope to avoid drifting the new
  Rust contract spine. If this helper later graduates into a stable packet,
  promote it through `crates/runx-contracts` and generated schemas in a separate
  hard-cutover spec.
