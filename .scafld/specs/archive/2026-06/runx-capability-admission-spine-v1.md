---
spec_version: '2.0'
task_id: runx-capability-admission-spine-v1
created: '2026-06-10T01:08:05Z'
updated: '2026-06-10T01:51:38Z'
status: completed
harden_status: error
size: medium
risk_level: high
---

# Capability admission spine

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-10T01:51:38Z
Review gate: pass

## Summary

Make the Tier 0 governance promise true for local Rust execution: privileged
runtime capabilities are admitted against operator-granted authority before
they can affect child processes, managed-agent tool calls, provider-scope
effects, private-network HTTP, or MCP HTTP exposure.

This is the precondition layer. It intentionally does not implement the later
receipt-proof, agent-loop demotion, payment-authority, or cloud-authz tiers.
Those depend on this pass because receipts cannot prove governance if
privileged effects can self-grant before admission.

## Objectives

- Preserve the existing Rust ownership boundary:
  - pure admission decisions stay in `runx-core::policy`,
  - runtime-side enforcement stays in `runx-runtime`,
  - CLI owns presentation and explicit operator flags only.
- Close self-grant and key-leak holes:
  - receipt signing seed env is never reachable by child processes or hosted
    tool/MCP subprocesses after signer construction,
  - reserved runx env names are rejected consistently at parser and runtime
    env-admission seams,
  - skill/graph authors cannot place operator-only approvals into manifests.
- Make `allowed_tools` a hard boundary:
  - parser rejects absolute paths, path separators, `..`, empty values, and
    manifest-like paths in agent `allowed_tools`,
  - runtime rejects model-selected tools outside `allowed_tools` before any
    local resolution,
  - model-selected tool resolution never accepts explicit filesystem manifest
    paths even when the string is allowlisted.
- Keep provider scopes operator/grant sourced:
  - `provider_permission.granted_scopes` remains rejected from graph policy,
  - granted scopes require an explicit operator grant id and are recorded in
    the admission context,
  - defaults that pretend a grant exists are removed from the enforcing path.
- Keep private-network access operator gated:
  - `source.http.allow_private_network: true` is only a request,
  - runtime must require an operator grant/approval signal before constructing
    a private-network-capable transport,
  - outbound HTTP still fails closed by default for loopback, link-local,
    private, metadata, NAT64, and other non-public targets.
- Harden MCP HTTP exposure:
  - loopback remains the default,
  - non-loopback remains an explicit CLI/operator opt-in,
  - bearer authentication remains mandatory and tested.
- Add a small, reusable admission surface only where it removes drift; do not
  build a generic governance framework or duplicate existing policy modules.

## Scope

In scope:

- `runx-core::policy` pure helpers for reusable admission predicates:
  reserved env names and agent tool refs.
- `runx-parser` validation for skill and graph `allowed_tools` plus sandbox
  env allowlists.
- `runx-runtime` enforcement at impure seams:
  runtime env construction, skill execution env, sandbox child env, managed
  agent tool execution, provider permission effect, HTTP adapter, runtime HTTP
  transport, MCP HTTP server.
- `runx-cli` MCP operator flag parsing/presentation only if needed to preserve
  the loopback/auth/non-loopback contract.
- Focused tests and fixtures proving self-grant attempts fail closed.
- Documentation notes in the Rust security/runtime docs if a new helper
  boundary is introduced.

Out of scope:

- Tier 1 receipt-proof changes: offline `runx verify`, tree integrity changes,
  and per-effect grant ids in sealed receipt nodes.
- Tier 2 agent-loop demotion or adapter relocation.
- Tier 3 `runx-pay` spend-cap/SPT/refund hardening.
- Tier 4 hosted/cloud authz and billing hardening.
- Changing public JSON schemas unless a hardening pass proves the existing
  contract cannot express the required denial.
- Broad cargo workspace cleanup or unrelated active spec work.

## Dependencies

- Rust local runtime is the authority for advertised native local behavior per
  [docs/trusted-kernel-package-truth.md](/Users/kam/dev/runx/runx/oss/docs/trusted-kernel-package-truth.md).
- Existing local admission and sandbox policy in
  [crates/runx-core/src/policy](/Users/kam/dev/runx/runx/oss/crates/runx-core/src/policy).
- Existing runtime env/signing service split in
  [crates/runx-runtime/src/services/receipts.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/services/receipts.rs).
- Existing runtime process sandbox env path in
  [crates/runx-runtime/src/sandbox/env.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/sandbox/env.rs).
- Existing managed-agent tool executor in
  [crates/runx-runtime/src/adapters/agent_tools.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/agent_tools.rs).
- Existing provider-permission effect in
  [crates/runx-runtime/src/effects/provider_permission.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/effects/provider_permission.rs).
