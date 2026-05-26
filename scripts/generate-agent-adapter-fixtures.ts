import { readFile } from "node:fs/promises";
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

const fixtureRoot = path.join(workspaceRoot, "fixtures", "runtime", "adapters", "agent");
const oracleRoot = path.join(fixtureRoot, "oracles");
const check = process.argv.includes("--check");

process.chdir(workspaceRoot);

const cases: readonly OracleCase[] = [
  { name: "agent-plain-success", expectedStatus: "sealed" },
  { name: "agent-step-structured-success", expectedStatus: "sealed" },
  { name: "provider-error-sanitized", expectedStatus: "failure" },
];

const owner = {
  spec: ".scafld/specs/archive/2026-05/rust-runtime-adapters-agent.md",
  rustTest: "crates/runx-runtime/tests/agent_parity.rs",
  cargo: "cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features a2a,agent --test agent_parity",
  markers: ["AgentAdapter", "RecordingResolver", "run_harness_fixture_with_adapter"],
} as const;

if (!check) {
  throw new Error(
    "Agent adapter oracle generation is retired; checked-in fixtures are Rust-owned. "
      + "Run this script with --check and refresh behavior through the Rust owner if needed.",
  );
}

await assertCompletedRustOwner(owner);

for (const oracleCase of cases) {
  await assertCaseFixture(oracleCase);
}
await checkNoStaleOracleFiles(oracleRoot, cases, "agent adapter");

console.log(`checked ${cases.length} agent adapter oracle cases (retired TS generator; Rust owner: ${owner.rustTest})`);

async function assertCaseFixture(oracleCase: OracleCase): Promise<void> {
  const requestPath = path.join(casePath(fixtureRoot, oracleCase.name), "request.json");
  const request = await readJson(requestPath);
  assertEqual(request.case, oracleCase.name, `${relative(requestPath)} case`);
  assertEqual(request.mode, "agent-adapter", `${relative(requestPath)} mode`);
  const sourceType = recordField(request, "source").type;
  if (sourceType !== "agent" && sourceType !== "agent-step") {
    throw new Error(`${relative(requestPath)} source.type must be agent or agent-step.`);
  }
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
