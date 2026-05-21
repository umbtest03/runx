---
spec_version: '2.0'
task_id: rust-nitrosend-dogfood
created: '2026-05-18T00:00:00Z'
updated: '2026-05-21T16:51:50Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# Rust nitrosend dogfood

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T16:51:50Z
Review gate: pass

## Summary

Use Nitrosend as the external-shaped dogfood for the Rust runtime cutover, but
keep the plan honest about what exists in this OSS checkout today.

Current local facts:

- Generic runtime fixtures exist for the upstream `issue-intake` and
  `issue-to-pr` skills under `fixtures/runtime/skills/issue-intake/` and
  `fixtures/runtime/skills/issue-to-pr/`.
- A reusable operational policy contract exists as
  `runx.operational_policy.v1`, with CLI support in
  `packages/cli/src/commands/policy.ts` and schemas/fixtures under
  `schemas/operational-policy.schema.json` and `fixtures/operational-policy/`.
- `fixtures/operational-policy/nitrosend-like.json` is present, but it is only
  Nitrosend-like. It now includes `nitrosend/nitrosend`, `nitrosend/api`, and
  `nitrosend/app` so policy lint/inspect cover the real target set.
- `fixtures/external/nitrosend/issue-intake/api-source-thread.json` exists as
  the first sanitized external-shaped fixture contract. It cites the generic
  runtime fixtures, the Nitrosend-like policy, the target-runner planning and
  dedupe lookup contracts, and the post-merge harness receipt fixture.
- The current reusable follow-on work is not complete: target-repo runner
  live execution and the post-merge closure observer runtime are both draft
  specs.

The dogfood goal is preservation, not a new adopter flow. The real Nitrosend
workflow may keep using the human/product names `issue-intake` and
`issue-to-pr`, but cutover execution artifacts must be canonical harness,
decision, act, proof, and sealed `runx.harness_receipt.v1` objects only.

This spec must not invent live Nitrosend traffic capture. Initial validation is
fixture parity against checked-in, sanitized, deterministic external-shaped
fixtures. Production soak is a later confirmation step after fixture, policy,
target-runner, and post-merge observer gates are green.

Execution status: this is a dogfood holding spec until
`runx-target-repo-runners` and `runx-post-merge-closure-observer` are active.
The executable build phase can ratify current fixture, policy, and wrapper
evidence only; live target PR creation and final observation stay deferred.

## Context

CWD: `.` (runx OSS workspace)

Relevant existing local surfaces:

- `skills/issue-intake/SKILL.md`
- `skills/issue-intake/X.yaml`
- `skills/issue-to-pr/SKILL.md`
- `skills/issue-to-pr/X.yaml`
- `fixtures/runtime/skills/issue-intake/cases/*.yaml`
- `fixtures/runtime/skills/issue-to-pr/cases/*.yaml`
- `fixtures/operational-policy/nitrosend-like.json`
- `packages/contracts/src/schemas/operational-policy.ts`
- `schemas/operational-policy.schema.json`
- `packages/cli/src/commands/policy.ts`
- `.scafld/specs/drafts/runx-target-repo-runners.md`
- `.scafld/specs/drafts/runx-post-merge-closure-observer.md`

Missing local surfaces this plan must create or wait for:

- `fixtures/external/nitrosend/issue-intake/**`
- `crates/runx-runtime/tests/external/nitrosend_issue_intake.rs`
- Target PR creation through the reusable target-repo runner contract.
- Post-merge/provider observation through sealed closure/proof receipts.

## Invariants

- No compatibility reader or writer translates retired outcome/effect/report
  artifacts into harness receipts at runtime.
- Product copy, skill names, slash-command text, and source-thread comments may
  say `issue-intake` or `issue-to-pr`; persisted execution contract ids,
  fixture artifact kinds, outbox contract payload ids, and Rust structs must not
  use those names as legacy execution artifact identifiers.
- External fixtures are deterministic and sanitized. They are not captured live
  during CI, and they do not depend on private Nitrosend repositories being
  checked out.
- `runx.operational_policy.v1` is the policy boundary. Nitrosend-specific JSON
  can be an input to a one-time conversion, but core runtime behavior must not
  dual-read adopter policy formats.
- Missing source-thread metadata fails closed before any Slack/GitHub final
  publication.
- Target completion, failed verification, source issue closure, and final
  source-thread replies are modeled through contained act closure plus
  proof-bound verification on sealed harness receipts.

## Objectives

- Add an external-shaped Nitrosend fixture suite that layers the existing
  generic `issue-intake`/`issue-to-pr` runtime fixtures into a realistic
  multi-target policy scenario.
- Extend the Nitrosend-like operational policy fixture to include
  `nitrosend/api` and prove workspace, API, and app target routing.
- Validate `runx policy lint` and `runx policy inspect` against the
  Nitrosend-like policy without leaking raw provider locators.
- Coordinate with `runx-target-repo-runners` so target PR creation, dedupe, and
  source-thread metadata are reusable core behavior.
- Coordinate with `runx-post-merge-closure-observer` so merge/close/deploy
  observation, final publication, and source issue closure are sealed
  closure/proof receipts, not a Nitrosend-only observer path.
- Add Rust runtime replay coverage only after the external-shaped fixture and
  the reusable runner/observer contracts exist.

## Scope

In scope:

- Plan and fixture contract for Nitrosend-shaped issue intake.
- Policy fixture coverage for `nitrosend/api`.
- Runtime parity expectations for canonical harness receipts.
- Explicit dependency on target-repo runner and post-merge closure observer
  work.

Out of scope:

- Editing Nitrosend production repositories.
- Live external traffic capture.
- Compatibility with retired peer terminal artifacts, including legacy
  outcome/effect packets and report-shaped verification payloads.

## Dependencies

- `runx-contract-spine-hard-cutover` for canonical harness, decision, act,
  proof, and receipt shapes.
- `rust-runtime-skeleton` and `rust-runtime-skill-execution`.
- `rust-receipt-proof-verification`, `rust-receipt-tree-resolution`, and
  `rust-runtime-receipt-path-discovery`.
