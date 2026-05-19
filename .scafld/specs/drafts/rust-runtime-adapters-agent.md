---
spec_version: '2.0'
task_id: rust-runtime-adapters-agent
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T08:13:37Z'
status: draft
harden_status: passed
size: large
risk_level: high
---

# Rust runtime agent adapter

## Current State

Status: draft
Current phase: none
Next: approve
Reason: hardened for execution after `rust-tool-catalogs`; boundary,
fixtures, receipt proof, and provider transport decisions are explicit.
Blockers: `rust-runtime-skeleton`, `rust-approval-gate-parity`,
`rust-tool-catalogs`, `runx-contract-spine-hard-cutover`, and
post-cutover receipt proof/tree APIs.
Allowed follow-up command: `scafld approve rust-runtime-adapters-agent`
Latest runner update: none
Review gate: not_started

## Summary

Port the `agent` and `agent-step` adapter family to `runx-runtime` behind the
`features = ["agent"]` flag. The adapter resolves a contained agent act inside
the current governed harness node: it builds and validates the
`AgentActInvocation`, runs the provider loop through an injected transport,
dispatches declared runx tools back through the runtime tool catalog, and
returns adapter output to the harness so the harness can seal the canonical
`runx.harness_receipt.v1` node.

The adapter does not sign receipts, own authority, create standalone act
records, or publish provider-specific contracts. It is one source-type adapter
inside a harness boundary.

## Context

CWD: `.`

Packages:
- `@runxhq/adapters` agent subpath
- `@runxhq/runtime-local` sdk caller and host-protocol path
- `crates/runx-runtime`
- `crates/runx-contracts`
- `crates/runx-receipts`

Current TypeScript sources:
- `packages/adapters/src/agent/index.ts`
- `packages/adapters/src/agent/agent-act-invocation.ts`
- `packages/adapters/src/agent/anthropic.ts`
- `packages/adapters/src/agent/openai.ts`
- `packages/adapters/src/agent/runtime-tools.ts`
- `packages/runtime-local/src/sdk/caller.ts`
- `packages/runtime-local/src/sdk/host-protocol.ts`
- `../cloud/packages/agent-runner/src/hosted-agent-adapter.ts`
- `../cloud/packages/agent-runner/src/durable-step.ts`
- `../cloud/packages/agent-runner/src/anthropic.ts`

Files impacted:
- `crates/runx-runtime/src/adapters/agent/mod.rs`
- `crates/runx-runtime/src/adapters/agent/anthropic.rs`
- `crates/runx-runtime/src/adapters/agent/openai.rs`
- `crates/runx-runtime/src/adapters/agent/host_protocol.rs`
- `crates/runx-runtime/src/adapters/agent/runtime_tools.rs`
- `crates/runx-runtime/tests/agent_parity.rs`
- `fixtures/runtime/adapters/agent/**`
- `scripts/generate-agent-adapter-fixtures.ts`

Contract surfaces consumed:
- `runx-contracts::AgentActInvocation`
- `runx-contracts::ResolutionRequest`
- `runx-contracts::ResolutionResponse`
- `runx-contracts::ActAssignment`
- `runx-contracts::HarnessReceipt`
- `runx-contracts::Reference`

Invariants:
- Host protocol types come from `runx-contracts`; the adapter does not
  redeclare or loosen them.
- `agent` and `agent-step` remain source-type names. They are adapter source
  types, not central domain objects.
- Agent acts are contained harness payloads. They are provable only through
  the sealing harness receipt.
- The provider loop receives only the attenuated authority, allowed tools,
  current context, historical context, and provenance supplied by the harness.
- Declared runx tool calls execute through the runtime tool catalog and return
  child harness receipt refs where a nested tool run occurs.
- Nested resolution requests pause with typed host-protocol data. The adapter
  must not fabricate success output to keep a provider loop moving.
- Provider transports are injected. Unit and parity tests use deterministic
  fixture transports; no live provider hits are permitted.
- Hosted durable-step behavior remains owned by the cloud cutover. This spec
  may mirror the boundary in fixtures, but it does not replace hosted Durable
  Objects.
- No new schema aliases or alternate contract families are introduced.

## Objectives

- Port managed agent execution for `agent` and `agent-step` sources.
- Validate `AgentActInvocation` input and `ResolutionResponse` output at the
  Rust boundary.
