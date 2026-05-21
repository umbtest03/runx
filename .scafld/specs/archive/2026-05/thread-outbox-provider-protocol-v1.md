---
spec_version: '2.0'
task_id: thread-outbox-provider-protocol-v1
created: '2026-05-22T01:16:00+10:00'
updated: '2026-05-21T17:28:27Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# Thread outbox provider protocol v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T17:28:27Z
Review gate: pass

## Summary

Define the language-neutral provider protocol for pushing thread outbox entries
to systems such as GitHub issues/PRs, Slack threads, or support-channel
surfaces. This lane owns provider-side publication and readback for outbox
entries; it is not an execution-adapter protocol and not a TypeScript runtime
fallback.

Provider adapters that require tokens must consume
`credential-broker-delivery-contract-v1` through Rust-supervised
`CredentialDelivery`. Public frames may carry credential refs, profile ids,
provider, purpose, delivery mode, and material-ref hashes only. Raw secret
material must not appear in outbox entries, thread state, receipts, logs,
provider observations, or adapter responses.

## Context

- `docs/ts-interop-boundary.md` already lists thread/outbox provider adapters as
  a separate language-neutral extension lane.
- `packages/core/src/knowledge/file-thread.ts` implements only the local file
  thread adapter. It is credential-free and should stay a helper/read-model
  path, not become a provider mutation runtime.
- `external-adapter-plugin-protocol-v1` explicitly excludes thread/outbox
  provider queues.
- `credential-broker-delivery-contract-v1` needs outbox/provider specs either
  to consume its primitive or to name provider credentials as a blocker.

## Objectives

- Specify provider outbox adapter manifest and invocation frames for push,
  fetch/readback, dedupe, and idempotent retry.
- Specify credential needs using `credential-broker-delivery-contract-v1`; no
  provider adapter may define a private secret channel.
- Specify receipt/readback observations for published messages/comments/PR
  updates without leaking raw provider payloads or secrets.
- Preserve source-thread routing: missing recoverable thread targets must fail
  closed before provider mutation.
- Keep the local file-thread adapter credential-free and explicit.

## Scope

In scope:
- Provider push/fetch frames for thread outbox entries.
- Credential delivery profile references and Rust-supervised process/env
  delivery for provider adapters.
- Idempotency keys, provider locator readback, and receipt-safe metadata.
- Negative tests proving `@runxhq/core` and package helpers do not mutate real
  providers without this protocol.

Out of scope:
- General execution adapters, owned by `external-adapter-plugin-protocol-v1`.
- Source-event ingress and webhook admission.
- OAuth/BYO storage lifecycle, owned by credential/connect specs.
- Merging pull requests or deciding policy admission.
- Local file-thread persistence beyond the current helper behavior.

## Dependencies

- `credential-broker-delivery-contract-v1`
- `ts-extension-survivorship-boundary`
- `rust-ts-sunset-runtime-local`
- `runx-target-repo-runners`
- `runx-post-merge-closure-observer`

## Touchpoints

- `packages/core/src/knowledge/file-thread.ts`
- `packages/core/src/knowledge/thread.ts`
- `docs/ts-interop-boundary.md`
- `docs/thread-story-contract.md`
- `crates/runx-contracts/src/thread_outbox_provider.rs`
- future `crates/runx-runtime/src/outbox_provider.rs` or equivalent provider
  supervisor module
- future Rust runtime provider adapter CLI command surface

## Risks

- Reusing the external execution-adapter protocol would blur execution and
  publication semantics and recreate a catch-all plugin API.
- Letting TypeScript helpers perform provider mutations would keep a hidden
  trusted runtime alive after the Rust cutover.
- Provider retries without idempotency/readback can duplicate comments, PR
  updates, or source-thread replies.
- Credential material can leak if provider observations are recorded before
  redaction and receipt shaping.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` The outbox/provider lane has an owning spec distinct from the
  external execution-adapter protocol.
- [x] `dod2` Existing local file-thread outbox push explicitly skips provider
  adapters and names this protocol plus Rust-supervised `CredentialDelivery` as
  the blocker.
- [x] `dod3` Provider push/fetch/readback frames are defined in
  `runx-contracts` and generated TypeScript contracts.
- [x] `dod4` Provider adapters consume `credential-broker-delivery-contract-v1`
  and reject private secret fields.
- [x] `dod5` Idempotent provider mutation and readback receipt tests exist for
  at least one provider fixture.
- [x] `dod6` Runtime-local/adapters sunset can point provider outbox work at
  this protocol without preserving a TypeScript mutation fallback.

Validation:
- [x] `v1` Local knowledge tests prove provider outbox push is skipped, not
  silently handled by TS core.
  - Command:
    `pnpm vitest run packages/core/src/knowledge/index.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T01:17:00+10:00 passed 20 tests, including
    `skips push when no runtime adapter is registered`, whose reason now names
    `thread-outbox-provider-protocol-v1`/Rust-supervised `CredentialDelivery`
    as the provider mutation blocker.
