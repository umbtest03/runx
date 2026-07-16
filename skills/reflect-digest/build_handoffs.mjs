import path from "node:path";

const inputs = JSON.parse(process.env.RUNX_INPUTS_JSON || "{}");
const groups = Array.isArray(inputs.grouped_reflections) ? inputs.grouped_reflections : [];
const proposals = Array.isArray(inputs.proposals) ? inputs.proposals : [];
if (proposals.length > 20) throw new Error("reflect digest may emit at most 20 proposals");

const groupsBySkill = new Map(groups.map((group) => [stringValue(group?.skill_ref), group]));
const seen = new Set();
const normalized = proposals.map((proposal) => {
  if (!proposal || typeof proposal !== "object" || Array.isArray(proposal)) throw new Error("proposals must contain objects");
  const skillRef = requiredString(proposal.skill_ref, "proposal.skill_ref");
  if (seen.has(skillRef)) throw new Error(`duplicate proposal for ${skillRef}`);
  seen.add(skillRef);
  const group = groupsBySkill.get(skillRef);
  if (!group) throw new Error(`proposal has no grouped reflection evidence: ${skillRef}`);
  const targetDir = normalizeTarget(proposal.target_dir);
  const objective = requiredString(proposal.objective, `${skillRef}.objective`);
  const evidenceSummary = requiredString(proposal.evidence_summary, `${skillRef}.evidence_summary`);
  const receiptIds = stringArray(proposal.supporting_receipt_ids, `${skillRef}.supporting_receipt_ids`);
  const admittedIds = new Set(Array.isArray(group.supporting_receipt_ids) ? group.supporting_receipt_ids.map(String) : []);
  if (receiptIds.some((receiptId) => !admittedIds.has(receiptId))) {
    throw new Error(`${skillRef} proposal cites a receipt outside its grouped evidence`);
  }
  return {
    skill_ref: skillRef,
    target_dir: targetDir,
    objective,
    evidence_summary: evidenceSummary,
    supporting_receipt_ids: receiptIds,
    boundaries: Array.isArray(proposal.boundaries) ? proposal.boundaries.map(String).filter(Boolean).slice(0, 20) : [],
  };
});

const handoffs = normalized.map((proposal) => ({
  skill: "skill-lab",
  runner: "improve",
  target_skill_ref: proposal.skill_ref,
  supporting_receipt_ids: proposal.supporting_receipt_ids,
  inputs: {
    objective: proposal.objective,
    target_dir: proposal.target_dir,
    receipt_id: proposal.supporting_receipt_ids[0],
    receipt_summary: `${proposal.evidence_summary} Supporting receipts: ${proposal.supporting_receipt_ids.join(", ")}.`,
  },
  boundaries: proposal.boundaries,
}));

process.stdout.write(`${JSON.stringify({ proposals: normalized, skill_lab_handoffs: handoffs }, null, 2)}\n`);

function normalizeTarget(value) {
  const text = requiredString(value, "proposal.target_dir");
  if (path.isAbsolute(text)) throw new Error("proposal.target_dir must be repo-relative");
  const normalized = path.normalize(text);
  if (normalized === "." || normalized === ".." || normalized.startsWith(`..${path.sep}`)) {
    throw new Error("proposal.target_dir must stay inside the repository");
  }
  return normalized;
}

function stringArray(value, field) {
  if (!Array.isArray(value) || value.length === 0) throw new Error(`${field} must be a non-empty array`);
  return [...new Set(value.map((entry) => requiredString(entry, field)))];
}

function requiredString(value, field) {
  const result = stringValue(value);
  if (!result) throw new Error(`${field} must be a non-empty string`);
  return result;
}

function stringValue(value) {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}
