import { describe, expect, it } from "vitest";

import { renderHistory } from "./history.js";
import type { LocalReceiptSummary } from "@runxhq/runtime-local";

describe("renderHistory", () => {
  it("surfaces compact work-item status summaries", () => {
    const output = renderHistory([
      {
        id: "rx_history_work_item",
        name: "issue-intake",
        kind: "skill_execution",
        status: "success",
        sourceType: "agent-step",
        verification: {
          status: "verified",
          reason: "ok",
        },
        ledgerVerification: {
          status: "valid",
          reason: "ok",
          runId: "run_history_work_item",
          ledgerPath: "/tmp/runx-history-fixture/ledger.jsonl",
          entryCount: 1,
          headHash: "sha256:fixture",
        },
        workItemId: "wi_fixture_123",
        workItemState: "merge_gate",
        workItemStatusSummary: "PR is ready for human merge gate.",
      } as LocalReceiptSummary,
    ], { NO_COLOR: "1" } as NodeJS.ProcessEnv);

    expect(output).toContain("work item wi_fixture_123");
    expect(output).toContain("merge_gate");
    expect(output).toContain("PR is ready for human merge gate.");
  });
});
