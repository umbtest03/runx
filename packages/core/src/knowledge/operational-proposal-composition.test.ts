import { readFileSync } from "node:fs";

import { validateOperationalProposalContract } from "@runxhq/contracts";
import { describe, expect, it } from "vitest";

const compositionFixture = new URL("../../../../fixtures/operational-proposal/public/composition-paths.json", import.meta.url);

describe("operational proposal composition fixtures", () => {
  it("build fix without prior check remains governed by action authority", () => {
    const path = fixturePath("build_fix_without_prior_check");
    const proposal = validateOperationalProposalContract(path.proposal);

    expect(proposal.proposal_kind).toBe("escalation");
    expect(proposal.recommended_actions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          action_intent: "issue-to-pr",
          mutating: true,
        }),
      ]),
    );
    expect(proposal.authority).toMatchObject({
      proposal_only: true,
      mutation_authority_granted: false,
      publication_authority_granted: false,
      final_decision_authority_granted: false,
    });
    expect(proposal.human_gates).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          gate_kind: "final_change_approval",
          required: true,
        }),
      ]),
    );
    expect(proposal.result_refs?.map((link) => link.role)).toEqual(
      expect.arrayContaining(["tracking_item", "change_request"]),
    );
    expect(proposal.publication_refs?.map((link) => link.role)).toEqual(
      expect.arrayContaining(["source_thread_update", "tracking_item_comment", "change_request_comment"]),
    );
    expect(proposal.final_outcome).toMatchObject({
      observed: true,
      status: "merged",
    });
    expect(escalationExtension(proposal)).toMatchObject({
      severity: "high",
      urgency: "same_day",
    });
  });

  it("check does not grant mutation and prior check advisory stays explicit", () => {
    const path = fixturePath("read_only_check");
    const proposal = validateOperationalProposalContract(path.proposal);

    expect(proposal.recommended_actions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          action_intent: "check",
          mutating: false,
        }),
      ]),
    );
    expect(proposal.authority).toMatchObject({
      proposal_only: true,
      mutation_authority_granted: false,
      publication_authority_granted: false,
      final_decision_authority_granted: false,
    });
    expect(proposal.authority.notes).toEqual(
      expect.arrayContaining([
        "check does not grant mutation permission",
        "prior check advisory",
      ]),
    );
  });

  it("escalation proposals carry severity and urgency without provider fields", () => {
    for (const path of fixturePaths()) {
      const proposal = validateOperationalProposalContract(path.proposal);
      if (proposal.proposal_kind !== "escalation") {
        continue;
      }

      const escalation = escalationExtension(proposal);
      const proposalWire = JSON.stringify(proposal);

      expect(escalation).toMatchObject({
        severity: expect.any(String),
        urgency: expect.any(String),
      });
      expect(proposalWire).not.toContain("channel_id");
      expect(proposalWire).not.toContain("owner_email");
    }
  });
});

function fixturePath(pathId: string): {
  readonly path_id: string;
  readonly proposal: unknown;
} {
  const path = fixturePaths().find((candidate) => candidate.path_id === pathId);
  if (!path) {
    throw new Error(`missing operational proposal composition fixture: ${pathId}`);
  }
  return path;
}

function fixturePaths(): readonly {
  readonly path_id: string;
  readonly proposal: unknown;
}[] {
  const fixture = JSON.parse(readFileSync(compositionFixture, "utf8")) as {
    readonly paths: readonly {
      readonly path_id: string;
      readonly proposal: unknown;
    }[];
  };
  return fixture.paths;
}

function escalationExtension(proposal: {
  readonly extensions?: Readonly<Record<string, unknown>>;
}): Readonly<Record<string, unknown>> {
  const extension = proposal.extensions?.["runx.escalation"];
  if (!isRecord(extension)) {
    throw new Error("escalation proposal is missing runx.escalation extension.");
  }
  return extension;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}
