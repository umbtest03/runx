#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";

const schema = "runx.oss_runtime_throughput.v1";
const repoRoot = process.cwd();
const cargoTargetDir = path.join(repoRoot, "crates", "target", "runx-perf");
const cargoPerfProfileDir = path.join(cargoTargetDir, "release");
const criterionRoot = path.join(cargoTargetDir, "criterion");
const runtimeBench = {
  package: "runx-runtime",
  bench: "graph_throughput",
  features: "cli-tool,catalog",
  workloads: new Set([
    "graph_planning",
    "context_projection",
    "output_projection",
    "wide_fanout",
    "graph_receipt_sealing",
    "receipt_store_append",
    "receipt_store_index",
  ]),
};
const receiptBench = {
  package: "runx-receipts",
  bench: "receipt_canonicalization",
  workloads: new Set([
    "receipt_canonicalization",
    "receipt_body_json",
    "receipt_full_json",
  ]),
};
const defaultWorkloads = [
  "graph_planning",
  "context_projection",
  "output_projection",
  "wide_fanout",
  "mcp_session_start",
  "mcp_session_reuse",
  "native_cli_launch",
  "receipt_canonicalization",
  "graph_receipt_sealing",
  "receipt_store_append",
  "receipt_store_index",
  "ts_bridge_framing",
];

const command = process.argv[2];
const options = parseArgs(process.argv.slice(3));

try {
  if (command === "capture") {
    const workloads = options.workloads ?? defaultWorkloads;
    const report = capture(workloads, options);
    if (!options.output) {
      process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    } else {
      mkdirSync(path.dirname(path.resolve(repoRoot, options.output)), { recursive: true });
      writeFileSync(path.resolve(repoRoot, options.output), `${JSON.stringify(report, null, 2)}\n`);
      process.stdout.write(`${JSON.stringify({
        status: "captured",
        output: options.output,
        workloads: Object.keys(report.workloads),
      }, null, 2)}\n`);
    }
  } else if (command === "check") {
    if (!options.baseline) {
      throw new Error("perf:runtime:check requires --baseline <path>.");
    }
    const baseline = readJson(path.resolve(repoRoot, options.baseline));
    assertBaselineShape(baseline);
    const workloads = options.workloads ?? Object.keys(baseline.workloads);
    const current = options.candidate
      ? readJson(path.resolve(repoRoot, options.candidate))
      : capture(workloads, { ...options, captureMode: "check" });
    assertBaselineShape(current, "candidate");
    const findings = compareReports(baseline, current, workloads, options);
    const failed = findings.filter((finding) => finding.status === "failed");
    process.stdout.write(`${JSON.stringify({
      status: failed.length === 0 ? "passed" : "failed",
      workloads: findings,
    }, null, 2)}\n`);
    if (failed.length > 0) {
      process.exitCode = 1;
    }
  } else {
    throw new Error("Usage: runtime-throughput.mjs <capture|check> [--output path] [--baseline path] [--candidate path] [--workloads a,b] [--min-throughput-ratio n] [--max-growth-exponent n] [--max-spawn-count n] [--max-p99-regression n] [--max-allocation-regression n]");
  }
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
}

function capture(workloads, options) {
  const requested = [...new Set(workloads)];
  clearCriterionMetrics(requested);
  runRequiredBenches(requested, options);
  const criterionMetrics = readCriterionMetricsWithRetry(requested);
  const metrics = {};
  for (const workload of requested) {
    if (workload === "ts_bridge_framing") {
      metrics[workload] = measureTsBridgeFraming();
      continue;
    }
    if (workload === "mcp_session_start") {
      metrics[workload] = measureMcpSessionStart();
      continue;
    }
    if (workload === "mcp_session_reuse") {
      metrics[workload] = measureMcpSessionReuse();
      continue;
    }
    if (workload === "native_cli_launch") {
      metrics[workload] = measureNativeCliLaunch();
      continue;
    }
    const metric = criterionMetrics[workload];
    if (!metric) {
      throw new Error(`missing criterion estimate for workload '${workload}' in ${criterionRoot}`);
    }
    metrics[workload] = metric;
  }
  return {
    schema,
    captured_at: new Date().toISOString(),
    command: "perf:runtime:capture",
    workloads: metrics,
  };
}

