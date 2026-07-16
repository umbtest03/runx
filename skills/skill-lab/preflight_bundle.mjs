import fs from "node:fs";
import path from "node:path";

import { validatePackage } from "./validation.mjs";

const MAX_FILES = 500;
const MAX_BYTES = 20 * 1024 * 1024;
const inputs = JSON.parse(process.env.RUNX_INPUTS_JSON || "{}");
const callerRoot = path.resolve(process.env.RUNX_CWD || process.cwd());
const requestedRoot = String(inputs.repo_root || ".");
const repoRoot = path.isAbsolute(requestedRoot)
  ? path.normalize(requestedRoot)
  : path.resolve(callerRoot, requestedRoot);
const targetDir = normalizeTarget(inputs.target_dir);
const target = path.resolve(repoRoot, targetDir);
const parent = path.dirname(target);
const files = normalizeFiles(inputs.files, targetDir);

if (!isInside(repoRoot, target)) throw new Error("target_dir must stay inside repo_root");
const parentExists = fs.existsSync(parent) && fs.statSync(parent).isDirectory();
const stageRoot = fs.mkdtempSync(path.join(parentExists ? parent : repoRoot, ".runx-skill-lab-preflight-"));
const stage = parentExists ? stageRoot : path.join(stageRoot, targetDir);
try {
  fs.mkdirSync(stage, { recursive: true });
  if (fs.existsSync(target)) copyPackage(target, stage);
  for (const file of files) {
    const destination = path.resolve(stage, file.relative);
    if (!isInside(stage, destination)) throw new Error(`bundle path escapes staged target: ${file.relative}`);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, file.contents, "utf8");
  }

  const validationReport = validatePackage({
    repoRoot,
    target: stage,
    targetDir,
  });
  process.stdout.write(`${JSON.stringify({ validation_report: validationReport }, null, 2)}\n`);
  if (validationReport.verdict === "invalid" || validationReport.verdict === "failed") process.exitCode = 1;
} finally {
  fs.rmSync(stageRoot, { recursive: true, force: true });
}

function normalizeFiles(value, expectedTarget) {
  if (!Array.isArray(value) || value.length === 0) throw new Error("files must be a non-empty array");
  const targetPrefix = `${expectedTarget.split(path.sep).join("/")}/`;
  return value.map((entry) => {
    if (!entry || typeof entry !== "object" || Array.isArray(entry)) throw new Error("files entries must be objects");
    const filePath = typeof entry.path === "string" ? entry.path : "";
    if (!filePath.startsWith(targetPrefix)) throw new Error(`bundle path must stay under ${expectedTarget}: ${filePath}`);
    const relative = filePath.slice(targetPrefix.length);
    if (!relative || relative.startsWith("../") || path.posix.isAbsolute(relative)) {
      throw new Error(`invalid bundle path: ${filePath}`);
    }
    if (typeof entry.contents !== "string") throw new Error(`${filePath} contents must be a string`);
    return { relative, contents: entry.contents };
  });
}

function copyPackage(source, destination) {
  let fileCount = 0;
  let byteCount = 0;
  walk(source, destination);

  function walk(currentSource, currentDestination) {
    for (const entry of fs.readdirSync(currentSource, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name))) {
      if ([".git", ".runx", "node_modules"].includes(entry.name)) continue;
      const sourcePath = path.join(currentSource, entry.name);
      const destinationPath = path.join(currentDestination, entry.name);
      if (entry.isSymbolicLink()) throw new Error(`skill package contains unsupported symlink: ${sourcePath}`);
      if (entry.isDirectory()) {
        fs.mkdirSync(destinationPath, { recursive: true });
        walk(sourcePath, destinationPath);
        continue;
      }
      if (!entry.isFile()) throw new Error(`skill package contains unsupported entry: ${sourcePath}`);
      const stat = fs.statSync(sourcePath);
      fileCount += 1;
      byteCount += stat.size;
      if (fileCount > MAX_FILES || byteCount > MAX_BYTES) {
        throw new Error(`skill package exceeds preflight limit (${MAX_FILES} files, ${MAX_BYTES} bytes)`);
      }
      fs.copyFileSync(sourcePath, destinationPath);
    }
  }
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

function isInside(root, candidate) {
  const relative = path.relative(root, candidate);
  return relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}
