# @runxhq/core Changelog

## Unreleased

- `@runxhq/core/policy` now normalizes executable names with POSIX-only
  basename semantics for kernel parity. Backslashes are treated as path
  separators before stripping `.exe`, `.cmd`, or `.bat` suffixes, so behavior
  is deterministic across POSIX and Windows hosts.