- Existing governed HTTP adapter and transport in
  [crates/runx-runtime/src/adapters/http.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/http.rs) and
  [crates/runx-runtime/src/runtime_http.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/runtime_http.rs).
- Existing MCP HTTP server in
  [crates/runx-runtime/src/adapters/mcp/http_server.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/mcp/http_server.rs).

## Grounding Evidence

- `RuntimeOptions::from_env` constructs receipt services, then calls
  `strip_receipt_signing_env`. [crates/runx-runtime/src/execution/runner.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/execution/runner.rs:75)
- `execute_skill_run_with_overrides` constructs `ReceiptServices`, then strips
  receipt signing env before building `WorkspaceEnv`. [crates/runx-runtime/src/execution/skill_run.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/execution/skill_run.rs:92)
- Child sandbox env already rejects reserved runx env names and retains an
  allowlist only. [crates/runx-runtime/src/sandbox/env.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/sandbox/env.rs:110)
- The pure reserved-env predicate already exists in policy and covers
  `RUNX_RECEIPT_SIGN_*` plus secret-like `RUNX_*` names. [crates/runx-core/src/policy/sandbox.rs](/Users/kam/dev/runx/runx/oss/crates/runx-core/src/policy/sandbox.rs:35)
- The parser already rejects sandbox env allowlists that include receipt
  signing env names. [crates/runx-parser/tests/parser_sandbox.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/tests/parser_sandbox.rs:65)
- Skill-level `allowed_tools` validation currently only checks non-empty
  strings. [crates/runx-parser/src/skill/governance.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/src/skill/governance.rs:189)
- Graph step `allowed_tools` currently uses a raw optional string array rather
  than the skill validation helper. [crates/runx-parser/src/graph/step.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/src/graph/step.rs:59)
- Runtime managed-agent tool execution already rejects tools outside
  `allowed_tools` before local resolution and disables explicit manifest paths.
  [crates/runx-runtime/src/adapters/agent_tools.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/agent_tools.rs:52)
- Local tool inspect can resolve explicit manifest paths when allowed. That is
  correct for CLI/catalog use and must stay disabled for model-selected tools.
  [crates/runx-runtime/src/tool_catalogs/inspect.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/tool_catalogs/inspect.rs:204)
- Provider permission already rejects self-attested `granted_scopes` in graph
  policy and reads granted scopes from runtime env. [crates/runx-runtime/src/effects/provider_permission.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/effects/provider_permission.rs:72)
- Provider permission currently invents a fallback grant id
  `operator-provider-grant` when the env lacks one. That is not good enough for
  auditable authority. [crates/runx-runtime/src/effects/provider_permission.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/effects/provider_permission.rs:175)
- HTTP private-network access already requires both manifest intent and an
  operator env grant. [crates/runx-runtime/src/adapters/http.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/http.rs:292)
- Runtime HTTP blocks private hostnames/IP literals, uses a guarded DNS
  resolver, disables redirects, validates headers, caps response bodies, and
  uses timeouts. [crates/runx-runtime/src/runtime_http.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/runtime_http.rs:118)
- MCP HTTP defaults to `127.0.0.1:8080`, requires bearer auth, and requires
  explicit non-loopback opt-in. [crates/runx-runtime/src/adapters/mcp/http_server.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/mcp/http_server.rs:30)
- CLI MCP parsing preserves the non-loopback opt-in as an explicit flag.
  [crates/runx-cli/src/mcp.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/mcp.rs:20)

## Assumptions

- Operator-granted authority may currently arrive through env/CLI flags because
  hosted grant transport is outside this Tier 0 OSS pass.
- It is acceptable to add pure helper functions to `runx-core::policy` when the
  same admission predicate must be shared by parser and runtime.
- It is not acceptable to add a broad `CapabilityManager` that reimplements
  sandbox, credential, provider, or HTTP policy already owned elsewhere.
- Some listed audit items are already fixed on current main. This spec must
  preserve and prove them rather than rewrite them.
- Other agents may have long cargo tests running. Use
  `CARGO_TARGET_DIR=target/runx-capability-admission-spine` for focused Rust
  validation.

## Touchpoints

