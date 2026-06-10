# Runx Capability Admission Handoff

Date: 2026-06-10
Workspace: `/Users/kam/dev/runx/runx/oss`
Spec: `runx-capability-admission-spine-v1`
Spec status: completed (archived at
`.scafld/specs/archive/2026-06/runx-capability-admission-spine-v1.md`)

## Current State

RESOLVED 2026-06-10: all five phases were recorded through the normal
`scafld build` loop (13/13 acceptance items passed), a single official
`scafld review --provider claude` ran with verdict pass and no blocking
findings, and `scafld complete` succeeded with completion authority
`valid (review)`. The Tier 0 code dirt listed below remains uncommitted;
commit is still pending operator request. Remaining sections are kept for
the Tier 1-4 follow-up context.

The Tier 0 capability-admission implementation is coded and targeted
validation has passed. The scafld spec is active, but scafld phase evidence has
not been recorded through the normal `scafld build` loop. The user explicitly
asked to stop overusing harden/review, so do not restart another harden loop
unless asked.

Current dirt owned by this work:

- `.scafld/specs/active/runx-capability-admission-spine-v1.md`
- `.plans/active/runx-capability-admission-handoff.md`
- `crates/runx-core/src/policy.rs`
- `crates/runx-core/src/policy/tool_ref.rs`
- `crates/runx-parser/src/graph/step.rs`
- `crates/runx-parser/src/skill/governance.rs`
- `crates/runx-parser/tests/integration.rs`
- `crates/runx-parser/tests/parser_graph_allowed_tools.rs`
- `crates/runx-runtime/src/adapters/agent_tools.rs`
- `crates/runx-runtime/src/effects/provider_permission.rs`
- `crates/runx-runtime/src/sandbox.rs`
- `docs/security-authority-proof.md`

## What Was Built

### Shared Agent Tool-Ref Admission

Added `runx-core::policy::admit_agent_tool_ref` in
`crates/runx-core/src/policy/tool_ref.rs`.

It admits catalog-style refs like:

- `fs.read`
- `git.current_branch`
- `shell.exec`
- `cli.capture_help`

It rejects:

- empty refs
- absolute paths
- path separators
- `..`
- manifest/data-file-shaped refs such as `manifest.json` or `fs.json`
- shell-ish or whitespace-containing refs
- un-namespaced refs like `read`

This keeps the predicate pure and reusable. It is deliberately not a broad
capability manager.

### Parser Boundary

Parser validation now routes all agent `allowed_tools` through the shared
predicate:

- skill `runx.allowed_tools`
- runner manifest `runx.allowed_tools`
- graph step `allowed_tools`

New parser tests live in
`crates/runx-parser/tests/parser_graph_allowed_tools.rs` and are registered
through the consolidated `tests/integration.rs` binary.

### Runtime Managed-Agent Boundary

`RuntimeToolExecutor` now rejects an inadmissible model-selected tool ref before
the allowed-tools membership check and before local tool resolution.

This matters because `allowed_tools` is now a boundary, not a catalog hint. Even
if a path-like value somehow enters an allowlist, a model-selected
`/tmp/manifest.json` cannot route into explicit manifest resolution.

### Provider Permission Fail-Closed Grant ID

`provider_permission` no longer invents `operator-provider-grant`.

Provider-permission steps now require operator-carried runtime evidence:

- `RUNX_PROVIDER_PERMISSION_GRANT_ID`
- `RUNX_PROVIDER_PERMISSION_GRANTED_SCOPES`

The self-attested graph-policy `granted_scopes` denial remains intact.

An operator note was added to `docs/security-authority-proof.md`.

### Receipt-Signing Env Child-Process Regression Tests

Added sandbox tests proving receipt-signing env vars are not present in child
process env plans:

- normal process sandbox planning
- MCP subprocess sandbox planning

The MCP HTTP server process itself may still hold signer authority because it is
the operator-started receipt sealer. The child env is the security boundary.

## Validation Already Run

All commands used isolated Cargo target dir where relevant:
`CARGO_TARGET_DIR=target/runx-capability-admission-spine`.

Passed:

- `cd crates && cargo fmt --check`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-core policy::`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser parser_sandbox`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser allowed_tools`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-parser graph`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime sandbox`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "agent catalog" agent_tools`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" provider_permission`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" http`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" runtime_http`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" mcp_server`
- `cd crates && CARGO_TARGET_DIR=target/runx-capability-admission-spine cargo test -p runx-cli mcp_http`
- `! rg -n "operator-provider-grant|policy\\.[A-Za-z0-9_]*granted_scopes|allow_private_network.*unwrap_or\\(true\\)|allow_explicit_manifest_path: true" crates/runx-runtime/src/effects/provider_permission.rs crates/runx-runtime/src/adapters/agent_tools.rs crates/runx-runtime/src/adapters/http.rs`
- `git diff --check`
- `scafld validate runx-capability-admission-spine-v1`

Not run:

- full workspace `cargo test`
- full `pnpm test`
- adversarial `scafld review`

Those were intentionally skipped to avoid stalling and because the user asked
to stop overusing harden/review during this pass.

## Remaining Work To Finish This Spec

### 1. Record Or Reconcile scafld Build Evidence

The code is ahead of the scafld phase ledger. `scafld handoff` still reports all
acceptance items as pending because the commands were run manually, not through
scafld evidence recording.

Options:

- Best: run the scafld build/exec path enough to record evidence against the
  active spec, using the validation commands above.
- Pragmatic: add a concise manual evidence note to the spec/session if scafld
  supports that path.
