import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { createDefaultSkillAdapters } from "@runxhq/adapters";
import { runLocalGraph, type Caller, type SkillAdapter } from "@runxhq/runtime-local";
import { kernelEnv } from "./runx-binary.js";

const nonInteractiveCaller: Caller = {
  resolve: async () => undefined,
  report: () => undefined,
};
const adapters = createDefaultSkillAdapters();

describe("local fanout graph runner", () => {
  it("runs a fanout group with all-success sync policy", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-fanout-all-"));
    const receiptDir = path.join(tempDir, "receipts");
    const runxHome = path.join(tempDir, "home");

    try {
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/fanout/all.yaml"),
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome,
        env: kernelEnv(),
        adapters,
      });

      expect(result.status).toBe("sealed");
      if (result.status !== "sealed") {
        return;
      }

      expect(result.steps.map((step) => [step.stepId, step.status, step.fanoutGroup])).toEqual([
        ["market", "sealed", "advisors"],
        ["risk", "sealed", "advisors"],
        ["finance", "sealed", "advisors"],
        ["synthesize", "sealed", undefined],
      ]);
      expect(result.steps[3].stdout).toBe("approved");
      expect(result.receipt.schema).toBe("runx.receipt.v1");
      expect(runtimeSyncPoints(result.receipt)).toEqual([
        expect.objectContaining({
          group_id: "advisors",
          strategy: "all",
          decision: "proceed",
          rule_fired: "all.min_success",
          branch_count: 3,
          success_count: 3,
          failure_count: 0,
          required_successes: 3,
        }),
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("starts fanout branches concurrently", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-fanout-parallel-"));
    const receiptDir = path.join(tempDir, "receipts");
    const runxHome = path.join(tempDir, "home");
    const graphPath = path.join(tempDir, "parallel.yaml");
    const adapter = createConcurrencyProbeAdapter(3);

    try {
      await Promise.all([
        writeProbeSkill(path.join(tempDir, "market"), "market"),
        writeProbeSkill(path.join(tempDir, "risk"), "risk"),
        writeProbeSkill(path.join(tempDir, "finance"), "finance"),
      ]);
      await writeFile(
        graphPath,
        `name: timed-fanout
owner: runx
fanout:
  groups:
    advisors:
      strategy: all
      on_branch_failure: halt
steps:
  - id: market
    mode: fanout
    fanout_group: advisors
    skill: ./market
  - id: risk
    mode: fanout
    fanout_group: advisors
    skill: ./risk
  - id: finance
    mode: fanout
    fanout_group: advisors
    skill: ./finance
`,
      );

      const result = await runLocalGraph({
        graphPath,
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome,
        env: kernelEnv(),
        adapters: [adapter],
      });

      expect(result.status).toBe("sealed");
      expect(adapter.maxActive()).toBe(3);
      if (result.status !== "sealed") {
        return;
      }
      expect(result.steps.map((step) => step.stepId)).toEqual(["market", "risk", "finance"]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("runs a fanout group with quorum sync and linked branch receipts", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-fanout-"));
    const receiptDir = path.join(tempDir, "receipts");
    const runxHome = path.join(tempDir, "home");

    try {
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/fanout/graph.yaml"),
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome,
        env: kernelEnv(),
        adapters,
      });

      expect(result.status).toBe("sealed");
      if (result.status !== "sealed") {
        return;
      }

      expect(result.steps.map((step) => [step.stepId, step.status, step.fanoutGroup])).toEqual([
        ["market", "sealed", "advisors"],
        ["risk", "sealed", "advisors"],
        ["finance", "failure", "advisors"],
        ["synthesize", "sealed", undefined],
      ]);
      expect(result.steps.slice(0, 3).map((step) => step.parentReceipt)).toEqual([undefined, undefined, undefined]);
      expect(result.steps[3].stdout).toBe("go");
      expect(result.receipt.schema).toBe("runx.receipt.v1");
      expect(result.receipt.lineage?.children).toHaveLength(4);
      expect(runtimeSyncPoints(result.receipt)).toEqual([
        expect.objectContaining({
          group_id: "advisors",
          strategy: "quorum",
          decision: "proceed",
          rule_fired: "quorum.min_success",
          branch_count: 3,
          success_count: 2,
          failure_count: 1,
          required_successes: 2,
          branch_receipts: result.steps.slice(0, 3).map((step) => step.receiptId),
        }),
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("pauses and resumes deterministically when a structured threshold gate fires", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-fanout-threshold-"));
    const receiptDir = path.join(tempDir, "receipts");
    const runxHome = path.join(tempDir, "home");
    const graphPath = path.resolve("fixtures/graphs/fanout/threshold.yaml");
    const approvingCaller: Caller = {
      resolve: async (request) => request.kind === "approval" ? { actor: "human", payload: true } : undefined,
      report: () => undefined,
    };

    try {
      const result = await runLocalGraph({
        graphPath,
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome,
        env: kernelEnv(),
        adapters,
      });

      expect(result.status).toBe("needs_agent");
      if (result.status !== "needs_agent") {
        return;
      }
      expect(result.state.status).toBe("paused");

      expect(result.requests).toEqual([
        expect.objectContaining({
          id: "fanout.advisors.threshold.risk.risk_score.above",
          kind: "approval",
          gate: expect.objectContaining({
            type: "fanout-gate",
            reason: "risk.risk_score=0.91 exceeded 0.8",
          }),
        }),
      ]);

      const resumed = await runLocalGraph({
        graphPath,
        caller: approvingCaller,
        receiptDir,
        runxHome,
        env: kernelEnv(),
        adapters,
        resumeFromRunId: result.runId,
      });
      expect(resumed.status).toBe("sealed");
      if (resumed.status !== "sealed") {
        return;
      }
      expect(resumed.steps.map((step) => step.stepId)).toEqual(["market", "risk", "synthesize"]);
      expect(resumed.steps.slice(0, 2).map((step) => step.fanoutGroup)).toEqual(["advisors", "advisors"]);
      expect(resumed.output).toBe("go");
      expect(resumed.receipt.schema).toBe("runx.receipt.v1");
      expect(runtimeSyncPoints(resumed.receipt)).toEqual([
        expect.objectContaining({
          group_id: "advisors",
          decision: "pause",
          rule_fired: "threshold.risk.risk_score.above",
          reason: "risk.risk_score=0.91 exceeded 0.8",
          branch_receipts: resumed.steps.slice(0, 2).map((step) => step.receiptId),
        }),
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("records structured fanout conflicts as explicit escalation outcomes", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-fanout-escalate-"));
    const graphPath = path.join(tempDir, "graph.yaml");
    const receiptDir = path.join(tempDir, "receipts");
    const runxHome = path.join(tempDir, "home");
    const jsonSkillPath = path.resolve("fixtures/skills/json-output");

    try {
      await writeFile(
        graphPath,
        `name: fanout-escalate
fanout:
  groups:
    advisors:
      strategy: all
      on_branch_failure: halt
      conflict_gates:
        - field: recommendation
          action: escalate
          steps:
            - market
            - risk
steps:
  - id: market
    mode: fanout
    fanout_group: advisors
    skill: ${jsonSkillPath}
    inputs:
      recommendation: ship
  - id: risk
    mode: fanout
    fanout_group: advisors
    skill: ${jsonSkillPath}
    inputs:
      recommendation: hold
`,
      );

      const result = await runLocalGraph({
        graphPath,
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome,
        env: kernelEnv(),
        adapters,
      });

      expect(result.status).toBe("escalated");
      if (result.status !== "escalated") {
        return;
      }
      expect(result.receipt.schema).toBe("runx.receipt.v1");
      expect(result.receipt.seal.disposition).toBe("blocked");
      expect(result.state.status).toBe("escalated");
      expect(result.errorMessage).toBe("fanout escalation: fanout branches disagreed on structured field recommendation");
      expect(runtimeSyncPoints(result.receipt)).toEqual([
        expect.objectContaining({
          group_id: "advisors",
          decision: "escalate",
          rule_fired: "conflict.recommendation",
          reason: "fanout branches disagreed on structured field recommendation",
          branch_receipts: result.steps.slice(0, 2).map((step) => step.receiptId),
        }),
      ]);
      expect(result.receipt.metadata).toMatchObject({
        runx: {
          fanout_gate: {
            status: "escalated",
            group_id: "advisors",
            decision: "escalate",
            rule_fired: "conflict.recommendation",
          },
        },
      });
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("applies graph transition policy before fanout branch execution", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-fanout-transition-policy-"));
    const graphPath = path.join(tempDir, "graph.yaml");
    const receiptDir = path.join(tempDir, "receipts");
    const runxHome = path.join(tempDir, "home");
    const jsonSkillPath = path.resolve("fixtures/skills/json-output");

    try {
      await writeFile(
        graphPath,
        `name: fanout-transition-policy
policy:
  transitions:
    - to: market
      field: seed.allowed
      equals: true
fanout:
  groups:
    advisors:
      strategy: all
      on_branch_failure: halt
steps:
  - id: seed
    skill: ${jsonSkillPath}
    inputs:
      allowed: false
  - id: market
    mode: fanout
    fanout_group: advisors
    skill: ${jsonSkillPath}
    inputs:
      recommendation: go
  - id: risk
    mode: fanout
    fanout_group: advisors
    skill: ${jsonSkillPath}
    inputs:
      risk_score: 0.2
`,
      );

      const result = await runLocalGraph({
        graphPath,
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome,
        env: kernelEnv(),
        adapters,
      });

      expect(result.status).toBe("policy_denied");
      if (result.status !== "policy_denied") {
        return;
      }
      expect(result.reasons).toEqual([
        "transition policy blocked step 'market': expected seed.allowed == true",
      ]);
      expect(result.receipt?.schema).toBe("runx.receipt.v1");
      const graphSteps = runtimeGraphSteps(result.receipt);
      expect(graphSteps.map((step) => step.step_id)).toEqual(["seed", "market"]);
      expect(graphSteps[1]).toMatchObject({
        step_id: "market",
        status: "failure",
        disposition: "policy_denied",
        fanout_group: "advisors",
      });
      expect(result.receipt?.metadata).toMatchObject({
        authority_proof: {
          skill_name: "json-output",
          requested: {
            connected_auth: false,
            scopes: [],
            mutating: false,
          },
          credential_material: {
            status: "not_requested",
          },
        },
      });
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("denies mutating retry fanout branches without idempotency before adapter execution", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-fanout-retry-denied-"));
    const graphPath = path.join(tempDir, "graph.yaml");
    const receiptDir = path.join(tempDir, "receipts");
    const runxHome = path.join(tempDir, "home");
    const echoSkillPath = path.resolve("fixtures/skills/echo");
    const adapter = createCountingAdapter();

    try {
      await writeFile(
        graphPath,
        `name: fanout-retry-mutating-denied
fanout:
  groups:
    advisors:
      strategy: all
      on_branch_failure: halt
steps:
  - id: deploy
    mode: fanout
    fanout_group: advisors
    skill: ${echoSkillPath}
    mutation: true
    inputs:
      message: deploy
    retry:
      max_attempts: 2
`,
      );
      const result = await runLocalGraph({
        graphPath,
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome,
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
      const graphSteps = runtimeGraphSteps(result.receipt);
      expect(graphSteps).toMatchObject([
        {
          step_id: "deploy",
          status: "failure",
          disposition: "policy_denied",
          fanout_group: "advisors",
          receipt_id: undefined,
        },
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("records sync policy decisions on the harness receipt", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-fanout-inspect-"));
    const receiptDir = path.join(tempDir, "receipts");

    try {
      const result = await runLocalGraph({
        graphPath: path.resolve("fixtures/graphs/fanout/graph.yaml"),
        caller: nonInteractiveCaller,
        receiptDir,
        runxHome: path.join(tempDir, "home"),
        env: kernelEnv(),
        adapters,
      });
      expect(result.status).toBe("sealed");
      if (result.status !== "sealed") {
        return;
      }

      expect(runtimeSyncPoints(result.receipt)).toEqual([
        expect.objectContaining({
          group_id: "advisors",
          decision: "proceed",
          rule_fired: "quorum.min_success",
          reason: "2/3 branches succeeded; required 2",
        }),
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });
});

function runtimeSyncPoints(receipt: ({ readonly lineage?: { readonly sync?: unknown } } & { readonly sync_points?: unknown }) | undefined): readonly unknown[] {
  const syncPoints = receipt?.lineage?.sync ?? receipt?.sync_points;
  expect(Array.isArray(syncPoints)).toBe(true);
  return syncPoints as readonly unknown[];
}

interface RuntimeGraphStep {
  readonly step_id: string;
  readonly status?: string;
  readonly disposition?: string;
  readonly receipt_id?: string;
  readonly fanout_group?: string;
}

function runtimeGraphSteps(receipt: { readonly metadata?: Readonly<Record<string, unknown>> } | undefined): readonly RuntimeGraphStep[] {
  const runx = receipt?.metadata?.runx;
  expect(runx).toEqual(expect.any(Object));
  const steps = (runx as { readonly steps?: unknown } | undefined)?.steps;
  expect(Array.isArray(steps)).toBe(true);
  return steps as readonly RuntimeGraphStep[];
}

function createCountingAdapter(): SkillAdapter & { callCount: () => number } {
  let calls = 0;
  return {
    type: "cli-tool",
    callCount: () => calls,
    invoke: async (request) => {
      calls += 1;
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

function createConcurrencyProbeAdapter(expectedActive: number): SkillAdapter & { maxActive: () => number } {
  let active = 0;
  let maxActive = 0;
  let releaseBarrier: () => void = () => undefined;
  const barrier = new Promise<void>((resolve) => {
    releaseBarrier = resolve;
  });

  return {
    type: "cli-tool",
    maxActive: () => maxActive,
    invoke: async (request) => {
      active += 1;
      maxActive = Math.max(maxActive, active);
      if (active === expectedActive) {
        releaseBarrier();
      }
      await Promise.race([barrier, delay(500)]);
      active -= 1;
      return {
        status: "sealed",
        stdout: path.basename(request.skillDirectory),
        stderr: "",
        exitCode: 0,
        signal: null,
        durationMs: 1,
      };
    },
  };
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function writeProbeSkill(directory: string, label: string): Promise<void> {
  await mkdir(directory, { recursive: true });
  await writeFile(
    path.join(directory, "SKILL.md"),
    `---
name: ${label}
description: Emit the skill label through the injected concurrency probe.
source:
  type: cli-tool
  command: node
  args:
    - -e
    - "process.stdout.write('${label}')"
  timeout_seconds: 5
inputs: {}
---

Emit ${label}.
`,
  );
}