- [crates/runx-core/src/policy/sandbox.rs](/Users/kam/dev/runx/runx/oss/crates/runx-core/src/policy/sandbox.rs)
- [crates/runx-core/src/policy/types.rs](/Users/kam/dev/runx/runx/oss/crates/runx-core/src/policy/types.rs)
- [crates/runx-core/src/policy/mod.rs](/Users/kam/dev/runx/runx/oss/crates/runx-core/src/policy/mod.rs)
- [crates/runx-parser/src/skill/governance.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/src/skill/governance.rs)
- [crates/runx-parser/src/graph/step.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/src/graph/step.rs)
- [crates/runx-parser/tests/parser_sandbox.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/tests/parser_sandbox.rs)
- [crates/runx-parser/tests/parser_rejections.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/tests/parser_rejections.rs)
- [crates/runx-parser/tests/parser_fixtures.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/tests/parser_fixtures.rs)
- [crates/runx-parser/tests/parser_graph_allowed_tools.rs](/Users/kam/dev/runx/runx/oss/crates/runx-parser/tests/parser_graph_allowed_tools.rs)
- [crates/runx-runtime/src/execution/runner.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/execution/runner.rs)
- [crates/runx-runtime/src/execution/skill_run.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/execution/skill_run.rs)
- [crates/runx-runtime/src/sandbox/env.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/sandbox/env.rs)
- [crates/runx-runtime/src/adapters/agent_tools.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/agent_tools.rs)
- [crates/runx-runtime/src/effects/provider_permission.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/effects/provider_permission.rs)
- [crates/runx-runtime/src/adapters/http.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/http.rs)
- [crates/runx-runtime/src/runtime_http.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/runtime_http.rs)
- [crates/runx-runtime/src/adapters/mcp/adapter.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/mcp/adapter.rs)
- [crates/runx-runtime/src/adapters/mcp/transport.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/mcp/transport.rs)
- [crates/runx-runtime/src/adapters/mcp/server_skill.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/mcp/server_skill.rs)
- [crates/runx-runtime/src/adapters/mcp/http_server.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/adapters/mcp/http_server.rs)
- [crates/runx-cli/src/mcp.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/mcp.rs)
- [crates/runx-cli/tests/launcher.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/tests/launcher.rs)
- [docs/rust-kernel-architecture.md](/Users/kam/dev/runx/runx/oss/docs/rust-kernel-architecture.md) or a focused security doc if the admission boundary needs documentation.

## Risks

- Over-centralization can create a second policy engine. Mitigation: only add
  pure predicates or thin runtime helpers; do not duplicate existing sandbox,
  provider, credential, or HTTP logic.
- Parser rejection of unsafe `allowed_tools` may break existing fixtures that
  used path-like tool refs. Mitigation: fixtures should migrate to catalog refs;
  model-selected filesystem manifests are exactly the unsafe shape.
- Removing provider grant-id fallback can break tests or local examples that
  relied on implicit authority. Mitigation: update examples/tests to provide an
  explicit grant id; do not keep compatibility defaults.
- Outbound HTTP connect-time proof may require careful use of reqwest resolver
  APIs. Mitigation: keep implementation constrained to runtime HTTP transport
  tests and do not add new HTTP clients.
- MCP HTTP changes can affect local operator UX. Mitigation: preserve current
  bearer-token stderr guidance and loopback defaults.
- Workspace has unrelated dirt from other agents. Mitigation: do not touch
  unrelated files and avoid broad formatting commands outside affected crates.

## Acceptance

Profile: strict

Validation:
- `cd crates && cargo fmt --check`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-core policy::`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser parser_sandbox`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser allowed_tools`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser graph`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "agent catalog" agent_tools`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" provider_permission`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" http`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" runtime_http`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" mcp_server`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-cli mcp_http`
- `! rg -n "operator-provider-grant|policy\\.[A-Za-z0-9_]*granted_scopes|allow_private_network.*unwrap_or\\(true\\)|allow_explicit_manifest_path: true" crates/runx-runtime/src/effects/provider_permission.rs crates/runx-runtime/src/adapters/agent_tools.rs crates/runx-runtime/src/adapters/http.rs`
- `git diff --check`

## Phase 1: Pure Admission Predicates

Status: completed
Dependencies: none

Objective: Put shared admission predicates in the right pure owner without

Changes:
- Keep `is_reserved_runx_sandbox_env_name` as the reserved env authority; add small pure helpers only if parser/runtime currently duplicate logic.
- Add a pure, reusable agent-tool-ref predicate for `allowed_tools` entries and model-selected tool names: and shell-ish values, catalogs.
- Add focused unit tests proving accepted catalog refs and rejected manifest/path refs.
- Do not change public wire shape; this is validation logic only.

Acceptance:
- [x] `ac1` command - Core policy tests cover admission predicates
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-core policy::`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6

## Phase 2: Parser and Runtime Tool Boundary

Status: completed
Dependencies: phase1

Objective: Make `allowed_tools` a parse-time and runtime boundary, not a

