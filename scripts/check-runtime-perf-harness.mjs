#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const tempRoot = mkdtempSync(path.join(tmpdir(), "runx-perf-harness-"));

try {
  const baselinePath = path.join(tempRoot, "baseline.json");
  const passingPath = path.join(tempRoot, "candidate-pass.json");
  const failingPath = path.join(tempRoot, "candidate-fail.json");

  writeFixture(baselinePath, {
    throughput: 100,
    mean_ns: 10_000_000,
    p95_ns: 11_000_000,
    p99_ns: 12_000_000,
    allocation_count: 100,
    spawn_count: 1,
  });
  writeFixture(passingPath, {
    throughput: 210,
    mean_ns: 4_700_000,
    p95_ns: 5_000_000,
    p99_ns: 12_100_000,
    allocation_count: 90,
    spawn_count: 1,
  });
  writeFixture(failingPath, {
    throughput: 90,
    mean_ns: 11_000_000,
    p95_ns: 15_000_000,
    p99_ns: 20_000_000,
    allocation_count: 140,
    spawn_count: 3,
  });

  const pass = runCheck(baselinePath, passingPath);
  if (pass.status !== 0) {
    process.stderr.write(pass.stderr || pass.stdout);
    throw new Error("runtime perf harness rejected the passing candidate fixture");
  }

  const fail = runCheck(baselinePath, failingPath);
  if (fail.status === 0) {
    process.stderr.write(fail.stdout);
    throw new Error("runtime perf harness accepted the intentionally bad candidate fixture");
  }

  assertReleaseProbeInvariant();

  process.stdout.write("Runtime perf harness check passed.\n");
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
}

function runCheck(baselinePath, candidatePath) {
  return spawnSync(
    "node",
    [
      "scripts/runtime-throughput.mjs",
      "check",
      "--baseline",
      baselinePath,
      "--candidate",
      candidatePath,
      "--workloads",
      "graph_planning",
      "--min-throughput-ratio",
      "2.00",
      "--max-spawn-count",
      "1",
      "--max-p99-regression",
      "1.10",
      "--max-allocation-regression",
      "1.10",
    ],
    {
      cwd: workspaceRoot,
      encoding: "utf8",
    },
  );
}

function writeFixture(filePath, metric) {
  writeFileSync(
    filePath,
    `${JSON.stringify({
      schema: "runx.oss_runtime_throughput.v1",
      captured_at: "2026-05-27T00:00:00.000Z",
      command: "perf:harness-check",
      workloads: {
        graph_planning: {
          source: "fixture",
          unit: "iterations_per_second",
          ...metric,
        },
      },
    }, null, 2)}\n`,
  );
}

function assertReleaseProbeInvariant() {
  const source = readFileSync(path.join(workspaceRoot, "scripts/runtime-throughput.mjs"), "utf8");
  if (!/cargoPerfProfileDir\s*=\s*path\.join\(cargoTargetDir,\s*"release"\)/u.test(source)) {
    throw new Error("runtime perf harness must use the release profile directory for process probes");
  }
  if (!/source:\s*"node"/u.test(source) || !/measureTsBridgeFraming/u.test(source)) {
    throw new Error("runtime perf harness must keep the TypeScript framing row process-local");
  }
  const mcpProbeSource = functionSource(source, "mcpSessionProbe", "measureNativeCliLaunch");
  const nativeProbeSource = functionSource(source, "nativeCliProbe", "runNativeCliProbe");
  if (!/"--release"[\s\S]*"--bin"[\s\S]*"runx-mcp-session-probe"/u.test(mcpProbeSource)) {
    throw new Error("runtime perf harness must build the MCP session probe with --release");
  }
  if (!/"--release"[\s\S]*"--bin"[\s\S]*"runx"/u.test(nativeProbeSource)) {
    throw new Error("runtime perf harness must build the native runx launch probe with --release");
  }
  if (!/cargo build runx-mcp-session-probe did not produce/u.test(mcpProbeSource)) {
    throw new Error("runtime perf harness must verify the MCP release probe exists after build");
  }
  if (!/cargo build runx-cli did not produce/u.test(nativeProbeSource)) {
    throw new Error("runtime perf harness must verify the native runx release probe exists after build");
  }
  if (/crates",\s*"target",\s*"debug"/u.test(source)) {
    throw new Error("runtime perf harness must not fall back to stale crates/target/debug probe binaries");
  }
}

function functionSource(source, functionName, nextFunctionName) {
  const start = source.indexOf(`function ${functionName}(`);
  const end = source.indexOf(`function ${nextFunctionName}(`);
  if (start < 0 || end < 0 || end <= start) {
    throw new Error(`runtime perf harness is missing expected ${functionName} function boundary`);
  }
  return source.slice(start, end);
}
