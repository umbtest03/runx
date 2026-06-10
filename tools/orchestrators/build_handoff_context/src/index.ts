import { defineTool, failure, isRecord, jsonInput, prune, recordInput, stringInput } from "@runxhq/authoring";

const PLATFORM_CONFIG = {
  n8n: {
    scope: "orchestrator.n8n.workflow.invoke",
    audiencePrefix: "n8n:workflow:",
  },
  zapier: {
    scope: "orchestrator.zapier.workflow.invoke",
    audiencePrefix: "zapier:zap:",
  },
} as const;

const EVENT_ID_PATTERN = /^[A-Za-z0-9._:-]{3,200}$/;
const SENSITIVE_KEY_PATTERN = /^(authorization|bearer|password|secret|token|access_token|refresh_token|api_key|apikey|private_key|client_secret)$/i;

export default defineTool({
  name: "orchestrators.build_handoff_context",
  description: "Validate and normalize an orchestrator handoff execution context before n8n, Zapier, or another workflow host receives it.",
  inputs: {
    platform: stringInput({ description: "Workflow host receiving the handoff: n8n or zapier." }),
    event_id: stringInput({ description: "Stable event id used for receiver-side dedupe." }),
    handoff_scope: stringInput({ description: "Expected platform-specific handoff scope." }),
    handoff_audience: stringInput({ description: "Expected receiver audience, such as n8n:workflow:runx-governed-effect." }),
    execution_context: recordInput({ description: "Explicit caller/workflow context for the handoff." }),
    payload: jsonInput({ description: "Business payload to deliver to the workflow host." }),
    source: stringInput({ default: "runx", description: "Human-readable source label." }),
    idempotency_key: stringInput({ optional: true, description: "Optional explicit idempotency key. Defaults to event_id." }),
    receiver: recordInput({ optional: true, description: "Optional receiver metadata such as workflow id, endpoint ref, or support owner." }),
  },
  output: {
    wrap_as: "handoff_context",
  },
  scopes: ["orchestrator.handoff.prepare"],
  run({ inputs }) {
    const platform = normalizePlatform(inputs.platform);
    if (!platform) {
      return stop("unsupported_platform", `Unsupported orchestrator platform: ${inputs.platform}`);
    }
    const config = PLATFORM_CONFIG[platform];
    const eventId = normalizeText(inputs.event_id);
    const scope = normalizeText(inputs.handoff_scope);
    const audience = normalizeText(inputs.handoff_audience);
    const idempotencyKey = normalizeText(inputs.idempotency_key) || eventId;
    const source = normalizeText(inputs.source) || "runx";
    const executionContext = inputs.execution_context;
    const payload = inputs.payload;
    const receiver = inputs.receiver;

    const errors = [
      ...validateEventId(eventId),
      ...validateScope(scope, config.scope),
      ...validateAudience(audience, config.audiencePrefix),
      ...validateExecutionContext(executionContext, {
        platform,
        eventId,
        idempotencyKey,
        scope,
        audience,
      }),
      ...validatePayload(payload),
      ...validateReceiver(receiver),
    ];
    const sensitiveKeys = [
      ...findSensitiveKeys(executionContext, "execution_context"),
      ...findSensitiveKeys(payload, "payload"),
    ];
    if (sensitiveKeys.length > 0) {
      errors.push(`raw credential-like keys are not allowed in handoff material: ${sensitiveKeys.slice(0, 8).join(", ")}`);
    }
    if (errors.length > 0) {
      return stop("invalid_handoff_context", errors.join("; "), {
        platform,
        event_id: eventId || undefined,
        handoff_scope: scope || undefined,
        handoff_audience: audience || undefined,
      });
    }

    return prune({
      status: "ready",
      platform,
      event_id: eventId,
      idempotency: {
        key: idempotencyKey,
        receiver_should_dedupe: true,
      },
      handoff: {
        scope,
        audience,
        source,
      },
      receiver,
      execution_context: {
        ...executionContext,
        platform: executionContext.platform ?? platform,
        event_id: executionContext.event_id ?? eventId,
        idempotency_key: executionContext.idempotency_key ?? idempotencyKey,
        handoff_scope: executionContext.handoff_scope ?? scope,
        handoff_audience: executionContext.handoff_audience ?? audience,
      },
      payload,
      receiver_validation: {
        require_bearer: true,
        require_scope: scope,
        require_audience: audience,
        require_event_id: eventId,
        reject_duplicate_event_id: true,
      },
      receipt_expectations: {
        context_artifact: "handoff_context",
        outbound_effect_must_be_receipted: true,
        receiver_response_must_be_captured: true,
        raw_secrets_in_payload: false,
      },
      stop_conditions: [],
    });
  },
});

