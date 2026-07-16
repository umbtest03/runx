import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const inputs = JSON.parse(process.env.RUNX_INPUTS_JSON || "{}");
const requestedRef = stringValue(inputs.skill_ref);
if (!requestedRef) throw new Error("skill_ref is required");

const callerRoot = path.resolve(process.env.RUNX_CWD || process.cwd());
const skillRoot = path.dirname(fileURLToPath(import.meta.url));
const skillRef = resolveRef(requestedRef, callerRoot, skillRoot);
const runx = process.env.RUNX_BIN || "runx";
const env = { ...process.env };
delete env.RUNX_INPUTS_JSON;
delete env.RUNX_INPUTS_PATH;

const inspection = invoke(["skill", "inspect", skillRef, "--json"], 30_000);
let harness = {
  attempted: false,
  status: "skipped",
  reason: "inspection_failed",
};

if (inspection.ok) {
  const execution = inspection.data?.capabilities?.execution;
  if (execution === "read" || execution === "plan") {
    const result = invoke(["harness", skillRef, "--json"], 120_000);
    harness = {
      attempted: true,
      status: result.data?.status || (result.ok ? "passed" : "failed"),
      reason: result.ok ? null : "native_harness_failed",
      case_count: numberValue(result.data?.case_count),
      assertion_error_count: numberValue(result.data?.assertion_error_count),
      case_names: stringArray(result.data?.case_names),
    };
  } else {
    harness = {
      attempted: false,
      status: "skipped",
      reason: "needs_consequential_harness_approval",
    };
  }
}

const verdict = !inspection.ok
  ? "uninspectable"
  : harness.status === "failed"
    ? "harness_failed"
    : harness.reason === "needs_consequential_harness_approval"
      ? "needs_consequential_harness_approval"
      : "tested";

process.stdout.write(`${JSON.stringify({
  native_test_evidence: {
    schema: "runx.skill.native_test_evidence.v1",
    requested_ref: requestedRef,
    resolved_ref: skillRef,
    verdict,
    inspect: inspection.ok ? {
      status: inspection.data.status,
      name: inspection.data.name,
      version: inspection.data.version,
      readiness: inspection.data.readiness,
      capabilities: inspection.data.capabilities,
      runner: inspection.data.runner,
      runners: inspection.data.runners,
    } : {
      status: "failed",
      error: inspection.error,
    },
    harness,
    supplied_evidence: inputs.evidence_pack || null,
    constraints: stringValue(inputs.test_constraints),
  },
}, null, 2)}\n`);

function resolveRef(value, caller, owner) {
  if (value.startsWith("./") || value.startsWith("../")) {
    return path.resolve(owner, value);
  }
  const callerCandidate = path.resolve(caller, value);
  return fs.existsSync(callerCandidate) ? callerCandidate : value;
}

function invoke(args, timeout) {
  const result = spawnSync(runx, args, {
    cwd: callerRoot,
    env,
    encoding: "utf8",
    timeout,
    maxBuffer: 4 * 1024 * 1024,
  });
  let data;
  try {
    data = JSON.parse(result.stdout || "{}");
  } catch {
    data = null;
  }
  return {
    ok: result.status === 0 && data !== null,
    data,
    error: sanitize(result.stderr || result.error?.message || "runx command failed"),
  };
}

function sanitize(value) {
  return String(value).replace(/\s+/g, " ").trim().slice(0, 500);
}

function numberValue(value) {
  return Number.isFinite(value) ? value : 0;
}

function stringArray(value) {
  return Array.isArray(value) ? value.map(String).slice(0, 100) : [];
}

function stringValue(value) {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}
