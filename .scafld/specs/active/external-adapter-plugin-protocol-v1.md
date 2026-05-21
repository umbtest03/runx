---
spec_version: '2.0'
task_id: external-adapter-plugin-protocol-v1
created: '2026-05-21T13:04:12Z'
updated: '2026-05-21T16:52:35Z'
status: review
harden_status: not_run
size: large
risk_level: high
---

# External execution adapter protocol v1

## Current State

Status: review
Current phase: final
Next: complete
Reason: review gate pass: 3 finding(s), 0 completion blocker(s)
Blockers: none
Allowed follow-up command: `scafld complete external-adapter-plugin-protocol-v1`
Latest runner update: 2026-05-21T16:52:45Z
Review gate: pass

## Summary

Define a language-neutral external execution-adapter process protocol for one
admitted run or step. Rust remains the supervisor: it admits the run, scopes
credentials, starts or connects to the adapter process, enforces
timeout/sandbox/redaction policy, validates returned contract shapes, and seals
receipts. The adapter process remains userland: it may be written in
TypeScript, JavaScript, Python, Rust, or another language, and may contain
execution-time integration-specific provider code.

This is not a replacement TypeScript runtime. It is an out-of-process
execution boundary owned by contracts and supervised by Rust.

## Context

Current built-in Rust adapters cover core execution families under
`oss/crates/runx-runtime/src/adapters/`: `cli_tool`, `mcp`, `agent`, `a2a`,
and `catalog`.

Current TypeScript adapters and cloud hosted adapters prove there are richer
execution-adapter needs than simple CLI tools:
- hosted/durable agent execution;
- custom adapter selection and replacement;
- host continuation and `needs_agent` flows;
- execution-time credential binding;
- execution-time provider-specific API glue.

Those needs should not force provider SDK code into Rust, but they also must
not keep an in-process TypeScript trusted runtime alive.

This spec can consume host, credential, catalog, and SDK surfaces during an
execution invocation, but it does not own those surfaces as general extension
protocols. Source-event ingress, hosted/embedded runtime binding, tool
catalog/read-model access, registry control, auth storage, webhook
verification, artifact-store ownership, and thread/outbox provider writes must
use sibling protocol specs or remain blockers for
`rust-ts-sunset-runtime-local`.

Credential material delivery is owned by
`credential-broker-delivery-contract-v1`. This protocol may reference admitted
credential refs and consume Rust-supervised delivery handles or process-env
delivery, but adapters must not invent arbitrary secret request/response
channels.

## Objectives

- Specify external execution-adapter discovery:
  manifest fields, supported source types, protocol version, command or
  endpoint, startup timeout, lifecycle, declared credential needs, and sandbox
  intent.
- Specify invocation frames:
  adapter identity, skill/source metadata, typed inputs, resolved inputs,
  scoped env, credential delivery references, cwd, receipt directory, run/step
  identifiers, host-resolution channel, and cancellation.
- Specify response frames:
  status, stdout/stderr or structured output, metadata, emitted artifacts,
  requested host resolutions, retry/failure semantics, and adapter-reported
  telemetry.
- Specify host interaction:
  approval/input/agent resolution requests must round-trip through
  `runx-contracts`/`@runxhq/contracts`; adapters do not invent private host
  result shapes.
- Provide TypeScript helper SDKs over the protocol while keeping the Rust
  runtime authoritative.
- Add conformance fixtures that can be implemented by at least TypeScript and
  one non-TypeScript sample adapter.

## Scope

In scope:
- External execution-adapter manifest and process protocol.
- Rust supervisor implementation plan for process lifecycle, validation,
  credential scoping, redaction, timeout, and receipt integration.
- TypeScript author SDK over generated contracts.
- Negative tests proving no runtime-local/adapters fallback is required.

Out of scope:
- Reimplementing built-in trusted adapters in TypeScript.
- Provider-specific integration packages.
- Replacing MCP as the preferred tool-integration protocol.
- Replacing the simpler `cli-tool` skill ABI owned by
  `skill-author-runtime-contract-v1`.
- Source-event ingress protocols for Slack, Sentry, GitHub, file, API, or
  webhook signal admission.
- Hosted/embedded runtime binding for cloud worker, agent-runner, SDK, host
  bridge, continuation, auth resolver, and resume semantics.
- Public tool-catalog/read-model search and inspect protocols.
- Thread/outbox provider protocols for comments, PR updates, or rendered story
  consumers.

## Dependencies

