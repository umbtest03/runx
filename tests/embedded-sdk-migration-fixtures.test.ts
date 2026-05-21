import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import {
  validateExternalAdapterHostResolutionFrameContract,
  validateExternalAdapterInvocationContract,
  validateExternalAdapterManifestContract,
  validateExternalAdapterResponseContract,
  validateResolutionRequestContract,
} from "@runxhq/contracts";

const fixtureRoot = new URL("../fixtures/embedded-sdk-migration/", import.meta.url);
const forbiddenTargetImports = ["@runxhq/runtime-local", "@runxhq/adapters"] as const;

describe("embedded SDK migration fixtures", () => {
  it("pins the Rust-supervised service boundary without a runtime-local fallback", () => {
    const fixture = readFixture("runtime-service-boundary.json");
    const target = expectRecord(fixture.target, "target");
    const hostResult = expectRecord(fixture.host_result, "host_result");

    expect(fixture.schema).toBe("runx.embedded_sdk_migration.fixture.v1");
    expect(target.boundary).toBe("runx-runtime-service");
    expect(target.trusted_executor).toBe("runx-runtime");
    expect(target.typescript_role).toBe("client_only");
    expect(target.sdk_disposition).toBe("runx-sdk-cli-backed");
    expect(target.allowed_package_imports).toEqual(["@runxhq/contracts", "@runxhq/host-adapters"]);
    expect(target.forbidden_package_imports).toEqual([...forbiddenTargetImports]);
    expect(target.allowed_package_imports).not.toContain("@runxhq/runtime-local");
    expect(target.allowed_package_imports).not.toContain("@runxhq/adapters");
    expect(fixture.semantics).toEqual([
      "auth_resolution",
      "host_continuation",
      "receipt_production",
      "resume",
      "tool_catalog_resolution",
    ]);

    expect(hostResult.status).toBe("needs_agent");
    const requests = expectArray(hostResult.requests, "host_result.requests");
    expect(validateResolutionRequestContract(requests[0])).toMatchObject({
      id: "req_credentials",
      kind: "input",
    });
  });

  it("pins hosted agent migration to external adapter protocol frames", () => {
    const fixture = readFixture("hosted-agent-external-adapter.json");
    const target = expectRecord(fixture.target, "target");
    const manifest = validateExternalAdapterManifestContract(fixture.manifest);
    const invocation = validateExternalAdapterInvocationContract(fixture.invocation);
    const frame = validateExternalAdapterHostResolutionFrameContract(fixture.host_resolution_frame);
    const response = validateExternalAdapterResponseContract(fixture.response);

    expect(target.boundary).toBe("external-adapter-plugin-protocol");
    expect(target.trusted_supervisor).toBe("runx-runtime");
    expect(target.allowed_package_imports).toEqual(["@runxhq/authoring", "@runxhq/contracts"]);
    expect(target.forbidden_package_imports).toEqual([...forbiddenTargetImports]);
    expect(manifest.supported_source_types).toEqual(["agent", "agent-step"]);
    expect(invocation.source_type).toBe("agent-step");
    expect(invocation.credential_refs?.[0]?.credential_ref.uri).toBe(
      "runx:credential:openai-project:hosted-agent",
    );
    expect(frame.request).toEqual(response.metadata?.external_adapter_host_resolution_request);
    expect(response.status).toBe("host_resolution_requested");
    expect(response.metadata?.external_adapter_host_resolution_frame_id).toBe(frame.frame_id);
    expect(JSON.stringify(fixture)).not.toMatch(/ghp_|sk-|secret_material/);
  });
});

function readFixture(name: string): Record<string, unknown> {
  return JSON.parse(readFileSync(new URL(name, fixtureRoot), "utf8")) as Record<string, unknown>;
}

function expectRecord(value: unknown, label: string): Record<string, unknown> {
  expect(value, `${label} must be an object`).toBeTruthy();
  expect(Array.isArray(value), `${label} must not be an array`).toBe(false);
  expect(typeof value, `${label} must be an object`).toBe("object");
  return value as Record<string, unknown>;
}

function expectArray(value: unknown, label: string): readonly unknown[] {
  expect(Array.isArray(value), `${label} must be an array`).toBe(true);
  return value as readonly unknown[];
}
