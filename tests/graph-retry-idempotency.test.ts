import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { runLocalGraph, type Caller, type SkillAdapter } from "@runxhq/runtime-local";
import { kernelEnv, resolveRunxBinary } from "./runx-binary.js";

const nonInteractiveCaller: Caller = {
  resolve: async () => undefined,
  report: () => undefined,
};

describe("graph retry and idempotency", () => {
  it("retries a read-only step and records attempt receipts", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-retry-read-"));
    const adapter = createFlakyAdapter();

    try {
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/retry/read-only.yaml"),
        caller: nonInteractiveCaller,
        receiptDir: path.join(tempDir, "receipts"),
        runxHome: path.join(tempDir, "home"),
        env: kernelEnv(),
        adapters: [adapter],
      });

      expect(result.status).toBe("sealed");
      if (result.status !== "sealed") {
        return;
      }
      expect(result.steps.map((step) => [step.stepId, step.attempt, step.status])).toEqual([
        ["flaky-read", 1, "failure"],
        ["flaky-read", 2, "sealed"],
      ]);
      expect(result.receipt.schema).toBe("runx.receipt.v1");
      expect(result.steps.map((step) => step.retry)).toEqual([
        {
          attempt: 1,
          maxAttempts: 2,
          ruleFired: "initial_attempt",
        },
        {
          attempt: 2,
          maxAttempts: 2,
          ruleFired: "retry_attempt",
        },
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("denies mutating retry without idempotency before execution", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-retry-denied-"));
    const adapter = createFlakyAdapter();

    try {
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/retry/mutating-denied.yaml"),
        caller: nonInteractiveCaller,
        receiptDir: path.join(tempDir, "receipts"),
        runxHome: path.join(tempDir, "home"),
        env: kernelEnv(),
        adapters: [adapter],
      });

      expect(result.status).toBe("policy_denied");
      if (result.status !== "policy_denied") {
        return;
      }
      expect(result.reasons).toEqual(["step 'deploy' declares mutating retry without an idempotency key"]);
      expect(adapter.callCount()).toBe(0);
      expect(result.receipt?.schema).toBe("runx.receipt.v1");
      expect(result.receipt?.seal.disposition).toBe("declined");
      expect(runtimeGraphSteps(result.receipt)).toMatchObject([
          {
            step_id: "deploy",
            status: "failure",
            disposition: "policy_denied",
            receipt_id: undefined,
          },
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("fails closed with a signed receipt when retry admission cannot reach the Rust kernel", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-retry-bridge-missing-"));
    const adapter = createFlakyAdapter();

    try {
      const kernelWrapper = await writeRetryAdmissionOutageKernel(tempDir);
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/retry/read-only.yaml"),
        caller: nonInteractiveCaller,
        receiptDir: path.join(tempDir, "receipts"),
        runxHome: path.join(tempDir, "home"),
        env: { ...kernelEnv(), RUNX_KERNEL_EVAL_BIN: kernelWrapper },
        adapters: [adapter],
      });

      expect(result.status).toBe("policy_denied");
      if (result.status !== "policy_denied") {
        return;
      }
      expect(result.reasons).toEqual([
        "retry admission failed closed: Rust kernel eval failed with exit 1: simulated retry admission outage",
      ]);
      expect(adapter.callCount()).toBe(0);
      expect(result.receipt?.schema).toBe("runx.receipt.v1");
      expect(result.receipt?.seal.disposition).toBe("declined");
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("honors skill-level retry metadata when the graph step omits retry", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-retry-skill-"));
    const adapter = createFlakyAdapter();

    try {
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/retry/skill-level.yaml"),
        caller: nonInteractiveCaller,
        receiptDir: path.join(tempDir, "receipts"),
        runxHome: path.join(tempDir, "home"),
        env: kernelEnv(),
        adapters: [adapter],
      });

      expect(result.status).toBe("sealed");
      if (result.status !== "sealed") {
        return;
      }
      expect(result.steps.map((step) => [step.stepId, step.attempt, step.status])).toEqual([
        ["skill-retry", 1, "failure"],
        ["skill-retry", 2, "sealed"],
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("denies skill-level mutating retry without requiring duplicate graph-step metadata", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-retry-skill-denied-"));
    const adapter = createFlakyAdapter();

    try {
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/retry/skill-mutating-denied.yaml"),
        caller: nonInteractiveCaller,
        receiptDir: path.join(tempDir, "receipts"),
        runxHome: path.join(tempDir, "home"),
        env: kernelEnv(),
        adapters: [adapter],
      });

      expect(result.status).toBe("policy_denied");
      if (result.status !== "policy_denied") {
        return;
      }
      expect(result.reasons).toEqual(["step 'deploy' declares mutating retry without an idempotency key"]);
      expect(adapter.callCount()).toBe(0);
      expect(result.receipt?.schema).toBe("runx.receipt.v1");
      expect(result.receipt?.seal.disposition).toBe("declined");
      expect(runtimeGraphSteps(result.receipt)).toMatchObject([
          {
            step_id: "deploy",
            status: "failure",
            disposition: "policy_denied",
            receipt_id: undefined,
          },
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("retries a mutating step with idempotency key hash and no raw key in receipts", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-retry-idem-"));
    const receiptDir = path.join(tempDir, "receipts");
    const adapter = createFlakyAdapter();

    try {
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/retry/mutating-idempotent.yaml"),
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome: path.join(tempDir, "home"),
        env: kernelEnv(),
        adapters: [adapter],
      });

      expect(result.status).toBe("sealed");
      if (result.status !== "sealed") {
        return;
      }
      expect(result.steps).toHaveLength(2);
      const hashes = result.steps.map((step) => step.retry?.idempotencyKeyHash);
      expect(hashes[0]).toBeTruthy();
      expect(hashes[0]).toBe(hashes[1]);

      const graphReceipt = await readFile(path.join(receiptDir, `${result.receipt.id}.json`), "utf8");
      const firstAttemptReceipt = await readFile(path.join(receiptDir, `${result.steps[0].receiptId}.json`), "utf8");
      expect(graphReceipt).not.toContain("deploy-123");
      expect(firstAttemptReceipt).not.toContain("deploy-123");
      expect(firstAttemptReceipt).toContain("idempotency_key_hash");
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });
});

async function writeRetryAdmissionOutageKernel(directory: string): Promise<string> {
  const wrapperPath = path.join(directory, "retry-admission-outage-kernel.mjs");
  const realKernel = resolveRunxBinary();
  await writeFile(
    wrapperPath,
    `#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const input = readFileSync(0, "utf8");
const document = JSON.parse(input);
if (document.kind === "policy.admitRetryPolicy") {
  process.stderr.write("simulated retry admission outage");
  process.exit(1);
}

const result = spawnSync(${JSON.stringify(realKernel)}, process.argv.slice(2), {
  input,
  encoding: "utf8",
  env: process.env,
});
if (result.stdout) process.stdout.write(result.stdout);
if (result.stderr) process.stderr.write(result.stderr);
process.exit(result.status ?? 1);
`,
  );
  await chmod(wrapperPath, 0o755);
  return wrapperPath;
}

function createFlakyAdapter(): SkillAdapter & { callCount: () => number } {
  let calls = 0;
  return {
    type: "cli-tool",
    callCount: () => calls,
    invoke: async (request) => {
      calls += 1;
      if (calls === 1) {
        return {
          status: "failure",
          stdout: "",
          stderr: "transient failure",
          exitCode: 1,
          signal: null,
          durationMs: 1,
          errorMessage: "transient failure",
        };
      }
      return {
        status: "sealed",
        stdout: String(request.inputs.message ?? "ok"),
        stderr: "",
        exitCode: 0,
        signal: null,
        durationMs: 1,
      };
    },
  };
}

interface RuntimeGraphStep {
  readonly step_id: string;
  readonly runner?: string;
  readonly status?: string;
  readonly receipt_id?: string;
  readonly fanout_group?: string;
  readonly disposition?: string;
  readonly outcome_state?: string;
  readonly retry?: {
    readonly attempt?: number;
    readonly max_attempts?: number;
    readonly rule_fired?: string;
    readonly idempotency_key_hash?: string;
  };
  readonly governance?: unknown;
}

function runtimeGraphSteps(receipt: { readonly metadata?: Readonly<Record<string, unknown>> } | undefined): readonly RuntimeGraphStep[] {
  const runx = receipt?.metadata?.runx;
  expect(runx).toEqual(expect.any(Object));
  const steps = (runx as { readonly steps?: unknown } | undefined)?.steps;
  expect(Array.isArray(steps)).toBe(true);
  return steps as readonly RuntimeGraphStep[];
}
