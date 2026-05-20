import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const npm = process.platform === "win32" ? "npm.cmd" : "npm";

interface Finding {
  readonly rule: string;
  readonly file: string;
  readonly message: string;
}

interface Options {
  readonly artifactDir: string;
  readonly noJsDelegation: boolean;
  readonly verifySignatures: boolean;
}

const options = parseArgs(process.argv.slice(2));
const artifactDir = path.resolve(workspaceRoot, options.artifactDir);
const findings: Finding[] = [];

if (!existsSync(artifactDir)) {
  findings.push(finding("artifact_dir_missing", artifactDir, "release artifact directory does not exist"));
} else {
  const packageDirs = listPackageDirs(artifactDir);
  if (packageDirs.length === 0) {
    findings.push(finding("artifact_package_missing", artifactDir, "release artifact directory contains no package.json files"));
  }
  for (const packageDir of packageDirs) {
    inspectPackageDir(packageDir, findings);
  }
}

emit({
  status: findings.length === 0 ? "passed" : "blocked",
  artifact_dir: displayPath(artifactDir),
  findings,
});
process.exit(findings.length === 0 ? 0 : 1);

function parseArgs(argv: readonly string[]): Options {
  let artifactDir = ".runx/rust-cli-artifacts";
  let noJsDelegation = false;
  let verifySignatures = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--artifact-dir") {
      artifactDir = argv[index + 1] ?? "";
      index += 1;
      continue;
    }
    if (arg === "--no-js-delegation") {
      noJsDelegation = true;
      continue;
    }
    if (arg === "--verify-signatures") {
      verifySignatures = true;
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }
    throw new Error(`unknown argument: ${arg}`);
  }

  if (!artifactDir) {
    throw new Error("--artifact-dir requires a path");
  }
  return { artifactDir, noJsDelegation, verifySignatures };
}

