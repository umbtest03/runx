---
spec_version: '2.0'
task_id: process-credential-delivery-hardening-v1
created: '2026-05-25T17:51:35+10:00'
updated: '2026-05-26T03:20:04Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# process-credential-delivery-hardening-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-26T03:20:04Z
Review gate: pass

## Summary

This spec is a regression-lock and ratification pass for supervised process
credential delivery. The current runtime must keep failing closed when a
`CredentialDelivery` contains process-env secret material at the MCP process,
external-adapter process, and thread-outbox-provider process boundaries.

This spec does not introduce the future positive non-env delivery channel.
Opaque-reference, scoped-file, descriptor, or brokered delivery remains a
separate architectural decision. For this spec, redaction and public
observations are receipt-safety evidence only; they are not credential
containment.

## Context

Harden round 1 found the negative invariant is already implemented:
`CredentialDelivery::reject_process_env_boundary` is called before spawning
`cli-tool`, MCP process transport, external adapter process transport, and
thread-outbox-provider process transport. The draft is still useful as a
regression-lock because the old validation commands were vacuous or
mis-targeted. The work here is to correct the contract, add any missing focused
test coverage, and run the non-vacuous validations.

## Scope

In scope:
- MCP process transport fail-closed credential delivery behavior.
- External adapter process supervisor fail-closed credential delivery behavior.
- Thread outbox provider process supervisor fail-closed credential delivery
  behavior.
- Public credential-delivery observation metadata that carries references,
  roles, hashes, and redaction refs without raw secret material.
- Focused validation commands that compile the relevant cfg-gated tests with
  the required feature flags and do not pass vacuously.

Out of scope:
- Provider-specific OAuth flows.
- A new positive non-env credential delivery channel.
- `cli-tool` runtime changes; cli-tool remains a reference fail-closed boundary.
- Registry install, skill output attestation, Nitrosend live dogfood, monolith
  decomposition, cloud/root work, and `runx-rust-95-release-readiness`.

Touchpoints:
- `.scafld/specs/active/process-credential-delivery-hardening-v1.md`
- `crates/runx-runtime/src/credentials.rs`
- `crates/runx-runtime/src/adapters/mcp/transport.rs`
- `crates/runx-runtime/src/adapters/external_adapter.rs`
- `crates/runx-runtime/src/outbox_provider.rs`
- `crates/runx-runtime/tests/credential_delivery.rs`
- `crates/runx-runtime/tests/external_adapter.rs`
- `crates/runx-runtime/tests/thread_outbox_provider.rs`

## Phases

## Phase 1: Contract Correction

Status: completed
Dependencies: none

Objective: Replace the stale positive-channel wording with a bounded
regression-lock contract and fix non-vacuous validation targeting.

Changes:
- `.scafld/specs/active/process-credential-delivery-hardening-v1.md`

Acceptance:
- `scafld harden process-credential-delivery-hardening-v1 --provider claude`
  returns pass after this revision.
- `scafld validate process-credential-delivery-hardening-v1 --json` passes.

## Phase 2: Runtime Regression Lock

Status: completed
Dependencies: Phase 1

Objective: Keep supervised process boundaries fail-closed and prove public
credential-delivery observations remain secret-free.

Changes:
- `crates/runx-runtime/tests/credential_delivery.rs`
- Runtime source files listed in Touchpoints are read/verified and edited only if the focused validations expose a live gap.

Acceptance:
- All validation commands in this spec pass with non-zero test coverage.
- The focused grep finds no dangerous child process env injection shape for
  credential secret material.

## Phase 3: Scafld Review And Completion

Status: completed
Dependencies: Phase 2

Objective: Run the adversarial review gate and archive the spec only if no
completion blockers remain.

Changes:
- none expected

Acceptance:
- `scafld review process-credential-delivery-hardening-v1 --provider claude`
  passes or all blocking findings are resolved.
- `scafld complete process-credential-delivery-hardening-v1` archives the spec
  after a passing review gate.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` MCP process delivery rejects process-env secret material before
  spawning a child process.
- [x] `dod2` External adapter process delivery rejects process-env secret
  material before spawning a child process.
- [x] `dod3` Outbox provider process delivery rejects process-env secret
  material before spawning a child process.
- [x] `dod4` Public credential-delivery observations and adapter metadata carry
  opaque refs/hashes/roles without raw secret material or env-var handoff.

Validation:
- [x] `v1` MCP process credential delivery tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features "cli-tool mcp" --test credential_delivery mcp_process_transport_rejects_process_env_credential_delivery -- --exact`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit_code_zero; 1 passed, 0 failed, 12 filtered out
  - Status: passed
  - Evidence: MCP process delivery rejects process-env credential material before child spawn and does not leak raw secret material into output metadata
  - Source event: local validation run
  - Last attempt: 2026-05-26T03:03:48Z
  - Checked at: 2026-05-26T03:03:48Z
