import { createHash } from "node:crypto";

import {
  optionalEnum,
  optionalString,
  optionalStringArray,
  requireArray,
  requireEnum,
  requireRecord,
  requireString,
  validateEvidenceRef,
  type EvidenceRef,
} from "./internal-validators.js";
import type { OutboxEntry } from "./outbox.js";

export type FeedStoryMilestoneKind =
  | "signal"
  | "decision"
  | "spec"
  | "build"
  | "review"
  | "pull_request"
  | "merge_gate"
  | "outcome";

export type FeedStoryMilestoneStatus =
  | "pending"
  | "ready"
  | "passed"
  | "failed"
  | "blocked"
  | "completed";

export interface FeedStoryMilestone {
  readonly kind: FeedStoryMilestoneKind;
  readonly status?: FeedStoryMilestoneStatus;
  readonly title?: string;
  readonly summary: string;
  readonly details?: readonly string[];
  readonly evidence?: readonly EvidenceRef[];
}

export interface FeedStory {
  readonly thread_locator: string;
  readonly title?: string;
  readonly next_action?: string;
  readonly milestones: readonly FeedStoryMilestone[];
}

export interface BuildFeedStoryOutboxEntryOptions {
  readonly taskId: string;
  readonly threadLocator: string;
  readonly milestone: FeedStoryMilestone | Readonly<Record<string, unknown>>;
  readonly title?: string;
  readonly workflow?: string;
  readonly bodyMarkdown?: string;
  readonly outboxReceiptId?: string;
  readonly updatedAt?: string;
}

export interface RenderIssueToPrReviewerMarkdownOptions {
  readonly taskId: string;
  readonly title?: string;
  readonly sourceTitle?: string;
  readonly sourceLocator?: string;
  readonly branch?: string;
  readonly base?: string;
  readonly governanceStatus?: string;
  readonly checkStatus?: string;
  readonly buildPassed?: number;
  readonly buildFailed?: number;
  readonly reviewVerdict?: string;
  readonly blockingCount?: number;
  readonly nonBlockingCount?: number;
  readonly handoffMarkdown?: string;
}

const milestoneKinds = [
  "signal",
  "decision",
  "spec",
  "build",
  "review",
  "pull_request",
  "merge_gate",
  "outcome",
] as const;

const milestoneStatuses = [
  "pending",
  "ready",
  "passed",
  "failed",
  "blocked",
  "completed",
] as const;

export function validateFeedStoryMilestone(
  value: unknown,
  label = "feed_entry_milestone",
): FeedStoryMilestone {
  const record = requireRecord(value, label);
  const evidence = record.evidence === undefined
    ? undefined
    : requireArray(record.evidence, `${label}.evidence`).map((entry, index) =>
        validateEvidenceRef(entry, `${label}.evidence[${index}]`),
      );
  return {
    kind: requireEnum(record.kind, milestoneKinds, `${label}.kind`),
    status: optionalEnum(record.status, milestoneStatuses, `${label}.status`),
    title: sanitizePublicMarkdown(optionalString(record.title, `${label}.title`)),
    summary: sanitizePublicMarkdown(requireString(record.summary, `${label}.summary`)) ?? "",
    details: optionalStringArray(record.details, `${label}.details`)
      ?.map((entry) => sanitizePublicMarkdown(entry) ?? ""),
    evidence,
  };
}

export function validateFeedStory(value: unknown, label = "feed_entry"): FeedStory {
  const record = requireRecord(value, label);
  return {
    thread_locator: sanitizePublicMarkdown(requireString(record.thread_locator, `${label}.thread_locator`)) ?? "",
    title: sanitizePublicMarkdown(optionalString(record.title, `${label}.title`)),
    next_action: sanitizePublicMarkdown(optionalString(record.next_action, `${label}.next_action`)),
    milestones: requireArray(record.milestones, `${label}.milestones`).map((entry, index) =>
      validateFeedStoryMilestone(entry, `${label}.milestones[${index}]`),
    ),
  };
}

export function renderFeedStoryMarkdown(value: FeedStory | Readonly<Record<string, unknown>>): string {
  const story = validateFeedStory(value);
  const lines = [
    `## ${story.title ?? "Issue-to-PR story"}`,
    "",
    `Source thread: \`${story.thread_locator}\``,
    "",
    "### Gate Summary",
  ];

  for (const milestone of story.milestones) {
    const status = milestone.status ? ` (${milestone.status})` : "";
    lines.push(`- ${formatMilestoneKind(milestone.kind)}${status}: ${milestone.summary}`);
    for (const detail of milestone.details ?? []) {
      lines.push(`  - ${detail}`);
    }
  }

  if (story.next_action) {
    lines.push("", `Next: ${story.next_action}`);
  }

  return `${lines.join("\n")}\n`;
}