Changes:
- Route skill `runx.allowed_tools`, runner-level `runx.allowed_tools`, and graph step `allowed_tools` through the same pure tool-ref predicate.
- Add parser tests for:
- Keep runtime managed-agent enforcement before local tool resolution.
- Add/extend runtime tests so even an allowlisted path-like tool string cannot resolve as an explicit manifest path from a model-selected tool call.
- Preserve catalog/CLI inspect behavior where human CLI callers explicitly pass a manifest path; only model-selected tools are constrained.

Acceptance:
- [x] `ac2` command - Parser sandbox tests still pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser parser_sandbox`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-11
- [x] `ac3` command - Parser graph/tool admission tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser allowed_tools`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-12
- [x] `ac4` command - Parser graph admission tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser graph`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13
- [x] `ac5` command - Runtime managed-agent tool boundary tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "agent catalog" agent_tools`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14

## Phase 3: Operator Grants for Provider and Private Network

Status: completed
Dependencies: phase2

Objective: Remove self-attested and implicit authority from provider scopes and

Changes:
- Provider permission: required scopes are admitted, scopes.
- Private-network HTTP: another policy engine,
- Signing env: options, skill workspace env, CLI-tool child env, MCP adapter subprocess env (`mcp/transport.rs`), and MCP-served skill/CLI-tool child env (`mcp/server_skill.rs` through the normal execution path). The MCP HTTP server process may still hold signer authority because it is the operator-started receipt sealer.

Acceptance:
- [x] `ac6` command - Provider permission grant tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" provider_permission`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19
- [x] `ac7` command - HTTP/private-network runtime tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" http`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20

## Phase 4: Network and MCP Exposure Guardrails

Status: completed
Dependencies: phase3

Objective: Preserve MCP HTTP auth/loopback semantics and harden outbound HTTP

Changes:
- Keep MCP HTTP bearer auth mandatory and loopback default unchanged.
- Keep `--http-allow-non-loopback` as the only CLI path that admits non-loopback binding.
- Add or extend MCP HTTP tests for:
- Harden runtime HTTP resolved-address admission: IP fails closed,

Acceptance:
- [x] `ac8` command - Runtime HTTP and MCP server tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" runtime_http`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-25
- [x] `ac9` command - MCP server tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" mcp_server`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26
- [x] `ac10` command - CLI MCP parser tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-cli mcp_http`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-27

## Phase 5: Final Integration Gate

Status: completed
Dependencies: phase4

Objective: Prove Tier 0 hardening is narrow, formatted, and free of obvious

Changes:
- Run the final security grep before any documentation edits and again as the final gate; it is the lock that keeps provider grant fallback removal from regressing after Phase 3.
- Update docs only if a new pure helper or runtime admission seam needs a durable boundary note.
- If provider grant-id fallback removal lands, add a release-note/operator advisory documenting `RUNX_PROVIDER_PERMISSION_GRANT_ID` as mandatory for provider-permission steps.
- Run formatting, focused tests, security grep, and whitespace checks.
- Record any broader workspace test failures as out-of-scope only with exact evidence.

Acceptance:
- [x] `ac11` command - Rust formatting is clean
  - Command: `cd crates && cargo fmt --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-32
- [x] `ac12` command - Security grep finds no fallback grant or unsafe tool/path regression
  - Command: `! rg -n "operator-provider-grant|policy\\.[A-Za-z0-9_]*granted_scopes|allow_private_network.*unwrap_or\\(true\\)|allow_explicit_manifest_path: true" crates/runx-runtime/src/effects/provider_permission.rs crates/runx-runtime/src/adapters/agent_tools.rs crates/runx-runtime/src/adapters/http.rs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-33
- [x] `ac13` command - Diff has no whitespace errors
  - Command: `git diff --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-34

## Follow-Up Specs

The user's Tier 1-4 list remains valid but should be executed as separate
contracts after Tier 0 passes:

- Tier 1: receipt-proof scope adherence and offline `runx verify`.
- Tier 2: host-drives/MCP first-class entrypoints and agent-loop demotion.
- Tier 3: payment spend-cap/SPT/refund authority.
- Tier 4: hosted cloud authz core audit and hardening.

## Rollback

- Revert the admission predicate and parser/runtime call-site changes together.
- Preserve existing signing-env stripping, MCP bearer auth, and private-network
  default-deny behavior; rollback must not reintroduce known unsafe defaults.
- If a parser rejection breaks legitimate fixtures, migrate the fixture to a
  catalog-style tool ref instead of adding compatibility aliases.
- Removing the implicit provider grant id is a deliberate fail-closed behavior
  change. Operators must provide `RUNX_PROVIDER_PERMISSION_GRANT_ID`; do not
  loosen the final security grep as a rollback shortcut.
