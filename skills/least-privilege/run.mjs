import fs from "node:fs";

const inputs = readInputs();
const subject = stringValue(inputs.subject) || "unknown";
const grantedScopes = stringArray(inputs.granted_scopes, "granted_scopes");
const usageSummary = readUsageSummary(inputs.usage_summary);
const receiptIds = optionalStringArray(inputs.receipt_ids);
const ledgerEvidence = readLedgerEvidence(inputs.ledger_evidence);
const matchedReceiptIds = new Set(ledgerEvidence.matched_receipts.map((entry) => stringValue(entry?.receipt_id)).filter(Boolean));
const missingReceiptIds = receiptIds.filter((receiptId) => !matchedReceiptIds.has(receiptId));
const observed = collectObservedUsage(usageSummary);
const evidenceReady = receiptIds.length > 0 && missingReceiptIds.length === 0 && observed.size > 0;
const scopeDiff = evidenceReady
  ? grantedScopes.map((scope) => classifyScope(scope, observed))
  : grantedScopes.map((scope) => deferredScope(scope));
const removedScopes = scopeDiff.filter((entry) => entry.classification === "remove").map((entry) => entry.granted_scope);
const narrowedScopes = scopeDiff
  .filter((entry) => entry.classification === "narrow" && entry.proposal)
  .map((entry) => ({ from: entry.granted_scope, to: entry.proposal }));
const keptScopes = scopeDiff.filter((entry) => entry.classification === "keep").map((entry) => entry.granted_scope);
const deferredScopes = scopeDiff.filter((entry) => entry.classification === "defer").map((entry) => entry.granted_scope);
const attenuatedGrant = [
  ...keptScopes,
  ...narrowedScopes.map((entry) => entry.to),
  ...deferredScopes,
];

const limitations = [];
if (receiptIds.length === 0) {
  limitations.push("No receipt ids were supplied; scope observations are not attributable to sealed runs.");
}
if (missingReceiptIds.length > 0) {
  limitations.push(`Native ledger evidence did not resolve ${missingReceiptIds.length} supplied receipt id(s).`);
}
if (observed.size === 0) {
  limitations.push("No observed scope usage was provided; the grant cannot be safely narrowed.");
}
if (evidenceReady) {
  limitations.push("Native history proves the receipt references and statuses; normalized scope observations remain caller-supplied because history does not expose hydrated receipt bodies.");
}

const status = !evidenceReady
  ? "needs_more_evidence"
  : removedScopes.length > 0 || narrowedScopes.length > 0
    ? "attenuation_proposed"
    : "no_change";

const packet = {
  status,
  subject,
  evidence: {
    receipt_ids: receiptIds,
    matched_receipt_ids: [...matchedReceiptIds],
    missing_receipt_ids: missingReceiptIds,
    chain_verification: ledgerEvidence.chain_verification,
    receipt_window: stringValue(usageSummary.receipt_window) || null,
    grant_source: stringValue(inputs.grant_source) || null,
    limitations,
  },
  scope_diff: scopeDiff,
  attenuated_grant: attenuatedGrant,
  removed_scopes: removedScopes,
  narrowed_scopes: narrowedScopes,
  kept_scopes: keptScopes,
  deferred_scopes: deferredScopes,
  residual_risk: residualRisk({ keptScopes, deferredScopes, limitations }),
  reviewer_action: status === "attenuation_proposed"
    ? "applyable_now"
    : status === "needs_more_evidence"
      ? "gather_more_receipts"
      : "none",
  receipt_expectations: {
    classification_counts: countClassifications(scopeDiff),
    stop_status: status,
    unresolved_questions: limitations,
  },
};

const result = {
  audit_report: packet,
  attenuation_proposals: [
    ...removedScopes.map((scope) => ({
      action: "remove",
      scope,
      rationale: "No cited receipt exercised this authority.",
    })),
    ...narrowedScopes.map((entry) => ({
      action: "narrow",
      from: entry.from,
      to: entry.to,
      rationale: "Observed use fits the narrower grant.",
    })),
  ],
  verdict: renderVerdict(packet),
};

process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);

function readInputs() {
  const raw = process.env.RUNX_INPUTS_PATH
    ? fs.readFileSync(process.env.RUNX_INPUTS_PATH, "utf8")
    : process.env.RUNX_INPUTS_JSON || "{}";
  return JSON.parse(raw);
}

function readUsageSummary(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("usage_summary must be an object with receipt_ids and observed usage");
  }
  return value;
}

function readLedgerEvidence(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return { matched_receipts: [], chain_verification: { checked: false, intact: null, breaks: [] } };
  }
  return {
    matched_receipts: Array.isArray(value.matched_receipts) ? value.matched_receipts : [],
    chain_verification: value.chain_verification && typeof value.chain_verification === "object"
      ? value.chain_verification
      : { checked: false, intact: null, breaks: [] },
  };
}