export function buildFeedStoryOutboxEntry(
  options: BuildFeedStoryOutboxEntryOptions,
): OutboxEntry {
  const milestone = validateFeedStoryMilestone(options.milestone);
  const workflow = normalizeIdentifierSegment(options.workflow ?? "issue-to-pr");
  const taskId = normalizeIdentifierSegment(options.taskId);
  const threadLocator = normalizeSourceThreadLocator(options.threadLocator);
  const outboxReceiptId = sanitizePublicMarkdown(options.outboxReceiptId) ?? stableStoryOutboxReceiptId({
    workflow,
    taskId,
    milestoneKind: milestone.kind,
    threadLocator,
  });
  const bodyMarkdown = sanitizePublicMarkdown(
    options.bodyMarkdown ?? renderFeedStoryMarkdown({
      thread_locator: threadLocator,
      title: options.title,
      milestones: [milestone],
    }),
  );

  return {
    entry_id: `message:${taskId}:${milestone.kind}`,
    kind: "message",
    title: sanitizePublicMarkdown(options.title ?? milestone.title ?? formatMilestoneKind(milestone.kind)),
    status: "proposed",
    thread_locator: threadLocator,
    metadata: {
      schema_version: "runx.outbox-entry.feed-entry.v1",
      workflow,
      milestone_kind: milestone.kind,
      outbox_receipt_id: outboxReceiptId,
      body_markdown: bodyMarkdown,
      updated_at: options.updatedAt,
      source_thread: {
        required: true,
        publish_mode: "reply",
        missing_behavior: "fail_closed",
        thread_locator: threadLocator,
      },
      control: {
        workflow,
        lane: milestone.kind,
      },
    },
  };
}

function stableStoryOutboxReceiptId(options: {
  readonly workflow: string;
  readonly taskId: string;
  readonly milestoneKind: FeedStoryMilestoneKind;
  readonly threadLocator: string;
}): string {
  const digest = createHash("sha256")
    .update(JSON.stringify([
      "runx.feed_entry",
      options.workflow,
      options.taskId,
      options.milestoneKind,
      options.threadLocator,
    ]))
    .digest("hex")
    .slice(0, 20);
  return `feed:${options.workflow}:${options.taskId}:${options.milestoneKind}:${digest}`;
}

export function renderIssueToPrReviewerMarkdown(
  options: RenderIssueToPrReviewerMarkdownOptions,
): string {
  const title = sanitizePublicMarkdown(options.title) ?? options.taskId;
  const sourceTitle = sanitizePublicMarkdown(options.sourceTitle);
  const sourceLocator = sanitizePublicMarkdown(options.sourceLocator);
  const branch = sanitizePublicMarkdown(options.branch);
  const base = sanitizePublicMarkdown(options.base);
  const reviewVerdict = sanitizePublicMarkdown(options.reviewVerdict);
  const handoff = summarizePublicHandoffMarkdown(options.handoffMarkdown);
  const lines = [
    `# ${title}`,
    "",
    "## Source Thread",
    `- Thread: ${sourceLocator ? `\`${sourceLocator}\`` : "not provided"}`,
    `- Request: ${sourceTitle ?? "not provided"}`,
    "",
    "## Scoped Change",
    `- Task: \`${sanitizePublicMarkdown(options.taskId) ?? options.taskId}\``,
    `- Branch: ${branch ? `\`${branch}\`` : "not reported"}`,
    `- Base: ${base ? `\`${base}\`` : "not reported"}`,
    `- Governance status: ${sanitizePublicMarkdown(options.governanceStatus) ?? "not reported"}`,
    "",
    "## Validation",
    `- scafld build: ${sanitizePublicMarkdown(options.checkStatus) ?? "not reported"}`,
    `- Passed: ${formatReportedNumber(options.buildPassed)}`,
    `- Failed: ${formatReportedNumber(options.buildFailed)}`,
    "",
    "## Review",
    `- Verdict: ${reviewVerdict ?? "not reported"}`,
    `- Blocking findings: ${formatReportedNumber(options.blockingCount)}`,
    `- Non-blocking findings: ${formatReportedNumber(options.nonBlockingCount)}`,
    "",
    "## Human Merge Gate",
    "- This PR is generated and reviewable; runx will not merge it.",
    "- A human reviewer must merge, close, or request changes.",
    "- After provider state changes, the source thread can be updated with the observed outcome.",
    "",
    "## scafld Handoff",
    handoff ?? "No scafld handoff was reported.",
  ];
  return `${lines.join("\n")}\n`;
}

export function summarizePublicHandoffMarkdown(value: string | undefined): string | undefined {
  const sanitized = sanitizePublicMarkdown(value);
  if (!sanitized) {
    return undefined;
  }

  const lines: string[] = [];
  let inFence = false;
  for (const rawLine of sanitized.split(/\r?\n/)) {
    const line = rawLine.trimEnd();
    if (line.trim().startsWith("```")) {
      inFence = !inFence;
      continue;
    }
    if (inFence || line.trim().length === 0) {
      continue;
    }
    if (isPublicHandoffSummaryLine(line)) {
      lines.push(line);
    }
    if (lines.length >= 12) {
      break;
    }
  }

  const summary = lines.join("\n").slice(0, 1200).trim();
  if (summary.length === 0) {
    return "Detailed handoff omitted from public markdown; run `scafld handoff` in the workspace for private evidence.";
  }
  const omitted = sanitized.length > summary.length || sanitized.split(/\r?\n/).length > lines.length;
  return omitted
    ? `${summary}\n\nDetailed handoff output omitted from public markdown; run \`scafld handoff\` in the workspace for private evidence.`
    : summary;
}