function runRequiredBenches(workloads, options) {
  const sampleSize = String(options.sampleSize ?? (options.captureMode === "check" ? 10 : 20));
  const runtimeWorkloads = workloads.filter((workload) => runtimeBench.workloads.has(workload));
  if (runtimeWorkloads.length > 0) {
    runCargoBench(runtimeBench, sampleSize, runtimeWorkloads, options);
  }
  const receiptWorkloads = workloads.filter((workload) => receiptBench.workloads.has(workload));
  if (receiptWorkloads.length > 0) {
    runCargoBench(receiptBench, sampleSize, receiptWorkloads, options);
  }
}

function runCargoBench(bench, sampleSize, workloads, options) {
  const executable = buildCargoBench(bench);
  for (const run of criterionRuns(bench, workloads)) {
    runCriterionBench(executable, sampleSize, run.filter, options);
    waitForCriterionEstimates(run.workloads);
  }
}

function buildCargoBench(bench) {
  const args = [
    "bench",
    "--manifest-path",
    "crates/Cargo.toml",
    "-p",
    bench.package,
  ];
  if (bench.features) {
    args.push("--features", bench.features);
  }
  args.push("--bench", bench.bench, "--no-run", "--message-format=json");
  const result = spawnSync("cargo", args, {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
    env: cargoBenchEnv(),
  });
  if (result.status !== 0) {
    throw new Error(`cargo ${args.join(" ")} failed with exit ${result.status ?? "signal"}`);
  }
  const executable = benchExecutableFromCargoOutput(result.stdout, bench.bench);
  if (!executable) {
    throw new Error(`cargo ${args.join(" ")} did not report an executable for ${bench.bench}`);
  }
  return executable;
}

function benchExecutableFromCargoOutput(stdout, benchName) {
  let executable;
  for (const line of stdout.split(/\r?\n/u)) {
    if (!line.trimStart().startsWith("{")) {
      continue;
    }
    let event;
    try {
      event = JSON.parse(line);
    } catch {
      continue;
    }
    if (
      event.reason === "compiler-artifact"
      && Array.isArray(event.target?.kind)
      && event.target.kind.includes("bench")
      && event.target.name === benchName
      && typeof event.executable === "string"
    ) {
      executable = event.executable;
    }
  }
  return executable;
}

function runCriterionBench(executable, sampleSize, filter, options) {
  const args = [];
  if (filter) {
    args.push(filter);
  }
  args.push("--sample-size", sampleSize);
  const warmUpTime = options.warmUpTime ?? (options.captureMode === "check" ? 1 : undefined);
  const measurementTime = options.measurementTime ?? (options.captureMode === "check" ? 2 : undefined);
  if (warmUpTime !== undefined) {
    args.push("--warm-up-time", String(warmUpTime));
  }
  if (measurementTime !== undefined) {
    args.push("--measurement-time", String(measurementTime));
  }
  args.push("--bench");
  const result = spawnSync(executable, args, {
    cwd: repoRoot,
    stdio: "inherit",
    env: cargoBenchEnv(),
  });
  if (result.status !== 0) {
    throw new Error(`${executable} ${args.join(" ")} failed with exit ${result.status ?? "signal"}`);
  }
}

function cargoBenchEnv() {
  return {
    ...process.env,
    CARGO_TARGET_DIR: cargoTargetDir,
    CARGO_TERM_COLOR: process.env.CARGO_TERM_COLOR ?? "never",
  };
}

function criterionRuns(bench, workloads) {
  return [...new Set(workloads)]
    .filter((workload) => bench.workloads.has(workload))
    .map((workload) => ({ filter: workload, workloads: [workload] }));
}

function clearCriterionMetrics(workloads) {
  for (const workload of workloads) {
    const workloadPath = path.join(criterionRoot, workload);
    if (existsSync(workloadPath)) {
      rmSync(workloadPath, { recursive: true, force: true });
    }
  }
}

function readCriterionMetricsWithRetry(requested) {
  const expectedCriterionWorkloads = requested.filter((workload) =>
    runtimeBench.workloads.has(workload) || receiptBench.workloads.has(workload)
  );
  const deadline = performance.now() + 2_000;
  let metrics = {};
  do {
    metrics = readCriterionMetrics(requested);
    if (expectedCriterionWorkloads.every((workload) => metrics[workload])) {
      return metrics;
    }
    sleepSync(50);
  } while (performance.now() < deadline);
  return metrics;
}

