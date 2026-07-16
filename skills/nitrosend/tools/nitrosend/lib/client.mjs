import { createHash, randomUUID } from "node:crypto";
import fs from "node:fs";
import { isIP } from "node:net";
import path from "node:path";

const API_URL = "https://api.nitrosend.com/mcp";
const READ_OPERATIONS = new Map([
  ["status", "nitro_get_status"],
  ["insights", "nitro_get_insights"],
  ["review_delivery", "nitro_review_delivery"],
  ["import_status", "nitro_query"],
]);
const ACT_OPERATIONS = new Map([
  ["send_transactional", "nitro_send_message"],
  ["control_delivery", "nitro_control_delivery"],
  ["import_contacts", "nitro_import_contacts"],
  ["import_contacts_file", "__bulk_import__"],
  ["compose_campaign", "nitro_compose_campaign"],
  ["compose_flow", "nitro_compose_flow"],
  ["manage_template", "nitro_manage_template"],
  ["define_segment", "nitro_define_segment"],
]);
const SENSITIVE_KEYS = /authorization|api[_-]?key|bearer|credential|secret|token/i;
const SECRET_VALUE = /\b(?:nskey|wpkey)_(?:live|test)_[A-Za-z0-9_-]+\b/g;

export function readInputs() {
  const raw = process.env.RUNX_INPUTS_PATH
    ? fs.readFileSync(process.env.RUNX_INPUTS_PATH, "utf8")
    : process.env.RUNX_INPUTS_JSON || "{}";
  return JSON.parse(raw);
}

export function writePacket(packet) {
  process.stdout.write(`${JSON.stringify(packet, null, 2)}\n`);
}

export function fail(message) {
  process.stderr.write(`${JSON.stringify({ error: { message: redactText(message) } })}\n`);
  process.exitCode = 1;
}

function redactText(value) {
  return String(value).replaceAll(SECRET_VALUE, "[REDACTED]").slice(0, 2_000);
}

export function redact(value) {
  if (Array.isArray(value)) return value.map(redact);
  if (value && typeof value === "object") {
    const output = {};
    for (const [key, child] of Object.entries(value)) {
      output[key] = SENSITIVE_KEYS.test(key) ? "[REDACTED]" : redact(child);
    }
    return output;
  }
  return typeof value === "string" ? redactText(value) : value;
}

function parseJsonOrSse(text) {
  const trimmed = text.trim();
  if (!trimmed) throw new Error("Nitrosend returned an empty MCP response");
  if (trimmed.startsWith("{")) return JSON.parse(trimmed);

  const payloads = trimmed
    .split(/\r?\n/)
    .filter((line) => line.startsWith("data:"))
    .map((line) => line.slice(5).trim())
    .filter((line) => line && line !== "[DONE]");
  if (payloads.length === 0) throw new Error("Nitrosend returned an invalid MCP event stream");
  return JSON.parse(payloads.at(-1));
}

function parseToolContent(payload) {
  if (payload.error) {
    throw new Error(payload.error.message || "Nitrosend MCP request failed");
  }

  const content = payload.result?.content;
  if (!Array.isArray(content)) return payload.result ?? {};
  const text = content.find((item) => item?.type === "text")?.text;
  if (typeof text !== "string") return payload.result ?? {};
  try {
    const parsed = JSON.parse(text);
    if (parsed && typeof parsed === "object" && parsed.meta?.tool && Object.hasOwn(parsed, "result")) {
      return parsed.result;
    }
    return parsed;
  } catch {
    return { message: text };
  }
}

function operationMap(mode) {
  if (mode === "read") return READ_OPERATIONS;
  if (mode === "act") return ACT_OPERATIONS;
  throw new Error(`unsupported Nitrosend adapter mode: ${mode}`);
}