export function sanitizePublicMarkdown(value: string | undefined): string | undefined {
  if (value === undefined) {
    return undefined;
  }
  return value
    .replace(/\b([A-Z][A-Z0-9_]*(?:TOKEN|SECRET|PASSWORD|API[_-]?KEY|MATERIAL[_-]?REF)[A-Z0-9_]*)=("[^"]*"|'[^']*'|\S+)/gi, "$1=[secret]")
    .replace(/\b((?:bearer|authorization|token|secret|password|api[_-]?key|material[_-]?ref|materialRef)\s*[:=]\s*)(["']?)[^\s`),;]+/gi, "$1[secret]")
    .replace(/\b((?:bearer|authorization)\s+)[A-Za-z0-9._:-]{6,}\b/gi, "$1[secret]")
    .replace(/\b(gh[pousr]_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{20,})\b/g, "[secret]")
    .replace(/\bsk-(?:proj-)?[A-Za-z0-9_-]{16,}\b/g, "[secret]")
    .replace(/\b[A-Za-z0-9]+(?:[-_](?:secret|token|password|api[-_]?key))+[A-Za-z0-9_-]*\b(?!\s*=)/gi, "[secret]")
    .replace(/\b([A-Z][A-Z0-9_]*=)(?:\/Users|\/home|\/var|\/private|\/tmp|[A-Za-z]:\\)[^\s`)]+/g, "$1[local-path]")
    .replace(/(?:\/Users|\/home|\/var|\/private|\/tmp)\/[^\s`)]+/g, "[local-path]")
    .replace(/[A-Za-z]:\\[^\s`)]+/g, "[local-path]");
}

function isPublicHandoffSummaryLine(line: string): boolean {
  const trimmed = line.trim();
  return /^#{1,3}\s+Handoff:/i.test(trimmed)
    || /^Status:/i.test(trimmed)
    || /^Next:/i.test(trimmed)
    || /^Gate:/i.test(trimmed)
    || /^Review gate:/i.test(trimmed)
    || /^Current phase:/i.test(trimmed)
    || /^Blockers:/i.test(trimmed)
    || /^Allowed follow-up command:/i.test(trimmed)
    || /^- \[(?:pass|fail|pending)\]/i.test(trimmed);
}

function formatReportedNumber(value: number | undefined): string {
  return typeof value === "number" && Number.isFinite(value)
    ? String(value)
    : "not reported";
}

function normalizeIdentifierSegment(value: string): string {
  const normalized = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_.-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  if (normalized.length === 0) {
    throw new Error("feed entry identifier segment must not be empty.");
  }
  return normalized;
}

function normalizeSourceThreadLocator(value: string): string {
  const sanitized = sanitizePublicMarkdown(value)?.trim();
  if (!sanitized || sanitized === "thread:not-provided") {
    throw new Error("feed entry source thread locator is required.");
  }
  return sanitized;
}

function formatMilestoneKind(kind: FeedStoryMilestoneKind): string {
  switch (kind) {
    case "signal":
      return "Signal";
    case "decision":
      return "Decision";
    case "spec":
      return "Spec";
    case "build":
      return "Build";
    case "review":
      return "Review";
    case "pull_request":
      return "Pull request";
    case "merge_gate":
      return "Human merge gate";
    case "outcome":
      return "Outcome";
  }
}
