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

  for (const relativePath of [...declared.keys()]) {
    for (const relatedPath of relatedTestFilePaths(repoRoot, relativePath)) {
      rememberPath(relatedPath, "related.test");
    }
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

function relatedTestFilePaths(repoRoot, relativePath) {
  const normalized = normalizeRepoRelativePath(relativePath);
  if (!normalized || isTestLikePath(normalized) || !isCodeLikePath(normalized)) {
    return [];
  }

  const candidates = new Set(explicitTestCandidates(normalized));
  for (const candidate of fuzzyTestCandidates(repoRoot, normalized)) {
    candidates.add(candidate);
  }

  return [...candidates]
    .filter((candidate) => fs.existsSync(resolveInsideRepo(repoRoot, candidate)))
    .slice(0, 4);
}

function explicitTestCandidates(relativePath) {
  const extension = path.extname(relativePath);
  const withoutExtension = relativePath.slice(0, -extension.length);
  const candidates = [];

  if (extension === ".rb") {
    const railsAppMatch = withoutExtension.match(/^app\/([^/]+)\/(.+)$/u);
    if (railsAppMatch) {
      const [, appArea, areaPath] = railsAppMatch;
      candidates.push(`spec/${appArea}/${areaPath}_spec.rb`);
      candidates.push(`test/${appArea}/${areaPath}_test.rb`);
    }

    const controllerMatch = withoutExtension.match(/^app\/controllers\/(.+)_controller$/u);
    if (controllerMatch) {
      candidates.push(`spec/requests/${controllerMatch[1]}_spec.rb`);
      candidates.push(`spec/controllers/${controllerMatch[1]}_controller_spec.rb`);
      candidates.push(`test/controllers/${controllerMatch[1]}_controller_test.rb`);
      candidates.push(`test/integration/${controllerMatch[1]}_test.rb`);
    }
  }

  if (/\.[cm]?[jt]sx?$/u.test(relativePath)) {
    const directory = path.dirname(relativePath);
    const basename = path.basename(withoutExtension);
    for (const testExtension of [
      `.test${extension}`,
      `.spec${extension}`,
      ".test.ts",
      ".test.tsx",
      ".spec.ts",
      ".spec.tsx",
      ".test.js",
      ".spec.js",
    ]) {
      candidates.push(path.posix.join(directory, `${basename}${testExtension}`));
    }
    candidates.push(path.posix.join("test", `${basename}.test${extension}`));
    candidates.push(path.posix.join("tests", `${basename}.test${extension}`));
    candidates.push(path.posix.join("__tests__", `${basename}.test${extension}`));
  }

  return candidates;
}

function fuzzyTestCandidates(repoRoot, relativePath) {
  const tokens = path.basename(relativePath, path.extname(relativePath))
    .replace(/_controller$/u, "")
    .split(/[^A-Za-z0-9]+/u)
    .map((token) => token.toLowerCase())
    .filter((token) => token.length >= 4);
  if (tokens.length === 0) {
    return [];
  }

  const testRoots = ["spec", "test", "tests", "__tests__"];
  const matches = [];
  for (const testRoot of testRoots) {
    const absoluteRoot = path.join(repoRoot, testRoot);
    if (!fs.existsSync(absoluteRoot)) {
      continue;
    }
    for (const candidate of walkFiles(absoluteRoot, repoRoot)) {
      if (!isTestLikePath(candidate)) {
        continue;
      }
      const score = fuzzyTestScore(candidate, relativePath, tokens);
      if (score > 0) {
        matches.push({ path: candidate, score });
      }
    }
  }

  return matches
    .sort((left, right) => right.score - left.score || left.path.localeCompare(right.path))
    .map((match) => match.path)
    .slice(0, 4);
}

function fuzzyTestScore(candidate, sourcePath, tokens) {
  const candidateText = candidate.toLowerCase();
  let score = 0;
  for (const token of tokens) {
    if (candidateText.includes(token)) {
      score += 20;
    }
  }
  const sourceSegments = sourcePath.toLowerCase().split("/");
  for (const segment of sourceSegments.slice(1, -1)) {
    if (segment.length >= 3 && candidateText.includes(`/${segment}/`)) {
      score += 8;
    }
  }
  if (sourcePath.includes("/controllers/") && candidate.startsWith("spec/requests/")) {
    score += 15;
  }
  return score;
}

function walkFiles(root, repoRoot) {
  const files = [];
  const stack = [root];
  while (stack.length > 0 && files.length < 500) {
    const current = stack.pop();
    if (!current) {
      continue;
    }
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const absolutePath = path.join(current, entry.name);
      if (entry.isDirectory()) {
        if (!["node_modules", "vendor", "tmp", "log", ".git"].includes(entry.name)) {
          stack.push(absolutePath);
        }
        continue;
      }
      if (entry.isFile()) {
        files.push(path.relative(repoRoot, absolutePath).replace(/\\/gu, "/"));
      }
    }
  }
  return files;
}

function isCodeLikePath(relativePath) {
  return /\.(?:rb|[cm]?[jt]sx?|py|go|rs|java|kt|php|cs)$/u.test(relativePath);
}

function isTestLikePath(relativePath) {
  return /(^|\/)(?:spec|test|tests|__tests__)\//u.test(relativePath)
    || /(?:_spec|_test)\.[^.]+$/u.test(relativePath)
    || /\.(?:spec|test)\.[^.]+$/u.test(relativePath);
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