export function validateInvocation(mode, inputs) {
  const operations = operationMap(mode);
  const operation = typeof inputs.operation === "string" ? inputs.operation.trim() : "";
  if (!operations.has(operation)) {
    return {
      decision: "needs_input",
      blockers: [`operation must be one of: ${[...operations.keys()].join(", ")}`],
    };
  }
  if (inputs.arguments !== undefined && (inputs.arguments === null || typeof inputs.arguments !== "object" || Array.isArray(inputs.arguments))) {
    return { decision: "needs_input", blockers: ["arguments must be a JSON object"] };
  }

  const args = inputs.arguments ?? {};
  if (mode === "read" && operation === "insights") {
    const scopes = ["account", "flow", "campaign", "message"];
    if (!scopes.includes(args.scope)) {
      return { decision: "needs_input", blockers: [`arguments.scope must be one of: ${scopes.join(", ")}`] };
    }
    if (args.scope !== "account" && !Number.isInteger(Number(args.entity_id))) {
      return { decision: "needs_input", blockers: [`arguments.entity_id is required for ${args.scope} insights`] };
    }
  }
  if (mode === "read" && operation === "review_delivery") {
    if (!["template", "flow", "campaign"].includes(args.target_type) || !Number.isInteger(Number(args.target_id))) {
      return { decision: "needs_input", blockers: ["review_delivery requires a valid target_type and integer target_id"] };
    }
  }
  if (mode === "read" && operation === "import_status" && !Number.isInteger(Number(args.import_id))) {
    return { decision: "needs_input", blockers: ["import_status requires arguments.import_id"] };
  }
  if (mode === "act" && operation === "send_transactional") {
    if (!["email", "sms"].includes(args.channel) || typeof args.to !== "string" || !args.to.trim()) {
      return { decision: "needs_input", blockers: ["send_transactional requires channel email or sms and one recipient"] };
    }
  }
  if (mode === "act" && operation === "control_delivery") {
    const deliveryOperations = ["approve", "reject", "live", "schedule", "pause", "resume", "cancel", "archive", "restore", "delete"];
    if (!["flow", "campaign"].includes(args.target_type) || !Number.isInteger(Number(args.target_id)) || !deliveryOperations.includes(args.operation)) {
      return { decision: "needs_input", blockers: ["control_delivery requires a valid target_type, integer target_id, and lifecycle operation"] };
    }
    if (args.operation === "schedule" && !args.scheduled_at) {
      return { decision: "needs_input", blockers: ["scheduled campaign delivery requires arguments.scheduled_at"] };
    }
  }
  if (mode === "act" && ["import_contacts", "import_contacts_file"].includes(operation)) {
    if (typeof args.source_id !== "string" || !args.source_id.trim() || typeof args.consent_basis !== "string" || !args.consent_basis.trim()) {
      return { decision: "needs_input", blockers: ["contact imports require arguments.source_id and arguments.consent_basis"] };
    }
    if (/purchased|scraped|data\s*broker/i.test(args.consent_basis)) {
      return { decision: "refused", blockers: ["purchased, scraped, and data-broker contact sources are not permitted"] };
    }
  }
  if (mode === "act" && operation === "send_transactional" && args.dry_run !== true && !args.idempotency_key) {
    return {
      decision: "refused",
      blockers: ["a real transactional send requires arguments.idempotency_key"],
    };
  }
  if (
    mode === "act" &&
    operation === "control_delivery" &&
    ["live", "schedule"].includes(args.operation) &&
    args.target_type === "campaign" &&
    !args.idempotency_key
  ) {
    return {
      decision: "refused",
      blockers: ["live or scheduled campaign delivery requires arguments.idempotency_key"],
    };
  }
  if (mode === "act" && ["import_contacts", "import_contacts_file"].includes(operation) && args.dry_run !== true && !args.idempotency_key) {
    return {
      decision: "refused",
      blockers: ["a real contact import requires arguments.idempotency_key"],
    };
  }

  return { decision: "ready", operation, tool: operations.get(operation), arguments: args };
}

function providerData(packet) {
  return packet?.result?.data ?? packet?.result?.result ?? packet?.result ?? {};
}

async function checksumFile(filePath) {
  const hash = createHash("md5");
  await new Promise((resolve, reject) => {
    const stream = fs.createReadStream(filePath);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("error", reject);
    stream.on("end", resolve);
  });
  return hash.digest("base64");
}

