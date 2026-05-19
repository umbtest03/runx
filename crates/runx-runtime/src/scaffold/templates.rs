// rust-style-allow: large-file because the scaffold templates intentionally
// mirror the TypeScript scaffolder's checked output byte-for-byte.

use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaffoldTemplateVersions {
    pub authoring_package_version: String,
    pub authoring_toolkit_version: String,
    pub cli_package_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaffoldFile {
    pub relative_path: String,
    pub contents: String,
}

pub fn scaffold_package_files(
    name: &str,
    packet_namespace: &str,
    versions: &ScaffoldTemplateVersions,
) -> Vec<ScaffoldFile> {
    let packet_id = format!("{packet_namespace}.echo.v1");
    let tool_source = tool_source(&packet_id);
    let tool_runtime = tool_runtime(&packet_id);
    let source_hash = source_hash(&tool_source, &tool_runtime);
    let schema_hash = schema_hash(&packet_id);
    let prompt_fingerprint = prompt_fingerprint(&packet_id);
    vec![
        file(
            "package.json",
            package_json(
                name,
                &versions.authoring_package_version,
                &versions.cli_package_version,
            ),
        ),
        file("README.md", readme(name)),
        file("SKILL.md", skill_md(name)),
        file("X.yaml", x_yaml(name)),
        file("src/packets/echo.ts", packet_source(&packet_id)),
        file(
            "dist/packets/echo.v1.schema.json",
            packet_schema(packet_namespace, &packet_id),
        ),
        file("tools/docs/echo/src/index.ts", tool_source.clone()),
        file("tools/docs/echo/run.mjs", tool_runtime),
        file(
            "tools/docs/echo/manifest.json",
            tool_manifest(
                &packet_id,
                &source_hash,
                &schema_hash,
                &versions.authoring_toolkit_version,
            ),
        ),
        file("tools/docs/echo/fixtures/basic.yaml", tool_fixture(&packet_id)),
        file("fixtures/agent.yaml", agent_fixture_yaml(&packet_id)),
        file(
            "fixtures/agent.replay.json",
            agent_replay_json(&packet_id, &prompt_fingerprint),
        ),
        file("fixtures/repos/readme-only/README.md", format!("# {name}\n")),
        file(".github/workflows/publish.yml", publish_workflow()),
        file(".gitignore", "node_modules/\n.runx/\n*.tgz\n".to_owned()),
        file(
            ".gitattributes",
            "tools/**/run.mjs linguist-generated=true\ntools/**/manifest.json linguist-generated=true\ntools/**/dist/** linguist-generated=true\n".to_owned(),
        ),
        file("tsconfig.json", tsconfig_json()),
    ]
}

fn file(relative_path: &str, contents: String) -> ScaffoldFile {
    ScaffoldFile {
        relative_path: relative_path.to_owned(),
        contents,
    }
}

fn package_json(name: &str, authoring_version: &str, cli_version: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "Scaffolded runx skill package.",
  "type": "module",
  "publishConfig": {{
    "access": "public"
  }},
  "scripts": {{
    "build": "runx tool build --all --json",
    "runx:list": "runx list --json",
    "runx:doctor": "runx doctor --json",
    "runx:dev": "runx dev --lane deterministic --json",
    "prepublishOnly": "runx tool build --all --json && runx doctor --json"
  }},
  "runx": {{
    "packets": [
      "./dist/packets/*.schema.json"
    ]
  }},
  "devDependencies": {{
    "@runxhq/authoring": "{authoring_version}",
    "@runxhq/cli": "{cli_version}",
    "@tsconfig/node20": "^20.1.6",
    "tsx": "^4.20.6"
  }}
}}"#
    )
}

