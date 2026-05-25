---
spec_version: '2.0'
task_id: runx-security-hardening-v1
created: '2026-05-22T00:55:00Z'
updated: '2026-05-22T02:05:00Z'
status: active
harden_status: in_progress
size: large
risk_level: high
---

# runx security hardening v1

## Current State

Status: active
Current phase: mechanism implementation validated; review pending
Next: review the R1/R2/R3 mechanism changes with platform-specific sandbox
coverage where available.
Reason: C1/C2/C4/C5/C6/C7/R7 are implemented and validated. R1 now has
backend-gated local OS sandbox enforcement through bubblewrap on Linux and
sandbox-exec on macOS, fails closed when enforcement is required but unavailable,
and records declared-policy-only/non-enforcing metadata otherwise. R2 production
receipt signing is wired through runtime receipt creation paths. R3 payment rail
proof now requires a configured runtime supervisor and rejects skill claims
alone. Focused R1/R2/R3 validation and the heavy graph gate are green.
Blockers: formal review remains; platform-specific sandbox backends should stay
covered where the host provides bubblewrap or sandbox-exec.
Allowed follow-up command: `scafld exec runx-security-hardening-v1`
Latest runner update: 2026-05-22T12:05:00+10:00
Review gate: not_started

## Summary

Harden runx's security posture before any untrusted-skill execution or
production-payment story. The architecture is sound (capability/authority model,
receipt-centric design, pure policy kernel), but several controls are currently
*declarations or placeholders* rather than *enforced mechanisms*: the sandbox
does not confine, the receipt seal does not cryptographically bind, the payment
proof is skill-asserted, and a few core admission edges fail open. This spec
ranks the findings by severity and sequences them surgical-first.

This is a defensive-security spec. No finding here authorizes adding
provider/integration code to the kernel; the trust boundaries from
`external-adapter-plugin-protocol-v1`, `skill-author-runtime-contract-v1`, and
`canonical-json-fingerprint-contract-v1` still hold and are referenced where a
finding overlaps them.

## Context

The exposures cluster on three trust-boundary assumptions that do not yet hold:
the sandbox confines (it does not), the seal attests (it is a placeholder), and
the payment proof proves (it is asserted). Plus a set of narrower core-admission
and runtime edges. Findings are grounded in file:line.

Severity legend: **Critical** = breaks a core security claim; **High** =
exploitable bypass or secret exposure; **Medium** = hardening / defense-in-depth.

## Core findings (`runx-core`)

- **C1 [Critical, fail-open] — DONE (core OSS).** `find_matching_grant`
  (`policy/connected_auth.rs`) admitted any grant whose `status != Some(Revoked)`,
  so a grant with `status: None` (the field is `skip_serializing_if`, so omitted
  JSON deserializes to `None`) was admitted. Fixed to require `Some(Active)`
  (fail-closed on missing status). Added `not_before` / `expires_at` to
  `LocalAdmissionGrant`, explicit `connected_auth_checked_at` inputs for the
  pure policy evaluators, and fail-closed lifetime checks: active grants now deny
  if they are unbounded, malformed, expired, not yet valid, or evaluated without
  a timestamp. The current MIT OSS branch has no active `HttpConnectGrant`
  runtime type after the connect brokerage removal; private/hosted connect
  schemas still need parity if reintroduced.
- **C2 [Medium-High, audit integrity] — DONE (subset-proof gate).** The kernel
  recomputes `is_payment_authority_subset(child, parent)` (sound, correct
  direction), and the runtime now passes the typed `AuthoritySubsetProof` into
  `StepAuthorityAdmission`. Payment attenuation rejects missing or mismatched
  subset proofs by parent ref, compared child/parent term ids, relation, result,
  algorithm, and checked timestamp. This closes the caller-supplied boolean
  proof-presence gap. The remaining R3 rail-settlement proof issue stays open.
- **C3 [Medium, coverage] — deep attenuation is payment-only.**
  `admit_step_authority` runs the bounds/capability/condition subset algebra only
  for `resource_family == Payment && spends`; all other families return
  `verb: None` with no attenuation and rely solely on `scope_allows` prefix
  matching. → confirm intentional or extend attenuation to other high-value
  resource families (deploy, repo-write).
