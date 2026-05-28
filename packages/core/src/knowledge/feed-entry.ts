import { hashString } from "../util/hash.js";
import {
  type StoryMilestone,
  type StoryMilestoneId,
  type ThreadStory,
  assertSourceThreadPublicationAllowed,
  assertStoryMilestoneId,
  renderThreadStoryMarkdown,
  sanitizePublicMarkdown,
} from "./thread-story.js";
import { buildCoreStoryOutboxMetadata } from "./outbox.js";

export type FeedStoryMilestoneKind = StoryMilestoneId;

export interface FeedStoryOutboxEntryInput {
  readonly taskId?: unknown;
  readonly threadLocator?: unknown;
  readonly title?: unknown;
  readonly milestone?: Partial<StoryMilestone> & Record<string, unknown>;
  readonly bodyMarkdown?: unknown;
  readonly updatedAt?: unknown;
  readonly workflow?: unknown;
  readonly laneId?: unknown;
  readonly sourceId?: unknown;
  readonly provider?: unknown;
  readonly targetRef?: unknown;
  readonly proposalId?: unknown;
}

export function renderFeedStoryMarkdown(story: ThreadStory): string {
  return renderThreadStoryMarkdown(story);
}

export function buildFeedStoryOutboxEntry(input: FeedStoryOutboxEntryInput): Record<string, unknown> {
  const taskId = clean(input.taskId) ?? "unknown-task";
  const threadLocator = assertSourceThreadPublicationAllowed({
    requiresSourceThreadPublication: true,
    sourceThreadRef: input.threadLocator,
    missingBehavior: "fail_closed",
  });
  const milestone = input.milestone ?? {};
  const milestoneKind = assertStoryMilestoneId(milestone.kind, "outbox_entry.metadata.milestone_kind");
  const bodyMarkdown = clean(input.bodyMarkdown) ?? "";
  const workflow = clean(input.workflow) ?? "issue-to-pr";
  const coreMetadata = buildCoreStoryOutboxMetadata({
    sourceId: input.sourceId ?? taskId,
    provider: input.provider ?? "source_thread",
    sourceThreadRef: threadLocator,
    workflowId: workflow,
    laneId: input.laneId ?? workflow,
    milestoneId: milestoneKind,
    targetRef: input.targetRef,
    proposalId: input.proposalId,
    bodyMarkdown,
    requiresSourceThreadPublication: true,
  });
  const receiptHash = hashString(JSON.stringify({
    taskId,
    threadLocator,
    milestoneKind,
    bodyMarkdown,
    updatedAt: clean(input.updatedAt),
  })).slice(0, 20);

  return {
    entry_id: `message:${taskId}:${milestoneKind}`,
    kind: "message",
    status: "proposed",
    thread_locator: threadLocator,
    title: clean(input.title) ?? "Issue-to-PR story",
    metadata: {
      schema_version: "runx.outbox-entry.feed-entry.v1",
      workflow,
      milestone_kind: milestoneKind,
      outbox_receipt_id: `feed:${workflow}:${taskId}:${milestoneKind}:${receiptHash}`,
      idempotency: coreMetadata.idempotency,
      replay: coreMetadata.replay,
      source_thread: {
        required: true,
        publish_mode: "reply",
        missing_behavior: "fail_closed",
        thread_locator: threadLocator,
      },
      body_markdown: bodyMarkdown,
    },
  };
}

function clean(value: unknown): string | undefined {
  const sanitized = sanitizePublicMarkdown(value)?.trim();
  return sanitized || undefined;
}
