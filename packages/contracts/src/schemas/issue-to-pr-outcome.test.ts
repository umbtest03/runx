import { Value } from "@sinclair/typebox/value";
import { describe, expect, it } from "vitest";

import {
  issueToPrOutcomeSchema,
  issueToPrOutcomeSchemaVersion,
  lintIssueToPrOutcomeContract,
  validateIssueToPrOutcomeContract,
  validateIssueToPrOutcomeSemantics,
} from "./issue-to-pr-outcome.js";

const validOutcome = {
  schema: "runx.issue_to_pr_outcome.v1",
  schema_version: issueToPrOutcomeSchemaVersion,
  outcome_id: "outcome-123",
  task_id: "issue-123",
  observed_at: "2026-05-19T00:00:00.000Z",
  provider_outcome: "merged",
  source_thread: {
    required: true,
    publish_mode: "reply",
    missing_behavior: "fail_closed",
    thread_locator: "slack://nitrosend/C0APFMY0V8Q/1778834840.485629",
  },
  source_issue: {
    provider: "github",
    locator: "nitrosend/nitrosend#168",
    url: "https://github.com/nitrosend/nitrosend/issues/168",
    number: 168,
    status: "open",
  },
  pull_request: {
    provider: "github",
    repo: "nitrosend/nitrosend",
    number: 169,
    url: "https://github.com/nitrosend/nitrosend/pull/169",
    state: "merged",
    merged: true,
    merged_at: "2026-05-19T00:01:00.000Z",
    base_branch: "main",
    head_branch: "runx/issue-123",
  },
  verification: {
    required: true,
    status: "passed",
    summary: "Live verification passed with redacted output.",
    evidence: [
      {
        label: "smoke",
        summary: "HTTP 200 from product surface; identifiers redacted.",
        redacted: true,
      },
    ],
  },
  publish: {
    final_source_thread_update: true,
    close_source_issue: "when_verified",
    close_permitted: true,
  },
  human_gate: {
    required: true,
    merged_by: "Kam",
    reviewed_by: ["Kam"],
  },
} as const;

describe("issue-to-pr outcome schema", () => {
  it("validates merged PR outcome packets for final source-thread updates", () => {
    expect(Value.Check(issueToPrOutcomeSchema, validOutcome)).toBe(true);
    expect(validateIssueToPrOutcomeContract(validOutcome)).toMatchObject({
      outcome_id: "outcome-123",
      provider_outcome: "merged",
    });
    expect(validateIssueToPrOutcomeSemantics(validOutcome)).toMatchObject({
      publish: {
        close_source_issue: "when_verified",
      },
    });
  });

  it("rejects schema drift and raw unsupported fields", () => {
    expect(Value.Check(issueToPrOutcomeSchema, {
      ...validOutcome,
      local_path: "/Users/kam/dev/runx",
    })).toBe(false);
    expect(Value.Check(issueToPrOutcomeSchema, {
      ...validOutcome,
      schema_version: "runx.issue_to_pr_outcome.v0",
    })).toBe(false);
  });

  it("flags semantic mismatch when a merged outcome is not merged provider state", () => {
    const findings = lintIssueToPrOutcomeContract({
      ...validOutcome,
      pull_request: {
        ...validOutcome.pull_request,
        state: "open",
        merged: false,
      },
    });

    expect(findings.map((finding) => finding.code)).toContain("merged_outcome_mismatch");
  });

  it("requires passed verification before close_source_issue=when_verified", () => {
    expect(() =>
      validateIssueToPrOutcomeSemantics({
        ...validOutcome,
        verification: {
          ...validOutcome.verification,
          status: "inconclusive",
        },
      })
    ).toThrow(/close_requires_passed_verification/);
  });
});
