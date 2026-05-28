export const knowledgePackage = "@runxhq/core/knowledge";

import { RUNX_CONTRACT_IDS } from "@runxhq/contracts";

// Schema refs for the live local knowledge store + handoff/suppression contracts
// (still owned by @runxhq/contracts). Story/outbox helpers below stay
// provider-neutral: they render safe text/markdown and metadata, never provider
// API calls.
export const RUNX_SCHEMA_REFS = {
  knowledge_entry: "https://runx.ai/spec@runxhq/core/knowledge-entry.schema.json",
  handoff_signal: RUNX_CONTRACT_IDS.handoffSignal,
  handoff_state: RUNX_CONTRACT_IDS.handoffState,
  suppression_record: RUNX_CONTRACT_IDS.suppressionRecord,
} as const;

export {
  type Actor,
  type EvidenceRef,
  type ThreadAdapterDescriptor,
} from "./internal-validators.js";

export {
  type LocalKnowledgeEntryKind,
  type LocalKnowledgeReceiptEntry,
  type LocalKnowledgeProjectionEntry,
  type LocalKnowledgeAnswerEntry,
  type LocalKnowledgeArtifactEntry,
  type LocalKnowledgeEntry,
  type LocalKnowledge,
  type IndexReceiptOptions,
  type AddProjectionOptions,
  type LocalKnowledgeStore,
  createFileKnowledgeStore,
} from "./local-store.js";

export {
  STORY_MILESTONE_IDS,
  ISSUE_TO_PR_STORY_MILESTONES,
  LEGACY_STORY_MILESTONE_ID_MAP,
  STORY_MILESTONE_LABELS,
  type StoryMilestoneId,
  type ThreadStorySectionId,
  type StoryMilestone,
  type ThreadStory,
  isStoryMilestoneId,
  assertStoryMilestoneId,
  canonicalStoryMilestoneIdForPublishedRefresh,
  assertSourceThreadPublicationAllowed,
  renderThreadStoryMarkdown,
  renderStoryMilestoneMarkdown,
  sanitizePublicMarkdown,
  summarizePublicHandoffMarkdown,
  friendlyProposalLabel,
} from "./thread-story.js";

export {
  type StoryOutboxMetadataInput,
  type StoryOutboxIdempotencyMetadata,
  buildStoryOutboxIdempotencyMetadata,
  buildCoreStoryOutboxMetadata,
} from "./outbox.js";

export {
  type FeedStoryMilestoneKind,
  type FeedStoryOutboxEntryInput,
  renderFeedStoryMarkdown,
  buildFeedStoryOutboxEntry,
} from "./feed-entry.js";

export {
  storyMilestoneRefreshesPublishedEntry,
  canonicalStoryEntryIdForRefresh,
} from "./file-thread.js";
