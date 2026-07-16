import fs from "node:fs";
import { spawnSync } from "node:child_process";

const inputs = readInputs();
const objective = stringValue(inputs.objective);
if (!objective) throw new Error("objective is required");

const scope = stringValue(inputs.scope) || "workspace";
const skillFilter = ["workspace", "all"].includes(scope) ? null : scope;
const since = resolveSince(inputs.period);
const limit = resolveLimit(inputs.limit);
const historyOverride = Array.isArray(inputs.history_receipts);
const catalogOverride = Array.isArray(inputs.catalog_items);
const history = historyOverride
  ? {
      receipts: inputs.history_receipts.slice(0, limit),
      pendingRuns: Array.isArray(inputs.pending_runs) ? inputs.pending_runs.slice(0, limit) : [],
    }
  : nativeHistory({ skillFilter, since, limit });
const catalog = catalogOverride ? { items: inputs.catalog_items } : nativeCatalog();

const receipts = (Array.isArray(history.receipts) ? history.receipts : [])
  .filter((row) => !skillFilter || stringValue(row?.name)?.toLowerCase().includes(skillFilter.toLowerCase()));
const pending = (Array.isArray(history.pendingRuns) ? history.pendingRuns : [])
  .filter((row) => !skillFilter || stringValue(row?.name)?.toLowerCase().includes(skillFilter.toLowerCase()));
const catalogItems = (Array.isArray(catalog.items) ? catalog.items : [])
  .filter((row) => !skillFilter || stringValue(row?.name)?.toLowerCase().includes(skillFilter.toLowerCase()));

const statuses = countBy(receipts, (row) => stringValue(row?.status) || "unknown");
const terminalCount = receipts.length;
const closedCount = statuses.closed || 0;
const refusalCount = (statuses.blocked || 0) + (statuses.declined || 0);
const testedCatalogEntries = catalogItems.filter((item) => numberValue(item?.fixtures) + numberValue(item?.harness_cases) > 0).length;
const untestedCatalogEntries = catalogItems.length - testedCatalogEntries;
const decision = terminalCount === 0 && pending.length === 0 ? "needs_more_evidence" : "ready";

const report = {
  schema: "runx.history_report.v1",
  decision,
  objective,
  query: {
    scope,
    period: stringValue(inputs.period),
    since,
    skill_filter: skillFilter,
    limit,
  },
  sources: {
    history: historyOverride ? "supplied_replay" : `runx history --limit ${limit} --json`,
    catalog: catalogOverride ? "supplied_replay" : "runx list skills --ok-only --json",
  },
  runs: {
    terminal_count: terminalCount,
    pending_count: pending.length,
    statuses,
    closed_rate: rate(closedCount, terminalCount),
    refusal_rate: rate(refusalCount, terminalCount),
    top_subjects: topCounts(receipts, (row) => stringValue(row?.name) || "unknown", 10),
  },
  catalog: {
    entry_count: catalogItems.length,
    tested_entry_count: testedCatalogEntries,
    untested_entry_count: untestedCatalogEntries,
    coverage_rate: rate(testedCatalogEntries, catalogItems.length),
  },
  recommendations: recommendations({ decision, statuses, refusalCount, untestedCatalogEntries }),
  limitations: [
    "Native history exposes receipt outcomes and subject identifiers, not hydrated receipt bodies.",
    "Catalog coverage proves declared fixtures or inline cases, not live provider behavior.",
  ],
};

process.stdout.write(`${JSON.stringify({ history_report: report }, null, 2)}\n`);

function nativeHistory({ skillFilter, since, limit }) {
  const args = ["history", "--json"];
  if (skillFilter) args.push("--skill", skillFilter);
  if (since) args.push("--since", since);
  args.push("--limit", String(limit));
  return invokeRunx(args);
}

function nativeCatalog() {
  return invokeRunx(["list", "skills", "--ok-only", "--json"]);
}

function invokeRunx(args) {
  const runx = process.env.RUNX_BIN || "runx";
  const env = { ...process.env };
  delete env.RUNX_INPUTS_JSON;
  delete env.RUNX_INPUTS_PATH;
  const result = spawnSync(runx, args, {
    env,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  let parsed;
  try {
    parsed = JSON.parse(result.stdout || "{}");
  } catch {
    parsed = null;
  }
  if (result.status !== 0 || parsed === null) {
    const message = parsed?.error?.message || result.stderr || result.error?.message || `runx ${args[0]} failed`;
    throw new Error(String(message).replace(/\s+/g, " ").trim().slice(0, 500));
  }
  return parsed;
}

function resolveSince(value) {
  const period = stringValue(value);
  if (!period) return null;
  if (/^\d{4}-\d{2}-\d{2}T/.test(period)) return period;
  const match = period.match(/^(\d+)([dh])$/i);
  if (!match) throw new Error("period must be an RFC 3339 timestamp or a relative duration such as 7d or 24h");
  const amount = Number(match[1]);
  if (!Number.isSafeInteger(amount) || amount <= 0) throw new Error("period must be positive");
  const milliseconds = amount * (match[2].toLowerCase() === "d" ? 86_400_000 : 3_600_000);
  const now = process.env.RUNX_NOW ? Date.parse(process.env.RUNX_NOW) : Date.now();
  if (!Number.isFinite(now)) throw new Error("RUNX_NOW must be an RFC 3339 timestamp");
  return new Date(now - milliseconds).toISOString();
}

function resolveLimit(value) {
  if (value === undefined || value === null || value === "") return 1_000;
  const limit = Number(value);
  if (!Number.isInteger(limit) || limit < 1 || limit > 10_000) {
    throw new Error("limit must be an integer from 1 to 10000");
  }
  return limit;
}

function recommendations({ decision, statuses, refusalCount, untestedCatalogEntries }) {
  if (decision === "needs_more_evidence") {
    return [{ lane: "none", action: "Run governed skills before treating an empty ledger as platform health." }];
  }
  const items = [];
  if ((statuses.failed || 0) > 0 || (statuses.timed_out || 0) > 0) {
    items.push({ lane: "review-receipt", action: "Review representative failed or timed-out receipts and route bounded fixes through skill-lab improve." });
  }
  if (refusalCount > 0) {
    items.push({ lane: "audit-receipt", action: "Sample blocked or declined receipts to confirm the governance boundary is behaving as intended." });
  }
  if (untestedCatalogEntries > 0) {
    items.push({ lane: "skill-lab harness", action: `Add public-contract coverage to ${untestedCatalogEntries} catalog entr${untestedCatalogEntries === 1 ? "y" : "ies"} with no declared fixture or inline case.` });
  }
  return items;
}

function countBy(rows, keyFor) {
  return rows.reduce((counts, row) => {
    const key = keyFor(row);
    counts[key] = (counts[key] || 0) + 1;
    return counts;
  }, {});
}

function topCounts(rows, keyFor, limit) {
  return Object.entries(countBy(rows, keyFor))
    .map(([subject, count]) => ({ subject, count }))
    .sort((left, right) => right.count - left.count || left.subject.localeCompare(right.subject))
    .slice(0, limit);
}

function rate(numerator, denominator) {
  return denominator === 0 ? null : Number((numerator / denominator).toFixed(4));
}

function numberValue(value) {
  return Number.isFinite(value) ? Math.max(0, Math.trunc(value)) : 0;
}

function stringValue(value) {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function readInputs() {
  const raw = process.env.RUNX_INPUTS_PATH
    ? fs.readFileSync(process.env.RUNX_INPUTS_PATH, "utf8")
    : process.env.RUNX_INPUTS_JSON || "{}";
  return JSON.parse(raw);
}
