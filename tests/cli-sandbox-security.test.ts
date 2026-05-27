import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { Readable } from "node:stream";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { runCli, type CliIo } from "../packages/cli/src/index.js";

describe("CLI sandbox security", () => {
  it("fails closed for unrestricted local dev without explicit escalation", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-cli-sandbox-deny-"));

    try {
      const skillPath = await writeSkill(tempDir, {
        name: "cli-sandbox-deny",
        sandbox: "unrestricted-local-dev",
      });
      const stdout = createMemoryStream();
      const stderr = createMemoryStream();
      const exitCode = await runCli(
        ["skill", skillPath, "--receipt-dir", path.join(tempDir, "receipts")],
        createIo("", stdout, stderr),
        cliEnv(),
      );

      expect(exitCode).toBe(1);
      expect(stdout.contents()).toBe("");
      expect(stderr.contents()).toContain("sandbox violation");
      expect(stderr.contents()).toContain("unrestricted-local-dev requires approved escalation");
      expect(stderr.contents()).not.toContain("Approve? [y/N]");
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("returns structured JSON when sandbox enforcement rejects a run", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-cli-sandbox-json-"));

    try {
      const skillPath = await writeSkill(tempDir, {
        name: "cli-sandbox-json",
        sandbox: "unrestricted-local-dev",
      });
      const stdout = createMemoryStream();
      const stderr = createMemoryStream();
      const exitCode = await runCli(
        ["skill", skillPath, "--non-interactive", "--json", "--receipt-dir", path.join(tempDir, "receipts")],
        createIo("", stdout, stderr),
        cliEnv(),
      );

      expect(exitCode).toBe(1);
      expect(stderr.contents()).toBe("");
      expect(JSON.parse(stdout.contents())).toMatchObject({
        status: "failure",
        error: {
          message: expect.stringContaining("unrestricted-local-dev requires approved escalation"),
        },
      });
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("runs an X.yaml cli-tool skill with a signed receipt", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-cli-sandbox-signed-"));

    try {
      const skillPath = await writeSkill(tempDir, {
        name: "cli-sandbox-signed",
      });
      const stdout = createMemoryStream();
      const stderr = createMemoryStream();
      const exitCode = await runCli(
        ["skill", skillPath, "--non-interactive", "--json", "--receipt-dir", path.join(tempDir, "receipts")],
        createIo("", stdout, stderr),
        cliEnv(),
      );

      expect(exitCode).toBe(0);
      expect(stderr.contents()).toBe("");
      expect(JSON.parse(stdout.contents())).toMatchObject({
        status: "sealed",
        execution: {
          stdout: "approved",
        },
        receipt: {
          signature: {
            alg: "Ed25519",
          },
        },
      });
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });
});

async function writeSkill(
  tempDir: string,
  options: { readonly name: string; readonly sandbox?: "unrestricted-local-dev" },
): Promise<string> {
  const skillPath = path.join(tempDir, options.name);
  await mkdir(skillPath, { recursive: true });
  await writeFile(
    path.join(skillPath, "X.yaml"),
    `skill: ${options.name}
version: "0.1.0"

runners:
  default:
    default: true
    type: cli-tool
    command: node
    args:
      - -e
      - "process.stdout.write('approved')"
${options.sandbox ? `    sandbox:\n      profile: ${options.sandbox}\n` : ""}`,
  );
  return skillPath;
}

function cliEnv(): NodeJS.ProcessEnv {
  return {
    ...process.env,
    RUNX_CWD: process.cwd(),
    RUNX_DEV_RUST_CLI_BIN:
      process.env.RUNX_DEV_RUST_CLI_BIN ?? path.join(process.cwd(), "crates", "target", "debug", "runx"),
    RUNX_RECEIPT_SIGN_KID: process.env.RUNX_RECEIPT_SIGN_KID ?? "cli-sandbox-test-key",
    RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64:
      process.env.RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64 ?? "QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=",
    RUNX_RECEIPT_SIGN_ISSUER_TYPE: process.env.RUNX_RECEIPT_SIGN_ISSUER_TYPE ?? "hosted",
  };
}

function createIo(input: string, stdout = createMemoryStream(), stderr = createMemoryStream()): CliIo {
  return {
    stdin: Readable.from([input]) as NodeJS.ReadStream,
    stdout,
    stderr,
  };
}

function createMemoryStream(): NodeJS.WriteStream & { contents: () => string } {
  let buffer = "";
  return {
    write: (chunk: string | Uint8Array) => {
      buffer += chunk.toString();
      return true;
    },
    contents: () => buffer,
  } as NodeJS.WriteStream & { contents: () => string };
}