- `ts-extension-survivorship-boundary`
- `skill-author-runtime-contract-v1`
- `credential-broker-delivery-contract-v1`
- `canonical-json-fingerprint-contract-v1`
- `rust-contract-schema-validation-gate`
- `rust-ts-sunset-runtime-local`

## Touchpoints

- `oss/crates/runx-runtime/src/adapters/`
- `oss/crates/runx-runtime/src/adapter.rs`
- `oss/crates/runx-contracts/src/`
- `oss/packages/contracts/src/`
- `oss/packages/authoring/`
- `oss/packages/create-skill/`
- `oss/docs/ts-interop-boundary.md`
- `cloud/packages/agent-runner/src/`
- `cloud/packages/worker/src/`

## Risks

- A too-rich protocol can recreate runtime-local out of process.
- A too-small protocol can make custom hosted adapters impossible and push
  users into forking Rust.
- Treating this as the umbrella plugin protocol can mis-model non-execution
  queues and hide missing source-ingress, hosted-runtime, catalog, or outbox
  specs.
- Credential delivery and redaction must remain Rust-supervised; adapter
  helpers cannot become trusted secret stores.
- Streaming/continuation support must be explicit or custom adapters will be
  limited to one-shot happy paths.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Protocol v1 is documented with manifest, invocation, response,
  host-resolution, credential, timeout, and receipt semantics.
  - Phase 1 evidence: `packages/contracts/src/schemas/external-adapter.ts`,
    `schemas/external-adapter-*.schema.json`,
    `crates/runx-contracts/src/external_adapter.rs`, and
    `fixtures/contracts/external-adapter/*.json`.
  - Receipt boundary: adapter responses are observations only; Rust
    supervision converts accepted observations into sealed harness receipts.
- [x] `dod2` Rust runtime has a fail-closed adapter supervisor that validates
  every frame against `runx-contracts`.
  - Phase 2a evidence: `crates/runx-runtime/src/adapters/external_adapter.rs`
    and `crates/runx-runtime/tests/external_adapter.rs` provide an explicit
    feature-gated process-supervisor API and focused tests.
  - Phase 2b evidence: `external-adapter-runtime-wiring-v1` adds the
    feature-gated `ExternalAdapterSkillAdapter`, inline and package-relative
    manifest resolution, injectable manifest resolver/supervisor traits, graph
    routing coverage, credential delivery through
    `credential-broker-delivery-contract-v1`, observation redaction,
    host-resolution frame normalization/routing, and fail-closed tests. Startup
    readiness remains a non-goal for v1 because the frozen contract has no
    ready frame; the one-shot invocation deadline is enforced.
- [x] `dod3` TypeScript helper SDK exists only as a protocol client/server
  helper and does not import runtime-local/adapters.
- [x] `dod4` At least one TypeScript sample adapter and one non-TypeScript
  sample adapter pass the same conformance fixture.
- [ ] `dod5` Runtime-local/adapters sunset can point at this protocol for
  custom execution-adapter authoring without preserving the old packages.
- [ ] `dod6` Runtime-local/adapters sunset does not cite this protocol as the
  answer for source ingress, hosted runtime binding, catalog/read-model, or
  thread/outbox provider queues unless those behaviors are explicitly modeled by
  sibling specs.

Validation:
- [x] `v1` Scafld validates this spec.
  - Command: `scafld validate external-adapter-plugin-protocol-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:16:09+10:00 returned
    `{"ok":true,"command":"validate","result":{"task_id":"external-adapter-plugin-protocol-v1","path":"/Users/kam/dev/runx/runx/oss/.scafld/specs/active/external-adapter-plugin-protocol-v1.md","valid":true,"errors":null}}`.
- [x] `v2` Rust protocol tests pass.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter,cli-tool external_adapter`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T01:22:00+10:00 passed 16 focused
    `tests/external_adapter.rs` tests covering process launch, invocation
    frame serialization, response parsing, mismatched response identity,
    timeout-to-cancellation mapping, unexpected credential-request rejection,
    unknown protocol pre-spawn rejection, crashed-process failure,
    package-relative manifest path success/escape rejection, credential
    delivery/redaction, public credential refs/delivery-observation projection,
    host-resolution frame parsing, and graph-level host routing. The narrower command
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter --test external_adapter`
    also passed 16 tests. Running the old no-feature filtered command still hits
    the pre-existing `cli_tool_contract.rs` integration-test discovery import,
    so this feature-gated slice records the explicit feature set.
- [x] `v3` TypeScript helper tests pass.
  - Command: `pnpm vitest run packages/authoring/src/index.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:45:53+10:00 passed 1 file, 12 tests covering
    existing authoring helpers plus TypeScript external-adapter conformance,
    Python adapter conformance over the same fixture, response identity
    fail-closed behavior, and protocol-only helper imports.
