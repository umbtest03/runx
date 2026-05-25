import { Type, type Static } from "../internal.js";
import {
  JSON_SCHEMA_DRAFT_2020_12,
  RUNX_CONTRACT_IDS,
  RUNX_LOGICAL_SCHEMAS,
  type DeepReadonly,
  dateTimeStringSchema,
  generatedSchema,
  stringEnum,
  unknownRecordSchema,
  validateContractSchema,
} from "../internal.js";
import {
  actFormSchema,
  authorityTermSchema,
  closureSchema,
  criterionBindingSchema,
  criterionStatusSchema,
  decisionSchema,
  authorityAttenuationSchema,
  fanoutReceiptSyncPointSchema,
  intentSchema,
  receiptIssuerSchema,
  receiptSignatureSchema,
  redactionCommitmentAlgorithmSchema,
  referenceSchema,
  revisionDetailsSchema,
  verificationDetailsSchema,
} from "./spine.js";

/**
 * runx.receipt.v1 — the single signed governance receipt.
 *
 * One flat artifact, each top-level key answering one question: integrity
 * (envelope), dedup (idempotency), what ran (subject), what allowed it
 * (authority), the inbound triggers (`signals[]`), the reasoning
 * (`decisions[]`), what was done (`acts[]`), the outcome (seal), and
 * graph/resume lineage. The reasoning and the full acts (intent, success
 * criteria, criterion bindings) are INLINE: the proof and the training signal
 * are the same artifact. Only the bulky per-act execution I/O (the
 * agent-context envelope) is referenced via `acts[].context_ref` +
 * `artifact_refs` and hydrated by projections. The post-run verdict is a
 * `review`/`verification` act in `acts[]` (or a follow-up receipt linked by
 * `lineage`), never a side contract. Verification is computed at read time,
 * never part of the signed body.
 */

/** The canonicalization byte contract this receipt's digest commits under. */
export const RECEIPT_CANONICALIZATION = "runx.receipt.c14n.v1" as const;

const nonEmptyString = () => Type.String({ minLength: 1 });

/** Scoped byte commitment; unifies the old hash_commitments + enforcement.std*_hash. */
export const receiptCommitmentSchema = Type.Object(
  {
    scope: stringEnum(["input", "output", "stdout", "stderr", "error"] as const),
    algorithm: redactionCommitmentAlgorithmSchema,
    value: nonEmptyString(),
    canonicalization: nonEmptyString(),
  },
  { additionalProperties: false },
);

/** Where the run's input came from, a human-readable preview, and a content hash. */
export const receiptInputContextSchema = Type.Object(
  {
    source: nonEmptyString(),
    preview: Type.String(),
    value_hash: nonEmptyString(),
  },
  { additionalProperties: false },
);

export const receiptSubjectSchema = Type.Object(
  {
    kind: stringEnum(["skill", "graph"] as const),
    ref: referenceSchema,
    input_context: Type.Optional(receiptInputContextSchema),
    commitments: Type.Array(receiptCommitmentSchema),
  },
  { additionalProperties: false },
);

/** Enforcement profile is hashed; the granted authority stays readable. */
export const receiptEnforcementSchema = Type.Object(
  {
    profile_hash: nonEmptyString(),
    redaction_refs: Type.Array(referenceSchema),
    setup_refs: Type.Array(referenceSchema),
    teardown_refs: Type.Array(referenceSchema),
  },
  { additionalProperties: false },
);

export const receiptAuthoritySchema = Type.Object(
  {
    actor_ref: referenceSchema,
    grant_refs: Type.Array(referenceSchema),
    scope_refs: Type.Array(referenceSchema),
    authority_proof_refs: Type.Array(referenceSchema),
    attenuation: authorityAttenuationSchema,
    mandate_ref: Type.Optional(referenceSchema),
    terms: Type.Array(authorityTermSchema),
    enforcement: receiptEnforcementSchema,
  },
  { additionalProperties: false },
);

export const receiptIdempotencySchema = Type.Object(
  {
    intent_key: nonEmptyString(),
    trigger_fingerprint: nonEmptyString(),
    content_hash: nonEmptyString(),
  },
  { additionalProperties: false },
);

/** Runner provenance for agent acts (drives the trainable-export projection). */
export const receiptRunnerProvenanceSchema = Type.Object(
  {
    provider: Type.Union([Type.String(), Type.Null()]),
    model: Type.Union([Type.String(), Type.Null()]),
    prompt_version: Type.Union([Type.String(), Type.Null()]),
  },
  { additionalProperties: false },
);

