#!/usr/bin/env node
import { spawn } from "node:child_process";
import { basename } from "node:path";
import { readFileSync, writeFileSync } from "node:fs";

const mode = process.argv[2] ?? "inspect-env";

if (mode === "inspect-stdin") {
  const raw = await readStdin();
  const inputs = raw.trim().length === 0 ? {} : JSON.parse(raw);
  writeJson({
    inputs_source: "stdin",
    message: inputs.message ?? null,
    mode,
    stdin_keys: Object.keys(inputs).sort(),
  });
} else if (mode === "inspect-large-input") {
  const { inputs, source } = readEnvInputs();
  writeJson({
    inputs_source: source,
    large_env_present: Object.hasOwn(process.env, "RUNX_INPUT_LARGE"),
    large_length: typeof inputs.large === "string" ? inputs.large.length : null,
    message: inputs.message ?? null,
    mode,
  });
} else if (mode === "large-output") {
  process.stdout.write("a".repeat(2 * 1024 * 1024));
} else if (mode === "timeout-descendant") {
  spawnTimeoutDescendant();
  setInterval(() => undefined, 1000);
} else {
  const { inputs, source } = readEnvInputs();
  writeJson({
    cwd_basename: basename(process.cwd()),
    inputs_source: source,
    message: inputs.message ?? null,
    mode,
    repeated_separator_env: process.env.RUNX_INPUT_REPEATED_SEPARATOR ?? null,
    runx_cwd_basename: process.env.RUNX_CWD ? basename(process.env.RUNX_CWD) : null,
    thread_title_env: process.env.RUNX_INPUT_THREAD_TITLE ?? null,
  });
}

function readEnvInputs() {
  if (process.env.RUNX_INPUTS_PATH) {
    return {
      inputs: JSON.parse(readFileSync(process.env.RUNX_INPUTS_PATH, "utf8")),
      source: "path",
    };
  }
  return {
    inputs: JSON.parse(process.env.RUNX_INPUTS_JSON ?? "{}"),
    source: "json",
  };
}

function spawnTimeoutDescendant() {
  const sentinelPath = process.env.RUNX_SENTINEL_PATH ?? process.env.RUNX_INPUT_SENTINEL_PATH;
  if (!sentinelPath) {
    throw new Error("RUNX_SENTINEL_PATH is required");
  }
  const script = [
    `setTimeout(() => require("node:fs").writeFileSync(${JSON.stringify(sentinelPath)}, "survived"), 300);`,
    "setInterval(() => undefined, 1000);",
  ].join("");
  spawn(process.execPath, ["-e", script], { stdio: "ignore" });
}

function readStdin() {
  return new Promise((resolve, reject) => {
    let raw = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => {
      raw += chunk;
    });
    process.stdin.on("error", reject);
    process.stdin.on("end", () => resolve(raw));
  });
}

function writeJson(value) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}
