import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { runCli } from "./index.js";
import { TRAINING_SCHEMA_REFS } from "./trainable-receipts.js";

describe("trainable receipts export", () => {
  it("publishes the canonical trainable harness row schema ref", () => {
    expect(TRAINING_SCHEMA_REFS.trainable_receipt_row).toBe(
      "https://runx.ai/spec/training/trainable-receipt-row.schema.json",
    );
  });

  it("streams filtered JSONL records from receipt fixtures", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-trainable-receipts-"));
    const directory = path.join(tempDir, "receipts");

    try {
      await mkdir(directory, { recursive: true });
      const fixtureDocument = JSON.parse(await readFile(path.resolve("fixtures/contracts/harness-spine/receipt-success.json"), "utf8")) as {
        expected: { id: string };
      };
      const fixture = fixtureDocument.expected;
      await writeFile(path.join(directory, `${fixture.id}.json`), `${JSON.stringify(fixture, null, 2)}\n`);

      const stdout = createMemoryStream();
      const stderr = createMemoryStream();
      const exitCode = await runCli(
        [
          "export-receipts",
          "--trainable",
          "--receipt-dir",
          directory,
          "--since",
          "2026-05-17T00:00:00Z",
          "--status",
          "closed",
          "--source",
          "principal",
        ],
        { stdin: process.stdin, stdout, stderr },
        {
          ...process.env,
          RUNX_CWD: process.cwd(),
        },
      );

      expect(exitCode).toBe(0);
      expect(stderr.contents()).toBe("");

      const rows = stdout.contents().trim().split("\n").map((line) => JSON.parse(line));
      expect(rows).toHaveLength(1);
      const row = rows[0];
      expect(row).toMatchObject({
        kind: "runx.trainable-receipt-row.v1",
        receipt_id: fixture.id,
        disposition: "closed",
        actor_ref: {
          type: "principal",
        },
      });
      expect(typeof row.exported_at).toBe("string");
      expect(row.receipt.id).toBe(fixture.id);

      // A rich trainable row carries intent purposes, success-criteria
      // statements, decision justifications, and criterion OUTCOMES (not ids).
      expect(row.acts[0].intent_purpose).toBe("Execute the requested skill step");
      expect(row.acts[0].success_criteria[0]).toMatchObject({
        criterion_id: "process_exit",
        statement: "cli-tool exits successfully",
        required: true,
      });
      expect(row.acts[0].criterion_outcomes[0]).toMatchObject({
        criterion_id: "process_exit",
        status: "verified",
      });
      expect(row.acts[0].criterion_outcomes[0].summary).toBe("cli-tool exited successfully");
      expect(row.decisions[0].justification).toBe(
        "runtime graph planner selected this node",
      );
      expect(row.decisions[0].selected_act_id).toBe("act_echo");
      // The training INPUT and OUTCOME are present.
      expect(row.input?.source).toContain("runx:signal:");
      expect(row.outcome.disposition).toBe("closed");
      expect(row.outcome.criteria[0].criterion_id).toBe("process_exit");
      // Verification is computed on read.
      expect(row.verification).toMatchObject({
        criteria_bound: true,
        selected_acts_resolved: true,
        signature_present: true,
        digest_present: true,
      });
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
