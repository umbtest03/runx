import fs from "node:fs";
import path from "node:path";

import {
  arrayInput,
  defineTool,
  isRecord,
  resolveInsideRepo,
  resolveRepoRoot,
  stringInput,
} from "@runxhq/authoring";

export default defineTool({
  name: "spec.read_declared_files",
  description: "Read the current contents of files declared in a scafld 2 markdown spec before bounded fix authoring.",
  inputs: {
    spec_contents: stringInput({ description: "Raw scafld 2 markdown spec contents to inspect for declared file paths." }),
    extra_files: arrayInput({ optional: true, description: "Additional repo-relative file targets to read, such as repo_snapshot.recommended_files." }),
    repo_root: stringInput({ optional: true, description: "Repository root used to resolve declared file paths." }),
    fixture: stringInput({ optional: true, description: "Optional fixture workspace root used during dev and harness execution." }),
  },
  output: {
    packet: "runx.spec.declared_file_context.v1",
    wrap_as: "declared_file_context",
  },
  scopes: ["spec.read_declared_files"],
  run: runReadDeclaredFiles,
});

function runReadDeclaredFiles({ inputs, env }) {
  const specContents = inputs.spec_contents;
  const repoRoot = resolveRepoRoot(inputs, env);
  const declared = new Map();

  function rememberPath(relativePath, declaredIn) {
    const normalized = normalizeRepoRelativePath(relativePath);
    if (!normalized) {
      return;
    }
    const current = declared.get(normalized) || new Set();
    current.add(declaredIn);
    declared.set(normalized, current);
  }

  const lines = specContents.split(/\r?\n/);
  let section = "";
  let inFilesImpacted = false;
  let inPhaseChanges = false;

  for (const line of lines) {
    const trimmed = line.trim();

    const sectionMatch = line.match(/^##\s+(.+?)\s*$/);
    if (sectionMatch) {
      section = sectionMatch[1].trim();
      inFilesImpacted = false;
      inPhaseChanges = false;
      continue;
    }

    if (section === "Context" && /^Files impacted:\s*$/i.test(trimmed)) {
      inFilesImpacted = true;
      continue;
    }

    if (/^Phase\s+\d+:/i.test(section) && /^Changes:\s*$/i.test(trimmed)) {
      inPhaseChanges = true;
      continue;
    }

    if (inFilesImpacted) {
      if (trimmed.length === 0) {
        continue;
      }
      if (isContextLabel(trimmed)) {
        inFilesImpacted = false;
        continue;
      }
      const listPath = markdownListPath(line);
      if (listPath) {
        rememberPath(listPath, "context.files_impacted");
      }
      continue;
    }

    if (inPhaseChanges) {
      if (trimmed.length === 0) {
        continue;
      }
      if (/^[A-Z][A-Za-z ]+:\s*$/.test(trimmed)) {
        inPhaseChanges = false;
        continue;
      }
      const listPath = markdownListPath(line);
      if (listPath) {
        rememberPath(listPath, "phase.changes");
      }
    }
  }

  for (const extraPath of extraFilePaths(inputs.extra_files)) {
    rememberPath(extraPath, "input.extra_files");
  }

  const files = [...declared.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([relativePath, declaredIn]) => {
      const resolvedPath = resolveInsideRepo(repoRoot, relativePath);
      const exists = fs.existsSync(resolvedPath);
      return {
        path: relativePath,
        exists,
        kind:
          relativePath.startsWith(".scafld/")
            ? "governance_artifact"
            : "repo_file",
        declared_in: [...declaredIn].sort(),
        contents: exists ? fs.readFileSync(resolvedPath, "utf8") : null,
      };
    });

  return {
    repo_root: repoRoot,
    declared_count: files.length,
    files,
  };
}

function extraFilePaths(value) {
  if (!Array.isArray(value)) {
    return [];
  }

  return value
    .map((entry) => {
      if (typeof entry === "string") {
        return entry;
      }
      if (isRecord(entry) && typeof entry.path === "string") {
        return entry.path;
      }
      return undefined;
    })
    .filter((entry) => typeof entry === "string" && entry.trim().length > 0);
}

function normalizeRepoRelativePath(value) {
  const normalized = stripQuotes(value)
    .replace(/\\/gu, "/")
    .replace(/^\.\/+/u, "");
  if (!normalized || path.isAbsolute(normalized)) {
    return undefined;
  }
  if (normalized.split("/").includes("..")) {
    return undefined;
  }
  return normalized;
}

function stripQuotes(value) {
  const trimmed = String(value || "").trim();
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

function markdownListPath(line) {
  const listMatch = line.match(/^\s*-\s*(.+?)\s*$/);
  if (!listMatch) {
    return undefined;
  }
  const value = listMatch[1].trim();
  if (/^none\.?$/i.test(value)) {
    return undefined;
  }
  const backtickMatch = value.match(/`([^`]+)`/);
  if (backtickMatch) {
    return backtickMatch[1].trim();
  }
  return value
    .replace(/\s+\([^)]+\).*$/u, "")
    .replace(/\s+-\s+.*$/u, "")
    .trim();
}

function isContextLabel(value) {
  return /^(CWD|Packages|Files impacted|Invariants|Related docs):\s*/i.test(value);
}
