import { existsSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { buildInlineGraphStepSkill } from "./execution-targets.js";
import type { GraphStep } from "../parser-types.js";

const workspaceRoot = process.cwd();
const cargoTargetDir = process.env.CARGO_TARGET_DIR
  ? path.resolve(workspaceRoot, process.env.CARGO_TARGET_DIR)
  : path.join(workspaceRoot, "crates", "target");
const defaultRunxBinary = path.join(
  cargoTargetDir,
  "debug",
  process.platform === "win32" ? "runx.exe" : "runx",
);

function resolveRunxBinary(): string {
  const candidate = process.env.RUNX_RUST_CLI_BIN ?? process.env.RUNX_KERNEL_EVAL_BIN ?? defaultRunxBinary;
  const resolved = path.resolve(workspaceRoot, candidate);
  if (!existsSync(resolved)) {
    throw new Error(
      `execution-targets tests require a prebuilt Rust binary; set RUNX_RUST_CLI_BIN or build it at ${path.relative(
        workspaceRoot,
        defaultRunxBinary,
      )}.`,
    );
  }
  return resolved;
}

function graphStep(overrides: Partial<GraphStep> & Pick<GraphStep, "id" | "run">): GraphStep {
  return {
    inputs: {},
    context: {},
    contextEdges: [],
    scopes: [],
    mutating: false,
    ...overrides,
  };
}

describe("buildInlineGraphStepSkill", () => {
  // Regression guard: a `run: { type: approval }` step is a native run-step (the
  // approval gate), not a skill source. The kernel parser does not list approval
  // among its source kinds, so it must be synthesized locally rather than routed
  // through validateSkillSourceViaParser (which previously failed closed with
  // "source.type approval is not a supported source type").
  it("synthesizes the approval gate source instead of validating it as a skill source", async () => {
    // Before the fix this threw "source.type approval is not a supported source
    // type" because the run-step source was sent to the skill-source parser.
    const skill = await buildInlineGraphStepSkill(
      graphStep({
        id: "approve",
        run: { type: "approval" },
        inputs: { gate_id: "flow.approval", reason: "Approve before continuing." },
      }),
      undefined,
      { command: resolveRunxBinary(), cwd: workspaceRoot },
    );

    expect(skill.source.type).toBe("approval");
    expect(skill.source.args).toEqual([]);
    expect(skill.source.raw).toMatchObject({ type: "approval" });
  });

  // Guard the other branch: a `run:` block carrying a real skill source kind is
  // still validated through the parser as an inline skill.
  it("validates an inline skill-source run-step through the parser", async () => {
    const skill = await buildInlineGraphStepSkill(
      graphStep({
        id: "echo",
        run: { type: "cli-tool", command: "node", args: ["-e", "process.stdout.write('{}')"] },
      }),
      undefined,
      { command: resolveRunxBinary(), cwd: workspaceRoot },
    );

    expect(skill.source.type).toBe("cli-tool");
    expect(skill.source.command).toBe("node");
    expect(skill.source.args).toEqual(["-e", "process.stdout.write('{}')"]);
  });
});
