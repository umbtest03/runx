# @runxhq/core Changelog

## Unreleased

- `@runxhq/core/policy` now normalizes executable names with POSIX-only
  basename semantics for kernel parity. Backslashes are treated as path
  separators before stripping `.exe`, `.cmd`, or `.bat` suffixes, so behavior
  is deterministic across POSIX and Windows hosts.
- Rust integration tests now compile as one binary per crate (`autotests = false`
  with a `tests/integration.rs` module index), so each crate and its heavy
  dependencies link once instead of once per test file, cutting test build time
  substantially. A guard fails the build if a test file is left unreferenced or
  if a test mutates process-global state. CI runs the suite with cargo-nextest,
  keeps doctests in a separate `cargo test --doc` step, and caches compilation
  with sccache.
- Added verified doctests on public-API examples: the `runx-contracts`
  fingerprint helpers, the `runx-sdk` `CommandPlan` builder, and the
  `runx-parser` YAML scalar guard.
