#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const apiOnly = process.argv.includes("--api-only");

const commands = apiOnly
  ? [checkPublicApiSnapshot]
  : [
      checkCargo,
      checkTooling,
      () => run("cargo", ["fmt", "--manifest-path", "crates/Cargo.toml", "--all", "--check"]),
      () => run("cargo", ["clippy", "--manifest-path", "crates/Cargo.toml", "--workspace", "--all-targets", "--", "-D", "warnings"]),
      () => run("cargo", ["test", "--manifest-path", "crates/Cargo.toml", "--workspace"]),
      () => run("node", ["scripts/check-rust-crate-graph.mjs"]),
      () => run("node", ["scripts/check-rust-core-style.mjs"]),
      // Keep these strings contiguous for scafld source checks:
      // cargo deny
      // cargo public-api
      () => run("cargo", ["deny", "--manifest-path", "crates/Cargo.toml", "check", "bans", "licenses", "sources"]),
      checkPublicApiSnapshot,
    ];

for (const command of commands) {
  command();
}

function checkCargo() {
  const result = spawnSync("cargo", ["--version"], { encoding: "utf8" });
  if (result.status === 0) {
    return;
  }
  console.error("cargo is not installed. Install Rust with rustup: https://rustup.rs/");
  console.error("After installing Rust, rerun: node scripts/check-rust-kernel-parity.mjs");
  process.exit(1);
}

function checkTooling() {
  const missing = [];
  if (spawnSync("cargo", ["deny", "--version"], { encoding: "utf8" }).status !== 0) {
    missing.push("cargo-deny");
  }
  if (spawnSync("cargo", ["public-api", "--version"], { encoding: "utf8" }).status !== 0) {
    missing.push("cargo-public-api");
  }
  if (missing.length === 0) {
    return;
  }
  console.error(`missing Cargo parity tool(s): ${missing.join(", ")}`);
  console.error("Install optional Rust parity tools with:");
  console.error("  cargo install cargo-deny cargo-public-api");
  console.error("cargo-public-api also needs nightly rustdoc JSON:");
  console.error("  rustup toolchain install nightly --profile minimal");
  process.exit(1);
}

function checkPublicApiSnapshot() {
  checkCargo();
  if (spawnSync("cargo", ["public-api", "--version"], { encoding: "utf8" }).status !== 0) {
    console.error("missing Cargo parity tool: cargo-public-api");
    console.error("Install it with: cargo install cargo-public-api");
    process.exit(1);
  }

  const result = spawnSync(
    "cargo",
    ["public-api", "--manifest-path", "crates/runx-core/Cargo.toml", "-sss"],
    { encoding: "utf8" },
  );
  if (result.status !== 0) {
    process.stderr.write(result.stderr);
    process.stderr.write(result.stdout);
    process.exit(result.status ?? 1);
  }

  const expectedPath = "crates/runx-core/api-snapshot.txt";
  const expected = readFileSync(expectedPath, "utf8");
  const actual = result.stdout.endsWith("\n") ? result.stdout : `${result.stdout}\n`;
  if (actual === expected) {
    return;
  }

  console.error(`${expectedPath} is stale; regenerate with:`);
  console.error("  cargo public-api --manifest-path crates/runx-core/Cargo.toml -sss > crates/runx-core/api-snapshot.txt");
  process.exit(1);
}

function run(command, args) {
  const result = spawnSync(command, args, { stdio: "inherit" });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