- `runx-operational-policy-config` as the completed policy contract.
- `runx-target-repo-runners` before target dispatch parity can pass.
- `runx-post-merge-closure-observer` before final closure/proof parity can
  pass.

## Acceptance Criteria

- [x] Existing generic `issue-intake` and `issue-to-pr` runtime fixtures remain
  green and are referenced by the external-shaped Nitrosend fixture plan.
- [x] `fixtures/external/nitrosend/issue-intake/**` exists and contains
  sanitized deterministic inputs, not live external capture.
- [x] The Nitrosend-like operational policy fixture covers
  `nitrosend/nitrosend`, `nitrosend/api`, and `nitrosend/app`.
- [x] `runx policy lint fixtures/operational-policy/nitrosend-like.json`
  accepts the policy, and `runx policy inspect` redacts raw provider locators.
- [x] No fixture, schema id, persisted receipt, or replay expectation uses
  retired peer terminal artifacts, legacy outcome/effect packet fields, or
  report-shaped verification payloads; completion is represented as harness
  receipt closure with `proof.verification`.
- [x] Live target PR creation, final observation, and external replay are
  explicitly deferred to the reusable target-runner and observer specs.

## Deferred Follow-Up Gates

- Target PR creation must use `runx-target-repo-runners`; dedupe is represented
  in pull-request outbox `metadata.dedupe` and in the sealed receipt proof path.
- Target completion and final publication must use
  `runx-post-merge-closure-observer` only through closure/proof on sealed
  harness receipts.
- Replaying the same external-shaped fixture twice must produce identical
  canonical receipt bytes after normalized fixture ids/timestamps and pass proof
  verification once reusable runner/observer contracts exist.

## Phase 1: Ratify Current Dogfood Snapshot

Status: active
Dependencies: none

Objective: Re-run current OSS and sibling Nitrosend dogfood evidence, then

Changes:
- [x] Build the Rust CLI used by the policy commands.
- [x] Re-run Nitrosend-like fixture and policy validation.
- [x] Re-run the sibling Nitrosend wrapper suite when that checkout is present.
- [x] Keep live target-runner, observer, and deterministic external replay gates deferred.

Acceptance:
- none

## Validation Commands

Current local discovery/guard commands:

```sh
find fixtures/runtime/skills -maxdepth 4 -type f | sort
test -f fixtures/external/nitrosend/issue-intake/api-source-thread.json
cargo build --manifest-path crates/Cargo.toml -p runx-cli
cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test nitrosend_external_fixture -- --nocapture
./crates/target/debug/runx policy lint fixtures/operational-policy/nitrosend-like.json --json
./crates/target/debug/runx policy inspect fixtures/operational-policy/nitrosend-like.json --json
cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test operational_policy -- --nocapture
cargo test --manifest-path crates/Cargo.toml -p runx-contracts
! rg -n "runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:" fixtures/runtime fixtures/operational-policy fixtures/external skills crates/runx-runtime/src
scafld validate rust-nitrosend-dogfood --json
git diff --check -- .scafld/specs/active/rust-nitrosend-dogfood.md
```

2026-05-21 current refresh results:

- `find fixtures/runtime/skills -maxdepth 4 -type f | sort` passed and still
  lists the generic `issue-intake` and `issue-to-pr` runtime fixture cases.
- `test -f fixtures/external/nitrosend/issue-intake/api-source-thread.json`
  passed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test nitrosend_external_fixture -- --nocapture`
  passed: 2 tests.
- The documented `pnpm --filter @runxhq/cli run runx policy ...` commands are
  stale in the current checkout because `@runxhq/cli` exposes a `runx` binary
  but no `runx` npm script. The current safe invocation is the Rust binary
  directly.
- `./crates/target/debug/runx policy lint fixtures/operational-policy/nitrosend-like.json --json`
  passed with `status: "success"` and no findings.
- `./crates/target/debug/runx policy inspect fixtures/operational-policy/nitrosend-like.json --json`
  passed with `status: "success"` and no findings; source locators are redacted
  to counts.
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test operational_policy -- --nocapture`
  passed: 9 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts` passed. This
  includes contract-level target-runner and post-merge observer tests, but this
  is not live external replay evidence and does not clear the draft blockers.
- The original retired-artifact guard over `crates/runx-runtime` now false
  positives on `crates/runx-runtime/tests/external/aster_agent_step.rs`, where
  the retired names are rejection-test strings. The narrowed guard over
  `crates/runx-runtime/src` plus fixtures and skills passed.
- `scafld validate rust-nitrosend-dogfood --json` passed.
- In Nitrosend, `/Users/kam/dev/runx/runx/oss/crates/target/debug/runx policy lint config/runx-issue-flow.json --json`
  initially failed because `config/runx-issue-flow.json` still contained the
  retired top-level `post_merge` block. This was superseded by Nitrosend commit
  `b6770fd`, which moved the file to canonical `outcomes`.
- In Nitrosend, `/Users/kam/dev/runx/runx/oss/crates/target/debug/runx policy inspect config/runx-issue-flow.json --json`
  initially failed for the same retired `post_merge` surface, and now passes
  after `b6770fd`.
- In Nitrosend,
  `RUNX_BIN=/Users/kam/dev/runx/runx/oss/crates/target/debug/runx node --test scripts/onboarding.test.mjs scripts/segment-from-prose.test.mjs scripts/issue-intake.test.mjs scripts/github-issue-thread.test.mjs scripts/post-issue-intake-comments.test.mjs scripts/runx-target-outcome.test.mjs scripts/scafld-command-review.test.mjs scripts/runx-harness.test.mjs`
  passed: 122 tests, 0 skipped.
- In Nitrosend, the retired-artifact and fallback-knob guard passed:
  `! rg -n "needs_resolution|runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:|RUNX_JS_BIN|RUNX_NPM_PACKAGE|target_repositories|allowed_repositories|route_hints" scripts config fixtures/runx .github/workflows/issue-intake.yml .github/workflows/wrapper-ci.yml`.
- `git diff --check` passed in Nitrosend before this spec update.

2026-05-20 local refresh results:

- `find fixtures/runtime/skills -maxdepth 4 -type f | sort` passed and listed
  the existing generic `issue-intake` and `issue-to-pr` runtime fixtures.
- `test -f fixtures/external/nitrosend/issue-intake/api-source-thread.json`
  passed; a sanitized external-shaped Nitrosend fixture contract now exists.
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test nitrosend_external_fixture -- --nocapture`
  passed: 2 tests, deriving a target runner plan and provider dedupe lookup
  from the fixture and validating the cited post-merge harness receipt.
