import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { runxGeneratedSchemaArtifacts } from "../packages/contracts/src/index.js";

const workspaceRoot = process.cwd();
const check = process.argv.includes("--check");

await mkdir(path.join(workspaceRoot, "schemas"), { recursive: true });

const staleArtifacts: string[] = [];

for (const [fileName, schema] of Object.entries(runxGeneratedSchemaArtifacts)) {
  const schemaPath = path.join(workspaceRoot, "schemas", fileName);
  const generated = `${JSON.stringify(schema, null, 2)}\n`;
  if (check) {
    let current = "";
    try {
      current = await readFile(schemaPath, "utf8");
    } catch {
      staleArtifacts.push(fileName);
      continue;
    }
    if (current !== generated) {
      staleArtifacts.push(fileName);
    }
    continue;
  }
  await writeFile(schemaPath, generated);
}

if (staleArtifacts.length > 0) {
  console.error("Generated contract schemas are stale:");
  for (const fileName of staleArtifacts) {
    console.error(`- ${fileName}`);
  }
  process.exit(1);
}
