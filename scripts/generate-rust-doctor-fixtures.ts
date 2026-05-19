import { existsSync } from "node:fs";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { Writable } from "node:stream";
import { fileURLToPath } from "node:url";

import { runCli } from "../packages/cli/src/index.js";

const workspaceRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const fixtureRoot = path.join(workspaceRoot, "fixtures", "doctor");
const check = process.argv.includes("--check");

interface DoctorFixtureCase {
  readonly name: string;
  readonly expectedExitCode: number;
  readonly files: readonly FixtureFile[];
}

interface FixtureFile {
  readonly path: string;
  readonly contents: string;
}

const cases: readonly DoctorFixtureCase[] = [
  {
    name: "empty-success",
    expectedExitCode: 0,
    files: [],
  },
  {
    name: "removed-tool-yaml",
    expectedExitCode: 1,
    files: [
      file("tools/demo/removed/tool.yaml", `name: demo.removed
description: Removed tool fixture.
source:
  type: cli-tool
  command: node
  args:
    - ./run.mjs
`),
    ],
  },
  {
    name: "tool-fixture-missing",
    expectedExitCode: 1,
    files: [
      file("tools/demo/echo/manifest.json", `${JSON.stringify({
        name: "demo.echo",
        description: "Echo fixture.",
        source: {
          type: "cli-tool",
          command: "node",
          args: ["./run.mjs"],
        },
        inputs: {},
        scopes: [],
      }, null, 2)}\n`),
    ],
  },
  {
    name: "skill-fixture-missing",
    expectedExitCode: 1,
    files: [
      file("skills/uncovered/X.yaml", `skill: uncovered
runners:
  default:
    default: true
    type: cli-tool
    command: node
    args:
      - -e
      - "process.stdout.write('{}')"
`),
    ],
  },
  {
    name: "file-budget-exceeded",
    expectedExitCode: 1,
    files: [
      file(
        "packages/cli/src/index.ts",
        `${Array.from({ length: 3001 }, (_, index) => `line_${index}`).join("\n")}\n`,
      ),
    ],
  },
  {
    name: "cross-package-reach-in",
    expectedExitCode: 1,
    files: [
      file("packages/cli/src/index.ts", `import "../../core/src/index.js";\n`),
      file("packages/core/src/index.ts", "export const core = true;\n"),
    ],
  },
];

const expectedFiles = new Set<string>();

class MemoryWritable extends Writable {
  private readonly chunks: string[] = [];

  override _write(
    chunk: Buffer | string,
    _encoding: BufferEncoding,
    callback: (error?: Error | null) => void,
  ): void {
    this.chunks.push(Buffer.isBuffer(chunk) ? chunk.toString("utf8") : chunk);
    callback();
  }

  contents(): string {
    return this.chunks.join("");
  }
}

for (const fixtureCase of cases) {
  await writeWorkspace(fixtureCase);
  const report = await runDoctorFixture(fixtureCase);
  await writeOrCheck(
    path.join(fixtureRoot, fixtureCase.name, "expected.json"),
    `${JSON.stringify(report, null, 2)}\n`,
  );
}

if (check) {
  await checkNoStaleFiles();
}

console.log(`${check ? "checked" : "generated"} ${cases.length} doctor fixtures`);

function file(filePath: string, contents: string): FixtureFile {
  return { path: filePath, contents };
}

async function writeWorkspace(fixtureCase: DoctorFixtureCase): Promise<void> {
  for (const fixtureFile of fixtureCase.files) {
    await writeOrCheck(
      path.join(fixtureRoot, fixtureCase.name, "workspace", fixtureFile.path),
      fixtureFile.contents,
    );
  }
}

async function runDoctorFixture(fixtureCase: DoctorFixtureCase): Promise<unknown> {
  const workspacePath = path.join(fixtureRoot, fixtureCase.name, "workspace");
  if (!existsSync(workspacePath)) {
    if (check) {
      throw new Error(
        `fixture workspace is missing: ${path.relative(workspaceRoot, workspacePath)}`,
      );
    }
    await mkdir(workspacePath, { recursive: true });
  }
  const stdout = new MemoryWritable();
  const stderr = new MemoryWritable();
  const exitCode = await runCli(
    ["doctor", "--json"],
    { stdin: process.stdin, stdout: stdout as never, stderr: stderr as never },
    { ...process.env, RUNX_CWD: workspacePath },
  );
  if (exitCode !== fixtureCase.expectedExitCode) {
    throw new Error(
      `${fixtureCase.name}: expected exit ${fixtureCase.expectedExitCode}, got ${exitCode}`,
    );
  }
  if (stderr.contents() !== "") {
    throw new Error(`${fixtureCase.name}: expected empty stderr, got ${JSON.stringify(stderr.contents())}`);
  }
  return JSON.parse(stdout.contents());
}

async function writeOrCheck(filePath: string, contents: string): Promise<void> {
  expectedFiles.add(filePath);
  if (check) {
    const existing = await readFile(filePath, "utf8");
    if (existing !== contents) {
      throw new Error(`fixture is stale: ${path.relative(workspaceRoot, filePath)}`);
    }
    return;
  }
  await mkdir(path.dirname(filePath), { recursive: true });
  await writeFile(filePath, contents);
}

async function checkNoStaleFiles(): Promise<void> {
  if (!existsSync(fixtureRoot)) {
    throw new Error("doctor fixture root is missing");
  }
  for (const filePath of await collectFiles(fixtureRoot)) {
    if (!expectedFiles.has(filePath)) {
      throw new Error(`stale fixture file: ${path.relative(workspaceRoot, filePath)}`);
    }
  }
}

async function collectFiles(directory: string): Promise<readonly string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...await collectFiles(entryPath));
    } else if (entry.isFile()) {
      files.push(entryPath);
    }
  }
  return files.sort();
}
