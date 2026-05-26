import { existsSync, mkdtempSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it, vi } from "vitest";

import { invokeMcpTool, listMcpTools } from "./index.js";

describe("runtime-local MCP bridge", () => {
  afterEach(() => {
    vi.doUnmock("../runner-local/process-sandbox.js");
    vi.resetModules();
  });

  it("parses content-length framed responses across chunks", async () => {
    const skillDirectory = mkdtempSync(path.join(os.tmpdir(), "runx-mcp-skill-"));
    try {
      const result = await invokeMcpTool({
        server: {
          command: process.execPath,
          args: ["-e", framedMcpServerScript()],
        },
        skillDirectory,
        tool: "echo",
        args: { message: "hello" },
        timeoutMs: 5_000,
      });

      expect(result).toEqual({
        content: [{ type: "text", text: "echo:hello" }],
      });
    } finally {
      rmSync(skillDirectory, { recursive: true, force: true });
    }
  });

  it("rejects oversized MCP stdout frames", async () => {
    const skillDirectory = mkdtempSync(path.join(os.tmpdir(), "runx-mcp-skill-"));
    try {
      await expect(listMcpTools({
        server: {
          command: process.execPath,
          args: ["-e", oversizedMcpServerScript()],
        },
        skillDirectory,
        timeoutMs: 5_000,
      })).rejects.toThrow("MCP server response exceeded size limit");
    } finally {
      rmSync(skillDirectory, { recursive: true, force: true });
    }
  });

  it("cleans up sandbox resources after the MCP session closes", async () => {
    const skillDirectory = mkdtempSync(path.join(os.tmpdir(), "runx-mcp-skill-"));
    const cleanupDir = mkdtempSync(path.join(os.tmpdir(), "runx-mcp-cleanup-"));
    vi.resetModules();
    vi.doMock("../runner-local/process-sandbox.js", () => ({
      prepareLocalProcessSandbox: () => ({
        status: "allow",
        cwd: skillDirectory,
        env: process.env,
        command: process.execPath,
        args: ["-e", framedMcpServerScript()],
        cleanupPaths: [cleanupDir],
        metadata: { mocked: true },
      }),
      cleanupLocalProcessSandbox: () => {
        rmSync(cleanupDir, { recursive: true, force: true });
        return [];
      },
    }));
    try {
      const { invokeMcpToolWithMetadata } = await import("./index.js");
      const result = await invokeMcpToolWithMetadata({
        server: {
          command: "mocked",
          args: [],
        },
        skillDirectory,
        tool: "echo",
        args: { message: "cleanup" },
        timeoutMs: 5_000,
      });

      expect(result.sandboxMetadata).toEqual({ mocked: true });
      expect(existsSync(cleanupDir)).toBe(false);
    } finally {
      rmSync(skillDirectory, { recursive: true, force: true });
      rmSync(cleanupDir, { recursive: true, force: true });
    }
  });
});

function framedMcpServerScript(): string {
  return String.raw`
let buffer = Buffer.alloc(0);
process.stdin.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  parse();
});
function parse() {
  while (true) {
    const headerEnd = buffer.indexOf("\r\n\r\n");
    if (headerEnd === -1) return;
    const header = buffer.subarray(0, headerEnd).toString("utf8");
    const match = /Content-Length:\s*(\d+)/i.exec(header);
    if (!match) process.exit(2);
    const bodyStart = headerEnd + 4;
    const bodyEnd = bodyStart + Number(match[1]);
    if (buffer.length < bodyEnd) return;
    const message = JSON.parse(buffer.subarray(bodyStart, bodyEnd).toString("utf8"));
    buffer = buffer.subarray(bodyEnd);
    handle(message);
  }
}
function handle(message) {
  if (message.id === undefined) return;
  if (message.method === "initialize") {
    send(message.id, { protocolVersion: "2025-06-18", capabilities: {}, serverInfo: { name: "test", version: "0" } });
    return;
  }
  if (message.method === "tools/list") {
    send(message.id, { tools: [{ name: "echo", description: "Echo" }] });
    return;
  }
  if (message.method === "tools/call") {
    send(message.id, { content: [{ type: "text", text: "echo:" + message.params.arguments.message }] });
  }
}
function send(id, result) {
  const body = JSON.stringify({ jsonrpc: "2.0", id, result });
  const frame = "Content-Length: " + Buffer.byteLength(body, "utf8") + "\r\n\r\n" + body;
  process.stdout.write(frame.slice(0, 11));
  setTimeout(() => process.stdout.write(frame.slice(11)), 0);
}
setInterval(() => {}, 1000);
`;
}

function oversizedMcpServerScript(): string {
  return String.raw`
process.stdin.once("data", () => {
  process.stdout.write("Content-Length: 1048577\r\n\r\n");
});
setInterval(() => {}, 1000);
`;
}