- A direct contract-level policy check against
  `fixtures/operational-policy/nitrosend-like.json` passed with no findings and
  projected a readback that redacts raw Slack/Sentry source locators.
- `pnpm vitest run packages/contracts/src/schemas/operational-policy.test.ts`
  passed: 23 tests.
- Static target coverage check passed for `nitrosend/nitrosend`,
  `nitrosend/api`, and `nitrosend/app` across targets, runner target repos, and
  owner routes.
- The retired-artifact guard passed:
  `! rg -n "runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:" fixtures/runtime fixtures/operational-policy skills crates/runx-runtime`.
- `git diff --check -- .scafld/specs/drafts/rust-nitrosend-dogfood.md fixtures/operational-policy/nitrosend-like.json packages/contracts/src/schemas/operational-policy.test.ts packages/cli/src/index.test.ts packages/cli/src/commands/policy.ts`
  passed before this evidence note was added.
- `pnpm build` passed and refreshed ignored workspace `dist` output after the
  receipts sunset removed `@runxhq/core/receipts`.
- `pnpm --filter @runxhq/cli run runx policy lint fixtures/operational-policy/nitrosend-like.json --json`
  passed with `status: "success"` and no findings.
- `pnpm --filter @runxhq/cli run runx policy inspect fixtures/operational-policy/nitrosend-like.json --json`
  passed with `status: "success"` and no findings.
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test operational_policy -- --nocapture`
  passed: 9 tests, including Rust request-admission coverage for all three
  Nitrosend-like target repos plus unknown-target and missing-source-thread
  fail-closed cases.
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts` passed.

2026-05-20 actual Nitrosend dogfood integration verification:

- Nitrosend `config/runx-issue-flow.json` now uses
  `runx.operational_policy.v1` directly for `nitrosend/nitrosend`,
  `nitrosend/api`, and `nitrosend/app`.
- Nitrosend issue-intake derives target workspaces from the operational policy
  `targets` list instead of the retired repo-local target-repository shape.
- Nitrosend harness fixtures now expect sealed `runx.harness_receipt.v1`
  receipts from the Rust binary.
- The actual Rust binary replayed the Nitrosend onboarding, segment, and
  issue-intake harness fixtures through `RUNX_BIN=.../crates/target/debug/runx`
  with no TypeScript fallback knobs.
- The generic runx `issue-intake` runtime fixtures were refreshed to the
  current Rust receipt closure reason (`process_closed`) and replay green.

Validation:

- 2026-05-22 refresh:
  - `cargo build --manifest-path crates/Cargo.toml -p runx-cli` rebuilt the
    local Rust binary.
  - `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test nitrosend_external_fixture -- --nocapture`
    passed: 2 tests.
  - `/Users/kam/dev/runx/runx/oss/crates/target/debug/runx policy lint config/runx-issue-flow.json --json`
    passed in the Nitrosend repo with no findings.
  - `/Users/kam/dev/runx/runx/oss/crates/target/debug/runx policy inspect config/runx-issue-flow.json --json`
    passed in the Nitrosend repo and redacted raw locators via counts.
  - `RUNX_BIN=/Users/kam/dev/runx/runx/oss/crates/target/debug/runx node --test scripts/onboarding.test.mjs scripts/segment-from-prose.test.mjs scripts/issue-intake.test.mjs scripts/github-issue-thread.test.mjs scripts/post-issue-intake-comments.test.mjs scripts/runx-target-outcome.test.mjs scripts/scafld-command-review.test.mjs scripts/runx-harness.test.mjs`
    passed in the Nitrosend repo: 122 tests, 0 skipped.
  - `scafld validate rust-nitrosend-dogfood --json` passed.
  - `git diff --check` passed in Nitrosend.

- `/Users/kam/dev/runx/runx/oss/crates/target/debug/runx policy lint config/runx-issue-flow.json --json`
  passed in the Nitrosend repo with no findings.
- `/Users/kam/dev/runx/runx/oss/crates/target/debug/runx policy inspect config/runx-issue-flow.json --json`
  passed in the Nitrosend repo and redacted raw locators via counts.
