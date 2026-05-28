---
spec_version: '2.0'
task_id: runx-operational-story-outbox-v1
created: '2026-05-27T15:02:28Z'
updated: '2026-05-28T07:34:19Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Operational Story And Outbox

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-28T07:34:19Z
Review gate: pass

## Summary

Make operational followups readable for humans without turning provider threads
or work-item comments into raw context dumps. This child owns the generic
projection layer: concise source-thread milestones, reviewer-safe tracking
item/change request/proposal summaries, hidden receipt/artifact refs,
idempotent outbox entries, and provider-neutral message templates.

The public story should answer: what was received, what runx concluded, what
proposal or action exists, what the human needs to do, and how it ended. Full
local paths, command dumps, provider payloads, verbose model reasoning, and
private customer/account data stay behind receipts and artifacts.

Golden path invariant: originating source thread/event -> hydrated context
artifact -> optional read-only check/triage -> create/update a tracking item
when requested -> optional build-fix change request without requiring a prior
check -> governed change request with human final-change gate -> final outcome
posted back to the originating source thread and linked references.

## Objectives

- Define one canonical v1 milestone id set that covers tracking-to-change and proposal
  workflows without per-domain milestone sprawl:
  - `accepted`;
  - `hydrated`;
  - `triaged`;
  - `reply_drafted`;
  - `ask_for_info`;
  - `proposal_ready`;
  - `escalation_proposed`;
  - `tracking_item_created`;
  - `spec_ready`;
  - `build_started`;
  - `review_requested`;
  - `change_request_created`;
  - `review_fixup`;
  - `human_gate`;
  - `outcome_observed`;
  - `final_outcome`;
  - `no_action`;
  - `monitor`.
- Let renderers show friendly labels such as "Dev escalation proposed" or
  "Outreach proposal ready" from `proposal_kind`, without accepting those labels
  as data ids.
- Define density rules for public messages:
  - one clear title/status;
  - source/context summary;
  - evidence bullets and safe excerpts;
  - decision and rationale;
  - proposal/action summary;
  - links to tracking item/change request/source artifacts;
  - exact next human action;
  - compact outcome closure.
- Keep long reasoning, raw provider context, command output, and receipt trees
  out of public comments while preserving artifact refs for audit.
- Make publication idempotent and thread-safe: root-channel noise should be
  prevented by fail-closed source-thread policy.
- Provide provider-friendly text/markdown renderers without baking
  provider-specific API calls into core.
- Reject unknown or legacy milestone ids instead of relying on aliases or
  freeform string fallback.

## Scope

In scope:

- Core story/outbox helpers:
  - `packages/core/src/knowledge/thread-story.ts`;
  - `packages/core/src/knowledge/feed-entry.ts`;
  - `packages/core/src/knowledge/outbox.ts`;
  - `packages/core/src/knowledge/file-thread.ts`;
  - `packages/core/src/knowledge/index.ts`;
  - `packages/core/src/knowledge/index.test.ts`.
- Tool wrappers that consume story/outbox milestones:
  - `tools/outbox/build_feed_entry/**`;
  - `tools/outbox/build_pull_request/**`;
  - `tools/thread/push_outbox/**`.
- Tool tests that hardcode milestone ids:
  - `tests/outbox-build-feed-entry-tool.test.ts`;
  - `tests/thread-push-outbox-tool.test.ts`.
- Contract touchpoints only when required:
  - `packages/contracts/src/schemas/thread-outbox-provider.ts`;
  - `packages/contracts/src/schemas/run-summary.ts`;
  - `packages/contracts/src/schemas/artifact.ts`;
  - `crates/runx-contracts/src/thread_outbox_provider.rs`;
  - `crates/runx-contracts/tests/thread_outbox_provider_fixtures.rs`.
- Docs:
  - `docs/thread-story-contract.md`;
  - `docs/issue-to-pr.md`;
  - `docs/developer-issue-inbox.md`.
- Fixtures:
  - `fixtures/threads/**`;
  - `fixtures/operational-proposal/story-outbox/**`;
  - `fixtures/contracts/thread-outbox-provider/**` when contract fixtures
    change.

Out of scope:

- Chat, work-tracking, support-tool, or alert-provider API calls.
- Nitrosend-specific provider block layouts, channel ids, app buttons, or copy.
- Customer sends, final change approval authority, billing mutations, or
  provider actions.
- Domain-specific story renderers in runx core.
- Dumping model chain-of-thought or full local command logs into public output.

## Dependencies

- `runx-operational-contracts-v1` for proposal and authority semantics.
- `runx-operational-proposal-composition-v1` for flow composition and proposal
  examples.
- Existing provider outbox contract:
  - `packages/contracts/src/schemas/thread-outbox-provider.ts`;
  - `crates/runx-contracts/src/thread_outbox_provider.rs`.
- Existing intake and tracking-to-change lanes.
- Nitrosend integration child for provider UX and live dogfood.

## Assumptions

- Public messages can contain safe summaries and artifact refs, but not raw
  provider payloads or full receipt bodies.
- A source thread ref or locator is required whenever policy says updates must
  return to the originating thread.
- Final outcome publication links the source id, source thread ref, result refs
  for any tracking item/change request, and observed outcome so a human can
  reconstruct the flow without reading private artifacts.
- The existing thread outbox provider contract already owns public idempotency
  request/observation semantics. This child adds only core story/outbox helper
  metadata unless `runx-operational-contracts-v1` explicitly widens the public
  provider contract.
- Fail-closed source-thread enforcement is split: operational policy decides
  whether publication requires a source thread, and the knowledge helper only
  refuses to render or publish when that policy decision and locator are
  inconsistent.
- Renderer output should be provider-neutral text/markdown. Consuming adapters
  can translate to provider blocks, comments, or support notes.
- Replay should update or reuse known outbox entries where configured instead of
  posting duplicate root messages.
- Local paths in validation commands are never useful to a reviewer and must not
  appear in public story text.
- Core outbox idempotency metadata uses a stable key over source id, provider,
  source thread ref/locator, workflow/run id, lane id, milestone id, target ref
  when present, proposal id when present, and a normalized content hash.
- Same-key replay updates/reuses the outbox entry; different milestones, lanes,
  proposal ids, or target URLs must not collide.
- Canonical v1 milestone ids are hard-cut. Legacy or friendly labels such as
  `Change request created`, `pull_request`, `dev_escalation`, `outreach_proposal`,
  `human gate`, `outcome`, or `completion_update` are renderer copy only and
  must not be accepted as data ids.
- The canonical v1 milestone id set replaces the existing thread-story section,
  feed-entry milestone, and outbox metadata `milestone_kind` vocabularies for
  this operational story surface. This child must update `ThreadStorySectionId`,
  `FeedStoryMilestoneKind`, `outbox_entry.metadata.milestone_kind`, and the
  `outbox.build_feed_entry` / `thread.push_outbox` consumers together so there
  is one vocabulary rather than a parallel legacy namespace.
- The original change lifecycle must survive the hard cut by reshaping old
  lifecycle gates into canonical v1 milestones:
  `signal -> accepted`;
  `decision -> triaged`;
  `spec -> spec_ready`;
  `build -> build_started`;
  `review -> review_requested`;
  `pull_request -> change_request_created`;
  `merge_gate -> human_gate`;
  `outcome -> final_outcome`.
  These mappings are migration semantics only; runtime input must reject the
  legacy ids after cutover instead of accepting aliases.
- Published outbox entries that already carry legacy milestone ids must refresh
  into their canonical v1 entry without posting duplicate comments. The refresh
  lookup may use the legacy-to-canonical migration table only for previously
  published entries with the same source thread, target ref, provider,
  workflow/run id, lane id, and content lineage; it must preserve existing
  `comment_id`, locator, and receipt refs, then write the canonical milestone
  id on the refreshed entry. This is not runtime alias acceptance for new
  payloads.
- A single canonical vocabulary is intentional even though some ids read like
  actions and some read like sections: milestones are story events, while
  renderers own grouping, headings, and provider-specific labels.

## Touchpoints

- `packages/core/src/knowledge/thread-story.ts`
- `packages/core/src/knowledge/feed-entry.ts`
- `packages/core/src/knowledge/outbox.ts`
- `packages/core/src/knowledge/file-thread.ts`
- `packages/core/src/knowledge/index.ts`
- `packages/core/src/knowledge/index.test.ts`
- `tools/outbox/build_feed_entry/**`
- `tools/outbox/build_pull_request/**`
- `tools/thread/push_outbox/**`
- `tests/outbox-build-feed-entry-tool.test.ts`
- `tests/thread-push-outbox-tool.test.ts`
- `packages/contracts/src/schemas/thread-outbox-provider.ts`
- `packages/contracts/src/schemas/run-summary.ts`
- `packages/contracts/src/schemas/artifact.ts`
- `crates/runx-contracts/src/thread_outbox_provider.rs`
- `crates/runx-contracts/tests/thread_outbox_provider_fixtures.rs`
- `docs/thread-story-contract.md`
- `docs/issue-to-pr.md`
- `docs/developer-issue-inbox.md`
- `fixtures/threads/**`
- `fixtures/operational-proposal/story-outbox/**`

## Risks

- Overcorrection toward terse messages. Mitigation: every milestone requires
  enough context, evidence, links, proposal/action summary, and next action for
  the human gate.
- Verbose dumping. Mitigation: density rules separate public story from
  receipts/artifacts and tests block local paths, raw payloads, and command
  dumps.
- Provider root-channel noise. Mitigation: publication fails closed without a
  required thread locator.
- Provider lock-in. Mitigation: core renders provider-neutral text/markdown and
  outbox entries; adapters handle provider API details.
- Replay duplicates. Mitigation: outbox entries carry idempotency keys and
  update/reuse semantics.

## Acceptance

Profile: standard

Validation:
- `scafld validate runx-operational-story-outbox-v1`
- `pnpm typecheck`
- `pnpm test:fast`
- `pnpm boundary:check`
- If thread outbox contracts change:
  `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src`
- If Rust contracts change:
  `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features`

## Phase 1: Harden Story Density Rules

Status: completed
Dependencies: none

Objective: Challenge the public/private context boundary before implementation.

Changes:
- Define milestone vocabulary and public-message density rules.
- Define what belongs in public story versus private receipt/artifact.
- Define idempotency, missing-thread, and retry behavior.
- Define how `proposal_kind` becomes friendly copy without becoming a milestone id or fixed core lane.

Acceptance:
- [x] `p1_ac1` command - Public/private and idempotency boundaries are documented.
  - Command: `sh -c 'f=$(find .scafld/specs/drafts .scafld/specs/approved .scafld/specs/active -name runx-operational-story-outbox-v1.md -print -quit); test -n "$f" && for token in "provider contract already owns public idempotency" "proposal_kind" "source thread ref"; do rg -n "$token" "$f" >/dev/null || exit 1; done'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `p1_ac2` command - Spec validates after hardening edits.
  - Command: `scafld validate runx-operational-story-outbox-v1`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Fixture Matrix

Status: completed
Dependencies: Phase 1

Objective: Add representative public story snapshots before helper changes.

Changes:
- Add expected story outputs for reply drafted, ask-for-info, proposal ready, escalation proposed, issue created, spec ready, build started, review requested, change request created, human gate, monitor/no-action, outcome observed, and final outcome.
- Add private artifact refs in fixture inputs and public-safe refs in outputs.
- Include bad raw input cases containing local paths, command dumps, provider API fields, safe synthetic token markers, and customer identifiers. Fixtures must not contain real credential-like prefixes or strings that trigger repository secret scanning.
- Split fixtures into `inputs/private/` and `expected/public/`. Private inputs intentionally contain raw-looking leak markers; public expected outputs must contain only safe summaries and artifact refs.
- Add fail-closed missing-thread fixtures where policy requires source-thread publication and no locator is present.
- Add replay fixtures proving same-key milestones update/reuse and different milestones do not collide.
- Add text snapshots for source-thread updates, tracking-item comments, and change-request comments.

Acceptance:
- [x] `p2_ac1` command - Story fixtures include every generic milestone family.
  - Command: `test -d fixtures/operational-proposal/story-outbox/expected/public && for token in accepted hydrated triaged reply_drafted ask_for_info proposal_ready escalation_proposed tracking_item_created spec_ready build_started review_requested change_request_created review_fixup human_gate outcome_observed final_outcome no_action monitor; do rg -n "$token" fixtures/operational-proposal/story-outbox/expected/public >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-12
- [x] `p2_ac2` command - Private leak fixtures exist.
  - Command: `test -d fixtures/operational-proposal/story-outbox/inputs/private && rg -n "/Users/|RUNX_BIN=|url_private_download|slack_token_marker|sentry_token_marker|raw_provider_payload|private_key_marker" fixtures/operational-proposal/story-outbox/inputs/private`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13
- [x] `p2_ac3` command - Public outputs block obvious local/provider dumps.
  - Command: `sh -c 'test -d fixtures/operational-proposal/story-outbox/expected/public && if rg -n "/Users/|RUNX_BIN=|url_private_download|slack_token_marker|sentry_token_marker|raw_provider_payload|private_key_marker" fixtures/operational-proposal/story-outbox/expected/public; then exit 1; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `p2_ac4` command - Final outcome snapshots link source thread, tracking item, and change request story.
  - Command: `test -d fixtures/operational-proposal/story-outbox/expected/public && for token in source_thread_update tracking_item_comment change_request_comment source_ref source_thread_ref result_refs publication_refs final_outcome; do rg -n "$token" fixtures/operational-proposal/story-outbox/expected/public >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15

## Phase 3: Story Helpers And Outbox Semantics

Status: completed
Dependencies: Phase 2

Objective: Implement reusable story rendering and idempotent publication

Changes:
- Extend `thread-story` helpers with the canonical milestone vocabulary.
- Replace the existing thread-story section, feed-entry milestone, and outbox metadata milestone vocabularies with the same canonical v1 ids in one cutover.
- Prefer a single closed `StoryMilestoneId` type for the shared vocabulary; do not keep old type names as compatibility aliases unless they remain the direct canonical exported surface after the cutover.
- Update `outbox.build_feed_entry` and `thread.push_outbox` to consume and validate the same canonical milestone ids.
- Update `tools/outbox/build_feed_entry/fixtures/basic.yaml` in the same milestone cutover so fixture entry ids and `milestone_kind` no longer carry legacy ids.
- Update `tools/outbox/build_pull_request` outbox metadata so `story_milestones` does not preserve a parallel legacy lifecycle namespace; either emit canonical v1 ids or remove the redundant field.
- Update `tests/outbox-build-feed-entry-tool.test.ts` and `tests/thread-push-outbox-tool.test.ts` so fast tests assert the new canonical milestone ids and the original tracking-to-change lifecycle mapping.
- Rewrite the existing legacy milestone assertions in `packages/core/src/knowledge/index.test.ts` onto the canonical v1 vocabulary, including `decision -> triaged`, `merge_gate -> human_gate`, `review -> review_requested`, `signal -> accepted`, `pull_request -> change_request_created`, and `outcome -> final_outcome`; keep the unknown and legacy-id rejection coverage in this same file.
- Preserve idempotent refresh for already-published legacy entries by matching legacy ids to canonical ids during refresh lookup only, then persisting the canonical milestone id on the updated outbox entry.
- Add renderer helpers for concise provider/support-compatible markdown.
- Add core-only story/outbox metadata for create/update/replay that references the existing provider idempotency contract rather than changing it.
- Add a defensive projection-time guard that refuses render/publish when policy requires source-thread publication and the locator is absent.
- Replace freeform milestone acceptance with canonical v1 ids and tests that reject unknown or legacy ids, including removal of the current `| string` fallback from `ThreadStorySectionId` or replacement with an equivalent closed type.
- Add tests for message density, redaction, proposal rendering, idempotency, retry, missing-thread fail-closed behavior, and rendering.
- Update `docs/thread-story-contract.md` with the canonical milestone, idempotency key, content hash, same-key replay, and different-milestone collision semantics alongside the helper changes.
- Rewrite the existing `docs/thread-story-contract.md` `outbox_entry.metadata.milestone_kind` vocabulary list from legacy ids to the canonical v1 ids.

Acceptance:
- [x] `p3_ac1` command - TypeScript compiles.
  - Command: `pnpm typecheck`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20
- [x] `p3_ac2` command - Fast tests pass.
  - Command: `pnpm test:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21
- [x] `p3_ac3` command - Boundary checks pass.
  - Command: `pnpm boundary:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `p3_ac4` command - Canonical milestone ids reject aliases.
  - Command: `for token in rejects_alias_milestone_ids unknown_milestone legacy_signal legacy_decision legacy_spec legacy_build legacy_review legacy_pull_request legacy_merge_gate legacy_outcome; do rg -n "$token" packages/core/src/knowledge tests/outbox-build-feed-entry-tool.test.ts tests/thread-push-outbox-tool.test.ts >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `p3_ac5` command - Missing thread locator fails closed.
  - Command: `for token in "missing_thread_locator" "root_thread_fallback_rejected" "fail_closed"; do rg -n "$token" fixtures/operational-proposal/story-outbox packages/core/src/knowledge >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24
