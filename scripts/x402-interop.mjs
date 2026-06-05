#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

const TARGETS = new Set(["x402-rs", "cdp"]);
const DEFAULT_X402_RS_DIR = "/tmp/x402-rs";
const DEFAULT_X402_RS_TEST = "src/tests/v2-eip155-exact-ts-ts-rs.test.ts";
const X402_RS_REQUIRED_ENV = [
  "BASE_SEPOLIA_RPC_URL",
  "BASE_SEPOLIA_BUYER_PRIVATE_KEY",
  "BASE_SEPOLIA_FACILITATOR_PRIVATE_KEY",
  // x402-rs protocol-compliance currently validates Solana env at module load,
  // even for an EVM-only test selection.
  "SOLANA_DEVNET_RPC_URL",
  "SOLANA_DEVNET_BUYER_PRIVATE_KEY",
  "SOLANA_DEVNET_FACILITATOR_PRIVATE_KEY",
];

const args = process.argv.slice(2);

if (args.includes("--help") || args.includes("-h")) {
  usage(0);
}

const target = option("--target") || process.env.RUNX_X402_INTEROP_TARGET || "x402-rs";
if (!TARGETS.has(target)) {
  fail(`unsupported target '${target}'. Expected one of: ${Array.from(TARGETS).join(", ")}`);
}

const mode = args.includes("--run") ? "run" : "check";
const report = target === "x402-rs" ? x402RsReport(mode) : cdpReport(mode);

if (mode === "check") {
  write(report);
  process.exit(report.target_available === false ? 1 : 0);
}

if (target === "cdp") {
  write(report);
  fail("CDP hosted-facilitator live run is not implemented; use --check for the no-secret preflight report");
}

if (target === "x402-rs") {
  runX402Rs(report);
}

function x402RsReport(selectedMode) {
  const repoDir = option("--repo-dir") || process.env.X402_RS_DIR || DEFAULT_X402_RS_DIR;
  const artifactDir =
    option("--artifact-dir") || process.env.RUNX_X402_INTEROP_ARTIFACT_DIR || path.join(os.tmpdir(), "runx-x402-rs-interop");
  const testFile = option("--test") || process.env.RUNX_X402_RS_TEST || DEFAULT_X402_RS_TEST;
  const complianceDir = path.join(repoDir, "protocol-compliance");
  const upstream = inspectGitRepo(repoDir, path.join(complianceDir, "package.json"));
  const missingEnv = X402_RS_REQUIRED_ENV.filter((name) => !process.env[name]);
  const commands = [
    ["pnpm", "--dir", complianceDir, "install", "--frozen-lockfile"],
    ["cargo", "build", "--manifest-path", path.join(repoDir, "Cargo.toml"), "--package", "x402-facilitator"],
    ["pnpm", "--dir", complianceDir, "exec", "vitest", "run", testFile, "--reporter=verbose"],
  ];

  return {
    schema: "runx.x402.interop.v1",
    mode: selectedMode,
    target: "x402-rs",
    target_kind: "independent_implementation",
    target_repo: "https://github.com/x402-rs/x402-rs",
    target_dir: repoDir,
    target_available: upstream.available,
    target_sha: upstream.sha,
    artifact_dir: artifactDir,
    test_file: testFile,
    required_env: X402_RS_REQUIRED_ENV,
    missing_env: missingEnv,
    commands,
    can_run: upstream.available && missingEnv.length === 0,
    notes: [
      "This is an interop lane, not the canonical x402 standard conformance lane.",
      "The default test is TS client + TS server + Rust facilitator on v2 EIP-155 exact.",
      "Use dedicated funded testnet wallets only; x402-rs protocol-compliance performs real settlement.",
    ],
  };
}

function cdpReport(selectedMode) {
  return {
    schema: "runx.x402.interop.v1",
    mode: selectedMode,
    target: "cdp",
    target_kind: "hosted_facilitator",
    target_status: "planned",
    can_run: false,
    facilitator_url: "https://api.cdp.coinbase.com/platform/v2/x402",
    testnet_fallback_url: "https://x402.org/facilitator",
    network: "eip155:84532",
    scheme: "exact",
    token_path: "USDC / EIP-3009",
    required_external: [
      "CDP API credentials for hosted-facilitator authentication",
      "Dedicated funded Base Sepolia payer wallet for the v2 exact flow",
      "Operator-owned receipt/artifact directory outside the repository",
    ],
    missing_env: [],
    credential_env_contract: "not_implemented",
    required_next_step:
      "Add a hosted-facilitator run profile using official CDP authentication, then run the same Base Sepolia v2 exact flow against CDP.",
    notes: [
      "CDP is a hosted facilitator target, not a repository checkout.",
      "CDP supports Base Sepolia and Solana Devnet as well as mainnet networks; CDP endpoint requires API keys.",
      "The signup-free x402.org facilitator is testnet-only and useful before CDP credentials are available.",
    ],
  };
}

function runX402Rs(report) {
  if (!report.target_available) {
    write(report);
    fail(`x402-rs checkout not found at ${report.target_dir}`);
  }
  if (report.missing_env.length > 0) {
    write(report);
    fail(`missing required environment variables: ${report.missing_env.join(", ")}`);
  }

  mkdirSync(report.artifact_dir, { recursive: true });
  writeFileSync(path.join(report.artifact_dir, "x402-rs-interop-preflight.json"), `${JSON.stringify(report, null, 2)}\n`);

  for (const command of report.commands) {
    const result = spawnSync(command[0], command.slice(1), {
      cwd: report.target_dir,
      env: process.env,
      stdio: "inherit",
    });
    if (result.status !== 0) {
      process.exit(result.status ?? 1);
    }
  }
}

function inspectGitRepo(dir, requiredFile) {
  if (!existsSync(requiredFile)) {
    return { available: false, sha: null };
  }
  const result = spawnSync("git", ["-C", dir, "rev-parse", "HEAD"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  return {
    available: result.status === 0,
    sha: result.status === 0 ? result.stdout.trim() : null,
  };
}

function option(name) {
  const index = args.indexOf(name);
  if (index !== -1) {
    return args[index + 1];
  }
  const prefix = `${name}=`;
  const inline = args.find((arg) => arg.startsWith(prefix));
  return inline ? inline.slice(prefix.length) : undefined;
}

function write(value) {
  process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
}

function fail(message) {
  process.stderr.write(`x402-interop: ${message}\n`);
  process.exit(1);
}

function usage(code) {
  process.stderr.write(
    [
      "usage:",
      "  node scripts/x402-interop.mjs --target x402-rs --check [--repo-dir DIR] [--artifact-dir DIR]",
      "  node scripts/x402-interop.mjs --target x402-rs --run [--repo-dir DIR] [--artifact-dir DIR]",
      "  node scripts/x402-interop.mjs --target cdp --check",
      "",
      "default x402-rs repo dir: /tmp/x402-rs",
      "default x402-rs test: src/tests/v2-eip155-exact-ts-ts-rs.test.ts",
    ].join("\n") + "\n",
  );
  process.exit(code);
}
