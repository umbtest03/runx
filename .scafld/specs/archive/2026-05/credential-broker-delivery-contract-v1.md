---
spec_version: '2.0'
task_id: credential-broker-delivery-contract-v1
created: '2026-05-22T00:28:36+10:00'
updated: '2026-05-21T15:47:18Z'
status: completed
harden_status: passed
size: large
risk_level: high
---

# Credential broker and delivery contract v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T15:47:18Z
Review gate: pass

## Summary

Define the shared credential broker and delivery contract used by Rust-supervised
execution lanes. The contract starts after policy admission and authority proof:
Rust decides whether a grant may bind to a run, resolves the opaque
`material_ref` through a trusted broker/resolver, maps the material to a
declared delivery profile, and gives the child or provider adapter only the
scoped secret material it is allowed to receive.

This is not a general auth plugin protocol and not a secret store. It is the
runtime handoff primitive between admitted authority and a supervised side-effect
boundary. Raw secret material must never appear in manifests, invocation frames,
authority proofs, receipts, logs, captured output, public projections, or
adapter observations.

The completed `rust-adapter-credential-delivery` spec proved the first local
mechanics: `CredentialDelivery`, `MaterialResolver`, provider env profiles,
`SecretEnv`, and redaction for built-in `cli-tool` and MCP paths. This spec
turns that implementation idea into a stable contract that can be reused by:

- skill author subprocess execution through the existing `CredentialDelivery`
  channel;
- external execution adapters under `external-adapter-plugin-protocol-v1`;
- thread/outbox provider adapters that need provider tokens for comments, PR
  updates, or publication;
- hosted/cloud runtimes that broker secrets outside the local process.

## Context

Existing implemented Rust pieces:
- `crates/runx-runtime/src/credentials.rs` defines `CredentialDeliveryProfile`,
  `MaterialResolver`, `ResolvedCredentialMaterial`, `SecretEnv`, and
  `CredentialDelivery`.
- `crates/runx-runtime/src/adapters/cli_tool.rs` injects
  `CredentialDelivery.secret_env()` only at child process spawn and redacts
  captured stdout/stderr.
- `crates/runx-runtime/src/adapters/mcp/**` passes `SecretEnv` to MCP process
  spawn and redacts tool results.

Existing contract surfaces:
- `packages/contracts/src/schemas/credentials.ts` owns credential envelope and
  authority-proof shapes without raw material.
- `crates/runx-contracts/src/external_adapter.rs` has credential references and
  credential request frames, but no host-to-adapter delivery frame or delivery
  mode.
- `docs/security-authority-proof.md` bans raw tokens and records only
  `material_ref` hashes in public proof.

## Objectives

- Define a stable credential-delivery frame/envelope family in
  `runx-contracts` and `@runxhq/contracts`.
- Define delivery modes for v1. The default should be process environment
  injection because Rust already implements that safely for built-in adapters.
  File/socket/helper-process delivery remains future work unless a v1 consumer
  proves it is required.
- Define a broker response that carries only delivery handles or scoped secret
  material over trusted host-to-supervisor channels. Adapters must not request
  arbitrary credentials at runtime.
- Define delivery profiles: provider, auth mode, purpose, material roles, target
  env/file names, required/optional semantics, and redaction hints.
- Define redaction and non-leakage rules shared by cli-tool, MCP, external
  execution adapters, and outbox/provider adapters.
- Define receipt/proof observations: material is omitted; receipts may record
  credential refs, grant refs, provider, purpose, profile id, delivery mode, and
  material ref hash only.

## Scope

In scope:
- Contract schemas and Rust/TypeScript types for credential delivery requests,
  broker responses, delivery profiles, and redaction policy.
- Runtime mapping from admitted credential envelope + binding decision +
  material resolver to a delivery object.
- External execution-adapter host-to-process delivery semantics.
- Outbox/provider adapter delivery requirements where provider tokens are needed
  for side effects.
- Tests proving secrets do not enter receipts, authority proofs, stdout/stderr,
  metadata, or external adapter response observations.

Out of scope:
- Secret storage implementation, OAuth handshakes, hosted grant lifecycle, and
  BYO credential verification; owned by `byo-credential-foundations` and cloud
  auth specs.
- Provider-specific SDK packages.
- Source-event ingress protocol design.
- Replacing the existing authority proof or credential envelope shape.
- General-purpose auth resolver plugins.

## Dependencies

- `rust-adapter-credential-delivery` archived completed; provides the current
  Rust local implementation mechanics and non-leakage tests.
- `byo-credential-foundations`; owns storage, verification, and hosted material
  availability.
