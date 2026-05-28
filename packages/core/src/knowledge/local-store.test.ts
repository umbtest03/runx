import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { createFileKnowledgeStore } from "./index.js";

describe("file local knowledge store", () => {
  it("initializes an idempotent filesystem index and stores project-scoped projections", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-knowledge-"));
    const knowledgeDir = path.join(tempDir, "knowledge");

    try {
      const store = createFileKnowledgeStore(knowledgeDir);
      await expect(store.init()).resolves.toMatchObject({
        schema_version: "runx.knowledge.v1",
        entries: [],
      });
      await expect(store.init()).resolves.toMatchObject({
        schema_version: "runx.knowledge.v1",
      });

      const receipt = {
        id: "rx_knowledge_1",
        kind: `skill_${"execution"}` as const,
        status: "sealed" as const,
        [`skill_${"name"}`]: "echo",
        source_type: "cli-tool",
        started_at: "2026-04-10T00:00:00Z",
        completed_at: "2026-04-10T00:00:01Z",
      };

      const project = path.join(tempDir, "project");
      await store.indexReceipt({
        receipt,
        receiptFile: path.join(tempDir, "receipts", `${receipt.id}.json`),
        project,
        indexedAt: "2026-04-10T00:00:02Z",
      });
      await store.addProjection({
        project,
        scope: "project",
        key: "homepage_url",
        value: "https://example.test",
        source: "test",
        confidence: 0.9,
        freshness: "fresh",
        receiptId: receipt.id,
        createdAt: "2026-04-10T00:00:03Z",
      });

      await expect(store.listReceipts({ project })).resolves.toEqual([
        expect.objectContaining({
          receipt_id: receipt.id,
          receipt_ref: {
            type: "receipt",
            uri: `runx:receipt:${receipt.id}`,
            label: "echo",
          },
          execution_name: "echo",
          source_type: "cli-tool",
          indexed_at: "2026-04-10T00:00:02Z",
        }),
      ]);
      await expect(store.listProjections({ project })).resolves.toEqual([
        expect.objectContaining({
          key: "homepage_url",
          value: "https://example.test",
          receipt_id: receipt.id,
        }),
      ]);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("preserves concurrent projection writes through the filesystem index", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-knowledge-concurrent-"));
    const knowledgeDir = path.join(tempDir, "knowledge");
    const project = path.join(tempDir, "project");

    try {
      const store = createFileKnowledgeStore(knowledgeDir);
      await store.init();

      await Promise.all(
        Array.from({ length: 20 }, async (_, index) =>
          createFileKnowledgeStore(knowledgeDir).addProjection({
            project,
            scope: "project",
            key: `projection_${index}`,
            value: index,
            source: "concurrency-test",
            confidence: 1,
            freshness: "fresh",
            createdAt: `2026-04-10T00:00:${String(index).padStart(2, "0")}Z`,
          }),
        ),
      );

      const projections = await store.listProjections({ project });
      expect(projections).toHaveLength(20);
      expect(projections.map((projection) => projection.key).sort()).toEqual(
        Array.from({ length: 20 }, (_, index) => `projection_${index}`).sort(),
      );
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("skips malformed stored index entries instead of throwing", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-knowledge-malformed-"));
    const knowledgeDir = path.join(tempDir, "knowledge");
    const project = path.join(tempDir, "project");

    try {
      await mkdir(knowledgeDir, { recursive: true });
      await writeFile(
        path.join(knowledgeDir, "index.json"),
        `${JSON.stringify({
          schema_version: "runx.knowledge.v1",
          entries: [
            {
              entry_id: "receipt_rx_valid",
              entry_kind: "receipt",
              receipt_id: "rx_valid",
              receipt_ref: {
                type: "receipt",
                uri: "runx:receipt:rx_valid",
                label: "echo",
              },
              status: "sealed",
              execution_name: "echo",
              indexed_at: "2026-04-10T00:00:00Z",
              project,
            },
            { receipt_id: 123, indexed_at: 1 },
            {
              entry_id: "projection_valid",
              entry_kind: "projection",
              project,
              scope: "project",
              key: "homepage_url",
              value: "https://example.test",
              source: "test",
              confidence: 0.9,
              freshness: "fresh",
              created_at: "2026-04-10T00:00:01Z",
            },
            { id: "projection_bad", key: 42 },
          ],
        }, null, 2)}\n`,
      );

      const warnings: string[] = [];
      const warn = console.warn;
      console.warn = (message?: unknown) => {
        warnings.push(String(message ?? ""));
      };

      try {
        const store = createFileKnowledgeStore(knowledgeDir);
        const knowledge = await store.read();
        expect(knowledge.entries.filter((entry) => entry.entry_kind === "receipt")).toEqual([
          expect.objectContaining({
            receipt_id: "rx_valid",
            receipt_ref: {
              type: "receipt",
              uri: "runx:receipt:rx_valid",
              label: "echo",
            },
            execution_name: "echo",
          }),
        ]);
        expect(knowledge.entries.filter((entry) => entry.entry_kind === "projection")).toEqual([
          expect.objectContaining({
            entry_id: "projection_valid",
            key: "homepage_url",
          }),
        ]);
      } finally {
        console.warn = warn;
      }

      expect(warnings).toHaveLength(2);
      expect(warnings[0]).toContain("malformed local knowledge entry");
      expect(warnings[1]).toContain("malformed local knowledge entry");
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });
});
