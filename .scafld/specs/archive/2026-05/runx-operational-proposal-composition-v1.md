---
spec_version: '2.0'
task_id: runx-operational-proposal-composition-v1
created: '2026-05-27T16:34:34Z'
updated: '2026-05-28T06:36:14Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Operational Proposal Composition

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-28T06:36:14Z
Review gate: pass

## Summary

Use existing runx architecture to compose operational work without creating a
new fixed lane for every business domain. A source event becomes redacted
context, runx emits signal/decision state, and the next result is either an
existing governed action or a reviewable proposal.

The original operational shape remains intact:

- check/triage can inspect without mutation;
- reply-only can draft a useful response;
- issue-intake can create or attach tracking state;
- issue-to-PR can build a governed fix;
- work-plan can hand off larger work;
- manual-review can stop safely;
- escalation can route a precise human decision;
- outreach/product/support behavior can be expressed as application proposal
  kinds instead of core platform lanes.

Golden path invariant: originating source thread/event -> hydrated context
artifact -> optional read-only check/triage -> create/update GitHub issue when
requested -> optional build-fix issue-to-PR without requiring a prior check ->
governed PR with human merge gate -> final outcome posted back to the
originating source thread and linked issue/PR.

## Objectives

- Compose existing source, policy, skill graph, receipt, story, and outbox
  primitives into one operational flow.
- Add a generic proposal handoff where a decision needs human/application
  review before mutation or send.
- Keep escalation as a first-class operational outcome, represented as
  `proposal_kind: escalation` plus owner route, urgency, evidence, and exact
  decision needed.
- Keep customer sends, PR merges, billing actions, and destructive provider
  mutations outside proposal authority.
- Support product-specific proposal kinds such as:
  - `support_reply`;
  - `outreach_proposal`;
  - `product_signal`;
  - `incident_response`.
- Preserve escalation as the generic `proposal_kind: escalation`; products can
  label the owner route as dev/support/ops without creating a new core kind.
- Avoid adding a new core action enum for every proposal kind.
- Preserve the issue-to-PR path as one governed action within the larger
  operational flow.

## Scope

In scope:

- Docs explaining composition over existing runx primitives.
- Skill graph examples or fixtures showing:
  - read-only check;
  - build fix without prior check;
  - reply-only with proposal context;
  - escalation proposal;
  - outreach proposal as an app-specific proposal kind;
  - no-action/manual-review.
- Core helper updates only if existing skill graph or story helpers cannot pass
  proposal artifacts between steps cleanly.
- Policy/readback notes for proposal authority, existing action lanes, and
  owner route ids.

Out of scope:

- New fixed runx lanes for support response, alert triage, outreach, roadmap, or
  customer success.
- Provider API calls.
- Nitrosend-specific Slack UX, customer enrichment, owner maps, templates, or
  live dogfood.
- Auto-send, auto-merge, billing/account mutation, or destructive provider
  mutation.
- Legacy aliases for cancelled fixed-lane names.

## Dependencies

- `runx-operational-contracts-v1` for the generic proposal contract decision.
- `runx-operational-story-outbox-v1` for source-thread projection.
- Existing runx surfaces:
  - `docs/developer-issue-inbox.md`;
  - `docs/issue-to-pr.md`;
  - `docs/thread-story-contract.md`;
  - `packages/core/src/source/index.ts`;
  - `packages/core/src/knowledge/thread-story.ts`;
  - `packages/core/src/knowledge/outbox.ts`;
  - `packages/contracts/src/schemas/operational-policy.ts`;
  - `skills/issue-intake/SKILL.md`;
  - `skills/issue-triage/SKILL.md`;
  - `skills/issue-to-pr/SKILL.md`.
- Nitrosend integration for live application dogfood.

## Assumptions

- Existing action intents are enough for mutation lanes. Proposal kinds annotate
  reviewable handoffs and should not explode the action enum.
- `runx-operational-contracts-v1` must be hardened or later in the scafld
  lifecycle before this spec is approved, because this child relies on the
  committed `runx.operational_proposal.v1` contract boundary.
- `check` may remain a UI verb mapped to read-only triage unless runx deliberately
  adds a canonical read-only action name.
- `create issue` maps to `issue-intake`.
- `build fix` maps to `issue-to-pr`.
- `escalate` maps to a proposal/manual-review style handoff unless a consuming
  application explicitly wires a provider notification.
- Prior check output is useful context, not mutation permission. Build fix must
  also work without a prior check when source context and policy are sufficient.
- A proposal may include draft text, but draft text is not sent content.
- Issue and PR creation must preserve the source-thread story. The issue URL,
  PR URL when present, human gate, and final outcome must be linkable back to
  the originating source thread.

## Touchpoints

- `docs/operational-intelligence.md`
- `docs/developer-issue-inbox.md`
- `docs/issue-to-pr.md`
- `docs/thread-story-contract.md`
- `skills/issue-intake/SKILL.md`
- `skills/issue-intake/X.yaml`
- `skills/issue-triage/SKILL.md`
- `skills/issue-to-pr/SKILL.md`
- `packages/core/src/source/index.ts`
- `packages/core/src/knowledge/thread-story.ts`
- `packages/core/src/knowledge/outbox.ts`
- `fixtures/operational-proposal/**`
- `fixtures/runtime/skills/**` only where composition fixtures are added

## Risks

- Reintroducing fixed lanes through skill names. Mitigation: app-specific skills
  may exist, but the core composition contract is proposal/action based.
- Making escalation vague. Mitigation: escalation proposals require severity,
  owner route, evidence, suspected area, and exact human decision.
- Overloading proposals as mutation authority. Mitigation: proposals carry
  approval gates and allowed next actions; they do not send, merge, or mutate by
  themselves.
- Losing the original issue-to-PR flow. Mitigation: build fix remains a direct
  governed path and must not require a prior check.

## Acceptance

Profile: strict

Validation:
- `scafld validate runx-operational-proposal-composition-v1`
- `pnpm typecheck`
- `pnpm test:fast`
- `pnpm boundary:check`
- `git diff --check -- docs skills packages fixtures .scafld/specs/drafts/runx-operational-proposal-composition-v1.md`

## Phase 1: Composition Boundary

Status: completed
Dependencies: none

Objective: Prove the work is composition on existing architecture, not a new

Changes:
- Document the source/context/signal/decision/proposal/action/outcome spine.
- Document UI verb mappings to canonical runx actions.
- Document escalation as a generic proposal kind and not a separate public `dev_escalation` schema.
- Document the composition spine in `docs/operational-intelligence.md`, not only in this spec.

Acceptance:
- [x] `p1_ac1` command - Spec validates.
  - Command: `scafld validate runx-operational-proposal-composition-v1`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `p1_ac2` command - Original flow and escalation are preserved.
  - Command: `sh -c 'f=$(find .scafld/specs/drafts .scafld/specs/approved .scafld/specs/active -name runx-operational-proposal-composition-v1.md -print -quit); test -n "$f" && for token in "build fix without prior check" "proposal_kind: escalation" "issue-to-PR" "manual-review" "final outcome posted back"; do rg -n "$token" "$f" >/dev/null || exit 1; done'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `p1_ac3` command - Contract precondition is at approval gate or later.
  - Command: `scafld status runx-operational-contracts-v1 --json | node -e "let s='';process.stdin.on('data',d=>s+=d).on('end',()=>{const r=JSON.parse(s).result;if(!r)process.exit(1);const hardened=r.next==='scafld approve runx-operational-contracts-v1';const later=['approved','in_progress','review','completed'].includes(r.status);if(!hardened&&!later)process.exit(1)})"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8
- [x] `p1_ac4` command - Operational intelligence docs record the generic spine.
  - Command: `test -f docs/operational-intelligence.md && for token in "source/context/signal/decision/proposal/action/outcome" "build fix without prior check" "proposal_kind: escalation" "originating source thread"; do rg -n "$token" docs/operational-intelligence.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-9

## Phase 2: Fixtures And Examples

Status: completed
Dependencies: Phase 1

Objective: Add small deterministic examples for the important composition paths.

Changes:
- Add fixtures for:
- Keep private source payloads separate from public expected outputs.
- Create `fixtures/operational-proposal/public/` for redacted expected outputs; the leak guard must fail if the public fixture directory is missing.

Acceptance:
- [x] `p2_ac0` command - Proposal contract surfaces exist before composition fixtures depend on them.
  - Command: `for token in "runx.operational_proposal.v1" "proposal_kind" "source_thread_locator"; do rg -n "$token" packages/contracts/src crates/runx-contracts/src >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `p2_ac1` command - Proposal composition fixtures cover core paths.
  - Command: `test -d fixtures/operational-proposal && for token in read_only_check build_fix_without_prior_check escalation_proposal outreach_proposal manual_review no_action; do rg -n "$token" fixtures/operational-proposal >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `p2_ac2` command - Public proposal fixtures avoid obvious leaks.
  - Command: `test -d fixtures/operational-proposal/public && ! rg -n "/Users/|RUNX_BIN=|SENTRY_AUTH_TOKEN|xox[baprs]-|url_private_download|raw_payload|BEGIN .*PRIVATE KEY" fixtures/operational-proposal --glob '!private/**' --glob '!inputs/private/**' --glob '!raw/**'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16
