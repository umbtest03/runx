import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

const workspaceRoot = process.cwd();
const pnpm = process.platform === "win32" ? "pnpm.cmd" : "pnpm";

describe("Rust CLI cutover scripts", () => {
  it("accepts clean cutover candidates and blocks launcher shim flags", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-cutover-script-candidate-"));

    try {
      const clean = path.join(tempDir, executableName());
      const shim = path.join(tempDir, executableName("shim"));
      await writeExecutable(clean, exitScript(64));
      await writeExecutable(shim, shimAcceptingScript());

      const cleanResult = runTsx("scripts/check-rust-cli-cutover.ts", [
        "--candidate",
        clean,
        "--no-legacy-shapes",
        "--no-v2",
        "--no-aliases",
        "--no-js-fallback",
      ]);
      expect(cleanResult.status).toBe(0);
      expect(JSON.parse(cleanResult.stdout)).toMatchObject({
        status: "passed",
        findings: [],
      });

      const shimResult = runTsx("scripts/check-rust-cli-cutover.ts", [
        "--candidate",
        shim,
        "--no-js-fallback",
      ]);
      expect(shimResult.status).toBe(1);
      const payload = JSON.parse(shimResult.stdout) as { readonly findings: readonly { readonly rule: string }[] };
      expect(payload.findings.map((finding) => finding.rule)).toContain("launcher_shim_flag");
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("packages native CLI artifacts with checksum and signature metadata", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-rust-cli-package-test-"));

    try {
      const binary = path.join(tempDir, executableName());
      const outDir = path.join(tempDir, "artifacts");
      const signatureManifest = path.join(tempDir, "signatures.json");
      await writeExecutable(binary, exitScript(64));
      await writeFile(signatureManifest, `${JSON.stringify(await fixtureSignatureManifest(binary), null, 2)}\n`, "utf8");

      const packageResult = runTsx("scripts/package-rust-cli.ts", [
        "--binary",
        binary,
        "--out-dir",
        outDir,
        "--signature-manifest",
        signatureManifest,
      ]);
      expect(packageResult.status).toBe(0);
      expect(JSON.parse(packageResult.stdout)).toMatchObject({
        status: "passed",
        mode: "write",
        signature_manifest: "native/signatures.json",
      });

      const packageDir = path.join(outDir, platformKey(process.platform, process.arch));
      const checkResult = runTsx("scripts/check-rust-cli-release-artifacts.ts", [
        "--artifact-dir",
        outDir,
        "--no-js-delegation",
        "--verify-signatures",
      ]);
      expect(checkResult.status).toBe(0);
      expect(JSON.parse(checkResult.stdout)).toMatchObject({
        status: "passed",
        findings: [],
      });
      await expect(readFile(path.join(packageDir, "native", "signatures.json"), "utf8")).resolves.toContain(
        "runx.rust_cli_artifact_signatures.v1",
      );
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("fails closed for empty and malformed release artifact directories", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-rust-cli-artifact-check-"));

    try {
      const emptyDir = path.join(tempDir, "empty");
      const malformedDir = path.join(tempDir, "malformed");
      await mkdir(emptyDir);
      await mkdir(malformedDir);
      await writeFile(path.join(malformedDir, "package.json"), "{bad json", "utf8");

      const emptyResult = runTsx("scripts/check-rust-cli-release-artifacts.ts", ["--artifact-dir", emptyDir]);
      expect(emptyResult.status).toBe(1);
      expect(ruleIds(JSON.parse(emptyResult.stdout))).toEqual(["artifact_package_missing"]);

      const malformedResult = runTsx("scripts/check-rust-cli-release-artifacts.ts", ["--artifact-dir", malformedDir]);
      expect(malformedResult.status).toBe(1);
      expect(ruleIds(JSON.parse(malformedResult.stdout))).toEqual(["package_manifest_malformed"]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("requires signatures before release preparation", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-rust-cli-release-test-"));

    try {
      const binary = path.join(tempDir, executableName());
      await writeExecutable(binary, exitScript(64));

      const result = runTsx("scripts/release-rust-cli.ts", [
        "--binary",
        binary,
        "--artifact-dir",
        path.join(tempDir, "artifacts"),
      ]);

      expect(result.status).toBe(1);
      expect(result.stderr).toContain("--signature-manifest is required");
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("prepares signed release artifacts without publishing", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-rust-cli-release-prep-"));

    try {
      const binary = path.join(tempDir, executableName());
      const signatureManifest = path.join(tempDir, "signatures.json");
      await writeExecutable(binary, exitScript(64));
      await writeFile(signatureManifest, `${JSON.stringify(await fixtureSignatureManifest(binary), null, 2)}\n`, "utf8");

      const result = runTsx("scripts/release-rust-cli.ts", [
        "--binary",
        binary,
        "--artifact-dir",
        path.join(tempDir, "artifacts"),
        "--signature-manifest",
        signatureManifest,
      ]);

      expect(result.status).toBe(0);
      expect(result.stdout).toContain('"status": "prepared"');
      expect(result.stdout).toContain('"publish": false');
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });
});

function runTsx(script: string, args: readonly string[]) {
  return spawnSync(pnpm, ["exec", "tsx", script, ...args], {
    cwd: workspaceRoot,
    encoding: "utf8",
    maxBuffer: 4 * 1024 * 1024,
  });
}

function ruleIds(payload: { readonly findings: readonly { readonly rule: string }[] }): readonly string[] {
  return payload.findings.map((finding) => finding.rule);
}

async function fixtureSignatureManifest(binaryPath: string): Promise<unknown> {
  const manifest = JSON.parse(await readFile(path.join(workspaceRoot, "packages", "cli", "package.json"), "utf8")) as {
    readonly name: string;
    readonly version: string;
  };
  return {
    schema: "runx.rust_cli_artifact_signatures.v1",
    package: manifest.name,
    version: manifest.version,
    platform: platformKey(process.platform, process.arch),
    binary: process.platform === "win32" ? "bin/runx.exe" : "bin/runx",
    sha256: sha256(await readFile(binaryPath)),
    signatures: [
      {
        kind: "fixture",
        value: "fixture-signature",
      },
    ],
  };
}

async function writeExecutable(filePath: string, contents: string): Promise<void> {
  await writeFile(filePath, contents, "utf8");
  if (process.platform !== "win32") {
    await chmod(filePath, 0o755);
  }
}

function executableName(prefix = "runx"): string {
  return process.platform === "win32" ? `${prefix}.cmd` : prefix;
}

function exitScript(code: number): string {
  if (process.platform === "win32") {
    return `@echo off\r\nexit /b ${code}\r\n`;
  }
  return `#!/bin/sh\nexit ${code}\n`;
}

function shimAcceptingScript(): string {
  if (process.platform === "win32") {
    return '@echo off\r\nif "%1"=="--shim-help" exit /b 0\r\nexit /b 64\r\n';
  }
  return '#!/bin/sh\nif [ "$1" = "--shim-help" ]; then exit 0; fi\nexit 64\n';
}

function platformKey(platform: NodeJS.Platform, arch: string): string {
  if (platform === "darwin" && arch === "arm64") return "darwin-arm64";
  if (platform === "darwin" && arch === "x64") return "darwin-x64";
  if (platform === "linux" && arch === "arm64") return "linux-arm64";
  if (platform === "linux" && arch === "x64") return "linux-x64";
  if (platform === "win32" && arch === "x64") return "win32-x64";
  throw new Error(`unsupported Rust CLI package platform: ${platform}/${arch}`);
}

function sha256(bytes: Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}
