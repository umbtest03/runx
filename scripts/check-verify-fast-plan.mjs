#!/usr/bin/env node
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = readFileSync(path.join(workspaceRoot, "scripts", "verify-fast.mjs"), "utf8");
const parallelSourceGroup = sliceBetween(
  source,
  'await runParallelGroup("source checks"',
  'await runSerialGroup("package contract checks"',
);

for (const forbidden of [
  "authoring package contract",
  "create-skill package contract",
  "rust:crate-graph",
  "rust:style",
  "build native runx binary",
  "build harness fixture oracle binary",
  "test:fast",
]) {
  if (parallelSourceGroup.includes(forbidden)) {
    throw new Error(`verify:fast launches ${forbidden} inside the parallel source-check group`);
  }
}

for (const required of [
  'await runSerialGroup("package contract checks"',
  'await runSerialGroup("rust structure checks"',
  'step("build native runx binary"',
  'step("build harness fixture oracle binary"',
]) {
  if (!source.includes(required)) {
    throw new Error(`verify:fast is missing required serialized step marker: ${required}`);
  }
}

console.log("verify:fast plan keeps package and Rust-heavy checks serialized.");

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
