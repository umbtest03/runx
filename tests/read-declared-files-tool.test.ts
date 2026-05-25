import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

const toolPath = path.resolve("tools/spec/read_declared_files/run.mjs");

describe("spec.read_declared_files tool", () => {
  it("hydrates nearby request specs for declared Rails controller targets", () => {
    const repoRoot = mkdtempSync(path.join(os.tmpdir(), "runx-read-declared-files-"));
    writeFixture(repoRoot, "app/controllers/api/v1/my/subscription_controller.rb", "class Api::V1::My::SubscriptionController; end\n");
    writeFixture(repoRoot, "spec/requests/api/v1/my/subscription_authorization_spec.rb", "RSpec.describe 'subscription auth' do\nend\n");

    const result = runTool({
      repo_root: repoRoot,
      spec_contents: [
        "## Context",
        "",
        "Files impacted:",
        "- `app/controllers/api/v1/my/subscription_controller.rb`",
        "",
        "## Phase 1: Fix checkout",
        "",
        "Changes:",
        "- `app/controllers/api/v1/my/subscription_controller.rb` (all, exclusive) - Fix checkout return URLs.",
      ].join("\n"),
    });

    expect(result.status).toBe(0);
    const output = JSON.parse(result.stdout);
    expect(output.data.files.map((file: { path: string }) => file.path)).toContain("spec/requests/api/v1/my/subscription_authorization_spec.rb");
    expect(output.data.files.find((file: { path: string }) => file.path === "spec/requests/api/v1/my/subscription_authorization_spec.rb")).toMatchObject({
      declared_in: ["related.test"],
      exists: true,
    });
  });
});

function runTool(inputs: Readonly<Record<string, unknown>>) {
  return spawnSync("node", [toolPath], {
    cwd: path.resolve("."),
    encoding: "utf8",
    env: {
      ...process.env,
      RUNX_INPUTS_JSON: JSON.stringify(inputs),
    },
  });
}

function writeFixture(repoRoot: string, relativePath: string, contents: string) {
  const absolutePath = path.join(repoRoot, relativePath);
  mkdirSync(path.dirname(absolutePath), { recursive: true });
  writeFileSync(absolutePath, contents);
}