- The implementation must include a release-note/operator advisory for that
  fail-closed change so production users see a clear denial reason rather than
  treating the first failed provider-permission run as a runtime bug.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Tier 0 capability admission spine is implemented as scoped. The new pure predicate `admit_agent_tool_ref` lives in `runx-core::policy::tool_ref` and is wired from both parser surfaces (skill governance + graph step `allowed_tools`) and the runtime managed-agent executor before local tool resolution, with `allow_explicit_manifest_path: false` keeping path-shaped refs out of model-selected resolution. Provider permission now requires `RUNX_PROVIDER_PERMISSION_GRANT_ID`, rejects self-attested `granted_scopes` from graph policy, and never falls back to a synthesized grant id. HTTP private-network access still demands both `source.http.allow_private_network` and the operator env grant; the guarded DNS resolver covers loopback, link-local, ULA, NAT64, 6to4, and metadata addresses. MCP HTTP keeps loopback default, mandatory bearer auth with constant-time compare, and `--http-allow-non-loopback` as the sole opt-in. Sandbox env stripping continues at `allowed_base_env`, defending the MCP stdio/HTTP transport child env regardless of caller. Security grep ac12 confirms no fallback grant string or unsafe-flag regressions. Workspace classifier flagged the touchpoints as "ambient drift" because the spec lists `policy/mod.rs` while the file is `policy.rs`; the changes themselves are task-scoped and reviewed accordingly. No blocking findings.

Attack log:
- `runx-core::policy::tool_ref::admit_agent_tool_ref`: Bypass with whitespace/Unicode/control-char tool refs (LTR mark, NUL, embedded space/newline, '$(...)', 'fs..read', 'fs.', '.fs.read', '.', '.json') -> clean (trim+ASCII-only is_catalog_ref_byte plus empty-segment and manifest-extension checks reject all crafted variants; only catalog-shape ns.tool refs admitted.)
- `crates/runx-runtime/src/adapters/agent_tools.rs`: Allowlist a path-like string and have the model invoke it (e.g. allowed_tools: ['/tmp/manifest.json']) to coerce explicit manifest resolution -> clean (Executor calls admit_agent_tool_ref BEFORE the allowlist membership check; even if a path slipped through earlier validation, the runtime rejects model-selected refs with separators or manifest extensions. resolve_and_invoke_local_tool also receives allow_explicit_manifest_path: false.)
- `crates/runx-parser/src/graph/step.rs + skill/governance.rs`: Embed path/manifest refs in graph step allowed_tools and skill runx.allowed_tools to evade parser admission -> clean (Both validate_allowed_tools call admit_agent_tool_ref per entry and produce a clear validation error with the correct field path (steps.<i>.allowed_tools / runx.allowed_tools).)
- `crates/runx-runtime/src/effects/provider_permission.rs`: Self-grant provider scopes via graph policy (granted_scopes inline), camelCase key 'grantedScopes', or omit RUNX_PROVIDER_PERMISSION_GRANT_ID and rely on legacy fallback -> clean (granted_scopes presence in policy is denied as self-attested; unknown camelCase variants are silently ignored (fail-closed because missing scopes still denies); provider_grant_id requires non-empty env and the old 'operator-provider-grant' default is gone. Denial message identifies the env var to set.)
- `crates/runx-runtime/src/adapters/http.rs + runtime_http.rs`: Manifest self-authorizes private-network HTTP, or attacker tries metadata/link-local/loopback/NAT64/6to4 SSRF -> clean (allow_private_network unwrap_or(false) and operator_allows_private_network env gate keep the transport choice operator-controlled. is_private_network_ip covers metadata 169.254.169.254, loopback, link-local, ULA fc00::/7, fe80::/10, 2001:db8::/32, NAT64 64:ff9b::/96, 6to4 2002::/16, multicast, and unspecified.)
- `crates/runx-runtime/src/adapters/mcp/http_server.rs + cli/src/mcp.rs`: Bind non-loopback without opt-in, smuggle '--http-allow-non-loopback=false', or bypass bearer auth via timing oracle -> clean (checked_listen_addr rejects any non-loopback addrs.iter() without allow_non_loopback; CLI rejects inline value on the flag; bearer compare uses constant_time_eq with length-mixed XOR; empty bearer token rejected at validate_http_security.)
- `crates/runx-runtime/src/sandbox/env.rs + sandbox.rs`: Smuggle RUNX_RECEIPT_SIGN_* signing seed/kid/issuer through skill base env, MCP stdio adapter child, or skill workspace env into a spawned child -> clean (allowed_base_env retains DEFAULT_ENV_ALLOWLIST + non-reserved env_allowlist entries, then runs a final retain(!is_reserved_runx_sandbox_env_name). is_reserved_runx_sandbox_env_name covers RUNX_RECEIPT_SIGN_* and RUNX_*<secret-token-password-key-credential-seed> substrings. prepare_mcp_process_sandbox and prepare_process_sandbox both flow through child_base_env. New regression tests in sandbox.rs lock both transport child envs.)
- `Parser fixtures + integration.rs`: Existing fixtures with legacy path-like allowed_tools entries break parser -> clean (rg confirms all repo-shipped allowed_tools use catalog-style refs (fs.read, git.*, cli.capture_help, shell.exec, web.search). New parser_graph_allowed_tools is registered in integration.rs.)
- `docs/security-authority-proof.md`: Operators upgrade without notice and existing provider-permission steps fail at runtime -> clean (New 'Provider-Permission Grants' section documents the mandatory env vars and the intentional fail-closed change, satisfying the rollback advisory requirement.)

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: error
Started: 2026-06-10T01:15:48Z
Ended: 2026-06-10T01:15:48Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: invalid provider dossier evidence: observation "path": invalid anchor prefix "/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#scope L182" (expected "Anchor: spec_gap:<field>", "Anchor: code:<path>:<line>", or "Anchor: archive:<task-id>"); observation "path": invalid anchor prefix "/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/effects/provider_permission.rs:179" (expected "Anchor: spec_gap:<field>", "Anchor: code:<path>:<line>", or "Anchor: archive:<task-id>"); observation "command": invalid anchor prefix "/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#phases ac10" (expected "Anchor: spec_gap:<field>", "Anchor: code:<path>:<line>", or "Anchor: archive:<task-id>"); observation "command": invalid anchor prefix "/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#phases ac1" (expected "Anchor: spec_gap:<field>", "Anchor: code:<path>:<line>", or "Anchor: archive:<task-id>"); observation "scope": invalid anchor prefix "/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#scope L84" (expected "Anchor: spec_gap:<field>", "Anchor: code:<path>:<line>", or "Anchor: archive:<task-id>"); observation "timing": invalid anchor prefix "/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#phases phase5" (expected "Anchor: spec_gap:<field>", "Anchor: code:<path>:<line>", or "Anchor: archive:<task-id>"); observation "rollback": invalid anchor prefix "/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#rollback" (expected "Anchor: spec_gap:<field>", "Anchor: code:<path>:<line>", or "Anchor: archive:<task-id>"); observation "design": invalid anchor prefix "/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#objectives" (expected "Anchor: spec_gap:<field>", "Anchor: code:<path>:<line>", or "Anchor: archive:<task-id>")