function stringArray(value, field) {
  if (!Array.isArray(value) || value.length === 0) {
    throw new Error(`${field} must be a non-empty array`);
  }
  return value.map((entry) => {
    if (typeof entry !== "string" || entry.trim().length === 0) {
      throw new Error(`${field} entries must be non-empty strings`);
    }
    return entry.trim();
  });
}

function optionalStringArray(value) {
  if (!Array.isArray(value)) return [];
  return [...new Set(value.filter((entry) => typeof entry === "string" && entry.trim()).map((entry) => entry.trim()))];
}

function collectObservedUsage(summary) {
  const observed = new Map();
  const entries = Array.isArray(summary.observed) ? summary.observed : [];
  for (const entry of entries) {
    if (!entry || typeof entry !== "object") continue;
    const scope = stringValue(entry.scope);
    if (!scope) continue;
    const current = observed.get(scope) || { count: 0, refs: [] };
    current.count += Number.isFinite(entry.count) ? Math.max(0, Math.trunc(entry.count)) : 1;
    if (Array.isArray(entry.refs)) current.refs.push(...entry.refs.map(String));
    observed.set(scope, current);
  }
  return observed;
}

function classifyScope(scope, observed) {
  const normalized = normalizeScope(scope);
  const exact = observed.get(scope);
  if (exact && exact.count > 0) {
    return diffEntry({
      scope,
      normalized,
      observedUse: { count: exact.count, receipt_refs: exact.refs },
      classification: "keep",
      proposal: null,
      rationale: "Observed receipt usage exercised this exact authority.",
    });
  }

  const narrower = observedNarrowerScope(scope, observed);
  if (narrower) {
    return diffEntry({
      scope,
      normalized,
      observedUse: { count: narrower.count, receipt_refs: narrower.refs, scopes: narrower.scopes },
      classification: "narrow",
      proposal: commonScopePrefix(narrower.scopes) || narrower.scopes[0],
      rationale: "Observed usage fits a narrower scope than the granted wildcard.",
    });
  }

  return diffEntry({
    scope,
    normalized,
    observedUse: { count: 0, receipt_refs: [] },
    classification: "remove",
    proposal: null,
    rationale: "No cited receipt exercised this authority.",
  });
}

function deferredScope(scope) {
  return diffEntry({
    scope,
    normalized: normalizeScope(scope),
    observedUse: { count: 0, receipt_refs: [] },
    classification: "defer",
    proposal: null,
    rationale: "Receipt-backed usage evidence is incomplete, so this authority must remain unchanged.",
  });
}

function observedNarrowerScope(scope, observed) {
  const wildcardPrefix = scope.endsWith("*") ? scope.slice(0, -1) : null;
  const writePrefix = scope.endsWith(":write") ? scope.slice(0, -"write".length) : null;
  const matches = [...observed.entries()].filter(([used]) =>
    wildcardPrefix ? used.startsWith(wildcardPrefix) : writePrefix ? used === `${writePrefix}read` : false,
  );
  if (matches.length === 0) return null;
  return {
    scopes: matches.map(([used]) => used),
    count: matches.reduce((sum, [, usage]) => sum + usage.count, 0),
    refs: matches.flatMap(([, usage]) => usage.refs),
  };
}

function normalizeScope(scope) {
  const [verbPart, ...resourceParts] = scope.split(":");
  const resource = resourceParts.join(":") || null;
  return {
    verb: verbPart || null,
    resource,
    conditions: null,
  };
}

function diffEntry({ scope, normalized, observedUse, classification, proposal, rationale }) {
  return {
    granted_scope: scope,
    normalized,
    observed_use: {
      count: observedUse.count,
      verbs: normalized.verb ? [normalized.verb] : [],
      resources: normalized.resource ? [normalized.resource] : [],
      receipt_refs: observedUse.receipt_refs || [],
      scopes: observedUse.scopes || [],
    },
    classification,
    proposal,
    rationale,
  };
}

function commonScopePrefix(scopes) {
  if (scopes.length !== 1) return null;
  return scopes[0];
}

function countClassifications(entries) {
  return entries.reduce((counts, entry) => {
    counts[entry.classification] = (counts[entry.classification] || 0) + 1;
    return counts;
  }, {});
}

function residualRisk({ keptScopes, deferredScopes, limitations }) {
  const risks = [];
  if (keptScopes.length > 0) {
    risks.push(`The subject still retains ${keptScopes.length} observed scope(s).`);
  }
  if (deferredScopes.length > 0) {
    risks.push(`The subject has ${deferredScopes.length} deferred scope(s) requiring policy review.`);
  }
  risks.push(...limitations);
  return risks;
}

function renderVerdict(packet) {
  if (packet.status === "attenuation_proposed") {
    return `over-privileged: remove ${packet.removed_scopes.length}, narrow ${packet.narrowed_scopes.length}`;
  }
  if (packet.status === "needs_more_evidence") {
    return "needs_more_evidence: no exercised scopes were provided";
  }
  return "no_change: observed usage matches the grant";
}

function stringValue(value) {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : null;
}
