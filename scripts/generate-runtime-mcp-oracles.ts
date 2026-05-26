import { readFile, stat } from "node:fs/promises";
import path from "node:path";

import {
  assertCleanOracle,
  assertCompletedRustOwner,
  assertEqual,
  assertNoPackageBoundary,
  casePath,
  checkNoStaleOracleFiles,
  parseJson,
  readJson,
  recordField,
  relative,
  workspaceRoot,
  type OracleCase,
} from "./runtime-adapter-oracle-checks.js";

const fixtureRoot = path.join(workspaceRoot, "fixtures", "runtime", "adapters", "mcp");
const oracleRoot = path.join(fixtureRoot, "oracles");
const check = process.argv.includes("--check");

process.chdir(workspaceRoot);

const cases: readonly OracleCase[] = [
  { name: "fixture-success", expectedStatus: "sealed" },
  { name: "fixture-failure-sanitized", expectedStatus: "failure" },
  { name: "sandbox-env-allowed", expectedStatus: "sealed" },
  { name: "sandbox-env-blocked", expectedStatus: "sealed" },
  { name: "missing-metadata", expectedStatus: "failure" },
];

const owner = {
  spec: ".scafld/specs/archive/2026-05/rust-runtime-adapters-mcp.md",
  rustTest: "crates/runx-runtime/tests/mcp_adapter.rs",
  cargo: "cargo test --manifest-path crates/Cargo.toml -p runx-runtime mcp --features mcp -- --nocapture",
  markers: ["McpAdapter", "fixtures/runtime/adapters/mcp", "oracle_text"],
} as const;

if (!check) {
  throw new Error(
    "Runtime MCP oracle generation is retired; checked-in fixtures are Rust-owned. "
      + "Run this script with --check and refresh behavior through the Rust owner if needed.",
  );
}

await assertCompletedRustOwner(owner);
await assertSupportFixtures();

for (const oracleCase of cases) {
  await assertCaseFixture(oracleCase);
}
await checkNoStaleOracleFiles(oracleRoot, cases, "runtime MCP");

console.log(`checked ${cases.length} runtime MCP oracle cases (retired TS generator; Rust owner: ${owner.rustTest})`);

async function assertSupportFixtures(): Promise<void> {
  for (const relativePath of [
    "stdio-server.mjs",
    "wire-contract/basic-lifecycle.requests.jsonl",
    "wire-contract/basic-lifecycle.responses.jsonl",
    "wire-contract/error-paths.requests.jsonl",
    "wire-contract/error-paths.responses.jsonl",
  ]) {
    const filePath = path.join(fixtureRoot, relativePath);
    const fileStat = await stat(filePath);
    if (!fileStat.isFile()) {
      throw new Error(`${relative(filePath)} must be a file.`);
    }
  }
}

async function assertCaseFixture(oracleCase: OracleCase): Promise<void> {
  const requestPath = path.join(casePath(fixtureRoot, oracleCase.name), "request.json");
  const request = await readJson(requestPath);
  assertEqual(request.case, oracleCase.name, `${relative(requestPath)} case`);
  assertEqual(request.mode, "mcp-adapter", `${relative(requestPath)} mode`);
  assertEqual(recordField(request, "source").type, "mcp", `${relative(requestPath)} source.type`);
  assertNoPackageBoundary(requestPath, JSON.stringify(request));

  for (const extension of ["stdout", "stderr", "json"] as const) {
    const oraclePath = path.join(oracleRoot, `${oracleCase.name}.${extension}`);
    const contents = await readFile(oraclePath, "utf8");
    assertCleanOracle(oracleCase.name, oraclePath, contents);
    if (extension === "json") {
      const receipt = parseJson(contents, oraclePath);
      assertEqual(receipt.status, oracleCase.expectedStatus, `${relative(oraclePath)} status`);
    }
  }

  const statusPath = path.join(oracleRoot, `${oracleCase.name}.status`);
  const status = await readFile(statusPath, "utf8");
  assertCleanOracle(oracleCase.name, statusPath, status);
  assertEqual(status, `${oracleCase.expectedStatus}\n`, `${relative(statusPath)} contents`);
}
