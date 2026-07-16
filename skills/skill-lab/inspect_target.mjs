import fs from "node:fs";
import path from "node:path";

const inputs = JSON.parse(process.env.RUNX_INPUTS_JSON || "{}");
const callerRoot = path.resolve(process.env.RUNX_CWD || process.cwd());
const requestedRoot = String(inputs.repo_root || ".");
const repoRoot = path.isAbsolute(requestedRoot)
  ? path.normalize(requestedRoot)
  : path.resolve(callerRoot, requestedRoot);
const targetDir = normalizeRelativePath(inputs.target_dir, { optional: true });
const targetRoot = targetDir ? path.resolve(repoRoot, targetDir) : null;

if (targetRoot && !isInside(repoRoot, targetRoot)) {
  throw new Error("target_dir must stay inside repo_root");
}

const catalogRoot = fs.existsSync(path.join(repoRoot, "skills"))
  ? path.join(repoRoot, "skills")
  : repoRoot;
const catalogSkills = fs.readdirSync(catalogRoot, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .filter((name) => fs.existsSync(path.join(catalogRoot, name, "SKILL.md")))
  .sort()
  .slice(0, 500);

const targetFiles = targetRoot && fs.existsSync(targetRoot)
  ? listFiles(targetRoot, targetRoot, 200)
  : [];

process.stdout.write(`${JSON.stringify({
  authoring_context: {
    schema: "runx.skill_lab.authoring_context.v1",
    repo_root: repoRoot,
    target_dir: targetDir,
    target_exists: Boolean(targetRoot && fs.existsSync(targetRoot)),
    target_files: targetFiles,
    catalog_root: path.relative(repoRoot, catalogRoot) || ".",
    catalog_skills: catalogSkills,
    objective: stringValue(inputs.objective),
    failure_evidence_present: Boolean(
      stringValue(inputs.receipt_id)
      || stringValue(inputs.receipt_summary)
      || stringValue(inputs.harness_output),
    ),
  },
}, null, 2)}\n`);

function listFiles(root, current, remaining) {
  if (remaining <= 0) return [];
  const files = [];
  for (const entry of fs.readdirSync(current, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    if (files.length >= remaining) break;
    if ([".git", ".runx", "node_modules"].includes(entry.name)) continue;
    const absolute = path.join(current, entry.name);
    if (entry.isSymbolicLink()) continue;
    if (entry.isDirectory()) {
      files.push(...listFiles(root, absolute, remaining - files.length));
      continue;
    }
    if (!entry.isFile()) continue;
    const stat = fs.statSync(absolute);
    files.push({
      path: path.relative(root, absolute),
      bytes: stat.size,
    });
  }
  return files;
}

function normalizeRelativePath(value, { optional = false } = {}) {
  const text = stringValue(value);
  if (!text) {
    if (optional) return null;
    throw new Error("target_dir is required");
  }
  if (path.isAbsolute(text)) throw new Error("target_dir must be repo-relative");
  const normalized = path.normalize(text);
  if (normalized === "." || normalized.startsWith(`..${path.sep}`) || normalized === "..") {
    throw new Error("target_dir must name a child path inside repo_root");
  }
  return normalized;
}

function isInside(root, candidate) {
  const relative = path.relative(root, candidate);
  return relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}

function stringValue(value) {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}
