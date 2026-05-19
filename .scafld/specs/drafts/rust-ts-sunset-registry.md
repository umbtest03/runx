---
spec_version: '2.0'
task_id: rust-ts-sunset-registry
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T06:13:24Z'
status: draft
harden_status: passed
size: medium
risk_level: high
---

# TS sunset: registry (core domain)

## Current State

Status: draft
Current phase: hardened
Next: approve
Reason: hardened against the active `runx-runtime::registry` plan and the
harness-spine vocabulary. Sixth TS sunset.
Blockers: `rust-ts-sunset-receipts` complete; `runx-runtime::registry` complete;
`crates/runx-runtime/src/registry/` consumed by every registry IO surface in the
CLI and runtime.
Allowed follow-up command: `scafld approve rust-ts-sunset-registry`
Latest runner update: none
Review gate: not_started

## Summary

Delete the TypeScript registry core domain and its public subpath export:
`packages/core/src/registry/**` and `@runxhq/core/registry`. Registry IO becomes
Rust-owned through `crates/runx-runtime/src/registry/`; TS does not retain compat
shims, legacy emitted registry shapes, or a second-version registry surface.

Product registry names stay product names. Keep registry skill names, package
path names, owner/name refs, install refs, `skill_id`, `trust_tier`, and hosted
registry route fields where they are the registry product surface. Do not carry
old receipt or harness wording forward just to keep TS fixtures compiling.

This spec only sunsets OSS-side TS registry code. Hosted cloud registry routes,
namespace policy, and publisher authorization stay in the cloud package.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `@runxhq/cli`
- `@runxhq/runtime-local`
- `crates/runx-runtime` (`registry` module owns registry IO and install logic)
- `cloud/packages/api` (registry HTTP routes; not touched)

Current TypeScript sources:
- `packages/core/src/registry/**` (to be deleted)
- `packages/runtime-local/src/runner-local/registry-resolver.ts`
- `packages/runtime-local/src/runner-local/skill-install.ts`
- `packages/runtime-local/src/runner-local/official-cache.ts`
- CLI dispatch, skill refs, and tests that still import `@runxhq/core/registry`

Files impacted:
- `packages/core/src/registry/` (deleted)
- `packages/core/package.json` (`"./registry"` export removed)
- TS importers only as needed to stop using `@runxhq/core/registry`
- Registry fixture/test references only as needed to point at
  `runx-runtime::registry`

Invariants:
- Rust source of truth: all registry search, read, acquire, resolve, and local
  install semantics are owned by `crates/runx-runtime/src/registry/`.
- Hosted registry HTTP behavior is unchanged. This sunset consumes the surface
  already implemented by `runx-runtime::registry`; it does not add endpoints,
  negotiate a new version, or invent fallback payloads.
- There is no `@runxhq/core/registry` re-export, proxy module, TS wrapper,
  cross-language adapter, compat shim, legacy shape emitter, or `/v2` registry
  path.
- Trust tiers (`first_party`, `verified`, `community`) remain exact
  server-provided values. TS deletion must not reintroduce owner-derived trust
  logic.
- Registry install package paths keep the `runx-runtime::registry`
  normalization: namespaced refs derive owner/name path components; bare refs
  derive from the skill name.
- Direct `runx skill add` / `runx skill add` remains a local install action. When
  registry install happens inside execution, evidence belongs to the enclosing
  sealed harness receipt or runtime ledger metadata. It does not emit retired
  `skill_execution` or `graph_execution` receipt shapes.
- Harness assertions use harness-spine terms: harness receipt refs, sealed
  receipt state, contained decisions, contained acts, artifact refs, signal
  refs, proof status, and verification checks.

## Objectives

- Prove every live registry IO caller has moved to
  `crates/runx-runtime/src/registry/` or a Rust-owned launcher/runtime boundary
  before deleting TS.
- Delete `packages/core/src/registry/**` and remove the `@runxhq/core/registry`
  package export.
- Remove or port TS tests whose only purpose was to validate the deleted TS
  registry implementation.
- Preserve product-facing registry behavior: search, inspect/read, acquire,
  bare-ref resolution, idempotent local install, profile binding validation,
  trust tier round-trip, and package path derivation.
- Keep registry install evidence in harness-spine vocabulary when it is part of
  an execution.

## Scope

In scope:
- TS registry core deletion and subpath export removal.
- Importer cleanup for OSS packages that still reference `@runxhq/core/registry`.
- Test and fixture cleanup required to assert the Rust registry client is the
  only live registry IO implementation.

Out of scope:
- Cloud-side registry routes / logic.
- Hosted namespace ownership and publisher authorization policy.
- Registry signing / attestation hierarchy beyond pass-through validation that
  already belongs to `runx-runtime::registry`.
- Adding a TS-to-Rust compatibility layer.
- Adding a second registry API version.
- Changing product registry skill names, owner/name refs, or install package
  names solely to satisfy the sunset.

## Dependencies

- `rust-ts-sunset-receipts`.
- `runx-runtime::registry` completed and handed off, accepted as the source for
  registry search, read, acquire, resolve, and local install.
- `rust-harness` or equivalent harness-spine receipt support completed before
  any registry install evidence is used as cutover proof.

## Sequencing

1. Finish `runx-runtime::registry` first. The runtime module must expose and
   test the surfaces currently represented by `RegistryClient`, `RegistryStore`,
   `resolveRegistrySkill`, `acquireRegistrySkill`, `materializeRegistrySkill`,
   and local skill install helpers.
