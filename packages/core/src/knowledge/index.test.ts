import { describe, expect, it } from "vitest";

import {
  LEGACY_STORY_MILESTONE_ID_MAP,
  STORY_MILESTONE_IDS,
  assertSourceThreadPublicationAllowed,
  assertStoryMilestoneId,
  buildFeedStoryOutboxEntry,
  buildStoryOutboxIdempotencyMetadata,
  renderFeedStoryMarkdown,
} from "./index.js";

describe("@runxhq/core/knowledge story outbox helpers", () => {
  it("canonical_index_story_milestone maps the tracking-to-change lifecycle", () => {
    expect(STORY_MILESTONE_IDS).toEqual(expect.arrayContaining([
      "accepted",
      "triaged",
      "spec_ready",
      "build_started",
      "review_requested",
      "change_request_created",
      "human_gate",
      "final_outcome",
    ]));
    expect(LEGACY_STORY_MILESTONE_ID_MAP).toMatchObject({
      signal: "accepted",
      decision: "triaged",
      spec: "spec_ready",
      build: "build_started",
      review: "review_requested",
      pull_request: "change_request_created",
      merge_gate: "human_gate",
      outcome: "final_outcome",
    });
  });

  it("rejects_alias_milestone_ids unknown_milestone and legacy ids", () => {
    const legacyAliasCases = [
      ["legacy_signal", "signal"],
      ["legacy_decision", "decision"],
      ["legacy_spec", "spec"],
      ["legacy_build", "build"],
      ["legacy_review", "review"],
      ["legacy_pull_request", "pull_request"],
      ["legacy_merge_gate", "merge_gate"],
      ["legacy_outcome", "outcome"],
    ] as const;

    for (const [label, value] of legacyAliasCases) {
      expect(() => assertStoryMilestoneId(value, label)).toThrow(/legacy milestone id/u);
    }
    expect(() => assertStoryMilestoneId("dev_escalation", "unknown_milestone")).toThrow(/unknown_milestone/u);
  });

  it("renders concise public markdown and redacts private details", () => {
    const markdown = renderFeedStoryMarkdown({
      title: "Operational follow-up",
      next_action: "Review the change request.",
      source_ref: "support://case/ops-123",
      source_thread_ref: "provider://thread/ops-123",
      result_refs: ["tracking_item=track://issue/77", "change_request=change://pr/88"],
      publication_refs: ["source_thread_update", "tracking_item_comment", "change_request_comment"],
      milestones: [
        {
          kind: "proposal_ready",
          status: "ready",
          proposal_kind: "dev_escalation",
          summary: "Proposal ready from proposal_kind without accepting a domain milestone id.",
          details: [
            "Receipt: artifact://runx/private/ops-123",
            "RUNX_BIN=/Users/example/dev/runx/dist/index.js",
          ],
        },
      ],
    });

    expect(markdown).toContain("Dev Escalation");
    expect(markdown).toContain("source_ref");
    expect(markdown).toContain("artifact://runx/private/ops-123");
    expect(markdown).toContain("[local-path]");
    expect(markdown).not.toContain("/Users/example");
  });

  it("builds idempotent outbox metadata with content hash and replay semantics", () => {
    const entry = buildFeedStoryOutboxEntry({
      taskId: "story-task",
      threadLocator: "provider://thread/ops-123",
      title: "Story update",
      milestone: {
        kind: "review_requested",
        status: "ready",
      },
      bodyMarkdown: "Public story body.",
      updatedAt: "2026-05-28T00:00:00Z",
    });

    expect(entry).toMatchObject({
      entry_id: "message:story-task:review_requested",
      metadata: {
        milestone_kind: "review_requested",
        idempotency: {
          key: expect.stringMatching(/^story:[a-f0-9]{32}$/u),
          content_hash: expect.stringMatching(/^[a-f0-9]{64}$/u),
        },
        replay: {
          same_key: "update_or_reuse",
          different_milestones: "distinct_entries",
        },
      },
    });

    const first = buildStoryOutboxIdempotencyMetadata({
      sourceId: "source-123",
      provider: "provider",
      sourceThreadRef: "provider://thread/ops-123",
      workflowId: "issue-to-pr",
      laneId: "change",
      milestoneId: "human_gate",
      targetRef: "change://pr/88",
      bodyMarkdown: "Body.",
      requiresSourceThreadPublication: true,
    });
    const sameKey = buildStoryOutboxIdempotencyMetadata({
      sourceId: "source-123",
      provider: "provider",
      sourceThreadRef: "provider://thread/ops-123",
      workflowId: "issue-to-pr",
      laneId: "change",
      milestoneId: "human_gate",
      targetRef: "change://pr/88",
      bodyMarkdown: "Body.",
      requiresSourceThreadPublication: true,
    });
    const differentMilestones = buildStoryOutboxIdempotencyMetadata({
      sourceId: "source-123",
      provider: "provider",
      sourceThreadRef: "provider://thread/ops-123",
      workflowId: "issue-to-pr",
      laneId: "change",
      milestoneId: "final_outcome",
      targetRef: "change://pr/88",
      bodyMarkdown: "Body.",
      requiresSourceThreadPublication: true,
    });

    expect(sameKey.key).toBe(first.key);
    expect(differentMilestones.key).not.toBe(first.key);
  });

  it("missing_thread_locator root_thread_fallback_rejected fail_closed", () => {
    expect(() => assertSourceThreadPublicationAllowed({
      requiresSourceThreadPublication: true,
      missingBehavior: "fail_closed",
    })).toThrow(/missing_thread_locator: root_thread_fallback_rejected/u);
  });
});