- `RUNX_BIN=/Users/kam/dev/runx/runx/oss/crates/target/debug/runx node --test scripts/onboarding.test.mjs scripts/segment-from-prose.test.mjs scripts/issue-intake.test.mjs scripts/github-issue-thread.test.mjs scripts/post-issue-intake-comments.test.mjs scripts/runx-target-outcome.test.mjs scripts/scafld-command-review.test.mjs scripts/runx-harness.test.mjs`
  passed in the Nitrosend repo: 122 tests, 0 skipped.
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts` passed in
  runx OSS.
- `! rg -n "needs_resolution|runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:|RUNX_JS_BIN|RUNX_NPM_PACKAGE|target_repositories|allowed_repositories|route_hints" scripts config fixtures/runx .github/workflows/issue-intake.yml .github/workflows/wrapper-ci.yml`
  passed in the Nitrosend repo.
- `! rg -n "runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:" fixtures/runtime fixtures/operational-policy fixtures/external skills crates/runx-runtime`
  passed in runx OSS.
- `git diff --check` passed in Nitrosend; `git diff --check -- fixtures/runtime/skills/issue-intake/cases .scafld/specs/drafts/rust-nitrosend-dogfood.md`
  passed in runx OSS.

Remaining blocker:

- Live target PR creation and final post-merge/provider observation still depend
  on the reusable `runx-target-repo-runners` and
  `runx-post-merge-closure-observer` work. No external replay was added here.

2026-05-21 Segment dogfood evidence verification:

- `scripts/segment-from-prose.mjs` now validates the returned sealed Runx
  result contains an embedded `runx.harness_receipt.v1` receipt and returns
  `receipt_id` to success and rejection callers.
- `scripts/segment-from-prose.test.mjs` now asserts the surfaced `receipt_id`
  and rejects non-harness receipt evidence.
- `RUNX_BIN=/Users/kam/dev/runx/runx/oss/crates/target/debug/runx node --test scripts/segment-from-prose.test.mjs scripts/runx-harness.test.mjs`
  passed in the Nitrosend repo: 12 tests, 0 skipped.

2026-05-20 narrow policy-fixture slice verification:

- The current safe slice is limited to verifying the Nitrosend-like policy and
  external dogfood fixture cover `nitrosend/api` alongside
  `nitrosend/nitrosend` and `nitrosend/app`.
- Static fixture coverage passed for all three repos across policy targets,
  runner `target_repos`, and owner-route `target_repos`.
- Focused Rust contract tests passed for operational policy admission,
  Nitrosend external fixture derivation, and target-runner planning/dedupe.
- TypeScript operational-policy fixture tests passed.
- CLI policy lint/inspect passed with no findings and redacted source
  locators.
- `scafld validate rust-nitrosend-dogfood --json` passed. `scafld build` and
  `scafld complete` were not run because this parent dogfood spec remains a
  draft with runtime replay acceptance still intentionally pending.

Future validation once the missing external fixture and runner/observer specs
land:

```sh
cargo test --manifest-path crates/Cargo.toml -p runx-runtime nitrosend_issue_intake
cargo test --manifest-path crates/Cargo.toml -p runx-receipts
```

## Rollback And Repair

- Before any Nitrosend launcher flip, rollback is to keep the existing
  production path unchanged and disable Rust side-by-side validation.
- If the Nitrosend-like policy fixture later omits a real target such as
  `nitrosend/api`, repair the policy fixture and policy admission tests rather
  than adding special-case runtime routing.
- If an external-shaped fixture is accidentally generated from live traffic,
  replace it with a sanitized deterministic fixture before merging.
- If retired artifact fields appear in fixtures or replay output, repair the
  producer and expected receipts. Do not whitelist retired fields.

## Open Questions

- Exact Rust binary pinning mechanism for Nitrosend CI.
- Whether Nitrosend wrapper behavior remains repo-local or is fully absorbed by
  reusable runx target-runner and post-merge observer code.
- Soak duration after fixture parity is green.

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-21T15:37:09Z
Ended: 2026-05-21T15:37:09Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: The draft is honest about its current state — completed [x] items (Nitrosend-like policy covers nitrosend/api, sanitized external fixture exists, runx-contracts nitrosend_external_fixture tests pass, policy lint/inspect green) are verifiable against the checkout. However, the spec has no `## Phases` section, so `scafld build` has nothing to advance through; the four remaining acceptance criteria are gated on two other draft specs (`runx-target-repo-runners`, `runx-post-merge-closure-observer`) without sequencing or ownership, and the validation block presupposes a `./crates/target/debug/runx` build artifact without making the prerequisite `cargo build -p runx-cli` an explicit pre-step. Approval is unsafe until the spec either explicitly stays a holding spec (with clear "no build phases until dependencies land" framing) or adds executable phases.

Checks:
- path audit
  - Grounded in: code:fixtures/operational-policy/nitrosend-like.json:60-128
  - Result: passed
  - Evidence: nitrosend-like.json now covers nitrosend/nitrosend, nitrosend/api, and nitrosend/app in runners.target_repos, owner_routes.target_repos, and the targets list, satisfying the [x] acceptance item for multi-target coverage.
- command audit
  - Grounded in: spec_gap:validation_commands
  - Result: failed
  - Evidence: Validation commands depend on the pre-built artifact `./crates/target/debug/runx` for `policy lint` and `policy inspect`. The 2026-05-22 refresh notes `cargo build --manifest-path crates/Cargo.toml -p runx-cli` was run, but the canonical Validation Commands block does not list the build as a prerequisite step. A reader running the block as-is from a fresh checkout will hit a missing-binary error before the policy lint call. Additionally, the 2026-05-20 evidence block still cites `pnpm --filter @runxhq/cli run runx policy ...`, which is acknowledged elsewhere in the spec as stale (no `scripts.runx` in packages/cli/package.json — only a `bin.runx` launcher).
- scope/migration audit
  - Grounded in: spec_gap:phases
  - Result: failed
  - Evidence: The harden context manifest reports `phases` body=0 (empty) and no `## Phases` section appears in the draft. The spec only carries an Acceptance Criteria checklist, of which 4 of 7 items are unchecked and depend on `runx-target-repo-runners` and `runx-post-merge-closure-observer` — both still draft. Without a phase plan, `scafld build` has nothing to open and there is no recorded sequencing between completed-now items and dependency-gated items. This is the central executability gap.
- acceptance timing audit
  - Grounded in: code:.scafld/specs/drafts/runx-target-repo-runners.md:1-40
  - Result: failed
  - Evidence: Acceptance items 'Target PR creation uses runx-target-repo-runners' and 'Target completion and final publication use runx-post-merge-closure-observer' and 'Replaying the same external-shaped fixture twice produces identical canonical receipt bytes' cannot pass until those parent draft specs themselves land. `runx-target-repo-runners` is still draft with blockers listed for live target checkout/git mutation, PR create/update, outbox pushers, and Aster scheduling. `runx-post-merge-closure-observer` is still draft with blockers for live observer transport, source/target readback, and publication transports. The dogfood spec does not specify a build-time ordering or a 'do not run before X' guard for these acceptance items.
