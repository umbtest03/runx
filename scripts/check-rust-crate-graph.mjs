import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const cratesRoot = path.join(workspaceRoot, "crates");

const expectedMembers = [
  "runx-cli",
  "runx-contracts",
  "runx-core",
  "runx-parser",
  "runx-receipts",
  "runx-runtime",
  "runx-sdk",
];

const apiBearingPublishedCrates = new Set([
  "runx-contracts",
  "runx-core",
  "runx-parser",
]);

const reservationVersionCrates = new Set([
  "runx-receipts",
  "runx-runtime",
  "runx-sdk",
]);

const publishableLibraryCrates = new Set([
  ...apiBearingPublishedCrates,
  ...reservationVersionCrates,
]);

const allowedRunxDeps = new Map([
  ["runx-cli", new Set(["runx-runtime", "runx-contracts"])],
  ["runx-contracts", new Set()],
  ["runx-core", new Set(["runx-contracts"])],
  ["runx-parser", new Set(["runx-contracts", "runx-core"])],
  ["runx-receipts", new Set(["runx-contracts"])],
  ["runx-runtime", new Set(["runx-contracts", "runx-core", "runx-parser", "runx-receipts"])],
  ["runx-sdk", new Set(["runx-contracts"])],
]);

const requiredRunxDeps = new Map([
  ["runx-core", new Set(["runx-contracts"])],
  ["runx-parser", new Set(["runx-contracts", "runx-core"])],
  ["runx-receipts", new Set(["runx-contracts"])],
  ["runx-runtime", new Set(["runx-contracts", "runx-core", "runx-parser", "runx-receipts"])],
  ["runx-sdk", new Set(["runx-contracts"])],
]);

const nonCliDisallowedDeps = [
  "tokio",
  "rmcp",
  "clap",
];

const workspaceDisallowedDeps = [
  "reqwest",
  "hyper",
  "serde_yml",
];

const findings = [];
const workspaceManifest = await readManifest("Cargo.toml");
const actualMembers = parseWorkspaceMembers(workspaceManifest);
const workspaceRunxVersions = parseWorkspaceRunxDependencyVersions(workspaceManifest);

checkMembers(actualMembers);
checkDisallowedDependencies("workspace", workspaceManifest);

for (const crateName of expectedMembers) {
  const manifest = await readManifest(`${crateName}/Cargo.toml`);
  const packageName = parsePackageName(manifest);
  if (packageName !== crateName) {
    findings.push(`${crateName}/Cargo.toml package name is ${packageName ?? "missing"}, expected ${crateName}`);
  }
  checkWorkspaceDependencyVersion(crateName, manifest);
  checkPublishingReadiness(crateName, manifest);
  checkRunxDependencies(crateName, manifest);
  await checkRunxDependencyUsage(crateName, manifest);
  checkPrematureRuntimeDependencies(crateName, manifest);
  checkDisallowedDependencies(crateName, manifest);
}

if (findings.length > 0) {
  console.error("Rust crate graph check failed:");
  for (const finding of findings) {
    console.error(`- ${finding}`);
  }
  process.exit(1);
}

console.log("Rust crate graph check passed.");

async function readManifest(relativePath) {
  return readFile(path.join(cratesRoot, relativePath), "utf8");
}

function parseWorkspaceMembers(manifest) {
  const body = sectionBody(manifest, "workspace");
  if (!body) {
    findings.push("crates/Cargo.toml is missing [workspace]");
    return [];
  }
  const membersMatch = /members\s*=\s*\[(?<body>.*?)\]/msu.exec(body);
  if (!membersMatch?.groups) {
    findings.push("crates/Cargo.toml is missing workspace members");
    return [];
  }
  return [...membersMatch.groups.body.matchAll(/"([^"]+)"/gu)].map((entry) => entry[1]).sort();
}

function parsePackageName(manifest) {
  const packageBody = sectionBody(manifest, "package");
  const match = /^name\s*=\s*"([^"]+)"/mu.exec(packageBody);
  return match?.[1];
}

function parsePackageVersion(manifest) {
  const packageBody = sectionBody(manifest, "package");
  const match = /^version\s*=\s*"([^"]+)"/mu.exec(packageBody);
  return match?.[1];
}

