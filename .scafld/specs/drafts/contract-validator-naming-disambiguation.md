---
spec_version: '2.0'
task_id: contract-validator-naming-disambiguation
created: '2026-05-24T00:00:00Z'
updated: '2026-05-24T00:00:00Z'
status: draft
harden_status: not_run
size: small
risk_level: medium
---

# Disambiguate same-name / divergent-contract helpers

## Current State

Status: draft
Current phase: planning
Next: harden
Reason: A+ roadmap step 3. A deep review found several helpers that share a NAME
across the TS codebase but have DIFFERENT behavior, so importing the wrong one is
a silent footgun:
- `optionalString` / `requireString`: the `@runxhq/core/util` versions THROW on
  bad/empty input; ~10 scattered copies silently RETURN UNDEFINED.
- `recordField`: some copies return the field as `unknown`; others return it only
  when it is itself a record (`Record | undefined`).
- `firstNonEmpty`: some return `string` (default ""), others `string | undefined`.
Blockers: none. The shared lenient accessors were already consolidated into
`@runxhq/core/util`; this spec finishes the job by naming the divergent contracts
honestly.

## Summary

Rename the helpers so the name states the contract, eliminating the
import-the-wrong-one hazard. Throwing validators keep `require*`/`optional*`
(they validate); lenient extractors get a distinct verb (e.g. `coerce*` /
`read*Field`); the two `firstNonEmpty` shapes keep the `…OrUndefined` suffix
distinction already established in `@runxhq/core/util`.

## Objectives

- One canonical name per contract; no two functions named identically with
  different throw/return behavior anywhere in `packages/`.
- Point all call sites at the canonical helpers in `@runxhq/core/util` (and the
  package-local equivalents that cannot import core).
- Keep behavior identical at every call site (a rename, not a semantics change):
  a throwing site stays throwing, a lenient site stays lenient.

## Scope

In scope: `@runxhq/core/util` validators/accessors and their call sites across
`packages/core`, `packages/runtime-local`, `packages/cli`, `plugins/`.

Out of scope: `packages/authoring` and `packages/adapters` exported public API
names (coordinate separately if they collide); the Rust side.

## Acceptance

- [ ] `dod1` No identifier is defined with two different throw/return contracts
  across `packages/` (verified by a grep/lint over the helper names).
- [ ] `dod2` Every migrated call site preserves its prior runtime behavior
  (throwing stays throwing, lenient stays lenient); `verify:fast` stays green.
- [ ] `dod3` The lenient accessors live once in `@runxhq/core/util`; duplicates
  are removed where the package can import core.

## Origin

A+ roadmap (2026-05-24), step 3. Surfaced by the structural/dark-pattern review
during the shared-helper consolidation.
