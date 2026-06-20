function readInputs() {
  if (process.env.RUNX_INPUTS_JSON) {
    return JSON.parse(process.env.RUNX_INPUTS_JSON);
  }
  return {
    lane: process.env.RUNX_INPUT_LANE ?? "",
    signal: process.env.RUNX_INPUT_SIGNAL ?? "",
  };
}

const inputs = readInputs();
const lane = String(inputs.lane || "classify");
const signal = String(inputs.signal || "");

const laneDetails = {
  classify: {
    decision: "route",
    summary: "Classify the signal and choose the smallest governed lanes.",
    approval: "not_required",
    next: ["sourcey", "release.prepare", "issue-to-pr", "send-as.draft", "spend.quote", "receipt-audit"],
  },
  sourcey: {
    decision: "prepare",
    summary: "Refresh docs or launch notes from repo evidence before publishing claims.",
    approval: "plan_required",
    next: ["approval.docs_plan"],
  },
  "release.prepare": {
    decision: "prepare",
    summary: "Build a read-only release brief with checks, changelog, risks, and unresolved gates.",
    approval: "publish_required",
    next: ["release.publish.approval"],
  },
  "issue-to-pr": {
    decision: "prepare",
    summary: "Turn a bounded issue signal into a scoped change packet and draft PR handoff.",
    approval: "human_merge_required",
    next: ["review", "merge_gate"],
  },
  "send-as.draft": {
    decision: "draft",
    summary: "Draft outbound comms only; customer-visible send stops at approval.",
    approval: "send_required",
    next: ["approval.send"],
  },
  "spend.quote": {
    decision: "quote",
    summary: "Quote spend intent and caps; money movement stops before settlement authority.",
    approval: "spend_required",
    next: ["approval.spend"],
  },
  "receipt-audit": {
    decision: "verify",
    summary: "Check the receipts and readbacks that prove what happened.",
    approval: "not_required",
    next: ["history", "verify"],
  },
};

const packet = laneDetails[lane] ?? {
  decision: "needs_input",
  summary: `Unknown lane: ${lane}`,
  approval: "not_required",
  next: [],
};

process.stdout.write(JSON.stringify({
  lane_packet: {
    lane,
    signal,
    ...packet,
  },
}, null, 2));
process.stdout.write("\n");