- [x] `v2` external-adapter credential delivery tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter --test external_adapter external_adapter_process_supervisor_rejects_process_env_credential_delivery -- --exact`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit_code_zero; 1 passed, 0 failed, 16 filtered out
  - Status: passed
  - Evidence: external adapter process delivery rejects process-env credential material before child spawn and keeps denial output secret-free
  - Source event: local validation run
  - Last attempt: 2026-05-26T03:03:48Z
  - Checked at: 2026-05-26T03:03:48Z
- [x] `v3` outbox provider credential delivery tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test thread_outbox_provider provider_process_rejects_process_env_credential_delivery -- --exact`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit_code_zero; 1 passed, 0 failed, 5 filtered out
  - Status: passed
  - Evidence: thread outbox provider process delivery rejects process-env credential material before child spawn and keeps denial output secret-free
  - Source event: local validation run
  - Last attempt: 2026-05-26T03:03:48Z
  - Checked at: 2026-05-26T03:03:48Z
- [x] `v4` credential observation serialization contract tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features "cli-tool mcp" --test credential_delivery public_observation_metadata_serializes_without_secret_material -- --exact`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit_code_zero; 1 passed, 0 failed, 12 filtered out
  - Status: passed
  - Evidence: public credential-delivery observation metadata serializes refs and hashes without the raw token or env-var handoff
  - Source event: local validation run
  - Last attempt: 2026-05-26T03:03:48Z
  - Checked at: 2026-05-26T03:03:48Z
- [x] `v5` external-adapter public observation tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter --test external_adapter external_adapter_skill_adapter_projects_public_credential_refs_and_observation -- --exact`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit_code_zero; 1 passed, 0 failed, 16 filtered out
  - Status: passed
  - Evidence: external adapter invocation receives public credential refs and output metadata records credential observations without raw secret material
  - Source event: local validation run
  - Last attempt: 2026-05-26T03:03:48Z
  - Checked at: 2026-05-26T03:03:48Z