fn readme(name: &str) -> String {
    format!(
        r#"# {name}

Runx authoring package: composable skills governed by typed contracts.

## Layout

- `SKILL.md`: Anthropic-standard skill description. Read by humans and agents.
- `X.yaml`: runx execution profile layered on top of `SKILL.md`.
- `src/packets/`: typed packet contracts authored with TypeBox.
- `tools/`: deterministic implementation units authored with `defineTool`.
- `fixtures/`: examples and tests across deterministic, agent, and repo-integration lanes.

## Authoring Loop

```bash
pnpm install
pnpm build
pnpm runx:list
pnpm runx:doctor
pnpm runx:dev
```

Edit `tools/docs/echo/src/index.ts`, then run `runx tool build --all` to regenerate `manifest.json` and `run.mjs`. Add fixtures in `tools/<namespace>/<name>/fixtures/` to lock behaviour.

Packet IDs are immutable. Schema changes mean a new packet ID, not an in-place edit.

## Bootstrap

- Canonical: `runx new {name}`
- Cold start: `npm create @runxhq/skill@latest {name}`

## Publish

The scaffold includes `.github/workflows/publish.yml`, which publishes with npm provenance from GitHub Actions. Before publishing, update `package.json` metadata for your repo and package.
"#
    )
}

fn skill_md(name: &str) -> String {
    format!(
        r#"---
name: {name}
description: Scaffolded runx skill package.
---

Use this skill to demonstrate a governed runx authoring package.
"#
    )
}

fn x_yaml(name: &str) -> String {
    format!(
        r#"skill: {name}

runners:
  default:
    default: true
    type: graph
    inputs:
      message:
        type: string
        required: false
        default: hello
    graph:
      name: {name}
      steps:
        - id: echo
          tool: docs.echo
          inputs:
            message: inputs.message
"#
    )
}

fn packet_source(packet_id: &str) -> String {
    format!(
        r#"import {{ definePacket, t }} from "@runxhq/authoring";

export const EchoPacket = definePacket({{
  id: "{packet_id}",
  schema: t.Object({{
    message: t.String(),
  }}),
}});
"#
    )
}

fn packet_schema(packet_namespace: &str, packet_id: &str) -> String {
    let schema_path = packet_namespace.replace('.', "/");
    format!(
        r#"{{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.runx.dev/{schema_path}/echo/v1.json",
  "x-runx-packet-id": "{packet_id}",
  "type": "object",
  "required": [
    "message"
  ],
  "properties": {{
    "message": {{
      "type": "string"
    }}
  }},
  "additionalProperties": false
}}"#
    )
}

fn tool_source(packet_id: &str) -> String {
    format!(
        r#"import {{ defineTool, stringInput }} from "@runxhq/authoring";

export default defineTool({{
  name: "docs.echo",
  version: "0.1.0",
  description: "Echo a docs message.",
  inputs: {{
    message: stringInput({{ default: "hello" }}),
  }},
  output: {{
    packet: "{packet_id}",
    wrap_as: "echo_packet",
  }},
  scopes: ["docs.read"],
  run({{ inputs }}) {{
    return {{ message: inputs.message }};
  }},
}});
"#
    )
}

fn tool_runtime(packet_id: &str) -> String {
    format!(
        r#"const fs = require("node:fs");
const rawInputs = process.env.RUNX_INPUTS_PATH
  ? fs.readFileSync(process.env.RUNX_INPUTS_PATH, "utf8")
  : (process.env.RUNX_INPUTS_JSON || "{{}}");
const inputs = JSON.parse(rawInputs);
process.stdout.write(JSON.stringify({{ schema: "{packet_id}", data: {{ message: String(inputs.message || "hello") }} }}));
"#
    )
}

