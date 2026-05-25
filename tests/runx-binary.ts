import { existsSync } from "node:fs";
import path from "node:path";

const workspaceRoot = process.cwd();
const defaultRunxBinary = path.join(
  workspaceRoot,
  "crates",
  "target",
  "debug",
  process.platform === "win32" ? "runx.exe" : "runx",
);

export function resolveRunxBinary(env: NodeJS.ProcessEnv = process.env): string {
  const configured = firstNonEmpty(
    env.RUNX_RUST_CLI_BIN,
    env.RUNX_KERNEL_EVAL_BIN,
    env.RUNX_PARSER_EVAL_BIN,
  );
  const candidate = configured ?? defaultRunxBinary;
  const resolved = path.isAbsolute(candidate) ? candidate : path.resolve(workspaceRoot, candidate);
  if (!existsSync(resolved)) {
    throw new Error(
      `tests require a prebuilt Rust binary; set RUNX_RUST_CLI_BIN/RUNX_KERNEL_EVAL_BIN or build ${path.relative(
        workspaceRoot,
        defaultRunxBinary,
      )}.`,
    );
  }
  return resolved;
}

export function kernelEnv(env: NodeJS.ProcessEnv = process.env): NodeJS.ProcessEnv {
  return {
    ...env,
    RUNX_KERNEL_EVAL_BIN: resolveRunxBinary(env),
  };
}

function firstNonEmpty(...values: Array<string | undefined>): string | undefined {
  return values.find((value): value is string => typeof value === "string" && value.length > 0);
}
