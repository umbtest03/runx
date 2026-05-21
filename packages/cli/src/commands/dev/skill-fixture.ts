import {
  failedFixture,
  resolveSkillDirFromRef,
  type DevFixtureResult,
} from "./internal.js";

export async function runSkillFixture(
  root: string,
  fixturePath: string,
  name: string,
  lane: string,
  target: Readonly<Record<string, unknown>>,
  startedAt: number,
): Promise<DevFixtureResult> {
  const ref = typeof target.ref === "string" ? target.ref : "";
  const skillPath = resolveSkillDirFromRef(root, ref);
  if (!skillPath) {
    return failedFixture(name, lane, target, startedAt, [{
      path: "target.ref",
      expected: "existing skill",
      actual: ref,
      kind: "exact_mismatch",
      message: `Skill or graph ${ref} was not found.`,
    }]);
  }
  return failedFixture(name, lane, target, startedAt, [{
    path: "target.ref",
    expected: "native runx dev fixture execution",
    actual: fixturePath,
    kind: "exact_mismatch",
    message: "TypeScript skill-fixture execution is retired; run native `runx dev --json` for governed fixture execution.",
  }]);
}
