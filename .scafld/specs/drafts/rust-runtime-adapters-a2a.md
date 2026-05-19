---
spec_version: '2.0'
task_id: rust-runtime-adapters-a2a
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T08:13:37Z'
status: draft
harden_status: passed
size: medium
risk_level: medium
---

# Rust runtime a2a adapter

## Current State

Status: draft
Current phase: none
Next: approve
Reason: hardened for execution after `rust-runtime-adapters-agent`; transport,
authority attenuation, receipt linkage, and fixture gates are explicit.
Blockers: `rust-runtime-skeleton`, `rust-runtime-adapters-agent`,
`runx-contract-spine-hard-cutover`, and post-cutover receipt proof/tree APIs.
Allowed follow-up command: `scafld approve rust-runtime-adapters-a2a`
Latest runner update: none
Review gate: not_started

## Summary

Port the `a2a` adapter family to `runx-runtime` behind the
`features = ["a2a"]` flag. A2A dispatch sends a contained act to another agent
surface through an explicit transport while the current harness preserves
authority attenuation, parent/child harness receipt linkage, and seal proof.

The adapter owns message/task transport and task polling only. The harness owns
authority, decisions, contained acts, receipt signing, receipt tree
verification, and publication of proof.

## Context

CWD: `.`

Packages:
- `@runxhq/adapters` a2a subpath
- `@runxhq/runtime-local` harness a2a fixture path
- `crates/runx-runtime`
- `crates/runx-contracts`
- `crates/runx-core`
- `crates/runx-receipts`

Current TypeScript sources:
- `packages/adapters/src/a2a/index.ts`
- `packages/adapters/src/a2a/index.test.ts`
- `packages/runtime-local/src/harness/a2a-fixture.ts`
- `packages/core/src/parser/index.ts`
- `packages/core/src/policy/index.ts`

Files impacted:
- `crates/runx-runtime/src/adapters/a2a.rs`
- `crates/runx-runtime/src/adapters/a2a/fixture.rs`
- `crates/runx-runtime/tests/a2a_parity.rs`
- `fixtures/runtime/adapters/a2a/**`
- `scripts/generate-a2a-adapter-fixtures.ts`

Contract surfaces consumed:
- `runx-contracts::ActAssignment`
- `runx-contracts::HarnessReceipt`
- `runx-contracts::Reference`
- `runx-core::policy`

Invariants:
- A2A never invents its own authority model. Child dispatch must pass through
  runtime admission and policy checks, and child authority must be a checked
  subset of the parent harness authority.
- The adapter requires an explicit transport. Fixture transport is allowed only
  in tests and harness replay.
- No live cross-network calls are permitted in acceptance tests.
- Task id derivation, polling, timeout, cancellation, and failure messages must
  match the TS fixture oracle after deterministic normalization.
- Parent/child receipt linkage is expressed through harness receipt refs. The
  adapter may report task metadata, but it does not sign or validate receipt
  trees itself.
- Unsupported target status values fail closed with stable diagnostics.
- No new schema aliases or alternate contract families are introduced.

## Objectives

- Port A2A source parsing and dispatch to Rust.
- Implement deterministic fixture transport matching the TS harness fixture.
- Preserve success, failed, canceled, missing-task, timeout, abort, and cancel
  failure behavior.
- Preserve argument-template mapping for raw and resolved inputs.
- Attach source task metadata needed by the sealing harness without leaking
  absolute paths, secrets, or raw provider credentials.
- Prove parent/child harness receipt linkage when A2A dispatch spawns a child
  harness.

## Scope

In scope:
- `a2a` feature-gated runtime adapter.
- Explicit transport trait and fixture transport.
- Argument mapping and output serialization parity.
- Timeout, abort, polling, and cancellation behavior.
- Harness replay fixtures for trusted, semi-trusted, untrusted, failed, and
  timed-out targets.

