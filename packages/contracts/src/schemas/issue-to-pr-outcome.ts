import { Type, type Static } from "@sinclair/typebox";
import {
  JSON_SCHEMA_DRAFT_2020_12,
  RUNX_CONTRACT_IDS,
  RUNX_LOGICAL_SCHEMAS,
  type DeepReadonly,
  dateTimeStringSchema,
  stringEnum,
  validateContractSchema,
} from "../internal.js";

export const issueToPrOutcomeSchemaVersion = "runx.issue_to_pr_outcome.v1" as const;

export const issueToPrOutcomeProviders = ["github", "gitlab", "other"] as const;
export const issueToPrProviderOutcomes = ["draft", "open", "merged", "closed", "unknown"] as const;
export const issueToPrVerificationStatuses = ["not_run", "passed", "failed", "inconclusive"] as const;
export const issueToPrSourceIssueCloseModes = ["never", "when_verified", "when_terminal"] as const;

const idSchema = Type.String({ minLength: 1, pattern: "^[A-Za-z0-9_.:-]+$" });
const repoSlugSchema = Type.String({
  minLength: 3,
  pattern: "^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$",
});
const uriSchema = Type.String({ minLength: 1 });
const providerSchema = stringEnum(issueToPrOutcomeProviders);
const providerOutcomeSchema = stringEnum(issueToPrProviderOutcomes);
const verificationStatusSchema = stringEnum(issueToPrVerificationStatuses);
const closeModeSchema = stringEnum(issueToPrSourceIssueCloseModes);

const sourceThreadSchema = Type.Object(
  {
    required: Type.Literal(true),
    publish_mode: Type.Literal("reply"),
    missing_behavior: Type.Literal("fail_closed"),
    thread_locator: uriSchema,
  },
  { additionalProperties: false },
);

const sourceIssueSchema = Type.Object(
  {
    provider: providerSchema,
    locator: uriSchema,
    url: Type.Optional(uriSchema),
    number: Type.Optional(Type.Integer({ minimum: 1 })),
    status: Type.Optional(stringEnum(["open", "closed", "unknown"] as const)),
  },
  { additionalProperties: false },
);

const pullRequestSchema = Type.Object(
  {
    provider: providerSchema,
    repo: repoSlugSchema,
    number: Type.Integer({ minimum: 1 }),
    url: uriSchema,
    state: providerOutcomeSchema,
    merged: Type.Boolean(),
    merged_at: Type.Optional(dateTimeStringSchema()),
    base_branch: Type.Optional(Type.String({ minLength: 1 })),
    head_branch: Type.Optional(Type.String({ minLength: 1 })),
  },
  { additionalProperties: false },
);

const verificationEvidenceSchema = Type.Object(
  {
    label: Type.String({ minLength: 1 }),
    summary: Type.String({ minLength: 1 }),
    redacted: Type.Optional(Type.Boolean()),
  },
  { additionalProperties: false },
);

const verificationSchema = Type.Object(
  {
    required: Type.Boolean(),
    status: verificationStatusSchema,
    summary: Type.Optional(Type.String({ minLength: 1 })),
    evidence: Type.Optional(Type.Array(verificationEvidenceSchema)),
  },
  { additionalProperties: false },
);

const publishPolicySchema = Type.Object(
  {
    final_source_thread_update: Type.Literal(true),
    close_source_issue: closeModeSchema,
    close_permitted: Type.Boolean(),
  },
  { additionalProperties: false },
);

const humanGateSchema = Type.Object(
  {
    required: Type.Literal(true),
    merged_by: Type.Optional(Type.String({ minLength: 1 })),
    reviewed_by: Type.Optional(Type.Array(Type.String({ minLength: 1 }))),
  },
  { additionalProperties: false },
);

export const issueToPrOutcomeSchema = Type.Object(
  {
    schema: Type.Literal(RUNX_LOGICAL_SCHEMAS.issueToPrOutcome),
    schema_version: Type.Literal(issueToPrOutcomeSchemaVersion),
    outcome_id: idSchema,
    task_id: idSchema,
    observed_at: dateTimeStringSchema(),
    provider_outcome: providerOutcomeSchema,
    source_thread: sourceThreadSchema,
    source_issue: Type.Optional(sourceIssueSchema),
    pull_request: pullRequestSchema,
    verification: verificationSchema,
    publish: publishPolicySchema,
    human_gate: humanGateSchema,
  },
  {
    $schema: JSON_SCHEMA_DRAFT_2020_12,
    $id: RUNX_CONTRACT_IDS.issueToPrOutcome,
    "x-runx-schema": RUNX_LOGICAL_SCHEMAS.issueToPrOutcome,
    additionalProperties: false,
  },
);

export type IssueToPrOutcomeContract = DeepReadonly<Static<typeof issueToPrOutcomeSchema>>;

export interface IssueToPrOutcomeFinding {
  readonly code: string;
  readonly path: string;
  readonly message: string;
}

export function validateIssueToPrOutcomeContract(
  value: unknown,
  label = "issue_to_pr_outcome",
): IssueToPrOutcomeContract {
  return validateContractSchema(issueToPrOutcomeSchema, value, label);
}

export function lintIssueToPrOutcomeContract(value: unknown): readonly IssueToPrOutcomeFinding[] {
  const outcome = validateIssueToPrOutcomeContract(value);
  const findings: IssueToPrOutcomeFinding[] = [];

  if (outcome.provider_outcome === "merged" && (outcome.pull_request.state !== "merged" || !outcome.pull_request.merged)) {
    findings.push({
      code: "merged_outcome_mismatch",
      path: "/pull_request",
      message: "provider_outcome=merged requires pull_request.state=merged and pull_request.merged=true.",
    });
  }
  if (outcome.publish.close_source_issue === "when_verified" && outcome.verification.status !== "passed") {
    findings.push({
      code: "close_requires_passed_verification",
      path: "/publish/close_source_issue",
      message: "close_source_issue=when_verified requires verification.status=passed.",
    });
  }
  if (outcome.verification.required && outcome.verification.status === "not_run") {
    findings.push({
      code: "verification_required_not_run",
      path: "/verification/status",
      message: "verification.required=true cannot publish a terminal outcome with verification.status=not_run.",
    });
  }
  if (outcome.publish.close_permitted && outcome.publish.close_source_issue === "never") {
    findings.push({
      code: "close_permission_without_close_mode",
      path: "/publish/close_permitted",
      message: "close_permitted=true requires close_source_issue to be when_verified or when_terminal.",
    });
  }

  return findings;
}

export function validateIssueToPrOutcomeSemantics(
  value: unknown,
  label = "issue_to_pr_outcome",
): IssueToPrOutcomeContract {
  const outcome = validateIssueToPrOutcomeContract(value, label);
  const findings = lintIssueToPrOutcomeContract(outcome);
  if (findings.length > 0) {
    const first = findings[0];
    throw new Error(`${label}${first.path} failed semantic validation (${first.code}): ${first.message}`);
  }
  return outcome;
}
