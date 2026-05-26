import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdir, mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import {
  searchRegistry,
  type RegistrySkillVersion,
  type RegistryStore,
} from "@runxhq/core/registry";
import { firstNonEmpty, isRecord, recordField } from "@runxhq/core/util";
import { parse as parseYaml } from "yaml";

export interface IdeActionCoreOptions {
  readonly command?: string;
  readonly cwd?: string;
  readonly env?: NodeJS.ProcessEnv;
  readonly receiptDir?: string;
  readonly registryStore?: RegistryStore;
}

export interface RunSkillOptions {
  readonly skillPath: string;
  readonly inputs?: Readonly<Record<string, unknown>>;
}

export interface HistoryOptions {
  readonly query?: string;
}

export interface SearchSkillsOptions {
  readonly query: string;
  readonly limit?: number;
}

export interface AddSkillOptions {
  readonly ref: string;
  readonly to?: string;
}

export interface HarnessRunResult {
  readonly assertionErrors: readonly string[];
  readonly receipt: ReceiptLike;
  readonly inputs?: Readonly<Record<string, unknown>>;
}

export interface ReceiptLike {
  readonly id?: string;
  readonly schema?: string;
  readonly seal?: {
    readonly digest?: string;
    readonly disposition?: string;
    readonly reason_code?: string;
    readonly [key: string]: unknown;
  };
  readonly [key: string]: unknown;
}

export interface IdeActionResult<T = unknown> {
  readonly action: string;
  readonly status: "success" | "needs_agent" | "policy_denied" | "failure" | "error";
  readonly data?: T;
  readonly resolutions: readonly unknown[];
  readonly events: readonly unknown[];
  readonly error?: string;
}

export interface IdeActionCore {
  readonly runSkill: (options: RunSkillOptions) => Promise<IdeActionResult>;
  readonly inspectReceipt: (receiptId: string, options?: { readonly receiptDir?: string }) => Promise<IdeActionResult>;
  readonly history: (options?: HistoryOptions) => Promise<IdeActionResult>;
  readonly searchSkills: (options: SearchSkillsOptions) => Promise<IdeActionResult>;
  readonly addSkill: (options: AddSkillOptions) => Promise<IdeActionResult>;
  readonly harnessRun: (fixturePath: string) => Promise<IdeActionResult<HarnessRunResult>>;
}

export function createIdeActionCore(options: IdeActionCoreOptions = {}): IdeActionCore {
  const env = options.env ?? process.env;
  const receiptDir = options.receiptDir ?? path.join(env.RUNX_HOME ?? path.join(os.homedir(), ".runx"), "receipts");
  return {
    runSkill: async (runOptions) => await wrapAction("runx.skill.run", async () =>
      await runSkillViaHarness(runOptions, { ...options, env, receiptDir }),
    ),
    inspectReceipt: async (receiptId, inspectOptions = {}) =>
      await wrapAction("runx.receipt.inspect", async () => await readReceipt(receiptId, inspectOptions.receiptDir ?? receiptDir)),
    history: async (historyOptions = {}) =>
      await wrapAction("runx.history", async () => await readHistory(receiptDir, historyOptions.query)),
    searchSkills: async (searchOptions) =>
      await wrapAction("runx.skill.search", async () => await searchSkills(options.registryStore, searchOptions)),
    addSkill: async (addOptions) =>
      await wrapAction("runx.skill.add", async () => await addSkill(options.registryStore, addOptions)),
    harnessRun: async (fixturePath) =>
      await wrapAction("runx.harness.run", async () => await runHarness(fixturePath, { ...options, env, receiptDir })),
  };
}

