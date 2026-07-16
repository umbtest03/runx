# Runx

Runx is the governed runtime for agent skills. Skills declare their inputs,
authority, tools, credential needs, and completion evidence; the Rust runtime
admits the act, executes it inside the declared boundary, and seals what
happened in a verifiable receipt.

## Start locally

Build the native CLI:

```bash
cargo build --manifest-path crates/Cargo.toml -p runx-cli
```

Run the checked-in example:

```bash
export RUNX_RECEIPT_DIR="$(mktemp -d)"
crates/target/debug/runx skill examples/hello-world \
  --message "hello from runx" \
  --non-interactive \
  --json
```

The native path does not require Node or TypeScript. The npm package
`@runxhq/cli` distributes the same Rust-owned behavior.

## Provider-backed skills

Credential requirements live in each skill's `X.yaml`. Configure a local
profile once by piping material on stdin:

```bash
printf '%s' "$NITROSEND_API_KEY" |
  crates/target/debug/runx credential set nitrosend --from-stdin
```

Runx resolves explicit profiles, project bindings, global defaults, hosted
handles, and the workspace environment through one canonical path. Agents,
resume, and MCP use that same readiness check. See
[Credential Resolution](docs/credentials.md).

## Documentation

- [Getting Started](docs/getting-started.md)
- [CLI and architecture reference](docs/reference.md)
- [Credential Resolution](docs/credentials.md)
- [Skill to Graph](docs/skill-to-graph.md)
- [Agent exports for Claude and Codex](docs/agent-skills.md)
- [How We Test](docs/how-we-test.md)
- [Security](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

The trusted-kernel and package boundaries are documented in
[Rust Kernel Architecture](docs/rust-kernel-architecture.md) and
[Trusted Kernel Package Truth](docs/trusted-kernel-package-truth.md).

## Repository map

- `crates/`: trusted Rust contracts, parser, policy, runtime, receipts, CLI,
  and SDK.
- `skills/`: official governed skills.
- `examples/`: executable authoring and integration examples.
- `fixtures/`: deterministic conformance and parity evidence.
- `packages/`: TypeScript contracts, authoring helpers, host adapters, and npm
  distribution surfaces over the Rust-owned runtime.

Run `runx --help` for the native command surface and `pnpm verify:fast` for the
fast repository validation lane.
