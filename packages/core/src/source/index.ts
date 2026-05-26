import { canonicalJsonStringify, sha256Prefixed } from "@runxhq/contracts";

export type RunxSourceProvider = "github" | "slack" | "sentry" | "file" | "api" | "manual" | "other";

export type RunxSourceKind =
  | "github_issue"
  | "github_pull_request"
  | "github_repo"
  | "slack_thread"
  | "sentry_issue"
  | "sentry_event"
  | "file"
  | "api"
  | "manual"
  | "unsupported";

export type RunxSourceCommandAction =
  | "issue-intake"
  | "issue-to-pr"
  | "work-plan"
  | "reply-only"
  | "manual-review"
  | "pr-review"
  | "pr-fix-up"
  | "merge-assist";

export type RunxCommandResponseStatus = "accepted" | "blocked" | "unsupported" | "failed" | "skipped";

export interface RunxParsedSourceLocator {
  readonly input: string;
  readonly provider: RunxSourceProvider;
  readonly kind: RunxSourceKind;
  readonly locator: string;
  readonly canonicalLocator: string;
  readonly threadLocator?: string;
  readonly sourceLocator: string;
  readonly targetRepo?: string;
  readonly issueUrl?: string;
  readonly pullRequestUrl?: string;
  readonly title?: string;
  readonly requiresHydration: boolean;
  readonly supported: boolean;
  readonly diagnostics: readonly string[];
  readonly github?: {
    readonly owner: string;
    readonly repo: string;
    readonly number?: number;
    readonly type: "issue" | "pull_request" | "repo";
  };
  readonly slack?: {
    readonly team?: string;
    readonly channel: string;
    readonly messageTs: string;
    readonly threadTs: string;
  };
  readonly sentry?: {
    readonly organization?: string;
    readonly issueId: string;
    readonly eventId?: string;
  };
}

export interface NormalizeRunxSourceCommandOptions {
  readonly action: RunxSourceCommandAction;
  readonly source: string | RunxParsedSourceLocator;
  readonly defaultTargetRepo?: string;
  readonly title?: string;
  readonly body?: string;
  readonly sourceId?: string;
}

export interface RunxSourceCommand {
  readonly action: RunxSourceCommandAction;
  readonly source: RunxParsedSourceLocator;
  readonly sourceId?: string;
  readonly targetRepo?: string;
  readonly sourceLocator: string;
  readonly threadLocator?: string;
  readonly dedupeKey: string;
  readonly requiresHydration: boolean;
  readonly supported: boolean;
  readonly diagnostics: readonly string[];
  readonly skillInputs: {
    readonly thread_title: string;
    readonly thread_body: string;
    readonly thread_locator?: string;
    readonly source_event: {
      readonly provider: RunxSourceProvider;
      readonly kind: RunxSourceKind;
      readonly source_locator: string;
      readonly thread_locator?: string;
      readonly target_repo?: string;
      readonly requires_hydration: boolean;
      readonly supported: boolean;
    };
  };
  readonly operationalPolicyRequest: {
    readonly source_id?: string;
    readonly target_repo?: string;
    readonly action: RunxSourceCommandAction;
    readonly source_thread_locator?: string;
  };
}

export interface BuildRunxCommandResponseOptions {
  readonly status: RunxCommandResponseStatus;
  readonly label?: string;
  readonly summary?: string;
  readonly nextAction?: string;
  readonly source?: RunxParsedSourceLocator | RunxSourceCommand;
  readonly issueUrl?: string;
  readonly pullRequestUrl?: string;
  readonly error?: unknown;
  readonly maxChars?: number;
}

const DEFAULT_RESPONSE_MAX_CHARS = 900;
const DEFAULT_TEXT_MAX_CHARS = 420;

export function parseRunxSourceLocator(input: string): RunxParsedSourceLocator {
  const trimmed = input.trim();
  if (trimmed.length === 0) {
    return unsupportedSource(input, "source locator is empty");
  }

  return parseGithubSource(trimmed)
    ?? parseSlackSource(trimmed)
    ?? parseSentrySource(trimmed)
    ?? parseSchemeSource(trimmed)
    ?? unsupportedSource(trimmed, "source provider is not supported by the core parser");
}

