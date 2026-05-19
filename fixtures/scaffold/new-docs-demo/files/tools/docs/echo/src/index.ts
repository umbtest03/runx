import { defineTool, stringInput } from "@runxhq/authoring";

export default defineTool({
  name: "docs.echo",
  version: "0.1.0",
  description: "Echo a docs message.",
  inputs: {
    message: stringInput({ default: "hello" }),
  },
  output: {
    packet: "docs.demo.echo.v1",
    wrap_as: "echo_packet",
  },
  scopes: ["docs.read"],
  run({ inputs }) {
    return { message: inputs.message };
  },
});