- `external-adapter-plugin-protocol-v1`; consumes this contract for external
  execution-adapter delivery and must not invent a separate credential channel.
- `skill-author-runtime-contract-v1`; the author-facing subprocess ABI must stay
  compatible with the same delivery primitive.
- `github-outbox-receipts`; outbox/provider side effects must use this primitive
  or a named blocker if provider credentials are required.
- `security-authority-proof.md`; public proof remains metadata-only and
  secret-free.

## Touchpoints

- `crates/runx-contracts/src/`
- `packages/contracts/src/schemas/`
- `schemas/credential-*.schema.json`
- `fixtures/contracts/credential-delivery/`
- `crates/runx-runtime/src/credentials.rs`
- `crates/runx-runtime/src/adapters/cli_tool.rs`
- `crates/runx-runtime/src/adapters/mcp/`
- `crates/runx-runtime/src/adapters/external_adapter.rs`
- `docs/security-authority-proof.md`
- `oss/.scafld/specs/active/external-adapter-plugin-protocol-v1.md`

## Risks

- If delivery is left per-protocol, cli-tool, external adapters, hosted runtimes,
  and outbox providers will each grow incompatible secret channels.
- If the contract is too rich, it becomes an auth plugin system or secret store.
- If v1 allows HTTP delivery without auth, retry, and idempotency semantics, it
  can leak secrets or double-use scoped credentials.
- If redaction is exact-string only, transformed or boundary-split secrets can
  leak through captured output. The contract must specify the minimum v1
  guarantee and the limits of that guarantee honestly.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Contract schemas exist for delivery profile, delivery request,
  broker response, and runtime delivery observation.
- [x] `dod2` Rust and TypeScript contract fixtures cross-validate and reject raw
  secret material in public frames.
- [x] `dod3` Runtime delivery uses the contract for cli-tool/MCP without
  weakening the existing `CredentialDelivery` secret channel.
- [x] `dod4` External execution-adapter Phase 2 wiring consumes this contract
  instead of accepting arbitrary credential-request frames from the adapter.
- [x] `dod5` Outbox/provider adapter specs either consume this contract or mark
  provider credentials as an explicit blocker.
- [x] `dod6` Redaction tests cover stdout, stderr, metadata, response
  observations, receipt metadata, and truncation boundaries.

Validation:
- [ ] `v1` Scafld validates this spec.
  - Command: `scafld validate credential-broker-delivery-contract-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T01:18:00+10:00 returned
    `{"ok":true,"command":"validate","result":{"task_id":"credential-broker-delivery-contract-v1","path":"/Users/kam/dev/runx/runx/oss/.scafld/specs/active/credential-broker-delivery-contract-v1.md","valid":true,"errors":null}}`.
- [ ] `v2` Contract schema generation is fresh.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:43:33+10:00 exited 0 after generating the four
    credential-delivery schemas.
- [ ] `v3` Rust contract fixtures pass.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-contracts credential_delivery`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:43:54+10:00 focused fixture command passed
    `credential_delivery_fixtures` and `schema_validation`.
- [ ] `v4` Runtime credential delivery tests pass.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test credential_delivery --features cli-tool,mcp -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:52:00+10:00 passed 9 focused tests, including
    contract-profile mapping, unsupported role rejection, empty material
    rejection, redact-before-truncate behavior, and MCP real process transport
    delivery/redaction.
