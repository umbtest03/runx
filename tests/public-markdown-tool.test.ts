import { describe, expect, it } from "vitest";

import { sanitizePublicMarkdown } from "../tools/public_markdown.mjs";

describe("public markdown sanitizer tool", () => {
  it("redacts material refs and generic secret-looking values", () => {
    expect(sanitizePublicMarkdown("Status: material_ref=nango:github:conn_1")).toBe("Status: material_ref=[secret]");
    expect(sanitizePublicMarkdown("Blockers: leaked bearer abc123")).toBe("Blockers: leaked bearer [secret]");
    expect(sanitizePublicMarkdown("Next: super-secret-token")).toBe("Next: [secret]");
  });
});
