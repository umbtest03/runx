import { spawnSync } from "node:child_process";
import path from "node:path";

const inputs = JSON.parse(process.env.RUNX_INPUTS_JSON || "{}");
const callerRoot = path.resolve(process.env.RUNX_CWD || process.cwd());
const requestedRoot = String(inputs.repo_root || ".");
const repoRoot = path.isAbsolute(requestedRoot)
  ? path.normalize(requestedRoot)
  : path.resolve(callerRoot, requestedRoot);
const targetDir = normalizeTarget(inputs.target_dir);
const target = path.resolve(repoRoot, targetDir);
const runx = process.env.RUNX_BIN || "runx";
const env = { ...process.env };
delete env.RUNX_INPUTS_JSON;
delete env.RUNX_INPUTS_PATH;

const inspection = invoke(["skill", "inspect", target, "--json"], 30_000);
let harness = {
  attempted: false,
  status: "skipped",
  reason: "inspection_failed",
};

if (inspection.ok) {
  const execution = inspection.data?.capabilities?.execution;
  if (execution === "read" || execution === "plan") {
    const result = invoke(["harness", target, "--json"], 120_000);
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
  ? "invalid"
  : harness.status === "failed"
    ? "failed"
    : harness.reason === "needs_consequential_harness_approval"
      ? "needs_consequential_test"
      : "validated";

process.stdout.write(`${JSON.stringify({
  validation_report: {
    schema: "runx.skill_lab.validation.v1",
    target_dir: targetDir,
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
  },
}, null, 2)}\n`);
if (verdict === "invalid" || verdict === "failed") process.exitCode = 1;

function invoke(args, timeout) {
  const result = spawnSync(runx, args, {
    cwd: repoRoot,
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

function normalizeTarget(value) {
  const text = typeof value === "string" ? value.trim() : "";
  if (!text || path.isAbsolute(text)) throw new Error("target_dir must be a repo-relative child path");
  const normalized = path.normalize(text);
  if (normalized === "." || normalized === ".." || normalized.startsWith(`..${path.sep}`)) {
    throw new Error("target_dir must stay inside repo_root");
  }
  return normalized;
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
