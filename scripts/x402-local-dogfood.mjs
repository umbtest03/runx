#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const node = process.execPath;

const checks = [
  {
    name: "runx-payment-receipts",
    command: [node, "scripts/check-demos.mjs"],
    required: true,
  },
  {
    name: "upstream-x402-preflight",
    command: [node, "scripts/x402-upstream-conformance.mjs", "--check"],
    required: false,
  },
  {
    name: "x402-rs-preflight",
    command: [node, "scripts/x402-interop.mjs", "--target", "x402-rs", "--check"],
    required: false,
  },
  {
    name: "cdp-hosted-facilitator-plan",
    command: [node, "scripts/x402-interop.mjs", "--target", "cdp", "--check"],
    required: false,
  },
];

for (const check of checks) {
  log(`start ${check.name}`);
  const result = spawnSync(check.command[0], check.command.slice(1), {
    cwd: root,
    env: process.env,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });

  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);

  if (result.status === 0) {
    log(`pass ${check.name}`);
    continue;
  }

  if (!check.required && printedPreflightReport(result.stdout)) {
    log(`info ${check.name} needs upstream checkout, credentials, or funded testnet env`);
    continue;
  }

  process.stderr.write(`[x402:dogfood:local] fail ${check.name} exit=${result.status}\n`);
  process.exit(result.status || 1);
}

log("pass zero-funded dogfood");

function printedPreflightReport(stdout) {
  try {
    const report = JSON.parse(stdout);
    return (
      report &&
      typeof report.schema === "string" &&
      (report.schema === "runx.x402.upstream_conformance.v1" || report.schema === "runx.x402.interop.v1")
    );
  } catch {
    return false;
  }
}

function log(message) {
  process.stderr.write(`[x402:dogfood:local] ${message}\n`);
}
