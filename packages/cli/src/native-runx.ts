import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import process from "node:process";

import { errorMessage, firstNonEmpty, parsePositiveInt } from "@runxhq/core/util";

const DEFAULT_NATIVE_RUNX_TIMEOUT_MS = 300_000;
const DEFAULT_NATIVE_RUNX_OUTPUT_LIMIT_BYTES = 1_048_576;

export interface NativeRunxProcessResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

export interface NativeRunxOptions {
  readonly env: NodeJS.ProcessEnv;
  readonly cwd?: string;
  readonly timeoutMs?: number;
}

export async function runNativeRunxJson(
  args: readonly string[],
  options: NativeRunxOptions,
): Promise<unknown> {
  const result = await runNativeRunx(args, options);
  if (result.status !== 0) {
    throw nativeRunxError(args, result);
  }
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`native runx ${args.join(" ")} returned invalid JSON: ${errorMessage(error)}`);
  }
}

export async function runNativeRunx(
  args: readonly string[],
  options: NativeRunxOptions,
): Promise<NativeRunxProcessResult> {
  const timeoutMs = options.timeoutMs
    ?? parsePositiveInt(options.env.RUNX_RUST_CLI_TIMEOUT_MS)
    ?? DEFAULT_NATIVE_RUNX_TIMEOUT_MS;
  const maxOutputBytes = parsePositiveInt(options.env.RUNX_RUST_CLI_OUTPUT_LIMIT_BYTES)
    ?? DEFAULT_NATIVE_RUNX_OUTPUT_LIMIT_BYTES;
  return await spawnNativeRunx({
    command: resolveNativeRunxBinary(options.env),
    args,
    cwd: options.cwd ?? options.env.RUNX_CWD ?? process.cwd(),
    env: {
      ...process.env,
      ...options.env,
      NO_COLOR: "1",
      RUNX_RUST_CLI: "1",
    },
    timeoutMs,
    maxOutputBytes,
  });
}

export function resolveNativeRunxBinary(env: NodeJS.ProcessEnv): string {
  const override = env.RUNX_DEV_RUST_CLI_BIN;
  if (override) {
    if (!path.isAbsolute(override)) {
      throw new Error(`RUNX_DEV_RUST_CLI_BIN must be an absolute path: ${override}`);
    }
    if (existsSync(override)) {
      return override;
    }
    throw new Error(`RUNX_DEV_RUST_CLI_BIN does not exist: ${override}`);
  }
  return "runx";
}

interface SpawnNativeRunxOptions {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly env: NodeJS.ProcessEnv;
  readonly timeoutMs: number;
  readonly maxOutputBytes: number;
}

export function spawnNativeRunx(options: SpawnNativeRunxOptions): Promise<NativeRunxProcessResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(options.command, options.args, {
      cwd: options.cwd,
      env: options.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let settled = false;
    let timedOut = false;
    let stdout = "";
    let stderr = "";
    let stdoutBytes = 0;
    let stderrBytes = 0;
    let outputLimitExceeded: string | undefined;
    let killTimer: NodeJS.Timeout | undefined;

    const terminate = () => {
      child.kill("SIGTERM");
      killTimer = setTimeout(() => {
        if (settled) return;
        settled = true;
        child.kill("SIGKILL");
        reject(new Error(outputLimitExceeded ?? `native runx ${options.args.join(" ")} timed out after ${options.timeoutMs}ms.`));
      }, 1_000);
    };

    const timer = setTimeout(() => {
      if (settled) return;
      timedOut = true;
      terminate();
    }, options.timeoutMs);

    const clearTimers = () => {
      clearTimeout(timer);
      if (killTimer) clearTimeout(killTimer);
    };

    const appendOutput = (stream: "stdout" | "stderr", chunk: string) => {
      if (outputLimitExceeded) return;
      const chunkBytes = Buffer.byteLength(chunk, "utf8");
      if (stream === "stdout") {
        stdoutBytes += chunkBytes;
        if (stdoutBytes <= options.maxOutputBytes) {
          stdout += chunk;
          return;
        }
      } else {
        stderrBytes += chunkBytes;
        if (stderrBytes <= options.maxOutputBytes) {
          stderr += chunk;
          return;
        }
      }
      outputLimitExceeded = `native runx ${stream} exceeded ${options.maxOutputBytes} bytes.`;
      terminate();
    };

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      appendOutput("stdout", chunk);
    });
    child.stderr.on("data", (chunk: string) => {
      appendOutput("stderr", chunk);
    });
    child.on("error", (error) => {
      if (settled) return;
      settled = true;
      clearTimers();
      reject(new Error(`failed to spawn native runx '${options.command}': ${error.message}`));
    });
    child.on("close", (status) => {
      if (settled) return;
      settled = true;
      clearTimers();
      if (outputLimitExceeded) {
        reject(new Error(outputLimitExceeded));
        return;
      }
      if (timedOut) {
        reject(new Error(`native runx ${options.args.join(" ")} timed out after ${options.timeoutMs}ms.`));
        return;
      }
      resolve({ status, stdout, stderr });
    });
  });
}

function nativeRunxError(args: readonly string[], result: NativeRunxProcessResult): Error {
  return new Error(
    `native runx ${args.join(" ")} failed with exit ${result.status}: ${firstNonEmpty(result.stderr, result.stdout, "no output")}`,
  );
}
