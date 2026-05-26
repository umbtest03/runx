import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { beforeAll, describe, expect, it } from "vitest";

import { readLocalReplaySeed } from "@runxhq/runtime-local";
import { runCli } from "../packages/cli/src/index.js";
import { ensureRunxBinary, kernelTestEnv } from "./host-protocol-test-utils.js";

describe("run replay cutover", () => {
  beforeAll(() => {
    ensureRunxBinary();
  });

  it("rejects the retired replay command before dispatching a skill run", async () => {
    const stdout = createMemoryStream();
    const stderr = createMemoryStream();

    const exitCode = await runCli(
      ["replay", "rx_123", "--json"],
      { stdin: process.stdin, stdout, stderr },
      kernelTestEnv(),
    );

    expect(exitCode).toBe(64);
    expect(stdout.contents()).toBe("");
    expect(stderr.contents()).toContain("Usage:");
  });

  it("does not synthesize a TypeScript replay seed from native-only receipts", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-replay-run-"));
    const receiptDir = path.join(tempDir, "receipts");
    const runxHome = path.join(tempDir, "home");

    try {
      const firstStdout = createMemoryStream();
      const firstExit = await runCli(
        ["skill", "fixtures/skills/echo", "--message", "hi", "--receipt-dir", receiptDir, "--json"],
        { stdin: process.stdin, stdout: firstStdout, stderr: createMemoryStream() },
        kernelTestEnv({
          RUNX_HOME: runxHome,
        }),
      );
      expect(firstExit).toBe(0);
      const first = JSON.parse(firstStdout.contents()) as { readonly receipt: { readonly id: string } };

      await expect(readLocalReplaySeed({ referenceId: first.receipt.id, receiptDir, runxHome })).rejects.toThrow(
        "missing replay seed details",
      );
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("runs graph skills natively without synthesizing a TypeScript replay path", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-replay-graph-"));
    const runxHome = path.join(tempDir, "home");

    try {
      const stdout = createMemoryStream();
      const stderr = createMemoryStream();
      const exitCode = await runCli(
        ["skill", "skills/sourcey", "--receipt-dir", path.join(tempDir, "receipts"), "--non-interactive", "--json"],
        { stdin: process.stdin, stdout, stderr },
        kernelTestEnv({
          RUNX_HOME: runxHome,
        }),
      );
      expect(exitCode).toBe(2);
      expect(stderr.contents()).toBe("");
      const response = JSON.parse(stdout.contents()) as { readonly status: string; readonly requests: unknown[] };
      expect(response.status).toBe("needs_agent");
      expect(response.requests.length).toBeGreaterThan(0);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });
});

function createMemoryStream(): NodeJS.WriteStream & { contents: () => string } {
  let contents = "";
  return {
    write(chunk: unknown) {
      contents += String(chunk);
      return true;
    },
    contents: () => contents,
  } as NodeJS.WriteStream & { contents: () => string };
}