function listPackageDirs(root: string): readonly string[] {
  if (existsSync(path.join(root, "package.json"))) {
    return [root];
  }
  const entries = readdirSync(root, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .filter((entry) => existsSync(path.join(root, entry.name, "package.json")))
    .map((entry) => path.join(root, entry.name))
    .sort();
  return entries;
}

function inspectPackageDir(packageDir: string, output: Finding[]): void {
  const manifestPath = path.join(packageDir, "package.json");
  if (!existsSync(manifestPath)) {
    output.push(finding("package_manifest_missing", packageDir, "artifact package is missing package.json"));
    return;
  }

  const manifest = readJson<{
    readonly bin?: string | { readonly runx?: string };
    readonly dependencies?: Record<string, string>;
    readonly optionalDependencies?: Record<string, string>;
    readonly devDependencies?: Record<string, string>;
    readonly peerDependencies?: Record<string, string>;
  }>(manifestPath, output, "package_manifest_malformed");
  if (!manifest) {
    return;
  }
  const bin = typeof manifest.bin === "string" ? manifest.bin : manifest.bin?.runx;
  if (!bin) {
    output.push(finding("package_bin_missing", manifestPath, "package.json must declare bin.runx"));
    return;
  }
  if (/\.(?:js|mjs|cjs)$/u.test(bin)) {
    output.push(finding("package_bin_js", manifestPath, `bin.runx points to JavaScript: ${bin}`));
  }
  const binaryPath = path.resolve(packageDir, bin);
  if (!isInside(binaryPath, packageDir)) {
    output.push(finding("package_bin_escapes", manifestPath, `bin.runx points outside the package: ${bin}`));
    return;
  }
  if (!existsSync(binaryPath)) {
    output.push(finding("package_bin_target_missing", binaryPath, "bin.runx target is missing"));
  } else {
    const entry = statSync(binaryPath);
    if (!entry.isFile() || (process.platform !== "win32" && (entry.mode & 0o111) === 0)) {
      output.push(finding("package_bin_not_executable", binaryPath, "bin.runx target is not executable"));
    }
  }

  inspectDependencySections(manifest, manifestPath, output);
  inspectChecksum(packageDir, bin, output);
  inspectSignature(packageDir, bin, output);
  inspectPackList(packageDir, output);

  if (options.noJsDelegation) {
    inspectTextFiles(packageDir, output);
  }
}

function inspectDependencySections(
  manifest: {
    readonly dependencies?: Record<string, string>;
    readonly optionalDependencies?: Record<string, string>;
    readonly devDependencies?: Record<string, string>;
    readonly peerDependencies?: Record<string, string>;
  },
  manifestPath: string,
  output: Finding[],
): void {
  for (const sectionName of ["dependencies", "optionalDependencies", "devDependencies", "peerDependencies"] as const) {
    const section = manifest[sectionName];
    if (!section) continue;
    for (const [name, spec] of Object.entries(section)) {
      if (["@runxhq/adapters", "@runxhq/authoring", "@runxhq/contracts", "@runxhq/core", "@runxhq/runtime-local"].includes(name)) {
        output.push(finding("ts_runtime_dependency", manifestPath, `${sectionName}.${name} is not allowed in the Rust CLI artifact`));
      }
      if (spec.startsWith("workspace:")) {
        output.push(finding("workspace_dependency", manifestPath, `${sectionName}.${name} still uses ${spec}`));
      }
    }
  }
}

function inspectChecksum(packageDir: string, bin: string, output: Finding[]): void {
  const checksumPath = path.join(packageDir, "native", "checksums.json");
  if (!existsSync(checksumPath)) {
    output.push(finding("checksum_manifest_missing", checksumPath, "native/checksums.json is required"));
    return;
  }
  const checksum = readJson<{
    readonly binary?: string;
    readonly sha256?: string;
  }>(checksumPath, output, "checksum_manifest_malformed");
  if (!checksum) {
    return;
  }
  if (checksum.binary !== stripDotSlash(bin)) {
    output.push(finding("checksum_binary_mismatch", checksumPath, `checksum binary ${checksum.binary ?? "<missing>"} does not match ${bin}`));
    return;
  }
  if (!checksum.sha256 || !/^[0-9a-f]{64}$/u.test(checksum.sha256)) {
    output.push(finding("checksum_sha256_invalid", checksumPath, "checksum sha256 must be a 64-character lowercase hex digest"));
    return;
  }
  const binaryPath = path.resolve(packageDir, bin);
  if (existsSync(binaryPath) && checksum.sha256 !== sha256(readFileSync(binaryPath))) {
    output.push(finding("checksum_mismatch", checksumPath, "binary sha256 does not match native/checksums.json"));
  }
}

function inspectSignature(packageDir: string, bin: string, output: Finding[]): void {
  const signaturePath = path.join(packageDir, "native", "signatures.json");
  if (!existsSync(signaturePath)) {
    if (options.verifySignatures) {
      output.push(finding("signature_manifest_missing", signaturePath, "native/signatures.json is required by --verify-signatures"));
    }
    return;
  }

  const signature = readJson<{
    readonly schema?: string;
    readonly binary?: string;
    readonly sha256?: string;
    readonly signatures?: readonly unknown[];
  }>(signaturePath, output, "signature_manifest_malformed");
  if (!signature) {
    return;
  }
  if (signature.schema !== "runx.rust_cli_artifact_signatures.v1") {
    output.push(finding("signature_schema_invalid", signaturePath, "signature manifest schema must be runx.rust_cli_artifact_signatures.v1"));
  }
  if (signature.binary !== stripDotSlash(bin)) {
    output.push(finding("signature_binary_mismatch", signaturePath, `signature binary ${signature.binary ?? "<missing>"} does not match ${bin}`));
  }
  if (!signature.sha256 || !/^[0-9a-f]{64}$/u.test(signature.sha256)) {
    output.push(finding("signature_sha256_invalid", signaturePath, "signature sha256 must be a 64-character lowercase hex digest"));
  }
  const checksum = readJson<{ readonly sha256?: string }>(path.join(packageDir, "native", "checksums.json"), [], "checksum_manifest_malformed");
  if (checksum?.sha256 && signature.sha256 && checksum.sha256 !== signature.sha256) {
    output.push(finding("signature_checksum_mismatch", signaturePath, "signature sha256 does not match native/checksums.json"));
  }
  if (!Array.isArray(signature.signatures) || signature.signatures.length === 0) {
    output.push(finding("signature_entries_missing", signaturePath, "signature manifest must include at least one signature entry"));
  } else {
    for (const [index, entry] of signature.signatures.entries()) {
      if (!isSignatureEntry(entry)) {
        output.push(finding("signature_entry_invalid", signaturePath, `signature entry ${index} must include non-empty kind and value strings`));
      }
    }
  }
}

function inspectPackList(packageDir: string, output: Finding[]): void {
  try {
    const pack = execFileSync(npm, ["pack", "--dry-run", "--json"], {
      cwd: packageDir,
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
    });
    const [report] = JSON.parse(pack) as [{ readonly files?: readonly { readonly path: string }[] }];
    for (const entry of report.files ?? []) {
      if (/^(dist|src|tools|node_modules)\//u.test(entry.path) || /^bin\/runx\.(?:js|mjs|cjs)$/u.test(entry.path)) {
        output.push(finding("pack_contains_js_runtime", path.join(packageDir, entry.path), "packed Rust CLI artifact contains a JS/TS runtime path"));
      }
    }
  } catch (error) {
    output.push(finding("pack_dry_run_failed", packageDir, errorMessage(error)));
  }
}

function inspectTextFiles(packageDir: string, output: Finding[]): void {
  for (const filePath of listFiles(packageDir)) {
    const relative = path.relative(packageDir, filePath).split(path.sep).join("/");
    if (!/\.(?:json|md|txt|js|mjs|cjs|ts|tsx)$/u.test(relative)) {
      continue;
    }
    const text = readFileSync(filePath, "utf8");
    for (const token of ["RUNX_JS_BIN", "npm exec", "packages/cli/src", "packages/cli/dist", "process.execPath"]) {
      if (text.includes(token)) {
        output.push(finding("js_delegation_token", filePath, `artifact contains forbidden delegation token ${token}`));
      }
    }
  }
}

function listFiles(root: string): readonly string[] {
  const files: string[] = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const filePath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFiles(filePath));
    } else if (entry.isFile()) {
      files.push(filePath);
    }
  }
  return files.sort();
}