- [x] `v2` Scafld validates this spec.
  - Command: `scafld validate thread-outbox-provider-protocol-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:26:40+10:00 returned
    `{"ok":true,"command":"validate","result":{"task_id":"thread-outbox-provider-protocol-v1","path":"/Users/kam/dev/runx/runx/oss/.scafld/specs/active/thread-outbox-provider-protocol-v1.md","valid":true,"errors":null}}`.
- [x] `v3` TypeScript contracts validate provider frames and negative rules.
  - Command:
    `pnpm vitest run packages/contracts/src/schemas/thread-outbox-provider.test.ts packages/contracts/src/index.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:25:21+10:00 passed 24 tests across the provider
    schema and contracts index suites.
- [x] `v4` Generated JSON Schema artifacts are fresh.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:25:21+10:00 exited zero after generating no diff.
- [x] `v5` Rust contract fixtures roundtrip and reject unsafe inputs.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test thread_outbox_provider_fixtures -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:25:21+10:00 passed 5 tests, including secret-field,
    missing-thread-locator, missing-fetch-target, and HTTP-transport rejection.
- [x] `v6` Rust fixture-to-generated-schema validation includes provider frames.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test schema_validation -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:25:21+10:00 passed 5 tests and mapped all
    `fixtures/contracts/thread-outbox-provider/*.json` payloads to generated
    provider schemas.
- [x] `v7` Rust runtime supervisor invokes the provider fixture and proves
  credential/redaction/idempotency/readback behavior.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test thread_outbox_provider -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T03:45:00+10:00 passed 6 tests covering supervisor
    invocation, process-only transport rejection, secret-field rejection,
    credential redaction in stderr and observation errors, idempotent push
    created/replayed states, delivery-observation injection, and fetch/readback
    receipt shaping.
- [x] `v8` Provider runtime lane has no TypeScript runtime-local/adapters
  fallback import.
  - Command:
    `! rg -n "@runxhq/(core|runtime-local|adapters)|packages/(runtime-local|adapters)" crates/runx-runtime/src/outbox_provider.rs crates/runx-runtime/tests/thread_outbox_provider.rs fixtures/runtime/thread-outbox-provider`
  - Expected kind: `no_matches`
  - Status: passed
  - Evidence: 2026-05-22T03:26:51+10:00 returned no matches.

## Phase 1: Boundary

Status: completed
Dependencies: `ts-extension-survivorship-boundary`

Changes:
- [x] Name thread/outbox provider adapters as a separate protocol lane.
- [x] Keep file-thread local and credential-free.
- [x] Fail closed for provider adapter types until this protocol exists.

## Phase 2: Contracts

Status: completed
Dependencies: Phase 1, `credential-broker-delivery-contract-v1`

Changes:
- Define push/fetch/readback frames.
- Define credential profile references and delivery observations.
- Define idempotency/readback receipt metadata.

### Phase 2A: Contract-Only Frames

Status: completed
Dependencies: Phase 1, `credential-broker-delivery-contract-v1`

Buildable touchpoints:
- `packages/contracts/src/schemas/thread-outbox-provider.ts`
- `packages/contracts/src/index.ts`
- `packages/contracts/src/internal.ts`
- generated `schemas/thread-outbox-provider-*.schema.json`
- `fixtures/contracts/thread-outbox-provider/*.json`
- `crates/runx-contracts/src/thread_outbox_provider.rs`
- `crates/runx-contracts/src/lib.rs`
- Rust and TypeScript fixture/schema validation tests

Contract slice:
- `runx.thread_outbox_provider.manifest.v1`: adapter id, provider, supported
  operations, protocol version, process-only transport, declared credential
  needs, and receipt/redaction capabilities.
- `runx.thread_outbox_provider.push.v1`: outbox entry id, thread locator,
  idempotency key, rendered payload, provider profile, credential delivery refs,
  and receipt-safe context.
- `runx.thread_outbox_provider.fetch.v1`: provider locator or thread locator,
  readback cursor, idempotency key, provider profile, and credential delivery
  refs.
- `runx.thread_outbox_provider.observation.v1`: accepted/skipped/failed status,
  provider locator, stable provider event id/hash, readback summary,
  idempotency result, delivery observations, redaction metadata, and safe error.

Negative contract rules:
- Public frames must not accept fields named like `token`, `access_token`,
  `api_key`, `secret`, `password`, or `authorization`.
- Provider observations may include hashes, refs, delivery modes, and redaction
  flags only; never raw credential material or unbounded provider response
  bodies.
- A missing thread locator is a fail-closed input error, not permission to
  publish into a fallback root channel/thread.
- HTTP transport is rejected in v1; a future contract must define HTTP auth,
  retry, idempotency, and secret-delivery semantics before it is allowed.