- [x] `p2_ac3` command - Create issue fixture returns a source-thread story update.
  - Command: `test -d fixtures/operational-proposal && for token in create_issue github_issue_url source_thread story_update; do rg -n "$token" fixtures/operational-proposal >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17
- [x] `p2_ac4` command - Build fix fixture covers source-thread story, issue, PR, human gate, and outcome.
  - Command: `test -d fixtures/operational-proposal && for token in build_fix_without_prior_check source_thread story_update github_issue_url github_pr_url human_gate final_outcome; do rg -n "$token" fixtures/operational-proposal >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18

## Phase 3: Composition Implementation

Status: completed
Dependencies: Phase 2

Objective: Implement only the missing generic seams discovered by the fixtures.

Changes:
- Update docs and skill graph examples.
- Update core helpers only if needed to pass proposal artifacts through graph steps and render story milestones.
- Avoid creating fixed domain action variants.
- Add tests proving build fix does not require a prior check and check does not grant mutation permission.

Acceptance:
- [x] `p3_ac1` command - TypeScript compiles.
  - Command: `pnpm typecheck`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `p3_ac2` command - Fast tests pass.
  - Command: `pnpm test:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24
- [x] `p3_ac3` command - Boundary checks pass.
  - Command: `pnpm boundary:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-25
- [x] `p3_ac4` command - No fixed domain action enum variants are added.
  - Command: `sh -c 'if rg -n "support-response|dev-escalation|outreach-proposal|roadmap-signal|support_response|dev_escalation|outreach_proposal|roadmap_signal|SupportResponse|DevEscalation|OutreachProposal|RoadmapSignal" packages crates skills; then exit 1; fi; test -z "$(find skills -mindepth 1 -maxdepth 1 -type d -print 2>/dev/null | sed "s#^.*/##" | rg -x "dev-escalation|outreach-proposal|roadmap-signal|support-response" || true)"'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26
- [x] `p3_ac5` command - Build fix/no-prior-check and check/no-mutation tests are explicit.
  - Command: `for token in "build fix without prior check" "check.*does not grant mutation" "prior check.*advisory"; do rg -n "$token" packages/core --glob '*.test.ts' >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-27

## Phase 4: Consumer Notes

Status: completed
Dependencies: Phase 3

Objective: Make app-specific proposal skills straightforward without leaking app

Changes:
- Document how products define proposal kinds and route ids.
- Document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes.
- Include Nitrosend examples only as consuming-app examples, not core policy.

