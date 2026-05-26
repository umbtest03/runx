# How We Test

Runx has two local test lanes: a fast loop for package-adjacent work and a full
workspace suite for release confidence.

## Fast Loop

Use this while editing core runtime, harness, parser, policy, or nearby tests:

```bash
pnpm test:fast
```

`test:fast` uses `vitest.fast.config.ts`. It includes package tests plus
coverage for surviving TypeScript package boundaries.

For canonical local runtime behavior, prefer the Rust lane directly. Payment,
authority, receipt, harness, dogfood, registry, and policy-config changes need
Rust coverage or a TS-free Rust CLI fixture:

```bash
cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment
cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood
```

For one file:

```bash
pnpm vitest run tests/examples/hello-world.test.ts
```

## Full Suite

Use this before review or when changing CLI packaging, dist output, package
exports, or cross-package TypeScript wrapper behavior:

```bash
pnpm test
```

`pnpm test` runs `scripts/test-workspace.mjs`. With no explicit target, it runs
the workspace suite except `tests/cli-package.test.ts`, then runs
`tests/cli-package.test.ts` in a second pass with:

```bash
RUNX_VITEST_BATCH=cli-package
```

That ordering is intentional. `cli-package.test.ts` rebuilds and inspects
package output, so isolating it avoids races with tests that import from the
same dist trees.

To run the CLI package test directly:

```bash
RUNX_VITEST_BATCH=cli-package pnpm vitest run tests/cli-package.test.ts
```

## Fixtures

Use checked-in fixtures when a behavior should remain stable:

- `fixtures/skills/` for reusable skill packages
- `fixtures/graphs/` for graph execution shapes
- `fixtures/harness/` for harness-level contracts
- `examples/` for public docs examples that should also be executable

Prefer small fixtures with one purpose. If an example appears in docs, add a
test or harness so the docs fail loudly when the runtime shape changes.

## Adding Tests

Use package-local tests for package internals and `tests/` for cross-package
wrapper behavior. Trusted local skill, graph, harness, receipt, policy,
authority, registry, config, and payment behavior needs a Rust test or a
TS-free Rust CLI fixture. TypeScript tests may wrap those paths, but they
should not be the only proof.

For docs examples, keep the test focused on the public command or runtime path
the docs promise. The hello-world and hello-graph tests are the reference shape.
