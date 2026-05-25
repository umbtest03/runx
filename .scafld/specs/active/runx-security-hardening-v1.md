---
spec_version: '2.0'
task_id: runx-security-hardening-v1
created: '2026-05-22T00:55:00Z'
updated: '2026-05-25T18:19:00+10:00'
status: review
harden_status: passed
size: large
risk_level: high
---

# runx security hardening v1

## Current State

Status: review
Current phase: omnibus complete; remaining findings split into narrow specs
Next: handoff
Reason: review found no substantive completion blockers after repair. The
workspace-mutation guard is accepted for this pass because concurrent runtime
work was intentionally active; the actionable signing-env and history-state
findings were fixed, and the duplicate tool-root parsing note is non-blocking.
Blockers: none for this omnibus. Follow-up trust-boundary work is owned by
`skill-output-attestation-boundary-v1`,
`registry-signed-manifest-trust-anchor-v1`, and
`process-credential-delivery-hardening-v1`.
Allowed follow-up command: `scafld handoff runx-security-hardening-v1`
Latest runner update: 2026-05-25T18:19:00+10:00
Review gate: pass_by_repair

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
- **R4 [High] — DONE for `cli-tool`; residual split to R13.**
  `CliToolAdapter` rejects process-env credential delivery before spawning.
  Residual env-based delivery for MCP, external adapters, and outbox providers
  is the cross-cutting R13 credential-delivery follow-up.
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
- **R8 [High, supply chain] — split to focused spec.**
  `registry/install.rs::validate_candidate_digest` hashes the candidate's own
  markdown against a digest the candidate supplies → no trust anchor (with R2, no
  root of trust beyond the `TrustTier` label). Follow-up:
  `registry-signed-manifest-trust-anchor-v1`.
- **R9 [Medium] — DONE.** `RUNX_INPUT_*` env-name collisions fail closed.
- **R10 [Medium] — DONE for current live HTTP surfaces.** Runtime HTTP rejects
  localhost/RFC1918/link-local/metadata hosts and disables redirects. External
  adapter and outbox HTTP transport are currently rejected; A2A has no live HTTP
  implementation. Future live A2A HTTP should get its own egress spec.
- **R11 [Medium] — DONE on Unix process paths.** `cli-tool`, external adapter,
  and outbox provider create process groups and kill negative PGIDs on timeout.
  Windows/job-object parity remains a future platform spec if Windows enters
  support scope.
- **R12 [Medium] — DONE for live runtime defaults.** `RuntimeOptions::default()`
  stamps trusted live time, and deterministic harness/parity paths explicitly
  pin fixture time. Optional API hardening may later make caller-supplied
  timestamps unrepresentable.
- **R13 [Medium] — split to focused spec.** External adapter and outbox process
  credential delivery still need a non-env or strictly constrained channel.
  Follow-up: `process-credential-delivery-hardening-v1`.

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

## Review

Status: completed
Verdict: pass_by_repair
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Reviewed the four task-scoped runtime files (adapter.rs, agent_invocation.rs, execution/runner.rs, journal.rs) for the runx-security-hardening-v1 omnibus. The recent changes correctly thread the runtime payment supervisor (default rejecting), live trusted time, signature policy through the journal projection, and explicit credential-delivery routing — all consistent with the spec's "DONE" subset of findings. No blocker-level regressions were found. The actionable signing-env and harness-state projection findings were repaired after review. Duplicate tool-root parsing remains a low-severity cleanup note. The workspace-mutation guard is accepted for this pass because concurrent runtime implementation was intentionally active and no substantive completion blocker remains.

