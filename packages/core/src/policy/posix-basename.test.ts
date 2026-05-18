import { describe, expect, it } from "vitest";

import { posixBasename } from "./posix-basename.js";

describe("posixBasename", () => {
  it("returns the executable name from POSIX paths", () => {
    expect(posixBasename("/usr/local/bin/node")).toBe("node");
  });

  it("normalizes Windows separators into POSIX semantics", () => {
    expect(posixBasename("C:\\Tools\\node.exe")).toBe("node.exe");
  });

  it("handles mixed separators and trailing slashes deterministically", () => {
    expect(posixBasename("C:\\Tools/bin/bash/")).toBe("bash");
    expect(posixBasename("/")).toBe("");
  });
});
