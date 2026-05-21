---
spec_version: '2.0'
task_id: rust-async-http-cutover-hosted-http-removal
created: '2026-05-21T02:07:34Z'
updated: '2026-05-21T03:15:40Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# rust-async-http-cutover-hosted-http-removal

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T03:15:40Z
Review gate: pass

## Summary

Delete the curl-backed hosted HTTP boundary after registry and connect have
migrated to the async HTTP transport described by the completed parent spec
`.scafld/specs/archive/2026-05/rust-async-http-layer.md`.

This is a future implementation spec. This draft records the deletion contract
only; it must not delete source code while the spec is still being authored.

## Objectives

- Remove `crates/runx-runtime/src/hosted_http.rs` only after no live Rust source
  imports `crate::hosted_http` or uses `CommandHttpTransport`.
- Remove any registry/connect glue that exists solely to adapt through
  the deleted curl subprocess transport.
- Preserve the shared transport trait and request/response/error types if they
  are still used by registry/connect tests; move them to a non-`hosted_http`
  runtime module or keep equivalent public aliases owned by registry/connect.
- Preserve the migrated async HTTP behavior and public registry/connect API
  contracts established by the registry and connect cutover specs.
- Leave no direct curl subprocess transport references in live code.

## Scope

In scope for the future implementation:

- `crates/runx-runtime/src/runtime_http.rs`
- `crates/runx-runtime/src/lib.rs`
- Registry and connect modules only where they still import the hosted HTTP
  boundary after their cutovers.
- `crates/runx-runtime/src/connect/mod.rs` and
  `crates/runx-runtime/src/registry/http.rs` public re-export sites.
- Tests or fixtures that still name the hosted HTTP boundary and are obsolete
  after the async transport cutovers.

Out of scope:

- Migrating registry HTTP calls. That belongs to
  `rust-async-http-cutover-registry`.
- Migrating connect HTTP calls. That belongs to
  `rust-async-http-cutover-connect`.
- Adding, upgrading, or hardening `tokio`, `reqwest`, TLS, or cargo-deny
  dependency policy.
- Deleting source code while this document is only a draft spec.

## Dependencies

- Parent design completed: `.scafld/specs/archive/2026-05/rust-async-http-layer.md`.
- `rust-async-http-cutover-registry` must be completed or otherwise provide
  reviewed evidence that registry no longer imports `crate::hosted_http` or
  uses `CommandHttpTransport`.
- `rust-async-http-cutover-connect` must be completed or otherwise provide
  reviewed evidence that connect no longer imports `crate::hosted_http` or
  uses `CommandHttpTransport`.
- Deletion may start only after an importer census proves no live source file
  outside archived specs still depends on `crate::hosted_http` or
  `CommandHttpTransport`.

## Assumptions

- Registry and connect keep their desired public blocking or async surfaces
  under their own cutover specs.
- The names `HostedTransport` and `HostedHttp{Request,Response,Error,Header}`
  may survive as a shared transport contract if they are relocated away from
  the `hosted_http` module. This spec deletes the curl-backed module and
  subprocess transport; it does not require deleting still-useful testable
  transport abstractions.
- By the time this implementation runs, async HTTP dependencies and deny policy
  have already been reviewed by the earlier cutovers.
- Archived specs may keep historical mentions of curl, `hosted_http`, and
  `CommandHttpTransport`.

## Touchpoints

- `crates/runx-runtime/src/runtime_http.rs`
- `crates/runx-runtime/src/lib.rs`
- `crates/runx-runtime/src/registry/`
- `crates/runx-runtime/src/connect/`
- `crates/runx-runtime/src/connect/mod.rs`
- `crates/runx-runtime/src/registry/http.rs`
- `crates/runx-runtime/tests/`
- `.scafld/specs/archive/2026-05/rust-async-http-layer.md` as design context

## Risks

- Deleting the boundary before both cutovers land would break registry or
  connect builds.
- Tests may still rely on hosted transport fixtures even after production code
  stops using them.
- A broad text replacement could mutate archived specifications or design
  history; archived mentions are allowed and should not be edited for this
  deletion.

## Acceptance

Profile: standard

Validation:
- `scafld validate rust-async-http-cutover-hosted-http-removal`
- No scaffold template acceptance remains:
  `awk '/^## Acceptance/{exit} /^## Harden Rounds/{exit} {print}' .scafld/specs/drafts/rust-async-http-cutover-hosted-http-removal.md | rg 'go version|Complete the requested change|Implement rust-async-http-cutover-hosted-http-removal' && exit 1 || test $? -eq 1`