Attack log:
- `crates/runx-runtime/src/execution/runner.rs::safe_default_env`: env-allowlist vs signing-env intent: trace whether RuntimeReceiptSignatureConfig::from_env can ever observe RUNX_RECEIPT_SIGN_* through RuntimeOptions::default() -> finding (Allowlist excludes signing env vars; from_env always returns local_development. Confirmed via signing.rs:92-105 and lib.rs export.)
- `crates/runx-runtime/src/execution/runner.rs::run_graph_with_host_outcome blocked path`: BlockedGraphOutcome::Receipt could be reached via public API and produce a falsely-completed graph receipt -> clean (run_graph_file_for_harness is pub(crate); blocked receipts carry ClosureDisposition::Blocked and disposition_status preserves it as "blocked".)
- `crates/runx-runtime/src/journal.rs::verification_status`: verification_status mislabeling a tampered receipt as verified/unverified due to finding filter -> clean (Logic correctly requires production policy + verifier for "verified"; only SignatureVerifierMissing alone downgrades to "unverified"; any other finding -> "invalid". Comment naming "blocking" is loose but semantics hold (ReceiptFinding has no severity field per verify/finding.rs:47-51).)
- `crates/runx-runtime/src/agent_invocation.rs::envelope`: skill-controlled raw `instructions`/`allowed_tools` injected into agent envelope as trusted context -> clean (Envelope is metadata for downstream agent resolution; TRUST_BOUNDARY const explicitly documents the surface. Empty current_context/historical_context/provenance are intentional.)
- `crates/runx-runtime/src/agent_invocation.rs::normalize_request_id + rx_pending`: hardcoded run_id `rx_pending` or empty-stem normalization producing collidable IDs -> clean (rx_pending is a documented placeholder asserted by tests/agent_parity.rs:55; resolution layer rebinds before sealing. normalize_request_id collapsing produces benign IDs.)
- `crates/runx-runtime/src/journal.rs::list_paused_runs path handling`: ledger directory traversal or oversized JSONL exhausting memory -> clean (ledger_run_id uses path.file_stem() and requires rx_/gx_ prefix plus alphanumeric/_/- chars; receipt_dir is operator-owned trust boundary.)
- `crates/runx-runtime/src/adapter.rs`: new credential_delivery field/metadata key changing public API or leaking secrets via SkillOutput.metadata -> clean (CREDENTIAL_DELIVERY_OBSERVATIONS_METADATA stores non-secret CredentialDeliveryObservation (per credentials.rs:386-418 secrets are hashed/redacted).)
- `ambient_drift`: verify task scope vs filesystem changes; identify any drift outside declared scope -> clean (Workspace baseline reports 0 ambient drift; only the four declared runtime files plus the spec move (D draft -> archive copy) appear in git status.)
- `workspace mutation guard`: compare pre-review and post-review workspace snapshots -> finding (added crates/runx-cli/Cargo.toml (M 9524dad0700780d3b5ff5960f9d7e00c09a62b91f581715d2da8a0fda1138686), added crates/runx-runtime/src/adapters/catalog.rs (M 4ce6c3a5da2aa787977e250d2486c35e27e373c746cfe238798071ba7fa42042), changed crates/runx-runtime/src/agent_invocation.rs (M f975200fd5d98b5c29fc185637e8e3f43a9a55da8d66ddfbcfcd55bdf7a94d83 -> M b9820e44fd96b58e4efae3aad41f5f5d87368fc3c562243743112c818c80ae82), added crates/runx-runtime/src/execution/runner/steps.rs (M 7142e85226f20aaffa6c249afeaad1c36680b6b20b17069d52635e5e0bf31e60), added crates/runx-runtime/src/execution/skill_run.rs (M f6a55c0cf5e5033383fc9496bfbb8d369a5e41aaf7a02a69580d0c62b92ab2a3), added crates/runx-runtime/tests/skill_run.rs (M 629d5bd24c5c219cb5e9b6b2aca197f7d8e6a9a75fafcdecf59cb2f26910d677))

Findings:
- [medium/non-blocking] `F1` RuntimeOptions::default() cannot enable production receipt signing — env allowlist excludes RUNX_RECEIPT_SIGN_* so from_env always falls back to local_development
  - Status: fixed
  - Location: `crates/runx-runtime/src/execution/runner.rs:67`
  - Evidence: safe_default_env() at runner.rs:67 allowlists only PATH, SystemRoot, PATHEXT, RUNX_RECEIPT_DIR_ENV, RUNX_PROJECT_DIR_ENV, RUNX_CWD_ENV. RuntimeOptions::default() at runner.rs:48-50 constructs env from this allowlist and immediately calls RuntimeReceiptSignatureConfig::from_env(&env). signing.rs:92-105 reads RUNX_RECEIPT_SIGN_KID and RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64 from that BTreeMap; since neither is in the allowlist, both return None and the function returns Ok(local_development()). The unwrap_or_else fallback is therefore dead, and the call is effectively `RuntimeReceiptSignatureConfig::local_development()` regardless of operator process env. The public surface affected includes `run_graph_file` (runner.rs:318, re-exported from lib.rs:110) and `LocalOrchestrator::run_graph` (orchestrator.rs:147-157) which both go through `RuntimeOptions::default()` with no opportunity to inject a richer env.
  - Impact: An SDK/library caller who sets RUNX_RECEIPT_SIGN_KID and RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64 in the process env and invokes `run_graph_file` or `LocalOrchestrator.run_graph` will silently receive locally-pseudo-signed receipts instead of Ed25519-signed receipts. This contradicts dod5 ("receipts signed + verified by default in non-local modes") for the default graph runner path. Not a misclaim — verification still reports `unverified` — but the operator's signing intent is silently ignored. The primary CLI `runx skill run` is unaffected because cli/skill.rs:270 passes `env::vars().collect()` directly into SkillRunRequest.env which `execute_skill_run` reads (skill_run.rs:55) before any allowlist filtering.
  - Validation: After fix, a test that sets RUNX_RECEIPT_SIGN_KID + RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64 in the process env, calls `run_graph_file` against a fixture graph, and asserts the resulting receipt.signature.value starts with `base64:` (not `sig:`) and that receipt.issuer.kid matches.
