import { afterEach, describe, expect, it, vi } from "vitest";

import {
  compactHttpFailure,
  compactPublishSummary,
  hostedSkillMatchesPublishedState,
} from "../../.github/scripts/registry-publish-summary.js";

describe("registry publish log summaries", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("keeps hosted publish output compact and maps unchanged responses explicitly", () => {
    const summary = compactPublishSummary({
      status: "published",
      record: {
        skill_id: "runx/sourcey",
        version: "sha-abc123",
        digest: "digest-1",
        profile_digest: "profile-1",
      },
      harness: {
        status: "passed",
        case_count: 2,
        assertion_error_count: 0,
        assertion_errors: [],
        case_names: ["triage", "review"],
        receipt_ids: ["rx_1"],
      },
      apiBaseUrl: "https://api.runx.ai",
      sourcePath: "/Users/kam/dev/runx/runx/oss/skills/sourcey",
      hostedBody: JSON.stringify({
        status: "success",
        publish: {
          status: "unchanged",
          skill_id: "runx/sourcey",
          version: "sha-abc123",
          digest: "digest-1",
          profile_digest: "profile-1",
        },
        markdown: "full private markdown body",
        profile_document: "full private profile body",
      }),
    });

    expect(summary).toMatchObject({
      status: "already_published",
      skill_id: "runx/sourcey",
      version: "sha-abc123",
      digest: "digest-1",
      profile_digest: "profile-1",
      source_path: "oss/skills/sourcey",
      registry_url: "https://api.runx.ai/v1/skills/runx/sourcey%40sha-abc123",
      harness: {
        status: "passed",
        case_count: 2,
        assertion_error_count: 0,
        case_names: ["triage", "review"],
        receipt_ids: ["rx_1"],
      },
    });
    const rendered = JSON.stringify(summary);
    expect(rendered).not.toContain("full private markdown body");
    expect(rendered).not.toContain("full private profile body");
    expect(rendered).not.toContain("/Users/kam");
  });

  it("does not echo opaque hosted error bodies", () => {
    expect(compactHttpFailure(400, JSON.stringify({
      error: "Registry version already exists with a different digest.",
      markdown: "full private markdown body",
    }))).toBe("HTTP 400: Registry version already exists with a different digest.");

    const opaque = compactHttpFailure(500, "full private markdown body");
    expect(opaque).toMatch(/^HTTP 500: response_body_bytes=\d+$/);
    expect(opaque).not.toContain("full private markdown body");
  });

  it("reads back the exact hosted version when checking duplicate publishes", async () => {
    let requestedUrl = "";
    vi.stubGlobal("fetch", async (url: string) => {
      requestedUrl = url;
      return new Response(JSON.stringify({
        skill: {
          version: "1.0.0",
          digest: "digest-1",
          profile_digest: "profile-1",
        },
      }), {
        status: 200,
        headers: {
          "content-type": "application/json",
        },
      });
    });

    await expect(hostedSkillMatchesPublishedState("https://api.runx.ai", {
      skill_id: "runx/sourcey",
      version: "1.0.0",
      digest: "digest-1",
      profile_digest: "profile-1",
    })).resolves.toBe(true);
    expect(requestedUrl).toBe("https://api.runx.ai/v1/skills/runx/sourcey%401.0.0");
  });
});
