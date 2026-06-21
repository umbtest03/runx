import { spawnSync } from "node:child_process";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { validateDataOperationResultContract } from "../packages/contracts/src/index.js";

const adapterPath = path.resolve("skills/data-store/tools/data/local/run.mjs");

describe("data-store local adapter", () => {
  it("emits the governed data operation result contract", () => {
    const storeId = `contract-test-${process.pid}-${Date.now()}`;
    const result = runLocalDataAdapter({
      operation: "append_event",
      data_source_ref: "local://runx-data-store/contract-test",
      store_id: storeId,
      resource: "board_events",
      aggregate_id: "posting-123",
      expected_version: 0,
      idempotency_key: "posting-123:create:v1",
      event: {
        type: "posting.created",
        payload: {
          title: "verify a receipt link",
        },
      },
    });

    const packet = validateDataOperationResultContract(JSON.parse(result.stdout));
    expect(packet.status).toBe("committed");
    expect(packet.operation).toBe("append_event");
    expect(packet.provider).toBe("local-json-event-store");
  });
});

function runLocalDataAdapter(inputs: unknown): { readonly stdout: string } {
  const result = spawnSync(process.execPath, [adapterPath], {
    cwd: path.resolve("."),
    encoding: "utf8",
    env: {
      ...process.env,
      RUNX_INPUTS_JSON: JSON.stringify(inputs),
    },
  });

  expect(result.status).toBe(0);
  expect(result.stderr).toBe("");
  expect(result.stdout.trim()).not.toBe("");
  return { stdout: result.stdout };
}
