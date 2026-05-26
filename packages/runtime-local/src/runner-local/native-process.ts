import { spawn } from "node:child_process";

export interface BoundedNativeProcessOptions {
  readonly operation: string;
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly env: NodeJS.ProcessEnv;
  readonly stdin: string;
  readonly timeoutMs: number;
  readonly outputLimitBytes?: number;
}

export interface BoundedNativeProcessResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

const defaultOutputLimitBytes = 1024 * 1024;
const forceKillGraceMs = 1_000;

export function runBoundedNativeProcess(
  options: BoundedNativeProcessOptions,
): Promise<BoundedNativeProcessResult> {
  const outputLimitBytes = options.outputLimitBytes ?? defaultOutputLimitBytes;
  return new Promise((resolve, reject) => {
    const child = spawn(options.command, options.args, {
      cwd: options.cwd,
      env: options.env,
      stdio: ["pipe", "pipe", "pipe"],
    });
    let settled = false;
    let timedOut = false;
    let outputExceeded: Error | undefined;
    let timeoutError: Error | undefined;
    const stdout = new BoundedProcessOutput(outputLimitBytes);
    const stderr = new BoundedProcessOutput(outputLimitBytes);
    let killTimer: NodeJS.Timeout | undefined;

    const clearTimers = () => {
      clearTimeout(timer);
      if (killTimer) {
        clearTimeout(killTimer);
      }
    };

    const failAfterGrace = (error: Error) => {
      if (killTimer) {
        return;
      }
      killTimer = setTimeout(() => {
        child.kill("SIGKILL");
        if (settled) {
          return;
        }
        settled = true;
        clearTimers();
        reject(error);
      }, forceKillGraceMs);
    };

    const terminate = (error: Error) => {
      child.kill("SIGTERM");
      failAfterGrace(error);
    };

    const timer = setTimeout(() => {
      if (settled) {
        return;
      }
      timedOut = true;
      timeoutError = new Error(`${options.operation} timed out after ${options.timeoutMs}ms.`);
      terminate(timeoutError);
    }, options.timeoutMs);

    const handleOutput = (stream: "stdout" | "stderr", output: BoundedProcessOutput, chunk: Buffer) => {
      if (outputExceeded) {
        return;
      }
      if (output.append(chunk)) {
        return;
      }
      outputExceeded = new Error(`${options.operation} ${stream} exceeded ${outputLimitBytes} bytes.`);
      terminate(outputExceeded);
    };

    child.stdout.on("data", (chunk: Buffer) => {
      handleOutput("stdout", stdout, chunk);
    });
    child.stderr.on("data", (chunk: Buffer) => {
      handleOutput("stderr", stderr, chunk);
    });
    child.on("error", (error) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimers();
      reject(new Error(`Failed to spawn ${options.operation} command '${options.command}': ${error.message}`));
    });
    child.on("close", (status) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimers();
      if (outputExceeded) {
        reject(outputExceeded);
        return;
      }
      if (timedOut) {
        reject(timeoutError ?? new Error(`${options.operation} timed out after ${options.timeoutMs}ms.`));
        return;
      }
      resolve({
        status,
        stdout: stdout.toString(),
        stderr: stderr.toString(),
      });
    });
    child.stdin.on("error", () => {
      // The child may exit before consuming stdin. The close handler reports
      // the process status with captured stdout/stderr.
    });
    child.stdin.end(options.stdin);
  });
}

class BoundedProcessOutput {
  private readonly chunks: Buffer[] = [];
  private bytes = 0;

  constructor(private readonly limitBytes: number) {}

  append(chunk: Buffer): boolean {
    if (this.bytes + chunk.length > this.limitBytes) {
      return false;
    }
    this.chunks.push(chunk);
    this.bytes += chunk.length;
    return true;
  }

  toString(): string {
    return Buffer.concat(this.chunks, this.bytes).toString("utf8");
  }
}