2. Confirm CLI/runtime registry callers use the Rust client path. This includes
   search/add/inspect/publish/list dispatch, graph registry refs, official skill
   cache acquisition, and runtime materialization.
3. Run an importer census. Every live `@runxhq/core/registry` import must be
   removed, replaced by a Rust-owned call path, or deleted with the TS test it
   only supported.
4. Delete `packages/core/src/registry/**`, remove the `./registry` export from
   `packages/core/package.json`, and remove stale build references.
5. Refresh tests and fixtures to assert Rust behavior. Registry HTTP payload
   fixtures may still contain product fields such as `owner`, `skill_id`,
   `source_type`, `trust_tier`, and `install_command`; receipt fixtures must use
   harness-spine vocabulary only.
6. Run the full acceptance command set before approval. If a command needs new
   Rust integration that is not yet present, stop and return to
   `runx-runtime::registry` rather than adding a TS shim here.

## Acceptance Criteria

- `packages/core/src/registry/` is gone.
- `packages/core/package.json` no longer exports `./registry`.
- No live source, test, script, tool, or first-party skill imports
  `@runxhq/core/registry` or reaches into `packages/core/src/registry`.
- Registry search, read, acquire, bare-name resolution, idempotent install,
  digest/profile digest validation, runner manifest validation, and safe package
  path derivation are covered by `runx-runtime::registry` tests.
- CLI/runtime registry flows use the Rust registry client as the source for
  registry IO. TS may invoke a Rust binary or launcher boundary, but it must not
  duplicate registry semantics in TS.
- Hosted registry payload shape remains the stabilized registry product shape;
  no legacy emitted TS registry shape, compatibility adapter, or `/v2` endpoint
  is introduced.
- Product surface names remain stable: `skill_id`, owner/name refs, product
  skill names, package path names, install commands, run commands, trust tiers,
  publisher metadata, source metadata, and attestations round-trip where the
  registry surface owns them.
- Execution evidence uses harness-spine terms. No active registry cutover
  fixture asserts retired receipt fields such as `skill_execution`,
  `graph_execution`, `skill_name`, or `graph_name`.
- Rollback instructions below are documented in the implementation PR.

## Validation Commands

```sh
test ! -d packages/core/src/registry
node -e 'const pkg = require("./packages/core/package.json"); if (pkg.exports && pkg.exports["./registry"]) process.exit(1)'
! rg -n '@runxhq/core/registry|packages/core/src/registry|from "\.?\.?/.*registry/store|from "\.?\.?/.*registry/resolve' packages tests scripts tools skills --glob '!**/dist/**' --glob '!**/node_modules/**'
cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test registry_client
cargo test --manifest-path crates/Cargo.toml -p runx-runtime registry
pnpm test -- tests/graph-registry-refs.test.ts tests/graph-registry-refs.integration.test.ts tests/skill-add.test.ts tests/skill-search.test.ts tests/skill-publish.test.ts tests/registry-ce.test.ts packages/cli/src/index.test.ts
! rg -n 'skill_execution|graph_execution|skill_name|graph_name' fixtures/registry crates/runx-runtime
cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets -- -D warnings
cargo fmt --manifest-path crates/Cargo.toml --all --check
pnpm typecheck
```

## Rollback And Repair

- Pre-merge rollback is to back out the whole sunset implementation patch and
  return to the `runx-runtime::registry` blocker. Do not add an interim
  `@runxhq/core/registry` proxy to keep partial deletion alive.
- Post-merge repair is forward through `crates/runx-runtime/src/registry/`,
  `crates/runx-runtime`, or the launcher boundary. Do not resurrect TS registry
  modules, legacy emitted shapes, or a second registry version.
- If a CLI/runtime caller still needs a registry capability, add it to
  `runx-runtime::registry` with tests, then wire the caller to that Rust-owned
  capability.
- If hosted payload validation is too strict for the live registry, repair the
  Rust payload parser and fixture against the hosted surface; do not tolerate
  missing required fields in a TS fallback.
- If harness evidence is wrong, repair the harness-spine fixture/projection and
  proof checks. Do not restore retired receipt expectation fields.
- If cloud registry behavior is the issue, stop this OSS sunset and fix the
  cloud route or fixture in its owning repo/spec. This spec must not patch
  cloud-side behavior.

## Open Questions

- None at draft time.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T06:13:24Z
Ended: 2026-05-19T06:13:24Z
Verdict: passed
Provider: manual
Summary: Reframed the draft around `runx-runtime::registry` ownership,
explicit TS deletion, harness-spine receipt vocabulary, acceptance commands,
sequencing, and repair rules.

Checks:
- registry client alignment
  - Result: passed
  - Evidence: The spec names `runx-runtime::registry` as the source for
    registry search, read, acquire, resolve, and local install.
- sunset target
  - Result: passed
  - Evidence: The spec forbids TS compatibility shims, legacy emitted shapes,
    and a second registry version.
- vocabulary audit
  - Result: passed
  - Evidence: Registry product fields are preserved, while execution evidence
    is constrained to harness-spine receipt terms.
- acceptance coverage
  - Result: passed
  - Evidence: Acceptance criteria and validation commands cover deletion,
    import cleanup, Rust tests, TS flow tests, and retired receipt field scans.
- rollback audit
  - Result: passed
  - Evidence: Rollback/repair requires whole-patch rollback before merge or
    forward repair through Rust after merge.

Issues:
- none
