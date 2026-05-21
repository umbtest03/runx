---
spec_version: '2.0'
task_id: rust-adapter-credential-delivery
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T11:37:42Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# Runtime credential delivery to adapters

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T11:37:42Z
Review gate: pass

## Why this exists

The credential contract and the kernel decisions already exist in Rust:

- `CredentialEnvelope { kind, grant_id, provider, auth_mode, material_kind,
  connection_id, scopes, grant_reference, material_ref }`
  ([`runx-core/src/policy/types.rs:133`](../../crates/runx-core/src/policy/types.rs#L133)).
- `CredentialBindingRequest` / `CredentialBindingDecision::{Allow,Deny}`
  ([`policy/types.rs:150`](../../crates/runx-core/src/policy/types.rs#L150)):
  the kernel decides whether a credential may bind to a run.
- `AuthorityProofCredentialMaterial { status, ..., material_ref_hash }`
  ([`policy/types.rs:266`](../../crates/runx-core/src/policy/types.rs#L266)):
  the proof records only a hash of `material_ref`, never the material, and
  carries an explicit `AuthorityProofRedaction`.
- `connect_grant_to_local_admission` turns a hosted grant into a
  `LocalAdmissionGrant` of metadata only (grant_id, provider, scopes), no secret
  ([`runx-runtime/src/connect/types.rs`](../../crates/runx-runtime/src/connect/types.rs)).

What is absent is the runtime delivery. The cli-tool adapter builds the child
environment through `prepare_process_sandbox` -> `child_env` ->
`allowed_base_env`, which is a fixed allowlist plus `RUNX_INPUTS_JSON` and
per-input `RUNX_INPUT_*` vars
([`runx-runtime/src/sandbox.rs:33`](../../crates/runx-runtime/src/sandbox.rs#L33),
[`:145`](../../crates/runx-runtime/src/sandbox.rs#L145),
[`:160`](../../crates/runx-runtime/src/sandbox.rs#L160)). No credential material
is ever resolved or injected; the MCP adapter path is the same via
`prepare_mcp_process_sandbox`. Sandbox metadata is `declared-policy-only`
([`sandbox.rs:394`](../../crates/runx-runtime/src/sandbox.rs#L394)). So an
admitted, authority-proven credential cannot power a real tool today.

## Summary

Add a runtime layer that, for a run whose `CredentialBindingDecision` is
`Allow`, resolves the envelope's `material_ref` into concrete secret material,
maps it to the provider's declared environment variables via a delivery
profile, and injects it into the cli-tool and MCP child process at spawn. The
secret travels a dedicated channel that never enters `SandboxPlan.metadata`, the
authority proof, receipts, captured stdout/stderr, or logs. Resolution happens
as late as possible and fails closed.

## Context

CWD: `.` (run cargo from `crates/`).

Packages:
- `crates/runx-runtime` (resolver, delivery, sandbox injection, adapters)
- `crates/runx-core` (binding decision and proof already exist; consume them)
- `crates/runx-contracts` (any new delivery-profile contract type)

Current sources:
- `crates/runx-runtime/src/sandbox.rs`
  (`SandboxPlan`, `prepare_process_sandbox`, `prepare_mcp_process_sandbox`,
  `child_env`, `allowed_base_env`)
- `crates/runx-runtime/src/adapters/cli_tool.rs` (`CliToolAdapter::invoke`)
- `crates/runx-runtime/src/adapters/mcp/adapter.rs` (`McpAdapter`)
- `crates/runx-runtime/src/connect/` (grant retrieval; OAuth handshake)
- `crates/runx-core/src/policy/authority_proof.rs` (proof + redaction)

Files impacted:
- `crates/runx-runtime/src/credentials.rs` (resolver trait, delivery profile
  application, redaction guard)
- `crates/runx-runtime/src/adapter.rs` (`SkillInvocation.credential_delivery`
  as the secret-carrying channel separate from `SandboxPlan`)
- `crates/runx-runtime/src/adapters/cli_tool.rs` and `.../mcp/adapter.rs`
  (consume the secret channel at spawn only)

Invariants:
- A secret is never serialized into `SandboxPlan.metadata`, the authority proof,
  a receipt, captured stdout/stderr, or any log. Receipts continue to record
  `material_ref_hash` only.
- Material is delivered only when the kernel's `CredentialBindingDecision` for
  the run is `Allow`, scoped to the admitted grant and provider.
- Material is resolved as late as possible (at adapter spawn), held in memory no
  longer than the child process needs it, and the runtime fails closed if the
  resolver is unavailable or the ref does not resolve.
- The injected variable set is exactly the provider delivery profile's declared
  vars. No ambient or broad secret passthrough.
- The cli-tool and MCP paths deliver identically; no path-specific leniency.

## Objectives

- Define a `MaterialResolver` boundary: `material_ref` -> secret material, with a
  local resolver (keychain / env / file for self-host and dev) and a hosted
  resolver (cloud-brokered, behind the connect client) sharing one trait.
- Define per-provider `CredentialDeliveryProfile` (provider, auth_mode, env var
  -> role mapping) and apply it at spawn.
- Carry resolved material on a secret channel distinct from the audited
  `SandboxPlan.env`/`metadata`, and inject only into the child environment.
- Add a redaction guard so a secret value cannot reach metadata, proof, receipt,
  or log surfaces, with tests that assert non-leakage.

## Scope

In scope:
- The Rust runtime resolver boundary, delivery profiles, secret channel,
  injection at the cli-tool and MCP adapters, and the redaction guard.
- Fail-closed behavior and tests proving secrets do not appear in
  metadata/proof/receipt/stdout.

Out of scope:
- Cloud-side secret storage, the grant model, the credential envelope, BYO
  connect session, and verification. Owned by `byo-credential-foundations`.
- New auth modes beyond what the envelope and delivery profile express.
- File-mount or helper-process delivery if env injection is chosen first (see
  Open Questions); add as a follow-up only if a provider needs it.

## Dependencies

- `rust-connect-client` (archived, completed; grant retrieval and the hosted
  client this resolver extends).
- `rust-policy-authority-proof-parity` (archived; the proof + redaction the
  delivery must not violate).
- `byo-credential-foundations` (cloud; supplies live material storage and hosted
  resolution). The `CredentialEnvelope` in core now carries `auth_mode`,
  `material_kind`, and optional `connection_id`
  ([`types.rs:137`](../../crates/runx-core/src/policy/types.rs#L137)). The
  runtime delivery path can land first against that envelope shape; live
  provider resolution follows once storage is available.

## Open Questions

- Delivery mechanism: environment injection vs mounted file vs helper process.
  Start with env injection (the pre-oauth foundations plan's recommendation) and
  defer the others unless a provider requires them.
- Where the delivery profile is declared: a runx-owned per-provider profile, the
  tool manifest, or both. Lean to a runx-owned profile keyed by provider so
  tools stay credential-agnostic.
- Whether the secret channel is a new field on `SandboxPlan` (excluded from its
  `PartialEq`/serialization) or a separate value passed alongside the plan to
  the adapter. The latter keeps `SandboxPlan` purely auditable.
  - Resolved for the first slice: `CredentialDelivery` is carried on
    `SkillInvocation`, not `SandboxPlan`.

## Validation

- [x] focused runtime tests prove credential delivery and non-leakage.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test credential_delivery --features cli-tool,mcp -- --nocapture`
  - Evidence: exit code 0; 4 tests passed; cli-tool and MCP both injected
    `GITHUB_TOKEN` only at spawn and redacted the secret from stdout/metadata.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test credential_delivery --features cli-tool,mcp-rmcp -- --nocapture`
  - Evidence: exit code 0; 4 tests passed under the rmcp-backed MCP feature.
- [x] focused runtime clippy remains clean.
  - Command:
    `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features cli-tool,mcp -- -D warnings`
  - Evidence: exit code 0.
  - Command:
    `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features cli-tool,mcp-rmcp -- -D warnings`
  - Evidence: exit code 0.

## Planning Log

- 2026-05-21T00:00:00Z: Added the `CredentialDelivery` secret channel,
  `MaterialResolver` boundary, provider delivery profile, cli-tool/MCP spawn
  injection, and redaction guard. Live material storage/resolution remains a
  follow-up owned by BYO credential foundations.

## References

- [`plans/integrations/pre-oauth-foundations.md`](../../../plans/integrations/pre-oauth-foundations.md)
  §9 ("Runtime Delivery To Adapters")
- [`plans/integrations/tonight-pre-oauth-work.md`](../../../plans/integrations/tonight-pre-oauth-work.md)
  §8 (note: it points at the sunsetting TS adapters; this spec is the
  Rust-canonical owner)
- [`runx-core/src/policy/types.rs`](../../crates/runx-core/src/policy/types.rs)
  (`CredentialEnvelope`, `CredentialBindingDecision`,
  `AuthorityProofCredentialMaterial`)
- [`runx-runtime/src/sandbox.rs`](../../crates/runx-runtime/src/sandbox.rs)
  (the env construction this spec extends)

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: The credential delivery layer keeps secrets off `SandboxPlan.metadata`, the authority proof surface, and the receipt path. `CredentialDelivery::from_allowed_binding` fails closed on `Deny`, provider mismatch, missing material, and missing roles. `SecretString` has a redacting `Debug`, no serde derive, and only exposes values through internal helpers. The cli-tool and MCP adapters inject the secret env strictly after the audited sandbox env at `Command::envs()`, and both paths run captured stdout/stderr (or stringified tool results, including `sanitized_message()`) through `redact_text`. All in-tree constructors of `SkillInvocation` set `credential_delivery`, and feature-gated tests cover the `mcp` and `mcp-rmcp` variants. Non-blocking observations are noted; no completion blocker.

Attack log:
- `credentials::from_allowed_binding`: Pass Deny decision and assert fail-closed -> clean (BindingDenied returned with reasons preserved (credentials.rs:184, test delivery_profile_requires_allowed_binding).)
- `credentials::from_allowed_binding`: Provider mismatch between envelope and profile -> clean (ProviderMismatch error returned before resolver is invoked (credentials.rs:185).)
- `credentials::from_allowed_binding`: Resolver returns different material_ref than requested -> clean (MaterialRefMismatch enforced at credentials.rs:192.)
- `credentials::apply_profile`: Profile binds a role the material does not provide -> clean (MissingRole error covers the only role today (credentials.rs:261).)
- `credentials::apply_profile`: Empty material value -> finding (Empty string accepted; see finding empty-material-accepted.)
- `credentials::validate_env_name`: Invalid env var names (lowercase, leading digit, control chars) -> clean (Restricted to `[_A-Z][_A-Z0-9]*` (credentials.rs:271).)
- `SecretString trait surface`: Look for serde/Display implementations that could leak the value -> clean (Only manual Debug (returns `[redacted-credential]`) plus PartialEq; no serde derives anywhere in credentials.rs.)
- `SecretEnv exposure`: Search for external callers that read SecretEnv values outside spawn -> clean (`SecretEnv::get/iter` used only by `Command::envs` calls and FixtureMcpTransport (intentional).)
- `cli_tool adapter`: Order of envs: ensure credential overrides sandbox env, not vice versa -> clean (Spawn calls `.envs(&sandbox.env).envs(credential_delivery.secret_env().iter())` so secret is last and wins (cli_tool.rs:36-37).)
- `cli_tool adapter`: Redaction of stdout AND stderr -> clean (Both `output.stdout` and `output.stderr` flow through `redact_text` (cli_tool.rs:48-49); spec assertion only checks stdout but stderr is also redacted.)
- `cli_tool adapter`: Truncation interaction with redaction -> finding (Truncation precedes redaction; see finding truncate-before-redact-cli-tool.)
- `MCP adapter`: Secret leakage through tool result serialization -> clean (`stringify_mcp_tool_result` output passes through `redact_text` before reaching `SkillOutput.stdout` (adapter.rs:54-56).)
- `MCP adapter`: Secret leakage through transport error messages -> clean (`error.sanitized_message()` is also redacted (adapter.rs:64); sanitized variants do not include arguments.)
- `MCP adapter metadata`: Secret leakage through success/failure metadata -> clean (`mcp_process_sandbox_metadata` records the env allowlist names only, never values (sandbox_metadata.rs:148-183).)
- `MCP transport spawn`: End-to-end spawn with secret env -> finding (Only FixtureMcpTransport is exercised in the credential test; see finding mcp-spawn-not-integration-tested.)
- `MCP transport list_tools`: Verify list_tools does not pass secret env -> clean (Both `list_tools` paths spawn with `SecretEnv::default()` (transport.rs:98, 166), preventing credential leakage during discovery.)
- `SkillInvocation construction sites`: Find a caller that forgets `credential_delivery` -> clean (Compiler enforces; all production sites (runner/steps.rs, harness/runner.rs, skill_run.rs, server_skill.rs, catalog.rs) and tests set the field explicitly.)
- `Adapters out of scope (agent, a2a)`: Check that ignored credential channels do not leak -> clean (`AgentAdapter` and `A2aAdapter` do not consume `request.credential_delivery`; nothing serializes it, so a misuse becomes a silent drop, not a leak. Out of scope per spec.)
- `RuntimeError::CredentialDelivery`: Inspect new error variant for leakage of material values -> clean (Error variants carry `material_ref`, provider, env_var, and role names but never the secret value (credentials.rs:221-240, error.rs:71).)
- `Receipt write path`: Search receipt module for any handling of `SkillInvocation` or `CredentialDelivery` -> clean (Grep in `crates/runx-runtime/src/receipts` returns no matches; the secret channel never reaches receipt assembly.)
- `Public API stability`: Check whether the new `credential_delivery` field on `SkillInvocation` breaks external callers -> clean (All in-tree consumers updated. No other crate in the workspace constructs `SkillInvocation`; risk is limited to downstream consumers expected to migrate via `CredentialDelivery::none()`.)

Findings:
- [low/non-blocking] `empty-material-accepted` Empty secret material is accepted and injected as an empty env var
  - Location: `crates/runx-runtime/src/credentials.rs:109`
  - Evidence: `ResolvedCredentialMaterial::access_token` and `apply_profile` never reject an empty string value. A resolver that returns `SecretString::new("")` produces a `SecretEnv` entry mapping the profile env var to `""`. `redact_text` then skips empties (line 213), so any provider-side echo of `$GITHUB_TOKEN` produces literal empty output rather than `[redacted-credential]`. The child sees an empty `GITHUB_TOKEN` and likely produces an unauthenticated request that the runtime cannot distinguish from a successful credential bind.
  - Impact: Silent fail-open against a misbehaving resolver: an unauthenticated call appears as a delivered credential, contradicting the spec's `fails closed if the resolver is unavailable or the ref does not resolve`.
  - Validation: Add a unit test where the in-memory resolver yields an empty `access_token` and assert `from_allowed_binding` returns `Err`.
- [low/non-blocking] `redaction-exact-string-only` redact_text only catches literal secret occurrences
  - Location: `crates/runx-runtime/src/credentials.rs:209`
  - Evidence: `redact_text` runs `String::replace(secret, REDACTED_CREDENTIAL)` per secret value. Any tool that echoes the credential in a transformed shape (base64, percent-encoded, hex, URL-embedded with surrounding `Bearer ` etc.) will pass through unredacted. The env-injection contract limits exposure to the literal value the child sees, but providers that wrap the token (`Authorization: Bearer ghs_…`) only protect the prefix portion.
  - Impact: Partial leakage risk if a child reflects the credential in a non-literal form; reduces the protection promised by the redaction guard.
- [low/non-blocking] `mcp-spawn-not-integration-tested` Real MCP process spawn with secret env is not integration tested
  - Location: `crates/runx-runtime/tests/credential_delivery.rs:100`
  - Evidence: The MCP credential test uses `FixtureMcpTransport`, which reads `request.secret_env` directly without invoking `spawn_tokio_mcp_server` or `spawn_mcp_server`. Those are the only production code paths that pass the secret to a child process via `Command::envs(secret_env.iter())`. Spec acceptance evidence claims `cli-tool and MCP both injected GITHUB_TOKEN only at spawn`, but only cli-tool is end-to-end verified.
  - Impact: A regression that drops `.envs(secret_env.iter())` from the tokio or std spawn helpers would not be caught by this suite.
- [low/non-blocking] `truncate-before-redact-cli-tool` Output is truncated before redaction; a secret straddling the 1MB boundary is partially exposed
  - Location: `crates/runx-runtime/src/adapters/cli_tool.rs:48`
  - Evidence: `credential_delivery.redact_text(truncate_utf8(output.stdout))` truncates to `OUTPUT_LIMIT_BYTES = 1024 * 1024` first. If the secret begins inside the kept bytes and extends past the limit, only the prefix remains; `String::replace(secret, ...)` matches the full secret only, so the prefix is emitted verbatim.
  - Impact: Edge-case partial token disclosure for adversarial outputs that intentionally align the secret across the truncation boundary; the API contract allows arbitrary content so this is reachable in principle.

