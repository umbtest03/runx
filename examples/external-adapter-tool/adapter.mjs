// Minimal external-adapter subprocess (runx.external_adapter.v1).
//
// The runtime spawns this under the governed sandbox, writes the invocation
// frame to stdin (newline-terminated JSON), and reads one response frame from
// stdout. The response must echo the invocation's adapter_id and invocation_id.
let input = "";
process.stdin.on("data", (chunk) => {
  input += chunk;
});
process.stdin.on("end", () => {
  let invocation = {};
  try {
    invocation = JSON.parse(input.trim() || "{}");
  } catch {
    invocation = {};
  }
  const inputs = invocation.inputs || invocation.resolved_inputs || {};
  const response = {
    schema: "runx.external_adapter.response.v1",
    protocol_version: "runx.external_adapter.v1",
    adapter_id: invocation.adapter_id,
    invocation_id: invocation.invocation_id,
    status: "completed",
    exit_code: 0,
    observed_at: "2026-06-02T00:00:00Z",
    stdout: JSON.stringify({ ok: true, inputs }),
    stderr: "",
    output: { ok: true, inputs },
    artifacts: [],
    telemetry: []
  };
  process.stdout.write(JSON.stringify(response));
});