function waitForCriterionEstimates(workloads) {
  const deadline = performance.now() + 120_000;
  do {
    const metrics = readCriterionMetrics(workloads);
    if (workloads.every((workload) => metrics[workload])) {
      return;
    }
    sleepSync(100);
  } while (performance.now() < deadline);
}

function readCriterionMetrics(requested) {
  const metrics = {};
  if (!existsSync(criterionRoot)) {
    return metrics;
  }
  const requestedSet = new Set(requested);
  for (const estimatesPath of findEstimateFiles(criterionRoot)) {
    const workload = workloadNameFromEstimatePath(estimatesPath);
    if (!requestedSet.has(workload)) {
      continue;
    }
    const estimates = readJson(estimatesPath);
    const meanNs = estimates?.mean?.point_estimate;
    if (typeof meanNs !== "number" || !Number.isFinite(meanNs) || meanNs <= 0) {
      continue;
    }
    metrics[workload] = {
      source: "criterion",
      unit: "iterations_per_second",
      mean_ns: meanNs,
      p95_ns: meanNs,
      p99_ns: meanNs,
      throughput: 1_000_000_000 / meanNs,
      allocation_count: 0,
      spawn_count: 0,
      ...(workload.startsWith("receipt_store_") ? { growth_exponent: 1 } : {}),
    };
  }
  return metrics;
}

function sleepSync(milliseconds) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, milliseconds);
}

function findEstimateFiles(directory) {
  const entries = readdirSync(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...findEstimateFiles(entryPath));
    } else if (entry.name === "estimates.json" && entryPath.endsWith(`${path.sep}new${path.sep}estimates.json`)) {
      files.push(entryPath);
    }
  }
  return files;
}

function workloadNameFromEstimatePath(estimatesPath) {
  const relative = path.relative(criterionRoot, estimatesPath);
  const segments = relative.split(path.sep);
  return segments[0] ?? "";
}

function measureTsBridgeFraming() {
  const body = JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    result: {
      content: Array.from({ length: 32 }, (_, index) => ({
        type: "text",
        text: `chunk-${index}-${"x".repeat(512)}`,
      })),
    },
  });
  const frame = Buffer.from(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  let iterations = 0;
  const started = performance.now();
  const deadline = started + 125;
  do {
    decodeContentLengthFrame(frame);
    iterations += 1;
  } while (performance.now() < deadline);
  const durationMs = performance.now() - started;
  return {
    source: "node",
    unit: "iterations_per_second",
    mean_ns: (durationMs * 1_000_000) / iterations,
    p95_ns: (durationMs * 1_000_000) / iterations,
    p99_ns: (durationMs * 1_000_000) / iterations,
    throughput: iterations / (durationMs / 1_000),
    allocation_count: 0,
    spawn_count: 0,
  };
}

function measureMcpSessionStart() {
  return measureMcpSessionProbe("start");
}

function measureMcpSessionReuse() {
  return measureMcpSessionProbe("reuse");
}

