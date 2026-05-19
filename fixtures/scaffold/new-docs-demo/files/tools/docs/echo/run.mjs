const fs = require("node:fs");
const rawInputs = process.env.RUNX_INPUTS_PATH
  ? fs.readFileSync(process.env.RUNX_INPUTS_PATH, "utf8")
  : (process.env.RUNX_INPUTS_JSON || "{}");
const inputs = JSON.parse(rawInputs);
process.stdout.write(JSON.stringify({ schema: "docs.demo.echo.v1", data: { message: String(inputs.message || "hello") } }));
