import path from "node:path";

import { validatePackage } from "./validation.mjs";

const inputs = JSON.parse(process.env.RUNX_INPUTS_JSON || "{}");
const callerRoot = path.resolve(process.env.RUNX_CWD || process.cwd());
const requestedRoot = String(inputs.repo_root || ".");
const repoRoot = path.isAbsolute(requestedRoot)
  ? path.normalize(requestedRoot)
  : path.resolve(callerRoot, requestedRoot);
const targetDir = normalizeTarget(inputs.target_dir);
const target = path.resolve(repoRoot, targetDir);
const validationReport = validatePackage({ repoRoot, target, targetDir });
process.stdout.write(`${JSON.stringify({ validation_report: validationReport }, null, 2)}\n`);
if (validationReport.verdict === "invalid" || validationReport.verdict === "failed") process.exitCode = 1;

function normalizeTarget(value) {
  const text = typeof value === "string" ? value.trim() : "";
  if (!text || path.isAbsolute(text)) throw new Error("target_dir must be a repo-relative child path");
  const normalized = path.normalize(text);
  if (normalized === "." || normalized === ".." || normalized.startsWith(`..${path.sep}`)) {
    throw new Error("target_dir must stay inside repo_root");
  }
  return normalized;
}