- rollback/repair audit
  - Grounded in: code:.scafld/specs/drafts/rust-nitrosend-dogfood.md:354-364
  - Result: passed
  - Evidence: Rollback section is concrete: (1) keep production path unchanged and disable Rust side-by-side validation before any launcher flip, (2) repair the policy fixture if a target is omitted rather than special-casing routing, (3) replace any accidentally-live external fixture with a sanitized one, (4) repair producers and expected receipts rather than whitelisting retired fields. These are credible and code-local.
- design challenge
  - Grounded in: code:.scafld/specs/drafts/rust-nitrosend-dogfood.md:14-23
  - Result: passed
  - Evidence: The architectural posture (Nitrosend as dogfood, no live capture, dependency on reusable runner/observer contracts so this is preservation rather than a Nitrosend-only adopter path) is the right move and explicitly disclaims short-sighted bandaids. The risk is not architectural — it is the under-specified executability surface noted above.

Issues:
- [high/blocks approval] `harden-1` executability - No Planned Phases section — `scafld build` has nothing to enter.
  - Status: open
  - Grounded in: spec_gap:phases
  - Evidence: The harden context manifest shows phases body=0 and no `## Phases` heading exists in the draft. Acceptance Criteria is a flat checklist mixing items already satisfied [x] and items gated on two other draft specs. Without phases, the scafld build cycle cannot meaningfully advance this spec, and there is no recorded sequencing of when each pending acceptance item should be re-verified.
  - Recommendation: Either (a) add an explicit `## Phases` plan that maps each unchecked acceptance item to a phase with entry criteria referencing the parent draft spec it depends on, or (b) state explicitly under Current State that this spec is intentionally a holding/coordination spec with no build phases until `runx-target-repo-runners` and `runx-post-merge-closure-observer` advance past draft, and document the gate that would let phases be added.
  - Question: Is this spec intended to remain a coordination/holding draft, or should it carry an executable phase plan now?
  - Recommended answer: Holding draft. Add a one-paragraph clarification that build phases are deferred until the two dependency specs leave draft, and that the only currently-actionable work (sanitized external fixture, nitrosend/api policy coverage, contract tests) is already represented as completed [x] items.
  - If unanswered: Treat as holding spec and document the deferral explicitly.
- [medium/blocks approval] `harden-2` command_audit - Validation block uses `./crates/target/debug/runx` without listing the required cargo build prerequisite.
  - Status: open
  - Grounded in: code:crates/runx-cli/Cargo.toml
  - Evidence: The Validation Commands block invokes `./crates/target/debug/runx policy lint ...` and `./crates/target/debug/runx policy inspect ...`, but only the 2026-05-22 refresh note records `cargo build --manifest-path crates/Cargo.toml -p runx-cli`. A reader running the canonical block on a clean checkout will hit ENOENT before the policy commands run.
  - Recommendation: Add `cargo build --manifest-path crates/Cargo.toml -p runx-cli` as the first step in the Validation Commands block, or replace the prebuilt-binary invocations with `cargo run --manifest-path crates/Cargo.toml -p runx-cli -- policy ...` so the toolchain handles the build automatically.
  - If unanswered: Prepend the cargo build step to the validation block.
- [low/advisory] `harden-3` documentation_drift - 2026-05-20 evidence block still cites the stale `pnpm --filter @runxhq/cli run runx policy ...` command.
  - Status: open
  - Grounded in: code:packages/cli/package.json:20-22
  - Evidence: packages/cli/package.json defines only `bin.runx -> ./bin/runx`; there is no `scripts.runx`, so `pnpm --filter @runxhq/cli run runx ...` will fail with 'Command "runx" not found'. The 2026-05-21 refresh acknowledges this and switches to the Rust binary, but the 2026-05-20 evidence block retains the broken pnpm form alongside the corrected Rust path.
  - Recommendation: Either delete the stale 2026-05-20 pnpm lines or annotate them as historical (pre-stale-detection) so future readers do not copy them. If you want the npm path to keep working, use `pnpm --filter @runxhq/cli exec runx ...` since `runx` is a bin, not a script.
  - If unanswered: Annotate the 2026-05-20 pnpm commands as historical and superseded by the 2026-05-21 Rust binary commands.
- [low/advisory] `harden-4` doc_hygiene - Truncated runner update sentence: 'rebuilt the current local Rust' lacks an object.
  - Status: open
  - Grounded in: code:.scafld/specs/drafts/rust-nitrosend-dogfood.md:22
  - Evidence: Current State line 22: `Latest runner update: 2026-05-22T01:42:00+10:00 rebuilt the current local Rust` — the sentence is cut off. The intended object is almost certainly `Rust binary`, but as written the entry reads as truncated.
  - Recommendation: Finish the sentence (likely `... rebuilt the current local Rust binary via cargo build -p runx-cli for the 2026-05-22 refresh`).
  - If unanswered: Append `binary via cargo build -p runx-cli for the 2026-05-22 refresh.`
- [low/advisory] `harden-5` future_path - Future test `crates/runx-runtime/tests/external/nitrosend_issue_intake.rs` is referenced but has no acceptance trigger for when it must exist.
  - Status: open
  - Grounded in: code:crates/runx-runtime/tests/external/aster_agent_step.rs
  - Evidence: The spec lists `crates/runx-runtime/tests/external/nitrosend_issue_intake.rs` under Missing local surfaces and `cargo test -p runx-runtime nitrosend_issue_intake` under Future validation. Today only `aster_agent_step.rs` is in that directory. No acceptance item ties the creation of this Rust runtime replay test to a specific gating event.
  - Recommendation: Add an acceptance item like 'Once both runx-target-repo-runners and runx-post-merge-closure-observer leave draft, add crates/runx-runtime/tests/external/nitrosend_issue_intake.rs that replays the external fixture end-to-end and asserts canonical receipt bytes are identical across two runs.' This makes the deferred work scheduleable.
  - If unanswered: Add the explicit dependency-gated acceptance item described above.

### round-2