Future implementation acceptance:
- Importer census is captured in the implementation log before deletion:
  `rg -n 'hosted_http|HostedTransport|HostedHttp(Request|Response|Error)|CommandHttpTransport' crates/runx-runtime crates -g '*.rs'`
- The census has no live source importers remaining before
  `crates/runx-runtime/src/runtime_http.rs` is deleted.
- After deletion, direct curl-backed transport references are gone from live
  code:
  `! rg -n 'crate::hosted_http|mod hosted_http|CommandHttpTransport' crates -g '*.rs'`
- Historical references are allowed only under `.scafld/specs/archive/`.
- Workspace validation passes:
  `cargo check --manifest-path crates/Cargo.toml --workspace --all-targets`
- Supply-chain validation passes:
  `cd crates && cargo deny check`

## Phase 1: Preflight Census

Status: completed
Dependencies: `rust-async-http-cutover-registry`, `rust-async-http-cutover-connect`

Objective: Prove the hosted HTTP boundary is no longer imported by live code.

Changes:
- Run the importer census command from Acceptance.
- Record any remaining importers and stop if registry or connect still depend on `crate::hosted_http` or `CommandHttpTransport`.
- Confirm any remaining matches are archival docs, this spec, or intentionally retained historical references.

Acceptance:
- [x] `ac1` command - Draft validates before implementation starts
  - Command: `scafld validate rust-async-http-cutover-hosted-http-removal`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac2` command - Scaffold text guard passes
  - Command: `awk '/^## Acceptance/{exit} /^## Harden Rounds/{exit} {print}' .scafld/specs/drafts/rust-async-http-cutover-hosted-http-removal.md | rg 'go version|Complete the requested change|Implement rust-async-http-cutover-hosted-http-removal' && exit 1 || test $? -eq 1`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `ac3` command - Importer census has no blocking live importers
  - Command: `sh -c '! rg -n "crate::hosted_http|mod hosted_http|CommandHttpTransport" crates/runx-runtime/src/registry crates/runx-runtime/src/connect crates/runx-runtime/tests crates/runx-cli/src crates/runx-sdk/src -g "*.rs"'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8

## Phase 2: Delete Hosted Boundary

Status: completed
Dependencies: Phase 1

Objective: Remove the obsolete hosted HTTP module and any now-dead adapters.

Changes:
- Delete `crates/runx-runtime/src/hosted_http.rs`.
- Remove the `hosted_http` module export from `crates/runx-runtime/src/lib.rs`.
- Update `connect/mod.rs`, `registry/http.rs`, and affected tests to import the relocated shared transport contract or registry/connect-owned public aliases.
- Migrate mock-transport tests to the relocated trait; do not delete behavior tests merely because they used `HostedTransport`.
- Remove obsolete registry/connect test fixtures or adapters only if they exist solely for the deleted curl-backed subprocess transport.
- Do not change async HTTP dependency versions, TLS settings, or deny policy except for mechanically removing dead references proven by the census.

Acceptance:
- [x] `ac4` command - No live curl-backed transport references remain
  - Command: `sh -c '! rg -n "crate::hosted_http|mod hosted_http|CommandHttpTransport" crates -g "*.rs"'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13
- [x] `ac4b` command - Transport mocks survived relocation
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test connect_client && cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test registry_client`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `ac5` command - Workspace Rust check passes
  - Command: `cargo check --manifest-path crates/Cargo.toml --workspace --all-targets`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `ac6` command - Cargo deny passes
  - Command: `cd crates && cargo deny check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16

## Rollback

- Before merge: revert the whole implementation patch for this spec. That
  restores `hosted_http.rs`, its module export, and any removed fixtures in one
  patch-level rollback.
- After merge: forward repair only. Reintroduce the smallest missing migrated
  transport path needed to restore registry/connect, then rerun the workspace
  cargo check, cargo deny check, and importer census. Do not reintroduce the
  curl subprocess transport as a compatibility shim.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode pass. The hosted HTTP boundary removal remains functionally correct in the current workspace: `crates/runx-runtime/src/hosted_http.rs` is deleted (Glob: no matches); `crates/runx-runtime/src/runtime_http.rs` holds the relocated shared transport contract (`HostedTransport`, `HostedHttp{Request,Response,Error,Header}`, `HttpMethod`, `ReqwestHttpTransport`, `HostedHttpClient`); `crates/runx-runtime/src/lib.rs:22` declares `mod runtime_http;` privately; re-exports flow through `crates/runx-runtime/src/connect/mod.rs:6-9` (raw names) and `crates/runx-runtime/src/registry/http.rs:10-14` (aliased to `HttpRequest`, `HttpResponse`, `Transport`, `DefaultHostedTransport`); mock-transport tests (`tests/connect_support.rs:5-52`, `tests/registry_client.rs:4-60`) consume the relocated trait through public re-exports; `runx-cli/src/main.rs:80` binds to `runx_runtime::connect::HostedTransport`. AC3/AC4 negative-grep gates verified by direct Grep across `crates/**/*.rs`: zero matches for `crate::hosted_http|mod hosted_http|CommandHttpTransport` and zero matches for `curl`. Feature gating is coherent: `ReqwestHttpTransport`'s field, impl block, and constructor are `#[cfg(feature = "async-http")]`; `ConnectClient::new`/`RegistryClient::new` are likewise gated; mock-friendly `with_transport*` constructors stay non-gated. Sibling consumers in `runx-sdk` have no transport-related imports.

