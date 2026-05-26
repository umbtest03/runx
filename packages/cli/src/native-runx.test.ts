import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { chmod } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";

import { describe, expect, it } from "vitest";

import { resolveNativeRunxBinary, spawnNativeRunx } from "./native-runx.js";

describe("resolveNativeRunxBinary", () => {
  it("does not discover binaries from the caller cwd", async () => {
    const tempDir = mkdtempSync(path.join(os.tmpdir(), "runx-native-cwd-"));
    const fakeRunx = path.join(tempDir, "crates", "target", "debug", "runx");
    mkdirSync(path.dirname(fakeRunx), { recursive: true });
    writeFileSync(fakeRunx, "#!/bin/sh\nexit 99\n");
    await chmod(fakeRunx, 0o755);

    const previousCwd = process.cwd();
    try {
      process.chdir(tempDir);
      expect(resolveNativeRunxBinary({})).toBe("runx");
    } finally {
      process.chdir(previousCwd);
    }
  });

  it("does not use RUNX_RUST_CLI_BIN as a packaged binary override", () => {
    expect(resolveNativeRunxBinary({ RUNX_RUST_CLI_BIN: "/missing/runx" })).toBe("runx");
  });

  it("fails when RUNX_DEV_RUST_CLI_BIN points at a missing binary", () => {
    expect(() => resolveNativeRunxBinary({ RUNX_DEV_RUST_CLI_BIN: "/missing/runx" })).toThrow(
      "RUNX_DEV_RUST_CLI_BIN does not exist",
    );
  });

  it("requires RUNX_DEV_RUST_CLI_BIN to be absolute", () => {
    expect(() => resolveNativeRunxBinary({ RUNX_DEV_RUST_CLI_BIN: "runx" })).toThrow(
      "RUNX_DEV_RUST_CLI_BIN must be an absolute path",
    );
  });
});

describe("spawnNativeRunx", () => {
  it("rejects when stdout exceeds the configured byte limit", async () => {
    await expect(
      spawnNativeRunx({
        command: process.execPath,
        args: ["-e", "process.stdout.write('abcdef')"],
        cwd: process.cwd(),
        env: {},
        timeoutMs: 5_000,
        maxOutputBytes: 3,
      }),
    ).rejects.toThrow("native runx stdout exceeded 3 bytes");
  });
});