Status: passed
Started: 2026-05-21T15:49:30Z
Ended: 2026-05-21T15:49:30Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round 2 verifies that round-1's blocking gaps are closed: Phase 1 ("Ratify Current Dogfood Snapshot") now exists with executable acceptance commands p1a/p1b/p1c, and p1c explicitly chains `cargo build --manifest-path crates/Cargo.toml -p runx-cli &&` before invoking `./crates/target/debug/runx`, so the runx binary prerequisite is no longer implicit. Paths cited in the draft were spot-checked: `fixtures/operational-policy/nitrosend-like.json` includes `nitrosend/api` in runners.target_repos, owner_routes.target_repos, and the targets list; `fixtures/external/nitrosend/issue-intake/api-source-thread.json` exists and is exercised by `crates/runx-contracts/tests/nitrosend_external_fixture.rs`; `crates/runx-contracts/tests/operational_policy.rs`, `target_runner.rs`, and `post_merge_observer.rs` all exist; runx-cli's `[[bin]] name = "runx"` confirms the binary path. The two dependency drafts (`runx-target-repo-runners`, `runx-post-merge-closure-observer`) remain in_progress drafts, and the spec's "deferred" acceptance items correctly route through those parent specs. The architectural posture (preservation dogfood, no live capture, reusable runner/observer dependency) is sound. Remaining findings are advisory: harden-3 (stale pnpm command line in the 2026-05-20 evidence block), harden-4 (truncated runner-update sentence at line 22), harden-5 (deferred Rust runtime replay test still lacks an explicit acceptance trigger), plus a new medium-advisory note that Phase 1 acceptance does not run `cargo test -p runx-runtime` even though an unchecked criterion claims the generic runtime fixtures remain green. None of these block approval.

Checks:
- path audit
  - Grounded in: code:fixtures/operational-policy/nitrosend-like.json:60-114
  - Result: passed
  - Evidence: Verified nitrosend-like.json lists nitrosend/nitrosend, nitrosend/api, and nitrosend/app in runners.target_repos (lines 60-64), owner_routes.target_repos (lines 74-78), and as `targets[]` entries (lines 88, 102, ...). fixtures/external/nitrosend/issue-intake/api-source-thread.json exists. crates/runx-contracts/tests/{nitrosend_external_fixture,operational_policy,target_runner,post_merge_observer}.rs all exist. crates/runx-cli/Cargo.toml defines `[[bin]] name = "runx"` (lines 24-26), so `./crates/target/debug/runx` is the correct artifact path after building -p runx-cli. .scafld/specs/drafts/{runx-target-repo-runners,runx-post-merge-closure-observer}.md both exist and remain draft, matching the spec's coordination claims.
- command audit
  - Grounded in: code:.scafld/specs/drafts/rust-nitrosend-dogfood.md:202-212
  - Result: passed
  - Evidence: Round-1 blocker harden-2 is resolved: Phase 1 acceptance p1c chains `cargo build --manifest-path crates/Cargo.toml -p runx-cli && ./crates/target/debug/runx policy lint ... && ./crates/target/debug/runx policy inspect ...`, so the runx binary prerequisite is now explicit in the acceptance command itself. The Validation Commands block (lines 218-228) also now lists `cargo build --manifest-path crates/Cargo.toml -p runx-cli` before the binary invocations. p1a (`scafld validate ... --json`) and p1b (cargo test runx-contracts nitrosend_external_fixture + operational_policy) are well-formed and reflect tests confirmed to exist.
- scope/migration audit
  - Grounded in: code:.scafld/specs/drafts/rust-nitrosend-dogfood.md:185-212
  - Result: passed
  - Evidence: Round-1 blocker harden-1 is resolved: `## Phase 1: Ratify Current Dogfood Snapshot` is present (line 185) with explicit objective, change list, and three acceptance items mapped to executable commands. The phase is correctly scoped as a snapshot ratification — it explicitly states 'this phase must not claim live target PR creation or final closure observation' and leaves live target-runner/observer/external-replay work deferred to the named parent drafts. Bundled cutovers are not hidden: the spec is explicit that Nitrosend product names may remain but execution artifacts must be canonical harness/decision/act/proof/receipt only.
- acceptance timing audit
  - Grounded in: code:.scafld/specs/drafts/runx-target-repo-runners.md:16-30
  - Result: passed
  - Evidence: The four acceptance items that depend on `runx-target-repo-runners` and `runx-post-merge-closure-observer` are explicitly marked deferred or wait-gated; both parent drafts remain in_progress with documented blockers (target checkout/git mutation, PR create/update, source publication pushers, Aster scheduling for the runner spec; live GitHub observer/webhook/scheduler, target-runner readback, publication transports for the observer spec). Phase 1's acceptance does not attempt to flip those deferred items, so timing is honest. Deferred Follow-Up Gates (lines 176-184) names the exact dependency contract per item.
- rollback/repair audit
  - Grounded in: code:.scafld/specs/drafts/rust-nitrosend-dogfood.md:391-401
  - Result: passed
  - Evidence: Rollback section remains concrete and credible: (1) before any Nitrosend launcher flip, keep production path unchanged and disable Rust side-by-side validation; (2) if the policy fixture omits a real target, repair the fixture and admission tests rather than special-casing routing; (3) if an external fixture is accidentally live-captured, replace it with a sanitized deterministic fixture before merging; (4) repair producers/expected receipts rather than whitelisting retired fields. Each repair action is local and reversible.
- design challenge
  - Grounded in: code:.scafld/specs/drafts/rust-nitrosend-dogfood.md:50-63
  - Result: passed
  - Evidence: The architectural posture — Nitrosend as preservation-only dogfood, no live capture, reusable target-runner/observer contracts rather than Nitrosend-only adopter code — is the right move. The spec explicitly disclaims short-sighted bandaids ('preservation, not a new adopter flow'; 'This spec must not invent live Nitrosend traffic capture') and binds itself to canonical harness_receipt.v1 sealing. The choice to keep the spec a holding draft until two reusable parent specs leave draft avoids premature bundling. Invariants forbid dual-reading retired peer terminal artifacts, which prevents future compatibility-shim bloat.