export function validateUploadUrl(rawUrl) {
  const url = new URL(rawUrl);
  if (url.protocol !== "https:") throw new Error("Nitrosend returned a non-HTTPS upload URL");
  if (url.hostname === "localhost" || isIP(url.hostname)) {
    throw new Error("Nitrosend returned a disallowed upload host");
  }
  return url;
}

async function importContactsFile(inputs, options) {
  const args = inputs.arguments ?? {};
  const csvPath = String(args.csv_path ?? "");
  if (!path.isAbsolute(csvPath) || path.extname(csvPath).toLowerCase() !== ".csv") {
    return {
      decision: "needs_input",
      provider: "nitrosend",
      mode: "act",
      operation: "import_contacts_file",
      result: null,
      evidence: null,
      blockers: ["arguments.csv_path must be an absolute path to a .csv file"],
    };
  }

  let stat;
  try {
    stat = fs.statSync(csvPath);
  } catch {
    return {
      decision: "needs_input",
      provider: "nitrosend",
      mode: "act",
      operation: "import_contacts_file",
      result: null,
      evidence: null,
      blockers: ["the CSV file does not exist or is not readable"],
    };
  }
  if (!stat.isFile() || stat.size === 0) {
    return {
      decision: "needs_input",
      provider: "nitrosend",
      mode: "act",
      operation: "import_contacts_file",
      result: null,
      evidence: null,
      blockers: ["the CSV path must identify a non-empty regular file"],
    };
  }

  const upload = {
    filename: path.basename(csvPath),
    content_type: "text/csv",
    byte_size: stat.size,
    checksum: await checksumFile(csvPath),
  };
  const common = {
    brand_sid: inputs.brand_sid,
    operation: "import_contacts",
  };
  const reservation = await invokeNitrosend("act", {
    ...common,
    arguments: {
      upload,
      source_id: args.source_id,
      consent_basis: args.consent_basis,
      dry_run: args.dry_run === true,
      idempotency_key: args.idempotency_key,
    },
  }, options);
  if (reservation.decision !== "ok" || args.dry_run === true) return reservation;

  const reserved = providerData(reservation);
  const directUpload = reserved.direct_upload;
  const signedId = reserved.signed_id;
  if (!directUpload?.url || !directUpload?.headers || !signedId) {
    return {
      decision: "provider_error",
      provider: "nitrosend",
      mode: "act",
      operation: "import_contacts_file",
      result: null,
      evidence: reservation.evidence,
      blockers: ["Nitrosend did not return a complete authorized upload reservation"],
    };
  }

  let uploadUrl;
  try {
    uploadUrl = validateUploadUrl(directUpload.url);
  } catch (error) {
    return {
      decision: "provider_error",
      provider: "nitrosend",
      mode: "act",
      operation: "import_contacts_file",
      result: null,
      evidence: reservation.evidence,
      blockers: [error instanceof Error ? error.message : String(error)],
    };
  }

  const uploadResponse = await options.fetchImpl(uploadUrl, {
    method: "PUT",
    headers: directUpload.headers,
    body: fs.createReadStream(csvPath),
    duplex: "half",
  });
  if (!uploadResponse.ok) {
    return {
      decision: "provider_error",
      provider: "nitrosend",
      mode: "act",
      operation: "import_contacts_file",
      result: null,
      evidence: reservation.evidence,
      blockers: [`authorized CSV upload failed with HTTP ${uploadResponse.status}`],
    };
  }

  const finalized = await invokeNitrosend("act", {
    ...common,
    arguments: {
      signed_id: signedId,
      source_id: args.source_id,
      consent_basis: args.consent_basis,
      resource: "contacts",
      parser: "default",
      columns: args.columns,
      options: args.options,
      dry_run: false,
      idempotency_key: args.idempotency_key,
    },
  }, options);
  if (finalized.evidence) {
    finalized.evidence.upload = {
      filename: upload.filename,
      byte_size: upload.byte_size,
      checksum_verified: true,
      signed_url_retained: false,
    };
  }
  finalized.operation = "import_contacts_file";
  return finalized;
}

