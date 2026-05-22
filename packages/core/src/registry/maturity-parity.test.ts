import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { computeMaturity, type MaturitySignals } from "./maturity.js";
import type { MaturityTier } from "./store.js";

/**
 * Cross-language parity for compute_maturity. This reads the same fixture as
 * the Rust test (runx-core/tests/maturity_parity.rs), so the TypeScript
 * `computeMaturity` mirror cannot drift from the canonical Rust implementation.
 */
interface ParityCase {
  readonly name: string;
  readonly signals: MaturitySignals;
  readonly expected: MaturityTier;
}

const fixturePath = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../../fixtures/kernel/maturity/compute-maturity-cases.json",
);
const cases = JSON.parse(readFileSync(fixturePath, "utf8")) as ParityCase[];

describe("computeMaturity cross-language parity (TS mirror of runx-core)", () => {
  it("declares at least one case", () => {
    expect(cases.length).toBeGreaterThan(0);
  });

  it.each(cases)("$name", (testCase) => {
    expect(computeMaturity(testCase.signals)).toBe(testCase.expected);
  });
});