function parseWorkspaceRunxDependencyVersions(manifest) {
  const body = sectionBody(manifest, "workspace.dependencies");
  const versions = new Map();
  for (const match of body.matchAll(/^(runx-[A-Za-z0-9_-]+)\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"/gmu)) {
    versions.set(match[1], match[2]);
  }
  return versions;
}

function checkMembers(actualMembers) {
  const expected = [...expectedMembers].sort();
  if (actualMembers.join("\n") !== expected.join("\n")) {
    findings.push(`workspace members are ${actualMembers.join(", ")}, expected ${expected.join(", ")}`);
  }
  if (actualMembers.includes("runx-authoring")) {
    findings.push("runx-authoring must not be an initial Rust crate");
  }
}

function checkWorkspaceDependencyVersion(crateName, manifest) {
  const workspaceVersion = workspaceRunxVersions.get(crateName);
  if (!workspaceVersion) {
    return;
  }
  const packageVersion = parsePackageVersion(manifest);
  if (workspaceVersion !== packageVersion) {
    findings.push(
      `workspace dependency ${crateName} version ${workspaceVersion} must match ${crateName}/Cargo.toml version ${packageVersion ?? "missing"}`,
    );
  }
}

function checkPublishingReadiness(crateName, manifest) {
  const packageBody = sectionBody(manifest, "package");
  const hasPublishFalse = /^publish\s*=\s*false\s*$/mu.test(packageBody);
  const version = parsePackageVersion(manifest);
  if (apiBearingPublishedCrates.has(crateName) && version === "0.0.1") {
    findings.push(`${crateName}/Cargo.toml must not reuse the published reservation version 0.0.1 for API-bearing publishability`);
  }
  if (reservationVersionCrates.has(crateName)) {
    if (version !== "0.0.1") {
      findings.push(`${crateName}/Cargo.toml must use placeholder reservation version 0.0.1, found ${version ?? "missing"}`);
    }
  }
  if (publishableLibraryCrates.has(crateName)) {
    if (hasPublishFalse) {
      findings.push(`${crateName}/Cargo.toml must remain publishable so the crates.io package can be reserved or updated`);
    }
  }
  if (crateName === "runx-cli" && hasPublishFalse) {
    findings.push("runx-cli should remain publishable because it is the usable launcher package");
  }
}

function checkRunxDependencies(crateName, manifest) {
  const deps = parseDependencyNames(manifest).filter((dep) => dep.startsWith("runx-"));
  const allowed = allowedRunxDeps.get(crateName) ?? new Set();
  const required = requiredRunxDeps.get(crateName) ?? new Set();

  for (const dep of deps) {
    if (!allowed.has(dep)) {
      findings.push(`${crateName} must not depend on ${dep}`);
    }
  }
  for (const dep of required) {
    if (!deps.includes(dep)) {
      findings.push(`${crateName} must depend on ${dep}`);
    }
  }
}

async function checkRunxDependencyUsage(crateName, manifest) {
  if (crateName !== "runx-parser") {
    return;
  }
  const deps = parseDependencyNames(manifest).filter((dep) => dep.startsWith("runx-"));
  const source = await readCrateSource(crateName);
  for (const dep of deps) {
    const importName = dep.replaceAll("-", "_");
    if (!source.includes(importName)) {
      findings.push(`${crateName} declares ${dep} but does not use ${importName} in src/`);
    }
  }
}

async function readCrateSource(crateName) {
  const files = await collectRustFiles(path.join(cratesRoot, crateName, "src"));
  const contents = await Promise.all(files.map((filePath) => readFile(filePath, "utf8")));
  return contents.join("\n");
}

async function collectRustFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...await collectRustFiles(entryPath));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(entryPath);
    }
  }
  return files.sort();
}

function checkPrematureRuntimeDependencies(crateName, manifest) {
  if (crateName === "runx-cli") {
    return;
  }
  const dependencyNames = parseDependencyNames(manifest);
  for (const dep of nonCliDisallowedDeps) {
    if (dependencyNames.includes(dep)) {
      findings.push(`${crateName} must not depend on ${dep} before its implementation spec allows it`);
    }
  }
}

function checkDisallowedDependencies(crateName, manifest) {
  const dependencyNames = parseDependencyNames(manifest);
  for (const dep of workspaceDisallowedDeps) {
    if (dependencyNames.includes(dep)) {
      findings.push(`${crateName} must not depend on ${dep}`);
    }
  }
}

function parseDependencyNames(manifest) {
  const names = new Set();
  for (const sectionName of ["dependencies", "dev-dependencies", "build-dependencies"]) {
    for (const name of dependencyNamesFromSection(sectionBody(manifest, sectionName))) {
      names.add(name);
    }
    for (const name of dependencyNamesFromSubtables(manifest, sectionName)) {
      names.add(name);
    }
  }
  return [...names].sort();
}

function dependencyNamesFromSection(body) {
  const names = [];
  for (const line of body.split("\n")) {
    const match = /^([A-Za-z0-9_-]+)(?:\.[A-Za-z0-9_-]+)?\s*=/u.exec(line.trim());
    if (match) {
      names.push(match[1]);
    }
    const packageMatch = /^package\s*=\s*"([^"]+)"/u.exec(line.trim());
    if (packageMatch) {
      names.push(packageMatch[1]);
    }
  }
  return names;
}

function dependencyNamesFromSubtables(manifest, sectionName) {
  const names = [];
  const headerPattern = new RegExp(`^\\[${escapeRegExp(sectionName)}\\.([A-Za-z0-9_-]+)\\]\\s*$`, "gmu");
  for (const match of manifest.matchAll(headerPattern)) {
    names.push(match[1]);
    const bodyStart = match.index + match[0].length;
    const nextSection = /^\[/mu.exec(manifest.slice(bodyStart));
    const body = nextSection ? manifest.slice(bodyStart, bodyStart + nextSection.index) : manifest.slice(bodyStart);
    const packageName = dependencyPackageNameFromTable(body);
    if (packageName) {
      names.push(packageName);
    }
  }
  return names;
}

function dependencyPackageNameFromTable(body) {
  for (const line of body.split("\n")) {
    const packageMatch = /^package\s*=\s*"([^"]+)"/u.exec(line.trim());
    if (packageMatch) {
      return packageMatch[1];
    }
  }
  return null;
}

function sectionBody(manifest, sectionName) {
  const pattern = new RegExp(`^\\[${escapeRegExp(sectionName)}\\]\\s*$`, "mu");
  const match = pattern.exec(manifest);
  if (!match) {
    return "";
  }
  const bodyStart = match.index + match[0].length;
  const nextSection = /^\[/mu.exec(manifest.slice(bodyStart));
  return nextSection ? manifest.slice(bodyStart, bodyStart + nextSection.index) : manifest.slice(bodyStart);
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
