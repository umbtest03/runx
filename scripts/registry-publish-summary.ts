interface HarnessSummary {
  readonly status: string;
  readonly case_count: number;
  readonly assertion_error_count: number;
  readonly assertion_errors?: readonly string[];
  readonly case_names?: readonly string[];
  readonly receipt_ids?: readonly string[];
}

interface RegistryRecordSummary {
  readonly skill_id: string;
  readonly version: string;
  readonly digest: string;
  readonly profile_digest?: string;
}

interface HostedPublishPayload {
  readonly status?: string;
  readonly publish?: {
    readonly status?: string;
    readonly skill_id?: string;
    readonly version?: string;
    readonly digest?: string;
    readonly profile_digest?: string | null;
    readonly registry_url?: string;
  };
}

export type RegistryPublishStatus = "published" | "already_published" | "dry_run";

export function compactHarnessSummary(harness: HarnessSummary): {
  readonly status: string;
  readonly case_count: number;
  readonly assertion_error_count: number;
  readonly case_names: readonly string[];
  readonly receipt_ids: readonly string[];
} {
  return {
    status: harness.status,
    case_count: harness.case_count,
    assertion_error_count: harness.assertion_error_count,
    case_names: harness.case_names ?? [],
    receipt_ids: harness.receipt_ids ?? [],
  };
}

export function compactPublishSummary(input: {
  readonly status: RegistryPublishStatus;
  readonly record: RegistryRecordSummary;
  readonly harness: HarnessSummary;
  readonly apiBaseUrl?: string;
  readonly sourcePath?: string;
  readonly hostedBody?: string;
}): Readonly<Record<string, unknown>> {
  const hosted = parseHostedPublishPayload(input.hostedBody);
  const hostedPublish = hosted?.publish;
  return {
    status: hostedPublish?.status === "unchanged" ? "already_published" : input.status,
    skill_id: hostedPublish?.skill_id ?? input.record.skill_id,
    version: hostedPublish?.version ?? input.record.version,
    digest: hostedPublish?.digest ?? input.record.digest,
    profile_digest: hostedPublish?.profile_digest ?? input.record.profile_digest,
    source_path: input.sourcePath ? publicRepoPath(input.sourcePath) : undefined,
    harness: compactHarnessSummary(input.harness),
    registry_url: hostedPublish?.registry_url ?? registryUrlForRecord(input.apiBaseUrl, input.record),
  };
}

export async function hostedSkillMatchesPublishedState(
  apiBaseUrl: string,
  record: RegistryRecordSummary,
): Promise<boolean> {
  const [owner, name] = record.skill_id.split("/", 2);
  if (!owner || !name) {
    return false;
  }
  const versionedName = `${name}@${record.version}`;
  const response = await fetch(`${apiBaseUrl.replace(/\/$/, "")}/v1/skills/${encodeURIComponent(owner)}/${encodeURIComponent(versionedName)}`);
  if (!response.ok) {
    return false;
  }
  const payload = await response.json() as {
    skill?: {
      version?: string;
      digest?: string;
      profile_digest?: string | null;
    };
  };
  return payload.skill?.version === record.version
    && payload.skill?.digest === record.digest
    && (payload.skill?.profile_digest ?? undefined) === record.profile_digest;
}

export function compactHttpFailure(responseStatus: number, body: string): string {
  const parsed = parseJsonObject(body);
  const error = typeof parsed?.error === "string" ? parsed.error : undefined;
  if (error) {
    return `HTTP ${responseStatus}: ${error}`;
  }
  return `HTTP ${responseStatus}: response_body_bytes=${Buffer.byteLength(body, "utf8")}`;
}

function registryUrlForRecord(apiBaseUrl: string | undefined, record: RegistryRecordSummary): string | undefined {
  if (!apiBaseUrl) {
    return undefined;
  }
  const [owner, name] = record.skill_id.split("/", 2);
  if (!owner || !name) {
    return undefined;
  }
  return `${apiBaseUrl.replace(/\/$/, "")}/v1/skills/${encodeURIComponent(owner)}/${encodeURIComponent(`${name}@${record.version}`)}`;
}

function publicRepoPath(rawPath: string): string {
  const normalized = rawPath.replace(/\\/g, "/");
  const match = normalized.match(/(?:^|\/)(oss\/(?:skills|bindings|fixtures)\/.+)$/);
  if (match?.[1]) {
    return match[1];
  }
  return normalized.split("/").filter(Boolean).slice(-2).join("/");
}

function parseHostedPublishPayload(body: string | undefined): HostedPublishPayload | undefined {
  if (!body?.trim()) {
    return undefined;
  }
  const parsed = parseJsonObject(body);
  return parsed as HostedPublishPayload | undefined;
}

function parseJsonObject(body: string): Record<string, unknown> | undefined {
  try {
    const parsed = JSON.parse(body) as unknown;
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : undefined;
  } catch {
    return undefined;
  }
}
