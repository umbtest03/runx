import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { buildRegistrySkillVersion } from "@runxhq/core/registry";
import {
  parseRunnerManifestYaml,
  parseSkillMarkdown,
  validateRunnerManifest,
  validateSkill,
} from "@runxhq/core/parser";

const paymentSecretKeyPattern = /(?:^|_)(?:pan|cvv|cvc|card_number|cardnumber|account_number|routing_number|private_key|seed_phrase|mnemonic|secret_key)(?:$|_)/i;

describe("payment skill execution profiles", () => {
  it("parse payment profiles and ingest packaged skills without raw payment credential fields", async () => {
    const skillDirs = await discoverPaymentSkillDirs();

    for (const skillDir of skillDirs) {
      const skillName = path.basename(skillDir);
      const profileDocument = await readFile(path.join(skillDir, "X.yaml"), "utf8");
      const manifest = validateRunnerManifest(parseRunnerManifestYaml(profileDocument));

      expect(manifest.skill, `${skillName} profile names its skill`).toBe(skillName);
      expect(Object.keys(manifest.runners), `${skillName} declares runners`).not.toHaveLength(0);
      expect(findPaymentSecretFields(manifest.raw.document), `${skillName} raw payment credential fields`).toEqual([]);

      const markdown = await readOptionalFile(path.join(skillDir, "SKILL.md"));
      if (markdown) {
        const skill = validateSkill(parseSkillMarkdown(markdown), { mode: "strict" });
        expect(manifest.skill ?? skill.name, `${skill.name} profile skill binding`).toBe(skill.name);

        const version = buildRegistrySkillVersion(markdown, {
          owner: "runx-payments",
          version: "validation",
          profileDocument,
        });
        expect(version.profile_document).toBe(profileDocument);
        expect(version.profile_digest).toMatch(/^[a-f0-9]{64}$/);
        expect(version.runner_names).toEqual(Object.keys(manifest.runners));
      }
    }
  });
});

async function discoverPaymentSkillDirs(): Promise<readonly string[]> {
  const skillsRoot = path.resolve("skills");
  const entries = await readdir(skillsRoot, { withFileTypes: true });
  const candidates = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory())
      .map(async (entry) => {
        const skillDir = path.join(skillsRoot, entry.name);
        const profileDocument = await readOptionalFile(path.join(skillDir, "X.yaml"));
        if (!profileDocument) {
          return undefined;
        }
        if (entry.name.includes("payment")) {
          return skillDir;
        }
        return /\bresource_family:\s*payment\b|\bpayment[.:_-]/.test(profileDocument) ? skillDir : undefined;
      }),
  );
  return candidates.filter((entry): entry is string => entry !== undefined).sort();
}

async function readOptionalFile(filePath: string): Promise<string | undefined> {
  try {
    return await readFile(filePath, "utf8");
  } catch (error) {
    if (isRecord(error) && error.code === "ENOENT") {
      return undefined;
    }
    throw error;
  }
}

function findPaymentSecretFields(value: unknown, pathParts: readonly string[] = []): readonly string[] {
  if (Array.isArray(value)) {
    return value.flatMap((entry, index) => findPaymentSecretFields(entry, [...pathParts, `[${index}]`]));
  }
  if (!isRecord(value)) {
    return [];
  }
  return Object.entries(value).flatMap(([key, entry]) => {
    const fieldPath = [...pathParts, key];
    const current = paymentSecretKeyPattern.test(key) ? [fieldPath.join(".")] : [];
    return [...current, ...findPaymentSecretFields(entry, fieldPath)];
  });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