- Implement deterministic Anthropic and OpenAI provider fixture transports
  that exercise tool calls, final payloads, provider errors, and nested
  resolution pauses.
- Dispatch allowed runx tools through the post-`rust-tool-catalogs` catalog
  path and preserve returned receipt refs in adapter metadata for harness
  sealing.
- Preserve TS behavior for success, failure, policy-denied tool calls,
  needs-resolution tool calls, provider malformed output, and cancellation.
- Keep the adapter narrow: approvals, authority attenuation, receipt signing,
  and receipt tree verification remain harness/runtime responsibilities.

## Scope

In scope:
- Rust `agent` feature and adapter module.
- Anthropic-shaped message/tool protocol parity.
- OpenAI provider protocol parity when it can share the same transport seam and
  fixture oracle.
- Runtime-tool dispatch through the catalog adapter path.
- Fixture generation from TypeScript adapter behavior.
- Local harness replay coverage for agent and agent-step cases.

Out of scope:
- Live provider network calls.
- Hosted agent-runner Durable Object replacement.
- New provider routing policy.
- Cloud API changes.
- Additional public CLI flags.
- Any second contract reader path.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-approval-gate-parity`.
- `rust-tool-catalogs`.
- `runx-contract-spine-hard-cutover`.
- `rust-receipts-parity` completed against harness receipts.
- `rust-receipt-proof-verification` and `rust-receipt-tree-resolution` before
  this spec can claim cutover evidence.

## Sequencing Notes

- This spec runs after `rust-tool-catalogs` because managed agent tools depend
  on catalog resolution, local manifest precedence, and exact fixture oracles.
- Harness replay may use the agent fixture mode only after the adapter returns
  deterministic data and not-yet-supported provider modes fail closed.
- A hosted cutover may consume this module later, but this spec only proves the
  local runtime adapter and host-protocol boundary.

## Acceptance

Profile: strict

Validation:
- [ ] `cmd_fixture_oracle` - Agent adapter fixtures are current.
  - Command: `pnpm tsx scripts/generate-agent-adapter-fixtures.ts --check`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_ts_agent_adapter` - Existing TypeScript agent adapter behavior still
  passes.
  - Command: `pnpm test -- packages/adapters/src/agent/index.test.ts packages/adapters/src/runtime.test.ts`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_contracts_host_protocol` - Rust host protocol contracts validate the
  agent boundary.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts host_protocol`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_runtime_agent` - Rust agent parity tests pass with deterministic
  provider transports.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features agent,catalog,cli-tool --test agent_parity`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_receipts` - Nested tool-call receipts verify when the fixture emits
  child harness receipts.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_fmt` - Rust formatting passes.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_clippy` - Rust linting passes for the touched crates.
  - Command: `cargo clippy --manifest-path crates/Cargo.toml -p runx-contracts -p runx-runtime --all-targets --features agent,catalog,cli-tool -- -D warnings`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_no_cutover_drift` - Touched Rust code and generated fixtures keep
  the post-cutover vocabulary and do not add schema aliases.
  - Command: `rg -n "schema ali[a]s(es)?|dual rea[d]er|alternate receipt fami[l]y|standalone act reco[r]d" crates/runx-runtime/src/adapters/agent crates/runx-runtime/tests/agent_parity.rs fixtures/runtime/adapters/agent && exit 1 || exit 0`
  - Expected kind: `exit_code_zero`

Definition of done:
- [ ] `dod1` `agent` and `agent-step` source types run through one Rust adapter
  family with separate source-type metadata and shared provider transport
  plumbing.
- [ ] `dod2` `AgentActInvocation` and `ResolutionResponse` validation rejects
  unknown fields and malformed provider payloads with stable diagnostics.
- [ ] `dod3` Allowed tools are resolved through the runtime catalog path, not a
  provider-local registry.
- [ ] `dod4` Nested runx tool calls preserve child harness receipt refs for the
  sealing harness.
- [ ] `dod5` Provider failure, malformed final payload, timeout, cancellation,
  and nested resolution pause all produce deterministic adapter output.
- [ ] `dod6` The adapter never signs receipts or emits standalone act proof.
  Harness sealing remains the only proof boundary.
- [ ] `dod7` No live network key or provider token is required for tests.

## Phases

