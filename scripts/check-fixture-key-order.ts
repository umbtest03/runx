import { readFile } from "node:fs/promises";

import { collectKernelFixtureFiles, stableFixtureJson } from "./generate-kernel-parity-fixtures.js";

const failures: string[] = [];

for (const filePath of await collectKernelFixtureFiles()) {
  const actual = await readFile(filePath, "utf8");
  const expected = stableFixtureJson(JSON.parse(actual));
  if (actual !== expected) {
    failures.push(filePath);
  }
}

if (failures.length > 0) {
  console.error(`Fixture key order is not canonical:\n${failures.map((file) => `- ${file}`).join("\n")}`);
  process.exit(1);
}

console.log("Kernel parity fixture keys are sorted.");