async function runSkillViaHarness(
  options: RunSkillOptions,
  context: IdeActionCoreOptions & { readonly env: NodeJS.ProcessEnv; readonly receiptDir: string },
): Promise<unknown> {
  const missingInputs = await missingRequiredInputs(options.skillPath, options.inputs ?? {});
  if (missingInputs.length > 0) {
    return {
      status: "needs_agent",
      requests: [
        {
          kind: "input",
          questions: missingInputs.map((input) => ({
            id: input.id,
            type: input.type,
            required: true,
            prompt: input.description ?? input.id,
          })),
        },
      ],
    };
  }

  const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-ide-skill-"));
  try {
    const fixturePath = path.join(tempDir, "skill-harness.json");
    await writeFile(
      fixturePath,
      `${JSON.stringify({
        name: `ide-${path.basename(options.skillPath)}`,
        kind: "skill",
        target: path.resolve(context.cwd ?? context.env.RUNX_CWD ?? process.cwd(), options.skillPath),
        inputs: options.inputs ?? {},
      }, null, 2)}\n`,
      "utf8",
    );
    return {
      ...await runHarness(fixturePath, context),
      inputs: options.inputs ?? {},
    };
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
}

async function runHarness(
  fixturePath: string,
  context: IdeActionCoreOptions & { readonly env: NodeJS.ProcessEnv; readonly receiptDir: string },
): Promise<HarnessRunResult> {
  const normalized = await normalizeHarnessFixture(fixturePath);
  try {
    const result = await runNativeRunx(["harness", normalized.path, "--json"], context);
    if (result.status !== 0) {
      throw new Error(firstNonEmpty(result.stderr, result.stdout, `runx harness exited with ${result.status ?? 1}.`));
    }
    const receipt = receiptFromJson(parseJson(result.stdout));
    await persistReceipt(receipt, context.receiptDir);
    return {
      assertionErrors: [],
      receipt,
    };
  } finally {
    if (normalized.cleanup) {
      await rm(normalized.path, { force: true });
    }
  }
}

async function normalizeHarnessFixture(fixturePath: string): Promise<{ readonly path: string; readonly cleanup: boolean }> {
  const raw = await readFile(fixturePath, "utf8");
  const parsed = parseYaml(raw);
  if (!isRecord(parsed)) {
    return { path: fixturePath, cleanup: false };
  }
  let changed = false;
  const fixture = { ...parsed };
  const expect = recordField(fixture, "expect");
  if (expect?.status === "success") {
    fixture.expect = { ...expect, status: "sealed" };
    changed = true;
  }
  if (typeof fixture.target === "string" && !path.isAbsolute(fixture.target)) {
    fixture.target = path.resolve(path.dirname(fixturePath), fixture.target);
    changed = true;
  }
  if (!changed) {
    return { path: fixturePath, cleanup: false };
  }
  const normalizedPath = path.join(os.tmpdir(), `runx-ide-harness-${process.pid}-${Date.now()}.json`);
  await writeFile(normalizedPath, `${JSON.stringify(fixture, null, 2)}\n`, "utf8");
  return { path: normalizedPath, cleanup: true };
}

async function missingRequiredInputs(
  skillPath: string,
  inputs: Readonly<Record<string, unknown>>,
): Promise<readonly RequiredInput[]> {
  const manifestPath = path.join(skillPath, "SKILL.md");
  if (!existsSync(manifestPath)) {
    return [];
  }
  const markdown = await readFile(manifestPath, "utf8");
  const frontmatter = parseFrontmatter(markdown);
  const required: RequiredInput[] = [];
  const declaredInputs = recordField(frontmatter, "inputs");
  for (const [id, value] of Object.entries(declaredInputs ?? {})) {
    if (!isRecord(value) || value.required !== true || inputs[id] !== undefined) {
      continue;
    }
    required.push({
      id,
      type: typeof value.type === "string" ? value.type : "string",
      description: typeof value.description === "string" ? value.description : undefined,
    });
  }
  return required;
}

interface RequiredInput {
  readonly id: string;
  readonly type: string;
  readonly description?: string;
}

async function readReceipt(receiptId: string, receiptDir: string): Promise<unknown> {
  return parseJson(await readFile(path.join(receiptDir, `${receiptId}.json`), "utf8"));
}

async function readHistory(receiptDir: string, query: string | undefined): Promise<unknown> {
  const entries = await readdir(receiptDir).catch(() => []);
  const receipts = [];
  for (const entry of entries.sort()) {
    if (!entry.endsWith(".json")) {
      continue;
    }
    const receipt = parseJson(await readFile(path.join(receiptDir, entry), "utf8"));
    if (!query || JSON.stringify(receipt).includes(query)) {
      receipts.push(receipt);
    }
  }
  return { receipts };
}

async function searchSkills(
  store: RegistryStore | undefined,
  options: SearchSkillsOptions,
): Promise<unknown> {
  if (!store) {
    return [];
  }
  return await searchRegistry(store, options.query, { limit: options.limit });
}

async function addSkill(store: RegistryStore | undefined, options: AddSkillOptions): Promise<unknown> {
  if (!store) {
    throw new Error("IDE skill add requires a registry store.");
  }
  const parsed = parseSkillRef(options.ref);
  const version = await store.getVersion(parsed.skillId, parsed.version);
  if (!version) {
    throw new Error(`Registry skill ${options.ref} was not found.`);
  }
  const installRoot = options.to ?? path.join(os.homedir(), ".runx", "skills");
  const installDir = path.join(installRoot, version.name);
  await mkdir(installDir, { recursive: true });
  await writeFile(path.join(installDir, "SKILL.md"), version.markdown, "utf8");
  if (version.profile_document) {
    await writeFile(path.join(installDir, "X.yaml"), version.profile_document, "utf8");
  }
  return {
    skill_id: version.skill_id,
    version: version.version,
    path: installDir,
  };
}

function parseSkillRef(ref: string): { readonly skillId: string; readonly version?: string } {
  const [skillId, version] = ref.split("@", 2);
  return { skillId, version };
}

async function wrapAction<T>(
  action: string,
  run: () => Promise<T>,
): Promise<IdeActionResult<T>> {
  try {
    const data = await run();
    return {
      action,
      status: normalizeStatus(isRecord(data) && typeof data.status === "string" ? data.status : undefined),
      data,
      resolutions: [],
      events: [],
    };
  } catch (error) {
    return {
      action,
      status: "error",
      resolutions: [],
      events: [],
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

function normalizeStatus(status: string | undefined): IdeActionResult["status"] {
  if (status === "success" || status === "needs_agent" || status === "policy_denied" || status === "failure") {
    return status;
  }
  return "success";
}

async function persistReceipt(receipt: unknown, receiptDir: string): Promise<void> {
  if (!isRecord(receipt) || typeof receipt.id !== "string") {
    return;
  }
  await mkdir(receiptDir, { recursive: true });
  await writeFile(path.join(receiptDir, `${receipt.id}.json`), `${JSON.stringify(receipt, null, 2)}\n`, "utf8");
}

interface NativeRunxResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

function runNativeRunx(
  args: readonly string[],
  context: IdeActionCoreOptions & { readonly env: NodeJS.ProcessEnv; readonly receiptDir: string },
): Promise<NativeRunxResult> {
  const command = context.command ?? resolveNativeRunxBinary(context.env);
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: context.cwd ?? context.env.RUNX_CWD ?? process.cwd(),
      env: {
        ...process.env,
        ...context.env,
        NO_COLOR: "1",
        RUNX_RECEIPT_DIR: context.receiptDir,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk: string) => {
      stderr += chunk;
    });
    child.on("error", reject);
    child.on("close", (status) => resolve({ status, stdout, stderr }));
  });
}

function resolveNativeRunxBinary(env: NodeJS.ProcessEnv): string {
  for (const candidate of [
    env.RUNX_RUST_CLI_BIN,
    env.RUNX_BIN,
    path.join(process.cwd(), "crates", "target", "debug", "runx"),
    path.join(process.cwd(), "crates", "target", "release", "runx"),
  ]) {
    if (candidate && existsSync(candidate)) {
      return candidate;
    }
  }
  return "runx";
}

function parseFrontmatter(markdown: string): Readonly<Record<string, unknown>> {
  const match = /^---\n([\s\S]*?)\n---/.exec(markdown);
  if (!match) {
    return {};
  }
  const parsed = parseYaml(match[1]);
  return isRecord(parsed) ? parsed : {};
}

function parseJson(value: string): unknown {
  return JSON.parse(value);
}

function receiptFromJson(value: unknown): ReceiptLike {
  return isRecord(value) ? value : {};
}