- Avoid: rerunning Claude harden unless explicitly requested.

### 2. Decide Whether To Review/Complete

The normal lifecycle wants review before complete. The user explicitly asked to
stop overusing review/harden. Choose one:

- run a narrow command/codex review only if requested
- complete with a human-reviewed/manual reason if the operator accepts the
  targeted validation
- leave active until the next agent has time to run the official gate

Do not silently mark complete without a review or explicit human-reviewed path.

### 3. Commit If Asked

No commit has been made in this pass. If asked to commit:

- include only this owned dirt
- conventional commit example: `fix(runtime): harden capability admission`
- do not include unrelated workspace dirt

### 4. Optional Near-Term Polish

The implementation is narrow and acceptable, but the following would make it
cleaner:

- Consider whether `ToolRefAdmission` should be named `AgentToolRefAdmission`
  for clarity. Current name is shorter and acceptable.
- Consider adding a dedicated parser fixture for invalid `allowed_tools` if the
  parser fixture matrix expects every validation rule to have JSON fixture
  coverage. Current integration tests are enough for this change.
- If release notes exist outside docs, add the provider grant-id fail-closed
  note there too. I found docs but no obvious top-level changelog.

## Work Discovered Outside This Spec

These should be separate specs. Do not fold them into Tier 0.

### Tier 1: Receipts Prove Scope Adherence

Current receipts are signed and useful, but the next product jump is making the
receipt prove admitted authority, not just sealed output.

Work:

- Record the grant/operator authority behind each sealed privileged effect.
- Add offline `runx verify` that checks:
  - signature validity
  - linked-tree integrity
  - every sealed privileged effect was within a recorded grant
- Fix execution graph retry/child sealing defects before relying on the tree:
  a failed retry child must not be sealed under a succeeded step.

Why it matters: this is the gap between "signed log" and "governance proof."

### Tier 2: Host-Driven And MCP-Driven Execution As First-Class Entrypoints

The long-term story says the agent loop is swappable. The code should make
that true.

Work:

- Make host-driven execution and authenticated MCP the primary governed
  execution surfaces.
- Keep the agent loop and provider-specific agent adapters clearly marked as
  sample/dev/borrowed-loop adapters, not critical enforcement substrate.
- Ensure Tier 0/Tier 1 admission and receipt proof wrap those entrypoints
  identically.

Why it matters: fewer high-risk enforcement surfaces, cleaner orchestrator
story, better operator trust.

### Tier 3: Payment Authority Runtime Enforcement

Payment authority remains differentiated but not fully hardened.

Work:

- Enforce per-period spend cap at runtime, not only per-call/per-run.
- Fix SPT integrity check if it still compares issuance to itself.
- Confirm cloud-side refund is bounded by captured amount and settlement
  idempotency.
- Add receipts/proof evidence for payment authority gates.

Why it matters: spend authority must be a primitive, not an adapter convention.

### Tier 4: Cloud Authz Core Review

Hosted governance relies on cloud authz and billing code outside this local OSS
runtime pass.

Work:

- Audit grant expiry enforcement.
- Audit secret separation: AES master, ticket HMAC, HS256/JWT signing must not
  share one root secret.
- Audit revocation, BYO verification, OAuth broker, and billing authz.
- Back findings with code-level tests before changing policy.

Why it matters: local receipts only matter if hosted grant issuance and
revocation are trustworthy.

### Registry Resolver Follow-Up

From the immediately prior registry work:

- Clarify multi-version install layout: whether filesystem cache stores multiple
  versions side-by-side or only latest.
- Ensure resolver errors explain trusted/untrusted registry status.
- Add operator UX for selecting registry trust policy without weakening default
  verification.
- Keep third-party registry resolution possible, but visibly untrusted unless
  the operator grants trust.

### CLI Operator UX Follow-Up

The CLI is improving, but the operator path can still be sharper:

- `runx skill` should explain exactly what was resolved: local path, registry
  package, version, digest, trust status.
- Export commands should print exact created skill paths and permission policy
  changes.
- Errors should name the fix: missing registry trust key, invalid tool ref,
  missing provider grant id, non-loopback MCP HTTP denied, etc.
- Add a concise `runx doctor security` or `runx doctor authority` view for
  operator-facing runtime readiness.

### Nitrosend/Operational Intelligence Integration Follow-Up

Not part of this spec, but affected by the new provider-permission behavior:

- Any Nitrosend wrapper or Aster runner that executes provider-permission graph
  steps must pass `RUNX_PROVIDER_PERMISSION_GRANT_ID`.
- Slack/GitHub actions should show a clean operator-facing denial if the grant
  id is missing, not a generic runtime failure.
- If issue-to-PR paths depend on provider scopes, their receipts should include
  the explicit grant id once Tier 1 lands.

## Suggested Next Order

1. Finish/record this active scafld spec without another harden loop.
2. Commit the Tier 0 changes.
3. Create a small Tier 1 spec for receipt scope-adherence proof.
4. Create a separate payment authority runtime-cap spec.
5. Create a cloud authz audit spec only after the local Tier 1 receipt proof
   shape is settled.

## Fresh Agent Prompt

You are in `/Users/kam/dev/runx/runx/oss`. Continue from the active scafld spec
`runx-capability-admission-spine-v1`. The code for Tier 0 is already
implemented and targeted validation has passed. Do not rerun harden unless the
operator asks. First reconcile scafld evidence/status with the manual commands
listed in `.plans/active/runx-capability-admission-handoff.md`, then decide
with the operator whether to complete via the normal review gate or an explicit
human-reviewed path. Keep all Tier 1-4 work as separate specs.