/** Result binding only: criterion_id -> status. The skill declares the criteria. */
export const receiptCriterionSchema = Type.Object(
  {
    criterion_id: nonEmptyString(),
    status: criterionStatusSchema,
    evidence_refs: Type.Array(referenceSchema),
    verification_refs: Type.Array(referenceSchema),
    summary: Type.Optional(nonEmptyString()),
  },
  { additionalProperties: false },
);

/**
 * What was done, rich and inline. The act's semantic core (intent, success
 * criteria, criterion bindings, outcome) stays in the signed body; only the
 * bulky execution I/O (the agent-context envelope: instructions/inputs/output
 * and tool calls) is referenced via `context_ref` and hydrated by projections.
 */
export const receiptActSchema = Type.Object(
  {
    id: nonEmptyString(),
    form: actFormSchema,
    intent: intentSchema,
    summary: nonEmptyString(),
    criterion_bindings: Type.Array(criterionBindingSchema),
    by: Type.Optional(receiptRunnerProvenanceSchema),
    source_refs: Type.Array(referenceSchema),
    target_refs: Type.Array(referenceSchema),
    artifact_refs: Type.Array(referenceSchema),
    // The agent-context envelope (instructions/inputs/output/tool-calls) is
    // referenced here and hydrated by the trainable/inspection projections.
    context_ref: Type.Optional(referenceSchema),
    closure: closureSchema,
    revision: Type.Optional(revisionDetailsSchema),
    verification: Type.Optional(verificationDetailsSchema),
  },
  { additionalProperties: false },
);

/** Exactly one seal. `deferred` expresses a suspended (waiting/delegated) run. */
export const receiptSealSchema = Type.Object(
  {
    disposition: stringEnum([
      "closed",
      "deferred",
      "superseded",
      "declined",
      "blocked",
      "failed",
      "killed",
      "timed_out",
    ] as const),
    reason_code: nonEmptyString(),
    summary: nonEmptyString(),
    closed_at: dateTimeStringSchema(),
    last_observed_at: dateTimeStringSchema(),
    criteria: Type.Array(receiptCriterionSchema),
  },
  { additionalProperties: false },
);

export const receiptLineageSchema = Type.Object(
  {
    parent: Type.Optional(referenceSchema),
    previous: Type.Optional(referenceSchema),
    children: Type.Array(referenceSchema),
    sync: Type.Array(fanoutReceiptSyncPointSchema),
    // Open resolution request when seal.disposition === "deferred".
    resume_ref: Type.Optional(referenceSchema),
  },
  { additionalProperties: false },
);

const receiptV1TypeSchema = Type.Object(
  {
    schema: Type.Literal(RUNX_LOGICAL_SCHEMAS.receipt),
    id: nonEmptyString(),
    created_at: dateTimeStringSchema(),
    canonicalization: nonEmptyString(),
    issuer: receiptIssuerSchema,
    signature: receiptSignatureSchema,
    digest: nonEmptyString(),
    idempotency: receiptIdempotencySchema,
    subject: receiptSubjectSchema,
    authority: receiptAuthoritySchema,
    // Inbound triggers: runx:signal: references; the body lives in the signal.
    signals: Type.Array(referenceSchema),
    // Governance reasoning, inline (admit / escalate / defer / close).
    decisions: Type.Array(decisionSchema),
    acts: Type.Array(receiptActSchema),
    seal: receiptSealSchema,
    lineage: Type.Optional(receiptLineageSchema),
    // Runtime-local read aid (history projection labels); never part of the
    // signed body. The canonicalizer strips it before the digest.
    metadata: Type.Optional(unknownRecordSchema()),
  },
  {
    $schema: JSON_SCHEMA_DRAFT_2020_12,
    $id: RUNX_CONTRACT_IDS.receipt,
    "x-runx-schema": RUNX_LOGICAL_SCHEMAS.receipt,
    additionalProperties: false,
  },
);

export type ReceiptContract = DeepReadonly<Static<typeof receiptV1TypeSchema>>;

export const receiptV1Schema = generatedSchema<ReceiptContract>("receipt.schema.json");

export function validateReceiptContract(value: unknown, label = "receipt"): ReceiptContract {
  return validateContractSchema(receiptV1Schema, value, label);
}