- **C4 [Medium] — DONE (aggregate spend capped).** `minor_unit_caps_subset`
  (`policy/payment_authority.rs`) now requires at least one aggregate cap
  (`max_per_run_minor` or `max_per_period_minor`) on both parent and child spend
  authority before per-call caps can pass subset comparison. The policy fixture
  `payment-authority-denies-unbounded-aggregate-spend` covers the fail-closed
  path.
- **C5 [Medium] — DONE (untrusted wildcard denied; prefix narrowed).**
  `scope_allows` now gates universal `*` behind an explicit trusted-callsite
  flag. Graph scope propagation can still use first-party `*`, but connected
  auth / credential grants fail closed on universal wildcard input. Prefix
  wildcards are single-segment: `repo:*` admits `repo:read` and denies
  `repo:admin:keys`. Added kernel parity fixtures and TS/Rust policy coverage for
  both edges.
- **C6 [Medium, design] — DONE (success requires admission witness).** The pure
  single-step and sequential graph state machines now require a
  `StepAdmissionWitness` on success transitions. Sequential success also checks
  that the witness step id and receipt id match the step being sealed, so a
  runner cannot transition a step to succeeded through the kernel without an
  admission/receipt witness.
- **C7 [Medium] — DONE (input limits).** `kernel_eval.rs` now fails closed before
  dispatch on oversized kernel-eval documents and structurally excessive JSON:
  max document bytes, JSON depth, node count, array length, object field count,
  object key bytes, and string bytes. Added fail-closed tests for oversized,
  deeply nested, and wide documents. Fuzzing remains a follow-up hardening item.

## Runtime findings (`runx-runtime`)

- **R1 [Critical] — DONE (backend-gated sandbox enforcement, harden pending).**
  `sandbox.rs` resolves a local enforcement runtime, wraps process execution
  with bubblewrap on Linux or sandbox-exec on macOS when available, fails closed
  when `require_enforcement` or `RUNX_SANDBOX_REQUIRE_ENFORCEMENT` is set and no
  backend can enforce, and emits runtime/filesystem/network metadata such as
  `bubblewrap-mount-namespace`, `sandbox-exec-seatbelt`, or
  `not-enforced-local`. `cli_tool_contract.rs` includes a backend-gated readonly
  write-denial regression. Full harden/review and platform validation remain
  pending.
- **R2 [Critical] — DONE (production signing path wired, harden pending).**
  Runtime receipt creation now resolves `RuntimeReceiptSignatureConfig` from
  `RUNX_RECEIPT_SIGN_*` env, uses Ed25519 production signing when configured,
  rejects incomplete production signing env, verifies production signatures with
  configured public keys, and routes step/graph/single-skill/MCP receipt writes
  through the active policy. Local development pseudo-signing remains explicit.
- **R3 [Critical] — DONE (runtime supervisor boundary, harden pending).**
  Payment spend success now requires `RuntimePaymentSupervisor` settlement
  evidence before a success receipt can stand. The default supervisor rejects,
  so a skill-produced `Verification`+`PaymentRail` reference is denied unless a
  configured supervisor returns matching settlement evidence bound to admitted
  rail, counterparty, amount, currency, idempotency key, spend capability, act,
  receipt ref, and receipt digest. Focused payment tests cover the no-supervisor
  and proofless-rail failures.
- **R4 [High] — secrets via env + post-hoc redaction.** `secret_env` injected
  into child env (`adapters/cli_tool.rs`); leaks via `/proc/<pid>/environ`,
  grandchildren, dumps. `redact_text` is substring replacement (encoding-bypass),
  and with R1 the skill can just exfiltrate over the network. → scoped/short-lived
  tokens, broker delivery; don't treat redaction as containment.
- **R5 [High] — skill stdout trusted as structured output.** stdout-as-JSON →
  `outputs` feeds receipts and authority fields; attacker-controlled. → separate
  skill-asserted output from supervisor-attested facts.
- **R6 [High] — canonical-JSON byte-identity unpinned across runtimes.** 4
  independent canonicalizers stamp `runx.stable-json.v1`; `canonical_json_number`
  = `JsonNumber::to_string()` (float/precision divergence). Digest confusion.
  Owned by `canonical-json-fingerprint-contract-v1` — cross-reference, do not
  duplicate.