Out of scope:
- New cross-org trust models.
- Hosted A2A service routing.
- Live network transport acceptance.
- Registry acquisition flows.
- Cloud API changes.
- Any second contract reader path.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-runtime-adapters-agent`.
- `runx-contract-spine-hard-cutover`.
- `rust-receipts-parity` completed against harness receipts.
- `rust-receipt-proof-verification` and `rust-receipt-tree-resolution` before
  this spec can claim cutover evidence.

## Sequencing Notes

- A2A runs after the agent adapter so the runtime has a stable child-agent
  source boundary before cross-agent dispatch is added.
- A2A can land before MCP because its fixture transport is small and explicit.
- Live transport may be introduced by a later hosted or integration spec only
  after local fixture parity is complete.

## Acceptance

Profile: strict

Validation:
- [ ] `cmd_fixture_oracle` - A2A adapter fixtures are current.
  - Command: `pnpm tsx scripts/generate-a2a-adapter-fixtures.ts --check`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_ts_a2a_adapter` - Existing TypeScript A2A adapter behavior still
  passes.
  - Command: `pnpm test -- packages/adapters/src/a2a/index.test.ts`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_runtime_a2a` - Rust A2A parity tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features a2a,agent --test a2a_parity`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_policy` - Policy tests still cover local A2A admission.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_receipts` - Receipt proof and tree checks pass for child harness
  refs used by A2A fixtures.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_fmt` - Rust formatting passes.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_clippy` - Rust linting passes for the touched crates.
  - Command: `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime -p runx-core --all-targets --features a2a,agent -- -D warnings`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_no_cutover_drift` - Touched Rust code and generated fixtures keep
  the post-cutover vocabulary and do not add schema aliases.
  - Command: `rg -n "schema ali[a]s(es)?|dual rea[d]er|alternate receipt fami[l]y|standalone act reco[r]d" crates/runx-runtime/src/adapters/a2a.rs crates/runx-runtime/src/adapters/a2a crates/runx-runtime/tests/a2a_parity.rs fixtures/runtime/adapters/a2a && exit 1 || exit 0`
  - Expected kind: `exit_code_zero`

Definition of done:
- [ ] `dod1` A2A source metadata validation rejects missing agent card URL,
  missing task, unknown source fields, and unsupported task status with stable
  diagnostics.
- [ ] `dod2` The Rust fixture transport returns the same deterministic task ids
  and task output shape as the TS fixture transport.
- [ ] `dod3` Argument-template mapping matches TS for exact template tokens,
  interpolated template tokens, missing values, and resolved inputs.
- [ ] `dod4` Timeout and abort paths attempt cancellation when available and
  preserve cancel failure metadata without hiding the original failure.
- [ ] `dod5` Successful child dispatch records child harness receipt refs for
  the parent harness seal.
- [ ] `dod6` Failed or canceled child dispatch closes the contained act with a
  non-success closure and does not publish proof as successful.
- [ ] `dod7` No live network calls, provider tokens, or real agent-card URLs are
  required for tests.

## Phases

### Phase 1 - Fixture oracle

Goal: capture current TypeScript A2A behavior in deterministic fixtures.

Tasks:
- Add `scripts/generate-a2a-adapter-fixtures.ts`.
- Generate cases for success, failure, canceled task, missing task, timeout,
  abort, cancel failure, raw argument mapping, resolved argument mapping, and
  unsupported non-fixture URL.
- Normalize ids, durations, timestamps, and temp paths.
- Store canonical inputs and adapter outputs under
  `fixtures/runtime/adapters/a2a/**`.

Exit criteria:
- Fixture generation is deterministic and `--check` fails on drift.

### Phase 2 - Rust transport trait

Goal: make transport explicit and testable.

Tasks:
- Define a narrow Rust transport trait for send, get, and cancel.
- Implement fixture transport with deterministic task id derivation.
- Reject construction without a transport.
- Add tests for send/get/cancel error paths.

Exit criteria:
- The adapter cannot accidentally perform live network work in tests.

### Phase 3 - Argument mapping and polling

Goal: match TS runtime behavior for inputs and task lifecycle.

Tasks:
- Port exact and interpolated template-token mapping.
- Preserve raw-input fallback behavior.
- Implement polling, timeout, abort, and cancellation.
- Preserve sanitized error messages.

Exit criteria:
- Rust parity tests pass for all non-receipt A2A fixture cases.

### Phase 4 - Harness linkage

Goal: make A2A proof part of the harness receipt tree.

Tasks:
- Route admitted child dispatch through runtime child-harness creation.
- Attach child harness receipt refs to the parent harness.
- Assert failed and canceled child tasks close the contained act without
  claiming successful proof.
- Verify parent/child receipts through `runx-receipts`.

Exit criteria:
- A2A fixtures prove both transport parity and receipt-tree linkage.

### Phase 5 - Verification

Goal: leave the adapter ready for later hosted transport work.

Tasks:
- Run all acceptance commands.
- Document unsupported live transport behavior as an explicit diagnostic.
- Confirm no code outside this spec's declared paths is required.

Exit criteria:
- All validation commands pass and unsupported live paths fail closed.

## Risks

- Medium: cross-agent dispatch can bypass the harness if implemented as a raw
  HTTP client. Mitigation: explicit transport plus runtime admission and child
  harness receipt refs.
- Medium: timeout and cancellation behavior can become flaky. Mitigation:
  deterministic fixture transport and normalized oracle fields.
- Medium: source metadata names differ across provider surfaces. Mitigation:
  parser parity and exact fixture assertions.

## Rollback

Strategy: per_phase

Commands:
- Revert only the A2A adapter files, generated fixtures, and fixture generator
  named in this spec.
- Re-run `pnpm tsx scripts/generate-a2a-adapter-fixtures.ts --check` if the
  generator remains.
- Re-run `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features a2a,agent --test a2a_parity` after rollback to confirm no partial adapter registration remains.

## Open Questions

- None for approval. Live transport is intentionally outside this spec.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T08:13:37Z
Ended: 2026-05-19T08:13:37Z

Checks:
- source audit
  - Result: passed
  - Evidence: The current TS adapter already requires an explicit transport and
    has deterministic fixture transport behavior.
- harness-spine audit
  - Result: passed
  - Evidence: The spec keeps authority and proof on harness receipt nodes and
    limits A2A to message/task transport.
- execution-readiness audit
  - Result: passed
  - Evidence: Open questions were closed, fixture generation was added, and
    acceptance commands are concrete.

Issues:
- none