function providerReference(operation, result) {
  const data = result?.data ?? result;
  const id = data?.id ?? data?.message_id ?? data?.import_id ?? data?.target_id ?? data?.campaign_id ?? data?.flow_id;
  return id === undefined || id === null ? null : `nitrosend:${operation}:${id}`;
}

export function normalizeResult({ mode, operation, tool, requestId, httpStatus, result }) {
  const safeResult = redact(result);
  const error = safeResult?.error === true || safeResult?.isError === true;
  return {
    decision: error ? "provider_error" : "ok",
    provider: "nitrosend",
    mode,
    operation,
    tool,
    provider_ref: providerReference(operation, safeResult),
    result: safeResult,
    evidence: {
      request_id: requestId,
      http_status: httpStatus,
      observed_at: new Date().toISOString(),
      credential_material: "redacted",
    },
    blockers: error ? [safeResult?.message || "Nitrosend rejected the operation"] : [],
  };
}

export async function invokeNitrosend(mode, inputs, { fetchImpl = fetch, apiKey = process.env.NITROSEND_API_KEY } = {}) {
  const validated = validateInvocation(mode, inputs);
  if (validated.decision !== "ready") {
    return {
      provider: "nitrosend",
      mode,
      operation: inputs.operation ?? null,
      result: null,
      evidence: null,
      ...validated,
    };
  }
  if (typeof apiKey !== "string" || !apiKey.trim()) {
    return {
      decision: "needs_input",
      provider: "nitrosend",
      mode,
      operation: validated.operation,
      result: null,
      evidence: null,
      blockers: ["Nitrosend credential is missing; configure a Nitrosend Runx grant or deliver NITROSEND_API_KEY"],
    };
  }

  if (validated.tool === "__bulk_import__") {
    return importContactsFile(inputs, { fetchImpl, apiKey });
  }

  const requestId = randomUUID();
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 30_000);
  let response;
  try {
    response = await fetchImpl(API_URL, {
      method: "POST",
      headers: {
        accept: "application/json, text/event-stream",
        authorization: `Bearer ${apiKey}`,
        "content-type": "application/json",
        ...(inputs.brand_sid ? { "x-brand-sid": String(inputs.brand_sid) } : {}),
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: requestId,
        method: "tools/call",
        params: { name: validated.tool, arguments: providerArguments(validated.operation, validated.arguments) },
      }),
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timeout);
  }

  const body = await response.text();
  if (!response.ok) {
    return {
      decision: response.status === 401 || response.status === 403 ? "needs_input" : "provider_error",
      provider: "nitrosend",
      mode,
      operation: validated.operation,
      result: null,
      evidence: { request_id: requestId, http_status: response.status, credential_material: "redacted" },
      blockers: [response.status === 401 || response.status === 403
        ? "Nitrosend rejected the configured credential"
        : `Nitrosend returned HTTP ${response.status}`],
    };
  }

  let result;
  try {
    result = parseToolContent(parseJsonOrSse(body));
  } catch (error) {
    return {
      decision: "provider_error",
      provider: "nitrosend",
      mode,
      operation: validated.operation,
      result: null,
      evidence: { request_id: requestId, http_status: response.status, credential_material: "redacted" },
      blockers: [redactText(error instanceof Error ? error.message : String(error))],
    };
  }
  return normalizeResult({
    mode,
    operation: validated.operation,
    tool: validated.tool,
    requestId,
    httpStatus: response.status,
    result,
  });
}

function providerArguments(operation, args) {
  if (operation === "import_status") {
    return { entity: "imports", filters: { id: Number(args.import_id) }, page: 1, per: 1 };
  }
  if (operation !== "import_contacts") return args;
  const { source_id: sourceId, consent_basis: _consentBasis, ...providerArgs } = args;
  if (Array.isArray(providerArgs.records)) {
    providerArgs.records = providerArgs.records.map((record) => ({
      ...record,
      source: record.source || sourceId,
    }));
  }
  return providerArgs;
}

export async function run(mode) {
  writePacket(await invokeNitrosend(mode, readInputs()));
}