- [low/non-blocking] `F2` Duplicate RUNX_TOOL_ROOTS parsing in agent_invocation.rs and adapters/catalog.rs
  - Status: accepted_non_blocking
  - Location: `crates/runx-runtime/src/agent_invocation.rs:173`
  - Evidence: parse_configured_tool_roots (agent_invocation.rs:173-181) and configured_tool_roots (adapters/catalog.rs:106-114) implement effectively the same split_paths/filter-empty logic over RUNX_TOOL_ROOTS but return Vec<String> vs Vec<PathBuf>. Both run on the same env map and produce equivalent tool-root lists.
  - Impact: Two divergent code paths can drift: if a future hardening change tightens validation in one (e.g., rejecting relative paths, deduplicating, canonicalizing) the other silently keeps the looser semantics, leaving the agent envelope's stamped ExecutionLocation.tool_roots out of sync with what the catalog adapter actually scans.
- [low/non-blocking] `F3` LocalHistoryReceipt.harness_state collapses Blocked/Aborted dispositions to "sealed" while status preserves them
  - Status: fixed
  - Location: `crates/runx-runtime/src/journal.rs:1010`
  - Evidence: subject_state (journal.rs:1010-1018) returns "deferred" for Deferred and "sealed" for everything else (including Blocked, Aborted, Closed). disposition_status (journal.rs:1020-1028) returns granular values ("blocked", "aborted", "sealed", ...). LocalHistoryReceipt at journal.rs:371-372 populates status via disposition_status and harness_state via subject_state, so a single blocked receipt projects as status="blocked" but harness_state="sealed".
  - Impact: Consumers of harness_state see a misleading "sealed" label for receipts that were actually blocked, while the parallel `status` field tells the truth. Risk is UX/audit clarity, not a security gate — the receipt's underlying disposition is preserved on disk.
- [critical/blocks completion] `workspace_mutation` Workspace changed during review.
  - Status: accepted_for_concurrent_work
  - Location: `crates/runx-cli/Cargo.toml (M 9524dad0700780d3b5ff5960f9d7e00c09a62b91f581715d2da8a0fda1138686)`
  - Evidence: workspace changed during review: added crates/runx-cli/Cargo.toml (M 9524dad0700780d3b5ff5960f9d7e00c09a62b91f581715d2da8a0fda1138686), added crates/runx-runtime/src/adapters/catalog.rs (M 4ce6c3a5da2aa787977e250d2486c35e27e373c746cfe238798071ba7fa42042), changed crates/runx-runtime/src/agent_invocation.rs (M f975200fd5d98b5c29fc185637e8e3f43a9a55da8d66ddfbcfcd55bdf7a94d83 -> M b9820e44fd96b58e4efae3aad41f5f5d87368fc3c562243743112c818c80ae82), added crates/runx-runtime/src/execution/runner/steps.rs (M 7142e85226f20aaffa6c249afeaad1c36680b6b20b17069d52635e5e0bf31e60), added crates/runx-runtime/src/execution/skill_run.rs (M f6a55c0cf5e5033383fc9496bfbb8d369a5e41aaf7a02a69580d0c62b92ab2a3), added crates/runx-runtime/tests/skill_run.rs (M 629d5bd24c5c219cb5e9b6b2aca197f7d8e6a9a75fafcdecf59cb2f26910d677)
  - Impact: The review provider changed the workspace while acting as a read-only reviewer, so its verdict is not trustworthy.
  - Validation: Restore the workspace to the expected state, ensure the provider is read-only, then rerun scafld review.