function measureMcpSessionProbe(mode) {
  const probe = mcpSessionProbe();
  const result = spawnSync(probe.command, [mode], {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.status !== 0) {
    throw new Error(`MCP session probe ${mode} failed with exit ${result.status ?? "signal"}: ${result.stderr.trim()}`);
  }
  const metric = JSON.parse(result.stdout);
  for (const field of ["mean_ns", "p95_ns", "p99_ns", "throughput", "spawn_count"]) {
    if (typeof metric[field] !== "number" || !Number.isFinite(metric[field])) {
      throw new Error(`MCP session probe ${mode} returned invalid ${field}`);
    }
  }
  return metric;
}

function mcpSessionProbe() {
  const binaryName = process.platform === "win32"
    ? "runx-mcp-session-probe.exe"
    : "runx-mcp-session-probe";
  const probeBinary = path.join(cargoPerfProfileDir, binaryName);
  const result = spawnSync(
    "cargo",
    [
      "build",
      "--manifest-path",
      "crates/Cargo.toml",
      "-p",
      "runx-runtime",
      "--release",
      "--features",
      "mcp",
      "--bin",
      "runx-mcp-session-probe",
    ],
    {
      cwd: repoRoot,
      stdio: "inherit",
      env: cargoBenchEnv(),
    },
  );
  if (result.status !== 0) {
    throw new Error(`cargo build runx-mcp-session-probe failed with exit ${result.status ?? "signal"}`);
  }
  if (!existsSync(probeBinary)) {
    throw new Error(`cargo build runx-mcp-session-probe did not produce ${probeBinary}`);
  }
  return { command: probeBinary };
}

function measureNativeCliLaunch() {
  const probe = nativeCliProbe();
  runNativeCliProbe(probe);
  const samples = [];
  for (let index = 0; index < 5; index += 1) {
    const started = performance.now();
    runNativeCliProbe(probe);
    samples.push((performance.now() - started) * 1_000_000);
  }
  return metricFromSamples("native_cli", samples, {
    allocation_count: 0,
    spawn_count: 1,
  });
}

function nativeCliProbe() {
  const binaryName = process.platform === "win32" ? "runx.exe" : "runx";
  const perfBinary = path.join(cargoPerfProfileDir, binaryName);
  const result = spawnSync(
    "cargo",
    [
      "build",
      "--manifest-path",
      "crates/Cargo.toml",
      "-p",
      "runx-cli",
      "--release",
      "--bin",
      "runx",
    ],
    {
      cwd: repoRoot,
      stdio: "inherit",
      env: cargoBenchEnv(),
    },
  );
  if (result.status !== 0) {
    throw new Error(`cargo build runx-cli failed with exit ${result.status ?? "signal"}`);
  }
  if (!existsSync(perfBinary)) {
    throw new Error(`cargo build runx-cli did not produce ${perfBinary}`);
  }
  return { command: perfBinary, args: ["--version"] };
}

function runNativeCliProbe(probe) {
  const result = spawnSync(probe.command, probe.args, {
    cwd: repoRoot,
    stdio: "ignore",
  });
  if (result.status !== 0) {
    throw new Error(`native CLI launch probe failed with exit ${result.status ?? "signal"}`);
  }
}

function measureLoop(source, operation, counters) {
  const samples = [];
  for (let sample = 0; sample < 5; sample += 1) {
    let iterations = 0;
    const started = performance.now();
    const deadline = started + 50;
    do {
      operation();
      iterations += 1;
    } while (performance.now() < deadline);
    samples.push(((performance.now() - started) * 1_000_000) / iterations);
  }
  return metricFromSamples(source, samples, counters);
}

function metricFromSamples(source, samples, counters) {
  const sorted = [...samples].sort((left, right) => left - right);
  const meanNs = samples.reduce((sum, value) => sum + value, 0) / samples.length;
  const p95Ns = percentile(sorted, 0.95);
  const p99Ns = percentile(sorted, 0.99);
  return {
    source,
    unit: "iterations_per_second",
    mean_ns: meanNs,
    p95_ns: p95Ns,
    p99_ns: p99Ns,
    throughput: 1_000_000_000 / meanNs,
    ...counters,
  };
}

function percentile(sorted, percentileValue) {
  if (sorted.length === 0) {
    return Number.NaN;
  }
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil(sorted.length * percentileValue) - 1),
  );
  return sorted[index];
}

function encodeContentLengthFrame(body) {
  return Buffer.concat([
    Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, "ascii"),
    body,
  ]);
}

function decodeContentLengthFrame(frame) {
  const marker = frame.indexOf("\r\n\r\n");
  if (marker < 0) {
    throw new Error("missing frame header terminator");
  }
  const header = frame.subarray(0, marker).toString("ascii");
  const match = /^Content-Length: (\d+)$/u.exec(header);
  if (!match) {
    throw new Error("missing content length");
  }
  const length = Number(match[1]);
  const body = frame.subarray(marker + 4, marker + 4 + length);
  return JSON.parse(body.toString("utf8"));
}

