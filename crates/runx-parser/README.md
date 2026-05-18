# runx-parser

Pure Rust parser parity crate for runx parser boundaries.

The TypeScript parser remains the authoring reference while this crate proves
fixture parity. The Rust implementation currently covers:

- execution graphs
- skill markdown frontmatter and body preservation
- runner manifests and harness cases
- tool manifests from YAML and JSON
- skill install envelopes

The crate intentionally stays pure: it parses and validates typed intermediate
representations, uses `runx_contracts::JsonValue` and the
`runx_contracts::execution` semantic types at public parser boundaries, reuses
`runx_core::policy` sandbox normalization, and has no filesystem,
environment, network, or provider SDK dependencies.

Fixture generation is TypeScript-authored and checked by
`scripts/generate-rust-parser-fixtures.ts`; Rust tests assert byte-level shape
parity against `fixtures/parser/**`.
