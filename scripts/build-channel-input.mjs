import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

// Collects the per-target release-archive checksums into the single input that
// gen-channel-manifests.ts consumes. Reads the *.sha256 sidecars produced by
// build-release-archives.ts and keys them by rust target triple.

const options = parseArgs(process.argv.slice(2));
const TARGETS = [
  "aarch64-apple-darwin",
  "x86_64-apple-darwin",
  "aarch64-unknown-linux-musl",
  "x86_64-unknown-linux-musl",
  "x86_64-pc-windows-msvc",
];

const artifacts = {};
for (const entry of readdirSync(options.archives)) {
  if (!entry.endsWith(".sha256")) continue;
  const line = readFileSync(path.join(options.archives, entry), "utf8").trim();
  const [sha256, file] = line.split(/\s+/u);
  const target = TARGETS.find((t) => file.includes(t));
  if (!target) {
    console.warn(`skipping unrecognized archive: ${file}`);
    continue;
  }
  artifacts[target] = { file, sha256 };
}

const missing = TARGETS.filter((t) => !artifacts[t]);
if (missing.length > 0) {
  throw new Error(`missing release archives for targets: ${missing.join(", ")}`);
}

const manifest = {
  version: options.version,
  repo: options.repo,
  tag: options.tag,
  homepage: "https://github.com/runxhq/runx",
  description: "Native governed runtime for agent skills, tools, graphs, and packets.",
  artifacts,
};

writeFileSync(options.out, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(JSON.stringify({ status: "built", out: options.out, targets: Object.keys(artifacts) }, null, 2));

function parseArgs(argv) {
  const opts = { version: "", repo: "", tag: "", archives: "", out: "" };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--version") { opts.version = argv[++i] ?? ""; continue; }
    if (arg === "--repo") { opts.repo = argv[++i] ?? ""; continue; }
    if (arg === "--tag") { opts.tag = argv[++i] ?? ""; continue; }
    if (arg === "--archives") { opts.archives = argv[++i] ?? ""; continue; }
    if (arg === "--out") { opts.out = argv[++i] ?? ""; continue; }
    throw new Error(`unknown argument: ${arg}`);
  }
  for (const [k, v] of Object.entries(opts)) {
    if (!v) throw new Error(`--${k} is required`);
  }
  return opts;
}
