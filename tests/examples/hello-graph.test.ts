import { execFile } from "node:child_process";
import { existsSync } from "node:fs";
import { promisify } from "node:util";

import { describe, expect, it } from "vitest";

const execFileAsync = promisify(execFile);
const nativeRunx = `crates/target/debug/${process.platform === "win32" ? "runx.exe" : "runx"}`;

describe("hello-graph example", () => {
  it("runs through the native graph harness", async () => {
    const { stdout, stderr } = await execFileAsync(
      requireNativeRunx(),
      ["harness", "examples/hello-graph/harness.yaml", "--json"],
      {
        env: { ...process.env, NO_COLOR: "1" },
      },
    );

    expect(stderr).toBe("");
    const receipt = JSON.parse(stdout) as {
      readonly schema?: string;
      readonly harness?: { readonly state?: string; readonly child_harness_receipt_refs?: readonly unknown[] };
      readonly seal?: { readonly disposition?: string };
    };
    expect(receipt.schema).toBe("runx.harness_receipt.v1");
    expect(receipt.harness?.state).toBe("sealed");
    expect(receipt.seal?.disposition).toBe("closed");
    expect(receipt.harness?.child_harness_receipt_refs?.length).toBe(2);
  });
});

function requireNativeRunx(): string {
  if (!existsSync(nativeRunx)) {
    throw new Error(`native example tests require a built runx binary at ${nativeRunx}`);
  }
  return nativeRunx;
}