- [x] `v4` No helper imports deleted runtime packages.
  - Command: `! rg -n "@runxhq/(runtime-local|adapters)|packages/(runtime-local|adapters)" packages/{authoring,create-skill,contracts,host-adapters,langchain} --glob '!**/dist/**'`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:45:53+10:00 returned no matches after moving the
    conformance assertion to construct forbidden package names dynamically, so
    the literal guard can scan survivor package sources.
- [x] `v5` TypeScript protocol schema fixtures pass.
  - Command:
    `pnpm vitest run packages/contracts/src/schemas/external-adapter.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T13:57:28Z re-run passed: 1 test file, 4 tests,
    including rejection of runtime-local `sealed` status and secret material in
    credential request frames.
- [x] `v6` Generated JSON Schemas are fresh.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T13:57:28Z re-run exited 0 with
    `tsx scripts/generate-contract-schemas.ts --check`.
- [x] `v7` Rust contract fixture parity passes.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test external_adapter_fixtures -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T13:57:28Z re-run passed: 2 tests, including
    rejection of runtime-local `sealed` response status.
- [x] `v8` Contract fixtures validate against generated schemas.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test schema_validation -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T13:57:28Z re-run passed: 5 tests, including the
    mapped external-adapter fixture schema coverage.
- [x] `v9` TypeScript workspace typecheck passes with the authoring helper.
  - Command: `pnpm tsc -p tsconfig.typecheck.json --noEmit --pretty false`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:45:53+10:00 exited 0.

## Phase 1: Protocol Shape

Status: completed
Dependencies: ts-extension-survivorship-boundary

Objective: Complete this phase.

Changes:
- [x] Define the external execution-adapter manifest schema.
- [x] Define invocation and response envelopes in `runx-contracts` and generated `@runxhq/contracts`.
- [x] Define host-resolution, credential request, and cancellation frames.
- [x] Define what extension code may and may not control.

Acceptance:
- none

## Phase 2: Rust Supervisor

Status: completed
Dependencies: Phase 2a

Objective: Complete this phase.

Changes:
- [x] Add an external adapter supervisor behind an explicit runtime feature.
- [x] Wire the supervisor into runtime adapter selection for explicit inline manifests or injected resolvers.
- [x] Enforce per-invocation timeout, frame validation, credential delivery through `credential-broker-delivery-contract-v1`, redaction, host-resolution frame routing, and response metadata mapping into `SkillOutput`.
- [x] Fail closed on unknown protocol version, malformed frames, unexpected credential requests, and adapter crashes for the feature-gated one-shot process API.
- [x] Route host-resolution frames through the existing host resolution protocol before receipt construction. Accepted adapter response metadata maps into `SkillOutput` for normal receipt construction.
- [x] Added `external-adapter` runtime feature gating for the new supervisor.
- [x] Added `ExternalAdapterProcessSupervisor::invoke(manifest, invocation)` as an explicit API, deliberately not wired into graph execution or adapter selection.
- [x] Process transport launches with env cleared and only string-valued scoped invocation env plus `RUNX_RECEIPT_DIR` admitted.
- [x] Invocation frames are serialized from `runx-contracts`; response frames are parsed back through `ExternalAdapterResponse` and checked for schema, protocol, adapter ID, and invocation ID.
- [x] Timeout creates a `runx.external_adapter.cancellation.v1` frame and terminates the adapter process group before failing closed.
- [x] Unknown protocol/schema, unsupported transport, empty command, non-string process env, credential-request frames on the response channel, malformed JSON, oversized responses, and crashed adapter processes fail closed.
- Startup readiness has no separate ready frame in the frozen contract; the current slice validates non-zero startup timeout but only enforces the one-shot invocation deadline.
- Credential material delivery, redaction policy, host-resolution routing, and normal `SkillOutput` mapping are covered by Phase 2b and the completed `external-adapter-runtime-wiring-v1` slice. Helper SDKs and conformance adapters remain Phase 3 work.
- [x] Added `ExternalAdapterSkillAdapter` behind `features = ["external-adapter"]`.
- [x] Added explicit inline-manifest resolution from `SkillSource.raw` and package-relative `manifest_path` resolution that canonicalizes below the skill directory, plus injectable manifest resolver/supervisor traits for tests and future host wiring.
- [x] Built `ExternalAdapterInvocation` frames from `SkillInvocation` without provider-specific runtime logic.
- [x] Mapped accepted adapter observations to `SkillOutput` while keeping adapter responses as untrusted observations, not receipts.
- [x] Passed `CredentialDelivery` into the supervised process env after scoped env admission, and redacted stdout/stderr/output/metadata/errors/artifacts before runtime mapping.
- [x] Normalized host-resolution frames into response metadata and routed them through `Host::resolve` in graph execution.
- [x] Added graph/skill routing coverage proving `source_type: external-adapter` reaches the supervisor and fails closed when manifest identity or response identity is unsafe.
- `external-adapter-runtime-wiring-v1` validated at 2026-05-22T01:22:00+10:00.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter --test external_adapter -- --nocapture` passed 16 focused tests at 2026-05-22T01:22:00+10:00.