export function normalizeRunxSourceCommand(options: NormalizeRunxSourceCommandOptions): RunxSourceCommand {
  const source = typeof options.source === "string"
    ? parseRunxSourceLocator(options.source)
    : options.source;
  const targetRepo = source.targetRepo ?? normalizeRepoSlug(options.defaultTargetRepo);
  const threadTitle = sanitizeRunxCommandText(options.title ?? source.title ?? source.kind, 160) ?? source.kind;
  const threadBody = sanitizeRunxCommandText(options.body ?? source.locator, 4_000) ?? source.locator;
  const diagnostics = compactStrings([
    ...source.diagnostics,
    !source.supported ? "source is not dispatchable by this parser" : undefined,
    source.requiresHydration ? "provider context must be hydrated by an adapter before mutation" : undefined,
  ]);

  return {
    action: options.action,
    source,
    sourceId: options.sourceId,
    targetRepo,
    sourceLocator: source.sourceLocator,
    threadLocator: source.threadLocator,
    dedupeKey: buildRunxSourceDedupeKey({
      action: options.action,
      source,
      targetRepo,
    }),
    requiresHydration: source.requiresHydration,
    supported: source.supported,
    diagnostics,
    skillInputs: {
      thread_title: threadTitle,
      thread_body: threadBody,
      thread_locator: source.threadLocator,
      source_event: {
        provider: source.provider,
        kind: source.kind,
        source_locator: source.sourceLocator,
        thread_locator: source.threadLocator,
        target_repo: targetRepo,
        requires_hydration: source.requiresHydration,
        supported: source.supported,
      },
    },
    operationalPolicyRequest: {
      source_id: options.sourceId,
      target_repo: targetRepo,
      action: options.action,
      source_thread_locator: source.threadLocator,
    },
  };
}

export function buildRunxSourceDedupeKey(input: {
  readonly action: RunxSourceCommandAction;
  readonly source: RunxParsedSourceLocator;
  readonly targetRepo?: string;
}): string {
  return sha256Prefixed(`runx:source:${canonicalJsonStringify({
    action: input.action,
    provider: input.source.provider,
    kind: input.source.kind,
    source_locator: input.source.sourceLocator,
    thread_locator: input.source.threadLocator ?? null,
    target_repo: normalizeRepoSlug(input.targetRepo) ?? null,
  })}`);
}

export function buildRunxCommandResponse(options: BuildRunxCommandResponseOptions): string {
  const maxChars = options.maxChars ?? DEFAULT_RESPONSE_MAX_CHARS;
  const label = sanitizeRunxCommandText(options.label, 80) ?? "Runx";
  const summary = sanitizeRunxCommandText(options.summary, DEFAULT_TEXT_MAX_CHARS);
  const nextAction = sanitizeRunxCommandText(options.nextAction, DEFAULT_TEXT_MAX_CHARS);
  const source = options.source ? sourceFromResponseInput(options.source) : undefined;
  const error = sanitizeRunxCommandText(errorToMessage(options.error), DEFAULT_TEXT_MAX_CHARS);
  const issueUrl = sanitizeRunxCommandText(options.issueUrl ?? source?.issueUrl, 2_000);
  const pullRequestUrl = sanitizeRunxCommandText(options.pullRequestUrl ?? source?.pullRequestUrl, 2_000);
  const diagnostics = source?.diagnostics
    .map((diagnostic) => sanitizeRunxCommandText(diagnostic, 180))
    .filter((diagnostic): diagnostic is string => Boolean(diagnostic));
  const lines = compactStrings([
    `${label}: ${options.status}`,
    summary,
    source ? `Source: ${source.provider}/${source.kind}` : undefined,
    issueUrl ? `Issue: ${issueUrl}` : undefined,
    pullRequestUrl ? `PR: ${pullRequestUrl}` : undefined,
    error ? `Blocker: ${error}` : undefined,
    diagnostics && diagnostics.length > 0 ? `Notes: ${diagnostics.join("; ")}` : undefined,
    nextAction ? `Next: ${nextAction}` : undefined,
  ]);
  return truncateText(lines.join("\n"), maxChars);
}

