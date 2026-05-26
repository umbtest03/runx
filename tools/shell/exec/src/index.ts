import { spawn } from "node:child_process";
import path from "node:path";

import { defineTool, failure, numberInput, rawInput, stringInput } from "@runxhq/authoring";

const DEFAULT_TIMEOUT_MS = 30_000;
const MAX_TIMEOUT_MS = 300_000;
const OUTPUT_LIMIT_BYTES = 1024 * 1024;
const FORCE_KILL_GRACE_MS = 100;

export default defineTool({
  name: "shell.exec",
  description: "Execute an explicit command as a high-risk escape hatch.",
  inputs: {
    command: stringInput({ description: "Executable to invoke." }),
    args: rawInput({ optional: true, description: "Optional argument array for the command." }),
    cwd: stringInput({ optional: true, description: "Optional working directory override for the command." }),
    repo_root: stringInput({ optional: true, description: "Optional repository root used when cwd is not supplied." }),
    timeout_ms: numberInput({
      optional: true,
      default: DEFAULT_TIMEOUT_MS,
      description: "Maximum execution time in milliseconds.",
    }),
  },
  output: {
    packet: "runx.shell.execution.v1",
    wrap_as: "shell_execution",
  },
  scopes: ["shell.exec"],
  async run({ inputs, env, cwd: processCwd }) {
    const args = Array.isArray(inputs.args) ? inputs.args.map((value) => String(value)) : [];
    const repoRoot = path.resolve(inputs.repo_root || env.RUNX_CWD || processCwd);
    const cwd = resolveContainedCwd(repoRoot, inputs.cwd);
    const timeoutMs = boundedTimeout(inputs.timeout_ms);
    const result = await runCommand({
      command: inputs.command,
      args,
      cwd,
      timeoutMs,
    });

    if (result.error) {
      throw result.error;
    }

    const output = {
      command: inputs.command,
      args,
      cwd,
      stdout: result.stdout,
      stderr: result.stderr,
      exit_code: result.exitCode ?? 1,
      timed_out: result.timedOut,
      stdout_truncated: result.stdoutTruncated,
      stderr_truncated: result.stderrTruncated,
    };
    return result.exitCode === 0 && !result.timedOut && !result.stdoutTruncated && !result.stderrTruncated
      ? output
      : failure(output, { exitCode: result.exitCode ?? 1, stderr: result.stderr });
  },
});

interface CommandRun {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly timeoutMs: number;
}

interface CommandResult {
  readonly stdout: string;
  readonly stderr: string;
  readonly exitCode: number | null;
  readonly timedOut: boolean;
  readonly stdoutTruncated: boolean;
  readonly stderrTruncated: boolean;
  readonly error?: Error;
}

function resolveContainedCwd(repoRoot: string, requestedCwd: string | undefined): string {
  const resolved = requestedCwd ? path.resolve(repoRoot, requestedCwd) : repoRoot;
  if (resolved === repoRoot || resolved.startsWith(`${repoRoot}${path.sep}`)) {
    return resolved;
  }
  throw new Error(`shell.exec cwd '${resolved}' is outside repo_root '${repoRoot}'`);
}

function boundedTimeout(value: number | undefined): number {
  if (!Number.isFinite(value) || value <= 0) {
    return DEFAULT_TIMEOUT_MS;
  }
  return Math.min(Math.floor(value), MAX_TIMEOUT_MS);
}

function runCommand(run: CommandRun): Promise<CommandResult> {
  return new Promise((resolve) => {
    const child = spawn(run.command, run.args, {
      cwd: run.cwd,
      detached: process.platform !== "win32",
      shell: false,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout = createBoundedCapture();
    const stderr = createBoundedCapture();
    let timedOut = false;
    let settled = false;

    const timeout = setTimeout(() => {
      timedOut = true;
      terminateProcessTree(child.pid);
      setTimeout(() => terminateProcessTree(child.pid, "SIGKILL"), FORCE_KILL_GRACE_MS).unref();
    }, run.timeoutMs);
    timeout.unref();

    child.stdout?.on("data", (chunk) => {
      if (stdout.push(chunk)) {
        terminateProcessTree(child.pid);
      }
    });
    child.stderr?.on("data", (chunk) => {
      if (stderr.push(chunk)) {
        terminateProcessTree(child.pid);
      }
    });
    child.once("error", (error) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timeout);
      resolve({
        stdout: stdout.text(),
        stderr: stderr.text(),
        exitCode: 1,
        timedOut,
        stdoutTruncated: stdout.truncated(),
        stderrTruncated: stderr.truncated(),
        error,
      });
    });
    child.once("close", (exitCode) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timeout);
      resolve({
        stdout: stdout.text(),
        stderr: stderr.text(),
        exitCode,
        timedOut,
        stdoutTruncated: stdout.truncated(),
        stderrTruncated: stderr.truncated(),
      });
    });
  });
}

function createBoundedCapture() {
  const chunks: Buffer[] = [];
  let capturedBytes = 0;
  let truncated = false;
  return {
    push(chunk: Buffer | string): boolean {
      if (truncated) {
        return true;
      }
      const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
      const nextSize = capturedBytes + buffer.byteLength;
      if (nextSize > OUTPUT_LIMIT_BYTES) {
        truncated = true;
        return true;
      }
      chunks.push(buffer);
      capturedBytes = nextSize;
      return false;
    },
    text(): string {
      return Buffer.concat(chunks).toString("utf8");
    },
    truncated(): boolean {
      return truncated;
    },
  };
}

function terminateProcessTree(pid: number | undefined, signal: NodeJS.Signals = "SIGTERM"): void {
  if (pid === undefined) {
    return;
  }
  try {
    if (process.platform === "win32") {
      process.kill(pid, signal);
    } else {
      process.kill(-pid, signal);
    }
  } catch {
    // The process may already have exited.
  }
}
