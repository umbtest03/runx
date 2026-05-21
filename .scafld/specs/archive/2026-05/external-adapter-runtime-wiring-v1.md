---
spec_version: '2.0'
task_id: external-adapter-runtime-wiring-v1
created: '2026-05-22T00:00:00+10:00'
updated: '2026-05-21T15:52:28Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# External adapter runtime wiring

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T15:52:28Z
Review gate: pass

## Summary

Expose the existing feature-gated external adapter process supervisor as a
usable Rust runtime adapter path. The adapter path must accept
`source_type: external-adapter`, build the contract invocation frame under Rust
authority, call the supervisor, and convert accepted adapter observations into
normal `SkillOutput` values. It must not add provider-specific integration code
to the runtime kernel.

This spec is intentionally narrower than
`external-adapter-plugin-protocol-v1`. It is a Rust runtime wiring and
hardening slice, not the helper SDK, conformance adapter, registry discovery, or
custom provider package story.

## Context

Primary owner spec:
- `.scafld/specs/active/external-adapter-plugin-protocol-v1.md`

Owned touchpoints:
- `crates/runx-runtime/src/adapters/external_adapter.rs`
- `crates/runx-runtime/tests/external_adapter.rs`
- minimal runtime adapter-selection code when needed

Out of scope:
- x402 payment runtime, tests, and fixtures.
- canonical-json, core, and cloud files.
- provider-specific GitHub, Slack, Sentry, or hosted adapter logic.
- TypeScript helper SDKs.
- registry-backed or remote manifest discovery semantics.

## Objectives

- Add a feature-gated `SkillAdapter` facade for `external-adapter`.
- Keep the supervisor authoritative: the facade only builds contract frames,
  resolves manifests, calls the supervisor, and maps observations into runtime
  outputs.
- Support explicit inline manifests and package-relative manifest files without
  registry lookup or provider-specific runtime behavior.
- Deliver admitted credential material only through the existing private
  `CredentialDelivery` process-env channel, project only public credential refs
  and delivery-observation metadata across the external adapter boundary, and
  redact adapter observations before runtime mapping.
- Normalize external host-resolution frames into existing host protocol
  resolution requests.
- Preserve fail-closed behavior when the source type is unsupported, the
  manifest is absent or malformed, the response identity mismatches, the
  adapter crashes, or the response cannot safely map to runtime output.
- Prove a graph or skill invocation with `source_type: external-adapter`
  reaches the supervisor.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Runtime exposes a feature-gated external adapter `SkillAdapter`
  path for `source_type: external-adapter`.
- [x] `dod2` The path builds `ExternalAdapterInvocation` from runtime
  `SkillInvocation` without importing provider-specific logic.
- [x] `dod3` Manifest discovery remains explicit. Inline manifests or injected
  resolvers are allowed for this slice; package-relative manifest files are
  allowed when they canonicalize below the skill directory; implicit registry
  lookup is blocked until `external-adapter-plugin-protocol-v1` settles it.
- [x] `dod4` Tests prove a graph/skill invocation reaches the supervisor,
  package-relative manifest paths fail closed on directory escape, credentials
  are delivered/redacted, public credential delivery metadata is projected,
  host-resolution frames reach `Host`, and fail-closed behavior is preserved.

Validation:
- [x] `v1` Focused Rust tests pass.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter --test external_adapter -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T01:22:00+10:00 passed 16 tests, including
    `external_adapter_graph_invocation_reaches_process_supervisor`,
    `external_adapter_manifest_path_resolves_below_skill_directory`,
    `external_adapter_manifest_path_rejects_directory_escape`,
    `external_adapter_process_supervisor_delivers_credentials_and_redacts_observations`,
    `external_adapter_skill_adapter_passes_credential_delivery_to_supervisor`,
    `external_adapter_skill_adapter_projects_public_credential_refs_and_observation`,
    `external_adapter_process_supervisor_maps_host_resolution_frame`,
    `external_adapter_graph_host_resolution_frame_reaches_host`,
    `external_adapter_skill_adapter_fails_closed_without_inline_manifest`, and
    `external_adapter_skill_adapter_preserves_supervisor_fail_closed_response_mismatch`.