- **R7 [High, payments] — DONE (file lock + reload before mutation).**
  `FileBackedPaymentStateStore` now takes a sidecar lock for state mutations,
  reloads the current persisted document under the lock before applying the
  mutation, and writes through the locked state. `payment_state` regression tests
  cover stale stores refusing to overwrite already-recorded idempotency state.
- **R8 [High, supply chain] — self-asserted install digests.**
  `registry/install.rs::validate_candidate_digest` hashes the candidate's own
  markdown against a digest the candidate supplies → no trust anchor (with R2, no
  root of trust beyond the `TrustTier` label). → verify against a publisher-signed
  manifest.
- **R9 [Medium] — input→env name collisions.** `sandbox.rs::input_env_name` maps
  non-alphanumerics to `_` + uppercases, so `foo-bar`/`foo.bar`/`foo_bar` collide
  to `RUNX_INPUT_FOO_BAR`. → reject colliding keys or pass inputs only via JSON.
- **R10 [Medium] — SSRF in A2A + external-HTTP adapters.** `agent_card_url` and
  external-adapter `endpoint` are influenceable, no egress allowlist / metadata-IP
  guard. → block link-local/RFC1918 unless declared; egress allowlist.
- **R11 [Medium] — timeout kills child, not process group.**
  `adapters/cli_tool.rs::wait_for_exit` calls `child.kill()`; grandchildren
  orphan. With R1 (no rlimits) → fork-bomb / disk-fill. → kill process group /
  job object; apply rlimits.
- **R12 [Medium] — `created_at` caller-influenced.** Receipt timestamps come from
  `RuntimeOptions`/env with a fixed fallback → forgeable, no freshness. → stamp
  at a trusted boundary.
- **R13 [Medium] — credential delivery channel for external adapters
  unspecified.** Owned by `external-adapter-plugin-protocol-v1`; flagged as the
  cross-cutting primitive (cli-tool, external adapter, outbox all need it).

## Phases

## Phase 1: Surgical fail-closed core fixes

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- none

Acceptance:
- none

## Phase 2: Enforcement mechanisms

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- none

Acceptance:
- none

## Phase 3: Proof verification + type-enforced gate

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- none

Acceptance:
- none

## Phase 4: Cross-cutting hardening

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- none

Acceptance:
- none

## Risks

- Fail-closed changes (C1/C4/C5) can deny previously-admitted edges; gate each on
  fixture review so a security tightening does not silently break a legitimate
  flow.
- R1/R2 are platform- and key-management-heavy; scope per-OS and per-environment.
- Several items touch files under active parallel work (payment, target_runner,
  contract spine); sequence around those workstreams.

## Acceptance

- [x] `dod1` C1 fail-closed on missing grant status, tests green.
- [x] `dod2` grant expiry/not_before added and enforced in core OSS.
- [x] `dod3` spend verbs require at least one aggregate cap (C4).
- [x] `dod4` `*` scope not acceptable from untrusted grants (C5).
- [x] `dod5` receipts signed + verified by default in non-local modes (R2).
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-receipts && cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test receipt_signing --test skill_run --test harness_fixtures`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 receipt proof and runtime signing/skill/harness tests
    passed.
- [x] `dod6` sandbox profiles OS-enforced or documented as non-enforcing (R1).
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool,mcp --test cli_tool_contract --test mcp_adapter`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 CLI-tool contract tests passed, including the
    backend-gated readonly sandbox write-denial regression; MCP adapter tests
    also passed.
- [x] `dod7` payment rail settlement proofs are supervisor-verified, not
  skill-asserted (R3).
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-25 payment execution, state, ledger projection, and SPT
    rail tests passed.
- [x] `dod8` state-machine success transition requires an admission witness (C6).
- [x] `dod9` payment-state writes are atomic/locked (R7).

## Rollback

Phase 1 fixes are independent and individually revertible. Phases 2–4 are
additive enforcement; nothing here changes wire shapes except the grant
expiry/proof-verification additions, which are gated on the schema-validation
work.

## Build Evidence

- 2026-05-22T02:42:26+10:00: `cargo test -p runx-core` passed.
- 2026-05-22T02:42:26+10:00: `cargo clippy -p runx-core --all-targets -- -D warnings`
  passed.
- 2026-05-22T02:42:26+10:00:
  `cargo test -p runx-runtime --test connect_policy_integration` passed.