- [x] `v6` outbox provider public observation tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test thread_outbox_provider provider_process_pushes_idempotently_and_injects_delivery_observation -- --exact`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit_code_zero; 1 passed, 0 failed, 5 filtered out
  - Status: passed
  - Evidence: thread outbox provider observation injects non-secret credential-delivery observation metadata
  - Source event: local validation run
  - Last attempt: 2026-05-26T03:03:48Z
  - Checked at: 2026-05-26T03:03:48Z
- [x] `v7` focused process-env delivery grep review
  - Command: `rg -n "\\.envs\\([^\\n]*secret_env|\\.env\\([^\\n]*secret_env|CredentialDelivery::ProcessEnv" crates/runx-runtime/src crates/runx-runtime/tests`
  - Expected kind: `no_matches`
  - Timeout seconds: none
  - Result: no_matches; rg exited 1 with no output
  - Status: passed
  - Evidence: no supervised process adapter injects `secret_env` into child process environment and no legacy `CredentialDelivery::ProcessEnv` variant exists
  - Source event: local validation run
  - Last attempt: 2026-05-26T03:03:48Z
  - Checked at: 2026-05-26T03:03:48Z

## Rollback

- If any runtime source edit becomes necessary and regresses the fail-closed
  posture, restore the pre-spawn credential-delivery rejection at each
  supervised boundary: the `reject_process_env_boundary` calls in cli-tool,
  external adapter, and outbox provider, and the `secret_env.is_empty()` guard
  in MCP transport before retrying.
- If the added test proves flaky or incorrectly scoped, remove only the new
  credential-delivery observation serialization test and keep the existing
  fail-closed runtime behavior unchanged.
- If review determines positive delivery must be solved now, fail or cancel this
  regression-lock spec and write a separate design spec for the chosen non-env
  channel instead of expanding this scope in place.

## Deviations

- Harden round 1 showed the original draft was stale as a positive-channel
  implementation spec. This revision intentionally narrows it to the still-valid
  regression lock: raw process-env credential material must not cross supervised
  process boundaries.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Regression-lock spec is sound. The fail-closed credential-delivery posture is enforced at all four supervised process boundaries: MCP transport (transport.rs:165 via `request.secret_env.is_empty()` before `spawn_tokio_mcp_server` at line 170), external-adapter supervisor (external_adapter.rs:301-303 via `reject_process_env_boundary("external-adapter")` before any spawn), thread-outbox-provider supervisor (outbox_provider.rs:93-95 via `reject_process_env_boundary("thread-outbox-provider")` before `Command::spawn` at line 112), and cli-tool (cli_tool.rs:34, out of scope but verified as reference boundary). The seven validations target real, cfg-gated, non-vacuous test functions with correct feature flags; v7's tightened grep returns zero matches across `crates/runx-runtime/src` and tests. Public observation metadata serializes opaque refs + sha256 material hashes without secret material or env-var binding names, verified by `public_observation_metadata_serializes_without_secret_material` at tests/credential_delivery.rs:124. Only one call site reads `credential_delivery.secret_env()` (adapter.rs:132 into the McpToolCallRequest seam), and that seam is gated by the transport-side emptiness check. Workspace baseline shows only `crates/runx-runtime/tests/credential_delivery.rs` was dirty before review and zero task-scoped changes since baseline; no review_self_mutation. Other modified files in `git status` (CLI registry, IDE plugins, runtime install, packages/cli, etc.) are ambient drift outside this spec's scope. All four DODs map onto v1-v7 acceptance; phases 1-3 are completed; harden round 2 passed. No completion blockers.

Attack log:
- `crates/runx-runtime/src/adapters/mcp/transport.rs:162-170`: Verify pre-spawn rejection: confirm `request.secret_env.is_empty()` check at line 165 returns Err before `spawn_tokio_mcp_server` is called at line 170; trace from McpAdapter::invoke -> adapter.rs:132 (puts secret_env into request) to ensure no other code path bypasses the check. -> clean (Rejection is structural and pre-spawn. list_tools_with_rmcp_async at line 135 takes McpListToolsRequest which has no secret_env field (types.rs:22-26), so the listing path carries no credentials by type construction.)
- `crates/runx-runtime/src/adapters/external_adapter.rs:295-303 and process_env at 816-833`: Verify that credential_delivery cannot flow into child env via the supervisor: `reject_process_env_boundary` runs before validate_invocation_contract and spawn_process; process_env intentionally takes `_credential_delivery` (unused) and only merges scoped invocation env + RUNX_RECEIPT_DIR. -> clean (Defense by design: process_env signature accepts CredentialDelivery but discards it, so even if secret_env were non-empty, it could not reach .envs() without modifying process_env signature, which the v7 grep would catch.)
- `crates/runx-runtime/src/outbox_provider.rs:87-112`: Verify that `reject_process_env_boundary` runs before `child.spawn()` at line 112; trace that `child.env_clear().stdin(...).stdout(...).stderr(...)` at lines 105-109 never adds any env from credential_delivery or invocation. -> clean (Provider supervisor never calls .env() or .envs() at all — credential refs and observations flow only via stdin JSON payloads parsed elsewhere.)
- `crates/runx-runtime/src/credentials.rs:311-321 and public_observation pipeline 333-430`: Verify public_observation never serializes the secret value or env var name; confirm `build_local_provision_observation` only emits provider, profile_id, credential_refs (with `runx:credential:` URI), material_ref_hash (sha256-prefixed), delivered_roles, and timestamps. -> clean (Confirmed via tests/credential_delivery.rs:124-159: `public_observation_metadata_serializes_without_secret_material` asserts serialized form contains neither the raw secret 'ghs_observation_secret_must_not_leak' nor the env var name 'GITHUB_TOKEN'.)
- `v7 grep pattern `\.envs\([^\n]*secret_env|\.env\([^\n]*secret_env|CredentialDelivery::ProcessEnv` against crates/runx-runtime/src and tests`: Try to bypass the structural-shape grep by tracing all paths from `credential_delivery.secret_env()` to a child process .env()/.envs() call. -> clean (Only one production call site reaches `credential_delivery.secret_env()` (adapters/mcp/adapter.rs:132), and that value flows only into McpToolCallRequest.secret_env, which is gated by the transport-side emptiness check before any spawn. Grep returns zero matches in the runtime crate.)
- `Spec acceptance gate: DOD -> validation mapping`: Verify each DOD has a non-vacuous validation. Confirm v1/v4 compile under `--features 'cli-tool mcp'` (file gated by `#![cfg(all(feature = "cli-tool", feature = "mcp"))]` at credential_delivery.rs:1), v2/v5 compile under `--features external-adapter`, v3/v6 compile without feature flags, and each names a real `-- --exact` test function. -> clean (All seven validation targets correspond to real tests (verified by reading the test files); harden round-2 already verified line numbers at credential_delivery.rs:124,309 / external_adapter.rs:368,398 / thread_outbox_provider.rs:22,141. Result lines record `1 passed, 0 failed, N filtered out` for each, proving non-vacuous execution.)
- `Workspace scope vs git status drift`: Classify modified files in git status against declared task scope. -> clean (Only `crates/runx-runtime/tests/credential_delivery.rs` is task-scoped and was baseline-dirty (the v4 observation-serialization test addition); all other modified files (crates/runx-cli/*, packages/*, plugins/*, scripts/*, two other active specs) are ambient drift, not attributable to this regression-lock spec. Two new active specs (process-credential-delivery-hardening-v1 and registry-signed-manifest-trust-anchor-v1) appear in the active dir with their drafts deleted — consistent with `scafld approve` lifecycle moves.)
- `Rollback section vs actual code touchpoints`: Verify rollback wording lines 217-221 accurately names the call sites that restore the fail-closed posture. -> clean (Rollback now correctly distinguishes the three boundaries that use `reject_process_env_boundary` (cli-tool, external-adapter, outbox-provider) from MCP transport which uses the sibling `secret_env.is_empty()` guard at transport.rs:165 — addresses harden round-2 advisory finding.)

Findings:
- none

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-26T02:39:40Z
Ended: 2026-05-26T02:39:40Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Spec's negative invariant (no ambient process-env secrets) is already enforced today at all four supervised boundaries (mcp/transport.rs:165-169, external_adapter.rs:301-303, outbox_provider.rs:93-95, cli_tool.rs:34) and has regression tests in tests/credential_delivery.rs, tests/external_adapter.rs:368, tests/thread_outbox_provider.rs:141, tests/local_credential_provision.rs. That makes this spec largely a regression-lock, yet the Summary promises a positive delivery channel ("opaque references, scoped files, or runtime-owned descriptor channel") that no DOD, validation, or phase actually pins down. Three of four validation commands are mis-targeted or missing required feature flags, so they will pass vacuously: v1 runs the wrong test file, and v2/v3 disable themselves via cfg gates. Scope and Phases sections are empty, dod4 has no receipt-side validation, and the cli-tool "already implemented" framing is misleading because cli-tool's current behavior is the same hard-reject as the in-scope boundaries. Spec needs to choose a positive channel (or explicitly declare itself a regression-lock-only spec), fix all four validation commands, and add phases/rollback before approval.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/Cargo.toml:18-26
  - Result: passed
  - Evidence: Spec references crates/Cargo.toml workspace and -p runx-runtime: workspace at crates/Cargo.toml:1-12 lists runx-runtime; runx-runtime/Cargo.toml:18-26 declares features mcp, cli-tool, external-adapter. All referenced source files in the implied scope exist: adapters/mcp/transport.rs, adapters/external_adapter.rs, outbox_provider.rs, adapters/cli_tool.rs, credentials.rs.
- command audit
  - Grounded in: code:crates/runx-runtime/tests/credential_delivery.rs:1
  - Result: failed
  - Evidence: v1 command `cargo test ... --features mcp --test mcp_server` runs tests/mcp_server.rs which has zero matches for secret_env|credential|GITHUB_TOKEN (confirmed by grep) — it cannot prove dod1. v2 `--test external_adapter` requires the external-adapter feature (tests/external_adapter.rs:1: `#![cfg(feature = "external-adapter")]`) but the spec command omits `--features external-adapter`, so the file compiles to zero tests and reports vacuous success. v3 `--test credential_delivery` is gated by `#![cfg(all(feature = "cli-tool", feature = "mcp"))]` (credential_delivery.rs:1) but the command supplies neither feature, so the actual MCP/cli-tool rejection assertions never run. The real MCP rejection test (`mcp_process_transport_rejects_process_env_credential_delivery`) lives at credential_delivery.rs:271, not in mcp_server.rs.
- scope/migration audit
  - Grounded in: spec_gap:scope
  - Result: failed
  - Evidence: Context manifest shows Scope And Touchpoints rendered=0 body=0 and Planned Phases rendered=0 body=0 — the spec contains no scoped touchpoints and no planned phases. Summary claims credentials will be brokered by 'opaque references, scoped files, or a runtime-owned descriptor channel' but no phase, DOD, validation, or scope entry chooses or specifies that channel. The current implementation (adapters/mcp/transport.rs:165-169, external_adapter.rs:301-303, outbox_provider.rs:93-95, cli_tool.rs:34) hard-rejects secret_env at every supervised boundary; tests/local_credential_provision.rs:18-65 confirms even cli-tool with a local descriptor is rejected before spawn. So both 'in-scope' and 'out-of-scope' boundaries behave identically today, making the in/out-of-scope split inside the spec arbitrary.
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: failed
  - Evidence: All four DODs name supervised process boundaries and an observation/receipt invariant, but only three of the four validations cover delivery boundaries and the fourth (v4) is a grep review. dod4 (`Receipts record credential handle/observation metadata without leaking secret material`) has no validation directly inspecting a sealed receipt; CredentialDelivery::public_observation is populated in credentials.rs:398-430 and exercised by skill_run.rs:739-743 but no acceptance check confirms a receipt round-trip carries the observation without secret material. Validations also lack timeouts and phase assignments (no phases exist), so there is no defined point at which they should be re-run after implementation.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: failed
  - Evidence: Spec body shows only `Profile: strict` under Acceptance; there is no Rollback or Repair section. The Summary introduces three candidate channels (opaque refs / scoped files / runtime descriptor) each with different failure modes (e.g. scoped file leaking to siblings, descriptor inheritance through fork, broker compromise). Without a chosen channel and an explicit rollback (e.g. 'if scoped file appears outside per-run dir, fail the run and unlink path X'), an operator hitting a misconfiguration in a future positive-delivery rollout has no recipe to restore the fail-closed baseline that exists today.
- design challenge
  - Grounded in: code:crates/runx-runtime/src/credentials.rs:311
  - Result: failed
  - Evidence: Today CredentialDelivery::reject_process_env_boundary (credentials.rs:311-321) is called by every supervised adapter and there is no alternate delivery path. SkillInvocation construction in execution/runner/steps.rs:96, 472, 676 passes CredentialDelivery::none() for all graph step adapters, and execution/skill_run.rs:699-710 only constructs a non-none delivery for the rejected local cli-tool path. So the runtime presently has *no* credential delivery channel for MCP/external/outbox at all — admitted credentials cannot reach those adapters. The spec's DODs lock that fail-closed state down, but its Summary promises an actual delivery mechanism, and no part of the spec resolves that tension. This is either a short-sighted bandaid (lock the regression and ship nothing usable) or it is missing the architectural decision (which channel? who owns brokering? how do receipts attest delivery?) that makes the spec coherent.

Issues:
- [critical/blocks approval] `harden-1` command audit - v1 validation command runs the wrong test file; mcp_server.rs has zero MCP credential rejection coverage.
  - Status: open
  - Grounded in: code:crates/runx-runtime/tests/mcp_server.rs
  - Evidence: v1 command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_server`. A grep across tests/mcp_server.rs for `secret_env|credential|ProcessEnv|secret_token|GITHUB_TOKEN` returns zero matches. The actual MCP process-env rejection assertion is `mcp_process_transport_rejects_process_env_credential_delivery` at tests/credential_delivery.rs:271-295, which v1 does not run. Result: v1 passes without proving dod1.
  - Recommendation: Retarget v1 at `--test credential_delivery` (or a dedicated `--test mcp_credential_delivery`) and add `--features "cli-tool mcp"` so the cfg-gated test actually compiles. If v1 should remain in mcp_server.rs, add a regression test there that exercises ProcessMcpTransport with a non-empty CredentialDelivery and asserts CredentialProcessEnvUnsupported.
- [critical/blocks approval] `harden-2` command audit - v2 and v3 validation commands omit required feature flags, so both test files compile to zero tests and pass vacuously.
  - Status: open
  - Grounded in: code:crates/runx-runtime/tests/external_adapter.rs:1
  - Evidence: tests/external_adapter.rs:1 declares `#![cfg(feature = "external-adapter")]`; v2 command supplies no `--features external-adapter`, so the entire file is excluded. tests/credential_delivery.rs:1 declares `#![cfg(all(feature = "cli-tool", feature = "mcp"))]`; v3 command supplies neither feature, so all 11 assertions including the MCP/cli-tool rejection tests are excluded. Cargo will report success because zero tests ran.
  - Recommendation: Update v2 to `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter --test external_adapter`. Update v3 to `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features "cli-tool mcp" --test credential_delivery`. Consider adding a workspace-level smoke that asserts each gated test file actually runs > 0 tests under its feature set.
- [high/blocks approval] `harden-3` scope/migration audit - Summary promises a positive credential delivery channel; DODs, scope, and phases only encode the negative invariant. The promised channel is never chosen.
  - Status: open
  - Grounded in: spec_gap:summary
  - Evidence: Summary: `Credentials crossing supervised process boundaries must be brokered by opaque references, scoped files, or a runtime-owned descriptor channel.` DODs 1-3 only assert the negative (`does not expose raw secrets through ambient child environment`), which is already true today (credentials.rs:311 + reject calls at mcp/transport.rs:165, external_adapter.rs:301, outbox_provider.rs:93, cli_tool.rs:34). No DOD, scope entry, validation, or phase picks one of the three candidate channels or describes how MCP/external/outbox processes will actually receive a credential reference, who resolves it, or how the receipt observation is bound to the delivery.
  - Recommendation: Either (a) reclassify the spec as a regression-lock for the already-shipped fail-closed state, drop the positive-channel language from the Summary, and add an explicit out-of-scope note pointing to a future spec that selects the channel; or (b) keep the positive-channel scope and add a chosen channel, a delivery contract (resolver ownership, lifetime, scoping), and DODs/validations for the positive path (handle reaches process, secret never serialized, receipt observation references the handle).
  - Question: Is this spec only locking down the existing fail-closed state, or is it also introducing the positive delivery channel mentioned in the Summary?
  - Recommended answer: Pick (a): split positive-channel work into a follow-up spec and rewrite the Summary to match the regression-lock scope. The negative invariant is already enforced and benefits from explicit tests; positive delivery is an architectural decision that deserves its own spec.
  - If unanswered: Document the spec as regression-lock only and remove the unfulfilled Summary promise.
- [high/blocks approval] `harden-4` scope/migration audit - Spec has no Scope/Touchpoints entries and no Planned Phases; it is not executable as drafted.
  - Status: open
  - Grounded in: spec_gap:phases
  - Evidence: Context Budget Manifest shows `scope` rendered=0 body=0 and `phases` rendered=0 body=0; the rendered spec under `## Scope` lists only in/out bullets without code touchpoints, and there is no `## Phases` section at all. AGENTS.md / CLAUDE.md require phases to be executed in order with acceptance criteria; with no phases, `scafld build` cannot open a coherent unit of work.
  - Recommendation: Add at minimum: (1) Touchpoints listing the files to be edited (credentials.rs, adapters/mcp/transport.rs, adapters/external_adapter.rs, outbox_provider.rs, adapters/cli_tool.rs, execution/skill_run.rs, execution/runner/steps.rs, tests files), and (2) Phases such as `scope`, `model`, `materialize`, `verify`, `ratify` with phase-scoped acceptance. If the spec is regression-lock-only, a single `verify` phase with the corrected validations may suffice — but it must be declared.
- [medium/blocks approval] `harden-5` acceptance timing audit - dod4 (receipts record credential observation without secret material) has no acceptance validation that inspects a receipt.
  - Status: open
  - Grounded in: spec_gap:acceptance.dod4
  - Evidence: DOD4: `Receipts record credential handle/observation metadata without leaking secret material.` Validations v1-v4 cover delivery boundary behavior and a grep audit. None of them open a sealed receipt and assert that `credential_delivery_observations` is present, contains opaque refs, and contains no plaintext secret material. CredentialDelivery::public_observation (credentials.rs:333) and add_credential_delivery_metadata (external_adapter.rs:614-631) write the observation, but no test in the validations directly asserts the receipt-side invariant for the three in-scope adapters.
  - Recommendation: Add v5 that runs a credential_delivery receipt test under the right features (e.g. `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features "cli-tool mcp external-adapter" --test credential_delivery -- receipt`) and asserts: (a) sealed receipt JSON contains `credential_delivery_observations`, (b) every observation's `credential_refs[].uri` is `runx:credential:...`, (c) the receipt JSON does not contain the raw token value seeded by the test fixture. Without this, dod4 is unverified.
- [medium/advisory] `harden-6` rollback/repair audit - Spec has no Rollback or Repair section; future channel-selection work has no defined recovery to the current fail-closed state.
  - Status: open
  - Grounded in: spec_gap:rollback
  - Evidence: Spec body under `## Acceptance` only sets `Profile: strict`; there is no `## Rollback` block. The Summary's three candidate channels each carry distinct failure modes (scoped file lingering after crash, descriptor inheritance across fork, broker compromise). Today, repair is trivial because every boundary is fail-closed; once a positive channel exists, an explicit repair recipe is needed.
  - Recommendation: Add a Rollback section that names the current fail-closed defaults (every adapter calls reject_process_env_boundary) as the repair target, and for whichever channel is later chosen, lists the operator-level recovery (e.g. `unlink per-run credential dir`, `flush in-memory broker cache`, `re-issue grant`).
- [medium/advisory] `harden-7` design challenge - Out-of-scope claim for `cli-tool` is misleading; cli-tool today rejects local-descriptor credentials at the same fail-closed boundary as the in-scope adapters.
  - Status: open
  - Grounded in: code:crates/runx-runtime/tests/local_credential_provision.rs:1-65
  - Evidence: Spec says `cli-tool env secret rejection, already implemented` (out of scope). cli_tool.rs:34 calls `credential_delivery.reject_process_env_boundary("cli-tool")` and tests/local_credential_provision.rs:18-65 documents `cli-tool execution no longer accepts process-env local credentials. This keeps the secret boundary fail-closed until a non-env delivery channel exists.` So cli-tool, MCP, external-adapter, and outbox-provider are all in the same fail-closed state. Treating cli-tool as 'done' while the others are 'in scope' creates a false design split — the actual missing work (the non-env channel referenced in the local_credential_provision test header) is shared across all four adapters.
  - Recommendation: Either bring cli-tool into scope as the reference channel implementation (it already has the only positive-delivery integration point via LocalCredentialDescriptor in execution/skill_run.rs:699-710), or explicitly clarify that cli-tool is excluded because it is unused in the supervised graph runner today (steps.rs always passes CredentialDelivery::none()). Update the Summary so that 'cli-tool is implemented' does not imply a positive delivery channel already exists for it.
- [low/advisory] `harden-8` command audit - v4 grep pattern produces noisy matches against unrelated `process_env` helpers and the safe `secret_env()` accessor.
  - Status: open
  - Grounded in: spec_gap:acceptance.v4
  - Evidence: Pattern: `CredentialDelivery::ProcessEnv|secret_env\(|\.envs\(secret_env|process_env`. In current code this matches: credentials.rs:307 (the safe `secret_env(&self)` accessor), adapters/mcp/adapter.rs:132 (`secret_env: credential_delivery.secret_env().clone()` — legitimate field name plus accessor), runner.rs:67 (`from_process_env`), harness/runner.rs:127, dev/tool.rs:200 (`fn process_env`), external_adapter.rs:816 (`fn process_env`). A reviewer must hand-classify ~7 unrelated hits per run, which weakens the audit signal.
  - Recommendation: Tighten the pattern to actually-dangerous shapes, e.g. `\.envs\(.*\bsecret_env\b|\.env\(.*\bsecret_env\b|CredentialDelivery::ProcessEnv\b`, and document an allowlist of legitimate sites (the accessor in credentials.rs and the McpToolCallRequest field). Optionally promote the grep to a build-time deny lint or a `cargo test --test` so it cannot be silently broken.

### round-2

Status: passed
Started: 2026-05-26T02:55:18Z
Ended: 2026-05-26T02:55:18Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 verification: all four round-1 blockers are resolved. The spec is now an explicit regression-lock with no positive-channel promise; Scope, Touchpoints, Phases, and Rollback are populated; validation commands target real, cfg-gated test functions with the correct feature flags. I confirmed each named test exists at the cited site: mcp_process_transport_rejects_process_env_credential_delivery (tests/credential_delivery.rs:309), public_observation_metadata_serializes_without_secret_material (tests/credential_delivery.rs:124), external_adapter_process_supervisor_rejects_process_env_credential_delivery (tests/external_adapter.rs:368), external_adapter_skill_adapter_projects_public_credential_refs_and_observation (tests/external_adapter.rs:398), provider_process_rejects_process_env_credential_delivery (tests/thread_outbox_provider.rs:141), provider_process_pushes_idempotently_and_injects_delivery_observation (tests/thread_outbox_provider.rs:22). The v7 grep pattern produces zero matches across src and tests, satisfying the no_matches expectation. The pre-spawn rejection invariant is verified at credentials.rs:311, cli_tool.rs:34, outbox_provider.rs:94, external_adapter.rs:302, and mcp/transport.rs:165 (the MCP boundary uses a sibling `secret_env.is_empty()` check rather than `reject_process_env_boundary` directly — same pre-spawn semantics, but the rollback wording should be tightened). One low/advisory issue is filed against that wording; nothing blocks approval.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/Cargo.toml:18-26
  - Result: passed
  - Evidence: Cargo.toml features confirm cli-tool, mcp, and external-adapter exist (Cargo.toml:21-26). All Touchpoint paths exist: credentials.rs, adapters/mcp/transport.rs, adapters/external_adapter.rs, outbox_provider.rs, tests/credential_delivery.rs, tests/external_adapter.rs, tests/thread_outbox_provider.rs (verified via Glob/Grep). The spec correctly omits cli_tool.rs from Touchpoints because cli-tool is explicitly declared out of scope as a reference fail-closed boundary.
- command audit
  - Grounded in: code:crates/runx-runtime/tests/credential_delivery.rs:1
  - Result: passed
  - Evidence: tests/credential_delivery.rs:1 declares `#![cfg(all(feature = "cli-tool", feature = "mcp"))]`; v1 and v4 supply `--features "cli-tool mcp"` and target `--test credential_delivery` with `-- --exact` names that exist at lines 309 and 124 respectively. tests/external_adapter.rs:1 declares `#![cfg(feature = "external-adapter")]`; v2 and v5 supply `--features external-adapter` and target real names at lines 368 and 398. tests/thread_outbox_provider.rs has no cfg gate; v3 and v6 target real names at lines 141 and 22. v7 grep pattern `\.envs\([^\n]*secret_env|\.env\([^\n]*secret_env|CredentialDelivery::ProcessEnv` returns no matches across crates/runx-runtime/src and tests.
- scope/migration audit
  - Grounded in: spec_gap:scope
  - Result: passed
  - Evidence: Round-1 issue (empty Scope and Phases) is closed. Scope section enumerates four supervised boundaries plus public observation metadata; Touchpoints list the runtime source files and the three test files; Phases 1-3 (Contract Correction, Runtime Regression Lock, Scafld Review And Completion) are defined with status, dependencies, objectives, changes, and acceptance. Summary explicitly disclaims the positive non-env delivery channel and defers it to a separate spec, eliminating the round-1 promise/implementation mismatch.
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: Four DODs map onto seven validations. dod1 -> v1 (MCP rejection), dod2 -> v2 (external-adapter rejection), dod3 -> v3 (outbox-provider rejection), dod4 -> v4 (observation serialization), v5 (external-adapter public refs), v6 (outbox-provider observation), plus v7 grep audit. Round-1 dod4 gap (no observation-receipt-side check) is closed by v4's assertion (credential_delivery.rs:152-157) that the serialized observation contains opaque refs and material hashes but not the raw secret or env var. Phase 2 acceptance binds the validations to runtime regression-lock; Phase 1 acceptance binds harden+validate to the contract correction. Profile is strict.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback section is now populated with three credible recipes: (1) restore pre-spawn rejection if a runtime edit regresses the invariant; (2) remove only the newly added observation serialization test if it proves flaky; (3) fail or cancel this spec and write a separate design spec if review demands positive delivery now. This matches the regression-lock framing — the safe state is the current code, and rollback returns to that state. One minor wording inaccuracy is flagged as advisory.
- design challenge
  - Grounded in: code:crates/runx-runtime/src/credentials.rs:311
  - Result: passed
  - Evidence: Round-1 raised whether regression-locking a fail-closed-with-no-delivery-channel is the right architectural move or a short-sighted bandaid. Operator response is materialized in the spec: this is a deliberately bounded regression-lock with positive delivery deferred to a separate architectural decision (Summary lines 32-36, Scope out-of-scope, Rollback bullet 3). The reject_process_env_boundary contract at credentials.rs:311 is the durable invariant the spec is locking down. Accepted as the right move because (a) it closes a real correctness risk with non-vacuous tests, (b) it keeps the positive-channel design unhurried, and (c) it does not paint future channel work into a corner — observations already carry opaque refs that a future channel can resolve.

Issues:
- [low/advisory] `harden-1` rollback/repair audit - Rollback bullet 1 names `reject_process_env_boundary` as the call to restore for MCP, but the MCP transport actually rejects via a sibling `request.secret_env.is_empty()` guard at transport.rs:165 — not the shared helper.
  - Status: open
  - Grounded in: code:crates/runx-runtime/src/adapters/mcp/transport.rs:165
  - Evidence: credentials.rs:311 defines `reject_process_env_boundary` and it is called by cli_tool.rs:34, outbox_provider.rs:94, and external_adapter.rs:302. mcp/transport.rs:165-169 instead does `if !request.secret_env.is_empty() { return Err(McpTransportError::failed("MCP process credential delivery must use structured credential refs, not ambient child environment.")); }` before `spawn_tokio_mcp_server` at line 170. Functionally equivalent (pre-spawn rejection, secret-free error message), but the rollback wording would mislead an operator who tries to literally `git grep reject_process_env_boundary` in the MCP transport to verify the restore.
  - Recommendation: Rewrite rollback bullet 1 to: "restore the pre-spawn credential-delivery rejection at each supervised boundary (the `reject_process_env_boundary` calls in cli_tool.rs, outbox_provider.rs, and external_adapter.rs, and the `secret_env.is_empty()` guard in mcp/transport.rs:165) before retrying." Optionally consider a follow-up to unify MCP onto `reject_process_env_boundary` so the invariant has a single named touchpoint.
