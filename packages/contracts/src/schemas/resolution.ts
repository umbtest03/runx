import { Type, type Static } from "@sinclair/typebox";
import {
  JSON_SCHEMA_DRAFT_2020_12,
  RUNX_CONTROL_SCHEMA_REFS,
  type DeepReadonly,
  asUnknownRecord,
  stringEnum,
  unknownRecordSchema,
  validateContractSchema,
} from "../internal.js";
import { agentActInvocationSchema, approvalGateSchema, questionSchema } from "./agent-act.js";

const resolutionResponseActors = ["human", "agent"] as const;
const actReceiptTerminalStatuses = ["success", "failure"] as const;
const nodeSignalNames = [
  "SIGABRT",
  "SIGALRM",
  "SIGBUS",
  "SIGCHLD",
  "SIGCONT",
  "SIGFPE",
  "SIGHUP",
  "SIGILL",
  "SIGINT",
  "SIGIO",
  "SIGIOT",
  "SIGKILL",
  "SIGPIPE",
  "SIGPOLL",
  "SIGPROF",
  "SIGPWR",
  "SIGQUIT",
  "SIGSEGV",
  "SIGSTKFLT",
  "SIGSTOP",
  "SIGSYS",
  "SIGTERM",
  "SIGTRAP",
  "SIGTSTP",
  "SIGTTIN",
  "SIGTTOU",
  "SIGUNUSED",
  "SIGURG",
  "SIGUSR1",
  "SIGUSR2",
  "SIGVTALRM",
  "SIGWINCH",
  "SIGXCPU",
  "SIGXFSZ",
  "SIGBREAK",
  "SIGLOST",
  "SIGINFO",
] as const;

export const inputResolutionRequestSchema = Type.Object(
  {
    id: Type.String({ minLength: 1 }),
    kind: Type.Literal("input"),
    questions: Type.Array(questionSchema),
  },
  { additionalProperties: false },
);

export type InputResolutionRequestContract = DeepReadonly<Static<typeof inputResolutionRequestSchema>>;

export const approvalResolutionRequestSchema = Type.Object(
  {
    id: Type.String({ minLength: 1 }),
    kind: Type.Literal("approval"),
    gate: approvalGateSchema,
  },
  { additionalProperties: false },
);

export type ApprovalResolutionRequestContract = DeepReadonly<Static<typeof approvalResolutionRequestSchema>>;

export const agentActResolutionRequestSchema = Type.Object(
  {
    id: Type.String({ minLength: 1 }),
    kind: Type.Literal("agent_act"),
    invocation: agentActInvocationSchema,
  },
  { additionalProperties: false },
);

export type AgentActResolutionRequestContract = DeepReadonly<Static<typeof agentActResolutionRequestSchema>>;

export const resolutionRequestSchema = Type.Union(
  [
    inputResolutionRequestSchema,
    approvalResolutionRequestSchema,
    agentActResolutionRequestSchema,
  ],
  {
    $schema: JSON_SCHEMA_DRAFT_2020_12,
    $id: RUNX_CONTROL_SCHEMA_REFS.resolution_request,
  },
);

export type ResolutionRequestContract = DeepReadonly<Static<typeof resolutionRequestSchema>>;

export const resolutionResponseSchema = Type.Object(
  {
    actor: stringEnum(resolutionResponseActors),
    payload: Type.Unknown(),
  },
  {
    $schema: JSON_SCHEMA_DRAFT_2020_12,
    $id: RUNX_CONTROL_SCHEMA_REFS.resolution_response,
    additionalProperties: false,
  },
);

export type ResolutionResponseContract = DeepReadonly<Static<typeof resolutionResponseSchema>>;

export const actReceiptTerminalEnvelopeSchema = Type.Object(
  {
    status: stringEnum(actReceiptTerminalStatuses),
    stdout: Type.String(),
    stderr: Type.String(),
    exitCode: Type.Union([Type.Integer(), Type.Null()]),
    signal: Type.Union([stringEnum(nodeSignalNames), Type.Null()]),
    durationMs: Type.Integer({ minimum: 0 }),
    errorMessage: Type.Optional(Type.String()),
    metadata: Type.Optional(unknownRecordSchema()),
  },
  { additionalProperties: false },
);

export const actReceiptNeedsResolutionEnvelopeSchema = Type.Object(
  {
    status: Type.Literal("needs_resolution"),
    stdout: Type.String(),
    stderr: Type.String(),
    exitCode: Type.Null(),
    signal: Type.Null(),
    durationMs: Type.Integer({ minimum: 0 }),
    request: resolutionRequestSchema,
    errorMessage: Type.Optional(Type.String()),
    metadata: Type.Optional(unknownRecordSchema()),
  },
  { additionalProperties: false },
);

const actReceiptNeedsResolutionUnknownRequestSchema = Type.Object(
  {
    status: Type.Literal("needs_resolution"),
    stdout: Type.String(),
    stderr: Type.String(),
    exitCode: Type.Null(),
    signal: Type.Null(),
    durationMs: Type.Integer({ minimum: 0 }),
    request: Type.Unknown(),
    errorMessage: Type.Optional(Type.String()),
    metadata: Type.Optional(unknownRecordSchema()),
  },
  { additionalProperties: false },
);

export const actReceiptEnvelopeSchema = Type.Union(
  [
    actReceiptTerminalEnvelopeSchema,
    actReceiptNeedsResolutionEnvelopeSchema,
  ],
  {
    $schema: JSON_SCHEMA_DRAFT_2020_12,
    $id: RUNX_CONTROL_SCHEMA_REFS.act_receipt,
  },
);

export type ActReceiptEnvelopeContract = DeepReadonly<Static<typeof actReceiptEnvelopeSchema>>;

export function validateResolutionRequestContract(
  value: unknown,
  label = "resolution_request",
): ResolutionRequestContract {
  const record = asUnknownRecord(value);
  if (record?.kind === "input") {
    return validateContractSchema(inputResolutionRequestSchema, value, label) as ResolutionRequestContract;
  }
  if (record?.kind === "approval") {
    return validateContractSchema(approvalResolutionRequestSchema, value, label) as ResolutionRequestContract;
  }
  if (record?.kind === "agent_act") {
    return validateContractSchema(agentActResolutionRequestSchema, value, label) as ResolutionRequestContract;
  }
  return validateContractSchema(resolutionRequestSchema, value, label);
}

export function validateResolutionResponseContract(
  value: unknown,
  label = "resolution_response",
): ResolutionResponseContract {
  return validateContractSchema(resolutionResponseSchema, value, label);
}

export function validateActReceiptEnvelopeContract(
  value: unknown,
  label = "act_receipt",
): ActReceiptEnvelopeContract {
  const record = asUnknownRecord(value);
  if (record?.status === "success" || record?.status === "failure") {
    return validateContractSchema(actReceiptTerminalEnvelopeSchema, value, label) as ActReceiptEnvelopeContract;
  }
  if (record?.status === "needs_resolution") {
    const result = validateContractSchema(actReceiptNeedsResolutionUnknownRequestSchema, value, label);
    return {
      ...result,
      request: validateResolutionRequestContract(result.request, `${label}.request`),
    } as ActReceiptEnvelopeContract;
  }
  return validateContractSchema(actReceiptEnvelopeSchema, value, label);
}