- 2026-05-22T11:00:00+10:00: `cargo test --manifest-path crates/Cargo.toml -p
  runx-core -- --nocapture` passed.
- 2026-05-22T11:00:00+10:00: `cargo clippy --manifest-path crates/Cargo.toml -p
  runx-core --all-targets -- -D warnings` passed.
- 2026-05-22T11:00:00+10:00: `cargo clippy --manifest-path crates/Cargo.toml -p
  runx-runtime --all-targets -- -D warnings` passed.
- 2026-05-22T11:00:00+10:00: `pnpm typecheck` passed.
- 2026-05-22T11:00:00+10:00: `pnpm exec vitest run --config vitest.config.ts
  packages/runtime-local/src/runner-local/kernel-bridge.test.ts
  --fileParallelism=false --maxWorkers=1` passed.
- 2026-05-22T12:05:00+10:00: `cargo build --manifest-path crates/Cargo.toml
  -p runx-cli --bin runx` passed.
- 2026-05-22T12:05:00+10:00:
  `RUNX_KERNEL_EVAL_BIN=$PWD/crates/target/debug/runx pnpm fixtures:kernel:generate`
  regenerated 67 kernel parity fixtures.
- 2026-05-22T12:05:00+10:00:
  `RUNX_KERNEL_EVAL_BIN=$PWD/crates/target/debug/runx pnpm fixtures:kernel:check`
  passed for 67 kernel parity fixtures.
- 2026-05-22T12:05:00+10:00: `pnpm exec vitest run --config
  vitest.config.ts packages/core/src/policy/index.test.ts
  packages/core/src/policy/scope-narrowing.test.ts --fileParallelism=false
  --maxWorkers=1` passed.
- 2026-05-22T12:05:00+10:00: `CARGO_TARGET_DIR=crates/target-codex-security
  cargo test --manifest-path crates/Cargo.toml -p runx-core policy --
  --nocapture` passed.
- 2026-05-22T12:05:00+10:00: `pnpm exec tsc -p tsconfig.typecheck.json
  --noEmit --pretty false` passed.
- 2026-05-22T11:05:00+10:00: `cargo test --manifest-path crates/Cargo.toml -p
  runx-runtime --test payment -- --nocapture` passed.
- 2026-05-22T11:08:00+10:00: `cargo test --manifest-path crates/Cargo.toml -p
  runx-core policy -- --nocapture` passed.
- 2026-05-22T11:08:00+10:00: `cargo test --manifest-path crates/Cargo.toml -p
  runx-runtime --test payment -- --nocapture` passed.
- 2026-05-22T11:38:00+10:00: `cargo test --manifest-path crates/Cargo.toml -p
  runx-core --test kernel_eval -- --nocapture` passed.
- 2026-05-22T11:38:00+10:00: `cargo test --manifest-path crates/Cargo.toml -p
  runx-core --lib kernel_eval -- --nocapture` passed.
- 2026-05-22T11:38:00+10:00: `cargo clippy --manifest-path crates/Cargo.toml -p
  runx-core --all-targets -- -D warnings` passed.

## Origin

User-directed pentest review on 2026-05-21/22: widen findings into the Rust core
(which must be S-tier) and capture all findings into a work spec. Core attenuation
algebra verified largely sound; exposures are at admission edges, enforcement
placeholders, and skill-asserted trust.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-22T00:31:32Z
Ended: 2026-05-22T00:33:10Z

Checks:
- path audit
  - Grounded in: code:crates/runx-core/src/policy/connected_auth.rs:38
  - Result: passed
  - Evidence: Phase 1 C1 is bound to the connected-auth grant matcher and
- command audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: The built C1 slice has concrete evidence commands recorded
- scope/migration audit
  - Grounded in: code:crates/runx-runtime/src/connect.rs:1
  - Result: passed
  - Evidence: The current OSS branch only exports connect redaction, so the C1
- acceptance timing audit
  - Grounded in: code:crates/runx-core/tests/policy_proptest.rs:370
  - Result: passed
  - Evidence: Deterministic policy/proptest fixtures now supply
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Phase 1 is independently revertible by backing out the
- design challenge
  - Grounded in: code:crates/runx-core/src/policy/connected_auth.rs:60
  - Result: passed
  - Evidence: Requiring explicit `Active`, `expires_at`, and a caller-supplied

Issues:
- none