- [x] `v2` Focused spec validates.
  - Command: `scafld validate external-adapter-runtime-wiring-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:42:00+10:00 returned
    `{"ok":true,"command":"validate","result":{"task_id":"external-adapter-runtime-wiring-v1","path":"/Users/kam/dev/runx/runx/oss/.scafld/specs/active/external-adapter-runtime-wiring-v1.md","valid":true,"errors":null}}`.

## Design Constraints

- Feature-gated code must not be reachable without `features =
  ["external-adapter"]`.
- Runtime-local or `@runxhq/adapters` must not be used as a fallback.
- The Rust runtime may parse an explicit manifest and frame data, but must not
  understand provider APIs.
- If manifest lookup requires a new package or registry convention, this spec
  must record that as a blocker rather than inventing a broad discovery system.

## Remaining Boundary

The current parser/runtime `SkillSource` still carries external-adapter manifest
metadata through `source.raw`; this slice deliberately keeps that explicit and
local. It supports inline manifests and package-relative `manifest_path` files
that canonicalize below the skill directory. It does not claim registry-backed,
remote, signed, or catalog-driven manifest discovery.

## Rollback

Remove the feature-gated adapter facade and keep the process supervisor as an
explicit API if runtime selection cannot preserve Rust authority or requires
provider-specific logic.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode review (light) of external-adapter-runtime-wiring-v1 confirms the previous critical workspace_mutation finding is resolved: the contract reports `workspace_baseline=clean` and no review-time self-mutation, so the prior reviewer's mid-flight workspace change is no longer present and the verdict can again be trusted. The single task-scoped change since approval baseline is `crates/runx-runtime/tests/cli_tool_contract.rs` (M); the file is a sandbox/CLI-tool test surface that does not import or alter `external_adapter` production paths (`grep` for `external.adapter|ExternalAdapter` in the file returns zero hits), so it cannot regress the supervisor wiring under review. Spot-checked `crates/runx-runtime/src/adapters/external_adapter.rs` and `tests/external_adapter.rs`: the feature-gated `ExternalAdapterSkillAdapter` still routes `source_type: external-adapter` through `InlineExternalAdapterManifestResolver` + `ExternalAdapterProcessSupervisor` with fail-closed identity/protocol/schema/transport/timeout/exit/response paths intact, matching dod1–dod4 and the v1 evidence (16 tests passing). No new completion blockers found; the previous workspace_mutation finding is treated as fixed.

Attack log:
- `previous_finding:workspace_mutation`: Re-check workspace baseline and review-time mutation classification against the verify-mode snapshot -> clean (workspace_baseline=clean; no review_self_mutation entry; spec file present at original path.)
- `task_changes:cli_tool_contract.rs`: Regression hunt — grep the only task-scoped change for external-adapter coupling and inspect imports for shared sandbox/runtime API surface that could regress external_adapter wiring -> clean (Zero matches for external.adapter|ExternalAdapter in the file; imports are `runx_runtime::adapters::cli_tool::CliToolAdapter`, `sandbox::prepare_process_sandbox`, `credentials::CredentialDelivery`, and shared env constants — none touched by external_adapter.rs, which has its own subprocess path.)
- `adapters/external_adapter.rs + tests/external_adapter.rs`: Convention/dark-pattern spot-check against acceptance criteria dod1–dod4: feature gating, fail-closed paths, identity/schema/protocol/transport mismatch handling, manifest path canonicalization below skill directory -> clean (ExternalAdapterSkillAdapter checks source_type, delegates to InlineExternalAdapterManifestResolver + ExternalAdapterProcessSupervisor; supervisor errors enumerate UnsupportedManifestProtocol/Schema/Transport, AdapterIdMismatch, TimedOut+cancellation, ResponseTooLarge, UnexpectedCredentialRequest, ResponseMismatch — matching v1 evidence.)
- `ambient_drift`: Confirm no ambient drift outside task scope is being attributed to this task -> clean (ambient_drift=0 per classifier; other dirty paths (credential-broker spec move, draft edits) are not enumerated here and are not within this task's declared scope.)

Findings:
- [critical/non-blocking] `workspace_mutation` Prior reviewer mutated the workspace during review; now resolved.
  - Location: `.scafld/specs/active/external-adapter-runtime-wiring-v1.md`
  - Evidence: Context manifest reports workspace_baseline=clean, task_changes=1 (cli_tool_contract.rs), ambient_drift=0, and no review_self_mutation entry. The previously removed `.scafld/specs/active/external-adapter-runtime-wiring-v1.md` is present and readable at the same path.
  - Impact: Previously made the prior verdict untrustworthy; no longer the case under the current snapshot.
  - Validation: Re-run scafld review under verify mode against the restored workspace (this run).

