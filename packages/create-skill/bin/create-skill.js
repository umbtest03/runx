#!/usr/bin/env node

import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import process from "node:process";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distEntry = path.join(packageRoot, "dist", "index.js");

if (existsSync(distEntry)) {
  const { runCreateSkill } = await import(pathToFileURL(distEntry).href);
  const exitCode = await runCreateSkill(process.argv.slice(2), {
    stdin: process.stdin,
    stdout: process.stdout,
    stderr: process.stderr,
  });
  process.exitCode = exitCode;
} else {
  const hint = [
    "create-skill: packaged dist is missing.",
    "Run the workspace build before invoking this package.",
    `Expected entry: ${distEntry}`,
  ].join("\n");
  process.stderr.write(`${hint}\n`);
  process.exitCode = 1;
}