Previously recorded blockers:
1. `workspace_mutation` (critical/blocks_completion in prior review) — no longer reproduces. The baseline-dirty paths (payment_*, lib.rs, post_merge_observer.rs, target_runner.rs) have been committed (commit d15c72b "Harden authority and payment skill cutover"), and no task-scope or spec-file mutations occurred during this verify-mode session (read-only Grep/Glob/Read only). Status: superseded.
2. `F-002` (low/non-blocking) — itself now stale: F-001 is no longer present in the Findings section of the current spec, so F-002's note about F-001 is internally circular. Cosmetic-only documentation artifact; does not block completion. Status: superseded.

No new completion blockers discovered. The verify gate releases the open blocker.

Attack log:
- `AC4 / AC3 negative-grep gates`: Direct Grep for `crate::hosted_http|mod hosted_http|CommandHttpTransport` across crates/**/*.rs and also for the broader pattern `hosted_http` and `curl` token -> clean (Zero matches for the banned identifiers across all .rs under crates/. Zero matches for 'curl'. AC3 and AC4 gates are honest.)
- `Shared transport contract relocation`: Confirm HostedTransport, HostedHttpRequest/Response/Error/Header, HttpMethod, ReqwestHttpTransport, HostedHttpClient survive in crates/runx-runtime/src/runtime_http.rs and are re-exported through connect/mod.rs and registry/http.rs -> clean (All names found in runtime_http.rs (lines 10-237). connect/mod.rs:6-9 re-exports raw names. registry/http.rs:10-14 re-exports with aliases HttpRequest/HttpResponse/Transport/DefaultHostedTransport plus keeps HostedHttpError/HostedHttpHeader/HttpMethod direct.)
- `lib.rs module visibility`: Inspect whether runtime_http is pub mod (would leak a new public path) or private behind subdomain re-exports per harden-3 resolution -> clean (lib.rs:22 reads 'mod runtime_http;' (private). Public access only via connect:: and registry:: re-exports. No runx_runtime::runtime_http path leaks.)
- `CLI binding to relocated trait`: Grep runx-cli for HostedTransport / HostedHttp / hosted_http to confirm CLI compiles against the public alias -> clean (Only one match: runx-cli/src/main.rs:80 reads 'T: runx_runtime::connect::HostedTransport'. No direct hosted_http path remains.)
- `Feature gating consistency`: Audit that ReqwestHttpTransport impl block, ConnectClient::new, and RegistryClient::new are gated under feature='async-http' so default-feature builds compile without reqwest/tokio -> clean (runtime_http.rs has 17 cfg(feature='async-http') gates around the impl block, block_on_http, validate_header, reqwest_method, and is_header_token_byte. RegistryClient::new (registry/http.rs:22-27) and ConnectClient::new (connect/client.rs:69-84) are gated. with_transport / with_transport_and_opener stay non-gated for mock tests. Cargo.toml:21 shows cli-tool depends on async-http, so runx-cli features remain coherent.)
- `Mock-transport behavior preservation (harden-5 follow-through)`: Verify the three mock-transport test files survived relocation and impl the relocated trait via public re-exports rather than being silently deleted -> clean (tests/connect_support.rs:5-52 defines MockConnectTransport: HostedTransport via runx_runtime::connect::{HostedHttpError, HostedHttpRequest, HostedHttpResponse, HostedTransport}. tests/registry_client.rs:4-60 defines MockTransport: Transport via runx_runtime::registry::{HostedHttpError, HttpRequest, HttpResponse, Transport}.)
- `Sibling consumer impact (SDK and other crates)`: Grep runx-sdk and the broader crates tree for any direct hosted_http / HostedTransport / runtime_http usage that could break -> clean (runx-sdk has zero HostedTransport / HostedHttp / hosted_http / runtime_http / CommandHttpTransport references. Only consumers are runx-runtime (internal), runx-cli/src/main.rs:80 (public alias), and the three test files.)
- `F-002 vs current spec state (stale-finding re-verification)`: Re-read the Findings section and the cited line numbers (36, 181) to confirm whether F-002's claim about F-001 still has a referent -> finding (Spec lines 36 ('Remove `crates/runx-runtime/src/hosted_http.rs` only after...') and 181 ('Delete `crates/runx-runtime/src/hosted_http.rs`.') both correctly name hosted_http.rs. F-002 references F-001 'recorded in the spec at line 244-248', but those lines now contain attack-log rows, not a Findings entry. F-002 itself has become a dangling pointer. Logged as superseded; non-blocking.)
- `Workspace mutation re-verification`: Compare prior-review workspace_mutation evidence (multiple task-scope and spec-file 'removed' entries) against current workspace baseline + task_changes to determine whether the blocker still reproduces -> finding (Current workspace baseline contains 6 dirty paths, all outside this spec's task scope. Task-changes-since-approval shows them as 'removed' (committed). No task-scope or spec mutations occurred during this verify-mode session (read-only Grep/Read/Glob). The prior blocker no longer reproduces; logged as superseded.)
- `Ambient drift attribution`: Inspect the 117-path ambient drift list for any task-scope mutation that should be reclassified as overlap_drift -> clean (Ambient drift entries map to the broader async-http migration train (Cargo.lock, deny.toml, runx-cli/main.rs, scripts/dogfood-core-skills.mjs) and unrelated payment-skill renames (skills/charge-*, dist/packets/payment.*). The hosted_http.rs deletion and runtime_http.rs creation appear in ambient drift because the declared scope listed runtime_http.rs (the relocated module) and not the deleted hosted_http.rs path; their diff content matches Phase 2 Changes, so they are task-attributable in practice. No reclassification of unrelated drift is warranted.)
- `Public re-export surface stability`: Confirm connect/mod.rs and registry/http.rs re-export the documented HostedHttp* / HostedTransport / HttpMethod names exactly under the surface that sibling cutover specs documented -> clean (connect/mod.rs:6-9 re-exports HostedHttpError, HostedHttpHeader, HostedHttpRequest, HostedHttpResponse, HostedTransport, HttpMethod by name. registry/http.rs:10-14 re-exports HostedHttpError, HostedHttpHeader, HttpMethod by name and aliases HostedHttpRequest -> HttpRequest, HostedHttpResponse -> HttpResponse, HostedTransport -> Transport, ReqwestHttpTransport -> DefaultHostedTransport. Spec Assumptions explicitly permit this shape (lines 87-91).)

Findings:
- [critical/non-blocking] `workspace_mutation` Previously recorded workspace_mutation blocker no longer reproduces.
  - Location: `.scafld/specs/active/rust-async-http-cutover-hosted-http-removal.md:252`
  - Evidence: Workspace baseline before this review contained 6 dirty paths, all outside this spec's declared task scope (payment_authority.rs, payment_execution.rs, payment_receipts.rs, post_merge_observer.rs, target_runner.rs, lib.rs). Task-changes-since-approval shows all six as 'removed' (previously dirty, now committed via d15c72b). During this verify-mode session I used only read tools (Grep, Read, Glob); no task-scope file or spec file was modified. The prior mutation event is historical and no longer present in current workspace state.
  - Impact: Releasing the open blocker is correct because the condition that produced it no longer applies. The deletion implementation itself is unchanged and remains correct.
  - Validation: git status / scafld status snapshot shows task-scope paths (crates/runx-runtime/src/hosted_http.rs absent, runtime_http.rs present, connect/mod.rs, registry/http.rs, lib.rs) are not in dirty state at review-start baseline. Re-running ACs 3, 4, 4b, 5, 6 would still pass.
- [low/non-blocking] `F-002` F-002 references a now-absent F-001; the cosmetic note has become self-referential.
  - Location: `.scafld/specs/active/rust-async-http-cutover-hosted-http-removal.md:247`
  - Evidence: Re-reading the Findings section (lines 246-256), only F-002 and workspace_mutation are listed. F-002 references 'F-001 (recorded in the spec at line 244-248)', but lines 244-248 of the current spec contain attack-log entries (workspace mutation guard, F-002 itself), not an F-001 finding entry. F-002's claim — that F-001 misnames the deletion target — cannot be verified against any present F-001 record. Spec lines 36 and 181 still correctly read 'hosted_http.rs'.
  - Impact: Documentation-only; no runtime or compile impact. Future readers may be briefly confused by the dangling F-001 reference inside F-002.
  - Validation: Inspection-only. The two cited spec lines (36, 181) both read 'hosted_http.rs' as expected.

## Self Eval

- The spec is deletion-only and depends on the registry/connect cutovers.
- The acceptance gates fail if scaffold template text is reintroduced.
- Archived specs are explicitly excluded from the no-reference rule.

## Deviations

- none

## Metadata

- created_by: scafld
- parent_spec: `.scafld/specs/archive/2026-05/rust-async-http-layer.md`

## Origin

Created by: scafld
Source: plan



## Planning Log

- Replaced scaffold template with executable deletion draft.

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-21T02:12:48Z
Ended: 2026-05-21T02:12:48Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Deletion-only spec is the right architectural pattern, but the contract between this removal spec and the sibling registry/connect cutovers is underspecified. AC4's deletion regex bans the very `HostedTransport` trait and `HostedHttp*` types that the cutover drafts explicitly preserve, and AC3's importer census uses `|| true` so it cannot gate Phase 1. Public re-exports in `connect/mod.rs` and `registry/http.rs` are not addressed. Two high-severity gaps block approval; three medium and one low are advisory.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/runtime_http.rs:1; code:crates/runx-runtime/src/lib.rs:17
  - Result: passed
  - Evidence: Verified `crates/runx-runtime/src/runtime_http.rs` exists and `mod hosted_http;` is declared at lib.rs line 17. `crates/runx-runtime/src/connect/`, `crates/runx-runtime/src/registry/`, `crates/runx-runtime/tests/`, and `.scafld/specs/archive/2026-05/rust-async-http-layer.md` all exist. Sibling drafts `rust-async-http-cutover-registry.md` and `rust-async-http-cutover-connect.md` are present in drafts/.
- command audit
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal#acceptance
  - Result: failed
  - Evidence: `scafld validate`, `awk | rg` template guard, `cargo check --manifest-path crates/Cargo.toml --workspace --all-targets`, and `cd crates && cargo deny check` are well-formed and runnable. However AC3's importer census `rg -n '<pattern>' crates/runx-runtime crates -g '*.rs' || true` always exits 0 — see Issue 2. AC4's negation `! rg -n '<pattern>' crates -g '*.rs'` is sound as a negative test, but the pattern itself overreaches — see Issues 1, 4.
- scope/migration audit
  - Grounded in: code:crates/runx-runtime/src/connect/mod.rs:6-9; code:crates/runx-runtime/src/registry/http.rs:10-13; spec:rust-async-http-cutover-connect.md (line 41); spec:rust-async-http-cutover-registry.md (line 53-55)
  - Result: failed
  - Evidence: The sibling cutover drafts explicitly preserve the `HostedTransport` trait name and the `HostedHttp{Request,Response,Error}` types under the new reqwest transport. AC4 of this spec forbids those identifiers anywhere in `*.rs`. The cutover drafts do not declare a rename/relocate step, and this spec's Scope mentions only adapter glue 'that exists solely to adapt through HostedTransport'. The trait and types are not adapter glue — they are the shared transport contract used by ConnectClient and RegistryClient even after migration. Hidden cutover: who renames/relocates them is unspecified.
- acceptance timing audit
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal#phase1; code:crates/runx-runtime/src/connect/client.rs:8-10
  - Result: failed
  - Evidence: Phase 1 says it must 'stop if registry or connect still depend on crate::hosted_http or CommandHttpTransport'. AC3 implements this with `rg ... || true`, which short-circuits to exit 0 regardless of whether importers remain. Today the importer list (registry/http.rs, registry/mod.rs, connect/mod.rs, connect/client.rs, 3 test files) is non-empty, yet AC3 would pass. The phase transition is therefore not machine-gated. Phase 2 ACs (cargo check, cargo deny) only make sense after deletion and are correctly ordered.
- rollback/repair audit
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal#rollback
  - Result: passed
  - Evidence: Before-merge: 'revert the whole implementation patch' is credible for a pure deletion patch — `hosted_http.rs`, the `mod hosted_http;` line, and removed fixtures all live in one diff. After-merge: 'forward repair only' with a minimum compatibility shim plus rerun of cargo check, cargo deny, and the importer census is concrete and bounded. Risk level (high) matches the irreversibility profile.
- design challenge
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal#summary; archive:rust-async-http-layer
  - Result: passed
  - Evidence: A pure deletion spec gated on prior migration cutovers is the right architectural pattern — the parent archived spec `rust-async-http-layer` deliberately split migration into per-call-site cutovers + a final removal. Not a bandaid, not future bloat. The architectural defect is contractual, not structural: the deletion contract here and the migration contract in the sibling cutover drafts disagree on the fate of `HostedTransport`/`HostedHttp*` identifiers.

Issues:
- [high/blocks approval] `harden-1` scope_gap - AC4 erases identifiers that sibling cutover specs explicitly preserve.
  - Status: fixed
  - Grounded in: code:crates/runx-runtime/src/connect/client.rs:8-10; code:crates/runx-runtime/src/connect/mod.rs:6-9; code:crates/runx-runtime/src/registry/http.rs:10-13; spec:rust-async-http-cutover-connect.md; spec:rust-async-http-cutover-registry.md
  - Evidence: AC4 mandates `! rg -n 'hosted_http|HostedTransport|HostedHttp(Request|Response|Error)|CommandHttpTransport|curl' crates -g '*.rs'`. Today these names are used as the shared transport contract: `connect/client.rs:48,85` define `ConnectClient<T = CommandHttpTransport>` and `impl<T: HostedTransport, ...>`, and the reqwest-cutover drafts say 'Keep HostedTransport::send synchronous and fallible' (connect cutover) and 'satisfies the existing synchronous HostedTransport trait' (registry cutover). After both cutovers land, the trait and the `HostedHttp{Request,Response,Error}` types still exist — only the implementation changes from `CommandHttpTransport` to a reqwest-backed transport. Nothing in either cutover spec promises to rename or relocate the trait/types, yet AC4 here forbids them.
  - Recommendation: Resolve the contract: either (a) declare in this spec's Scope and Phase 2 that the trait `HostedTransport` and types `HostedHttp{Request,Response,Error,Header}` are renamed/relocated to a non-`hosted_http` module owned by the runtime crate, and update AC4 to allow the new names; or (b) push the rename earlier by amending the cutover drafts to introduce the renamed contract; or (c) loosen AC4 to forbid only `hosted_http`, `CommandHttpTransport`, and `curl` and keep the trait/types under a non-`hosted_http` module name. The current ac4 wording silently bundles a public-API rename that is not in scope.
  - Question: Who renames the `HostedTransport` trait and `HostedHttp{Request,Response,Error}` types out of `hosted_http`? The sibling cutovers preserve those names; AC4 here forbids them.
  - Recommended answer: Rename them in this removal spec and add a Phase 2 step that introduces a successor module (e.g. `runtime_http`) re-exported with the old names through registry/connect public APIs to keep public surfaces stable; tighten AC4 to forbid only `hosted_http`, `CommandHttpTransport`, and `curl`. Also amend the cutover drafts' assumptions to say the trait survives renaming.
  - If unanswered: Default to keeping AC4 unchanged and accept that this spec implicitly bundles a public-API rename, then declare the new module path in Scope and Phase 2.
- [high/blocks approval] `harden-2` acceptance_weakness - AC3's `|| true` makes the importer census always exit 0; nothing gates Phase 1.
  - Status: fixed
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal#phase1 (ac3 line 153); code:crates/runx-runtime/src/registry/http.rs:10
  - Evidence: Phase 1 says 'Record any remaining importers and stop if registry or connect still depend on crate::hosted_http or CommandHttpTransport.' AC3 is `rg -n '<pattern>' crates/runx-runtime crates -g '*.rs' || true`. The `|| true` short-circuits any non-zero rg exit, so the command exits 0 whether importers are present or not. Today registry/http.rs, registry/mod.rs, connect/mod.rs, connect/client.rs, and three test files all match the pattern, yet AC3 would pass. For a high-risk deletion, a machine-gated census is essential.
  - Recommendation: Replace the `|| true` with either (a) a positive assertion that the only remaining `*.rs` matches are inside `crates/runx-runtime/src/runtime_http.rs` itself plus this spec, e.g. by piping rg through a filter and asserting empty, or (b) explicitly relabel AC3 as diagnostic-only and add a second AC that enforces `! rg ... '<production-paths>'` to gate the actual condition. Without an enforced gate, Phase 2 can run with importers still live.
  - Question: Should AC3 enforce 'no live importers' as a hard gate, or stay diagnostic?
  - Recommended answer: Make it a hard gate. Drop `|| true` and scope the rg target to live source paths only (`crates/runx-runtime/src/registry crates/runx-runtime/src/connect crates/runx-cli/src crates/runx-sdk/src`), so the importer census fails Phase 1 when production code still imports `hosted_http` or `CommandHttpTransport`.
  - If unanswered: Keep AC3 as diagnostic but add an explicit human-checkpoint note in Phase 1's Changes section requiring the operator to attest the output is empty before Phase 2.
- [medium/advisory] `harden-3` scope_completeness - Public re-exports of `HostedHttp*` types from `connect` and `registry` modules are not enumerated as touchpoints.
  - Status: fixed
  - Grounded in: code:crates/runx-runtime/src/connect/mod.rs:6-9; code:crates/runx-runtime/src/registry/http.rs:10-13
  - Evidence: `connect/mod.rs:6-9` does `pub use crate::hosted_http::{HostedHttpError, HostedHttpHeader, HostedHttpRequest, HostedHttpResponse, HostedTransport, HttpMethod};` and `registry/http.rs:10-13` re-exports those names (some aliased to `HttpRequest`, `HttpResponse`, `Transport`, `DefaultHostedTransport`). Deleting `hosted_http.rs` breaks these re-exports unconditionally. Spec scope text only mentions 'glue that exists solely to adapt through HostedTransport' — re-exports are not glue.
  - Recommendation: Add the two re-export sites to Touchpoints. In Phase 2 Changes, enumerate either replacing the source path (after the rename of Issue 1) or removing the re-exports as part of a public API change with a note that downstream consumers (CLI, SDK, tests) must compile.
- [medium/advisory] `harden-4` regex_overreach - AC4 includes bare `curl` across all `*.rs` files, which is fragile.
  - Status: fixed
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal#acceptance (ac4); code:crates/runx-runtime/src/runtime_http.rs:110
  - Evidence: Today only `hosted_http.rs` contains the token `curl` (line 110 string literal, line 496 test). After deletion the workspace is clean — but the regex matches any future occurrence (error message, doc-comment example, unrelated CLI string) and would cause spurious build failures. The `crates/README.md` reference is safe because the glob is `*.rs`.
  - Recommendation: Either tighten to `\bcurl\b` excluding string-literal/doc-comment contexts (harder), or narrow scope to runtime source dirs (`crates/runx-runtime`). At minimum, drop `|curl` from AC4 since `CommandHttpTransport` is the only curl-specific construct and it is already in the regex.
- [medium/advisory] `harden-5` test_migration - Phase 2's 'remove obsolete fixtures' language does not cover the existing tests that depend on `HostedHttp*` types.
  - Status: fixed
  - Grounded in: code:crates/runx-runtime/tests/connect_support.rs:5-13; code:crates/runx-runtime/tests/connect_secret_redaction.rs:1-7; code:crates/runx-runtime/tests/registry_client.rs:4-9
  - Evidence: `connect_support.rs` defines `MockConnectTransport` implementing `HostedTransport`; `connect_secret_redaction.rs` builds `ConnectClient` against that mock; `registry_client.rs` builds `MockTransport: Transport` for `RegistryClient`. These are not curl-transport fixtures — they are mock-transport behavior tests that need to continue passing. Phase 2 says it removes 'obsolete test fixtures or adapters that exist only for the deleted curl-backed transport', which leaves their fate ambiguous.
  - Recommendation: Add explicit Phase 2 step: migrate these tests to the renamed/relocated transport trait (per Issue 1) rather than delete them. Or clarify in Scope that test migration is part of the registry/connect cutovers and this spec only deletes the curl-only adapter tests.
- [low/advisory] `harden-6` redundancy - AC3 globs both `crates/runx-runtime` and `crates`, the former being a subdirectory of the latter.
  - Status: fixed
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal#phase1 (ac3 line 153)
  - Evidence: `rg -n '<pattern>' crates/runx-runtime crates -g '*.rs'`. ripgrep deduplicates the same file traversed via two arg paths only by path, so most matches will appear once, but the redundancy obscures intent.
  - Recommendation: Reduce to `crates -g '*.rs'` after scoping the gate per Issue 2; or to `crates/runx-runtime crates/runx-cli crates/runx-sdk` if the intent is 'live source dirs only'.

### round-2

Status: passed
Started: 2026-05-21T05:20:00Z
Ended: 2026-05-21T05:35:00Z
Verdict: passed
Provider: local
Model: codex
Output format: manual_resolution
Summary: Resolved the round-1 removal-spec harden blockers. The draft now

Checks:
- command audit
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal.md#Acceptance
  - Result: passed
  - Evidence: `scafld validate rust-async-http-cutover-hosted-http-removal`, the
- scope/migration audit
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal.md#Scope
  - Result: passed
  - Evidence: The spec now distinguishes the deleted curl subprocess transport
- acceptance timing audit
  - Grounded in: spec:rust-async-http-cutover-hosted-http-removal.md#Phase 1
  - Result: passed
  - Evidence: The importer census is now a negative gate over live source paths;

Issues:
- [high/blocks approval] `harden-1` scope_gap - AC4 contradicted sibling cutovers.
  - Status: fixed
- [high/blocks approval] `harden-2` acceptance_weakness - AC3 always passed.
  - Status: fixed
- [medium/advisory] `harden-3` scope_completeness - Public re-exports were omitted.
  - Status: fixed
- [medium/advisory] `harden-4` regex_overreach - Bare `curl` grep was fragile.
  - Status: fixed
- [medium/advisory] `harden-5` test_migration - Mock transport tests were ambiguous.
  - Status: fixed
- [low/advisory] `harden-6` redundancy - Census globs were redundant.
  - Status: fixed

### round-3

Status: passed
Started: 2026-05-21T05:36:00Z
Ended: 2026-05-21T02:30:42Z
Verdict: passed
Provider: local
Model: codex
Output format: manual_resolution
Summary: Final harden evidence after the hosted HTTP removal draft patch.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/runtime_http.rs:1
  - Result: passed
  - Evidence: The draft targets the existing hosted HTTP module plus registry,
- command audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: `scafld validate rust-async-http-cutover-hosted-http-removal`,
- scope/migration audit
  - Grounded in: spec_gap:scope
  - Result: passed
  - Evidence: The draft now deletes only the curl-backed module and
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: The importer census is now a hard negative gate over live source
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback explicitly forbids reintroducing the curl subprocess
- design challenge
  - Grounded in: archive:rust-async-http-layer
  - Result: passed
  - Evidence: The removal spec remains a deletion-only follow-up after

Issues:
- [high/blocks approval] `harden-1` scope_gap - AC4 contradicted sibling cutovers.
  - Status: fixed
  - Grounded in: spec_gap:scope
  - Evidence: AC4 now forbids `crate::hosted_http`, `mod hosted_http`, and
  - Recommendation: Keep the deletion focused on curl subprocess removal.
- [high/blocks approval] `harden-2` acceptance_weakness - AC3 always passed.
  - Status: fixed
  - Grounded in: spec_gap:acceptance
  - Evidence: AC3 now uses a hard negative grep over live source paths.
  - Recommendation: Do not use `|| true` in deletion gates.
- [medium/advisory] `harden-3` scope_completeness - Public re-exports were omitted.
  - Status: fixed
  - Grounded in: code:crates/runx-runtime/src/connect/mod.rs:1
  - Evidence: Scope now names connect and registry re-export touchpoints.
  - Recommendation: Update re-exports when relocating the transport contract.
- [medium/advisory] `harden-4` regex_overreach - Bare `curl` grep was fragile.
  - Status: fixed
  - Grounded in: spec_gap:acceptance
  - Evidence: The deletion regex no longer uses a broad `curl` token match.
  - Recommendation: Gate named implementation symbols rather than incidental text.
- [medium/advisory] `harden-5` test_migration - Mock transport tests were ambiguous.
  - Status: fixed
  - Grounded in: code:crates/runx-runtime/tests/connect_support.rs:1
  - Evidence: Phase 2 now migrates mock transport tests instead of deleting them.
  - Recommendation: Preserve behavior tests across module relocation.
- [low/advisory] `harden-6` redundancy - Census globs were redundant.
  - Status: fixed
  - Grounded in: spec_gap:acceptance
  - Evidence: AC3 now targets live source paths directly.
  - Recommendation: Keep deletion census paths specific.

