#!/usr/bin/env node
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = readFileSync(path.join(workspaceRoot, "scripts", "verify-fast.mjs"), "utf8");
const parallelSourceGroup = sliceBetween(
  source,
  'await runParallelGroup("source checks"',
  'await runSerialGroup("rust structure checks"',
);

for (const forbidden of [
  "authoring package contract",
  "create-skill package contract",
  "rust:crate-graph",
  "rust:style",
  "cutover:legacy-check",
  "build native runx binary",
  "build harness fixture oracle binary",
  "test:fast",
]) {
  if (parallelSourceGroup.includes(forbidden)) {
    throw new Error(`verify:fast launches ${forbidden} inside the parallel source-check group`);
  }
}

for (const required of [
  'await runSerialGroup("rust structure checks"',
  'step("cutover:legacy-check"',
  'step("build native runx binary"',
  'step("build harness fixture oracle binary"',
  'step("build workspace"',
  'step("authoring package contract"',
  'step("create-skill package contract"',
]) {
  if (!source.includes(required)) {
    throw new Error(`verify:fast is missing required serialized step marker: ${required}`);
  }
}

const buildWorkspaceIndex = source.indexOf('step("build workspace"');
for (const requiredAfterBuild of [
  'step("authoring package contract"',
  'step("create-skill package contract"',
]) {
  const stepIndex = source.indexOf(requiredAfterBuild);
  if (stepIndex < buildWorkspaceIndex) {
    throw new Error(`verify:fast runs ${requiredAfterBuild} before the workspace build`);
  }
}

console.log("verify:fast plan keeps package checks after build and Rust-heavy checks serialized.");

function sliceBetween(contents, start, end) {
  const startIndex = contents.indexOf(start);
  if (startIndex === -1) {
    throw new Error(`missing start marker: ${start}`);
  }
  const endIndex = contents.indexOf(end, startIndex);
  if (endIndex === -1) {
    throw new Error(`missing end marker: ${end}`);
  }
  return contents.slice(startIndex, endIndex);
}
