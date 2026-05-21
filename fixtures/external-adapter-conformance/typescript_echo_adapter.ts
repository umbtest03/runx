import { readFileSync } from "node:fs";

const invocation = JSON.parse(readFileSync(0, "utf8")) as {
  readonly invocation_id: string;
  readonly adapter_id: string;
  readonly inputs?: {
    readonly message?: unknown;
    readonly count?: unknown;
  };
};
const inputs = invocation.inputs ?? {};

process.stdout.write(JSON.stringify({
  schema: "runx.external_adapter.response.v1",
  protocol_version: "runx.external_adapter.v1",
  invocation_id: invocation.invocation_id,
  adapter_id: invocation.adapter_id,
  status: "completed",
  stdout: JSON.stringify({ message: inputs.message }),
  stderr: "",
  exit_code: 0,
  output: {
    adapter_language: "typescript",
    message: inputs.message,
    count: inputs.count,
  },
  observed_at: new Date().toISOString(),
}));
