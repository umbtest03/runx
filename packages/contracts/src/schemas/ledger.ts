import { Type, type Static } from "../internal.js";
import {
  JSON_SCHEMA_DRAFT_2020_12,
  RUNX_CONTRACT_IDS,
  type DeepReadonly,
  generatedSchema,
  validateContractSchema,
} from "../internal.js";
import { artifactEnvelopeSchema } from "./artifact.js";

export const ledgerRecordSchemaVersion = "runx.ledger.entry.v1" as const;
export const ledgerChainSchemaVersion = "runx.ledger.chain.v1" as const;
export const ledgerHashAlgorithm = "sha256" as const;
export const ledgerCanonicalization = "runx.stable-json.v1" as const;

const sha256HexSchema = Type.String({ pattern: "^[a-f0-9]{64}$" });

export const ledgerChainSchema = Type.Object(
  {
    version: Type.Literal(ledgerChainSchemaVersion),
    algorithm: Type.Literal(ledgerHashAlgorithm),
    canonicalization: Type.Literal(ledgerCanonicalization),
    index: Type.Integer({ minimum: 0 }),
    previous_hash: Type.Union([sha256HexSchema, Type.Null()]),
    entry_hash: sha256HexSchema,
  },
  { additionalProperties: false },
);

export type LedgerChainContract = DeepReadonly<Static<typeof ledgerChainSchema>>;

const ledgerRecordTypeSchema = Type.Object(
  {
    schema_version: Type.Literal(ledgerRecordSchemaVersion),
    chain: ledgerChainSchema,
    entry: artifactEnvelopeSchema,
  },
  {
    $schema: JSON_SCHEMA_DRAFT_2020_12,
    $id: RUNX_CONTRACT_IDS.ledgerEntry,
    "x-runx-schema": ledgerRecordSchemaVersion,
    additionalProperties: false,
  },
);

export type LedgerRecordContract = DeepReadonly<Static<typeof ledgerRecordTypeSchema>>;

export const ledgerRecordSchema = generatedSchema<LedgerRecordContract>("ledger-entry.schema.json");

export function validateLedgerRecordContract(
  value: unknown,
  label = "ledger_record",
): LedgerRecordContract {
  return validateContractSchema(ledgerRecordSchema, value, label);
}