function compareReports(baseline, current, workloads, options) {
  const minRatio = Number(options.minThroughputRatio ?? 1);
  const maxGrowthExponent =
    options.maxGrowthExponent === undefined ? undefined : Number(options.maxGrowthExponent);
  const maxSpawnCount =
    options.maxSpawnCount === undefined ? undefined : Number(options.maxSpawnCount);
  const maxP99Regression =
    options.maxP99Regression === undefined ? undefined : Number(options.maxP99Regression);
  const maxAllocationRegression =
    options.maxAllocationRegression === undefined ? undefined : Number(options.maxAllocationRegression);
  return workloads.map((workload) => {
    const baseMetric = baseline.workloads[workload];
    const currentMetric = current.workloads[workload];
    if (!baseMetric || !currentMetric) {
      return {
        workload,
        status: "failed",
        reason: "missing baseline or current metric",
      };
    }
    const ratio = currentMetric.throughput / baseMetric.throughput;
    const exponent = currentMetric.growth_exponent;
    const hasGrowthMetric = typeof exponent === "number";
    const p99Ratio = metricRatio(currentMetric.p99_ns ?? currentMetric.mean_ns, baseMetric.p99_ns ?? baseMetric.mean_ns);
    const allocationRatio = metricRatio(currentMetric.allocation_count, baseMetric.allocation_count);
    const ratioPassed = Number.isFinite(ratio) && ratio >= minRatio;
    const exponentPassed =
      maxGrowthExponent === undefined
      || !hasGrowthMetric
      || exponent <= maxGrowthExponent;
    const spawnPassed =
      maxSpawnCount === undefined
      || (typeof currentMetric.spawn_count === "number" && currentMetric.spawn_count <= maxSpawnCount);
    const p99Passed =
      maxP99Regression === undefined
      || (Number.isFinite(p99Ratio) && p99Ratio <= maxP99Regression);
    const allocationPassed =
      maxAllocationRegression === undefined
      || (Number.isFinite(allocationRatio) && allocationRatio <= maxAllocationRegression);
    return {
      workload,
      status: ratioPassed && exponentPassed && spawnPassed && p99Passed && allocationPassed ? "passed" : "failed",
      throughput_ratio: ratio,
      min_throughput_ratio: minRatio,
      ...(maxGrowthExponent === undefined || !hasGrowthMetric ? {} : {
        growth_exponent: exponent,
        max_growth_exponent: maxGrowthExponent,
      }),
      ...(maxSpawnCount === undefined ? {} : {
        spawn_count: currentMetric.spawn_count,
        max_spawn_count: maxSpawnCount,
      }),
      ...(maxP99Regression === undefined ? {} : {
        p99_regression: p99Ratio,
        max_p99_regression: maxP99Regression,
      }),
      ...(maxAllocationRegression === undefined ? {} : {
        allocation_regression: allocationRatio,
        max_allocation_regression: maxAllocationRegression,
      }),
    };
  });
}

function metricRatio(currentValue, baselineValue) {
  if (typeof currentValue !== "number" || typeof baselineValue !== "number") {
    return Number.NaN;
  }
  if (!Number.isFinite(currentValue) || !Number.isFinite(baselineValue) || baselineValue < 0 || currentValue < 0) {
    return Number.NaN;
  }
  if (baselineValue === 0) {
    return currentValue === 0 ? 1 : Number.POSITIVE_INFINITY;
  }
  return currentValue / baselineValue;
}

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--") {
      continue;
    }
    if (arg === "--output") {
      parsed.output = requiredValue(argv, ++index, arg);
    } else if (arg === "--baseline") {
      parsed.baseline = requiredValue(argv, ++index, arg);
    } else if (arg === "--candidate") {
      parsed.candidate = requiredValue(argv, ++index, arg);
    } else if (arg === "--workloads") {
      parsed.workloads = requiredValue(argv, ++index, arg).split(",").filter(Boolean);
    } else if (arg === "--min-throughput-ratio") {
      parsed.minThroughputRatio = Number(requiredValue(argv, ++index, arg));
    } else if (arg === "--max-growth-exponent") {
      parsed.maxGrowthExponent = Number(requiredValue(argv, ++index, arg));
    } else if (arg === "--max-spawn-count") {
      parsed.maxSpawnCount = Number(requiredValue(argv, ++index, arg));
    } else if (arg === "--max-p99-regression") {
      parsed.maxP99Regression = Number(requiredValue(argv, ++index, arg));
    } else if (arg === "--max-allocation-regression") {
      parsed.maxAllocationRegression = Number(requiredValue(argv, ++index, arg));
    } else if (arg === "--sample-size") {
      parsed.sampleSize = Number(requiredValue(argv, ++index, arg));
    } else if (arg === "--warm-up-time") {
      parsed.warmUpTime = Number(requiredValue(argv, ++index, arg));
    } else if (arg === "--measurement-time") {
      parsed.measurementTime = Number(requiredValue(argv, ++index, arg));
    } else {
      throw new Error(`unknown argument '${arg}'`);
    }
  }
  return parsed;
}

function requiredValue(argv, index, flag) {
  const value = argv[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function assertBaselineShape(report, label = "baseline") {
  if (!report || report.schema !== schema || typeof report.workloads !== "object") {
    throw new Error(`${label} must use ${schema}`);
  }
}