- [x] `p3_ac6` command - Replay key semantics are covered.
  - Command: `for token in "idempotency key" "content hash" "same-key" "different milestones"; do rg -n "$token" fixtures/operational-proposal/story-outbox packages/core/src/knowledge docs/thread-story-contract.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-25
- [x] `p3_ac7` command - One canonical milestone vocabulary spans story, feed, and outbox consumers.
  - Command: `for token in "StoryMilestoneId" "milestone_kind" "outbox.build_feed_entry" "thread.push_outbox" "spec_ready" "build_started" "review_requested"; do rg -n "$token" packages/core/src/knowledge tools/outbox/build_feed_entry tools/thread/push_outbox tests/outbox-build-feed-entry-tool.test.ts tests/thread-push-outbox-tool.test.ts docs/thread-story-contract.md >/dev/null || exit 1; done; if rg -n "\\| string;" packages/core/src/knowledge/thread-story.ts; then exit 1; fi`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26
- [x] `p3_ac8` command - Published legacy entries refresh into canonical entries without duplicate comments.
  - Command: `for token in legacy_published_refresh preserves_comment_id preserves_locator preserves_receipt_ref writes_canonical_milestone_id no_duplicate_comment; do rg -n "$token" tools/outbox/build_feed_entry tests/outbox-build-feed-entry-tool.test.ts fixtures/operational-proposal/story-outbox packages/core/src/knowledge >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-27
- [x] `p3_ac9` command - Core knowledge tests assert canonical tracking-to-change lifecycle mapping.
  - Command: `for token in canonical_index_story_milestone accepted triaged spec_ready build_started review_requested change_request_created human_gate final_outcome rejects_alias_milestone_ids; do rg -n "$token" packages/core/src/knowledge/index.test.ts >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-28
- [x] `p3_ac10` command - Pull-request outbox metadata does not keep legacy story milestones.
  - Command: `rg -n "build_pull_request_canonical_story_milestones" tools/outbox/build_pull_request tests >/dev/null && if rg -n '"signal"|"decision"|"merge_gate"|"outcome"' tools/outbox/build_pull_request/src/index.ts; then exit 1; fi`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29

## Phase 4: Contract Parity And Consumer Notes

Status: completed
Dependencies: Phase 3

Objective: Make story/outbox usable by existing lanes and consuming adapters.

Changes:
- Update thread outbox contracts only if required by new idempotency or observation fields.
- Add docs showing how lanes and proposal skills provide story inputs.
- Keep `docs/thread-story-contract.md` aligned with the Phase 3 helper contract and add consumer-facing examples in the issue/developer inbox docs.
- Add consumer notes for adapters translating core text/markdown into provider blocks, comments, or support notes.
- Verify tracking-to-change existing story remains compatible through the canonical lifecycle mapping, with no legacy ids accepted as runtime input.
- Add surface text snapshots for source-thread updates, tracking-item comments, and change-request comments. These are text/markdown snapshots only; provider block/API payloads remain adapter-owned.

Acceptance:
- [x] `p4_ac1` command - Core-only idempotency boundary remains documented.
  - Command: `for token in "core-only story/outbox metadata" "existing provider idempotency contract" "canonical v1 milestone"; do rg -n "$token" docs/thread-story-contract.md >/dev/null || exit 1; done; rg -n "StoryMilestoneId|FeedStoryMilestoneKind|ThreadStorySectionId|milestone_kind" packages/core/src/knowledge >/dev/null`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-34
- [x] `p4_ac2` command - Contract tests pass for provider parity.
  - Command: `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-35