Acceptance:
- [x] `p4_ac1` command - Consumer boundary docs mention proposal kinds and route ids.
  - Command: `test -f docs/operational-intelligence.md && for token in "## Consuming Application Boundary" "proposal_kind" "owner_route_id" "Aster" "source-thread"; do rg -n "$token" docs/operational-intelligence.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-32

## Rollback

- Revert docs, fixtures, helper changes, and tests introduced by this child.
- Do not alter provider messages, GitHub issues/PRs, customer state, or
  consuming product config from this child.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: codex
Output: codex.output_file
Summary: Verified the prior completion-blocking action-id regression is fixed: the build-fix fixture and test now use canonical `issue-to-pr`, matching the existing policy action lane. No remaining completion blockers found in the scoped docs, fixture, contract consumers, authority gates, or fixed-lane scans.

Attack log:
- `workspace`: Workspace classification and review state check -> clean (Read git status and scafld status --json. Confirmed task remains in review and observed the same task-scoped/ambient split from the packet; did not mutate files.)
- `fixtures/operational-proposal/public/composition-paths.json, packages/core/src/knowledge/operational-proposal-composition.test.ts`: Open blocker verification: canonical issue-to-pr action id -> clean (Previous blocker was the build-fix fixture/test using non-canonical issue-to-PR. Verified fixture action_intent and allowed_next_actions now use issue-to-pr, matching TS and Rust policy action ids.)
- `docs/operational-intelligence.md`: Spec compliance for composition spine and authority model -> clean (Read docs/operational-intelligence.md. It documents source/context/signal/decision/proposal/action/outcome, source-thread continuity, proposal-only authority, human gates, and consuming-app boundary.)
- `fixtures/operational-proposal/public/composition-paths.json`: Fixture coverage and redaction scan -> clean (Read the public composition fixture and searched for private payload markers/provider-specific leak patterns in task-relevant files. The proposal wires use provider-neutral references and keep mutation/publication/final-decision authority false.)
- `packages/contracts/src/schemas/operational-proposal.ts, packages/contracts/src/index.ts, crates/runx-contracts/src/operational_proposal.rs`: Contract/export consumer trace -> clean (Read TypeScript/Rust operational proposal contract surfaces and exports. The added runx.escalation extension typing is schema-neutral, and public exports/tests line up with the fixture consumers.)
- `docs, fixtures, packages, crates, skills`: Fixed-lane regression scan -> clean (Searched docs, fixtures, packages, crates, and skills for forbidden fixed-lane names and authority-widening markers. Hits were limited to allowed product proposal_kind examples, negative test strings, existing policy docs/tests, or unrelated ambient skill examples.)
- `fixtures/operational-proposal/public/composition-paths.json, packages/core/src/knowledge/operational-proposal-composition.test.ts`: Golden path regression trace -> clean (Checked build-fix, create-issue, final_outcome, source_thread/story_update, result_refs, publication_refs, and human_gate coverage in fixture/test paths. The build-fix path no longer requires prior check and remains gated.)
- `acceptance evidence`: Acceptance evidence handling -> clean (Per provider instruction, did not rerun build/test/mutation commands; treated recorded pnpm/scafld evidence as already executed and used read-only commands only.)

Findings:
- none

## Self Eval

- Pending implementation. Target bar: 9.5/10 composition clarity; existing runx
  primitives do most of the work, and new code is limited to missing generic
  seams.

## Deviations

- This spec replaces the cancelled fixed domain lane drafts.

## Metadata

- created_by: scafld
- parent_spec: runx-operational-intelligence-action-layer-v1

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: error
Started: 2026-05-27T17:00:25Z
Ended: 2026-05-27T17:00:25Z
Summary: provider error: provider failed: provider produced no submission; Claude must call submit_harden exactly once and final text is ignored: ... :{"type":"input_json_delta","partial_json":"ntellig"}},"session_id":"b1ea013f-bba6-4c90-89a4-62870532cd8d","parent_tool_use_id":null,"uuid":"3df68d29-ec53-405e-aaea-6445909211ab"}

Checks:
- none

Issues:
- none

### round-2

Status: needs_revision
Started: 2026-05-27T17:21:08Z
Ended: 2026-05-27T17:21:08Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: The draft is architecturally on-target (composition over fixed lanes, generic proposal_kind, escalation kept as a proposal kind, build-fix path preserved). However, several acceptance gates do not actually verify the changes they describe: Phase 1 says "Document the spine in docs/operational-intelligence.md" but only greps the spec file itself; Phase 3 says "add tests proving build fix does not require a prior check" but `pnpm test:fast` is the only signal and would pass even if those tests are missing; Phase 4's token gate trips on common words like `product`/`consuming` that already exist in unrelated docs. There is also a real cross-spec ordering risk: Phase 2 fixtures and Phase 3 implementation lean on `runx.operational_proposal.v1` from `runx-operational-contracts-v1`, which is still a separate draft with no precondition gate here. The Phase 2 leak guard silently passes when `fixtures/operational-proposal/public/` is absent, and the Phase 3 negative grep for fixed-lane names is scoped to two TS files only (Rust contracts and other schemas not covered). The composition design itself is sound; the executability of the acceptance gates needs revision before approval.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/docs
  - Result: passed
  - Evidence: Verified existing touchpoints: docs/developer-issue-inbox.md, docs/issue-to-pr.md, docs/thread-story-contract.md, packages/core/src/source/index.ts, packages/core/src/knowledge/thread-story.ts, packages/core/src/knowledge/outbox.ts, packages/contracts/src/schemas/operational-policy.ts, skills/issue-intake/{SKILL.md,X.yaml}, skills/issue-triage/SKILL.md, skills/issue-to-pr/SKILL.md all exist. docs/operational-intelligence.md and fixtures/operational-proposal/ do not exist yet — both are intentionally future per Phase 1/Phase 2.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/package.json:35
  - Result: passed
  - Evidence: package.json defines `test:fast` (vitest fast config) at line 35 and `boundary:check` (node scripts/check-boundaries.mjs) at line 37; `pnpm typecheck` is a standard root script. All Phase 3 commands resolve. Token greps in p1_ac2 match: `build fix without prior check` (spec line 79), `proposal_kind: escalation` (lines 57, 66), `issue-to-PR` (line 47), `manual-review` (lines 39, 56, 211), `final outcome posted back` (lines 47-48). Verified `Aster`/`source-thread`/`consuming`/`product` already appear in docs/.
- scope/migration audit
  - Grounded in: spec_gap:dependencies
  - Result: failed
  - Evidence: Spec lists `runx-operational-contracts-v1` as a dependency for the generic `runx.operational_proposal.v1` shape, and `proposal` is currently absent from packages/contracts/src/schemas (grep found only `improvement_proposals` in registry.ts). Phase 2 fixtures and Phase 3 tests will reference a schema that lives in a still-draft sibling spec. There is no precondition acceptance command verifying the proposal schema exists before this spec implements against it.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase1.acceptance
  - Result: failed
  - Evidence: Phase 1's changes claim to write docs/operational-intelligence.md and document the spine. But p1_ac1 only runs `scafld validate` and p1_ac2 only greps the spec file itself for tokens. Neither verifies docs/operational-intelligence.md was created or contains the spine vocabulary. Phase 1 could be marked complete without producing the doc. Same gap in Phase 3: changes say `Add tests proving build fix does not require a prior check and check does not grant mutation permission` but the only relevant gate is `pnpm test:fast`, which passes even if those tests are absent.
- rollback/repair audit
  - Grounded in: derived_spec:rollback
  - Result: passed
  - Evidence: Spec is additive (docs, fixtures, optional helper changes, tests). Rollback explicitly says revert these and do not touch provider messages, GitHub issues/PRs, customer state, or consuming product config. No provider mutations are introduced (out-of-scope list includes provider API calls, auto-send, auto-merge). Revert is credible because no external state changes.
- design challenge
  - Grounded in: archive:.scafld/specs/archive/2026-05/runx-outreach-proposal-v1.md
  - Result: passed
  - Evidence: Archived drafts (runx-outreach-proposal-v1, runx-roadmap-signal-v1, runx-support-triage-response-v1, runx-alert-dev-escalation-v1) and the parent runx-operational-intelligence-action-layer-v1 explicitly identify the fixed-lane direction as overfit. Replacing per-domain platform lanes with one composition spine plus `proposal_kind` metadata aligns with oss/CLAUDE.md (action enum closed; admin/control-plane stays in cloud/) and operational-policy.ts (closed `operationalPolicyActions` enum). Composition is the right architectural move, not a bandaid; the design challenge sits in the executability of gates, not the architecture.

Issues:
- [high/blocks approval] `harden-1` acceptance_gap - Phase 1 acceptance does not verify the new docs were written.
  - Status: open
  - Grounded in: spec_gap:phases.phase1.acceptance
  - Evidence: Phase 1 changes commit to `Document the source/context/signal/decision/proposal/action/outcome spine` in `docs/operational-intelligence.md` (a non-existent file per Glob), document UI verb mappings, and document escalation as a generic proposal kind. But p1_ac1 (`scafld validate`) and p1_ac2 (`rg` over the spec file at .scafld/specs/drafts/runx-operational-proposal-composition-v1.md) only inspect the spec itself. There is no `test -f docs/operational-intelligence.md` and no grep against docs/. Phase 1 could be marked complete without producing any doc change.
  - Recommendation: Add a Phase 1 acceptance command that asserts the new doc exists and contains the spine vocabulary, e.g. `test -f docs/operational-intelligence.md && for token in source context signal decision proposal action outcome escalation 'proposal_kind'; do rg -n "$token" docs/operational-intelligence.md >/dev/null || exit 1; done`, and also confirm UI-verb mapping is recorded in a doc (not just the spec).
  - Question: Should Phase 1 require docs/operational-intelligence.md to exist and cover the spine plus verb mapping, rather than greppingthe spec text?
  - Recommended answer: Yes — make the doc creation and content the gate, since that is the actual Phase 1 deliverable. Keep the spec-token check as a secondary sanity test if useful.
  - If unanswered: Default to the stricter doc-existence + content grep gate.
- [high/blocks approval] `harden-2` acceptance_gap - Phase 3 has no signal that the build-fix-without-prior-check and check-does-not-grant-mutation tests were added.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance
  - Evidence: Phase 3 changes say `Add tests proving build fix does not require a prior check and check does not grant mutation permission.` Acceptance commands are pnpm typecheck, pnpm test:fast, pnpm boundary:check, and a negative grep for forbidden enum strings in two TS files. `pnpm test:fast` passes whether or not those specific tests exist, so there is no gate against the central behavioral promise of the spec.
  - Recommendation: Add a Phase 3 acceptance step that asserts the new tests exist by name or pattern, e.g. `rg -n 'build[_-]fix[_-]without[_-]prior[_-]check' packages/core/src` (or wherever the test lives) and a separate grep for the read-only/no-mutation test. Alternatively, point to a test file path and `grep -n it\\(` count gate.
  - Question: Where should the build-fix-without-prior-check and check-does-not-grant-mutation tests live, and what identifier pattern should the gate enforce?
  - Recommended answer: Add them in packages/core (or the helper package touched in Phase 3) and gate on a specific describe/it identifier such as `build fix without prior check` and `check is read-only`. Then add an `rg` acceptance check anchored on those strings.
  - If unanswered: Default to gating with `rg -n 'build fix without prior check' packages/core packages/contracts` and `rg -n 'check is read-only|check does not grant mutation' packages/core packages/contracts`.
- [high/blocks approval] `harden-3` dependency_ordering - No precondition guards against running this spec before runx-operational-contracts-v1 has defined runx.operational_proposal.v1.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/packages/contracts/src/schemas
  - Evidence: Grep across packages/contracts/src found `proposal` only in registry.ts (`improvement_proposals`) and tests. The generic `runx.operational_proposal.v1` packet is owned by runx-operational-contracts-v1 (still draft, harden_status: passed but not approved/built). Phase 2 fixtures (escalation_proposal, outreach_proposal, etc.) and Phase 3 fixtures-driven implementation rely on that contract. If this spec lands first, fixtures encode a schema id and shape that does not yet exist, and Phase 3 typecheck/test:fast will pass only because no TS imports the missing schema.
  - Recommendation: Either (a) add an explicit precondition acceptance (e.g., a check that `RUNX_LOGICAL_SCHEMAS.operationalProposal` or `runx.operational_proposal.v1` exists in packages/contracts) and document that approval is blocked until contracts ships, or (b) collapse the work into a sequenced build plan in this spec so fixtures and helpers can only be added after the contract is on disk.
  - Question: Should this spec gate Phase 2 and Phase 3 on the operational_proposal contract being present, or is it acceptable to ship fixtures that reference a future schema id?
  - Recommended answer: Gate on the contract being present. Add an acceptance like `rg -n 'runx.operational_proposal.v1' packages/contracts/src/schemas` at the top of Phase 2 (and the Rust analog if relevant).
  - If unanswered: Default to the gating-with-rg-check approach so the spec fails fast when its dependency has not landed.
- [medium/advisory] `harden-4` audit_scope - p3_ac4 only checks two TS files for forbidden fixed-lane names; Rust contracts and other schemas are uncovered.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance
  - Evidence: The negative grep targets only `packages/contracts/src/schemas/operational-policy.ts` and `packages/core/src/source/index.ts`. Fixed-lane terminology (`support-response`, `dev-escalation`, `outreach-proposal`, `roadmap-signal`) could equally re-enter `crates/runx-contracts/src/operational_policy/*`, other contracts schemas, or skill metadata.
  - Recommendation: Expand the negative grep to include `packages/contracts/src/schemas/**`, `packages/core/src/**`, and `crates/runx-contracts/src/**`, or rephrase to acknowledge that the gate is a smoke test, not exhaustive.
  - Question: Should the forbidden-lane grep cover Rust contracts and the wider TS schema directory, or stay narrow as a sentinel?
  - Recommended answer: Broaden to schemas + crates; the sentinel approach is too easy to bypass by editing a sibling file.
  - If unanswered: Default to broadening to `packages/contracts/src/schemas`, `packages/core/src`, and `crates/runx-contracts/src`.
- [medium/advisory] `harden-5` weak_gate - Phase 4's token list includes common words that already exist in unrelated docs, so the gate can pass without the consumer-notes work.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/docs/runtime-throughput.md
  - Evidence: Verified via grep: `product` and `consuming` already appear in runtime-throughput.md, ts-interop-boundary.md, getting-started.md, issue-to-pr.md, rust-kernel-architecture.md, etc. (13 hits across 5+ files). `Aster` and `source-thread` are also present in current docs. Only `proposal_kind` and `owner_route_id` are genuinely new (0 hits today). With the current OR-of-existence loop, the gate trivially passes against pre-existing docs.
  - Recommendation: Tighten p4_ac1 to anchor on the new vocabulary (e.g., `for token in proposal_kind owner_route_id 'consuming product' Aster source-thread; do ...`) and ideally pin the search to specific docs touched by Phase 4.
  - Question: Should the Phase 4 token check be anchored to specific files (e.g., docs/operational-intelligence.md, docs/developer-issue-inbox.md) and restricted to the genuinely new vocabulary?
  - Recommended answer: Yes — pin to the files Phase 4 modifies and drop generic words that already exist project-wide.
  - If unanswered: Default to restricting the grep to docs/operational-intelligence.md and dropping `product`/`consuming` from the token list.
- [medium/advisory] `harden-6` weak_gate - p2_ac2 leak check is a no-op when fixtures/operational-proposal/public/ is absent.
  - Status: open
  - Grounded in: spec_gap:phases.phase2.acceptance
  - Evidence: The command is `sh -c 'if test -d fixtures/operational-proposal/public; then ! rg -n "<patterns>" fixtures/operational-proposal/public; fi'`. If Phase 2 implementation skips the `public/` subdirectory (or places fixtures elsewhere), the conditional silently passes. There is no positive assertion that public fixtures exist when private fixtures do.
  - Recommendation: Restructure the gate to require `fixtures/operational-proposal/public` to exist whenever any private fixture is present, e.g. `test ! -d fixtures/operational-proposal/private || test -d fixtures/operational-proposal/public`, and then run the rg leak scan unconditionally on the public directory.
  - Question: Should Phase 2 require the public/ split to exist whenever private payloads are present, and run the leak scan unconditionally on public/?
  - Recommended answer: Yes — require the split and run rg on public/ without the `if test -d` guard, since the guard hides accidental omissions.
  - If unanswered: Default to making the public/ split mandatory and dropping the conditional.
- [low/advisory] `harden-7` boundary_check - Phase 4 plans to document Aster/hosted readback expectations in OSS docs; oss/CLAUDE.md says admin/control-plane belongs in cloud/.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/CLAUDE.md
  - Evidence: oss/CLAUDE.md states: `Admin/control-plane features do **not** belong in `oss/`.` and lists approval inboxes and adoption analytics as cloud/ surfaces. Phase 4 says `Document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes.` Approvals and proposal queues are exactly the admin shape that lives in cloud/.
  - Recommendation: Clarify whether OSS docs should only describe the contract surface that hosted/Aster consumes (readback shapes, not workflows), and leave Aster admin workflow docs in cloud/. Update Phase 4 wording so OSS docs do not imply hosted product behavior beyond contract.
  - Question: Should Phase 4 limit OSS docs to the contract/readback surface and keep hosted admin workflow documentation in cloud/?
  - Recommended answer: Yes — document the contract shape in OSS, and reference cloud/ as the owner of the workflow side. Avoid placing Aster admin UX or approval-inbox specifics in OSS docs.
  - If unanswered: Default to limiting OSS docs to the contract-readback surface and noting that hosted workflow lives in cloud/.

### round-3

Status: needs_revision
Started: 2026-05-27T17:30:19Z
Ended: 2026-05-27T17:30:19Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 fixes addressed most acceptance-gap concerns: Phase 1 now has p1_ac4 asserting docs/operational-intelligence.md exists with spine tokens; Phase 3 has p3_ac5 verifying "build fix without prior check" and "check…does not grant mutation" text exists; p3_ac4 now scans the whole `packages crates skills` tree; p2_ac2 now fails closed when `fixtures/operational-proposal/public/` is absent; p4_ac1 anchors on the new `## Consuming Application Boundary` heading plus `proposal_kind`/`owner_route_id`. Composition design is sound. Three executability gaps remain: (1) the dependency-ordering gate p1_ac3 accepts scafld gate `approve`, but `scafld status` for `runx-operational-contracts-v1` would currently report gate `approve` while the spec is still draft/unbuilt — meaning the public `runx.operational_proposal.v1` schema may not actually be on disk when Phase 2 fixtures and Phase 3 tests reference it; (2) the Phase 2 leak guard scopes only to `fixtures/operational-proposal/public/`, so fixtures placed under any other subdirectory bypass the scan; (3) p3_ac5's regex token check is satisfied by any matching text in `packages/core` or `fixtures/operational-proposal/**`, including a fixture comment or doc paragraph — it does not actually require a real `*.test.ts` to be added in Phase 3. Phase 4's hosted-readback docs language remains advisory/low (boundary risk per oss/CLAUDE.md). Architecture (one composition spine + generic `proposal_kind`) is the right move, not a bandaid.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/packages/contracts/src/schemas/operational-policy.ts
  - Result: passed
  - Evidence: Verified touchpoints exist: docs/issue-to-pr.md, docs/developer-issue-inbox.md (per round-2 path check), docs/thread-story-contract.md, packages/contracts/src/schemas/operational-policy.ts (lines 12-32 show closed `operationalPolicyActions` enum). Future paths docs/operational-intelligence.md and fixtures/operational-proposal/ are intentionally created in Phase 1/Phase 2.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/package.json:33
  - Result: passed
  - Evidence: package.json defines `typecheck` (line 33), `test:fast` (line 35), and `boundary:check` (line 37). Token-grep loops in p1_ac2 and p1_ac4 use POSIX sh and rg paths that are valid. p3_ac4 negative-grep now scans `packages crates skills` directories (previously two files); each is present in oss/.
- scope/migration audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-operational-contracts-v1.md:14
  - Result: failed
  - Evidence: runx-operational-contracts-v1.md current state: status `draft`, harden_status `passed`, Next: approve. The public `runx.operational_proposal.v1` schema is added in that sibling spec's Phase 2 (TypeScript) and Phase 3 (Rust), not at approval. p1_ac3 here accepts scafld gate `approve` as the precondition, but at gate=approve the schema is still absent from packages/contracts and crates/runx-contracts (verified by grep — `runx.operational_proposal.v1` appears only in the two draft specs). So Phase 2 fixtures and Phase 3 tests can land referencing a schema id that does not yet exist in code.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac5
  - Result: failed
  - Evidence: Phase 3 commits to adding tests proving (a) build fix does not require prior check and (b) check does not grant mutation. p3_ac5 enforces this via `rg -n '<token>' packages/core fixtures/operational-proposal`. The patterns are matched anywhere in those trees, including fixture JSON values, fixture README text, or doc comments inside packages/core. A reviewer added text like `"note": "prior check is advisory"` in a fixture satisfies the gate without any *.test.ts file existing. The behavioral promise is not anchored to a test file path or describe/it identifier in a *.test.ts. p2_ac2 leak guard scopes only to fixtures/operational-proposal/public/, so private payloads placed under a sibling subdir (e.g., fixtures/operational-proposal/escalation/) skip the leak scan entirely.
- rollback/repair audit
  - Grounded in: derived_spec:rollback
  - Result: passed
  - Evidence: Spec is additive: docs, fixtures, optional helper updates, and tests. Out-of-scope explicitly excludes provider API calls, auto-send, auto-merge, billing, destructive provider mutation, and legacy aliases. Rollback section says revert docs/fixtures/helper/tests and forbids touching provider messages, GitHub issues/PRs, customer state, or consuming product config. Revert is credible because nothing external mutates.
- design challenge
  - Grounded in: archive:.scafld/specs/archive/2026-05/runx-outreach-proposal-v1.md
  - Result: passed
  - Evidence: Archived fixed-lane drafts (outreach-proposal, roadmap-signal, support-triage-response, alert-dev-escalation) plus parent operational-intelligence-action-layer-v1 and runx-operational-contracts-v1 all converge on one generic composition spine + namespaced proposal_kind. oss/CLAUDE.md keeps the operational policy action enum closed and pushes admin/control-plane to cloud/. The composition design therefore fits the kernel boundary, is not a bandaid, and avoids future enum/lane bloat. The remaining concerns are gate executability, not architecture.

Issues:
- [high/blocks approval] `harden-1` dependency_ordering - p1_ac3 accepts scafld gate `approve` as sufficient, but at that gate the runx.operational_proposal.v1 schema is still absent from packages/contracts and crates/runx-contracts.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-operational-contracts-v1.md:14
  - Evidence: Grep across oss/ shows `runx.operational_proposal.v1` exists only in the two draft spec files; no schema file declares it. runx-operational-contracts-v1.md is at status=draft, harden_status=passed, Next: approve — its TypeScript schema lands in Phase 2 and Rust in Phase 3, not at the approve gate. The current p1_ac3 set ['approve','build','review','complete'] passes as soon as the contracts spec is approved, before the schema is on disk. Phase 2 fixtures and Phase 3 tests then reference a schema id that does not yet exist.
  - Recommendation: Tighten the dependency precondition by adding a direct on-disk check, e.g. an acceptance step at the start of Phase 2 such as `rg -n 'runx\.operational_proposal\.v1' packages/contracts/src/schemas crates/runx-contracts/src` so this spec fails closed until the contracts implementation actually lands. Alternatively narrow p1_ac3 to ['build','review','complete'] so approval here cannot happen until contracts has built Phase 2.
  - Question: Should the precondition check that runx.operational_proposal.v1 is on disk in packages/contracts and/or crates/runx-contracts, rather than just that the contracts spec is at scafld gate `approve`?
  - Recommended answer: Yes — add a positive rg check for the schema id in packages/contracts/src/schemas (and the Rust analog), and either remove `approve` from the accepted gate set or pair the gate check with the rg check. Otherwise fixtures here ship pointing at a schema id that may not exist.
  - If unanswered: Default to adding a Phase 2 prelude acceptance: `rg -n 'runx\.operational_proposal\.v1' packages/contracts/src/schemas crates/runx-contracts/src` and removing `approve` from the p1_ac3 accept set.
- [high/blocks approval] `harden-2` weak_gate - Phase 2 leak guard only scans fixtures/operational-proposal/public/; private payloads placed under any other subdirectory bypass the scan entirely.
  - Status: open
  - Grounded in: spec_gap:phases.phase2.acceptance.p2_ac2
  - Evidence: p2_ac2 is `test -d fixtures/operational-proposal/public && ! rg -n '<patterns>' fixtures/operational-proposal/public`. The scope is the `public/` subtree only. Phase 2 changes say `Keep private source payloads separate from public expected outputs` but do not pin where private fixtures live; nothing forbids `fixtures/operational-proposal/escalation/raw-event.json` from containing a Slack token or `/Users/...` path. The token list (`/Users/`, `RUNX_BIN=`, `SENTRY_AUTH_TOKEN`, `xox[baprs]-`, `url_private_download`, `raw_payload`, `BEGIN .*PRIVATE KEY`) is meaningful, but the scan area is too narrow.
  - Recommendation: Either (a) scope the leak rg to the entire `fixtures/operational-proposal/` tree and add a separate gate that requires public/ to exist with N>0 expected-output files, or (b) require a strict `public/` and `private/` split with the leak scan applied to the full directory minus an explicit private allowlist that is itself audited.
  - Question: Should the leak scan cover the full `fixtures/operational-proposal/` tree (with a documented `public/` subdir for redacted outputs), rather than only `fixtures/operational-proposal/public/`?
  - Recommended answer: Yes — scan the full tree so misplaced private payloads cannot hide in a sibling subdir. Keep the requirement that public/ exists, but apply the leak scan tree-wide.
  - If unanswered: Default to `! rg -n '<patterns>' fixtures/operational-proposal` (whole tree) plus the existing public/ existence requirement.
- [high/blocks approval] `harden-3` acceptance_gap - p3_ac5 is satisfied by any matching text in `packages/core` or `fixtures/operational-proposal/**`, not by an actual test file being added.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac5
  - Evidence: The gate `for token in 'build fix without prior check' 'check.*does not grant mutation' 'prior check.*advisory'; do rg -n "$token" packages/core fixtures/operational-proposal; done` matches text anywhere in those trees. A fixture JSON `"note": "prior check is advisory"` or a doc paragraph in packages/core satisfies the gate, even if zero *.test.ts files are added. Phase 3's change item is `Add tests proving build fix does not require a prior check and check does not grant mutation permission`, so the behavioral promise is not anchored to a test file.
  - Recommendation: Anchor the gate to test files: e.g. `rg -n 'build fix without prior check' packages/core/src --type ts -g '*.test.ts'` and `rg -n 'check is read-only|check does not grant mutation' packages/core/src -g '*.test.ts'`. Optionally combine with a count check (`rg -c ... | awk '$NF>0'`).
  - Question: Should p3_ac5 require the tokens to appear specifically in a `*.test.ts` file (not in fixtures or doc comments)?
  - Recommended answer: Yes — pin the grep to packages/core/src with `-g '*.test.ts'` so the gate cannot be satisfied by description text or fixture JSON. Keep fixtures as evidence, not as test substitutes.
  - If unanswered: Default to scoping p3_ac5 to `packages/core/src -g '*.test.ts'` and dropping `fixtures/operational-proposal` from its search paths.
- [medium/advisory] `harden-4` audit_scope - p3_ac4 only flags fixed-lane names that appear as literal double-quoted strings; YAML keys, directory names, and bare identifiers slip past.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac4
  - Evidence: The pattern is `"support-response"|"dev-escalation"|"outreach-proposal"|"roadmap-signal"` with the quotes embedded. A new skill at `skills/dev-escalation/SKILL.md` with `name: dev-escalation` in YAML (no surrounding quotes) would not trip; a Rust enum variant `DevEscalation` or `SupportResponse` also would not. rg by default does not search file path names, so the directory name escapes too.
  - Recommendation: Drop the surrounding double-quotes from the patterns or add a parallel check that no `skills/<fixed-lane-name>/` directory exists. Optionally also forbid the snake_case forms (`dev_escalation`, etc.) in the new schema modules.
  - Question: Should the negative-lane check also flag bare identifiers and directory names, not just quoted strings?
  - Recommended answer: Yes — drop the quotes from the patterns and add a `find skills -maxdepth 2 -type d` step (or `ls skills`) that errors if any directory name matches the forbidden lane set. This closes the obvious bypass routes.
  - If unanswered: Default to dropping the quotes from the rg patterns and adding `! ls -1 skills | rg -x 'dev-escalation|outreach-proposal|roadmap-signal|support-response'` as a parallel check.
- [low/advisory] `harden-5` boundary_check - Phase 4 still plans to document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes in OSS docs; oss/CLAUDE.md keeps admin/control-plane (including approval inboxes) in cloud/.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/CLAUDE.md
  - Evidence: oss/CLAUDE.md states `Admin/control-plane features do not belong in oss/` and explicitly lists `approval inboxes` and adoption analytics as cloud/ surfaces. Phase 4 wording `Document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes` is broad enough to land workflow details (queue UX, approval flow) in OSS docs.
  - Recommendation: Narrow Phase 4 to documenting the contract/readback shape that hosted consumes (schema fields, receipt linkage, story milestones) and reference cloud/ as the owner of the approval-queue workflow. Update p4_ac1 anchors to favor contract-surface vocabulary over workflow vocabulary.
  - Question: Should Phase 4 OSS docs be limited to the contract/readback surface, with the hosted approval-queue workflow documented in cloud/?
  - Recommended answer: Yes — keep OSS docs at the contract surface (what hosted consumes), and note that hosted workflow docs live in cloud/. Avoid placing Aster approval-inbox UX in OSS docs.
  - If unanswered: Default to a Phase 4 wording change: replace `hosted readback expectations for proposal queues, approvals…` with `the OSS contract surface that hosted consumes` plus a one-line pointer that hosted workflow docs live in cloud/.

### round-4

Status: needs_revision
Started: 2026-05-27T17:38:38Z
Ended: 2026-05-27T17:38:38Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-3 → round-4 fixes resolved most prior blockers: p1_ac4 now asserts docs/operational-intelligence.md exists with spine tokens; p3_ac5 now scopes to `*.test.ts` files in packages/core (anchors the behavioral promise to real tests); p2_ac2 now scans the whole fixtures/operational-proposal tree minus private/raw subdirs; p4_ac1 anchors on "## Consuming Application Boundary", `proposal_kind`, `owner_route_id`, `Aster`, and `source-thread`. Architecture (one composition spine + generic `proposal_kind`, escalation as proposal_kind, build-fix preserved as governed path) remains the right move per archived fixed-lane drafts and oss/CLAUDE.md. One blocking executability gap remains: the new p2_ac0 includes docs/operational-intelligence.md alongside packages/contracts/src and crates/runx-contracts/src as satisfying paths for `runx.operational_proposal.v1`/`proposal_kind`/`source_thread_locator`. Phase 1 writes those tokens into docs/operational-intelligence.md, so p2_ac0 is trivially satisfied without the schema actually landing in TypeScript or Rust. This re-opens round-3's harden-1 (dependency ordering) under a new wrapper. Two remaining issues are advisory: p3_ac4 still pattern-matches quoted strings only (directory names and YAML keys bypass), and Phase 4 still plans to document Aster/hosted readback workflows in OSS docs.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/docs
  - Result: passed
  - Evidence: Verified existing touchpoints: docs/issue-to-pr.md, docs/thread-story-contract.md, docs/developer-issue-inbox.md all exist (rg over docs/). packages/contracts/src/schemas/operational-policy.ts exists with closed `operationalPolicyActions` enum (lines 12-32 per round-3 path audit). docs/operational-intelligence.md does NOT exist yet (rg returned no files) — intentional, created in Phase 1. fixtures/operational-proposal/ also intentionally future per Phase 2.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/package.json:33
  - Result: passed
  - Evidence: package.json defines `typecheck` (line 33), `test:fast` (line 35), `boundary:check` (line 37). Token-grep loops in p1_ac2, p1_ac4, p2_ac0, p2_ac1, p2_ac3, p2_ac4, p3_ac5, p4_ac1 use POSIX sh and rg syntax that is valid. p2_ac2 leak scan now uses `--glob '!private/**' --glob '!inputs/private/**' --glob '!raw/**'` to exclude known-private subdirs while scanning the rest of fixtures/operational-proposal. p3_ac5 now scopes to `packages/core --glob '*.test.ts'`.
- scope/migration audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-operational-contracts-v1.md:14
  - Result: failed
  - Evidence: runx-operational-contracts-v1 is still status=draft, harden_status=passed, Next: approve. Grep confirms `operational_proposal` / `runx.operational_proposal.v1` appear in zero files under packages/contracts/src and zero files under crates/runx-contracts/src — the schema has not landed in code. p2_ac0 attempts to gate on this with `rg -n "$token" packages/contracts/src crates/runx-contracts/src docs/operational-intelligence.md`, but rg succeeds when ANY of the three paths contains the token. Phase 1 writes the spine (`proposal_kind`, the spec id, and source-thread vocabulary) into docs/operational-intelligence.md, so after Phase 1 the docs file alone satisfies p2_ac0 for all three tokens. The intended schema-on-disk precondition from round-3 harden-1 is therefore effectively neutralized.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac5
  - Result: passed
  - Evidence: Round-2 and round-3 acceptance-timing gaps are now closed: p1_ac4 asserts docs/operational-intelligence.md exists and contains the spine vocabulary plus `proposal_kind: escalation`; p3_ac5 now scopes the grep for `build fix without prior check`, `check.*does not grant mutation`, and `prior check.*advisory` to `packages/core --glob '*.test.ts'`, so the gate cannot be satisfied by fixture JSON or doc comments; p2_ac2 leak scan now runs against the whole fixtures/operational-proposal tree minus private/raw subdirs. p4_ac1 anchors on `## Consuming Application Boundary` plus the genuinely new vocabulary (`proposal_kind`, `owner_route_id`). The remaining executability concern is encoded in the scope/migration check, not acceptance timing.
- rollback/repair audit
  - Grounded in: derived_spec:rollback
  - Result: passed
  - Evidence: Spec is additive — docs, fixtures, optional helper updates, tests. Out-of-scope explicitly excludes provider API calls, auto-send, auto-merge, billing/account mutation, destructive provider mutation, and legacy aliases. Rollback section commits to reverting docs/fixtures/helper/tests and forbids touching provider messages, GitHub issues/PRs, customer state, or consuming product config. Acceptance includes `git diff --check -- docs skills packages fixtures .scafld/specs/drafts/runx-operational-proposal-composition-v1.md`. Revert is credible because no external state changes.
- design challenge
  - Grounded in: archive:.scafld/specs/archive/2026-05/runx-outreach-proposal-v1.md
  - Result: passed
  - Evidence: Archived fixed-lane drafts (outreach-proposal, roadmap-signal, support-triage-response, alert-dev-escalation) and parent runx-operational-intelligence-action-layer-v1 all converged on a generic composition spine with namespaced `proposal_kind`. oss/CLAUDE.md keeps the operational policy action enum closed (`operationalPolicyActions` in packages/contracts/src/schemas/operational-policy.ts is closed) and pushes admin/control-plane (approval inboxes, adoption analytics) to cloud/. The composition design fits the trusted-kernel boundary, is not a bandaid, and avoids future enum/lane bloat. Remaining concerns are gate executability and one OSS/cloud boundary nuance in Phase 4 docs scope — not architectural.

Issues:
- [high/blocks approval] `harden-1` dependency_ordering - p2_ac0 includes docs/operational-intelligence.md as a satisfying path, so the schema-on-disk precondition is neutralized once Phase 1 writes the spine into docs.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-operational-contracts-v1.md:14
  - Evidence: Round-3 harden-1 asked for an on-disk check that runx.operational_proposal.v1 / proposal_kind / source_thread_locator exist in packages/contracts/src and/or crates/runx-contracts/src before Phase 2 fixtures land. The current p2_ac0 is `for token in 'runx.operational_proposal.v1' 'proposal_kind' 'source_thread_locator'; do rg -n "$token" packages/contracts/src crates/runx-contracts/src docs/operational-intelligence.md >/dev/null || exit 1; done`. rg succeeds when ANY of the listed paths contains the token. Phase 1 explicitly writes `proposal_kind: escalation`, the source/context/signal/decision/proposal/action/outcome spine, and originating-source-thread vocabulary into docs/operational-intelligence.md. Once Phase 1 lands, the docs file alone satisfies p2_ac0 — even if zero TypeScript or Rust schema work has happened. Verified: grep across packages/contracts/src and crates/runx-contracts/src finds zero files mentioning `operational_proposal`. runx-operational-contracts-v1 is still at status=draft, Next=approve; its TypeScript schema lands in its own Phase 2 and Rust in its own Phase 3, neither of which is gated by approval.
  - Recommendation: Either (a) drop `docs/operational-intelligence.md` from the p2_ac0 path list so the grep only resolves against `packages/contracts/src` and `crates/runx-contracts/src`, or (b) split p2_ac0 into two checks: a docs-vocabulary check (already covered by p1_ac4) and a schema-on-disk check `for token in 'runx.operational_proposal.v1' 'proposal_kind' 'source_thread_locator'; do rg -n "$token" packages/contracts/src crates/runx-contracts/src >/dev/null || exit 1; done`. Option (a) is simpler and matches the round-3 recommendation.
  - Question: Should p2_ac0 require the schema id, proposal_kind, and source_thread_locator to appear in packages/contracts/src and/or crates/runx-contracts/src — not in docs/operational-intelligence.md, which Phase 1 writes — so the schema-on-disk precondition is real?
  - Recommended answer: Yes — drop docs/operational-intelligence.md from p2_ac0 so the gate only resolves against the schema source paths. The docs vocabulary check is already covered by p1_ac4, so removing it from p2_ac0 does not lose coverage; it just restores the precondition's teeth.
  - If unanswered: Default to removing docs/operational-intelligence.md from the p2_ac0 search paths, leaving `rg -n "$token" packages/contracts/src crates/runx-contracts/src` as the precondition.
- [medium/advisory] `harden-2` audit_scope - p3_ac4 still only flags double-quoted fixed-lane string literals; YAML names, directory names, snake_case forms, and Rust enum variants bypass the gate.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac4
  - Evidence: The current pattern is `"support-response"|"dev-escalation"|"outreach-proposal"|"roadmap-signal"` with the double-quotes embedded in the regex. A skill at `skills/dev-escalation/SKILL.md` with `name: dev-escalation` (YAML, no surrounding quotes) does not match; a Rust enum variant `DevEscalation` does not match; rg does not by default search directory names. The negative-grep is therefore a thin sentinel and an obvious bypass route exists.
  - Recommendation: Drop the surrounding double-quotes from the regex so kebab-case identifiers are caught anywhere they appear, and add a parallel directory check such as `! ls -1 skills | rg -x 'dev-escalation|outreach-proposal|roadmap-signal|support-response'`. Optionally also forbid the snake_case forms in schema modules.
  - Question: Should the forbidden-lane gate cover YAML keys, directory names, and snake_case forms as well as quoted strings?
  - Recommended answer: Yes — drop the quotes and add a directory existence check. The current literal-quoted form is too easy to bypass and gives a false sense of safety.
  - If unanswered: Default to dropping the quotes from the rg patterns and adding `test -z "$(ls -1 skills 2>/dev/null | rg -x 'dev-escalation|outreach-proposal|roadmap-signal|support-response')"`.
- [low/advisory] `harden-3` boundary_check - Phase 4 still plans to document Aster/hosted readback expectations for proposal queues, approvals, and runners in OSS docs; oss/CLAUDE.md keeps approval-inbox/control-plane in cloud/.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/CLAUDE.md
  - Evidence: oss/CLAUDE.md: `Admin/control-plane features do not belong in oss/.` Explicit cloud/ surfaces include `approval inboxes` and adoption analytics. Phase 4 wording remains broad: `Document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes.` That language is broad enough to land workflow-level UX or approval-queue admin details in OSS docs.
  - Recommendation: Narrow Phase 4 to documenting the contract/readback surface that hosted consumes (schema fields, receipt linkage, story milestones) and reference cloud/ as the owner of the approval-queue workflow. Optionally update p4_ac1 anchors to favor contract-surface vocabulary (e.g., `readback_shape`, `receipt_ref`) over workflow vocabulary.
  - Question: Should Phase 4 OSS docs be limited to the contract/readback surface, with hosted approval-queue workflow documentation kept in cloud/?
  - Recommended answer: Yes — keep OSS docs at the contract surface and explicitly note that hosted workflow docs live in cloud/. Avoid placing Aster approval-inbox UX or queue management specifics in OSS.
  - If unanswered: Default to a Phase 4 wording change: replace `hosted readback expectations for proposal queues, approvals…` with `the OSS contract surface that hosted consumes` plus a one-line pointer that hosted workflow docs live in cloud/.

### round-5

Status: passed
Started: 2026-05-27T17:44:07Z
Ended: 2026-05-27T17:44:07Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-5: all prior blocking executability gaps are resolved in the current draft. p2_ac0 now greps only `packages/contracts/src crates/runx-contracts/src` (docs path removed), restoring the schema-on-disk precondition the round-4 harden-1 wanted; verified `operational_proposal` is absent from both trees today and runx-operational-contracts-v1 is still draft/Next=approve, so this gate will block Phase 2 fixtures until the sibling spec actually lands TS+Rust schemas. p3_ac4 now drops the embedded double-quotes from the forbidden-lane regex, adds snake_case (`dev_escalation` etc.) and PascalCase (`DevEscalation` etc.) variants, and adds a parallel `find skills -mindepth 1 -maxdepth 1 -type d` directory check that errors if any forbidden lane directory exists. p1_ac4 asserts `docs/operational-intelligence.md` exists and contains the spine vocabulary plus `proposal_kind: escalation` and `originating source thread`. p3_ac5 anchors `build fix without prior check`, `check.*does not grant mutation`, and `prior check.*advisory` to `packages/core --glob '*.test.ts'`, so fixture JSON or doc comments cannot satisfy the gate. p2_ac2 scans the full `fixtures/operational-proposal` tree minus `private/**`, `inputs/private/**`, `raw/**` for known-leak patterns and still requires `public/` to exist. p4_ac1 anchors on the new `## Consuming Application Boundary` heading plus genuinely new vocabulary (`proposal_kind`, `owner_route_id`, `Aster`, `source-thread`). Architecture (one composition spine + generic `proposal_kind`, escalation as proposal_kind, issue-to-PR preserved as a direct governed action) remains the right move per archived fixed-lane drafts and oss/CLAUDE.md's closed `operationalPolicyActions` enum. One advisory remains: Phase 4's wording "Document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes" is broad enough to drift into hosted workflow territory that oss/CLAUDE.md reserves for cloud/; this is wording polish, not a blocker.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/docs
  - Result: passed
  - Evidence: Existing touchpoints verified across earlier rounds: docs/developer-issue-inbox.md, docs/issue-to-pr.md, docs/thread-story-contract.md, packages/core/src/source/index.ts, packages/core/src/knowledge/thread-story.ts, packages/core/src/knowledge/outbox.ts, packages/contracts/src/schemas/operational-policy.ts (closed operationalPolicyActions enum), skills/issue-intake/{SKILL.md,X.yaml}, skills/issue-triage/SKILL.md, skills/issue-to-pr/SKILL.md. Intentionally future paths: docs/operational-intelligence.md (created in Phase 1), fixtures/operational-proposal/ (created in Phase 2). Verified no operational-intelligence.md or fixtures/operational-proposal/ on disk now.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/package.json:33
  - Result: passed
  - Evidence: package.json defines `typecheck` (line 33), `test:fast` (line 35), `boundary:check` (line 37). All token-grep loops use POSIX sh and rg syntax that resolves. p2_ac2 uses `--glob '!private/**' --glob '!inputs/private/**' --glob '!raw/**'`. p3_ac4 combines a multi-form negative grep across `packages crates skills` with a `find skills -mindepth 1 -maxdepth 1 -type d` directory check piped through `sed`/`rg -x`. p3_ac5 scopes to `packages/core --glob '*.test.ts'`. p1_ac3 inspects `scafld status runx-operational-contracts-v1 --json` and accepts gate=approve or status in {approved,in_progress,review,completed}; p2_ac0 separately asserts the schema id is on disk in packages/contracts/src and crates/runx-contracts/src, so the dependency precondition has teeth even at gate=approve.
- scope/migration audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-operational-contracts-v1.md:14
  - Result: passed
  - Evidence: runx-operational-contracts-v1 is status=draft, harden_status=passed, Next=approve. Grep across packages/contracts/src and crates/runx-contracts/src confirms zero references to operational_proposal today. The round-4 concern that p2_ac0 included docs/operational-intelligence.md and could be satisfied by docs alone is fixed in the current draft: p2_ac0 now reads `for token in 'runx.operational_proposal.v1' 'proposal_kind' 'source_thread_locator'; do rg -n "$token" packages/contracts/src crates/runx-contracts/src >/dev/null || exit 1; done`. This will fail-closed until the sibling spec actually lands TypeScript and Rust schemas, regardless of whether Phase 1 docs already mention `proposal_kind`.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac5
  - Result: passed
  - Evidence: p1_ac4 asserts `test -f docs/operational-intelligence.md` and that the spine vocabulary plus `proposal_kind: escalation` and `originating source thread` appear in the doc, so Phase 1's stated deliverable cannot be skipped. p3_ac5 anchors `build fix without prior check`, `check.*does not grant mutation`, and `prior check.*advisory` to `packages/core --glob '*.test.ts'`, so fixture JSON or doc comments cannot satisfy the behavioral promise. p2_ac2 requires `public/` to exist and runs the leak scan over the full tree minus private/raw subdirs, removing the silent-pass route. p4_ac1 anchors on the new `## Consuming Application Boundary` heading plus the new `proposal_kind`/`owner_route_id` vocabulary.
- rollback/repair audit
  - Grounded in: derived_spec:rollback
  - Result: passed
  - Evidence: Spec is strictly additive: docs, fixtures, optional helper updates, and tests. Out-of-scope explicitly excludes provider API calls, auto-send, auto-merge, billing/account mutation, destructive provider mutation, and legacy aliases. Rollback section commits to reverting docs/fixtures/helper/tests and forbids touching provider messages, GitHub issues/PRs, customer state, or consuming product config. Acceptance includes `git diff --check` across docs skills packages fixtures and the spec file. Revert is credible because nothing external mutates.
- design challenge
  - Grounded in: archive:.scafld/specs/archive/2026-05/runx-outreach-proposal-v1.md
  - Result: passed
  - Evidence: Archived fixed-lane drafts (outreach-proposal, roadmap-signal, support-triage-response, alert-dev-escalation) and parent runx-operational-intelligence-action-layer-v1 converged on a single composition spine with namespaced `proposal_kind`. oss/CLAUDE.md keeps the operational policy action enum closed (packages/contracts/src/schemas/operational-policy.ts operationalPolicyActions) and pushes admin/control-plane (approval inboxes, adoption analytics, registry moderation) to cloud/. The composition design fits the trusted-kernel boundary, avoids per-domain enum bloat, and preserves issue-to-PR as a direct governed action without forcing a prior check. The remaining concern is wording in Phase 4 about hosted readback documentation, which is advisory.

Issues:
- [low/advisory] `harden-1` boundary_check - Phase 4 still plans to document Aster/hosted readback expectations for proposal queues, approvals, and runners in OSS docs; CLAUDE.md keeps approval-inbox/control-plane in cloud/.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/CLAUDE.md:62
  - Evidence: CLAUDE.md states `Admin/control-plane features do **not** belong in `oss/`.` and explicitly lists approval inboxes as a cloud/ surface. Phase 4 wording remains `Document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes.` That language is broad enough to land workflow-level UX or approval-queue admin details in OSS docs. Reading it charitably it can mean only the contract surface hosted consumes (schema fields, receipt linkage, story milestones), but the wording does not pin to that.
  - Recommendation: Narrow Phase 4 to documenting the contract/readback surface (schema fields, receipt linkage, story milestones) and explicitly reference cloud/ as the owner of the approval-queue workflow. Optionally update p4_ac1 to favor contract-surface vocabulary (e.g., `readback_shape`, `receipt_ref`) over workflow vocabulary, and add a sentence to OSS docs noting that hosted workflow docs live in cloud/.
  - Question: Should Phase 4 OSS docs be limited to the contract/readback surface, with the hosted approval-queue workflow documentation kept in cloud/?
  - Recommended answer: Yes — keep OSS docs at the contract surface and note that hosted workflow docs live in cloud/. Avoid placing Aster approval-inbox UX or queue management specifics in OSS docs.
  - If unanswered: Default to a Phase 4 wording change: replace `hosted readback expectations for proposal queues, approvals…` with `the OSS contract surface that hosted consumes` plus a one-line pointer that hosted workflow docs live in cloud/.

### round-6

Status: passed
Started: 2026-05-27T17:51:46Z
Ended: 2026-05-27T17:51:46Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-6 re-verifies that the draft has not regressed since round-5 passed. All previously blocking executability gaps remain closed: p2_ac0 scopes only to `packages/contracts/src crates/runx-contracts/src` and will fail closed today (verified: zero references to `operational_proposal` in either tree), restoring the schema-on-disk precondition; p3_ac4 drops quotes from forbidden-lane regex, adds snake_case/PascalCase variants, and adds a `find skills` directory check; p1_ac4 pins `docs/operational-intelligence.md` existence and spine vocabulary; p3_ac5 anchors behavioral-promise tokens to `packages/core --glob '*.test.ts'` so fixture JSON cannot satisfy the gate; p2_ac2 runs leak scan over the full `fixtures/operational-proposal` tree minus private/raw subdirs and requires `public/` to exist; p4_ac1 anchors on the new `## Consuming Application Boundary` heading plus genuinely new vocabulary (`proposal_kind`, `owner_route_id`). The composition design (one spine + generic `proposal_kind`, escalation as proposal_kind, issue-to-PR preserved as direct governed action) fits the trusted-kernel boundary and matches the archived fixed-lane drafts' resolution. One advisory remains unchanged from round-5: Phase 4's wording about "Document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes" is broad enough to drift into hosted workflow territory that oss/CLAUDE.md reserves for cloud/ — wording polish, not a blocker. Rollback is credible (strictly additive, no provider mutations). Verdict: pass.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/docs
  - Result: passed
  - Evidence: Verified intentionally-future paths are still future: Glob found no docs/operational-intelligence.md and no fixtures/operational-proposal/ on disk — both created in Phase 1/Phase 2 respectively. Existing touchpoints confirmed across prior rounds: docs/issue-to-pr.md, docs/developer-issue-inbox.md, docs/thread-story-contract.md, packages/core/src/source/index.ts, packages/core/src/knowledge/thread-story.ts, packages/core/src/knowledge/outbox.ts, packages/contracts/src/schemas/operational-policy.ts (closed operationalPolicyActions enum lines 23-32), skills/issue-intake/{SKILL.md,X.yaml}, skills/issue-triage/SKILL.md, skills/issue-to-pr/SKILL.md.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/package.json:33
  - Result: passed
  - Evidence: package.json defines `typecheck` (line 33), `test:fast` (line 35), and `boundary:check` (line 37). All token-grep loops use POSIX sh and rg syntax that resolves. p2_ac2 uses `--glob '!private/**' --glob '!inputs/private/**' --glob '!raw/**'` to exclude known-private subdirs. p3_ac4 combines a multi-form negative grep across `packages crates skills` with `find skills -mindepth 1 -maxdepth 1 -type d` piped through sed/rg -x. p3_ac5 scopes to `packages/core --glob '*.test.ts'`. p2_ac0 greps only `packages/contracts/src crates/runx-contracts/src`.
- scope/migration audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-operational-contracts-v1.md:6
  - Result: passed
  - Evidence: runx-operational-contracts-v1 frontmatter shows status=draft, harden_status=passed (lines 6-7). Grep across packages/contracts/src and crates/runx-contracts/src for `operational_proposal|runx.operational_proposal.v1` returns zero files today. Current p2_ac0 is `for token in 'runx.operational_proposal.v1' 'proposal_kind' 'source_thread_locator'; do rg -n "$token" packages/contracts/src crates/runx-contracts/src >/dev/null || exit 1; done`. With both target trees empty of the schema, p2_ac0 will fail closed until the sibling contracts spec actually lands TypeScript and Rust schemas, regardless of how Phase 1 docs read. The dependency-ordering precondition has real teeth.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase1.acceptance.p1_ac4
  - Result: passed
  - Evidence: Phase 1: p1_ac4 asserts `test -f docs/operational-intelligence.md` AND that spine vocabulary plus `proposal_kind: escalation` and `originating source thread` appear in the doc — the doc cannot be skipped. Phase 2: p2_ac0 is a real on-disk precondition; p2_ac1/p2_ac3/p2_ac4 require specific fixture tokens; p2_ac2 runs leak scan over the full fixtures/operational-proposal tree minus private/raw subdirs and requires `public/` to exist. Phase 3: p3_ac5 anchors `build fix without prior check`, `check.*does not grant mutation`, `prior check.*advisory` to `packages/core --glob '*.test.ts'` so fixture JSON or doc comments cannot satisfy the gate. Phase 4: p4_ac1 anchors on the new `## Consuming Application Boundary` heading plus `proposal_kind`/`owner_route_id`/`Aster`/`source-thread`.
- rollback/repair audit
  - Grounded in: derived_spec:rollback
  - Result: passed
  - Evidence: Spec is strictly additive: docs, fixtures, optional helper updates, tests. Out-of-scope explicitly excludes provider API calls, auto-send, auto-merge, billing/account mutation, destructive provider mutation, and legacy aliases. Rollback section commits to reverting docs/fixtures/helper/tests and forbids touching provider messages, GitHub issues/PRs, customer state, or consuming product config. Acceptance includes `git diff --check -- docs skills packages fixtures .scafld/specs/drafts/runx-operational-proposal-composition-v1.md`. Revert is credible because nothing external mutates.
- design challenge
  - Grounded in: archive:.scafld/specs/archive/2026-05/runx-outreach-proposal-v1.md
  - Result: passed
  - Evidence: Archived fixed-lane drafts (outreach-proposal, roadmap-signal, support-triage-response, alert-dev-escalation) plus parent runx-operational-intelligence-action-layer-v1 converged on a single composition spine with namespaced `proposal_kind`. oss/CLAUDE.md keeps the operational policy action enum closed (verified at packages/contracts/src/schemas/operational-policy.ts:23-32) and pushes admin/control-plane (approval inboxes, adoption analytics, registry moderation) to cloud/. The composition design fits the trusted-kernel boundary, avoids per-domain enum bloat, and preserves issue-to-PR as a direct governed action without forcing a prior check. Remaining advisory concern is Phase 4 wording about hosted readback documentation, which is wording polish, not architecture.

Issues:
- [low/advisory] `harden-1` boundary_check - Phase 4 still plans to document Aster/hosted readback expectations for proposal queues, approvals, and runners in OSS docs; CLAUDE.md keeps approval-inbox/control-plane in cloud/.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/CLAUDE.md:62
  - Evidence: CLAUDE.md line 62 states `Admin/control-plane features do **not** belong in `oss/`.` and lines 63-65 explicitly list `approval inboxes` and adoption analytics as cloud/ surfaces. Phase 4 wording (spec lines 298-299) remains `Document Aster/hosted readback expectations for proposal queues, approvals, runners, receipts, and outcomes.` That language is broad enough to land workflow-level UX or approval-queue admin details in OSS docs. Reading it charitably it can mean only the contract surface hosted consumes (schema fields, receipt linkage, story milestones), but the wording does not pin to that. p4_ac1 anchors on `## Consuming Application Boundary`, `proposal_kind`, `owner_route_id`, `Aster`, `source-thread` — none of which forbid documenting hosted workflow itself.
  - Recommendation: Narrow Phase 4 to documenting the contract/readback surface (schema fields, receipt linkage, story milestones) and explicitly reference cloud/ as the owner of the approval-queue workflow. Optionally update p4_ac1 to favor contract-surface vocabulary over workflow vocabulary and add a sentence to the OSS doc noting that hosted workflow docs live in cloud/. Not a blocker — wording polish.
  - Question: Should Phase 4 OSS docs be limited to the contract/readback surface, with the hosted approval-queue workflow documentation kept in cloud/?
  - Recommended answer: Yes — keep OSS docs at the contract surface and note that hosted workflow docs live in cloud/. Avoid placing Aster approval-inbox UX or queue management specifics in OSS docs.
  - If unanswered: Default to a Phase 4 wording change: replace `hosted readback expectations for proposal queues, approvals…` with `the OSS contract surface that hosted consumes` plus a one-line pointer that hosted workflow docs live in cloud/.


## Planning Log

- Created after subagent review found support/alert/outreach/roadmap lanes were
  over-specific for runx core.