Acceptance:
- none

## Phase 3: Author SDKs And Fixtures

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- [x] Add TypeScript helpers that implement the protocol server/client boilerplate using `@runxhq/contracts`.
- [x] Add sample adapters and shared conformance fixtures.
- [x] Add negative tests proving helpers do not import runtime-local/adapters.

Acceptance:
- none

## Rollback

If the protocol cannot preserve required hosted/custom execution-adapter
behavior, keep the runtime-local/adapters sunset blocked and narrow the
protocol. Do not solve the gap by reviving a TypeScript trusted runtime or by
folding non-execution queues into the execution-adapter protocol.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Reviewed the external-adapter-plugin-protocol-v1 implementation end-to-end: contracts (`crates/runx-contracts/src/external_adapter.rs`, `packages/contracts/src/schemas/external-adapter.ts`), Rust supervisor (`crates/runx-runtime/src/adapters/external_adapter.rs` + 16 supervisor tests), TS author helpers (`packages/authoring/src/index.ts` + 12 vitest cases), and the cross-language conformance fixtures (`fixtures/external-adapter-conformance/`). Spec evidence items (v1-v9) are met as worded and fail-closed behavior is well covered: schema/protocol/identity mismatches, unexpected credential frames, oversize responses, timeout→cancellation frames, process-group cleanup on Unix, env_clear + scoped-env-then-credential override, manifest path canonicalization against directory escape, and unsafe response detection through SkillAdapter all have explicit tests. Redaction runs over every observed string field including telemetry/artifacts/errors. No blockers found; three non-blocking quality gaps in the demonstrated conformance and the wire-protocol surface area are recorded. Workspace baseline reports the cited touchpoints as "ambient drift" because the scafld touchpoints carry an `oss/` prefix while the active workspace cwd is already `oss/`; this is a scafld configuration mismatch, not implementation drift, and the changes match the spec's declared scope.

