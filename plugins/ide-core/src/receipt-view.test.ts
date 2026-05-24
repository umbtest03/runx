import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { buildReceiptViewModel } from "./receipt-view.js";

function loadReceipt(relativePath: string): unknown {
  const url = new URL(`../../../${relativePath}`, import.meta.url);
  return JSON.parse(readFileSync(fileURLToPath(url), "utf8"));
}

describe("buildReceiptViewModel", () => {
  it("renders the seal, acts, and decisions of a flat runx.receipt.v1", () => {
    const receipt = loadReceipt("fixtures/harness/oracle/echo-skill.receipt.json");
    const model = buildReceiptViewModel(receipt);

    const root = model.nodes[0];
    expect(root?.kind).toBe("receipt");
    expect(root?.status).toBe("closed");
    expect(root?.id).toBe("sha256:0bf91d3b12fe6c8c411f0a0c5b20d190947da6430dcb768f4f8887f08cff598f");

    const act = model.nodes.find((node) => node.kind === "act");
    expect(act?.status).toBe("closed");
    expect(act?.detail?.form).toBe("observation");

    const decision = model.nodes.find((node) => node.kind === "decision");
    expect(decision?.status).toBe("open");

    // Every non-root node hangs off the receipt root.
    for (const edge of model.edges) {
      expect(edge.from).toBe(root?.id);
    }
  });

  it("reads child lineage and fan-out sync points, not the legacy harness shape", () => {
    const model = buildReceiptViewModel({
      schema: "runx.receipt.v1",
      id: "sha256:abc",
      seal: { disposition: "closed", reason_code: "ok", summary: "done" },
      subject: { ref: { type: "skill", uri: "skill:demo", label: "Demo Skill" } },
      lineage: {
        children: [{ type: "receipt", uri: "runx:receipt:sha256:child" }],
        sync: [
          {
            group_id: "g1",
            strategy: "quorum",
            decision: "proceed",
            branch_count: 2,
            success_count: 2,
            failure_count: 0,
            required_successes: 1,
          },
        ],
      },
      acts: [],
      decisions: [],
    });

    expect(model.title).toBe("Demo Skill");
    expect(model.nodes.find((node) => node.kind === "child-ref")?.label).toContain("runx:receipt:sha256:child");
    expect(model.nodes.find((node) => node.kind === "sync")?.status).toBe("proceed");
  });

  it("returns an empty model for a non-record receipt", () => {
    const model = buildReceiptViewModel(null);
    expect(model).toEqual({ title: "Invalid receipt", nodes: [], edges: [] });
  });
});
