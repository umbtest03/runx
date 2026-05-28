import { type StoryMilestoneId, canonicalStoryMilestoneIdForPublishedRefresh } from "./thread-story.js";

export function storyMilestoneRefreshesPublishedEntry(existing: unknown, requested: StoryMilestoneId): boolean {
  const existingCanonical = canonicalStoryMilestoneIdForPublishedRefresh(existing);
  if (existingCanonical === requested) {
    return true;
  }
  return existing === "merge_gate" && requested === "final_outcome";
}

export function canonicalStoryEntryIdForRefresh(entryId: string | undefined, existing: unknown, requested: StoryMilestoneId): string | undefined {
  if (!entryId || typeof existing !== "string") {
    return entryId;
  }
  if (!storyMilestoneRefreshesPublishedEntry(existing, requested)) {
    return entryId;
  }
  return entryId.replace(new RegExp(`:${escapeRegExp(existing)}$`, "u"), `:${requested}`);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
