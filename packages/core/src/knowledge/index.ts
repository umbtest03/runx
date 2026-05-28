export const knowledgePackage = "@runxhq/core/knowledge";

import { RUNX_CONTRACT_IDS } from "@runxhq/contracts";

// Schema refs for the live local knowledge store + handoff/suppression contracts
// (still owned by @runxhq/contracts). The thread/outbox/feed/story projection
// shared library was retired; if a kernel-side projection capability is needed
// later, it lives in the Rust kernel alongside its execution and contracts.
export const RUNX_SCHEMA_REFS = {
  knowledge_entry: "https://runx.ai/spec@runxhq/core/knowledge-entry.schema.json",
  handoff_signal: RUNX_CONTRACT_IDS.handoffSignal,
  handoff_state: RUNX_CONTRACT_IDS.handoffState,
  suppression_record: RUNX_CONTRACT_IDS.suppressionRecord,
} as const;

export {
  type Actor,
  type EvidenceRef,
  type ThreadAdapterDescriptor,
} from "./internal-validators.js";

export {
  type LocalKnowledgeEntryKind,
  type LocalKnowledgeReceiptEntry,
  type LocalKnowledgeProjectionEntry,
  type LocalKnowledgeAnswerEntry,
  type LocalKnowledgeArtifactEntry,
  type LocalKnowledgeEntry,
  type LocalKnowledge,
  type IndexReceiptOptions,
  type AddProjectionOptions,
  type LocalKnowledgeStore,
  createFileKnowledgeStore,
} from "./local-store.js";