export function sanitizeRunxCommandText(value: unknown, maxChars = DEFAULT_TEXT_MAX_CHARS): string | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  const parsed = summarizeStructuredError(value);
  const text = (parsed ?? String(value))
    .replace(/\r\n/g, "\n")
    .replace(/\r/g, "\n")
    .replace(/<!--/g, "&lt;!--")
    .replace(/-->/g, "--&gt;")
    .replace(/\b(SENTRY_AUTH_TOKEN|GITHUB_TOKEN|GH_TOKEN|SLACK_BOT_TOKEN)=\S+/g, "$1=[redacted token]")
    .replace(/\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{20,}\b/g, "[redacted token]")
    .replace(/\bgithub_pat_[A-Za-z0-9_]{20,}\b/g, "[redacted token]")
    .replace(/\bxox[abprs]-[A-Za-z0-9-]{12,}\b/g, "[redacted token]")
    .replace(/\bnskey_(?:live|test)_[A-Fa-f0-9]{16,}\b/g, "[redacted token]")
    .replace(/\bBearer\s+[A-Za-z0-9._~+/=-]{20,}\b/gi, "Bearer [redacted token]")
    .replace(/(^|[\s=(])(?:\/Users|\/private\/tmp|\/tmp|\/home)\/[^\s)`'"]+/g, "$1[local path]")
    .replace(/(^|[\s=(])[A-Za-z]:\\[^\s)`'"]+/g, "$1[local path]")
    .trim();
  if (text.length === 0) {
    return undefined;
  }
  return truncateText(text, maxChars);
}

function parseGithubSource(input: string): RunxParsedSourceLocator | undefined {
  const url = parseUrl(input);
  const fromUrl = url && isGithubHost(url.hostname)
    ? parseGithubPath(url.pathname)
    : undefined;
  const fromLocator = url?.protocol === "github:"
    ? parseGithubPath(`/${url.hostname}${url.pathname}`)
    : undefined;
  const parsed = fromUrl ?? fromLocator;
  if (!parsed) {
    return undefined;
  }

  const targetRepo = `${parsed.owner}/${parsed.repo}`;
  const repoLocator = `github://${targetRepo}`;
  const numberSegment = parsed.type === "repo" ? "" : `/${parsed.type === "issue" ? "issues" : "pulls"}/${parsed.number}`;
  const canonicalLocator = `${repoLocator}${numberSegment}`;
  const issueUrl = parsed.type === "issue"
    ? `https://github.com/${targetRepo}/issues/${parsed.number}`
    : undefined;
  const pullRequestUrl = parsed.type === "pull_request"
    ? `https://github.com/${targetRepo}/pull/${parsed.number}`
    : undefined;
  return {
    input,
    provider: "github",
    kind: parsed.type === "issue"
      ? "github_issue"
      : parsed.type === "pull_request"
        ? "github_pull_request"
        : "github_repo",
    locator: input,
    canonicalLocator,
    threadLocator: canonicalLocator,
    sourceLocator: canonicalLocator,
    targetRepo,
    issueUrl,
    pullRequestUrl,
    requiresHydration: parsed.type !== "repo",
    supported: true,
    diagnostics: [],
    github: parsed,
    title: parsed.type === "repo"
      ? targetRepo
      : `${targetRepo}#${parsed.number}`,
  };
}

function parseGithubPath(pathname: string): {
  readonly owner: string;
  readonly repo: string;
  readonly number?: number;
  readonly type: "issue" | "pull_request" | "repo";
} | undefined {
  const segments = pathname.split("/").filter(Boolean);
  const owner = segments[0];
  const repo = segments[1];
  if (!owner || !repo || !validSlugPart(owner) || !validSlugPart(repo)) {
    return undefined;
  }
  if (segments.length === 2) {
    return { owner, repo, type: "repo" };
  }
  const number = parsePositiveInteger(segments[3]);
  if (segments[2] === "issues" && number !== undefined) {
    return { owner, repo, number, type: "issue" };
  }
  if ((segments[2] === "pull" || segments[2] === "pulls") && number !== undefined) {
    return { owner, repo, number, type: "pull_request" };
  }
  return undefined;
}

function parseSlackSource(input: string): RunxParsedSourceLocator | undefined {
  const url = parseUrl(input);
  const fromPermalink = url && url.protocol.startsWith("http") && url.hostname.endsWith(".slack.com")
    ? parseSlackPermalink(url)
    : undefined;
  const fromLocator = url?.protocol === "slack:"
    ? parseSlackLocator(url)
    : undefined;
  const parsed = fromPermalink ?? fromLocator;
  if (!parsed) {
    return undefined;
  }

  const teamSegment = parsed.team ? `team/${encodeLocatorSegment(parsed.team)}/` : "";
  const canonicalLocator =
    `slack://${teamSegment}channel/${encodeLocatorSegment(parsed.channel)}/thread/${parsed.threadTs}`;
  return {
    input,
    provider: "slack",
    kind: "slack_thread",
    locator: input,
    canonicalLocator,
    threadLocator: canonicalLocator,
    sourceLocator: canonicalLocator,
    requiresHydration: true,
    supported: true,
    diagnostics: ["slack thread body must be hydrated by a Slack adapter"],
    slack: parsed,
    title: `Slack ${parsed.channel} ${parsed.threadTs}`,
  };
}

function parseSlackPermalink(url: URL): {
  readonly team?: string;
  readonly channel: string;
  readonly messageTs: string;
  readonly threadTs: string;
} | undefined {
  const match = /^\/archives\/([^/]+)\/p(\d{10,})(?:$|[/?#])/.exec(url.pathname);
  if (!match) {
    return undefined;
  }
  const messageTs = slackTsFromPermalink(match[2]);
  if (!messageTs) {
    return undefined;
  }
  const threadTs = normalizeSlackTs(url.searchParams.get("thread_ts")) ?? messageTs;
  return {
    team: url.hostname.slice(0, -".slack.com".length) || undefined,
    channel: match[1],
    messageTs,
    threadTs,
  };
}

function parseSlackLocator(url: URL): {
  readonly team?: string;
  readonly channel: string;
  readonly messageTs: string;
  readonly threadTs: string;
} | undefined {
  const segments = [url.hostname, ...url.pathname.split("/")].filter(Boolean);
  if (segments[0] === "team" && segments[2] === "channel" && segments[4] === "thread") {
    const threadTs = normalizeSlackTs(segments[5]);
    return segments[1] && segments[3] && threadTs
      ? { team: segments[1], channel: segments[3], messageTs: threadTs, threadTs }
      : undefined;
  }
  if (segments[1] === "channel" && segments[3] === "thread") {
    const threadTs = normalizeSlackTs(segments[4]);
    return segments[0] && segments[2] && threadTs
      ? { team: segments[0], channel: segments[2], messageTs: threadTs, threadTs }
      : undefined;
  }
  const legacyThreadTs = normalizeSlackTs(segments[2]);
  if (segments[0] && segments[1] && legacyThreadTs) {
    return { team: segments[0], channel: segments[1], messageTs: legacyThreadTs, threadTs: legacyThreadTs };
  }
  return undefined;
}

function parseSentrySource(input: string): RunxParsedSourceLocator | undefined {
  const url = parseUrl(input);
  const parsed = url?.protocol === "sentry:"
    ? parseSentryLocator(url)
    : url && url.protocol.startsWith("http") && isSentryHost(url.hostname)
      ? parseSentryUrl(url)
      : undefined;
  if (!parsed) {
    return undefined;
  }

  const orgSegment = parsed.organization ? `${encodeLocatorSegment(parsed.organization)}/` : "";
  const canonicalLocator = parsed.eventId
    ? `sentry://${orgSegment}issues/${encodeLocatorSegment(parsed.issueId)}/events/${encodeLocatorSegment(parsed.eventId)}`
    : `sentry://${orgSegment}issues/${encodeLocatorSegment(parsed.issueId)}`;
  return {
    input,
    provider: "sentry",
    kind: parsed.eventId ? "sentry_event" : "sentry_issue",
    locator: input,
    canonicalLocator,
    threadLocator: canonicalLocator,
    sourceLocator: canonicalLocator,
    requiresHydration: true,
    supported: true,
    diagnostics: ["sentry issue payload must be hydrated and redacted by a Sentry adapter"],
    sentry: parsed,
    title: `Sentry issue ${parsed.issueId}`,
  };
}

function parseSentryUrl(url: URL): {
  readonly organization?: string;
  readonly issueId: string;
  readonly eventId?: string;
} | undefined {
  const segments = url.pathname.split("/").filter(Boolean);
  const orgIndex = segments[0] === "organizations" ? 1 : undefined;
  const issueIndex = segments.indexOf("issues");
  const issueId = issueIndex >= 0 ? segments[issueIndex + 1] : undefined;
  if (!issueId) {
    return undefined;
  }
  const eventIndex = segments.indexOf("events");
  const organization = orgIndex !== undefined
    ? segments[orgIndex]
    : url.hostname.endsWith(".sentry.io")
      ? url.hostname.slice(0, -".sentry.io".length)
      : undefined;
  return {
    organization,
    issueId,
    eventId: eventIndex >= 0 ? segments[eventIndex + 1] : undefined,
  };
}

function parseSentryLocator(url: URL): {
  readonly organization?: string;
  readonly issueId: string;
  readonly eventId?: string;
} | undefined {
  const segments = [url.hostname, ...url.pathname.split("/")].filter(Boolean);
  const issueIndex = segments.indexOf("issues");
  const issueId = issueIndex >= 0 ? segments[issueIndex + 1] : undefined;
  if (!issueId) {
    return undefined;
  }
  const eventIndex = segments.indexOf("events");
  return {
    organization: issueIndex > 0 ? segments[0] : undefined,
    issueId,
    eventId: eventIndex >= 0 ? segments[eventIndex + 1] : undefined,
  };
}

function parseSchemeSource(input: string): RunxParsedSourceLocator | undefined {
  const url = parseUrl(input);
  if (!url) {
    return undefined;
  }
  const provider = providerFromProtocol(url.protocol);
  if (!provider) {
    return undefined;
  }
  const canonicalLocator = input;
  return {
    input,
    provider,
    kind: provider,
    locator: input,
    canonicalLocator,
    threadLocator: provider === "manual" ? undefined : canonicalLocator,
    sourceLocator: canonicalLocator,
    requiresHydration: provider === "api",
    supported: true,
    diagnostics: provider === "api" ? ["api source payload must be supplied by the adapter"] : [],
    title: `${provider} source`,
  };
}

function providerFromProtocol(protocol: string): "file" | "api" | "manual" | undefined {
  if (protocol === "file:") {
    return "file";
  }
  if (protocol === "api:") {
    return "api";
  }
  if (protocol === "manual:") {
    return "manual";
  }
  return undefined;
}

function unsupportedSource(input: string, reason: string): RunxParsedSourceLocator {
  const safeInput = sanitizeRunxCommandText(input, 240) ?? "unknown";
  return {
    input,
    provider: "other",
    kind: "unsupported",
    locator: safeInput,
    canonicalLocator: `unsupported://${sha256Prefixed(`locator:${safeInput}`)}`,
    sourceLocator: `unsupported://${sha256Prefixed(`locator:${safeInput}`)}`,
    requiresHydration: false,
    supported: false,
    diagnostics: [reason],
  };
}

function sourceFromResponseInput(input: RunxParsedSourceLocator | RunxSourceCommand): RunxParsedSourceLocator {
  return "source" in input ? input.source : input;
}

function errorToMessage(error: unknown): string | undefined {
  if (error === undefined || error === null) {
    return undefined;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

function summarizeStructuredError(value: unknown): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }
  const trimmed = value.trim();
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) {
    return undefined;
  }
  try {
    const parsed = JSON.parse(trimmed) as unknown;
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      const record = parsed as Record<string, unknown>;
      const message = stringField(record, "message")
        ?? stringField(record, "error")
        ?? stringField(record, "summary")
        ?? stringField(record, "detail");
      return message ?? "provider returned a structured error";
    }
    return "provider returned a structured payload";
  } catch {
    return undefined;
  }
}

function parseUrl(input: string): URL | undefined {
  try {
    return new URL(input);
  } catch {
    return undefined;
  }
}

function isGithubHost(hostname: string): boolean {
  return hostname === "github.com" || hostname === "www.github.com";
}

function isSentryHost(hostname: string): boolean {
  return hostname === "sentry.io" || hostname.endsWith(".sentry.io");
}

function slackTsFromPermalink(value: string): string | undefined {
  if (!/^\d{11,}$/.test(value)) {
    return undefined;
  }
  return `${value.slice(0, 10)}.${value.slice(10).padEnd(6, "0").slice(0, 6)}`;
}

function normalizeSlackTs(value: string | null | undefined): string | undefined {
  if (!value) {
    return undefined;
  }
  if (/^\d{10}\.\d{1,6}$/.test(value)) {
    const [seconds, micros] = value.split(".");
    return `${seconds}.${micros.padEnd(6, "0").slice(0, 6)}`;
  }
  return slackTsFromPermalink(value);
}

function parsePositiveInteger(value: string | undefined): number | undefined {
  if (!value || !/^\d+$/.test(value)) {
    return undefined;
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : undefined;
}

function validSlugPart(value: string): boolean {
  return /^[A-Za-z0-9_.-]+$/.test(value);
}

function normalizeRepoSlug(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(trimmed) ? trimmed : undefined;
}

function compactStrings(values: readonly (string | undefined)[]): string[] {
  return values.filter((value): value is string => Boolean(value && value.trim().length > 0));
}

function truncateText(value: string, maxChars: number): string {
  const limit = Math.max(16, maxChars);
  return value.length > limit ? `${value.slice(0, limit - 3).trimEnd()}...` : value;
}

function encodeLocatorSegment(value: string): string {
  return encodeURIComponent(value);
}

function stringField(record: Record<string, unknown>, key: string): string | undefined {
  const value = record[key];
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : undefined;
}