Issues:
- [low/advisory] `harden-4` doc_hygiene - Truncated runner update sentence carried over from round 1.
  - Status: open
  - Grounded in: code:.scafld/specs/drafts/rust-nitrosend-dogfood.md:22
  - Evidence: Current State line 22 still reads `Latest runner update: 2026-05-22T01:42:00+10:00 rebuilt the current local Rust` with no object. Round-1 advisory harden-4 flagged this and the operator did not resolve it before requesting round 2.
  - Recommendation: Finish the sentence, e.g. `... rebuilt the current local Rust binary via cargo build -p runx-cli for the 2026-05-22 refresh.`
  - Question: Should the truncated runner-update line be completed before approval, or is it acceptable to carry forward as-is?
  - Recommended answer: Complete it now with the cargo build -p runx-cli context; it costs nothing and avoids the next reviewer thinking content was lost.
  - If unanswered: Append `binary via cargo build -p runx-cli for the 2026-05-22 refresh.`
- [low/advisory] `harden-3` documentation_drift - 2026-05-20 evidence block still cites the stale `pnpm --filter @runxhq/cli run runx policy ...` invocation.
  - Status: open
  - Grounded in: code:packages/cli/package.json
  - Evidence: Lines 295-298 keep the `pnpm --filter @runxhq/cli run runx policy lint/inspect ...` lines as recorded evidence. The spec itself acknowledges (lines 239-242) that `@runxhq/cli` defines `bin.runx` but no `scripts.runx`, so the `run runx` form would not work today. A future reader scanning the 2026-05-20 block in isolation could copy a broken command.
  - Recommendation: Either delete those two stale lines or annotate them inline as 'superseded by the 2026-05-21 Rust binary commands; `pnpm --filter @runxhq/cli exec runx ...` is the correct npm-bin form.'
  - If unanswered: Annotate the 2026-05-20 pnpm lines as historical and superseded by the 2026-05-21 Rust binary commands.
- [low/advisory] `harden-5` future_path - Deferred Rust runtime replay test still has no acceptance trigger.
  - Status: open
  - Grounded in: code:crates/runx-runtime/tests/external/aster_agent_step.rs
  - Evidence: Spec lists `crates/runx-runtime/tests/external/nitrosend_issue_intake.rs` under Missing local surfaces and `cargo test -p runx-runtime nitrosend_issue_intake` under Future validation, but the only file in that directory is `aster_agent_step.rs`. No acceptance item names the gating event (e.g., both dependency specs leaving draft) that would require this test to exist.
  - Recommendation: Add a deferred-but-explicit acceptance item: 'Once both runx-target-repo-runners and runx-post-merge-closure-observer leave draft, add crates/runx-runtime/tests/external/nitrosend_issue_intake.rs that replays the external fixture end-to-end and asserts canonical receipt bytes are identical across two runs.'
  - If unanswered: Add the explicit dependency-gated acceptance item described above.