fn tool_manifest(
    packet_id: &str,
    source_hash: &str,
    schema_hash: &str,
    toolkit_version: &str,
) -> String {
    format!(
        r#"{{
  "schema": "runx.tool.manifest.v1",
  "name": "docs.echo",
  "version": "0.1.0",
  "description": "Echo a docs message.",
  "source": {{
    "type": "cli-tool",
    "command": "node",
    "args": [
      "./run.mjs"
    ]
  }},
  "runtime": {{
    "command": "node",
    "args": [
      "./run.mjs"
    ]
  }},
  "inputs": {{
    "message": {{
      "type": "string",
      "required": false,
      "default": "hello"
    }}
  }},
  "output": {{
    "packet": "{packet_id}",
    "wrap_as": "echo_packet"
  }},
  "scopes": [
    "docs.read"
  ],
  "runx": {{
    "artifacts": {{
      "wrap_as": "echo_packet"
    }}
  }},
  "source_hash": "{source_hash}",
  "schema_hash": "{schema_hash}",
  "toolkit_version": "{toolkit_version}"
}}"#
    )
}

fn tool_fixture(packet_id: &str) -> String {
    format!(
        r#"name: echo-basic
lane: deterministic
target:
  kind: tool
  ref: docs.echo
inputs:
  message: hello
expect:
  status: success
  output:
    subset:
      schema: {packet_id}
      data:
        message: hello
"#
    )
}

fn agent_fixture_yaml(packet_id: &str) -> String {
    format!(
        r#"name: echo-agent-replay
lane: agent
target:
  kind: skill
  ref: .
inputs:
  message: hello
agent:
  mode: replay
expect:
  status: success
  outputs:
    echo_packet:
      matches_packet: {packet_id}
"#
    )
}

fn agent_replay_json(packet_id: &str, prompt_fingerprint: &str) -> String {
    format!(
        r#"{{
  "schema": "runx.replay.v1",
  "fixture": "echo-agent-replay",
  "prompt_fingerprint": "{prompt_fingerprint}",
  "recorded_at": "1970-01-01T00:00:00.000Z",
  "target": {{
    "kind": "skill",
    "ref": "."
  }},
  "status": "success",
  "outputs": {{
    "echo_packet": {{
      "schema": "{packet_id}",
      "data": {{
        "message": "hello"
      }}
    }}
  }},
  "usage": {{
    "mode": "scaffold"
  }}
}}"#
    )
}

fn publish_workflow() -> String {
    r#"name: publish

on:
  workflow_dispatch:
  release:
    types:
      - published

jobs:
  publish:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      id-token: write
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 10
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          registry-url: https://registry.npmjs.org
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm build
      - run: pnpm runx:doctor
      - run: npm publish --provenance --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
"#
    .to_owned()
}

fn tsconfig_json() -> String {
    r#"{
  "extends": "@tsconfig/node20/tsconfig.json",
  "compilerOptions": {
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true
  },
  "include": [
    "src/**/*.ts",
    "tools/**/*.ts"
  ]
}"#
    .to_owned()
}

fn source_hash(tool_source: &str, tool_runtime: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update("src/index.ts");
    hasher.update([0]);
    hasher.update(tool_source);
    hasher.update([0]);
    hasher.update("run.mjs");
    hasher.update([0]);
    hasher.update(tool_runtime);
    hasher.update([0]);
    format!("sha256:{:x}", hasher.finalize())
}

fn schema_hash(packet_id: &str) -> String {
    let stable = format!(
        r#"{{"artifacts":{{"wrap_as":"echo_packet"}},"inputs":{{"message":{{"default":"hello","required":false,"type":"string"}}}},"output":{{"packet":"{packet_id}","wrap_as":"echo_packet"}}}}"#
    );
    format!("sha256:{}", hash_string(&stable))
}

fn prompt_fingerprint(packet_id: &str) -> String {
    let stable = format!(
        r#"{{"agent":{{"mode":"replay"}},"expect":{{"outputs":{{"echo_packet":{{"matches_packet":"{packet_id}"}}}},"status":"success"}},"inputs":{{"message":"hello"}},"target":{{"kind":"skill","ref":"."}}}}"#
    );
    format!("sha256:{}", hash_string(&stable))
}

fn hash_string(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())
}