- [x] `v5` External adapter supervisor tests prove delivery or fail closed.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter,cli-tool external_adapter`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T01:19:54+10:00 passed the full filtered runtime
    command; the focused external-adapter file also passed 16 tests, including
    `external_adapter_process_supervisor_delivers_credentials_and_redacts_observations`,
    `external_adapter_skill_adapter_projects_public_credential_refs_and_observation`,
    `external_adapter_process_supervisor_rejects_unexpected_credential_request`,
    and graph-level host-resolution routing. The adapter still rejects
    adapter-originated credential request frames; raw material is delivered only
    through the private `CredentialDelivery` env channel.

## Phase 1: Contract Shape

Status: completed
Dependencies: rust-adapter-credential-delivery

Objective: Complete this phase.

Changes:
- [x] Add contract types for:
- [x] Declare v1 process-env delivery mode and reserve future modes explicitly.
- [x] Declare that public frames carry refs, hashes, provider, purpose, delivery mode, and profile ids only. Raw secret material is private to the trusted broker/supervisor channel.

Acceptance:
- none

## Phase 2: Runtime Adoption

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- [x] Map `CredentialDeliveryProfile` and `CredentialDelivery` to/from the contract where appropriate without adding serialization to secret-bearing Rust types.
- [x] Reject empty material values before env injection.
- [x] Redact before truncation for cli-tool process output.
- [x] Add an MCP real-spawn credential delivery integration test, not only fixture transport coverage.

Acceptance:
- none

## Phase 3: External Adapter And Outbox Consumption

Status: completed
Dependencies: Phase 2, `external-adapter-plugin-protocol-v1`

Objective: Complete this phase.

Changes:
- [x] Replace or narrow external adapter credential-request handling so adapters receive only host-delivered credential refs/handles/material through the approved delivery mode.
- [x] External adapter supervisor injects process-env delivery after audited scoped env and redacts stdout/stderr/response observations.
- [x] Outbox/provider adapter specs declare whether provider credentials are delivered through this primitive or are not in scope.
- `thread-outbox-provider-protocol-v1` now owns the provider outbox lane, blocks provider mutations until `CredentialDelivery` is consumed, and keeps the current local file-thread helper credential-free.

Acceptance:
- none

## Rollback

If the shared contract cannot cover a consumer, keep that consumer blocked or
give it a named sibling credential-delivery extension. Do not smuggle raw secret
material through existing invocation metadata, adapter env fields, receipts, or
provider-specific JSON blobs.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Discover-mode review of credential-broker-delivery-contract-v1 against the current task-scoped change to `crates/runx-runtime/src/credentials.rs` and the surrounding scope (cli-tool adapter, MCP adapter, external adapter supervisor, contract types, JSON schemas). Both prior-review findings are addressed: (F1) `CredentialDeliveryEnvBinding.required` is now propagated through `from_contract_profile` (credentials.rs:64) and honored by `apply_profile` (credentials.rs:341-343), with two unit tests covering the required-fails-closed and optional-skipped paths; (F2) `collect_redacted_output` (cli_tool.rs:90-94) now drops the entire captured stdout/stderr when `truncated == true`, so no pre-redaction prefix can ever escape the 1 MiB capture boundary, and `cli_tool_drains_large_stdout_and_omits_truncated_output` enforces that contract. Public frames in `runx_contracts::credential_delivery` and `schemas/credential-delivery-*.schema.json` carry only refs, hashes, profile ids, env-binding metadata, and timestamps with `additionalProperties: false` / `deny_unknown_fields`; `SecretString` debug-prints `[redacted-credential]` and never derives `Serialize`. The cli-tool, MCP, and external adapter supervisors inject `CredentialDelivery.secret_env()` last so credential env wins over scoped/receipt env, and the external supervisor still routes `CREDENTIAL_REQUEST_SCHEMA` to `UnexpectedCredentialRequest` (external_adapter.rs:855-868) with response/observation redaction. No new completion blockers found. The new silent-drop behavior at credentials.rs:56-58 (unsupported optional role -> skip) is intentional, contract-consistent ("optional means may be absent"), and explicitly covered by `delivery_profile_skips_optional_unsupported_contract_binding`; flagged in attack log as clean. Ambient drift (`tests/cli_tool_contract.rs`) is unrelated cli-tool sandbox/timeout coverage and does not change credential-delivery semantics.

Attack log:
- `credentials.rs:from_contract_profile and apply_profile`: Re-verify prior F1 fix: confirm `CredentialDeliveryEnvBinding.required` is now read end-to-end, optional bindings skip cleanly, required bindings fail closed with MissingRole. -> clean (credentials.rs:64 propagates `required: binding.required`; apply_profile:340-347 skips when !required and material role absent, errors with MissingRole when required. tests/credential_delivery.rs:101-131 covers optional-skip, credentials.rs:411-433 covers required-fail-closed. The contract schema still requires `required: bool` and runtime now honors it.)
- `adapters/cli_tool.rs:collect_redacted_output and capture_stream`: Re-verify prior F2 fix: confirm a secret straddling the 1 MiB capture boundary cannot leak its prefix into SkillOutput.stdout/stderr. -> clean (cli_tool.rs:90-94 short-circuits to an empty CapturedText when `output.truncated` is set, then cli_tool_output (line 112-121) replaces both stdout and stderr with a fixed 'output omitted' notice. Captured raw bytes are dropped before redaction is even attempted, so any pre-redaction prefix never reaches output_object/receipts. `cli_tool_drains_large_stdout_and_omits_truncated_output` (tests/cli_tool_contract.rs:289-305) enforces empty stdout + omission notice.)
- `adapters/external_adapter.rs:process_env and parse_response`: Confirm scoped env cannot shadow credential delivery, adapter cannot self-request credentials, and response observations are redacted. -> clean (process_env (external_adapter.rs:799-819) inserts scoped env first and receipt-dir second, then credential_delivery.secret_env() last so credential bindings win on conflict. parse_response (external_adapter.rs:836-880) refuses CREDENTIAL_REQUEST_SCHEMA frames with UnexpectedCredentialRequest (line 855-868) and redacts the request_id surface; redact_response (external_adapter.rs:925-974) walks every observable surface (schema/version/ids/stdout/stderr/output/metadata/artifacts/errors/telemetry/observed_at).)
- `adapters/mcp/* secret_env and redact_text`: Confirm MCP delivers credentials only at child spawn, redacts tool results and error messages, and does not write credential material into sandbox metadata. -> clean (adapter.rs:132 clones secret_env into the transport request, transport.rs:274/381 injects it on child spawn (.env_clear().envs(scoped).envs(secret_env)). adapter.rs:54-65 wraps both success stringification and error messages in `credential_delivery.redact_text`. sandbox_metadata.rs only emits env *names* (allowlist), profile/cwd/network policy declarations — no host env values, so credential env values cannot land in metadata.)
- `Public frame and schema purity (credential_delivery.rs + schemas/credential-delivery-*.schema.json)`: Verify all four public frames carry only refs/hashes/ids/metadata and reject unknown fields and raw material via serde + JSON schema. -> clean (All four contract types (Profile/Request/BrokerResponse/Observation) use `#[serde(deny_unknown_fields)]`; corresponding JSON schemas use `additionalProperties: false`. CredentialDeliveryHandle carries only role + delivery_handle_ref + optional env_var. BrokerResponse exposes `credential_refs`, optional `handles`, optional `material_ref_hash`, optional `denied_reasons` — never material. Observation carries refs, profile_id, provider, purpose, delivery_mode, material_ref_hash, delivered_roles, redaction_refs only. SecretString has no Serialize impl and Debug -> '[redacted-credential]' (credentials.rs:177-181).)
- `Forward-compat handling of unsupported optional roles (credentials.rs:54-66)`: Hunt for silent contract<->runtime drift introduced by the new `Err(_) if !binding.required => continue` path. -> clean (Semantically correct: 'optional' means 'may be absent', so dropping a binding the runtime cannot map is consistent with the contract. Required+unsupported still fails closed via UnsupportedMaterialRole (credentials.rs:97-102). Empty-material check (apply_profile:348-352) uses .trim().is_empty(), defending against whitespace-only secrets. `delivery_profile_skips_optional_unsupported_contract_binding` (tests/credential_delivery.rs:101-131) and `delivery_profile_rejects_unsupported_contract_role` (line 133-144) cover both branches. Observability of skipped optional bindings is a debuggability gap, not a security or correctness gap; downstream Observation is constructed externally with delivered_roles populated by the caller, so the public observation remains accurate to what was actually delivered.)
- `Scope-drift / ambient-drift classification`: Confirm only task-scoped change (`crates/runx-runtime/src/credentials.rs`) is load-bearing and the ambient `tests/cli_tool_contract.rs` change does not silently shift credential-delivery semantics. -> clean (Task-scoped change is credentials.rs (the F1 propagation + EmptyMaterial trim + optional-unsupported skip). Ambient drift in tests/cli_tool_contract.rs adds cli-tool sandbox/timeout coverage (process group kill, large-stdout omission, input env normalization) and only touches the credential surface to construct `CredentialDelivery::none()` for invocations — no semantic change to delivery.)

Findings:
- none

## Origin

User architecture review on 2026-05-22: after the external execution-adapter
scope was corrected, the remaining cross-cutting gap is credential
broker/delivery. It must be a shared primitive consumed by cli-tool, external
adapters, and outbox/provider side effects rather than a per-protocol
afterthought.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-21T15:05:44Z
Ended: 2026-05-21T15:07:50Z

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/credentials.rs:199
  - Result: passed
  - Evidence: The runtime secret-bearing object is `CredentialDelivery`, which
- command audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: The acceptance section names executable scafld, schema, contract,
- scope/migration audit
  - Grounded in: code:crates/runx-runtime/src/adapters/external_adapter.rs:110
  - Result: passed
  - Evidence: `ExternalAdapterSupervisor` currently receives only the manifest
- acceptance timing audit
  - Grounded in: spec_gap:acceptance.dod4
  - Result: passed
  - Evidence: The external adapter acceptance item remains open until the
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback is explicit: keep a consumer blocked or create a named
- design challenge
  - Grounded in: code:crates/runx-contracts/src/external_adapter.rs:98
  - Result: passed
  - Evidence: The external adapter contract still contains an adapter-emitted

Issues:
- none