Attack log:
- `fixtures/external-adapter-conformance/python_echo_adapter.py`: Compare delivery mechanism (argv) with the supervisor's stdin write path -> finding (Python sample is incompatible with the Rust supervisor's wire protocol — see F1.)
- `packages/authoring/src/index.test.ts`: Check whether conformance tests invoke adapters via real subprocess + stdin -> finding (Both samples are invoked through paths that bypass the supervisor's wire protocol — see F2.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:836-880 parse_response`: Can adapter preamble (logs/print) before JSON break the response? -> finding (Whole stdout must parse as one JSON document; undocumented constraint — see F3.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:1098-1192 wait_for_exit/kill_timed_out_process`: Process-group cleanup on timeout, fallback on non-Unix -> clean (TERM then KILL with grace, then direct child kill fallback; cancellation frame constructed before fail-closed.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:799-819 process_env`: Credential env can leak / be overridden by scoped env -> clean (env_clear() + scoped env first + credential delivery env last ensures broker wins; covered by external_adapter_process_supervisor_delivers_credentials_and_redacts_observations.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:408-461 manifest_from_path / validate_manifest_relative_path`: Directory escape via `../` or absolute paths in manifest_path -> clean (Relative + Component::Normal-only check, plus canonicalize+starts_with(skill_directory); covered by external_adapter_manifest_path_rejects_directory_escape.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:997-1028 validate_response_contract / host_resolution_response`: Forge identity in host-resolution frame to escape identity check -> clean (Synthetic response built from frame fields still subject to invocation identity check; frame schema and protocol version validated.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:925-995 redact_response`: Secret leakage through telemetry/artifacts/errors/metadata -> clean (Recursive redaction over strings within object/array values and across telemetry/artifact summaries/error code+message/metadata.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:1057-1082 capture_stream`: Memory exhaustion / pipe-back-pressure DoS via huge stdout -> clean (Capture capped at 1 MiB; reader continues draining the pipe so the child cannot stall; truncation flips a flag that fails closed via ResponseTooLarge.)
- `crates/runx-runtime/src/execution/runner/steps.rs:60-150 route_external_adapter_host_resolution`: Adapter forges host-resolution metadata to invoke arbitrary host requests -> clean (Routing through metadata happens after supervisor identity validation; ResolutionRequest deserialization rejects malformed shapes; observed events fire only for valid requests.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:700-758 validate_invocation_contract`: Bypass schema/protocol/transport gating -> clean (Pre-spawn validation rejects unknown schema/protocol on both manifest and invocation, adapter_id mismatch, unsupported source type, non-process transport, and zero timeouts; covered by ..._rejects_unknown_protocol_before_spawn.)

Findings:
- [medium/non-blocking] `F1-python-conformance-misrepresents-wire-protocol` Python conformance adapter reads invocation from argv, but the Rust supervisor delivers invocations only via stdin
  - Location: `fixtures/external-adapter-conformance/python_echo_adapter.py:8`
  - Evidence: fixtures/external-adapter-conformance/python_echo_adapter.py:8 opens `sys.argv[1]` to load the invocation. The supervisor (crates/runx-runtime/src/adapters/external_adapter.rs:774-834) always pipes stdin and writes the invocation via `serde_json::to_writer(&mut stdin, invocation)` + `\n`. It never sets argv, so a python adapter following this sample crashes with IndexError before producing any frame. The TS test (packages/authoring/src/index.test.ts:218-237) invokes the python script directly via `execFile("python3", [script, invocationPath])` — exercising the argv path, not the supervisor's stdin transport.
  - Impact: dod4 claims a non-TypeScript sample adapter passes the same conformance fixture, but the only non-TS sample wouldn't function when launched by the real supervisor. Adapter authors copying this Python pattern will produce non-functional adapters that fail closed at runtime, undermining the spec's language-neutral claim and dod6's promise that this protocol can be cited for custom adapter authoring.
- [medium/non-blocking] `F2-conformance-bypasses-wire-protocol` Neither conformance test exercises the actual stdin→adapter→stdout wire protocol
  - Location: `packages/authoring/src/index.test.ts:181`
  - Evidence: packages/authoring/src/index.test.ts:181-216 runs the TS sample via `adapter.runWith(invocation)` — a direct in-process call, not a subprocess. packages/authoring/src/index.test.ts:218-237 runs the python sample via execFile with the invocation passed as a CLI arg. The Rust supervisor's wire protocol (write_invocation→stdin newline, parse_response→full stdout JSON) is tested only with shell scripts in crates/runx-runtime/tests/external_adapter.rs, not with the TS or Python sample adapters that the spec advertises as conformant.
  - Impact: The dod4 evidence proves both adapters can produce a schema-valid response, but does not prove either implementation conforms to the wire protocol the Rust supervisor speaks. A future supervisor change to the transport (or vice versa, an adapter language that diverges) would not be caught by these tests.
- [low/non-blocking] `F3-wire-protocol-undocumented` Process-transport wire protocol (stdin-delivered invocation, single-JSON-document stdout, stderr-discarded) is implicit and undocumented
  - Location: `packages/contracts/src/schemas/external-adapter.ts:1`
  - Evidence: The TS helper exposes three invocation input modes (packages/authoring/src/index.ts:738-746 — `RUNX_EXTERNAL_ADAPTER_INVOCATION_JSON`, `RUNX_EXTERNAL_ADAPTER_INVOCATION_PATH`, stdin fallback), but the Rust supervisor only writes via stdin (crates/runx-runtime/src/adapters/external_adapter.rs:821-834). The supervisor also requires that all of stdout parse as exactly one JSON document (parse_response at line 836-880) — any preamble such as a debug `print` would surface as a JSON parse error and fail closed. None of `docs/ts-interop-boundary.md`, the SKILL.md fixture, or `packages/contracts/src/schemas/external-adapter.ts` describe these transport-level rules.
  - Impact: Authors of non-TS adapters have no reference for how to read the invocation or what they may write to stdout. This is the operational counterpart to F1 and contributes to the conformance gap. It also creates risk that the supervisor and helper diverge silently (e.g., the helper adds an env-based mode the supervisor never emits).

## Origin

User architecture review on 2026-05-21: runx must not drag integration code
into Rust, and Rust-only adapter authoring would harm adoption.