type Platform = keyof typeof PLATFORM_CONFIG;

function normalizePlatform(value: string | undefined): Platform | undefined {
  const normalized = normalizeText(value).toLowerCase();
  return normalized === "n8n" || normalized === "zapier" ? normalized : undefined;
}

function normalizeText(value: unknown): string {
  return typeof value === "string" ? value.trim() : "";
}

function validateEventId(eventId: string): string[] {
  if (!eventId) {
    return ["event_id is required"];
  }
  return EVENT_ID_PATTERN.test(eventId)
    ? []
    : ["event_id must be 3-200 characters and contain only letters, numbers, dot, underscore, colon, or dash"];
}

function validateScope(scope: string, expected: string): string[] {
  if (!scope) {
    return ["handoff_scope is required"];
  }
  return scope === expected ? [] : [`handoff_scope must be ${expected}`];
}

function validateAudience(audience: string, prefix: string): string[] {
  if (!audience) {
    return ["handoff_audience is required"];
  }
  if (!audience.startsWith(prefix) || audience.length <= prefix.length) {
    return [`handoff_audience must start with ${prefix} and include a receiver id`];
  }
  if (/[\s{}]/u.test(audience)) {
    return ["handoff_audience must not contain whitespace or template braces"];
  }
  return [];
}

function validateExecutionContext(
  context: Readonly<Record<string, unknown>>,
  expected: {
    readonly platform: Platform;
    readonly eventId: string;
    readonly idempotencyKey: string;
    readonly scope: string;
    readonly audience: string;
  },
): string[] {
  const errors: string[] = [];
  if (!isRecord(context)) {
    return ["execution_context must be an object"];
  }
  const originKeys = [
    "caller",
    "caller_id",
    "workflow",
    "workflow_id",
    "workflow_ref",
    "source_workflow",
    "upstream_execution_id",
    "upstream_run_id",
    "principal",
    "principal_id",
  ];
  if (!originKeys.some((key) => context[key] !== undefined && context[key] !== "")) {
    errors.push("execution_context must identify the caller, workflow, principal, or upstream run");
  }
  checkContextField(errors, context, "platform", expected.platform);
  checkContextField(errors, context, "event_id", expected.eventId);
  checkContextField(errors, context, "idempotency_key", expected.idempotencyKey);
  checkContextField(errors, context, "handoff_scope", expected.scope);
  checkContextField(errors, context, "handoff_audience", expected.audience);
  return errors;
}

function validatePayload(payload: unknown): string[] {
  if (payload === undefined || payload === null || payload === "") {
    return ["payload is required"];
  }
  return [];
}

function validateReceiver(receiver: Readonly<Record<string, unknown>> | undefined): string[] {
  if (!receiver) {
    return [];
  }
  const url = normalizeText(receiver.url);
  if (!url) {
    return [];
  }
  if (!url.startsWith("https://")) {
    return ["receiver.url must be HTTPS when provided"];
  }
  try {
    const parsed = new URL(url);
    const host = parsed.hostname.toLowerCase();
    if (host === "localhost" || host.endsWith(".localhost") || host === "127.0.0.1" || host === "0.0.0.0" || host === "::1") {
      return ["receiver.url must not be loopback"];
    }
  } catch {
    return ["receiver.url must be a valid HTTPS URL when provided"];
  }
  return [];
}

function checkContextField(
  errors: string[],
  context: Readonly<Record<string, unknown>>,
  key: string,
  expected: string,
): void {
  if (context[key] === undefined || context[key] === null || context[key] === "") {
    return;
  }
  const actual = normalizeText(context[key]);
  if (actual !== expected) {
    errors.push(`execution_context.${key} must match ${expected}`);
  }
}

function findSensitiveKeys(value: unknown, path: string, hits: string[] = []): string[] {
  if (!isRecord(value) && !Array.isArray(value)) {
    return hits;
  }
  if (hits.length >= 20) {
    return hits;
  }
  if (Array.isArray(value)) {
    value.slice(0, 50).forEach((entry, index) => findSensitiveKeys(entry, `${path}[${index}]`, hits));
    return hits;
  }
  for (const [key, nested] of Object.entries(value).slice(0, 100)) {
    const nestedPath = `${path}.${key}`;
    if (SENSITIVE_KEY_PATTERN.test(key)) {
      hits.push(nestedPath);
    }
    findSensitiveKeys(nested, nestedPath, hits);
  }
  return hits;
}

function stop(reasonCode: string, message: string, details: Record<string, unknown> = {}) {
  const output = prune({
    status: "needs_input",
    reason_code: reasonCode,
    message,
    ...details,
    stop_conditions: [message],
  });
  return failure(output, { exitCode: 1, stderr: message });
}
