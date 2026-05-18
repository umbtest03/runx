---
spec_version: '2.0'
task_id: rust-placeholder-crates-publish
created: '2026-05-17T02:30:00Z'
updated: '2026-05-17T12:35:00Z'
status: draft
harden_status: not_run
size: small
risk_level: high
---

# Rust placeholder crates publish

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld harden rust-placeholder-crates-publish`
Latest runner update: 2026-05-17T12:31:02Z - all seven crate names published and verified with cargo search.
Review gate: not_started

## Summary

Publish the reservation Rust crates to crates.io to claim the package names.
This is a reservation release, not a feature release. Placeholder crates must
clearly identify themselves as placeholders, while `runx-core` may contain
early conformance surfaces without claiming TypeScript cutover. Crates must
publish in dependency order so crates.io can resolve local workspace
dependencies as registry dependencies.

## Context

CWD: `crates`

Published packages:
- `runx-contracts` `0.0.1`
- `runx-core` `0.0.1`
- `runx-parser` `0.0.1`
- `runx-receipts` `0.0.1`
- `runx-runtime` `0.0.1`
- `runx-sdk` `0.0.1`
- `runx-cli` `0.1.0`

Files impacted:
- `crates/**/Cargo.toml`
- `crates/**/README.md`
- `crates/Cargo.lock`
- `docs/rust-kernel-architecture.md`

Invariants:
- Reservation publishing claims names only. It does not claim native feature
  parity or TypeScript cutover.
- Placeholder crates use reservation version `0.0.1`. `runx-cli` remains
  `0.1.0` because it is a usable launcher.
- Publish order is dependency order:
  1. `runx-contracts`
  2. `runx-core`
  3. `runx-parser`
  4. `runx-receipts`
  5. `runx-runtime`
  6. `runx-sdk`
  7. `runx-cli`
- Use Claude for review. Local review does not satisfy complete.
- Do not publish if any crate name is already taken by an unrelated owner.
- Do not publish if `cargo package --workspace --allow-dirty` fails.

Related docs:
- `docs/rust-kernel-architecture.md`
- `crates/README.md`

## Objectives

- Confirm the crate graph is publish-ready.
- Confirm names are available or already owned by the runx publisher.
- Package the workspace locally before and after publish.
- Publish the placeholder crates in dependency order.
- Record published versions and owners.

## Scope

In scope:
- crates.io placeholder name reservation.
- Cargo package metadata sanity.
- Publish order and post-publish verification.

Out of scope:
- Implementing any real Rust behavior.
- Changing crate names.
- Publishing non-placeholder feature releases.
- npm release changes.

## Dependencies

- `rust-contracts-bootstrap` completed and approved.
- A valid crates.io token is available via Cargo credentials or
  `CARGO_REGISTRY_TOKEN`.
- Crates.io organization/owner decision is settled.

## Assumptions

- The user intentionally wants placeholder crates published to claim names.
- If a name is already taken by an unrelated crate, execution stops and the
  crate graph is revisited.

## Risks

- High: crates.io publishing is irreversible for version numbers. Mitigated by
  package verification, exact-version search checks, and Claude review before
  future publishes.
- High: claiming placeholder names can confuse users. Mitigated by README and
  crate docs stating placeholder status.
- Medium: dependency-order mistakes can strand later publishes. Mitigated by
  fixed publish order and `cargo package --workspace --allow-dirty`.

## Acceptance

Profile: strict
Self-eval threshold: 8
Review provider: external Claude; local review does not satisfy complete.
Harden required before approve: yes

Definition of done:
- [ ] `dod1` command - crate graph and Rust style checks pass.
  - Command: `pnpm rust:crate-graph && pnpm rust:style`
- [ ] `dod2` command - `cargo package --workspace --allow-dirty` passes.
  - Command: `cargo package --workspace --allow-dirty`
- [ ] `dod3` command - all published crates are discoverable.
  - Command: `for p in runx-contracts runx-core runx-parser runx-receipts runx-runtime runx-sdk runx-cli; do cargo search "$p" --limit 5 | rg -n "^$p\\s=" >/dev/null || exit 1; done`
- [ ] `dod4` command - placeholder crates are published or already owned by
  the runx publisher at the expected versions.
  - Command: `for p in runx-contracts runx-core runx-parser runx-receipts runx-runtime runx-sdk; do cargo search "$p" --limit 5 | rg -n "^$p\\s= \"0\\.0\\.1\"" >/dev/null || exit 1; done && cargo search runx-cli --limit 5 | rg -n '^runx-cli\\s= "0\\.1\\.0"' >/dev/null`
- [ ] `dod5` command - published placeholder crates retain placeholder README
  language in local package sources. `runx-core` is checked separately because
  it now contains state-machine parity.
  - Command: `for p in runx-contracts runx-parser runx-receipts runx-runtime runx-sdk; do rg -n 'Placeholder|placeholder' "crates/$p/README.md" >/dev/null || exit 1; done && rg -n 'state-machine parity|conformance evidence|TypeScript remains authoritative' crates/runx-core/README.md`

Validation:
- [ ] `v1` command - graph and style checks pass.
  - Command: `pnpm rust:crate-graph && pnpm rust:style`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v2` command - package verification passes.
  - Command: `cargo package --workspace --allow-dirty`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v3` command - published versions are discoverable.
  - Command: `for p in runx-contracts runx-core runx-parser runx-receipts runx-runtime runx-sdk; do cargo search "$p" --limit 5 | rg -n "^$p\\s= \"0\\.0\\.1\"" >/dev/null || exit 1; done && cargo search runx-cli --limit 5 | rg -n '^runx-cli\\s= "0\\.1\\.0"' >/dev/null`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v4` command - crates can be found after publish.
  - Command: `for p in runx-contracts runx-core runx-parser runx-receipts runx-runtime runx-sdk runx-cli; do cargo search "$p" --limit 5 | rg -n "^$p\\s=" >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: none
  - Source event: none
  - Last attempt: none
  - Checked at: none

## Phase 1: Version bump and pre-publish verification

Goal: Verify placeholder crates use reservation version `0.0.1`, then verify
package metadata, dependency order, and placeholder messaging.

Status: pending
Dependencies: `rust-contracts-bootstrap`

Acceptance:
- [ ] `ac1_1` command - workspace package verification passes.
  - Command: `cargo package --workspace --allow-dirty`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac1_2` command - placeholder READMEs name placeholder status, and
  `runx-core` README names its conformance-only status.
  - Command: `for p in runx-contracts runx-parser runx-receipts runx-runtime runx-sdk; do rg -n 'Placeholder|placeholder' "crates/$p/README.md" >/dev/null || exit 1; done && rg -n 'state-machine parity|conformance evidence|TypeScript remains authoritative' crates/runx-core/README.md`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac1_3` command - publish versions are staged.
  - Command: `for p in runx-contracts runx-core runx-parser runx-receipts runx-runtime runx-sdk; do rg -n '^version = "0\\.0\\.1"$' "crates/$p/Cargo.toml" >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Publish preflight