function stripDotSlash(value: string): string {
  return value.replace(/^\.\//u, "");
}

function readJson<T>(filePath: string, output: Finding[], rule: string): T | null {
  try {
    return JSON.parse(readFileSync(filePath, "utf8")) as T;
  } catch (error) {
    output.push(finding(rule, filePath, errorMessage(error)));
    return null;
  }
}

function isSignatureEntry(value: unknown): value is { readonly kind: string; readonly value: string } {
  if (!value || typeof value !== "object") {
    return false;
  }
  const entry = value as { readonly kind?: unknown; readonly value?: unknown };
  return typeof entry.kind === "string" && entry.kind.trim() !== ""
    && typeof entry.value === "string" && entry.value.trim() !== "";
}

function isInside(candidatePath: string, rootPath: string): boolean {
  const relative = path.relative(rootPath, candidatePath);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

function sha256(bytes: Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function finding(rule: string, filePath: string, message: string): Finding {
  return { rule, file: displayPath(filePath), message };
}

function displayPath(filePath: string): string {
  const relative = path.relative(workspaceRoot, filePath);
  return relative && !relative.startsWith("..") ? relative.split(path.sep).join("/") : filePath;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function emit(payload: unknown): void {
  console.log(JSON.stringify(payload, null, 2));
}

function printUsage(): void {
  console.log("Usage: pnpm exec tsx scripts/check-rust-cli-release-artifacts.ts [--artifact-dir .runx/rust-cli-artifacts] [--no-js-delegation] [--verify-signatures]");
}