## Phase 3: Provider Fixture

Status: completed
Dependencies: Phase 2

Objective: Implement one Rust-supervised provider process fixture without live
provider network mutation.

Changes:
- Add a narrow Rust `ThreadOutboxProviderProcessSupervisor` that is not a `SkillAdapter` and is not `ExternalAdapterProcessSupervisor`.
- Invoke provider adapters with a strict process protocol:
  - stdin receives exactly one `ThreadOutboxProviderPush` or
    `ThreadOutboxProviderFetch` JSON frame.
  - stdout emits exactly one `ThreadOutboxProviderObservation` JSON frame.
  - stderr is diagnostic only and must be redacted before any receipt projection.
- Define timeout, output-size, cwd, command/args, and cancellation defaults in the supervisor rather than leaving provider fixtures to invent them.
- Deliver credentials only through the shared Rust `CredentialDelivery`
  primitive, using the declared environment delivery contract and the runtime
  allowlist.
- Validate adapter id, provider, operation, request id, protocol version, and
  manifest-supported operation before accepting output.
- Reject raw secret-like fields and redact credential material from stdout, stderr, metadata, errors, and observations before any receipt projection.
- Add one provider fixture adapter under Rust supervision.
- Prove idempotent push and readback receipt shaping.
- Prove secrets are redacted from observations and receipts.
- Buildable write set:
  - [x] `crates/runx-runtime/src/outbox_provider.rs`
  - [x] `crates/runx-runtime/src/lib.rs`
  - [x] `crates/runx-runtime/tests/thread_outbox_provider.rs`
  - [x] `fixtures/runtime/thread-outbox-provider/mock-provider.sh`
  - [x] this spec evidence fields
- Non-goals:
  - No live GitHub, Slack, or support-channel network mutation.
  - No reuse of `external-adapter-plugin-protocol-v1` as a provider queue.
  - No `@runxhq/core`, `@runxhq/runtime-local`, or `@runxhq/adapters`
    provider mutation fallback.

Acceptance:
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test thread_outbox_provider -- --nocapture`
  exits zero and covers supervisor invocation, HTTP-transport rejection,
  secret-field rejection, credential redaction in stdout/stderr/observations,
  idempotent push, and readback receipt shaping.
- [x] The Rust source, test, and fixture paths named in the buildable write set
  exist, and the supervisor is exported or otherwise reachable from
  `crates/runx-runtime/src/lib.rs`.
- [x] Strict DoD items `dod4`, `dod5`, and `dod6` are checked only after that
  validation exists.

## Rollback

If provider outbox semantics cannot be made language-neutral, keep provider
mutation blocked. Do not route provider outbox writes through
`external-adapter-plugin-protocol-v1` and do not revive `@runxhq/runtime-local`
or `@runxhq/adapters` as a trusted provider mutation fallback.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: command
Output: command.stdout
Summary: Command-provider verification pass. Rechecked thread-outbox-provider-protocol-v1 after Phase 3 implementation: the dedicated Rust ThreadOutboxProviderProcessSupervisor exists under runx-runtime, is exported from lib.rs, is not a SkillAdapter or ExternalAdapterProcessSupervisor, invokes process-only provider fixtures over stdin/stdout, injects CredentialDelivery env values, rejects HTTP endpoint transport and secret-like response fields, redacts credential material from stderr and parsed observations, and validates adapter/provider/operation/request identity. Runtime fixture tests prove idempotent push created/replayed states and fetch/readback receipt shaping. Strict DoD dod4-dod6 and v7 evidence are now checked. No completion blockers found.

Attack log:
- `Phase 3 source paths`: verify crates/runx-runtime/src/outbox_provider.rs, lib export, runtime test, and fixtures/runtime/thread-outbox-provider/mock-provider.sh exist -> clean
- `process supervisor separation`: inspect that ThreadOutboxProviderProcessSupervisor is a dedicated module and not a SkillAdapter or ExternalAdapterProcessSupervisor reuse -> clean
- `credential delivery`: run runtime fixture with CredentialDelivery env injection and public observation insertion -> clean
- `secret-field rejection`: run fixture that emits access_token in observation and expect SecretFieldRejected -> clean
- `redaction`: run fixture that leaks credential in stderr and observation error text and assert redacted output -> clean
- `idempotency`: run push fixture for created and replayed idempotency states -> clean
- `readback`: run fetch fixture and assert readback summary plus provider event hash -> clean
- `transport boundary`: mutate manifest endpoint to HTTP-like transport and expect UnsupportedTransport -> clean

Findings:
- none

## Origin

User architecture review on 2026-05-22: external execution adapters, source
ingress, hosted runtime binding, catalog/read-model access, and thread/outbox
provider queues are separate lanes. The outbox/provider credential blocker
needed an owning spec so the credential broker contract could stop treating it
as an unnamed future gap.