### Phase 1 - Fixture oracle

Goal: capture the current TypeScript boundary with deterministic provider
fixtures.

Tasks:
- Add `scripts/generate-agent-adapter-fixtures.ts`.
- Generate cases for agent success, agent-step success, provider error,
  malformed final payload, allowed tool success, allowed tool failure,
  allowed tool needs-resolution, cancellation, and provider timeout.
- Store canonical input, provider transcript, adapter output, and expected
  harness metadata under `fixtures/runtime/adapters/agent/**`.
- Normalize ids, durations, timestamps, and temp paths.

Exit criteria:
- Fixture generation is deterministic and `--check` fails on drift.

### Phase 2 - Contract boundary

Goal: make the Rust boundary typed before implementing provider behavior.

Tasks:
- Add Rust request/response adapters that convert runtime source metadata into
  `AgentActInvocation`.
- Validate incoming fixture input through `runx-contracts`.
- Validate provider final payloads through `ResolutionResponse`.
- Add negative tests for unknown fields, missing provider final payload, and
  invalid source type.

Exit criteria:
- Boundary tests fail before provider transport is called when the contract is
  malformed.

### Phase 3 - Provider transports

Goal: support deterministic provider loops without live network calls.

Tasks:
- Implement injected Anthropic-shaped transport.
- Implement OpenAI provider transport if the TS oracle shows behavior not
  already covered by the Anthropic fixture shape.
- Map provider tool calls to runtime tool invocations and final payloads.
- Preserve provider telemetry needed by harness receipts without leaking keys,
  absolute paths, or raw secrets.

Exit criteria:
- The Rust adapter passes provider-loop success and failure fixtures with exact
  canonical output.

### Phase 4 - Runtime tool dispatch

Goal: connect managed agent tool calls to the runtime catalog.

Tasks:
- Resolve allowed tools through the catalog path from `rust-tool-catalogs`.
- Execute nested tools through runtime adapters when supported.
- Return structured not-yet-supported diagnostics for tool source types outside
  the enabled feature set.
- Preserve child harness receipt refs in adapter metadata for the parent
  harness seal.

Exit criteria:
- Tool round-trip fixtures prove success, failure, policy denial, and
  needs-resolution paths.

### Phase 5 - Harness receipt integration

Goal: prove the adapter fits the harness spine.

Tasks:
- Run agent fixtures through harness replay mode.
- Assert contained decisions and acts remain inside the harness receipt.
- Assert child harness receipt refs are present for nested tool runs.
- Verify generated receipts through `runx-receipts`.

Exit criteria:
- Adapter fixtures can be used by `rust-harness` without special-case
  receipt logic.

## Risks

- High: provider loops can hide boundary drift behind free-form model text.
  Mitigation: final payload validation and fixture transports are mandatory.
- High: nested tools can accidentally bypass authority attenuation. Mitigation:
  only the harness/runtime may invoke child harnesses, and every nested run must
  return a child harness receipt ref.
- Medium: provider APIs differ. Mitigation: keep provider-specific mapping
  behind injected transports and normalize only the runx boundary.
- Medium: hosted durable-step semantics can be mistaken for local runtime
  scope. Mitigation: hosted replacement is out of scope and explicitly covered
  by cloud cutover specs.

## Rollback

Strategy: per_phase

Commands:
- Revert only the agent adapter files, generated fixtures, and fixture
  generator named in this spec.
- Re-run `pnpm tsx scripts/generate-agent-adapter-fixtures.ts --check` if the
  generator remains.
- Re-run `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features agent,catalog,cli-tool --test agent_parity` after rollback to confirm no partial adapter registration remains.

## Open Questions

- None for approval. Provider transports are injected; live HTTP is outside
  this spec.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T08:13:37Z
Ended: 2026-05-19T08:13:37Z

Checks:
- source audit
  - Result: passed
  - Evidence: The current TS adapter uses `AgentActInvocation`,
    provider-specific modules, and runtime tool dispatch through the catalog
    path.
- harness-spine audit
  - Result: passed
  - Evidence: The spec keeps proof on harness receipts and keeps acts contained
    in the harness.
- execution-readiness audit
  - Result: passed
  - Evidence: Open questions were closed, deterministic fixture generation was
    added, and acceptance commands are concrete.

Issues:
- none
