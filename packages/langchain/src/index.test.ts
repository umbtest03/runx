import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it, vi } from "vitest";
import { z } from "zod";

import {
  createLangChainToolCatalogAdapter,
  createRunxCliSkillRunner,
  createRunxLangChainTool,
  type RunxSkillCliResult,
} from "./index.js";

describe("@runxhq/langchain", () => {
  it("sunsets in-process LangChain tool-catalog adapters explicitly", () => {
    expect(() => createLangChainToolCatalogAdapter({
      source: "langchain-demo",
      label: "LangChain Demo",
      namespace: "langchain",
      baseDirectory: process.cwd(),
      tools: [],
    })).toThrow("was sunset with the Rust runtime takeover");
  });

  it("invokes governed runx skills through the CLI JSON boundary", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-langchain-"));
    const commandPath = path.join(tempDir, "fake-runx.mjs");
    const capturePath = path.join(tempDir, "argv.json");
    try {
      await writeFile(commandPath, [
        "#!/usr/bin/env node",
        "import { writeFileSync } from 'node:fs';",
        "writeFileSync(process.env.RUNX_LANGCHAIN_CAPTURE_PATH, JSON.stringify(process.argv.slice(2)));",
        "process.stdout.write(JSON.stringify({ status: 'sealed', execution: { stdout: 'from-cli', stderr: '', exit_code: 0 } }));",
        "",
      ].join("\n"));
      await chmod(commandPath, 0o755);

      const runner = createRunxCliSkillRunner({
        command: commandPath,
        env: {
          ...process.env,
          RUNX_LANGCHAIN_CAPTURE_PATH: capturePath,
        },
      });
      const result = await runner.runSkill({
        skillPath: "/tmp/skills/docs-pr",
        receiptDir: "/tmp/receipts",
        runId: "run_123",
        answersPath: "/tmp/answers.json",
        inputs: {
          repo_url: "acme/docs",
          count: 3,
          nested: { ok: true },
        },
      });

      expect(result).toEqual({
        status: "sealed",
        execution: {
          stdout: "from-cli",
          stderr: "",
          exit_code: 0,
        },
      });
      await expect(readFile(capturePath, "utf8").then(JSON.parse)).resolves.toEqual([
        "skill",
        "/tmp/skills/docs-pr",
        "--json",
        "--receipt-dir",
        "/tmp/receipts",
        "--run-id",
        "run_123",
        "--answers",
        "/tmp/answers.json",
        "--repo-url",
        "acme/docs",
        "--count",
        "3",
        "--nested",
        "{\"ok\":true}",
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  }, 60_000);

  it("wraps a governed runx workflow as a LangChain tool", async () => {
    const runSkill = vi.fn(async (): Promise<RunxSkillCliResult> => successResult("wrapped-output"));

    const wrapped = createRunxLangChainTool({
      name: "docs_pr",
      description: "Open a governed docs PR workflow.",
      schema: z.object({
        repo: z.string(),
      }),
      skillPath: "/tmp/skills/docs-pr",
      cli: { runSkill },
      mapInput: (input) => {
        const record = input as { repo: string };
        return { repo_url: record.repo };
      },
    });

    const output = await wrapped.invoke({ repo: "acme/docs" });
    expect(output).toBe("wrapped-output");
    expect(runSkill).toHaveBeenCalledWith(expect.objectContaining({
      skillPath: "/tmp/skills/docs-pr",
      inputs: { repo_url: "acme/docs" },
    }));
  });

  it("fails fast when a wrapped runx workflow pauses for resolution", async () => {
    const runSkill = vi.fn(async (): Promise<RunxSkillCliResult> => ({
      status: "needs_agent",
      schema: "runx.skill_run.v1",
      run_id: "run_123",
      requests: [],
    }));

    const wrapped = createRunxLangChainTool({
      name: "review_pr",
      description: "Run governed review.",
      schema: z.object({
        repo: z.string(),
      }),
      skillPath: "/tmp/skills/review",
      cli: { runSkill },
    });

    await expect(wrapped.invoke({ repo: "acme/docs" })).rejects.toThrow(
      "needs agent input",
    );
  });
});

function successResult(stdout: string): RunxSkillCliResult {
  return {
    status: "sealed",
    schema: "runx.skill_run.v1",
    execution: {
      stdout,
      stderr: "",
      exit_code: 0,
    },
  };
}