- [x] `p4_ac3` command - Docs mention public/private story split.
  - Command: `for token in "public story" "private receipt" "artifact refs" "source-thread" "idempotent"; do rg -n "$token" docs/thread-story-contract.md packages/core/src/knowledge >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `p4_ac4` command - Provider text snapshots exist without provider-specific ids.
  - Command: `for token in "source_thread_update" "tracking_item_comment" "change_request_comment"; do rg -n "$token" fixtures/operational-proposal/story-outbox/expected/public >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `p4_ac5` command - Final outcome renderer preserves source/tracking/change continuity.
  - Command: `for token in "source_ref" "source_thread_ref" "tracking_item" "change_request" "final_outcome"; do rg -n "$token" fixtures/operational-proposal/story-outbox/expected/public packages/core/src/knowledge docs/thread-story-contract.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38

## Rollback

- Revert new story helpers, outbox metadata, fixtures, and docs introduced by
  this child.
- If a thread-outbox public contract changed, revert TS/Rust/schema fixtures
  together.
- Leave existing tracking-to-change story behavior intact unless this spec explicitly
  replaced a shared helper.
- No live provider messages are sent by this child, so rollback is local source
  and fixture changes only.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: The two blocker findings from the prior discover review are resolved. tools/outbox/markdown.ts no longer redeclares STORY_MILESTONE_IDS, LEGACY_STORY_MILESTONE_ID_MAP, assertStoryMilestoneId, sanitizePublicMarkdown, summarizePublicHandoffMarkdown, buildFeedStoryOutboxEntry, or renderFeedStoryMarkdown — it now re-exports the canonical helpers from @runxhq/core/knowledge and keeps only the domain-specific renderIssueToPrReviewerMarkdown. tools/outbox/build_feed_entry/src/index.ts imports buildFeedStoryOutboxEntry and renderFeedStoryMarkdown from @runxhq/core/knowledge and wires canonicalStoryEntryIdForRefresh + storyMilestoneRefreshesPublishedEntry into latestTrustedStoryOutbox, so the dead canonical refresh helpers are now consumed at the wired call site. Canonical milestone vocabulary is consistent across thread-story, feed-entry, outbox, build_pull_request story_milestones, push_outbox normalizeStoryMilestones, fixtures, docs, and tests. Legacy ids reject at assertStoryMilestoneId with the canonical mapping in the error. Legacy-published refresh (merge_gate→human_gate and merge_gate→final_outcome) preserves locator/comment_id/outbox_receipt_id and rewrites the slug to the canonical id. Fail-closed source-thread guard rejects missing locator and non-fail_closed missing_behavior. Two low/non-blocking items remain: tools/public_markdown.mjs still uses an unanchored path-redaction regex that diverges from the canonical sanitizer in packages/core/src/knowledge/thread-story.ts (the third copy of the sanitizer was not consolidated when the markdown.ts duplicate was removed), and docs/thread-story-contract.md:32 documents outbox_entry.metadata.schema_version as runx.outbox-entry.message.v1 while buildFeedStoryOutboxEntry actually emits runx.outbox-entry.feed-entry.v1. Neither blocks completion.

Attack log:
- `tools/outbox/markdown.ts canonical-helper duplication (prior blocker #1)`: Confirm the duplicate STORY_MILESTONE_IDS / LEGACY_STORY_MILESTONE_ID_MAP / sanitizePublicMarkdown / summarizePublicHandoffMarkdown / renderFeedStoryMarkdown / buildFeedStoryOutboxEntry / assertStoryMilestoneId definitions were removed and that build_feed_entry consumes the canonical surface from @runxhq/core/knowledge. -> clean (tools/outbox/markdown.ts now imports sanitizePublicMarkdown/summarizePublicHandoffMarkdown from @runxhq/core/knowledge and re-exports buildFeedStoryOutboxEntry/renderFeedStoryMarkdown/sanitizePublicMarkdown/summarizePublicHandoffMarkdown from the same canonical surface; only renderIssueToPrReviewerMarkdown (domain-specific) remains defined locally. tools/outbox/build_feed_entry/src/index.ts:9-15 imports the canonical helpers directly from @runxhq/core/knowledge.)
- `packages/core/src/knowledge/file-thread.ts dead canonical refresh helpers (prior blocker #2)`: Verify storyMilestoneRefreshesPublishedEntry and canonicalStoryEntryIdForRefresh are now consumed by the wired tool instead of dead-exported. -> clean (tools/outbox/build_feed_entry/src/index.ts:12-15 imports both helpers and uses them inside latestTrustedStoryOutbox (lines 326-345) to match published legacy outbox entries and rewrite the entry_id slug to the canonical milestone id. The legacy merge_gate→human_gate and merge_gate→final_outcome refresh transitions are handled centrally in packages/core/src/knowledge/file-thread.ts.)
- `tools/public_markdown.mjs sanitizePublicMarkdown divergence (prior low #3)`: Diff the remaining sanitizePublicMarkdown implementations to spot regex drift after the markdown.ts consolidation. -> finding (Three copies became two. tools/outbox/markdown.ts no longer has its own copy, but tools/public_markdown.mjs:13 still uses an unanchored path-redaction regex that diverges from the canonical anchored regex in packages/core/src/knowledge/thread-story.ts:192. Carried forward as story-outbox-sanitize-mjs-divergence-residual.)
- `Canonical milestone vocabulary single source of truth`: Walk every wired consumer (tools/outbox/build_feed_entry, tools/outbox/build_pull_request, tools/thread/push_outbox, tools/outbox/markdown.ts, packages/core/src/knowledge/* and the consumer tests) and confirm they share the canonical 18-id vocabulary and reject legacy ids at runtime. -> clean (STORY_MILESTONE_IDS is defined once in thread-story.ts and re-exported via index.ts. build_pull_request uses `build_pull_request_canonical_story_milestones = [...ISSUE_TO_PR_STORY_MILESTONES]`. push_outbox runs assertStoryMilestoneId via validateStoryMessageMilestone (line 559-569) and normalizeStoryMilestones (line 551-557). LEGACY_STORY_MILESTONE_ID_MAP entries throw with the canonical replacement in the error message; the tests in packages/core/src/knowledge/index.test.ts iterate every legacy alias and assert the throw.)
- `Idempotency key behavior for same vs different milestones`: Verify buildStoryOutboxIdempotencyMetadata produces stable keys for same-milestone replays and distinct keys for different-milestone calls with otherwise identical inputs. -> clean (packages/core/src/knowledge/outbox.ts:27-49 hashes a fixed-key keyMaterial (source_id, provider, source_thread_ref, workflow_id, lane_id, milestone_id, target_ref, proposal_id, content_hash) via hashStable. The test in packages/core/src/knowledge/index.test.ts:112-148 asserts sameKey.key === first.key for identical inputs and differentMilestones.key !== first.key when only milestone_id changes. Replay metadata `same_key: 'update_or_reuse'`, `different_milestones: 'distinct_entries'` reflects the contract.)
- `Legacy-published refresh preserves comment_id, locator, receipt_ref`: Trace what happens when a thread already carries a published `:merge_gate` outbox entry and build_feed_entry runs after the cutover. -> clean (tools/outbox/build_feed_entry/src/index.ts:316-349 latestTrustedStoryOutbox uses canonicalStoryEntryIdForRefresh to map the existing entry's id slug (`message:<task>:merge_gate`) to the requested canonical slug (`message:<task>:human_gate` or `message:<task>:final_outcome`) before comparing. The test at tests/outbox-build-feed-entry-tool.test.ts:353-417 (`legacy_published_refresh preserves_comment_id preserves_locator preserves_receipt_ref writes_canonical_milestone_id no_duplicate_comment`) and tests/outbox-build-feed-entry-tool.test.ts:419-479 (file adapter) both assert canonical entry_id with preserved locator/comment_id/outbox_receipt_id.)
- `Fail-closed source-thread guard at the projection layer`: Confirm assertSourceThreadPublicationAllowed refuses publication when requires_source_thread_publication is true and either missing_behavior is not fail_closed or sourceThreadRef is absent. -> clean (packages/core/src/knowledge/thread-story.ts:123-140 returns sanitized ref when not required, throws `source_thread.missing_behavior must be fail_closed` when missing_behavior diverges, and throws `missing_thread_locator: root_thread_fallback_rejected` when sourceThreadRef is absent. tools/outbox/build_feed_entry/src/index.ts:80-82 also throws `source thread locator is required` before reaching the helper. tools/thread/push_outbox/src/index.ts:611-637 enforces the same constraint at the push site. The unit test `missing_thread_locator root_thread_fallback_rejected fail_closed` covers it; the tool test `fails closed when no source thread locator is available` exercises the integrated path.)
- `Ambient drift containment`: Check that ambient drift files (operational-proposal contract, composition-paths fixture, outbox markdown re-exports) do not silently bypass the canonical surface. -> clean (operational-proposal* changes are scoped to proposal authority, not the milestone vocabulary. tools/outbox/markdown.ts is included in scope and now re-exports the canonical surface, so the ambient outbox/markdown.ts change supports rather than bypasses the canonical helpers. tests/outbox-build-pull-request-tool.test.ts asserts canonical story_milestones.)

Findings:
- [low/non-blocking] `story-outbox-sanitize-mjs-divergence-residual` tools/public_markdown.mjs still carries a divergent unanchored path-redaction regex relative to the canonical sanitizePublicMarkdown in packages/core/src/knowledge/thread-story.ts; the previous review's triple-sanitizer divergence is reduced from three copies to two but not fully resolved.
  - Location: `tools/public_markdown.mjs:13`
  - Evidence: tools/public_markdown.mjs:13 uses /(?:\/Users|\/home|\/var|\/private|\/tmp)\/[^\s`)]+/g, '[local-path]' (no leading-delimiter anchor, replaces with bare '[local-path]'). packages/core/src/knowledge/thread-story.ts:192 uses /(^|[\s=("'`])(?:\/Users|\/home|\/var|\/private|\/tmp)\/[^\s`)]+/g, '$1[local-path]' (anchored, preserves leading delimiter). The .mjs file is imported by tools/thread/github_adapter.mjs (lines 5, 581, 715) which is consumed by tools/thread/push_outbox/src/index.ts (pushGitHubMessage / pushGitHubPullRequest body sanitization). The fix consolidated the duplicate that previously lived in tools/outbox/markdown.ts (which now re-exports from @runxhq/core/knowledge), but the .mjs copy was not consolidated, so identical inputs still produce different sanitized outputs depending on which call site rendered them.
  - Impact: Public-safety contracts the spec depends on (no local paths, no token leakage) are not uniformly enforced. The .mjs version is broader (it will redact mid-string path-prefix matches that the anchored core version skips) so it is leak-safer rather than leak-prone, but adding new redaction rules in @runxhq/core/knowledge will not propagate to the .mjs path that GitHub provider messages take. Carried forward from the prior discover review's story-outbox-triple-sanitize-divergence finding.
  - Validation: After consolidation, `rg -n "sanitizePublicMarkdown\b" tools/public_markdown.mjs` should show only the import/re-export, and `node -e "const a=require('./tools/public_markdown.mjs').sanitizePublicMarkdown('foo /Users/x/y'); const b=require('./packages/core/src/knowledge/thread-story.ts').sanitizePublicMarkdown('foo /Users/x/y'); console.log(a===b);"` (or equivalent test) should print true.
- [low/non-blocking] `story-outbox-doc-schema-version-mismatch` docs/thread-story-contract.md documents outbox_entry.metadata.schema_version as runx.outbox-entry.message.v1, but buildFeedStoryOutboxEntry (the helper this contract describes) emits runx.outbox-entry.feed-entry.v1.
  - Location: `docs/thread-story-contract.md:32`
  - Evidence: docs/thread-story-contract.md:32 lists `outbox_entry.metadata.schema_version`: `runx.outbox-entry.message.v1` in the section describing what `buildFeedStoryOutboxEntry` produces. packages/core/src/knowledge/feed-entry.ts:72 writes `schema_version: "runx.outbox-entry.feed-entry.v1"` and tools/outbox/build_feed_entry/src/index.ts:339 keys the trusted-state preservation off the same `runx.outbox-entry.feed-entry.v1` literal. Tests at tests/outbox-build-feed-entry-tool.test.ts:105, :312, :376, :442 and tests/thread-push-outbox-tool.test.ts:66, :302, :318 all assert the feed-entry schema. The `runx.outbox-entry.message.v1` literal exists elsewhere (tools/thread/github_adapter.mjs:780, :944; scripts/dogfood-github-issue-to-pr.mjs:1625) for messages built by the GitHub adapter itself, not by the canonical feed-entry helper.
  - Impact: Readers of docs/thread-story-contract.md will expect the feed-entry helper to emit `runx.outbox-entry.message.v1` and may write consumers, fixtures, or schema validators against the wrong identifier. Phase 4 acceptance commands grep the doc for vocabulary terms but do not pin the schema_version literal, so the mismatch slipped through.
  - Validation: After the fix, `rg -n "runx.outbox-entry.feed-entry.v1" docs/thread-story-contract.md` should match the section that describes buildFeedStoryOutboxEntry, and no occurrence of `runx.outbox-entry.message.v1` should remain in that section unless it is explicitly labeled as the adapter-direct schema.

## Self Eval

- Pending implementation. Target bar: public messages are concise but useful,
  proposal-aware, source-thread-safe, and backed by private evidence artifacts.

## Deviations

- Previous story draft listed fixed domain milestone ids. This version uses
  generic proposal milestones and derives friendly labels from `proposal_kind`.

## Metadata

- created_by: scafld
- parent_spec: runx-operational-intelligence-action-layer-v1

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-27T17:53:57Z
Ended: 2026-05-27T17:53:57Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Spec is mostly coherent: the existing thread-outbox-provider contract already exposes the `idempotency.key`+`content_hash` shape the spec defers to, the listed code/doc touchpoints exist, and the boundary scripts referenced are real. Two issues block approval. (1) The proposed canonical v1 milestone id set collides with three existing vocabularies — `FeedStoryMilestoneKind` in `packages/core/src/knowledge/feed-entry.ts:16-24`, `ThreadStorySectionId` in `packages/core/src/knowledge/thread-story.ts:7-13`, and the documented outbox metadata `milestone_kind` in `docs/thread-story-contract.md:31,41-50` — but touchpoints exclude `feed-entry.ts`, `tools/outbox/build_feed_entry/`, and `tools/thread/push_outbox/`. Without an explicit ownership statement the "hard cut" rule is ambiguous and may produce two parallel milestone universes. (2) Phase 3 acceptance `p3_ac6` greps `docs/thread-story-contract.md` for "idempotency key", "content hash", "same-key", and "different milestones", but Phase 3's Changes list does not include doc edits — those land in Phase 4. The gate cannot pass at the time it's evaluated unless doc work is moved into Phase 3. Additional advisory issues: fixture leak markers under `inputs/private/` use patterns (`xox[baprs]-`, `SENTRY_AUTH_TOKEN`, `BEGIN .*PRIVATE KEY`) that can trigger GitHub secret-scanning push protection; `p4_ac1` and `p2_ac1` are weak gates that pass trivially from the spec file or any single fixture; and `docs/operational-intelligence.md` is in Touchpoints/Scope but no acceptance command requires it and the file does not exist today.

Checks:
- path audit
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:7
  - Result: passed
  - Evidence: All declared TS touchpoints (knowledge/thread-story.ts, outbox.ts, file-thread.ts, index.ts, index.test.ts), contracts touchpoints (thread-outbox-provider.ts, run-summary.ts, artifact.ts), Rust touchpoints (crates/runx-contracts/src/thread_outbox_provider.rs, crates/runx-contracts/tests/thread_outbox_provider_fixtures.rs), and docs (thread-story-contract.md, issue-to-pr.md, developer-issue-inbox.md) exist. `fixtures/threads/` exists with two real fixtures. Exception: docs/operational-intelligence.md is listed in Scope/Touchpoints but does not exist and no acceptance command requires it — captured as a separate advisory issue.
- command audit
  - Grounded in: code:package.json:35
  - Result: passed
  - Evidence: `pnpm test:fast` (line 35), `pnpm boundary:check` (line 37), and `pnpm typecheck` are real package.json scripts; `scripts/check-boundaries.mjs` exists. `pnpm exec vitest run --config vitest.fast.config.ts` and `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features` are well-formed. `scafld validate` is the standard harden gate.
- scope/migration audit
  - Grounded in: code:packages/core/src/knowledge/feed-entry.ts:16
  - Result: failed
  - Evidence: Spec proposes a canonical v1 milestone id set (accepted/hydrated/triaged/.../final_outcome/no_action/monitor) and forbids legacy ids `pull_request`, `merge_gate`, `outcome`, `completion_update`, `dev_escalation`, `outreach_recommendation`. But `FeedStoryMilestoneKind` in feed-entry.ts:16-24 still encodes the legacy vocabulary (`pull_request`/`merge_gate`/`outcome`), `outbox_entry.metadata.milestone_kind` is exported with those values via feed-entry.ts:189, and `ThreadStorySectionId` in thread-story.ts:7-13 hard-codes `pr_created`/`human_merge_gate`/`completion_update` plus `| string` fallback. Touchpoints exclude feed-entry.ts and tools/outbox/build_feed_entry/. Hard-cut intent and migration scope therefore disagree.
- acceptance timing audit
  - Grounded in: spec_gap:phase3.changes
  - Result: failed
  - Evidence: Phase 3 `p3_ac6` requires `docs/thread-story-contract.md` to contain `idempotency key`, `content hash`, `same-key`, and `different milestones`. The existing doc does not contain those terms, and Phase 3's Changes list mentions code/test work only — doc edits are owned by Phase 4 (`Add docs showing how lanes and proposal skills provide story inputs`). Unless Phase 3 explicitly includes those doc edits, p3_ac6 cannot pass at the moment `scafld build` exits Phase 3.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback section is credible: no live provider sends are issued by this child, so reverting helpers + fixtures + docs (and Rust/TS contract fixtures together if touched) is mechanical. Existing issue-to-PR story is documented as preserved unless explicitly replaced.
- design challenge
  - Grounded in: code:packages/contracts/src/schemas/thread-outbox-provider.ts:143
  - Result: passed
  - Evidence: Core architectural call is sound: the provider contract already owns the public idempotency key/content_hash shape (thread-outbox-provider.ts:143-164) and supplies observation status enums, so layering core-only outbox metadata that *references* that contract without re-issuing it respects the trusted-kernel boundary and the project's `no_legacy_code`/`public_api_stable` invariants. The fail-closed source-thread guard at the projection layer is the right place for that policy. The remaining concerns are scope/migration plumbing rather than architectural drift.

Issues:
- [high/blocks approval] `milestone_vocabulary_scope` scope_migration - Canonical v1 milestone id set collides with existing FeedStoryMilestoneKind, ThreadStorySectionId, and outbox metadata milestone_kind vocabularies — touchpoints do not cover all collision sites.
  - Status: open
  - Grounded in: code:packages/core/src/knowledge/feed-entry.ts:16
  - Evidence: Spec lists `accepted/hydrated/triaged/reply_drafted/.../final_outcome/no_action/monitor` as the v1 milestone ids and says legacy/friendly ids `pull_request`, `merge_gate`, `outcome`, `completion_update`, `dev_escalation`, `outreach_recommendation`, `human gate` must NOT be accepted as data ids. But feed-entry.ts:16-24 defines `FeedStoryMilestoneKind = signal|decision|spec|build|review|pull_request|merge_gate|outcome`, feed-entry.ts:189 writes `milestone_kind: milestone.kind` into outbox metadata, thread-story.ts:7-13 declares `ThreadStorySectionId` with `pr_created|human_merge_gate|completion_update|... | string`, and docs/thread-story-contract.md:31,41-50 documents `outbox_entry.metadata.milestone_kind` with the legacy `pull_request|merge_gate|outcome` set. Touchpoints exclude feed-entry.ts, tools/outbox/build_feed_entry/, and tools/thread/push_outbox/, so the 'hard cut' is undefined for those surfaces.
  - Recommendation: Decide explicitly whether the canonical v1 milestone id set replaces (a) `ThreadStorySectionId` only, (b) `ThreadStorySectionId` + `FeedStoryMilestoneKind` + outbox metadata `milestone_kind`, or (c) a new third namespace. Then either add the additional files to Touchpoints/Scope with a coordinated rename and acceptance gate, or carve out a narrow statement that feed-entry milestone_kind is a separate vocabulary that this spec does not touch. Update the 'legacy/friendly labels' list accordingly so future readers can see where each name still lives.
  - Question: Does the v1 milestone id set replace ThreadStorySectionId only, or also FeedStoryMilestoneKind and outbox metadata milestone_kind?
  - Recommended answer: Treat it as a single canonical vocabulary used across thread-story sections and outbox milestone_kind. That means feed-entry.ts and the build_feed_entry/push_outbox tools must be added to Touchpoints and Phase 3 must include the coordinated rename with a test that rejects legacy ids on every surface.
  - If unanswered: Default to scoping the v1 id set to thread-story.ts only, document that feed-entry.ts retains its own enum, and remove the broad 'legacy or friendly labels must not be accepted as data ids' wording.
- [high/blocks approval] `p3_ac6_docs_timing` acceptance_timing - Phase 3 acceptance p3_ac6 requires doc content that is only authored in Phase 4.
  - Status: open
  - Grounded in: spec_gap:phase3.changes
  - Evidence: p3_ac6 greps `docs/thread-story-contract.md` for `idempotency key`, `content hash`, `same-key`, `different milestones`. The current docs/thread-story-contract.md does not contain those terms (verified by reading the file). Phase 3 Changes describe helper/test work, not documentation; Phase 4 Changes own the doc additions (`Add docs showing how lanes and proposal skills provide story inputs`).
  - Recommendation: Either (a) move the docs/thread-story-contract.md updates that introduce the idempotency-key/content-hash/replay vocabulary into Phase 3 Changes explicitly, or (b) drop docs/thread-story-contract.md from the p3_ac6 path list and reassert that check inside Phase 4 (e.g., add it to p4_ac3 or a new p4_ac6).
  - Question: Should the docs/thread-story-contract.md updates land in Phase 3 alongside the helper changes, or should p3_ac6 not reference docs at all?
  - Recommended answer: Move the docs/thread-story-contract.md edits into Phase 3 Changes — the idempotency vocabulary is born with the helpers, and splitting it across two phases makes the gate non-runnable mid-build.
  - If unanswered: Default to drop docs/thread-story-contract.md from p3_ac6's path list and add an equivalent docs-only check to Phase 4.
- [medium/advisory] `fixture_secret_scanner_risk` operational_risk - Private leak-marker fixtures may trigger GitHub secret scanning push protection.
  - Status: open
  - Grounded in: spec_gap:phase2.changes
  - Evidence: p2_ac2 requires fixtures under `fixtures/operational-proposal/story-outbox/inputs/private/` to literally match patterns including `xox[baprs]-`, `SENTRY_AUTH_TOKEN`, and `BEGIN .*PRIVATE KEY`. A grep across the OSS repo (`xox[baprs]-`, `SENTRY_AUTH_TOKEN`, `BEGIN .*PRIVATE KEY`) returned no existing fixtures using those shapes, suggesting prior intent was to avoid such patterns. GitHub's default secret scanning detects Slack tokens (`xoxb-`/`xoxp-`/`xoxa-`/etc.) and PEM private-key headers and can block pushes.
  - Recommendation: Use unambiguously-fake but pattern-matching values (e.g., `xoxb-FIXTURE-NOT-A-REAL-TOKEN-0000000000000000`, `SENTRY_AUTH_TOKEN=FIXTURE_NOT_A_REAL_TOKEN`, and a `BEGIN FAKE PRIVATE KEY` marker that does not use the `RSA|EC|OPENSSH|PGP` standard subtypes). Document the convention in Phase 2 Changes so reviewers know the fixtures are deliberately leak-shaped but obviously synthetic.
- [low/advisory] `weak_acceptance_gates` weak_gate - Several acceptance commands pass trivially without exercising every intended target.
  - Status: open
  - Grounded in: spec_gap:phase4.acceptance.p4_ac1
  - Evidence: p4_ac1 runs `rg -n PATTERN <spec_file> docs/thread-story-contract.md packages/core/src/knowledge`. `rg` exits 0 if ANY match is found across ANY of the listed paths, so the spec file alone (which contains all three patterns by design) satisfies the gate even if neither the docs nor the code mention them. p2_ac1 loops tokens through `fixtures/operational-proposal/story-outbox` without restricting to `expected/public/`, so an input fixture containing the token id satisfies the gate.
  - Recommendation: Strengthen p4_ac1 by running one `rg` per target path and `|| exit 1` for each (as p3_ac6 already does), and restrict p2_ac1 to `fixtures/operational-proposal/story-outbox/expected/public` to match the intent of 'every generic milestone family appears in expected public output'.
- [low/advisory] `operational_intelligence_doc` spec_consistency - docs/operational-intelligence.md is listed as a touchpoint but does not exist and no acceptance command requires it.
  - Status: open
  - Grounded in: spec_gap:scope.docs
  - Evidence: Touchpoints and Scope both reference `docs/operational-intelligence.md`. The repo does not contain that file (verified via `rg operational-intelligence` returning only spec/archive matches). None of p1_ac1..p4_ac5 reference it, so it is silently in scope without being verifiable.
  - Recommendation: Either drop docs/operational-intelligence.md from Touchpoints/Scope (the parent spec `runx-operational-intelligence-action-layer-v1` may own that doc) or add a Phase 4 acceptance check that requires `test -f docs/operational-intelligence.md` and a topical grep.

### round-2

Status: needs_revision
Started: 2026-05-27T18:05:06Z
Ended: 2026-05-27T18:05:06Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-1 issues are largely answered in wording (Touchpoints now include feed-entry.ts, build_feed_entry/, and push_outbox/; Phase 3 owns docs/thread-story-contract.md; fixtures use safe synthetic markers; p4_ac1/p2_ac1 are tightened; operational-intelligence.md was dropped). Two new blockers surface from the round-2 read. (1) The "canonical v1" hard cut replaces lifecycle-shaped ids (signal/decision/spec/build/review/pull_request/merge_gate/outcome) with outcome-shaped ids (accepted/hydrated/triaged/reply_drafted/.../final_outcome). That is a model change, not a rename — the existing build_feed_entry currently emits one milestone per lifecycle gate, and none of those ids survive in the new set, yet Phase 4 asserts "issue-to-PR existing story remains compatible." The spec does not say how the 8-milestone lifecycle render is reshaped under the new vocabulary, nor what `kind: "signal"|"decision"|"spec"|"build"|"review"` should become after the cutover. (2) `pnpm test:fast` (p3_ac2) will fail unless `tests/outbox-build-feed-entry-tool.test.ts` and `tests/thread-push-outbox-tool.test.ts` are also updated — both files hardcode legacy milestone ids (`kind: "merge_gate"|"outcome"`, `entry_id: "message:fixture-task:merge_gate"`) but they are not in spec Touchpoints. Two advisory issues: p4_ac4 requires `slack_thread_update`/`github_issue_comment`/`github_pr_comment` literals to appear inside `packages/core/src/knowledge`, which slightly contradicts the stated provider-neutrality assumption; and p4_ac1's grep tokens ("core-only story/outbox metadata", "existing provider idempotency contract", "canonical v1 milestone") must appear as comments in core source — coupling source-comment phrasing to docs is fragile coverage.

Checks:
- path audit
  - Grounded in: code:packages/core/src/knowledge/feed-entry.ts:16
  - Result: passed
  - Evidence: All declared TS touchpoints (thread-story.ts, feed-entry.ts, outbox.ts, file-thread.ts, index.ts, index.test.ts) exist. tools/outbox/build_feed_entry/ and tools/thread/push_outbox/ exist with src/manifest/fixtures. Contracts (thread-outbox-provider.ts, run-summary.ts, artifact.ts) and Rust contracts (crates/runx-contracts/src/thread_outbox_provider.rs, tests/thread_outbox_provider_fixtures.rs) exist. Docs (thread-story-contract.md, issue-to-pr.md, developer-issue-inbox.md) exist. fixtures/threads/ has issue-to-pr-file-thread.json and issue-to-pr-github-thread.json. round-1's docs/operational-intelligence.md path has been removed from Touchpoints/Scope.
- command audit
  - Grounded in: code:tools/thread/push_outbox/src/index.ts:80
  - Result: passed
  - Evidence: pnpm typecheck, pnpm test:fast, pnpm boundary:check are real package.json scripts. scafld validate is the standard harden gate. p3_ac4 grep with `--glob '*.test.ts' --glob '*.ts'` is well-formed ripgrep syntax. p1_ac1 substrings 'provider contract already owns public idempotency', 'proposal_kind', 'source-thread locator' are present in the current spec body (lines 142, 64, 78). The existing push_outbox tool already enforces fail_closed when source_thread.required is set (lines 80-82, 587-613), so the Phase 3 defensive projection-time guard maps onto a real surface.
- scope/migration audit
  - Grounded in: code:tests/outbox-build-feed-entry-tool.test.ts:90
  - Result: failed
  - Evidence: Round-1 milestone_vocabulary_scope was partially resolved at the file-add level (feed-entry.ts, build_feed_entry/, push_outbox/ are now in Touchpoints), but two cutover sites remain uncovered. (a) tests/outbox-build-feed-entry-tool.test.ts:90-118 and tests/thread-push-outbox-tool.test.ts:61,296 assert legacy milestone ids ('decision'/'spec'/'build'/'review'/'pull_request'/'merge_gate'/'outcome' and entry_id 'message:fixture-task:merge_gate'); they are not in Touchpoints but p3_ac2 (pnpm test:fast) executes them, so the cutover will break the suite unless those files are explicitly in scope. (b) The new canonical v1 ids are outcome-shaped (accepted/hydrated/triaged/.../final_outcome) and contain none of signal/decision/spec/build/review; the current build_feed_entry consumer emits one milestone per lifecycle gate (tools/outbox/build_feed_entry/src/index.ts:122-208). The spec does not state how that lifecycle output is reshaped under the new vocabulary, yet Phase 4 promises 'issue-to-PR existing story remains compatible'.
- acceptance timing audit
  - Grounded in: spec_gap:phase4.acceptance.p4_ac4
  - Result: failed
  - Evidence: p4_ac4 requires 'slack_thread_update', 'github_issue_comment', 'github_pr_comment' to be findable in both fixtures/operational-proposal/story-outbox/expected/public and packages/core/src/knowledge. Those literals are not in core today (verified by reading thread-story.ts/feed-entry.ts), and Phase 4 Changes describe 'Add surface text snapshots' rather than introducing core-level identifiers — Phase 3 owns the renderer helpers. The gate is timed correctly (Phase 4), but the spec should clarify whether 'in core/knowledge' means 'as exported snapshot identifier constants' or 'referenced inside renderer comments'. Without that, an implementer cannot tell when p4_ac4 should pass. p4_ac1's tokens must also appear in core/knowledge source, which only works if Phase 4 (or Phase 3) adds them as code comments — not stated.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: No live provider sends are issued by this child, so reverting helpers, fixtures, outbox metadata, and docs is mechanical. Rollback explicitly calls out reverting TS/Rust contract fixtures together if the provider contract changes, and explicitly preserves the existing issue-to-PR story behavior unless this spec replaced a shared helper. The remaining risk is operational: if the milestone-id hard cut lands and a downstream consumer (skill, fixture, or external lane) hardcodes the legacy ids, rollback restores the helper but the consumer must be re-shipped or a tag-cut release reverted. That's a Phase-3-implementation concern rather than a rollback-correctness concern.
- design challenge
  - Grounded in: code:packages/contracts/src/schemas/thread-outbox-provider.ts:143
  - Result: passed
  - Evidence: Core architectural call remains sound: the public provider contract already owns idempotency.key + content_hash (thread-outbox-provider.ts:143-164), so layering core-only outbox metadata that references that contract preserves the trusted-kernel boundary and the project's public_api_stable invariant. Splitting policy-decides-required-source-thread from helper-refuses-on-inconsistency is the right separation; tools/thread/push_outbox already fails closed on missing source_thread (src/index.ts:80-82, 587-613) so the projection-time guard has a coherent home. Provider-neutral text/markdown renderers with adapter-owned blocks/payloads respects the @runxhq/core domain boundary. The architectural concerns are (a) whether collapsing lifecycle milestones into outcome-state milestones is the right design choice — it may well be, but the spec must justify it rather than smuggle it in as a 'cutover', and (b) provider-named snapshot identifiers (slack_thread_update/github_issue_comment/github_pr_comment) inside core sit in tension with the stated provider-neutral assumption; this is a naming choice worth a one-line rationale.

Issues:
- [high/blocks approval] `milestone_vocabulary_semantic_shift` scope_migration - Canonical v1 milestone set is outcome-shaped and contains none of the lifecycle ids currently emitted by build_feed_entry; spec does not say how the consumer is reshaped, yet Phase 4 promises issue-to-PR story remains compatible.
  - Status: open
  - Grounded in: code:tools/outbox/build_feed_entry/src/index.ts:122
  - Evidence: tools/outbox/build_feed_entry/src/index.ts:122-208 currently emits one milestone per lifecycle gate ({kind: 'signal'},{kind:'decision'},{kind:'spec'},{kind:'build'},{kind:'review'},{kind:'pull_request'},{kind:'merge_gate'},{kind:'outcome'}). The spec's canonical v1 set is {accepted, hydrated, triaged, reply_drafted, ask_for_info, proposal_ready, escalation_proposed, issue_created, pull_request_created, review_fixup, human_gate, outcome_observed, final_outcome, no_action, monitor} — five of the current eight ids (signal/decision/spec/build/review) disappear entirely. The spec mandates rejection of 'unknown or legacy milestone ids' (Objectives) and a hard-cut cutover (Assumptions), but Phase 4 also requires 'issue-to-PR existing story remains compatible' (Changes). Either build_feed_entry must collapse to a single outcome-state milestone, or the lifecycle ids must be retained alongside the new set, or build_feed_entry must be rewritten to map state→outcome — the spec does not pick one.
  - Recommendation: Add an explicit statement to Phase 3 Changes saying which of (a) build_feed_entry now emits a single outcome milestone (with the lifecycle gates captured as `details:` bullets or evidence refs), (b) the canonical v1 set is additive and signal/decision/spec/build/review are retained as legacy-but-still-valid ids, or (c) build_feed_entry is rewritten to project each lifecycle gate onto one of the new outcome ids (e.g., review→review_fixup or proposal_ready). Update Phase 3 Changes and p3_ac4 accordingly, and clarify what 'issue-to-PR existing story remains compatible' means after the cutover.
  - Question: Under the canonical v1 hard cut, how does build_feed_entry's existing lifecycle output (signal/decision/spec/build/review/pull_request/merge_gate/outcome) survive? Single collapsed outcome milestone, additive vocabulary, or per-gate mapping?
  - Recommended answer: Collapse build_feed_entry to one outcome-state milestone (pull_request_created or final_outcome depending on observed provider state), and surface the prior lifecycle gates as `details:` bullets/evidence refs so the body markdown still narrates the gates without re-introducing lifecycle ids as data. Add a Phase 3 test that constructs a build_feed_entry call with the old fixture and asserts the new single-milestone shape.
  - If unanswered: Default to a single collapsed outcome milestone with lifecycle gates demoted to details/evidence; document that signal/decision/spec/build/review are no longer milestone ids and update the rejected-aliases list to include them explicitly.
- [high/blocks approval] `consumer_tests_outside_touchpoints` scope_migration - p3_ac2 (pnpm test:fast) will fail because tests/outbox-build-feed-entry-tool.test.ts and tests/thread-push-outbox-tool.test.ts assert legacy milestone ids and are not in Touchpoints.
  - Status: open
  - Grounded in: code:tests/outbox-build-feed-entry-tool.test.ts:90
  - Evidence: tests/outbox-build-feed-entry-tool.test.ts:90-118 asserts `kind: 'decision'|'spec'|'build'|'review'|'pull_request'|'merge_gate'|'outcome'` and `entry_id: 'message:fixture-task:merge_gate'`, plus `metadata.milestone_kind: 'merge_gate'` at lines 108,196,247,314,377,411,443,475. tests/thread-push-outbox-tool.test.ts:61,296 also references `entry_id: 'message:fixture-task:merge_gate'`. tools/outbox/build_feed_entry/fixtures/basic.yaml:84 also pins `milestone_kind: merge_gate`. Spec Touchpoints list tools/outbox/build_feed_entry/** (which covers the yaml fixture) but do not list tests/outbox-build-feed-entry-tool.test.ts or tests/thread-push-outbox-tool.test.ts. p3_ac2 requires pnpm test:fast to pass.
  - Recommendation: Add tests/outbox-build-feed-entry-tool.test.ts and tests/thread-push-outbox-tool.test.ts to Touchpoints/Scope, and add a Phase 3 Changes bullet stating that those test fixtures and assertions are rewritten as part of the milestone cutover. Alternatively, if the existing-issue-to-PR-story-remains-compatible answer (see the previous issue) is to keep lifecycle ids as a legacy-additive set, then call that out so the tests do not need to change.
  - Question: Should tests/outbox-build-feed-entry-tool.test.ts and tests/thread-push-outbox-tool.test.ts be added to Touchpoints with a coordinated rewrite under the milestone cutover?
  - Recommended answer: Yes — add both test files to Touchpoints, and add a Phase 3 Changes bullet committing to rewrite the milestone-id assertions and entry_id patterns in those tests. The fixture `tools/outbox/build_feed_entry/fixtures/basic.yaml` is already covered by tools/outbox/build_feed_entry/** and must be updated in the same cutover.
  - If unanswered: Default to adding both test files to Touchpoints with explicit cutover commitment, and update the basic.yaml fixture milestone_kind in the same Phase 3 commit.
- [medium/advisory] `core_provider_named_snapshots` design_question - p4_ac4 requires provider-named literals (slack_thread_update, github_issue_comment, github_pr_comment) inside packages/core/src/knowledge, which is in tension with the provider-neutral renderer assumption.
  - Status: open
  - Grounded in: spec_gap:phase4.acceptance.p4_ac4
  - Evidence: Assumptions state 'Renderer output should be provider-neutral text/markdown. Consuming adapters can translate to Slack blocks, GitHub comments, or support notes.' But p4_ac4 (line 380 of the spec) requires `slack_thread_update`, `github_issue_comment`, `github_pr_comment` to be greppable in `packages/core/src/knowledge`. The current docs/thread-story-contract.md non-goals section explicitly says 'This contract does not admit Slack, Sentry, or support-channel messages.' (line 89). Coupling core to provider-named snapshot identifiers walks back that boundary without a stated rationale.
  - Recommendation: Either (a) move the Slack/GitHub snapshot identifiers into the fixture layer only and drop `packages/core/src/knowledge` from the p4_ac4 grep path list, or (b) explicitly justify in Phase 4 Changes that core exports snapshot kind constants for provider-neutral text/markdown (e.g., as a `RenderTargetSnapshotKind` type whose values happen to be provider-named because the renderer produces provider-shaped markdown — not because core calls any Slack/GitHub API). Update docs/thread-story-contract.md non-goals so the boundary is internally consistent.
  - Question: Are `slack_thread_update`/`github_issue_comment`/`github_pr_comment` snapshot identifier strings exported from core, or only used in fixture filenames?
  - Recommended answer: Keep them fixture-only. Drop `packages/core/src/knowledge` from p4_ac4's grep paths, and document in docs/thread-story-contract.md that the Slack-/GitHub-friendly renderers in core are typed by markdown flavor (e.g., `compact|reviewer|status`) rather than by provider name. Fixtures then label which renderer flavor maps to which provider snapshot.
- [low/advisory] `core_comment_string_coverage` weak_gate - p4_ac1 grep tokens ('core-only story/outbox metadata', 'existing provider idempotency contract', 'canonical v1 milestone') must appear inside packages/core/src/knowledge — i.e., as code comments — which couples docs phrasing to source-comment strings.
  - Status: open
  - Grounded in: spec_gap:phase4.acceptance.p4_ac1
  - Evidence: p4_ac1: `for path in docs/thread-story-contract.md packages/core/src/knowledge; do for token in 'core-only story/outbox metadata' 'existing provider idempotency contract' 'canonical v1 milestone'; do rg -n "$token" "$path" >/dev/null || exit 1; done; done`. For the gate to pass, those exact substrings must live both in the doc and somewhere in core/knowledge source. CLAUDE.md says 'Default to writing no comments' — adding comments solely to satisfy a grep gate is incidental coverage and is fragile across renames.
  - Recommendation: Replace the prose grep with a behavioral check (e.g., a test that asserts the idempotency-metadata helper does not duplicate the provider contract's idempotency key field, or an exported constant whose name encodes the boundary) and keep the doc-side check via rg on docs/thread-story-contract.md only. If the gate must remain prose-based, lower its severity and acknowledge that the tokens may appear in a single docstring rather than throughout core.

### round-3

Status: needs_revision
Started: 2026-05-27T18:12:51Z
Ended: 2026-05-27T18:12:51Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 issues are mostly addressed in wording: consumer tests are now in Touchpoints, p4_ac1 only requires type-name greps in core/knowledge with prose greps confined to docs, p4_ac4 fixture-only literals respect provider-neutrality, and explicit per-gate lifecycle mapping (signal->accepted, decision->triaged, ..., merge_gate->human_gate, outcome->final_outcome) is documented. Two new blockers surface from reading the build_feed_entry refresh logic in this round. (1) The hard cut breaks idempotent refresh for in-flight published outbox entries: `tools/outbox/build_feed_entry/src/index.ts:352-367` has a `merge_gate -> outcome` bridge that allows storyEntryCanRefresh to upgrade an existing `:merge_gate` entry into an `:outcome` entry, preserving comment_id/locator/receipt_id. After cutover, fresh story builds will request `:human_gate` / `:final_outcome` ids; any thread that already has a published `:merge_gate` entry will neither match the new id nor be allowed under "reject legacy ids", so the helper will fall back to posting a brand-new comment — duplicating the human merge gate message in the source thread. The spec mandates idempotency and reject-aliases but does not say how the helper handles the legacy-id->canonical-id transition for entries already in `published` state. (2) The spec does not explicitly call out removing the `| string` fallback in `ThreadStorySectionId` at `packages/core/src/knowledge/thread-story.ts:13`. Without removing the fallback (or replacing the type), "tests that reject unknown or legacy ids" cannot be enforced at the type or validator level — any string still typechecks today. Phase 3 Changes mention "Replace freeform milestone acceptance with canonical v1 ids and tests that reject unknown or legacy ids" but don't say the `| string` is removed; touchpoints include the file but the change is implicit. Additional advisories: p3_ac4 and p3_ac5 are alternation-union greps that pass trivially once any one of the listed substrings exists anywhere in the listed paths (and several listed substrings like `signal`, `decision`, `fail_closed`, `pull_request_created`, `human_gate` are guaranteed to appear post-cutover, so the gate validates nothing about *rejection coverage* or *missing-thread-locator handling*); the existing `docs/thread-story-contract.md:41-50` currently documents a different `outbox_entry.metadata.milestone_kind` vocabulary (`intake|triage|spec|build|review|pull_request|merge_gate|outcome`) that the cutover must rewrite, but Phase 3 changes don't explicitly flag the rewrite of that existing list; and the design tension that ThreadStorySectionId (a section heading id for story rendering) and FeedStoryMilestoneKind (a lifecycle gate id for outbox metadata) are being collapsed into one canonical vocabulary, even though several canonical ids (e.g., `ask_for_info`, `reply_drafted`, `no_action`, `monitor`) read more like message *kinds* than story *sections* — worth a one-line rationale.

Checks:
- path audit
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:13
  - Result: passed
  - Evidence: All declared TS touchpoints exist: knowledge/thread-story.ts:1 (line 13 has `| string` fallback in ThreadStorySectionId), feed-entry.ts (line 16-24 defines FeedStoryMilestoneKind), outbox.ts, file-thread.ts, index.ts, index.test.ts. Both consumer tools exist at tools/outbox/build_feed_entry/src/index.ts and tools/thread/push_outbox/src/index.ts. Both newly-added consumer tests (tests/outbox-build-feed-entry-tool.test.ts and tests/thread-push-outbox-tool.test.ts) exist with legacy-id assertions on file. Contracts at packages/contracts/src/schemas/thread-outbox-provider.ts:143-164 expose idempotency.key + content_hash. Rust touchpoints exist (crates/runx-contracts/src/thread_outbox_provider.rs, tests/thread_outbox_provider_fixtures.rs). Docs (thread-story-contract.md, issue-to-pr.md, developer-issue-inbox.md) exist; fixtures/threads/ has issue-to-pr-{file,github}-thread.json. The fixtures/operational-proposal/story-outbox tree is intentionally future (Phase 2 will create it).
- command audit
  - Grounded in: code:tools/outbox/build_feed_entry/src/index.ts:352
  - Result: passed
  - Evidence: pnpm typecheck/test:fast/boundary:check are real package.json scripts (lines 33,35,37). scafld validate is the standard harden gate. The refresh bridge at tools/outbox/build_feed_entry/src/index.ts:352-367 (storyEntryCanRefresh + storyMilestoneCanRefresh with `merge_gate -> outcome` special case) is real and proves the legacy id is currently coupled into the idempotency contract via entry_id slug equality. Phase 3 acceptance commands are well-formed ripgrep syntax.
- scope/migration audit
  - Grounded in: code:tools/outbox/build_feed_entry/src/index.ts:356
  - Result: failed
  - Evidence: Round-2 milestone_vocabulary_semantic_shift was answered with per-gate mapping (signal->accepted, ..., merge_gate->human_gate, outcome->final_outcome). The hard cut therefore changes entry_id slugs from `message:<task>:merge_gate` to `message:<task>:human_gate` and from `:outcome` to `:final_outcome`. The build_feed_entry consumer at tools/outbox/build_feed_entry/src/index.ts:352-367 currently uses a hard-coded special case (`existingMilestone === 'merge_gate' && requestedMilestone === 'outcome'`) to allow an *existing published* `:merge_gate` outbox entry to be refreshed into an `:outcome` entry without duplicating the source-thread comment. After the cutover this bridge must (a) be renamed to `human_gate -> final_outcome` AND (b) bridge legacy `:merge_gate` entries that were already published before the cutover into the new id space — otherwise any thread with a previously-published `:merge_gate` entry will receive a duplicate `:human_gate` comment on the next refresh. The spec mandates `Reject unknown or legacy milestone ids` and `Same-key replay updates/reuses the outbox entry; different milestones, lanes, proposal ids, or target URLs must not collide` but never addresses the in-flight migration of already-published legacy entries. This is the operational counterpart to the round-2 design concern.
- acceptance timing audit
  - Grounded in: spec_gap:phase3.acceptance.p3_ac4
  - Result: failed
  - Evidence: p3_ac4 is `rg -n 'rejects_alias_milestone_ids|legacy milestone|unknown milestone|signal|decision|merge_gate|completion_update|dev_escalation|outreach_recommendation|pull_request_created|human_gate' <paths> --glob '*.test.ts' --glob '*.ts'`. After cutover, `pull_request_created` and `human_gate` MUST exist in core/knowledge and tests as canonical v1 ids; either substring alone satisfies the OR'd regex, so the gate passes without ever exercising alias-rejection coverage. Similarly p3_ac5 uses `'missing_thread_locator|no root fallback|fail_closed'` and `fail_closed` already appears in feed-entry.ts:196 today, so the gate passes without `missing_thread_locator` ever appearing. Both gates were intended to gate rejection-test and missing-thread fail-closed coverage, but the alternation collapses them into trivially-passing greps. (Per-token loops like p2_ac1 or p3_ac6 are the correct pattern.) Phase 3 timing for these checks is correct; the gates themselves do not prove what they claim.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback is mechanical because no live provider sends are issued by this child: revert helpers, fixtures, outbox metadata, and docs; revert TS/Rust contract fixtures together if the provider contract changed; leave existing issue-to-PR story behavior intact unless explicitly replaced. The remaining repair concern is operational and overlaps with the milestone_replay_breaks_idempotency finding: a rollback after the cutover lands in production would have to re-introduce the legacy `:merge_gate` recognition path for any threads that received a `:human_gate` comment during the cutover window. That belongs in the implementation plan, not in rollback shape, so this check stays passed.
- design challenge
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:7
  - Result: passed
  - Evidence: Core architectural call remains sound: provider contract already owns idempotency.key + content_hash (thread-outbox-provider.ts:143-164); core layering of idempotency *metadata* that references the provider contract preserves the trusted-kernel boundary; provider-neutral text/markdown with adapter-owned blocks/payloads respects @runxhq/core domain rules. The remaining design tension is whether collapsing two distinct typed concepts — ThreadStorySectionId (story section heading id for buildThreadStoryMarkdown) and FeedStoryMilestoneKind (outbox milestone lifecycle id for buildFeedStoryOutboxEntry) — into one canonical 18-id vocabulary is the right model. Several canonical ids (`reply_drafted`, `ask_for_info`, `no_action`, `monitor`) read more like message *kinds* than story *section headings*; conflating them risks awkward fit at the renderer (what is the section heading for `ask_for_info`?). The spec should add a one-line rationale: either (a) section_id is renamed to milestone_id and section headings are derived from milestone_id, or (b) the canonical vocabulary is the milestone id and ThreadStorySectionId is repurposed/removed.

Issues:
- [high/blocks approval] `milestone_replay_breaks_idempotency` scope_migration - Hard-cut renaming of milestone ids changes the entry_id slug used by storyEntryCanRefresh; published legacy `:merge_gate` outbox entries in active threads will not be matched after the cutover, producing duplicate source-thread comments and breaking the spec's idempotency promise.
  - Status: open
  - Grounded in: code:tools/outbox/build_feed_entry/src/index.ts:352
  - Evidence: tools/outbox/build_feed_entry/src/index.ts:181 builds the outbox entry_id as `message:${taskId}:${milestone.kind}`. tools/outbox/build_feed_entry/src/index.ts:352-367 has a hard-coded `existingMilestone === 'merge_gate' && requestedMilestone === 'outcome'` bridge so refreshes upgrade in place; this is the mechanism that prevents the human-merge-gate message from being duplicated when the final outcome is observed. The spec's per-gate map (Assumptions lines 178-188) renames `merge_gate -> human_gate` and `outcome -> final_outcome`. After the cutover, build_feed_entry will request `message:<task>:human_gate` and `message:<task>:final_outcome` entries, but any thread that already has a published `message:<task>:merge_gate` entry from before the cutover will not match (storyEntryCanRefresh returns false), and the spec's `Reject unknown or legacy milestone ids` rule (Objectives) blocks accepting the legacy slug as data. The result is a duplicate `:human_gate` comment in the source thread on the next refresh — exactly the root-channel-noise problem the spec lists as a risk. The spec mandates idempotency (`Make publication idempotent and thread-safe`) and same-key replay (`Same-key replay updates/reuses the outbox entry`) but never says how the cutover migrates pre-existing published entries whose entry_id slug contains a legacy milestone id.
  - Recommendation: Pick one of: (a) Add a one-time migration step to the cutover: when storyEntryCanRefresh sees a published entry whose entry_id slug ends in `:merge_gate` or `:outcome`, allow refresh into `:human_gate` or `:final_outcome` respectively, AND have build_feed_entry rewrite the slug on update. Document this as the *only* legacy bridge (not an alias accepted as input). (b) Keep the entry_id slug independent of milestone id — e.g., `message:<task>:human-merge-gate` derived from a stable workflow lane id rather than `milestone.kind`. Then milestone-id rename does not break refresh keys. (c) Add a Phase 3 Changes bullet committing to a one-shot migration of fixture and any local thread state, and document that production threads with published legacy entries must be drained before the cutover. In all three cases Phase 3 must also rewrite tools/outbox/build_feed_entry/src/index.ts:352-367's hard-coded `merge_gate/outcome` strings.
  - Question: How does the cutover preserve idempotency for outbox entries that were published under legacy `:merge_gate`/`:outcome` entry_id slugs before the cutover?
  - Recommended answer: Add a one-shot bridge inside storyEntryCanRefresh that recognizes a published `:merge_gate` entry as the refresh target when the requested entry is `:human_gate` (and `:outcome` -> `:final_outcome`), and have build_feed_entry rewrite the slug to the canonical id on update. Document this as a one-direction migration bridge, not an accepted alias on input. Add a Phase 3 test asserting that a thread with a published `:merge_gate` entry refreshes into a `:human_gate` entry without producing a duplicate comment, and that the legacy slug is no longer accepted on subsequent fresh builds.
  - If unanswered: Default to scoping the cutover narrowly: rename the milestone ids in code and tests as the spec already says, AND add a one-shot legacy-entry-id bridge inside storyEntryCanRefresh that maps published `:merge_gate` to refresh-target `:human_gate` (and `:outcome` to `:final_outcome`), rewriting the slug on update. Document the bridge as a one-direction migration only.
- [high/blocks approval] `section_id_string_fallback_remains` scope_migration - Phase 3 commits to `Replace freeform milestone acceptance with canonical v1 ids and tests that reject unknown or legacy ids`, but never says the `| string` fallback in ThreadStorySectionId is removed; without that removal, runtime rejection of unknown ids is structurally impossible.
  - Status: open
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:13
  - Evidence: packages/core/src/knowledge/thread-story.ts:7-13 declares `export type ThreadStorySectionId = 'initial_issue' | 'triage_results' | 'pr_created' | 'human_merge_gate' | 'completion_update' | string;`. The trailing `| string` collapses the union to `string`, so any value typechecks at compile time. THREAD_STORY_HEADINGS at thread-story.ts:108-114 also falls through to `section.section_id` for unknown ids (renderThreadStorySection line 319), so unknown ids render without throwing. Phase 3 mandates rejection but Touchpoints/Changes do not explicitly say `| string` is removed, the THREAD_STORY_HEADINGS map is replaced with the canonical set, or renderThreadStorySection rejects unknown ids.
  - Recommendation: Add an explicit Phase 3 Changes bullet: 'Remove the `| string` fallback in ThreadStorySectionId, replace THREAD_STORY_HEADINGS with the canonical v1 milestone set, and have buildThreadStoryMarkdown / validateFeedStoryMilestone throw on unknown or legacy ids.' Add a corresponding test that asserts a section_id outside the canonical set throws at validation time (not just renders an awkward heading). Optionally rename ThreadStorySectionId to align with the unified milestone id type so the type system enforces the canonical set.
  - Question: Is the `| string` fallback in ThreadStorySectionId (thread-story.ts:13) removed as part of Phase 3, with a test asserting unknown/legacy ids are rejected at validation rather than silently rendered?
  - Recommended answer: Yes. Replace `ThreadStorySectionId` with the unified canonical milestone-id union (no `| string` fallback), rewrite THREAD_STORY_HEADINGS to the canonical set, and add a validator that throws on unknown ids. Add a test in `packages/core/src/knowledge/index.test.ts` (already in Touchpoints) that constructs a section with a legacy id like `pr_created` and asserts the validator throws.
  - If unanswered: Default to: remove the `| string` fallback, rewrite THREAD_STORY_HEADINGS, add a validator throw for unknown ids, and add an index.test.ts rejection test. Document this as a Phase 3 Changes bullet so the gate is enforceable.
- [medium/advisory] `p3_ac4_p3_ac5_alternation_passes_trivially` weak_gate - p3_ac4 and p3_ac5 use single-regex alternation greps; both pass trivially once a single guaranteed substring (e.g., `pull_request_created`, `human_gate`, `fail_closed`) exists, so the gates validate nothing about alias-rejection coverage or missing-thread-locator handling.
  - Status: open
  - Grounded in: spec_gap:phase3.acceptance.p3_ac4
  - Evidence: p3_ac4: `rg -n 'rejects_alias_milestone_ids|legacy milestone|unknown milestone|signal|decision|merge_gate|completion_update|dev_escalation|outreach_recommendation|pull_request_created|human_gate' <paths> --glob '*.test.ts' --glob '*.ts'`. After cutover, `pull_request_created` and `human_gate` MUST appear in core and tests as canonical v1 ids — either substring alone makes the union match, so the gate exits 0 without proving alias-rejection coverage. p3_ac5: `rg -n 'missing_thread_locator|no root fallback|fail_closed' fixtures/operational-proposal/story-outbox packages/core/src/knowledge`. `fail_closed` already exists in feed-entry.ts:196 today and is preserved by Phase 3 changes — so the gate passes without `missing_thread_locator` or `no root fallback` ever appearing in fixtures or code. Compare to p2_ac1 and p3_ac6 which use per-token `for token in ...; do rg ... || exit 1; done` — those gates enforce *all* tokens.
  - Recommendation: Rewrite p3_ac4 and p3_ac5 as per-token loops with `|| exit 1`, mirroring p3_ac6. For p3_ac4, target the *rejection* substrings explicitly: each of `rejects_alias_milestone_ids`, `legacy milestone`, `unknown milestone` should each be found in the test files (not in source code where the new canonical ids naturally appear). For p3_ac5, require `missing_thread_locator` (or another distinctive phrase from the spec's fail-closed projection guard) to appear in core, separately from the broad `fail_closed` token that already exists upstream.
  - If unanswered: Default to: split p3_ac4 into a per-token loop targeting only the rejection-coverage substrings against the test files; split p3_ac5 into a per-token loop where `missing_thread_locator` is required in `packages/core/src/knowledge` as evidence of the new projection guard.
- [medium/advisory] `docs_thread_story_milestone_kind_list_drift` spec_consistency - docs/thread-story-contract.md:41-50 currently documents `outbox_entry.metadata.milestone_kind` with the legacy vocabulary (`intake|triage|spec|build|review|pull_request|merge_gate|outcome`) — different from both the existing `FeedStoryMilestoneKind` in code and the proposed canonical v1 set. Phase 3 needs to rewrite this list, but Phase 3 Changes do not explicitly mention overwriting the existing list.
  - Status: open
  - Grounded in: code:docs/thread-story-contract.md:41
  - Evidence: docs/thread-story-contract.md lines 41-50 enumerate `intake/triage/spec/build/review/pull_request/merge_gate/outcome` as the canonical milestone kinds. feed-entry.ts:16-24 actually uses `signal/decision/spec/build/review/pull_request/merge_gate/outcome` (no `intake`). The spec's canonical v1 set is 18 outcome-shaped ids. So docs are currently out of sync with code AND with the proposed cutover. Phase 3 says `Update docs/thread-story-contract.md with the canonical milestone, idempotency key, content hash, same-key replay, and different-milestone collision semantics alongside the helper changes` (Changes), but the existing 41-50 list is not explicitly called out as something to delete/rewrite.
  - Recommendation: Add an explicit Phase 3 Changes bullet: 'Rewrite docs/thread-story-contract.md lines 41-50 — replace the legacy `intake/triage/.../outcome` milestone-kind list with the canonical v1 vocabulary, document the per-gate migration mapping, and remove the lingering reference to `intake` (which is not used in any current consumer).' Cross-reference the issue-to-PR doc so the lifecycle mapping is visible to existing-flow readers.
  - If unanswered: Default to: add a Phase 3 Changes bullet committing to the rewrite of docs/thread-story-contract.md:41-50 to the canonical v1 vocabulary, and ensure p3_ac7's grep of `docs/thread-story-contract.md` for `spec_ready`, `build_started`, `review_requested` is satisfied by the new list.
- [low/advisory] `story_section_vs_milestone_conceptual_collapse` design_question - The cutover collapses ThreadStorySectionId (a section heading id used by buildThreadStoryMarkdown) and FeedStoryMilestoneKind (a lifecycle milestone id used by buildFeedStoryOutboxEntry) into one canonical 18-id vocabulary; several canonical ids read more like message kinds than story sections, suggesting these may be distinct concerns being conflated.
  - Status: open
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:108
  - Evidence: thread-story.ts:108-114 maps section ids to display headings (`initial_issue -> 'Initial Issue'`, `pr_created -> 'PR Created'`, ...) — these are *section headings* inside a markdown story. feed-entry.ts:16-24 declares lifecycle milestone ids used to label outbox metadata for a single message. The canonical v1 vocabulary mixes lifecycle-shaped ids (`spec_ready`, `build_started`, `review_requested`, `pull_request_created`, `human_gate`, `final_outcome`) with message-kind-shaped ids (`reply_drafted`, `ask_for_info`, `escalation_proposed`, `no_action`, `monitor`) and state-shaped ids (`accepted`, `hydrated`, `triaged`, `proposal_ready`, `outcome_observed`). Asking what the section heading for `ask_for_info` should be is awkward — it reads as a message kind, not as a section in a multi-section story.
  - Recommendation: Add a one-line rationale to the spec (in Assumptions or a new 'Conceptual Model' section) clarifying that the canonical v1 ids are *milestone ids* serving both as outbox metadata `milestone_kind` AND as story `section_id`, and explain how renderers handle ids that don't naturally form a section heading (e.g., they map to a single-message rendering rather than a multi-section story). Or split the vocabulary back into two related-but-distinct concepts and document the relationship explicitly.
  - If unanswered: Default to documenting the canonical v1 ids as milestone ids; section headings are derived from the milestone id; renderers may produce a single-block message (for `ask_for_info`, `reply_drafted`, etc.) or a multi-section story (for the lifecycle-shaped ids) depending on the milestone.

### round-4

Status: needs_revision
Started: 2026-05-27T18:21:33Z
Ended: 2026-05-27T18:21:33Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-4 read confirms round-3's five blockers are addressed in wording: the legacy-published-refresh migration is documented in Assumptions (lines 189-196) and Phase 3 Changes, p3_ac8 tokenizes its evidence; `| string` removal is now an explicit Phase 3 Changes bullet plus a p3_ac7 anti-grep on `thread-story.ts`; p3_ac4 and p3_ac5 are now per-token loops with `|| exit 1`; the legacy `outbox_entry.metadata.milestone_kind` list rewrite is a Phase 3 Changes bullet; and the section-vs-milestone rationale is documented in Assumptions (lines 197-199). However, a new high finding surfaces from a code read of `packages/core/src/knowledge/index.test.ts:320-380`. That file is in Scope and Touchpoints but Phase 3 Changes only explicitly names `tests/outbox-build-feed-entry-tool.test.ts` and `tests/thread-push-outbox-tool.test.ts`. `index.test.ts` hardcodes `kind: 'decision'|'merge_gate'|'review'` and `milestone_kind: 'review'` assertions that the canonical-v1 cutover will invalidate; `p3_ac2` (`pnpm test:fast`) will therefore fail unless `index.test.ts` is also rewritten in the Phase 3 commit. Additionally, `p4_ac2`'s conditional gate uses `git diff --name-only` against the contract files; once Phase 3 changes are committed (the normal scafld build cadence), the diff goes empty and the gate skips contract tests entirely — silently passing without verifying contracts at all. This is fragile gate timing, not architectural drift.

Checks:
- path audit
  - Grounded in: code:packages/core/src/knowledge/index.test.ts:320
  - Result: passed
  - Evidence: All declared TS touchpoints exist: knowledge/thread-story.ts (line 13 still has `| string` fallback in ThreadStorySectionId pre-implementation), feed-entry.ts (lines 16-24 FeedStoryMilestoneKind), outbox.ts, file-thread.ts, index.ts, index.test.ts (lines 320-380 contain buildFeedStoryOutboxEntry assertions). Both consumer tool dirs (tools/outbox/build_feed_entry/, tools/thread/push_outbox/) and their consumer tests (tests/outbox-build-feed-entry-tool.test.ts, tests/thread-push-outbox-tool.test.ts) exist. Contracts (thread-outbox-provider.ts:143-164 exposes idempotency.key + content_hash) and Rust contracts/tests exist. Docs (thread-story-contract.md, issue-to-pr.md, developer-issue-inbox.md) exist; fixtures/threads/ exists. fixtures/operational-proposal/story-outbox/ is intentionally future (Phase 2 creates it). docs/operational-intelligence.md is correctly dropped.
- command audit
  - Grounded in: code:package.json:35
  - Result: passed
  - Evidence: pnpm typecheck/test:fast/boundary:check (package.json lines 33,35,37) are real scripts. scafld validate is the standard harden gate. p1_ac1's `provider contract already owns public idempotency`, `proposal_kind`, `source-thread locator` substrings all appear in the current spec body (Assumptions section). p3_ac7's anti-grep `if rg -n 'ThreadStorySectionId.*\| string|\| string' .../thread-story.ts; then exit 1; fi` is well-formed and currently matches the existing `| string` fallback at line 13, providing real coverage for the removal.
- scope/migration audit
  - Grounded in: code:packages/core/src/knowledge/index.test.ts:327
  - Result: failed
  - Evidence: packages/core/src/knowledge/index.test.ts:320-380 directly tests buildFeedStoryOutboxEntry with legacy ids: line 327 `kind: 'decision'`, line 335 `kind: 'merge_gate'`, line 363 `kind: 'review'`, line 369 `entry_id: 'message:checkout-fix:review'`, line 377 `milestone_kind: 'review'`. The file is listed in Scope (line 97) and Touchpoints (line 208), but Phase 3 Changes only explicitly names tests/outbox-build-feed-entry-tool.test.ts and tests/thread-push-outbox-tool.test.ts for the cutover. The canonical v1 set rejects `decision`/`merge_gate`/`review` per spec Objectives and Assumptions. p3_ac2 (pnpm test:fast) will fail unless index.test.ts is also rewritten — the touchpoint is listed without committing to the change.
- acceptance timing audit
  - Grounded in: spec_gap:phase4.acceptance.p4_ac2
  - Result: failed
  - Evidence: p4_ac2 is `if git diff --name-only -- packages/contracts/.../thread-outbox-provider.ts crates/runx-contracts/.../thread_outbox_provider.rs | rg -q .; then <run tests>; else true; fi`. `git diff --name-only` shows unstaged working-tree changes only. The normal scafld build cadence commits phase work before evaluating phase acceptance, so by the time p4_ac2 runs the diff is empty and the conditional branch falls through to `true` — contract tests are skipped entirely even when contracts actually changed during this child. The gate cannot distinguish 'contracts unchanged' from 'contracts changed and committed', so it silently turns into a no-op rather than a verification.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: No live provider sends are issued by this child, so reverting helpers, outbox metadata, fixtures, and docs is mechanical. Rollback explicitly calls out reverting TS/Rust contract fixtures together if the provider contract changed, and explicitly preserves the existing issue-to-PR story behavior unless this spec replaced a shared helper. The in-flight legacy-published refresh bridge (Phase 3 Changes line 338) is implemented as a one-direction migration inside storyEntryCanRefresh and rewrites the slug on update; rollback would restore the helper, not undo customer-visible posts, and the spec correctly treats that as an implementation concern rather than a rollback shape issue.
- design challenge
  - Grounded in: code:packages/contracts/src/schemas/thread-outbox-provider.ts:143
  - Result: passed
  - Evidence: Core architectural call remains sound: the public provider contract already owns idempotency.key + content_hash (thread-outbox-provider.ts:143-164), so layering core-only outbox metadata that references that contract preserves the trusted-kernel boundary and the project's public_api_stable invariant. Provider-neutral text/markdown renderers with adapter-owned blocks/payloads respect the @runxhq/core domain rule; p4_ac4 correctly excludes packages/core/src/knowledge from its slack_thread_update/github_issue_comment/github_pr_comment grep paths, so provider-named identifiers stay in fixtures and docs. The defensive projection-time guard at the knowledge layer matches the existing fail-closed source_thread.required behavior already implemented in tools/thread/push_outbox. Round-3's section-vs-milestone collapse rationale is documented in Assumptions lines 197-199.

Issues:
- [high/blocks approval] `harden-1` scope_migration - packages/core/src/knowledge/index.test.ts hardcodes legacy milestone ids (decision/merge_gate/review) in buildFeedStoryOutboxEntry assertions; file is in Touchpoints but Phase 3 Changes does not name it for cutover, so p3_ac2 (pnpm test:fast) will fail.
  - Status: open
  - Grounded in: code:packages/core/src/knowledge/index.test.ts:327
  - Evidence: packages/core/src/knowledge/index.test.ts:327 asserts `kind: 'decision'`, line 335 `kind: 'merge_gate'`, line 363 `kind: 'review'`, line 369 `entry_id: 'message:checkout-fix:review'`, line 377 `milestone_kind: 'review'`. Spec Scope (line 97) and Touchpoints (line 208) list this file, but Phase 3 Changes only explicitly commits to rewriting tests/outbox-build-feed-entry-tool.test.ts and tests/thread-push-outbox-tool.test.ts: `Update `tests/outbox-build-feed-entry-tool.test.ts` and `tests/thread-push-outbox-tool.test.ts` so fast tests assert the new canonical milestone ids and the original issue-to-PR lifecycle mapping.` (Phase 3 Changes). Canonical v1 ids reject decision/merge_gate/review; pnpm test:fast (p3_ac2) cannot pass without rewriting index.test.ts. Round-3 already added a recommendation to add a *rejection* test inside index.test.ts for unknown ids, but the *existing* legacy-id assertions are not addressed by that recommendation.
  - Recommendation: Add an explicit Phase 3 Changes bullet: 'Rewrite the existing legacy milestone-id assertions in packages/core/src/knowledge/index.test.ts (renderFeedStoryMarkdown assertion at lines 320-356 and buildFeedStoryOutboxEntry assertion at lines 358-380) onto the canonical v1 vocabulary, mapping decision→triaged, merge_gate→human_gate, review→review_requested.' Keep the previously-recommended Phase 3 addition of a rejection test for legacy ids inside the same file. Optionally add packages/core/src/knowledge/index.test.ts to the p3_ac4 grep paths so the rejection test is observable.
  - Question: Should Phase 3 Changes also explicitly commit to rewriting the existing buildFeedStoryOutboxEntry and renderFeedStoryMarkdown legacy-id assertions in packages/core/src/knowledge/index.test.ts, in addition to adding the new rejection test?
  - Recommended answer: Yes. Add a Phase 3 Changes bullet naming packages/core/src/knowledge/index.test.ts with the lifecycle remapping (decision→triaged, merge_gate→human_gate, review→review_requested, signal→accepted, pull_request→pull_request_created, outcome→final_outcome) and commit those changes in the same cutover as the consumer tests.
  - If unanswered: Default to adding the Phase 3 Changes bullet that explicitly rewrites the lines 320-380 assertions onto canonical v1 ids; otherwise p3_ac2 (pnpm test:fast) will fail the moment Phase 3 helpers reject the legacy ids.
- [medium/advisory] `harden-2` weak_gate - p4_ac2's conditional `git diff --name-only` gate silently skips contract tests once Phase 3 changes are committed, so contract-touching cutovers can pass Phase 4 without any contract verification.
  - Status: open
  - Grounded in: spec_gap:phase4.acceptance.p4_ac2
  - Evidence: p4_ac2: `if git diff --name-only -- packages/contracts/src/schemas/thread-outbox-provider.ts crates/runx-contracts/src/thread_outbox_provider.rs | rg -q .; then pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features; else true; fi`. `git diff --name-only` (no --cached, no commit range) shows only unstaged working-tree changes. The standard scafld build cadence commits Phase 3 work before Phase 4's acceptance runs, so the diff returns empty and the conditional collapses to `true` — contract tests are skipped entirely, even when those exact files were modified during this child. The gate cannot distinguish 'contracts unchanged' from 'contracts changed and committed'.
  - Recommendation: Replace the working-tree diff with either (a) an unconditional run of the contract tests (cheap and deterministic), or (b) `git diff --name-only HEAD~N` against a baseline tag/commit the spec defines, or (c) a marker file the spec writes when contracts change so the gate becomes a sentinel check rather than a diff check. The simplest fix is (a): always run the contract tests in Phase 4 since they are fast and the spec already lists them under Acceptance/Validation conditionally.
  - Question: Should p4_ac2 unconditionally run the contract tests, or use a different baseline (e.g., HEAD against the approved spec commit) so committed changes are still detected?
  - Recommended answer: Unconditionally run both contract test commands in Phase 4 — they are cheap and the only false positive is a no-op run when contracts did not change. Drop the git-diff conditional and replace it with a direct `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features` command.
  - If unanswered: Default to making p4_ac2 unconditional. Drop the git-diff branch and run both contract test commands as the acceptance, so the gate verifies parity regardless of commit state.

### round-5

Status: passed
Started: 2026-05-27T18:28:09Z
Ended: 2026-05-27T18:28:09Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-5 read confirms round-4's two blockers are resolved in the current spec: (a) Phase 3 Changes now explicitly names `packages/core/src/knowledge/index.test.ts` for the lifecycle rewrite (`decision -> triaged`, `merge_gate -> human_gate`, `review -> review_requested`, `signal -> accepted`, `pull_request -> pull_request_created`, `outcome -> final_outcome`) and keeps the rejection coverage in the same file (lines 338-343, p3_ac9), and (b) p4_ac2 is now an unconditional `pnpm exec vitest run packages/contracts/src && cargo test -p runx-contracts --all-features` rather than a `git diff --name-only` conditional that silently skipped after commits. The earlier-round blockers all stayed fixed: the canonical v1 vocabulary is hard-cut across `ThreadStorySectionId`, `FeedStoryMilestoneKind`, and `outbox_entry.metadata.milestone_kind` with consumer tests in Touchpoints and Phase 3 Changes; the `| string` fallback removal is explicit with an anti-grep in p3_ac7; the legacy-published-refresh bridge is documented in Assumptions and exercised by p3_ac8 (`legacy_published_refresh`, `preserves_comment_id`, `preserves_locator`, `preserves_receipt_ref`, `writes_canonical_milestone_id`, `no_duplicate_comment`); p3_ac4 and p3_ac5 are per-token loops; doc rewrite of `docs/thread-story-contract.md:41-50` is an explicit Phase 3 Changes bullet; and the section-vs-milestone collapse rationale lives in Assumptions. Three low-severity advisories remain and are non-blocking: (i) `tools/outbox/build_feed_entry/fixtures/basic.yaml` lines 79,84 hardcode `:merge_gate` slugs and `milestone_kind: merge_gate` — covered implicitly by the `tools/outbox/build_feed_entry/**` touchpoint but not named in Phase 3 Changes, so explicit mention would prevent the same gap that produced round-4's `index.test.ts` finding; (ii) p3_ac7 requires both `FeedStoryMilestoneKind` and `ThreadStorySectionId` to remain greppable, which constrains the implementation to retain the existing type names (changing contents only) rather than renaming/unifying into a single `StoryMilestoneId` — the spec assumption uses the verb "update" which is consistent with that constraint, but worth a one-line clarification; (iii) p3_ac5's `no root fallback` token is a prose phrase that maps awkwardly to test identifiers or code comments — `missing_thread_locator` and `fail_closed` are natural, but `no root fallback` will likely land as a doc string or comment to satisfy the gate.

Checks:
- path audit
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:13
  - Result: passed
  - Evidence: All declared TS touchpoints exist: thread-story.ts (line 13 still has `| string` fallback pre-implementation), feed-entry.ts (lines 16-24 declares legacy FeedStoryMilestoneKind: signal|decision|spec|build|review|pull_request|merge_gate|outcome), outbox.ts, file-thread.ts, index.ts, index.test.ts (lines 320-380 hold the legacy assertions Phase 3 commits to rewrite). Both consumer tools (tools/outbox/build_feed_entry/, tools/thread/push_outbox/) and consumer tests exist. Contracts (packages/contracts/src/schemas/thread-outbox-provider.ts, run-summary.ts, artifact.ts) exist. Rust touchpoints (crates/runx-contracts/src/thread_outbox_provider.rs, crates/runx-contracts/tests/thread_outbox_provider_fixtures.rs) exist. Docs (thread-story-contract.md with the legacy intake/triage/.../outcome list at lines 41-50, issue-to-pr.md, developer-issue-inbox.md) exist. fixtures/threads/ exists. fixtures/operational-proposal/story-outbox/ is intentionally future (Phase 2 creates it). The previously-flagged docs/operational-intelligence.md path remains correctly removed.
- command audit
  - Grounded in: code:tools/outbox/build_feed_entry/src/index.ts:352
  - Result: passed
  - Evidence: pnpm typecheck, pnpm test:fast, and pnpm boundary:check are real package.json scripts. scafld validate is the standard harden gate. p1_ac1 substrings 'provider contract already owns public idempotency' (line 144 of spec), 'proposal_kind' (line 64-65), and 'source-thread locator' (line 142) all appear in the current spec body. p3_ac7's anti-grep on `ThreadStorySectionId.*\| string|\| string` against thread-story.ts is well-formed and currently matches the line 13 fallback, providing real coverage for its removal. p4_ac2 is now unconditional and exercises both TS and Rust contract suites. The build_feed_entry refresh bridge at index.ts:352-367 is the real legacy slug coupling site that the Assumptions migration bridge needs to update.
- scope/migration audit
  - Grounded in: code:packages/core/src/knowledge/index.test.ts:327
  - Result: passed
  - Evidence: Phase 3 Changes (lines 338-343 of spec) now explicitly names packages/core/src/knowledge/index.test.ts with the full lifecycle remapping (decision -> triaged, merge_gate -> human_gate, review -> review_requested, signal -> accepted, pull_request -> pull_request_created, outcome -> final_outcome) and commits to keeping rejection coverage in the same file. p3_ac9 reinforces this by greppin for `canonical_index_story_milestone`, `accepted`, `triaged`, `spec_ready`, `build_started`, `review_requested`, `pull_request_created`, `human_gate`, `final_outcome`, `rejects_alias_milestone_ids` in packages/core/src/knowledge/index.test.ts. The buildFeedStoryOutboxEntry assertion at index.test.ts:358-380 (kind: 'review', entry_id: 'message:checkout-fix:review', milestone_kind: 'review') and renderFeedStoryMarkdown assertion at 320-356 (kind: 'decision', kind: 'merge_gate') are unambiguously in scope for the cutover. The previously-uncaught fixture file tools/outbox/build_feed_entry/fixtures/basic.yaml (lines 79,84) is in the tools/outbox/build_feed_entry/** touchpoint but not explicitly named in Phase 3 Changes — captured as a low-severity advisory issue.
- acceptance timing audit
  - Grounded in: spec_gap:phase4.acceptance.p4_ac2
  - Result: passed
  - Evidence: p4_ac2 is now `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features` (unconditional). The git-diff conditional gate that silently skipped contract tests after commits is removed. p3_ac6's docs/thread-story-contract.md tokens (`idempotency key`, `content hash`, `same-key`, `different milestones`) are now grounded in Phase 3 Changes that explicitly include the doc rewrite (lines 358-360 of spec, 'Update `docs/thread-story-contract.md` with the canonical milestone, idempotency key, content hash, same-key replay, and different-milestone collision semantics alongside the helper changes'). p4_ac4 keeps provider-named snapshot literals in fixtures and docs only, respecting the provider-neutrality assumption.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: No live provider sends are issued by this child, so reverting helpers, outbox metadata, fixtures, and docs is mechanical. Rollback explicitly calls out reverting TS/Rust contract fixtures together if the provider contract changed, and explicitly preserves the existing issue-to-PR story behavior unless this spec replaced a shared helper. The in-flight legacy-published-refresh bridge is implemented as a one-direction migration inside storyEntryCanRefresh and rewrites the slug on update; rollback restores the helper without undoing customer-visible posts. The operational concern that a rollback after the cutover lands in production would need to re-introduce legacy `:merge_gate` recognition for threads that received a `:human_gate` comment during the cutover window is an implementation concern, not a rollback-shape gap.
- design challenge
  - Grounded in: code:packages/contracts/src/schemas/thread-outbox-provider.ts:143
  - Result: passed
  - Evidence: Core architectural call remains sound: the public provider contract owns idempotency.key + content_hash (thread-outbox-provider.ts:143-164), and core-only outbox metadata that references that contract preserves the trusted-kernel boundary and the public_api_stable invariant. Provider-neutral text/markdown renderers with adapter-owned blocks/payloads respect the @runxhq/core domain rule; p4_ac4 correctly excludes packages/core/src/knowledge from the slack_thread_update/github_issue_comment/github_pr_comment grep paths. The defensive projection-time guard at the knowledge layer matches the existing fail-closed source_thread.required behavior already implemented in tools/thread/push_outbox. The section-vs-milestone collapse rationale is documented in Assumptions. The per-gate lifecycle mapping has settled on outcome-shaped canonical ids with status carrying the lifecycle state, which is acceptable but creates mild semantic awkwardness for ids like `build_started` carrying `status: 'passed'` — captured as an inherited design tension, not a fresh blocker.

Issues:
- [low/advisory] `harden-1` scope_migration - tools/outbox/build_feed_entry/fixtures/basic.yaml lines 79 and 84 hardcode `entry_id: message:task-fixture:merge_gate` and `milestone_kind: merge_gate`; covered by the tools/outbox/build_feed_entry/** touchpoint but not explicitly named in Phase 3 Changes.
  - Status: open
  - Grounded in: code:tools/outbox/build_feed_entry/fixtures/basic.yaml:79
  - Evidence: tools/outbox/build_feed_entry/fixtures/basic.yaml:79 asserts `entry_id: message:task-fixture:merge_gate` and line 84 asserts `milestone_kind: merge_gate`. Spec Touchpoints list `tools/outbox/build_feed_entry/**` so the fixture is in scope, and Phase 3 says 'Update `outbox.build_feed_entry` and `thread.push_outbox` to consume and validate the same canonical milestone ids' which implicitly covers tool fixtures, but the explicit cutover commitment names only `tests/outbox-build-feed-entry-tool.test.ts`, `tests/thread-push-outbox-tool.test.ts`, and `packages/core/src/knowledge/index.test.ts`. After cutover, the consumer emits `:human_gate` entries and the fixture's expected subset must update to match, otherwise p3_ac2 (`pnpm test:fast`) will fail when the build_feed_entry harness replays this fixture. This is the same shape as round-4's index.test.ts finding — file is in touchpoints but missing an explicit Changes bullet.
  - Recommendation: Add a one-line Phase 3 Changes bullet: 'Rewrite tools/outbox/build_feed_entry/fixtures/basic.yaml expected subset to use the canonical v1 milestone id (`message:task-fixture:human_gate`, `milestone_kind: human_gate`) and any other lifecycle-shaped expected ids the fixture relies on.' Optionally also add the fixture path to a Phase 3 acceptance grep to make the cutover observable.
  - Question: Should Phase 3 Changes explicitly name `tools/outbox/build_feed_entry/fixtures/basic.yaml` for the canonical-id rewrite alongside the consumer tests?
  - Recommended answer: Yes — add a Phase 3 Changes bullet committing to rewriting basic.yaml's `entry_id`/`milestone_kind` fields to canonical v1 ids in the same cutover commit. This prevents pnpm test:fast (p3_ac2) from failing on the tool harness replay and mirrors the explicit naming pattern already applied to the three test files.
  - If unanswered: Default to adding the Phase 3 Changes bullet; otherwise the fixture is silently in scope and the test:fast gate may fail mid-build.
- [low/advisory] `harden-2` spec_consistency - p3_ac7 requires `FeedStoryMilestoneKind` AND `ThreadStorySectionId` (the original type names) to remain greppable after cutover, but the spec Assumptions use the verb 'update' which leaves rename-vs-retain ambiguous; implementers might unify them into a single `StoryMilestoneId` type and accidentally fail the gate.
  - Status: open
  - Grounded in: spec_gap:phase3.acceptance.p3_ac7
  - Evidence: p3_ac7: `for token in 'FeedStoryMilestoneKind' 'ThreadStorySectionId' 'milestone_kind' ...; do rg -n "$token" ... || exit 1; done`. Assumption line 173 says 'This child must update `ThreadStorySectionId`, `FeedStoryMilestoneKind`, `outbox_entry.metadata.milestone_kind`, and the `outbox.build_feed_entry` / `thread.push_outbox` consumers together so there is one vocabulary'. p4_ac1 uses an OR pattern `StoryMilestoneId|FeedStoryMilestoneKind|ThreadStorySectionId|milestone_kind` which would tolerate a rename, but p3_ac7 forces retention. The spec is internally consistent if 'update' means 'change union contents while keeping the type alias name', but the design intent could equally support unifying into a new `StoryMilestoneId` (especially given the section-vs-milestone rationale in Assumptions lines 197-199 that the canonical ids are milestone ids serving both roles). A future reader could plausibly choose either.
  - Recommendation: Add a one-line Assumption clarification: 'The existing type names `FeedStoryMilestoneKind` and `ThreadStorySectionId` are retained as aliases over the canonical v1 milestone-id union; this child does not introduce a new `StoryMilestoneId` type name.' Or, alternatively, soften p3_ac7 to use the same OR pattern as p4_ac1 (`'FeedStoryMilestoneKind|StoryMilestoneId'`, `'ThreadStorySectionId|StoryMilestoneId'`).
  - Question: Should the existing type names `FeedStoryMilestoneKind` and `ThreadStorySectionId` be retained as aliases over the canonical union, or unified into a new `StoryMilestoneId`?
  - Recommended answer: Retain both names as aliases over a shared canonical milestone-id union (e.g., `type FeedStoryMilestoneKind = StoryMilestoneId`, `type ThreadStorySectionId = StoryMilestoneId`). This keeps p3_ac7 enforceable, preserves call-site type readability, and matches the spec's 'update' wording.
  - If unanswered: Default to type-alias retention so p3_ac7 stays enforceable. Document the choice in the spec Assumptions so the implementer does not later choose unification and break the gate.
- [low/advisory] `harden-3` weak_gate - p3_ac5's `no root fallback` token is a prose phrase with spaces, awkward to encode as a test identifier or code comment; implementers will likely add it as a doc string solely to satisfy the gate, which is fragile coverage.
  - Status: open
  - Grounded in: spec_gap:phase3.acceptance.p3_ac5
  - Evidence: p3_ac5: `for token in 'missing_thread_locator' 'no root fallback' 'fail_closed'; do rg -n "$token" fixtures/operational-proposal/story-outbox packages/core/src/knowledge >/dev/null || exit 1; done`. `missing_thread_locator` and `fail_closed` are natural underscore-cased identifiers that map cleanly to test names, type fields, or enum values (e.g., feed-entry.ts:196 already uses `missing_behavior: 'fail_closed'`). `no root fallback` is a three-word phrase with spaces — it will not appear as a test identifier or symbol name; it would only land as a doc-comment string or a markdown fixture line, which is incidental coverage and fragile across refactors.
  - Recommendation: Replace `no root fallback` with a natural identifier-shaped token, such as `no_root_channel_fallback` or `source_thread_required`, that maps cleanly to a test name, fail-closed reason code, or fixture field. Keep `missing_thread_locator` and `fail_closed` as-is.
  - Question: Should the `no root fallback` token in p3_ac5 be replaced with an identifier-shaped token (e.g., `no_root_channel_fallback`) so the gate maps to natural code/test names rather than prose comments?
  - Recommended answer: Yes — change the token to `no_root_channel_fallback` (or `source_thread_required`) so implementers can use it as a fail-closed reason code, test name, or fixture key rather than a doc-only prose string.
  - If unanswered: Default to leaving the token but accept that the coverage is doc/comment-only. Mark p3_ac5 as a documentation-presence check rather than a behavioral one.

### round-6

Status: passed
Started: 2026-05-27T18:34:40Z
Ended: 2026-05-27T18:34:40Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-6 read confirms the spec is approval-ready. The five prior blockers (milestone vocabulary scope, p3_ac6 doc timing, lifecycle-shape semantic shift, consumer-test cutover, `| string` fallback removal, legacy-published-refresh bridge, p4_ac2 git-diff weakness, and index.test.ts cutover) all resolve in the current draft. p3_ac4/p3_ac5/p3_ac6/p3_ac9 are per-token loops; p3_ac7 includes the anti-grep that proves removal of `| string` from `packages/core/src/knowledge/thread-story.ts:13`; p3_ac8 tokenizes the legacy-published refresh requirements (`legacy_published_refresh`, `preserves_comment_id`, `preserves_locator`, `preserves_receipt_ref`, `writes_canonical_milestone_id`, `no_duplicate_comment`); `tools/outbox/build_feed_entry/fixtures/basic.yaml` is now explicitly named for the cutover in Phase 3 Changes; p3_ac5 replaced the prose `no root fallback` with the identifier-shaped `root_thread_fallback_rejected`; and `StoryMilestoneId` is the canonical type with the legacy names retained as aliases over the same union. Three low-severity advisories remain and are non-blocking: (a) `tools/outbox/build_pull_request/src/index.ts:247-256` hardcodes legacy lifecycle strings into `outbox_entry.metadata.story_milestones`, which round-trips through `push_outbox.safeStringArray` unvalidated — this is informational adjacent metadata that no renderer consumes, but it walks back the "one vocabulary, not a parallel legacy namespace" assumption; (b) p2_ac1's fixture-matrix coverage tokens (13) are a strict subset of the canonical v1 set (18) — `accepted`, `hydrated`, `triaged`, `review_fixup`, and `outcome_observed` are not enforced as expected-public fixture coverage even though Phase 2 Changes lists "outcome observed" in prose; (c) p4_ac4 requires `slack_thread_update`/`github_issue_comment`/`github_pr_comment` literals to appear in `docs/thread-story-contract.md`, which couples doc phrasing to fixture filenames rather than testing rendering behavior. None block approval.

Checks:
- path audit
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:13
  - Result: passed
  - Evidence: All declared TS touchpoints exist: thread-story.ts (line 13 still has the `| string` fallback pre-implementation), feed-entry.ts (lines 16-24 hold the legacy FeedStoryMilestoneKind union: signal|decision|spec|build|review|pull_request|merge_gate|outcome), outbox.ts, file-thread.ts, index.ts, index.test.ts (lines 320-380 hold the legacy buildFeedStoryOutboxEntry/renderFeedStoryMarkdown assertions Phase 3 commits to rewrite). Both consumer tool dirs (tools/outbox/build_feed_entry/, tools/thread/push_outbox/) and their consumer test files (tests/outbox-build-feed-entry-tool.test.ts, tests/thread-push-outbox-tool.test.ts) exist with legacy-id assertions on file. Contracts (packages/contracts/src/schemas/thread-outbox-provider.ts:143-164 exposes idempotency.key + content_hash, run-summary.ts, artifact.ts) and Rust touchpoints (crates/runx-contracts/src/thread_outbox_provider.rs, crates/runx-contracts/tests/thread_outbox_provider_fixtures.rs) exist. Docs (thread-story-contract.md with the legacy intake/triage/.../outcome list at lines 41-50, issue-to-pr.md, developer-issue-inbox.md) exist. fixtures/threads/ exists with the issue-to-pr-{file,github}-thread.json fixtures. fixtures/operational-proposal/story-outbox/ is intentionally future and is created by Phase 2.
- command audit
  - Grounded in: code:tools/outbox/build_feed_entry/src/index.ts:352
  - Result: passed
  - Evidence: pnpm typecheck/test:fast/boundary:check are real package.json scripts. scafld validate is the standard harden gate. p1_ac1 substrings 'provider contract already owns public idempotency', 'proposal_kind', and 'source-thread locator' all appear in the current spec body. p3_ac7's anti-grep `if rg -n '\| string;' packages/core/src/knowledge/thread-story.ts; then exit 1; fi` is well-formed and currently matches the line-13 fallback exactly once — confirmed by greping the file directly — so the gate provides true coverage for the removal. p4_ac2 is unconditional (no `git diff --name-only` branch) and exercises both TS and Rust contract suites. p3_ac8's `legacy_published_refresh|preserves_comment_id|preserves_locator|preserves_receipt_ref|writes_canonical_milestone_id|no_duplicate_comment` per-token loop maps to real surfaces in the storyEntryCanRefresh/storyMilestoneCanRefresh bridge at tools/outbox/build_feed_entry/src/index.ts:352-367. The push_outbox tool already enforces source_thread.required fail_closed (tools/thread/push_outbox/src/index.ts:80-82, 587-613), so the defensive projection-time guard has a real home.
- scope/migration audit
  - Grounded in: code:tools/outbox/build_pull_request/src/index.ts:247
  - Result: passed
  - Evidence: Phase 3 Changes now explicitly names all four cutover sites: ThreadStorySectionId/FeedStoryMilestoneKind/outbox metadata milestone_kind union, tools/outbox/build_feed_entry/fixtures/basic.yaml, tests/outbox-build-feed-entry-tool.test.ts, tests/thread-push-outbox-tool.test.ts, and packages/core/src/knowledge/index.test.ts with the lifecycle remap (signal->accepted, decision->triaged, spec->spec_ready, build->build_started, review->review_requested, pull_request->pull_request_created, merge_gate->human_gate, outcome->final_outcome). The Assumptions section captures the migration semantics, the legacy-published-refresh bridge, and the rationale for unifying section_id and milestone_id under a single canonical vocabulary. One adjacent metadata field remains uncovered: tools/outbox/build_pull_request/src/index.ts:247-256 emits a hardcoded `story_milestones: ['signal','decision','spec','build','review','pull_request','merge_gate','outcome']` array into pull_request outbox metadata, which is round-tripped through tools/thread/push_outbox/src/index.ts:464 via safeStringArray without canonical-set validation. Because no renderer consumes this field, the pipeline does not break — but it walks back the 'one vocabulary, not a parallel legacy namespace' promise. Captured as a low-severity advisory issue rather than a blocker since the field is informational and no story rendering surface reads it.
- acceptance timing audit
  - Grounded in: spec_gap:phase2.acceptance.p2_ac1
  - Result: passed
  - Evidence: All Phase 3 and Phase 4 gates are timed at the moment their grounding artifacts exist: docs/thread-story-contract.md updates are committed in Phase 3 Changes alongside the helper edits (resolving the earlier p3_ac6 timing miss); p4_ac2 is unconditional and no longer collapses to a no-op after commit; the contract greps in p4_ac1/p4_ac3/p4_ac4/p4_ac5 only require literals that Phase 3/4 Changes commit to introducing. One minor coverage gap: p2_ac1 enforces 13 of the 18 canonical v1 milestone ids in fixtures/operational-proposal/story-outbox/expected/public (it omits `accepted`, `hydrated`, `triaged`, `review_fixup`, and `outcome_observed`). Phase 2 Changes prose mentions `outcome observed` as a fixture target, so the omission appears unintentional. Captured as a low-severity advisory; the gate still proves the spec's headline milestones (reply_drafted/ask_for_info/proposal_ready/escalation_proposed/issue_created/spec_ready/build_started/review_requested/pull_request_created/human_gate/final_outcome/no_action/monitor) appear in expected public output.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: No live provider sends are issued by this child, so reverting helpers, outbox metadata, fixtures, and docs is mechanical. Rollback explicitly calls out reverting TS/Rust contract fixtures together if the provider contract changed, and explicitly preserves the existing issue-to-PR story behavior unless this spec replaced a shared helper. The in-flight legacy-published-refresh bridge is implemented as a one-direction migration inside storyEntryCanRefresh (Assumptions lines 189-196) that rewrites the slug on update; rollback restores the helper without undoing customer-visible posts. The operational concern that a rollback after the cutover lands in production would need to re-introduce legacy `:merge_gate` recognition for threads that received a `:human_gate` comment during the cutover window is an implementation concern rather than a rollback-shape gap, and is acknowledged in the migration semantics.
- design challenge
  - Grounded in: code:packages/contracts/src/schemas/thread-outbox-provider.ts:143
  - Result: passed
  - Evidence: Core architectural call remains sound: the public provider contract owns idempotency.key + content_hash (thread-outbox-provider.ts:143-164), and core-only outbox metadata that references that contract preserves the trusted-kernel boundary and the project's public_api_stable invariant. Provider-neutral text/markdown renderers with adapter-owned blocks/payloads respect the @runxhq/core domain rule; p4_ac4 correctly excludes packages/core/src/knowledge from the slack_thread_update/github_issue_comment/github_pr_comment grep paths so provider-named identifiers stay in fixtures and docs only. The defensive projection-time guard at the knowledge layer matches the existing fail-closed source_thread.required behavior already implemented in tools/thread/push_outbox. The section-vs-milestone collapse rationale is documented in Assumptions: canonical v1 ids are milestone ids serving both as outbox metadata milestone_kind AND as story section_id, with renderers choosing single-message or multi-section rendering based on the milestone shape. Inheriting mild semantic awkwardness (e.g., `build_started` carrying `status: 'passed'`) is acceptable and bounded by status/kind separation.

Issues:
- [low/advisory] `harden-1` scope_migration - tools/outbox/build_pull_request/src/index.ts hardcodes the legacy lifecycle vocabulary in outbox_entry.metadata.story_milestones; the array round-trips unvalidated through push_outbox, leaving a parallel legacy namespace alive after the canonical-v1 cutover.
  - Status: open
  - Grounded in: code:tools/outbox/build_pull_request/src/index.ts:247
  - Evidence: tools/outbox/build_pull_request/src/index.ts:247-256 writes `story_milestones: ['signal','decision','spec','build','review','pull_request','merge_gate','outcome']` into the pull_request outbox entry metadata. tools/thread/push_outbox/src/index.ts:464 reads the field via safeStringArray, which only validates against local-path/secret patterns — not against the canonical milestone vocabulary. No story renderer consumes this field today (greping for story_milestones returns only those two producer/copier sites), so the pipeline does not break; but the Assumptions section commits to 'one vocabulary rather than a parallel legacy namespace' for thread-story section, feed-entry milestone, and outbox metadata milestone_kind. The story_milestones array sits adjacent to milestone_kind in the same outbox_entry.metadata namespace and silently keeps the legacy ids.
  - Recommendation: Add a one-line Assumptions bullet clarifying that outbox_entry.metadata.story_milestones is informational lifecycle-gate metadata, not the canonical milestone vocabulary, and is intentionally out of scope for this child. Alternatively, expand Touchpoints to include tools/outbox/build_pull_request/src/index.ts with a Phase 3 Changes bullet rewriting the hardcoded list.
  - Question: Should tools/outbox/build_pull_request/src/index.ts and the story_milestones metadata field be in Touchpoints/Phase 3 Changes for the canonical-v1 cutover, or explicitly carved out as informational adjacent metadata that is not part of the milestone vocabulary?
  - Recommended answer: Carve it out explicitly: add an Assumptions bullet stating that outbox_entry.metadata.story_milestones is informational lifecycle-gate metadata distinct from the canonical milestone_kind vocabulary and is not in scope for this child. If a future change wants to align it, that is a separate cutover for build_pull_request. This keeps the current spec scope tight while preventing a future reader from treating the legacy strings as a missed-cutover bug.
  - If unanswered: Default to the carve-out: document the field as informational adjacent metadata outside the milestone vocabulary; otherwise add tools/outbox/build_pull_request/** to Touchpoints and a one-line Phase 3 Changes bullet rewriting the literal list onto canonical v1 ids.
- [low/advisory] `harden-2` weak_gate - p2_ac1 enforces 13 of the 18 canonical v1 milestone ids in fixtures/operational-proposal/story-outbox/expected/public; `accepted`, `hydrated`, `triaged`, `review_fixup`, and `outcome_observed` are not enforced as fixture coverage even though they are part of the canonical v1 vocabulary.
  - Status: open
  - Grounded in: spec_gap:phase2.acceptance.p2_ac1
  - Evidence: p2_ac1 token loop: `for token in reply_drafted ask_for_info proposal_ready escalation_proposed issue_created spec_ready build_started review_requested pull_request_created human_gate final_outcome no_action monitor`. The canonical v1 milestone id set in Objectives (lines 47-66 of the spec) has 18 ids: accepted, hydrated, triaged, reply_drafted, ask_for_info, proposal_ready, escalation_proposed, issue_created, spec_ready, build_started, review_requested, pull_request_created, review_fixup, human_gate, outcome_observed, final_outcome, no_action, monitor. Phase 2 Changes prose explicitly lists 'outcome observed' as a fixture target, but p2_ac1 omits it. The gate passes without proving that expected/public fixtures cover the full vocabulary.
  - Recommendation: Either expand p2_ac1's token loop to the full 18-id canonical v1 set (adding accepted, hydrated, triaged, review_fixup, outcome_observed), or document in Phase 2 Changes which canonical ids intentionally lack expected/public fixture coverage and why.
  - Question: Should p2_ac1 enforce all 18 canonical v1 milestone ids as expected/public fixture coverage, or is the 13-id subset intentional?
  - Recommended answer: Expand p2_ac1 to the full 18-id set so every canonical v1 milestone is represented in the expected/public fixture matrix. This matches the Phase 2 Changes prose, prevents silent drift, and makes the fixture matrix complete proof of the canonical vocabulary surface.
  - If unanswered: Default to expanding p2_ac1 to enumerate all 18 canonical v1 milestone ids; otherwise add an Assumptions bullet stating that `accepted`, `hydrated`, `triaged`, `review_fixup`, and `outcome_observed` are status/intermediate ids that do not require their own expected/public fixture and are validated by rendering tests rather than fixture matrix coverage.
- [low/advisory] `harden-3` spec_consistency - p4_ac4 requires the provider-named literals `slack_thread_update`/`github_issue_comment`/`github_pr_comment` to be greppable inside docs/thread-story-contract.md, coupling docs phrasing to fixture filenames rather than testing rendering behavior.
  - Status: open
  - Grounded in: spec_gap:phase4.acceptance.p4_ac4
  - Evidence: p4_ac4: `for token in 'slack_thread_update' 'github_issue_comment' 'github_pr_comment'; do rg -n "$token" fixtures/operational-proposal/story-outbox/expected/public docs/thread-story-contract.md >/dev/null || exit 1; done`. The acceptance correctly excludes packages/core/src/knowledge (resolving the round-2 concern about provider-named identifiers in core). But the doc-side requirement still couples docs prose to fixture filename literals: docs must mention `slack_thread_update`/`github_issue_comment`/`github_pr_comment` by name to satisfy the gate, even though docs/thread-story-contract.md's existing non-goals section explicitly says 'This contract does not admit Slack, Sentry, or support-channel messages.' (line 89). The gate works mechanically — those strings will be added to docs as fixture cross-references — but the coverage is incidental rather than behavioral.
  - Recommendation: Either move the provider-named snapshot literal check to fixtures only (drop docs/thread-story-contract.md from p4_ac4's path list) or extend p4_ac3 with a behavioral phrase that asserts the public renderer flavors map to provider snapshots without admitting provider-specific API behavior.
  - Question: Should p4_ac4 keep docs/thread-story-contract.md in the grep path, or move the provider-named snapshot literal check to fixtures only with a docs-side check that asserts the public/private snapshot split is documented behaviorally?
  - Recommended answer: Keep p4_ac4 as-is for now (the literals are explicit cross-references to fixtures that exist), but add a docs-side behavioral check (e.g., p4_ac3 already requires 'public story', 'private receipt', 'artifact refs', 'source-thread', 'idempotent' tokens in docs — extend that to include a phrase asserting the public renderer flavors map to provider snapshots without admitting provider-specific API behavior).
  - If unanswered: Default to leaving p4_ac4 as-is but acknowledge the docs coverage is fixture-cross-reference rather than behavioral. The non-blocking weakness is documented for future readers who may want to strengthen the gate.

### round-7

Status: passed
Started: 2026-05-27T18:47:08Z
Ended: 2026-05-27T18:47:08Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-7 read confirms the spec is approval-ready. All round-1..6 blockers stay fixed in the current draft: the canonical v1 vocabulary is hard-cut across `ThreadStorySectionId`, `FeedStoryMilestoneKind`, and `outbox_entry.metadata.milestone_kind`; Touchpoints include `feed-entry.ts`, the consumer tools, and the consumer tests; p3_ac7 has an anti-grep on `| string;` against `packages/core/src/knowledge/thread-story.ts:13` and proves the freeform fallback is removed; the legacy-published-refresh bridge for the `:merge_gate -> :human_gate` / `:outcome -> :final_outcome` slug rename is documented and exercised by p3_ac8 (`legacy_published_refresh|preserves_comment_id|preserves_locator|preserves_receipt_ref|writes_canonical_milestone_id|no_duplicate_comment`); p4_ac2 is unconditional (no `git diff --name-only` no-op); `packages/core/src/knowledge/index.test.ts:320-380` is explicitly named in Phase 3 Changes with the lifecycle remap and reinforced by p3_ac9; `tools/outbox/build_feed_entry/fixtures/basic.yaml` is covered by `tools/outbox/build_feed_entry/**` touchpoint plus the explicit Phase 3 commitment; `tools/outbox/build_pull_request/src/index.ts:247-256` legacy `story_milestones` array is now in Phase 3 Changes and gated by p3_ac10 (which forbids the legacy literal strings in build_pull_request source); p3_ac4/p3_ac5/p3_ac6/p3_ac9 are per-token loops; p2_ac1 enumerates all 18 canonical ids; p4_ac4 keeps provider-named literals to fixtures (no `packages/core/src/knowledge` coupling); section-vs-milestone collapse rationale is in Assumptions. Fresh verification: `kind: "pull_request"`/`thread_kind: "signal"` references in `tools/thread/github_adapter.mjs:343`, `plugins/ide-core/src/receipt-view.ts`, and `tests/outbox-build-pull-request-tool.test.ts:80` are the outbox `EntryKind` and `ThreadKind` vocabularies — distinct from `FeedStoryMilestoneKind`/milestone_kind — so the cutover does not silently break other consumers. Three non-blocking observations remain: (i) the spec is sizeable for a "medium" risk: 18 canonical ids, three TS vocabulary collapses, a one-direction migration bridge, and four phases — implementation should keep the milestone-id rename atomic so test:fast stays green within Phase 3; (ii) p3_ac10 requires the literal token `build_pull_request_canonical_story_milestones` to be greppable in `tools/outbox/build_pull_request` or `tests`, which will most likely land as a const name or test identifier — implementers should pick the canonical surface deliberately; (iii) the docs/thread-story-contract.md legacy list at lines 41-50 currently enumerates `intake/triage/spec/build/review/pull_request/merge_gate/outcome` (note: `intake` is NOT used by any current consumer — feed-entry.ts uses `signal/decision/...`), so the Phase 3 doc rewrite should both replace the list and clean the orphaned `intake` reference.

Checks:
- path audit
  - Grounded in: code:packages/core/src/knowledge/thread-story.ts:13
  - Result: passed
  - Evidence: All declared TS touchpoints exist: thread-story.ts (line 13 retains `| string` fallback pre-implementation, confirming p3_ac7 anti-grep target), feed-entry.ts (lines 16-24 hold legacy `FeedStoryMilestoneKind = signal|decision|spec|build|review|pull_request|merge_gate|outcome`), outbox.ts, file-thread.ts, index.ts, index.test.ts (lines 320-380 hold legacy `kind: 'decision'|'merge_gate'|'review'` and `milestone_kind: 'review'` assertions Phase 3 commits to rewriting). tools/outbox/build_feed_entry/ and tools/thread/push_outbox/ exist with src/manifest/fixtures; tests/outbox-build-feed-entry-tool.test.ts:89-118 and tests/thread-push-outbox-tool.test.ts:61,296,312,328 hold the legacy assertions named in Phase 3 Changes. Contracts (packages/contracts/src/schemas/thread-outbox-provider.ts:143-164 owns idempotency.key + content_hash, run-summary.ts, artifact.ts) exist. Rust touchpoints (crates/runx-contracts/src/thread_outbox_provider.rs, tests/thread_outbox_provider_fixtures.rs) exist. Docs (thread-story-contract.md with legacy intake/triage/.../outcome list at lines 41-50, issue-to-pr.md, developer-issue-inbox.md) exist. fixtures/threads/ has issue-to-pr-{file,github}-thread.json. fixtures/operational-proposal/story-outbox/ is intentionally future (Phase 2 creates it). docs/operational-intelligence.md remains correctly removed from earlier rounds.
- command audit
  - Grounded in: code:package.json:35
  - Result: passed
  - Evidence: pnpm typecheck/test:fast/boundary:check are real package.json scripts (lines 33,35,37). `scafld validate` is the standard harden gate. p1_ac1 substrings 'provider contract already owns public idempotency', 'proposal_kind', and 'source-thread locator' all appear in the current spec body. p3_ac7's anti-grep `if rg -n '\| string;' packages/core/src/knowledge/thread-story.ts; then exit 1; fi` matches exactly the line-13 fallback today, so the gate has real coverage. p4_ac2 is unconditional `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src && cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features` — no git-diff no-op branch. p3_ac8's per-token loop maps to real surfaces in the storyEntryCanRefresh/storyMilestoneCanRefresh bridge at tools/outbox/build_feed_entry/src/index.ts:352-367 (hardcoded `merge_gate -> outcome` bridge). push_outbox already enforces source_thread.required fail-closed (tools/thread/push_outbox/src/index.ts), so the defensive projection-time guard has a real home. p3_ac10 (`rg -n 'build_pull_request_canonical_story_milestones' tools/outbox/build_pull_request tests && if rg -n '"signal"|"decision"|"merge_gate"|"outcome"' tools/outbox/build_pull_request/src/index.ts; then exit 1; fi`) currently matches the legacy literal at tools/outbox/build_pull_request/src/index.ts:247-256, confirming the cutover target.
- scope/migration audit
  - Grounded in: code:packages/core/src/knowledge/index.test.ts:327
  - Result: passed
  - Evidence: All cutover sites are now in Touchpoints AND named in Phase 3 Changes with explicit cutover commitments: (a) `packages/core/src/knowledge/thread-story.ts` (ThreadStorySectionId, removal of `| string`, THREAD_STORY_HEADINGS), (b) `packages/core/src/knowledge/feed-entry.ts` (FeedStoryMilestoneKind), (c) `packages/core/src/knowledge/index.test.ts:320-380` (lifecycle remap decision→triaged, merge_gate→human_gate, review→review_requested, signal→accepted, pull_request→pull_request_created, outcome→final_outcome — Phase 3 Changes lines naming the file plus rejection coverage), (d) `tests/outbox-build-feed-entry-tool.test.ts` and `tests/thread-push-outbox-tool.test.ts` (Phase 3 Changes), (e) `tools/outbox/build_feed_entry/fixtures/basic.yaml:79,84` (covered by tools/outbox/build_feed_entry/** Touchpoint plus explicit Phase 3 Changes bullet), (f) `tools/outbox/build_pull_request/src/index.ts:247-256` story_milestones literal (Phase 3 Changes plus p3_ac10 gate forbidding legacy strings). The legacy-published-refresh bridge at tools/outbox/build_feed_entry/src/index.ts:352-367 has an explicit migration semantics paragraph in Assumptions (legacy_published_refresh, preserves_comment_id, preserves_locator, preserves_receipt_ref, writes_canonical_milestone_id, no_duplicate_comment) and a tokenized acceptance gate. Verified that adjacent vocabularies are NOT in scope and do not break: `tools/thread/github_adapter.mjs:343` uses outbox `EntryKind` (`pull_request`), not milestone_kind; `tests/recognizable-work-lanes.test.ts:177` uses `thread_kind: 'signal'`, a ThreadKind not a milestone; `tests/outbox-build-pull-request-tool.test.ts:80` uses outbox EntryKind; `plugins/ide-core/src/receipt-view.ts` does not reference milestone_kind.
- acceptance timing audit
  - Grounded in: spec_gap:phase4.acceptance.p4_ac2
  - Result: passed
  - Evidence: Every gate is grounded in an artifact authored by the same or earlier phase. Phase 1 gates check spec body presence and validate (no code touched). Phase 2 gates check fixtures that Phase 2 Changes commits to creating (`fixtures/operational-proposal/story-outbox/inputs/private/`, `expected/public/`); p2_ac1's token loop covers all 18 canonical ids. Phase 3 gates: p3_ac1/p3_ac2/p3_ac3 are pnpm typecheck/test:fast/boundary:check after the helper+test cutover (test files explicit in Changes so test:fast can pass); p3_ac6 docs greps `docs/thread-story-contract.md` for `idempotency key|content hash|same-key|different milestones` — Phase 3 Changes explicitly include the doc rewrite, resolving the earlier p3_ac6 timing miss; p3_ac7 anti-grep on `| string;` matches today and Phase 3 Changes commit to its removal; p3_ac8 tokens are introduced by Phase 3 Changes; p3_ac10 requires `build_pull_request_canonical_story_milestones` token (added by Phase 3) AND forbids legacy string literals (removed by Phase 3). Phase 4: p4_ac2 is unconditional so it cannot collapse to a no-op after commits; p4_ac4 fixture-only literal check respects provider neutrality. No gate references an artifact owned by a later phase.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: No live provider sends are issued by this child, so reverting helpers, outbox metadata, fixtures, and docs is mechanical. Rollback explicitly calls out reverting TS/Rust contract fixtures together if the provider contract changed, and explicitly preserves the existing issue-to-PR story behavior unless this spec replaced a shared helper. The in-flight legacy-published-refresh bridge is implemented as a one-direction migration inside storyEntryCanRefresh (Assumptions lines documenting the migration semantics) that rewrites the slug on update; rollback restores the helper without undoing customer-visible posts. The operational concern that a rollback after the cutover lands in production would need to re-introduce legacy `:merge_gate` recognition for threads that received a `:human_gate` comment during the cutover window is acknowledged in the migration semantics — it is an implementation-time concern (drain or backfill), not a rollback-shape gap.
- design challenge
  - Grounded in: code:packages/contracts/src/schemas/thread-outbox-provider.ts:143
  - Result: passed
  - Evidence: Core architectural call remains sound. The public provider contract owns `idempotency.key` + `content_hash` (thread-outbox-provider.ts:143-164) and observation status; core-only outbox metadata that references that contract preserves the trusted-kernel boundary and the project's `public_api_stable`/`no_legacy_code` invariants. Provider-neutral text/markdown renderers with adapter-owned blocks/payloads respect the @runxhq/core domain rule; p4_ac4 correctly excludes packages/core/src/knowledge from the slack_thread_update/github_issue_comment/github_pr_comment grep paths, so provider-named identifiers stay in fixtures and docs. The defensive projection-time guard at the knowledge layer mirrors the existing fail-closed source_thread.required behavior already implemented in tools/thread/push_outbox. The section-vs-milestone collapse rationale is documented in Assumptions (canonical v1 ids are milestone ids serving both as outbox metadata milestone_kind AND as story section_id; renderers choose single-message or multi-section rendering based on the milestone shape). Per-gate lifecycle mapping has settled on outcome-shaped canonical ids with status carrying the lifecycle state — mild semantic awkwardness (e.g., `build_started` carrying `status: 'passed'`) is acceptable and bounded by status/kind separation. The single canonical `StoryMilestoneId` type with optional legacy-name aliases over the same union (p3_ac7 requires the canonical name; p4_ac1's OR pattern keeps alias retention flexible) is internally consistent.

Issues:
- [low/advisory] `harden-1` spec_consistency - Existing docs/thread-story-contract.md milestone list (intake/triage/spec/build/review/pull_request/merge_gate/outcome) contains an orphaned `intake` id that is not used by any current consumer — feed-entry.ts:16-24 uses `signal/decision/...`. Phase 3 doc rewrite should remove `intake` along with the other legacy ids.
  - Status: open
  - Grounded in: code:docs/thread-story-contract.md:41
  - Evidence: docs/thread-story-contract.md lines 41-50 enumerate `intake|triage|spec|build|review|pull_request|merge_gate|outcome` as the canonical milestone kinds. feed-entry.ts:16-24 actually uses `signal|decision|spec|build|review|pull_request|merge_gate|outcome` (no `intake`). Phase 3 Changes commits to rewriting this list onto the canonical v1 ids; the rewrite should naturally drop `intake`, but flagging it now prevents a reader from preserving `intake` as a historical artifact.
  - Recommendation: When Phase 3 rewrites docs/thread-story-contract.md lines 41-50, ensure `intake` is dropped (not preserved as legacy mapping) since it has no current consumer. The canonical v1 list should not carry a per-gate mapping for `intake`.
  - If unanswered: Default to dropping `intake` from the rewrite; document in the Phase 3 Changes commit that the per-gate mapping table only includes the eight current FeedStoryMilestoneKind ids (signal/decision/spec/build/review/pull_request/merge_gate/outcome).
- [low/advisory] `harden-2` implementation_guidance - p3_ac10 requires the literal token `build_pull_request_canonical_story_milestones` to be greppable in `tools/outbox/build_pull_request` or `tests`. The spec does not pin whether this is a const export name, a test identifier, or a fixture key — implementers will pick one of three surfaces, and the choice subtly shapes the post-cutover surface.
  - Status: open
  - Grounded in: spec_gap:phase3.acceptance.p3_ac10
  - Evidence: p3_ac10: `rg -n 'build_pull_request_canonical_story_milestones' tools/outbox/build_pull_request tests >/dev/null && if rg -n '"signal"|"decision"|"merge_gate"|"outcome"' tools/outbox/build_pull_request/src/index.ts; then exit 1; fi`. The token is not currently in the repo. It could land as (a) an exported const array in tools/outbox/build_pull_request/src/index.ts holding the canonical story_milestones list, (b) a test name in tests/outbox-build-pull-request-tool.test.ts asserting the new array, or (c) a fixture key. Option (a) is the most behaviorally meaningful and matches the cutover intent; the others satisfy the gate but leave the canonical list inline.
  - Recommendation: Treat option (a) as the default: export `build_pull_request_canonical_story_milestones` as a named const array in tools/outbox/build_pull_request/src/index.ts and reference it from the metadata.story_milestones field. This makes the canonical surface re-usable, removes the inline literal, and naturally satisfies the p3_ac10 anti-grep on legacy strings.
  - Question: Which surface does `build_pull_request_canonical_story_milestones` land on — exported const, test name, or fixture key?
  - Recommended answer: Exported const array in tools/outbox/build_pull_request/src/index.ts referenced from metadata.story_milestones. This eliminates the inline legacy literal and gives a single shared canonical list.
  - If unanswered: Default to exported const array in tools/outbox/build_pull_request/src/index.ts; document the surface choice in Phase 3 Changes so future readers see why the gate exists.
- [low/advisory] `harden-3` implementation_risk - Phase 3 bundles three TS vocabulary collapses, a one-direction migration bridge, fixture/test rewrites in five files, doc rewrites, and adds a defensive projection-time guard. Keeping the cutover atomic so `pnpm test:fast` (p3_ac2) stays green across a single commit is feasible but requires sequencing care.
  - Status: open
  - Grounded in: spec_gap:phase3.changes
  - Evidence: Phase 3 Changes touches: thread-story.ts (type + headings + validator), feed-entry.ts (kind union + buildFeedStoryOutboxEntry), outbox.ts (likely metadata schema), index.ts (re-exports), index.test.ts:320-380 (assertion rewrite), tools/outbox/build_feed_entry/src/index.ts:352-367 (refresh bridge + entry_id slug), tools/outbox/build_feed_entry/fixtures/basic.yaml:70-85 (fixture), tools/thread/push_outbox/src/index.ts (any milestone_kind validation), tools/outbox/build_pull_request/src/index.ts:247-256 (story_milestones literal), tests/outbox-build-feed-entry-tool.test.ts:85-118+ (legacy id assertions), tests/thread-push-outbox-tool.test.ts:61,296,312,328 (legacy entry_id slugs), docs/thread-story-contract.md:41-50 (vocabulary list). Any in-flight commit that lands the helper rename before the test rewrite (or vice versa) will turn pnpm test:fast red.
  - Recommendation: Implement Phase 3 as a single commit (or use git rebase to squash before evaluation) so test:fast does not see an intermediate broken state. Order: (1) introduce StoryMilestoneId union and legacy aliases, (2) update validators to accept canonical ids only and add the legacy_published_refresh bridge, (3) rename milestone.kind values in tests/fixtures/index.test.ts/build_pull_request literals in the same commit, (4) update docs alongside. The scafld build phase model is one commit per phase, which matches this — just keep the diff atomic.
  - If unanswered: Default to single-commit Phase 3 with the ordering above; if mid-build commits are needed for review readability, gate them behind a feature flag temporarily so the test suite stays green throughout.


## Planning Log

- Reframed story/outbox from support/alert/outreach/roadmap milestones to a
  generic proposal/action/outcome story.
