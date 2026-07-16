import path from "node:path";

const inputs = JSON.parse(process.env.RUNX_INPUTS_JSON || "{}");
const targetDir = normalizeTarget(inputs.target_dir);
const mode = String(inputs.mode || "build");
const bundle = record(inputs.change_bundle);
const decision = String(bundle.decision || "write");
const rawFiles = Array.isArray(bundle.files) ? bundle.files : [];

if (!["build", "improve", "harness"].includes(mode)) {
  throw new Error("mode must be build, improve, or harness");
}
const allowedDecisions = mode === "build"
  ? ["write", "no_skill"]
  : mode === "improve"
    ? ["write", "no_change"]
    : ["write", "no_change"];
if (!allowedDecisions.includes(decision)) {
  throw new Error(`${mode} decision must be one of: ${allowedDecisions.join(", ")}`);
}
if (decision !== "write") {
  if (rawFiles.length > 0) throw new Error(`${decision} bundles must not contain files`);
} else if (rawFiles.length === 0) {
  throw new Error("a write decision must contain at least one file");
}

const seen = new Set();
const files = rawFiles.map((entry) => {
  const value = record(entry);
  const relative = normalizeFile(value.path);
  const contents = value.contents;
  if (typeof contents !== "string") throw new Error(`${relative} contents must be a string`);
  if (seen.has(relative)) throw new Error(`duplicate file path: ${relative}`);
  seen.add(relative);
  assertAllowed(relative, mode);
  assertNoSecretMaterial(relative, contents);
  return {
    path: path.posix.join(targetDir.split(path.sep).join("/"), relative),
    contents,
  };
});

process.stdout.write(`${JSON.stringify({
  files,
  bundle_manifest: {
    schema: "runx.skill_lab.bundle_manifest.v1",
    mode,
    decision,
    target_dir: targetDir,
    file_count: files.length,
    paths: files.map((file) => file.path),
    summary: typeof bundle.summary === "string" ? bundle.summary : "",
    non_goals: Array.isArray(bundle.non_goals) ? bundle.non_goals.map(String) : [],
  },
}, null, 2)}\n`);

function normalizeTarget(value) {
  const text = typeof value === "string" ? value.trim() : "";
  if (!text || path.isAbsolute(text)) throw new Error("target_dir must be a repo-relative child path");
  const normalized = path.normalize(text);
  if (normalized === "." || normalized === ".." || normalized.startsWith(`..${path.sep}`)) {
    throw new Error("target_dir must stay inside repo_root");
  }
  return normalized;
}

function normalizeFile(value) {
  const text = typeof value === "string" ? value.trim() : "";
  if (!text || path.posix.isAbsolute(text) || text.includes("\\")) {
    throw new Error("bundle paths must be relative POSIX paths");
  }
  const normalized = path.posix.normalize(text);
  if (normalized === "." || normalized === ".." || normalized.startsWith("../")) {
    throw new Error(`bundle path escapes target_dir: ${text}`);
  }
  return normalized;
}

function assertAllowed(relative, mode) {
  const basename = path.posix.basename(relative);
  if (["README.md", "CHANGELOG.md", "INSTALLATION_GUIDE.md", "QUICK_REFERENCE.md"].includes(basename)) {
    throw new Error(`${basename} is auxiliary package bloat; keep operating guidance in SKILL.md`);
  }
  const allowed = relative === "SKILL.md"
    || relative === "X.yaml"
    || relative === ".gitignore"
    || /^(fixtures|scripts|references|assets|agents)\/[A-Za-z0-9._/-]+$/.test(relative)
    || /^[A-Za-z0-9._-]+\.(mjs|js|ts|json|yaml|yml|md)$/.test(relative);
  if (!allowed) throw new Error(`unsupported skill package path: ${relative}`);
  if (mode === "harness" && !/^fixtures\/.+\.ya?ml$/.test(relative)) {
    throw new Error(`harness mode may only write fixtures/*.yaml files: ${relative}`);
  }
}

function assertNoSecretMaterial(relative, contents) {
  const secretPatterns = [
    /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/,
    /\bnskey_(?:live|test)_[A-Za-z0-9]+\b/,
    /\bsk-[A-Za-z0-9_-]{20,}\b/,
  ];
  if (secretPatterns.some((pattern) => pattern.test(contents))) {
    throw new Error(`refusing secret-like material in ${relative}`);
  }
}

function record(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return {};
  return value;
}
