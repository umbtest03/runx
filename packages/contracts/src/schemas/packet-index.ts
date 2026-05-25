import { Type, type Static } from "../internal.js";
import {
  JSON_SCHEMA_DRAFT_2020_12,
  RUNX_CONTRACT_IDS,
  RUNX_LOGICAL_SCHEMAS,
  type DeepReadonly,
  generatedSchema,
} from "../internal.js";

const packetIndexEntrySchema = Type.Object(
  {
    id: Type.String(),
    package: Type.String(),
    version: Type.String(),
    path: Type.String(),
    sha256: Type.String(),
  },
  { additionalProperties: false },
);

export type PacketIndexEntryContract = DeepReadonly<Static<typeof packetIndexEntrySchema>>;

const packetIndexV1TypeSchema = Type.Object(
  {
    schema: Type.Literal(RUNX_LOGICAL_SCHEMAS.packetIndex),
    packets: Type.Array(packetIndexEntrySchema),
  },
  {
    $schema: JSON_SCHEMA_DRAFT_2020_12,
    $id: RUNX_CONTRACT_IDS.packetIndex,
    "x-runx-schema": RUNX_LOGICAL_SCHEMAS.packetIndex,
    additionalProperties: false,
  },
);

export type PacketIndexContract = DeepReadonly<Static<typeof packetIndexV1TypeSchema>>;

export const packetIndexV1Schema = generatedSchema<PacketIndexContract>("packet-index.schema.json");