Goal: Run local workspace package verification for the dependency chain.

Status: pending
Dependencies: Phase 1

Acceptance:
- [ ] `ac2_1` command - publish preflight passes.
  - Command: `cargo package --workspace --allow-dirty`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Publish placeholders

Goal: Publish the placeholder crates in dependency order.

Status: pending
Dependencies: Phase 2 and explicit user confirmation immediately before
running real publish commands.

Commands:
- verify placeholder crate versions are `0.0.1`, including `crates/Cargo.toml`
  workspace dependency versions
- `cargo publish -p runx-contracts --allow-dirty`
- wait until `cargo search runx-contracts --limit 5` finds `runx-contracts`
- `cargo publish -p runx-core --allow-dirty`
- wait until `cargo search runx-core --limit 5` finds `runx-core`
- `cargo publish -p runx-parser --allow-dirty`
- wait until `cargo search runx-parser --limit 5` finds `runx-parser`
- `cargo publish -p runx-receipts --allow-dirty`
- wait until `cargo search runx-receipts --limit 5` finds `runx-receipts`
- `cargo publish -p runx-runtime --allow-dirty`
- wait until `cargo search runx-runtime --limit 5` finds `runx-runtime`
- `cargo publish -p runx-sdk --allow-dirty`
- wait until `cargo search runx-sdk --limit 5` finds `runx-sdk`
- `cargo publish -p runx-cli --allow-dirty`

Acceptance:
- [ ] `ac3_1` command - all crates are discoverable.
  - Command: `for p in runx-contracts runx-core runx-parser runx-receipts runx-runtime runx-sdk runx-cli; do cargo search "$p" --limit 5 | rg -n "^$p\\s=" >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

Strategy: manual

Published crates.io versions cannot be deleted. If a publish is wrong, yank
the affected version and publish a corrected version:

- `cargo yank --vers <version> <crate>`

## Review

Status: not_started
Verdict: none
Timestamp: none
Review rounds: none
Reviewer mode: none
Reviewer session: none
Round status: none
Override applied: none
Override reason: none
Override confirmed at: none
Reviewed head: none
Reviewed dirty: none
Reviewed diff: none
Blocking count: none
Non-blocking count: none

Reviewer requirements:
- Claude review required.
- Verify irreversible publish risk and dependency order.
- Verify placeholder READMEs do not claim feature parity.

Findings:
- none

Passes:
- none

## Self Eval

Status: not_started
Completeness: none
Architecture fidelity: none
Spec alignment: none
Validation depth: none
Total: none
Second pass performed: none

Threshold: 8

Notes:
none

Improvements:
- none

## Deviations

- none

## Metadata

Estimated effort hours: 2
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- crates-io
- publish
- placeholders

## Origin

Source:
- user clarified that placeholder crates should be published to claim names.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- depends_on: rust-contracts-bootstrap

## Harden Rounds

- none

## Planning Log

- 2026-05-17T02:30:00Z: Drafted publish/reservation spec after user clarified
  placeholders should be published to claim the crates.io names.
- 2026-05-17T12:31:02Z: Published and verified `runx-contracts` `0.0.1`,
  `runx-core` `0.0.1`, `runx-parser` `0.0.1`, `runx-receipts` `0.0.1`,
  `runx-runtime` `0.0.1`, `runx-sdk` `0.0.1`, and `runx-cli` `0.1.0`.
