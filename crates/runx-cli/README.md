# runx-cli

`runx-cli` is the Cargo package for installing the `runx` command.

The canonical runx CLI implementation currently ships as the npm package
`@runxhq/cli`. This Cargo package installs a small native launcher named
`runx` that delegates to the official npm CLI.

```bash
cargo install runx-cli
runx --help
```

The `runx` crate name on crates.io is already owned by an unrelated package, so
the published Cargo package is `runx-cli` while the installed binary remains
`runx`.

## Runtime Requirements

- Rust/Cargo for installation.
- Node.js 20+ and npm for executing the official CLI.

By default the launcher runs the latest published npm CLI:

```bash
npm exec --yes --package @runxhq/cli@latest -- runx <args>
```

Set `RUNX_NPM_PACKAGE` to pin a specific npm version:

```bash
RUNX_NPM_PACKAGE='@runxhq/cli@0.5.22' runx --help
```

Set `RUNX_JS_BIN` to delegate to a local JavaScript CLI entrypoint instead:

```bash
RUNX_JS_BIN=/path/to/runx/oss/packages/cli/bin/runx.js runx --help
```

For the launcher version itself:

```bash
runx --shim-version
```
