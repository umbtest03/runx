import { hashStable, hashString } from "../util/hash.js";
import {
  type StoryMilestoneId,
  assertSourceThreadPublicationAllowed,
  assertStoryMilestoneId,
  sanitizePublicMarkdown,
} from "./thread-story.js";

export interface StoryOutboxMetadataInput {
  readonly sourceId?: unknown;
  readonly provider?: unknown;
  readonly sourceThreadRef?: unknown;
  readonly workflowId?: unknown;
  readonly laneId?: unknown;
  readonly milestoneId?: unknown;
  readonly targetRef?: unknown;
  readonly proposalId?: unknown;
  readonly bodyMarkdown?: unknown;
  readonly requiresSourceThreadPublication?: boolean;
}

export interface StoryOutboxIdempotencyMetadata {
  readonly key: string;
  readonly content_hash: string;
}

export function buildStoryOutboxIdempotencyMetadata(input: StoryOutboxMetadataInput): StoryOutboxIdempotencyMetadata {
  const milestoneId = assertStoryMilestoneId(input.milestoneId, "outbox_entry.metadata.milestone_kind");
  const sourceThreadRef = assertSourceThreadPublicationAllowed({
    requiresSourceThreadPublication: input.requiresSourceThreadPublication,
    sourceThreadRef: input.sourceThreadRef,
    missingBehavior: "fail_closed",
  });
  const contentHash = hashString(sanitizePublicMarkdown(input.bodyMarkdown)?.trim() ?? "");
  const keyMaterial = {
    source_id: clean(input.sourceId),
    provider: clean(input.provider),
    source_thread_ref: sourceThreadRef,
    workflow_id: clean(input.workflowId),
    lane_id: clean(input.laneId),
    milestone_id: milestoneId,
    target_ref: clean(input.targetRef),
    proposal_id: clean(input.proposalId),
    content_hash: contentHash,
  };
  return {
    key: `story:${hashStable(keyMaterial).slice(0, 32)}`,
    content_hash: contentHash,
  };
}

export function buildCoreStoryOutboxMetadata(input: StoryOutboxMetadataInput): {
  readonly milestone_kind: StoryMilestoneId;
  readonly idempotency: StoryOutboxIdempotencyMetadata;
  readonly replay: {
    readonly same_key: "update_or_reuse";
    readonly different_milestones: "distinct_entries";
  };
} {
  const milestoneId = assertStoryMilestoneId(input.milestoneId, "outbox_entry.metadata.milestone_kind");
  return {
    milestone_kind: milestoneId,
    idempotency: buildStoryOutboxIdempotencyMetadata({
      ...input,
      milestoneId,
    }),
    replay: {
      same_key: "update_or_reuse",
      different_milestones: "distinct_entries",
    },
  };
}

function clean(value: unknown): string | undefined {
  const sanitized = sanitizePublicMarkdown(value)?.trim();
  return sanitized || undefined;
}