Observations:
- path
  - Result: clean
  - Anchor: /Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/effects/provider_permission.rs:179
  - Note: Verified `operator-provider-grant` fallback exists at the cited line and is the correct ac10 target after Phase 3 removes the fallback. Other code anchors (http.rs:292 allow_private_network, agent_tools.rs:69 allow_explicit_manifest_path:false, mcp/http_server.rs:34 loopback default, sandbox.rs:48 reserved env predicate, launcher.rs:85 mcp_http_listen test) all check out.
  - Default: Update the touchpoint to `parser_rejections.rs` and `parser_fixtures.rs`, and have Phase 2 add new graph-step `allowed_tools` tests under a new `parser_graph_allowed_tools.rs`.
  - Status: open
- command
  - Result: advisory
  - Anchor: /Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#phases ac1
  - Note: Phase 1 acceptance is `cargo test -p runx-core policy::sandbox policy::types`. The new pure `agent-tool-ref` predicate naturally belongs in a new module (e.g. `policy::agent_tools` or `policy::tool_ref`) that neither filter exercises. Without adding it to one of these existing test modules, the ac1 command can pass without proving the new helper has tests.
  - Default: Either place the predicate inside `policy::sandbox` / `policy::types` test modules, or extend ac1's filters with the new module name.
  - Status: open
- scope
  - Result: advisory
  - Anchor: /Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#scope L84
  - Note: Phase 3 promises regression tests covering signing-env stripping for runtime options, skill workspace env, CLI-tool child env, AND MCP process env. The touchpoint list names `runner.rs`, `skill_run.rs`, `sandbox/env.rs`, and `adapters/mcp/http_server.rs`, but the MCP stdio process env path (other files under `adapters/mcp/`) is not surfaced. Confirm whether MCP-process env stripping shares the stdio adapter and add the touchpoint if so, so the surface area matches the test promise.
  - Status: open
- timing
  - Result: advisory
  - Anchor: /Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#phases phase5
  - Note: Phase 5's ac10 grep is the only thing forcing the fallback grant-id removal to stick — but as written it will block on legitimate keep-the-rejection lines (see command observation above). It should run AFTER phase 3 lands and BEFORE phase 5 is recorded green, with a tightened pattern. Also worth running ac10 once at the start of Phase 5 to catch any unintentional reintroduction of `allow_explicit_manifest_path: true` in catalog.rs:118 leaking into agent_tools.rs/http.rs — current pattern correctly excludes catalog.rs from the path list, which is intentional.
  - Default: Run the corrected grep at the head of Phase 5 before doc updates, then again as the gate.