- [medium/advisory] `harden-6` coverage_gap - Phase 1 acceptance does not verify the unchecked criterion that generic issue-intake/issue-to-pr runtime fixtures remain green.
  - Status: open
  - Grounded in: code:crates/runx-runtime/tests/skill_issue_intake.rs
  - Evidence: Acceptance Criteria line 159 (`[ ] Existing generic issue-intake and issue-to-pr runtime fixtures remain green and are referenced by the external-shaped Nitrosend fixture plan.`) and line 169 (`[ ] No fixture, schema id, persisted receipt, or replay expectation uses retired peer terminal artifacts...`) are both unchecked. Phase 1 acceptance (p1a/p1b/p1c) only runs scafld validate and runx-contracts tests; it does not run `cargo test -p runx-runtime` (which is where skill_issue_intake.rs and skill_issue_to_pr.rs replay the generic fixtures) and does not run the retired-artifact ripgrep guard that appears in Validation Commands line 226.
  - Recommendation: Add a fourth Phase 1 acceptance command (e.g., p1d) that runs `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --tests issue_intake issue_to_pr` and the retired-artifact `! rg ...` guard, so completing Phase 1 actually flips the two unchecked snapshot-scope criteria. Alternatively, explicitly state those two items remain holding-checkbox items that Phase 1 does not attempt to close.
  - Question: Should Phase 1 acceptance be extended to actually re-verify the generic runtime fixtures and retired-artifact guard, or should those two checkboxes be re-scoped as holding items?
  - Recommended answer: Extend Phase 1 with a p1d that runs the runtime-package replay tests plus the retired-artifact rg guard. The commands are cheap and align with the phase objective of ratifying the current OSS snapshot.
  - If unanswered: Add a p1d covering `cargo test -p runx-runtime` for the generic skill fixtures and the retired-artifact ripgrep guard.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Reviewed rust-nitrosend-dogfood in discover mode. The spec is a holding/coordination dogfood spec that ratifies the current OSS Nitrosend snapshot; task_changes since approval baseline = 0 and ambient drift (60 paths) is concentrated in MCP/RMCP cutover, connect refactor, license-boundary, runx-core policy, and ambient kernel/policy fixture work — none touches fixtures/operational-policy/nitrosend-like.json, fixtures/external/nitrosend/**, the cited runtime fixture cases, or the spec itself, so it is properly attributed as context. All seven top-level Acceptance Criteria are [x] with verifiable grounding: nitrosend-like.json lists `nitrosend/api` in runners.target_repos (line 62), owner_routes.target_repos (line 76), and as a targets[] entry (line 102) alongside nitrosend/nitrosend and nitrosend/app; fixtures/external/nitrosend/issue-intake/api-source-thread.json exists, targets nitrosend/api, cites runtime fixtures (bounded-docs-fix.yaml, issue-to-pr-reaches-fix-boundary.yaml) and the post-merge harness receipt (post-merge-observer-merged-verified.json) that all resolve on disk; crates/runx-contracts/tests/nitrosend_external_fixture.rs exists alongside operational_policy.rs/target_runner.rs/post_merge_observer.rs. The retired-artifact ripgrep guard (`runx\.issue_to_pr_outcome\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|"effect"\s*:`) returns zero matches across fixtures/{runtime,operational-policy,external}, skills, and crates/runx-runtime/src, satisfying the invariant against retired outcome/effect/report shapes. Dependency drafts runx-target-repo-runners and runx-post-merge-closure-observer are correctly deleted from drafts in this checkout state but the spec routes deferred live work through them as named coordination gates. The two non-blocking findings from the prior review pass are now fixed: Phase 1 Status reads `completed` (no longer stale `active`), and the Phase 1 Objective at line 190 reads as a complete sentence ("Re-run current OSS and sibling Nitrosend dogfood evidence, then confirm live target-runner, observer, and deterministic external replay gates remain deferred to the named parent specs"). No new completion blockers, no scope drift, no regressions.

Attack log:
- `Acceptance Criteria items 1–7 (top-level checklist)`: Verify each [x] item is grounded by a cited file, fixture, or test that exists in the checkout. -> clean (All seven items map to verifiable evidence: nitrosend-like.json multi-target coverage, fixtures/external/nitrosend/issue-intake/api-source-thread.json existence, runx-contracts test files, runx policy lint/inspect refresh notes, retired-artifact guard, and explicit deferral language for live target/observer/replay.)
- `fixtures/operational-policy/nitrosend-like.json`: Confirm `nitrosend/api` appears in runners.target_repos, owner_routes.target_repos, and targets[] (the spec's explicit review scope). -> clean (Verified at lines 62, 76, and 102; all three nitrosend targets present with consistent runner_ids (aster-production), owner route (product-surface), and allowed_actions (issue-intake/issue-to-pr/pr-review). scafld_required true, mutate_target_repo true, require_human_merge_gate true.)
- `fixtures/external/nitrosend/issue-intake/api-source-thread.json + crates/runx-contracts/tests/nitrosend_external_fixture.rs`: Cross-check that the external fixture exists, targets nitrosend/api, and is exercised by the cited Rust contract test for plan + dedupe lookup. -> clean (Fixture targets `nitrosend/api` with action issue-to-pr, cites runtime fixtures + post-merge harness receipt. Test file present and registered in cargo target fingerprint cache.)
- `Cited runtime + post-merge fixtures`: Confirm the fixture's `runtime_fixtures` and `post_merge_fixture` references resolve to files on disk. -> clean (fixtures/runtime/skills/issue-intake/cases/bounded-docs-fix.yaml, fixtures/runtime/skills/issue-to-pr/cases/issue-to-pr-reaches-fix-boundary.yaml, and fixtures/contracts/harness-spine/post-merge-observer-merged-verified.json all present.)
- `Retired peer-terminal-artifact invariant`: Run the spec's retired-artifact regex (`runx\.issue_to_pr_outcome\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|"effect"\s*:`) over fixtures/{runtime,operational-policy,external}, skills, crates/runx-runtime/src. -> clean (Zero matches. Invariant against retired outcome/effect/report shapes holds. Spec-level invariant 'No compatibility reader or writer translates retired outcome/effect/report artifacts into harness receipts at runtime' upheld.)
- `Ambient workspace drift (60 paths)`: Scan ambient drift list for any path that overlaps the declared task scope (`nitrosend/api`, nitrosend fixtures, or nitrosend-related skills/tests). -> clean (Drift is concentrated in crates/runx-runtime MCP/RMCP cutover, connect refactor, license-boundary harness, runx-core policy, runx-cli launcher/connect, runx-sdk client, ambient kernel/policy fixture updates, and packages/authoring. None touch fixtures/operational-policy/nitrosend-like.json, fixtures/external/nitrosend/**, the cited runtime cases, fixtures/contracts/harness-spine/post-merge-observer-merged-verified.json, or the dogfood spec itself.)
- `Prior-review non-blocking findings (doc-hygiene-truncated-objective, phase-status-stale-active)`: Re-verify whether the two low-severity findings from the prior review pass are still present (regression check) or resolved. -> finding (Both resolved. Phase 1 Status now reads `completed` (was `active`), and the Phase 1 Objective at line 190 is a complete sentence. Filed as two status=fixed entries.)
- `Phase 1 body internal consistency`: Check Phase 1's Status / Changes / Acceptance against the parent spec's lifecycle state and harden round-2 expectations. -> clean (Phase 1 Status=completed aligns with Current phase=final; all four Changes are [x]; Acceptance=none is consistent with a holding/coordination snapshot phase. Harden round 2 noted Phase 1 originally carried p1a/p1b/p1c executable acceptance, but the Validation Commands block (lines 207-218) and the 2026-05-22 refresh notes (lines 312-324) still record the same executable evidence (cargo build runx-cli, cargo test nitrosend_external_fixture, runx policy lint/inspect, scafld validate). Not a regression — the executable evidence simply migrated from per-phase acceptance to the Validation Commands ledger.)

Findings:
- [low/non-blocking] `doc-hygiene-truncated-objective` Prior-review Phase 1 Objective truncation now resolved.
  - Location: `.scafld/specs/active/rust-nitrosend-dogfood.md:190`
  - Evidence: Phase 1 Objective at lines 190-192 now reads as a complete sentence: 'Re-run current OSS and sibling Nitrosend dogfood evidence, then confirm live target-runner, observer, and deterministic external replay gates remain deferred to the named parent specs.' The dangling `then` from the prior review pass is gone.
  - Validation: Re-read .scafld/specs/active/rust-nitrosend-dogfood.md lines 185-202.
- [low/non-blocking] `phase-status-stale-active` Prior-review stale Phase 1 Status now reads `completed`.
  - Location: `.scafld/specs/active/rust-nitrosend-dogfood.md:187`
  - Evidence: Phase 1 now reports `Status: completed` at line 187, aligned with the parent spec's `Current phase: final` and `status: review` frontmatter. All four Phase 1 Changes are [x] and Acceptance is `none`, so the phase content is materially complete and consistently marked.
  - Validation: Re-read .scafld/specs/active/rust-nitrosend-dogfood.md lines 1-22 and 185-201.

