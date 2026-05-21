import {
  createExternalAdapterResponse,
  defineExternalAdapter,
} from "../../packages/authoring/src/index.ts";

const adapter = defineExternalAdapter({
  adapterId: "adapter.conformance.echo",
  invoke({ invocation }) {
    return createExternalAdapterResponse(invocation, {
      stdout: JSON.stringify({ message: invocation.inputs.message }),
      stderr: "",
      exitCode: 0,
      output: {
        adapter_language: "typescript",
        message: invocation.inputs.message,
        count: invocation.inputs.count,
      },
    });
  },
});

await adapter.main();