- rollback
  - Result: advisory
  - Anchor: /Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#rollback
  - Note: Rollback covers code reversion but not the user-visible breakage from removing the `operator-provider-grant` fallback. Operators relying on the implicit grant id will see provider-permission steps fail closed on first run after upgrade. Add a one-line note that operators must set the provider-permission grant id env var, and that downgrading the runtime alone restores the implicit fallback. Also acknowledge that ac10 must not be loosened as a rollback shortcut.
  - Status: open
- design
  - Result: clean
  - Anchor: /Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/runx-capability-admission-spine-v1.md#objectives
  - Note: Architecturally this is the right move, not a bandaid: a small pure predicate in runx-core::policy plus thin enforcement at impure seams, with Tier 1-4 explicitly deferred. Avoids the CapabilityManager anti-pattern, preserves the Rust/TS boundary in CLAUDE.md, and treats receipt-signing-env stripping, provider grants, private-network gating, and MCP exposure as one coherent admission spine rather than scattered patches. No future-bloat surface added.

### round-2

Status: needs_revision
Started: 2026-06-10T01:22:15Z
Ended: 2026-06-10T01:22:15Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Capability-admission-spine v1 is structurally the right Tier 0 architecture — pure predicates in runx-core::policy plus thin enforcement at the verified runtime seams, with Tier 1–4 deferred. Code anchors check out (provider_permission.rs:179 fallback grant-id, agent_tools.rs:69 `allow_explicit_manifest_path: false`, http.rs:292 private-network unwrap_or(false), sandbox.rs:48 reserved-env predicate). One blocking gap: top-level Acceptance command at spec line 233 (`... mcp`) diverges from Phase 4 ac9 at line 375 (`... mcp_server`) and silently drops ac4 — summary and per-phase gate must mirror each other. Advisory gaps remain on MCP touchpoint precision, vacuous TS-wrapper scope, Phase 5 ac12 timing, and a missing rollback operator-advisory note for the fail-closed change to `RUNX_PROVIDER_PERMISSION_GRANT_ID`.

Observations:
- path
  - Result: clean
  - Anchor: code:crates/runx-runtime/src/effects/provider_permission.rs:179
  - Note: Verified the cited fail-open paths exist on main: provider_permission.rs:179 `.unwrap_or("operator-provider-grant")` is the fallback grant-id Phase 3 removes; agent_tools.rs:69 `allow_explicit_manifest_path: false` confirms the model-selected tool boundary is already in place; http.rs:292 `allow_private_network.unwrap_or(false)` confirms the request flag plus operator env grant pattern. The touchpoint `parser_graph_allowed_tools.rs` is intentionally a planned new file for Phase 2 — flagged but acceptable.
- command
  - Result: blocks
  - Anchor: spec_gap:acceptance
  - Note: Top-level Acceptance section line 233 lists `cargo test -p runx-runtime --features "..." mcp` while Phase 4 ac9 at line 375 lists the narrower filter `mcp_server`. Both are runnable but they aren't the same gate; scafld validates phase-level acceptance, so the top-level summary contradicts it. Reconcile to one filter so review can't pass on a different command than build executed. Also: top-level acceptance omits ac4 (`-p runx-parser graph`) that Phase 2 requires, masking which gates summary truly covers.
  - Default: Make the top-level Acceptance section a verbatim mirror of the phase-level ac# commands so summary and gate agree.
  - Status: open
- scope
  - Result: advisory
  - Anchor: spec_gap:scope
  - Note: Phase 3 promises signing-env stripping regression tests for `MCP-served skill/tool child env`. Touchpoints list `mcp/adapter.rs`, `mcp/server_skill.rs`, and `mcp/http_server.rs` but the spec never says which of these owns the stdio child-env path actually being asserted. Either pin the touchpoint that owns child-env construction or drop the others to avoid drift between scope and test surface. Separately: scope claim about TypeScript CLI wrapper parity for MCP HTTP flags is vacuous — `rg http-allow-non-loopback` found zero hits under `packages/`, so no TS wrapper exposes this today.
  - Default: Drop unused TS-wrapper scope line, and narrow MCP touchpoint to the file actually carrying child-env construction.
