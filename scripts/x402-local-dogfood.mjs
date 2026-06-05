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
  {
    name: "stripe-spt-preflight",
    command: [node, "scripts/stripe-spt-charge.mjs", "--check"],
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

  const preflight = !check.required ? parsePreflightReport(result.stdout) : undefined;
  if (preflight?.can_run === false) {
    log(`info ${check.name} live blocker: ${preflightBlockerSummary(preflight)}`);
    continue;
  }

  if (result.status === 0) {
    log(`pass ${check.name}`);
    continue;
  }

  if (preflight) {
    log(`info ${check.name} needs external checkout, credentials, or funded testnet env`);
    continue;
  }

  process.stderr.write(`[x402:dogfood:local] fail ${check.name} exit=${result.status}\n`);
  process.exit(result.status || 1);
}

log("pass zero-funded dogfood");

function parsePreflightReport(stdout) {
  try {
    const report = JSON.parse(stdout);
    if (
      report &&
      typeof report.schema === "string" &&
      (report.schema === "runx.x402.upstream_conformance.v1" ||
        report.schema === "runx.x402.interop.v1" ||
        report.schema === "runx.stripe_spt.preflight.v1")
    ) {
      return report;
    }
  } catch {
    return undefined;
  }
  return undefined;
}

function preflightBlockerSummary(report) {
  const blockers = [];
  if (report.upstream_available === false && report.upstream_dir) {
    blockers.push(`missing checkout: ${report.upstream_dir}`);
  }
  if (report.target_available === false && report.target_dir) {
    blockers.push(`missing checkout: ${report.target_dir}`);
  }
  if (Array.isArray(report.missing_env) && report.missing_env.length > 0) {
    blockers.push(`missing env: ${report.missing_env.join(", ")}`);
  }
  if (Array.isArray(report.invalid_env) && report.invalid_env.length > 0) {
    const names = report.invalid_env.map((item) => item?.name || String(item));
    blockers.push(`invalid env: ${names.join(", ")}`);
  }
  if (Array.isArray(report.required_external) && report.required_external.length > 0) {
    blockers.push(`external required: ${report.required_external.join("; ")}`);
  }
  if (report.credential_env_contract === "not_implemented") {
    blockers.push("credential env contract not implemented");
  }
  return blockers.length > 0 ? blockers.join("; ") : "external live resources not ready";
}

function log(message) {
  process.stderr.write(`[x402:dogfood:local] ${message}\n`);
}
