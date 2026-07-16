import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { invokeNitrosend, redact, validateInvocation } from "./client.mjs";

function response(status, body) {
  return { ok: status >= 200 && status < 300, status, text: async () => body };
}

function mcpResponse(data) {
  return response(200, JSON.stringify({
    jsonrpc: "2.0",
    id: "fixture",
    result: { content: [{ type: "text", text: JSON.stringify({ data }) }] },
  }));
}

test("normalizes a successful provider response without returning credentials", async () => {
  const packet = await invokeNitrosend(
    "read",
    { operation: "status", arguments: {} },
    {
      apiKey: "fixture-key",
      fetchImpl: async (_url, request) => {
        assert.match(request.headers.authorization, /^Bearer /);
        return response(200, JSON.stringify({
          jsonrpc: "2.0",
          id: "fixture",
          result: { content: [{ type: "text", text: JSON.stringify({ data: { status: "ready" } }) }] },
        }));
      },
    },
  );

  assert.equal(packet.decision, "ok");
  assert.equal(packet.result.data.status, "ready");
  assert.equal(JSON.stringify(packet).includes("fixture-key"), false);
});

test("unwraps the Nitrosend response envelope before sealing provider evidence", async () => {
  const packet = await invokeNitrosend(
    "read",
    { operation: "status", arguments: {} },
    {
      apiKey: "fixture-key",
      fetchImpl: async () => response(200, JSON.stringify({
        jsonrpc: "2.0",
        id: "fixture",
        result: {
          content: [{
            type: "text",
            text: JSON.stringify({ result: { status: "ready" }, meta: { tool: "nitro_get_status", current_brand: { id: 1 } } }),
          }],
        },
      })),
    },
  );

  assert.deepEqual(packet.result, { status: "ready" });
  assert.equal(JSON.stringify(packet).includes("current_brand"), false);
});

test("returns a bounded authentication blocker", async () => {
  const packet = await invokeNitrosend(
    "read",
    { operation: "status", arguments: {} },
    { apiKey: "fixture-key", fetchImpl: async () => response(401, "denied") },
  );

  assert.equal(packet.decision, "needs_input");
  assert.deepEqual(packet.blockers, ["Nitrosend rejected the configured credential"]);
  assert.equal(JSON.stringify(packet).includes("denied"), false);
});

test("normalizes JSON-RPC validation errors instead of leaking raw responses", async () => {
  const packet = await invokeNitrosend(
    "read",
    { operation: "insights", arguments: { scope: "campaign", entity_id: 1 } },
    {
      apiKey: "fixture-key",
      fetchImpl: async () => response(200, JSON.stringify({
        jsonrpc: "2.0",
        id: "fixture",
        error: { code: -32602, message: "entity_id is required" },
      })),
    },
  );

  assert.equal(packet.decision, "provider_error");
  assert.deepEqual(packet.blockers, ["entity_id is required"]);
});

test("rejects unsupported operations before network execution", () => {
  assert.equal(validateInvocation("read", { operation: "delete_account" }).decision, "needs_input");
});

test("validates required analytics context before credential resolution", () => {
  const missingScope = validateInvocation("read", { operation: "insights", arguments: {} });
  const missingEntity = validateInvocation("read", { operation: "insights", arguments: { scope: "campaign" } });
  const account = validateInvocation("read", { operation: "insights", arguments: { scope: "account" } });

  assert.equal(missingScope.decision, "needs_input");
  assert.equal(missingEntity.decision, "needs_input");
  assert.equal(account.decision, "ready");
});

test("requires idempotency for real transactional sends", () => {
  const invalid = validateInvocation("act", {
    operation: "send_transactional",
    arguments: { channel: "email", to: "person@example.com", dry_run: false },
  });
  const valid = validateInvocation("act", {
    operation: "send_transactional",
    arguments: { channel: "email", to: "person@example.com", dry_run: false, idempotency_key: "send-1" },
  });

  assert.equal(invalid.decision, "refused");
  assert.equal(valid.decision, "ready");
});

test("bulk import uploads bytes locally and returns only final import evidence", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "nitrosend-import-"));
  const csvPath = path.join(directory, "contacts.csv");
  fs.writeFileSync(csvPath, "email,first_name\nfixture@example.com,Fixture\n");
  let apiCalls = 0;
  let uploadCalls = 0;

  try {
    const packet = await invokeNitrosend(
      "act",
      {
        operation: "import_contacts_file",
        arguments: {
          csv_path: csvPath,
          source_id: "fixture-signup",
          consent_basis: "First-party signup form opt-in",
          dry_run: false,
          idempotency_key: "import-1",
        },
      },
      {
        apiKey: "fixture-key",
        fetchImpl: async (url, request) => {
          if (String(url).startsWith("https://uploads.example.com/")) {
            uploadCalls += 1;
            for await (const _chunk of request.body) {
              // Consume the stream exactly as the provider upload would.
            }
            return response(200, "");
          }
          apiCalls += 1;
          return apiCalls === 1
            ? mcpResponse({
              signed_id: "fixture-signed-id",
              direct_upload: { url: "https://uploads.example.com/contact.csv", headers: { "content-type": "text/csv" } },
            })
            : mcpResponse({ import_id: 42, status: "processing", total_rows: 1 });
        },
      },
    );

    assert.equal(packet.decision, "ok");
    assert.equal(packet.operation, "import_contacts_file");
    assert.equal(packet.result.data.import_id, 42);
    assert.equal(packet.evidence.upload.signed_url_retained, false);
    assert.equal(apiCalls, 2);
    assert.equal(uploadCalls, 1);
    assert.equal(JSON.stringify(packet).includes("fixture-signed-id"), false);
    assert.equal(JSON.stringify(packet).includes("uploads.example.com"), false);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("redacts nested credential-shaped fields and values", () => {
  const credentialShapedValue = ["nskey", "live", "abc"].join("_");
  const value = redact({ authorization: "Bearer secret", nested: { api_key: "secret", note: credentialShapedValue } });
  assert.deepEqual(value, {
    authorization: "[REDACTED]",
    nested: { api_key: "[REDACTED]", note: "[REDACTED]" },
  });
});

test("refuses non-consensual contact sources before provider execution", () => {
  const result = validateInvocation("act", {
    operation: "import_contacts",
    arguments: {
      records: [{ email: "fixture@example.com" }],
      source_id: "broker-list",
      consent_basis: "Purchased from a data broker",
      dry_run: true,
      idempotency_key: "import-1",
    },
  });
  assert.equal(result.decision, "refused");
});