- timing
  - Result: advisory
  - Anchor: spec_gap:phases
  - Note: Phase 5 ac12 grep is the only gate that locks the fallback grant-id removal in place; it depends on Phase 3 having landed. The current pattern `policy\.[A-Za-z0-9_]*granted_scopes` correctly skips the legitimate `policy.contains_key("granted_scopes")` rejection line (verified at provider_permission.rs:72) because `(` breaks the identifier class. Recommend running ac12 once at the *start* of Phase 5 (before any doc edits) as well as as the final gate, so a doc-edit regression that re-introduces the literal cannot ride out the window between Phase 3 closing and Phase 5 final gate.
  - Default: Add a Phase 5 pre-step that runs ac12 before doc updates and again as the final gate.
- rollback
  - Result: advisory
  - Anchor: spec_gap:rollback
  - Note: Rollback correctly states operators must set `RUNX_PROVIDER_PERMISSION_GRANT_ID` and that ac12 must not be loosened. Missing: the user-visible breakage path. Operators with provider_permission steps in production will see fail-closed denials on first run after upgrade — there is no migration warning, no startup-time advisory, and no CHANGELOG/release-note line called out in the rollback plan. Either add a one-line operator-advisory entry the implementer must produce, or commit to a release-note item so support can route the inevitable bug reports without loosening the security grep.
  - Default: Add a rollback bullet requiring a release-note line documenting `RUNX_PROVIDER_PERMISSION_GRANT_ID` as mandatory and pointing operators to the env var.
- design
  - Result: clean
  - Anchor: code:crates/runx-core/src/policy/sandbox.rs:48
  - Note: Architecturally this is the right move, not a bandaid. Pure admission predicates live next to `is_reserved_runx_sandbox_env_name` in `runx-core::policy`; impure enforcement stays in `runx-runtime` at the existing seams (agent_tools.rs:50, provider_permission.rs:175, http.rs:292, mcp/http_server.rs:34); CLI keeps presentation/flag-parsing only. No new `CapabilityManager` policy engine, no Rust/TS execution duplication, and Tier 1–4 are explicitly deferred so the spec stays executable and reviewable. Receipt-signing-env stripping, provider grants, private-network gating, and MCP exposure are treated as one coherent admission spine rather than scattered patches.

### round-3

Status: error
Started: 2026-06-10T01:25:40Z
Ended: 2026-06-10T01:25:40Z
Summary: provider error: provider failed: provider produced no submission; Claude must call submit_harden exactly once and final text is ignored: ... tes.rs\ncrates/runx-runtime/src/adapters/mcp/server.rs\ncrates/runx-runtime/src/adapters/mcp/types.rs\ncrates/runx-runtime/src/adapters/mcp/server_skill.rs\ncrates/runx-runtime/src/adapters/mcp/http_server.rs\ncrates/runx-runtime/src/adapters/mcp/transport.rs"}]},"parent_tool_use_id":null,"session_id":"39ac0d22-5310-4bf2-ad6d-c1da73ba1e92","uuid":"e9665221-cd8c-48bd-a7e2-f5a0e126668c","timestamp":"2026-06-10T01:26:14.006Z","tool_use_result":{"filenames":["crates/runx-runtime/src/adapters/mcp/framing.rs","crates/runx-runtime/src/adapters/mcp/rmcp_content_length.rs","crates/runx-runtime/src/adapters/mcp/adapter.rs","crates/runx-runtime/src/adapters/mcp/sandbox_metadata.rs","crates/runx-runtime/src/adapters/mcp/templates.rs","crates/runx-runtime/src/adapters/mcp/server.rs","crates/runx-runtime/src/adapters/mcp/types.rs","crates/runx-runtime/src/adapters/mcp/server_skill.rs","crates/runx-runtime/src/adapters/mcp/http_server.rs","crates/runx-runtime/src/adapters/mcp/transport.rs"],"durationMs":10351,"numFiles":10,"truncated":false}}
{"type":"system","subtype":"status","status":"requesting","uuid":"cde519f6-11c6-4630-9c94-3b41027698e0","session_id":"39ac0d22-5310-4bf2-ad6d-c1da73ba1e92"} (diagnostic: /Users/kam/dev/runx/runx/oss/.scafld/runs/runx-capability-admission-spine-v1/diagnostics/command-1781054795364227000.txt)

Observations:
- none


## Planning Log

- 2026-06-10: Completed `runx-rust-registry-skill-resolver` before opening this
  spec; Claude review passed and scafld archived it.
- 2026-06-10: Verified current main already contains partial hardening for
  receipt-signing env stripping, managed-agent allowed-tools checks, provider
  self-attested scope rejection, HTTP private-network operator gating, and MCP
  HTTP loopback/bearer auth.
- 2026-06-10: Folded in two read-only subagent investigations covering
  runtime/env/provider/tool admission and MCP/runtime HTTP guardrails.
- 2026-06-10: Chose a narrow Tier 0 scope. Tier 1-4 are intentionally recorded
  as follow-up specs so this contract remains executable and reviewable.
