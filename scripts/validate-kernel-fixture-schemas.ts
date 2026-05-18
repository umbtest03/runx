import path from "node:path";

import { collectKernelFixtureFiles, readKernelFixture, validateKernelFixture } from "./generate-kernel-parity-fixtures.js";

const failures: string[] = [];

for (const filePath of await collectKernelFixtureFiles()) {
  const fixture = await readKernelFixture(filePath);
  if (path.basename(filePath, ".json") !== fixture.name) {
    failures.push(`${filePath}\n  - fixture name '${fixture.name}' must match filename '${path.basename(filePath, ".json")}'`);
  }
  const result = await validateKernelFixture(fixture);
  if (!result.valid) {
    failures.push(`${filePath}\n${result.errors.map((error) => `  - ${error}`).join("\n")}`);
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n\n"));
  process.exit(1);
}

console.log("Kernel parity fixtures validate against schema.");
