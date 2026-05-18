export const actAssignmentSdkPackage = "@runxhq/runtime-local/sdk/act-assignment";

import {
  validateActAssignmentContract,
  type ActAssignmentActorContract,
  type ActAssignmentContract,
  type ActAssignmentHostContract,
} from "@runxhq/contracts";
import { hashStable, isRecord } from "@runxhq/core/util";

type JsonRecord = Readonly<Record<string, unknown>>;

export interface BuildActAssignmentOptions {
  readonly skillRef: string;
  readonly runner: string;
  readonly sourceRef?: string;
  readonly requestedAt?: string;
  readonly hostKind?: ActAssignmentHostContract["kind"];
  readonly triggerRef?: string;
  readonly scopeSet?: readonly string[];
  readonly actor?: ActAssignmentActorContract;
  readonly inputOverrides?: JsonRecord;
}

export function buildActAssignment(options: BuildActAssignmentOptions): ActAssignmentContract {
  const inputOverrides = normalizeActAssignmentRecord(options.inputOverrides);
  const host = normalizeActAssignmentHost({
    kind: options.hostKind ?? "cli",
    trigger_ref: options.triggerRef,
    scope_set: options.scopeSet,
    actor: options.actor,
  });
  const sourceRef = normalizeNonEmptyString(options.sourceRef);
  const requestedAt = normalizeNonEmptyString(options.requestedAt) ?? new Date().toISOString();

  return validateActAssignmentContract(pruneUndefined({
    schema: "runx.act_assignment.v1",
    skill_ref: options.skillRef,
    runner: options.runner,
    source_ref: sourceRef,
    requested_at: requestedAt,
    host,
    input_overrides: inputOverrides,
    idempotency: {
      algorithm: "sha256",
      intent_key: deriveActAssignmentIntentKey({
        skillRef: options.skillRef,
        runner: options.runner,
        sourceRef,
        inputOverrides,
      }),
      trigger_key: deriveActAssignmentTriggerKey({
        hostKind: host.kind,
        triggerRef: host.trigger_ref,
      }),
      content_hash: deriveActAssignmentContentHash(inputOverrides),
    },
  }));
}

export function deriveActAssignmentIntentKey(options: {
  readonly skillRef: string;
  readonly runner: string;
  readonly sourceRef?: string;
  readonly inputOverrides?: JsonRecord;
}): string {
  return withSha256Prefix(hashStable({
    skill_ref: options.skillRef,
    runner: options.runner,
    source_ref: normalizeNonEmptyString(options.sourceRef),
    input_overrides: normalizeActAssignmentRecord(options.inputOverrides),
  }));
}

export function deriveActAssignmentTriggerKey(options: {
  readonly hostKind: ActAssignmentHostContract["kind"];
  readonly triggerRef?: string;
}): string | undefined {
  const triggerRef = normalizeNonEmptyString(options.triggerRef);
  if (!triggerRef) {
    return undefined;
  }
  return withSha256Prefix(hashStable({
    host_kind: options.hostKind,
    trigger_ref: triggerRef,
  }));
}

export function deriveActAssignmentContentHash(inputOverrides?: JsonRecord): string {
  return withSha256Prefix(hashStable(normalizeActAssignmentRecord(inputOverrides) ?? {}));
}

export function normalizeActAssignmentRecord(value: unknown): JsonRecord | undefined {
  if (!isRecord(value)) {
    return undefined;
  }
  const normalized = normalizeUnknown(value);
  return isRecord(normalized) && Object.keys(normalized).length > 0 ? normalized : undefined;
}

function normalizeActAssignmentHost(value: {
  readonly kind: ActAssignmentHostContract["kind"];
  readonly trigger_ref?: string;
  readonly scope_set?: readonly string[];
  readonly actor?: ActAssignmentActorContract;
}): ActAssignmentHostContract {
  const actor = normalizeActAssignmentActor(value.actor);
  const scopeSet = normalizeStringArray(value.scope_set);
  return pruneUndefined({
    kind: value.kind,
    trigger_ref: normalizeNonEmptyString(value.trigger_ref),
    scope_set: scopeSet.length > 0 ? scopeSet : undefined,
    actor,
  }) as ActAssignmentHostContract;
}

function normalizeActAssignmentActor(value: ActAssignmentActorContract | undefined): ActAssignmentActorContract | undefined {
  if (!value) {
    return undefined;
  }
  const actor = {
    actor_id: normalizeNonEmptyString(value.actor_id),
    display_name: normalizeNonEmptyString(value.display_name),
    role: normalizeNonEmptyString(value.role),
    provider_identity: normalizeNonEmptyString(value.provider_identity),
  };
  return Object.values(actor).some((entry) => typeof entry === "string" && entry.length > 0)
    ? pruneUndefined(actor) as ActAssignmentActorContract
    : undefined;
}

function normalizeStringArray(value: readonly string[] | undefined): readonly string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((entry) => normalizeNonEmptyString(entry))
    .filter((entry): entry is string => typeof entry === "string");
}

function normalizeUnknown(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map((entry) => normalizeUnknown(entry));
  }
  if (!isRecord(value)) {
    return value;
  }
  const normalized: Record<string, unknown> = {};
  for (const [key, entry] of Object.entries(value)) {
    if (entry === undefined) {
      continue;
    }
    const normalizedEntry = normalizeUnknown(entry);
    if (normalizedEntry === undefined) {
      continue;
    }
    normalized[key] = normalizedEntry;
  }
  return normalized;
}


function withSha256Prefix(hash: string): string {
  return `sha256:${hash}`;
}

function normalizeNonEmptyString(value: unknown): string | undefined {
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

function pruneUndefined<T>(value: T): T {
  if (Array.isArray(value)) {
    return value.map((entry) => pruneUndefined(entry)) as T;
  }
  if (!isRecord(value)) {
    return value;
  }
  const result: Record<string, unknown> = {};
  for (const [key, entry] of Object.entries(value)) {
    if (entry === undefined) {
      continue;
    }
    result[key] = pruneUndefined(entry);
  }
  return result as T;
}
