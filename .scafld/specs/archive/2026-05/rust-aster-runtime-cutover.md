---
spec_version: '2.0'
task_id: rust-aster-runtime-cutover
created: '2026-05-18T00:00:00Z'
updated: '2026-05-21T17:11:57Z'
status: completed
harden_status: passed
size: large
risk_level: high
---

# Rust aster runtime cutover

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T17:11:57Z
Review gate: pass

## Summary

Plan the Aster runtime cutover from the local OSS state plus the adjacent Aster
checkout that is actually available. The OSS crate checkout does not include
`cloud/**`, so this spec cannot claim verified cloud package paths, UI paths,
hosted agent adapter files, or cloud DB approval routing. The full workspace
does include a sibling cloud repo, but those bindings stay deferred until a
dedicated pass inspects that tree and records exact paths. This draft therefore
does not settle the cloud `agent-runner` binding for the runtime-local/adapters
sunset.

Current local facts:

- `crates/runx-runtime/src/runtime_http.rs` is the hosted transport boundary
  visible in this checkout. It defines `HostedHttpClient`, `HostedTransport`,
  request and response types, header validation, reqwest/rustls-backed
  transport under the `async-http` feature, redirect suppression, bounded
  request/connect timeouts, and redacted debug behavior.
- Aster contract types exist in `crates/runx-contracts/src/aster.rs`.
- The contracts crate exports Aster control objects from
  `crates/runx-contracts/src/lib.rs`.
- A structural Aster control fixture exists at
  `fixtures/contracts/aster-control/public-feed-proof.json`.
- A runtime external fixture now exists for the local Aster Rust bridge shape:
  `fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml`.
- A focused runtime test now exists at
  `crates/runx-runtime/tests/external/aster_agent_step.rs`, wired through
  `crates/runx-runtime/tests/external.rs`.
- The local checkout has no `cloud/` directory and no
  `crates/runx-runtime/src/cloud_client.rs`.
- The readable Aster checkout at `/Users/kam/dev/runx/aster` currently routes
  Rust execution through `scripts/aster-core.mjs` into
  `scripts/runx-agent-bridge.mjs`; the accepted terminal skill report is
  `runx.skill_run.v1` with `status: "sealed"` and a stored
  `runx.harness_receipt.v1` receipt id.
- The Aster checkout dogfoods the Rust binary directly for harness execution;
  it does not invoke a JS/npm Runx CLI bridge for the proving-ground path.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external
  aster_agent_step` passes for the new fixture replay.

The cutover remains preservation-oriented: Aster should consume the Rust
runtime through a documented boundary and canonical contracts, but this draft
must not invent a cloud binding, claim an agent-step runtime fixture before
those files exist, or imply that custom adapter/plugin authors must link into
Rust. If Aster needs custom userland integration code, that belongs behind the
language-neutral external adapter/plugin protocol under Rust supervision rather
than behind `@runxhq/runtime-local`.

Cutover status: **deferred**. This draft ratifies the OSS-local Aster
contract, fixture, hosted-boundary, and dogfood evidence. It does not authorize
the cloud `agent-runner` binding or final runtime-local sunset. Those belong to
a future cloud-tree binding spec that can inspect `../cloud/**`.

## Context

CWD: `.` (runx OSS workspace)

Relevant existing local surfaces:

- `crates/runx-runtime/src/runtime_http.rs`
- `crates/runx-contracts/src/aster.rs`
- `crates/runx-contracts/src/lib.rs`
- `fixtures/contracts/aster-control/public-feed-proof.json`
- `crates/runx-contracts/tests/aster_control_fixtures.rs`
- `fixtures/operational-policy/nitrosend-like.json` as the current
  operational-policy readback proof point, not as an Aster runtime fixture.
- `.scafld/specs/drafts/runx-target-repo-runners.md`
- `.scafld/specs/drafts/runx-post-merge-closure-observer.md`

Surfaces not present in this checkout:

- `cloud/packages/**`
- `cloud/packages/agent-runner/**`
- `cloud/packages/api/**`
- `cloud/packages/db/**`
- `cloud/packages/receipts-store/**`
- `cloud/packages/ui/**`
- `crates/runx-runtime/src/cloud_client.rs`
- `cloud/**`

## Invariants

- Cloud binding is deferred until a checkout with the cloud tree is available.
  This spec may name the required boundary, but it must not assert verified
  cloud implementation paths in the OSS-only checkout.
- Cloud `agent-runner` binding is an open follow-up, not a settled Aster
  cutover fact. The later pass must choose an allowed stable boundary such as
  hosted HTTP, CLI JSON, service/FFI, or the external adapter/plugin protocol,
  and must not preserve a runtime-local fallback.
- Aster control objects use the existing `runx-contracts::aster` shapes. Do not
  create parallel Aster JSON shapes for target, opportunity, selection,
  reflection, skill binding, feed entry, or transition records.
- Runtime execution artifacts stay canonical harness, decision, act,
  verification/proof, and sealed `runx.harness_receipt.v1` objects.
- Aster must not read receipts through private local file paths in public or
  hosted projections; receipt access goes through runtime/store APIs or a
  documented hosted boundary.
- `runtime_http.rs` is the current internal hosted transport implementation.
  Its stable public consumers in this checkout are the re-exported hosted HTTP
  surfaces under `registry::http` and `execution::target_runner`; the previous
  Connect-facing wrapper has moved out of the public runtime boundary under
  `connect-auth-mit-boundary-v1`. Any future cloud binding should either use
  one of those current surfaces, widen/replace `runtime_http.rs` in a separate
  reviewed cloud-binding change, or choose another stable protocol boundary
  explicitly.
- External adapter/plugin use, if needed by Aster or cloud agent integrations,
  follows `external-adapter-plugin-protocol-v1`; this spec must not require
  provider-specific adapter code to become a Rust crate.
- No legacy/compat outcome, effect, verification proof alias, or Aster-only terminal
  packet is introduced.

## Objectives

- Preserve the Aster contract surface already present in
  `crates/runx-contracts/src/aster.rs` and its fixture coverage.
- Define the runtime external fixture that is missing today:
  `fixtures/external/aster/agent-step/**`.
- Add a Rust runtime replay test only after the fixture exists:
  `crates/runx-runtime/tests/external/aster_agent_step.rs`.
- Use `runtime_http.rs` plus the current public hosted HTTP re-exports under
  `registry::http` and `execution::target_runner` as the locally verified
  hosted transport boundary for OSS-side runtime-to-host interaction.
- Defer cloud package binding details until the cloud tree is available.
- Ensure Aster-run issue-to-PR and post-merge paths use
  `runx-target-repo-runners` and `runx-post-merge-closure-observer` when those
  contracts exist, with final state represented as sealed closure/proof
  receipts.

## Scope

In scope:

- OSS-local plan for Aster contract preservation.
- External Aster runtime fixture definition.
- Hosted boundary notes grounded in `runtime_http.rs` and the current public
  hosted HTTP re-exports.
- Dependency sequencing for target-runner and post-merge observer flows.

Out of scope:

- Editing or verifying `cloud/**` paths in this checkout.
- Implementing the cloud binding shim.
- Aster UI, feed curation, selector product behavior, or brand work.
- Legacy/compat execution artifact shapes.

## Dependencies

- `runx-contract-spine-hard-cutover`.
- `rust-runtime-skeleton`.
- `rust-runtime-skill-execution`.
- `rust-approval-gate-parity` for any hosted approval gates that Aster consumes.
- `rust-runtime-receipt-path-discovery`,
  `rust-receipt-tree-resolution`, and `rust-receipt-proof-verification`.
- `runx-operational-policy-config` for policy/admin readback.
- `runx-target-repo-runners` for Aster-scheduled source-to-target PR flows.
- `runx-post-merge-closure-observer` for final closure/proof observation and
  source-thread updates.
- `ts-extension-survivorship-boundary` for the rule that TypeScript may survive
  as cloud/product/helper code over stable protocols but not as trusted local
  runtime execution.
- `external-adapter-plugin-protocol-v1` for any Aster or cloud custom
  adapter/plugin boundary that needs no-Rust-required userland code.
- `embedded-sdk-migration-story` for embedded SDK and cloud runtime-local
  consumer disposition.
- A future cloud-tree binding pass that can inspect the real `cloud/**`
  implementation.

## Acceptance Criteria

- [x] Existing Aster contract fixture coverage remains green for
  `fixtures/contracts/aster-control/public-feed-proof.json`.
- [x] The runtime external fixture
  `fixtures/external/aster/agent-step/**` exists before any Aster runtime
  replay test is claimed.
- [x] The replay test
  `crates/runx-runtime/tests/external/aster_agent_step.rs` is added only after
  the external fixture exists.
- [x] The OSS-hosted boundary is documented against
  `crates/runx-runtime/src/runtime_http.rs` or a reviewed replacement.
- [x] Cloud binding details are marked deferred until `cloud/**` is available
  locally; no acceptance depends on absent cloud paths.
- [x] Cloud `agent-runner` binding mode is not claimed as settled by this draft.
- [x] Aster contract and runtime artifacts use harness receipt closure and
  `proof.verification`, not retired peer terminal artifacts or legacy
  outcome/effect packet fields.
- [x] Aster final publication and issue-to-PR completion are explicitly
  deferred to the reusable observer/runner specs and are not claimed by this
  draft.

## Deferred Follow-Up Gates

- A future cloud-tree binding spec must settle how `cloud/agent-runner` consumes
  Rust runtime execution without reviving `@runxhq/runtime-local`.
- Aster final publication and issue-to-PR completion, once implemented, must use
  sealed harness receipt closure/proof through `runx-target-repo-runners` and
  `runx-post-merge-closure-observer` rather than Aster-only terminal packets.

## Phase 1: Ratify OSS Preservation Snapshot

Status: active
Dependencies: none

Objective: Re-run local contract, fixture, replay, and dogfood evidence and

Changes:
- [x] Re-run validation commands from the OSS repo root.
- [x] Record changed validation evidence.
- [x] Keep cloud binding status deferred and point next work to a dedicated cloud-tree binding spec.

Acceptance:
- none

## Validation Commands

Current local discovery/guard commands:

```sh
test ! -d cloud
test -f crates/runx-runtime/src/runtime_http.rs
test -f crates/runx-contracts/src/aster.rs
test -f fixtures/contracts/aster-control/public-feed-proof.json
test -f fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml
test -f crates/runx-runtime/tests/external/aster_agent_step.rs
cargo test --manifest-path crates/Cargo.toml -p runx-contracts aster
cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external aster_agent_step
! rg -n "runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:" crates/runx-contracts/src/aster.rs fixtures/contracts/aster-control
! rg -n "runId|receiptId|issue_to_pr_outcome|verification[_-]report|verificationReport|target[_-]?effect|\"effect\"\\s*:|\"outcome\"\\s*:|/Users/kam" fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml
git diff --check -- .scafld/specs/drafts/rust-aster-runtime-cutover.md .scafld/specs/active/rust-aster-runtime-cutover.md
```

Latest local validation:

```sh
cargo build --manifest-path crates/Cargo.toml -p runx-cli
# passed

cargo test --manifest-path crates/Cargo.toml -p runx-contracts aster
# passed: aster_control_fixture_roundtrips

cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external aster_agent_step
# passed: 2 tests

ruby -ryaml -e 'YAML.load_file("fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml"); puts "yaml ok"'
# passed: yaml ok

! rg -n "runId|receiptId|issue_to_pr_outcome|verification[_-]report|verificationReport|target[_-]?effect|\"effect\"\\s*:|\"outcome\"\\s*:|/Users/kam" fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml
# passed: no matches
```

2026-05-21 Aster dogfood validation:

```sh
cd /Users/kam/dev/runx/aster
npm run check
# passed: aster check passed

node --test scripts/runx-agent-bridge.test.mjs scripts/run-issue-triage-workers.test.mjs scripts/promote-aster-state.test.mjs
# passed: 28 tests

/Users/kam/dev/runx/runx/oss/crates/target/debug/runx --version
# passed: runx-cli 0.1.0

RUNX_ROOT=/Users/kam/dev/runx/runx/oss ARTIFACT_DIR="$(mktemp -d /tmp/aster-proving-ground.XXXXXX)" PROVING_GROUND_PROFILE=minimal bash scripts/proving-ground.sh
node scripts/summarize-proving-ground.mjs "$ARTIFACT_DIR"
# passed: echo-skill and sequential-graph produced sealed runx.harness_receipt.v1 receipts
```

2026-05-22 Aster dogfood refresh:

```sh
cd /Users/kam/dev/runx/aster
npm run check
# passed: aster check passed

node --test scripts/runx-agent-bridge.test.mjs scripts/run-issue-triage-workers.test.mjs scripts/promote-aster-state.test.mjs
# passed: 31 tests

/Users/kam/dev/runx/runx/oss/crates/target/debug/runx --version
# passed: runx-cli 0.1.0

RUNX_ROOT=/Users/kam/dev/runx/runx/oss ARTIFACT_DIR="$(mktemp -d /tmp/aster-proving-ground.XXXXXX)" PROVING_GROUND_PROFILE=minimal bash scripts/proving-ground.sh
node scripts/summarize-proving-ground.mjs "$ARTIFACT_DIR"
# passed: echo-skill and sequential-graph produced sealed runx.harness_receipt.v1 receipts

git diff --check
# passed
```

## Rollback And Repair

- If cloud binding assumptions are wrong, repair the cloud binding spec after
  inspecting a checkout that contains `cloud/**`; do not encode guessed cloud
  paths in this OSS-only spec.
- If cloud or Aster integration needs custom adapter/plugin code, route it
  through `external-adapter-plugin-protocol-v1` or keep the binding blocked; do
  not revive `@runxhq/runtime-local` or force provider code into Rust.
- If the external runtime fixture is missing, keep Aster cutover blocked rather
  than treating the Aster control contract fixture as runtime execution proof.
- If a future binding bypasses the current hosted HTTP re-exports or needs
  direct access to `runtime_http.rs` internals, require explicit reviewed
  replacement or visibility-widening boundary in the cloud-tree binding spec.
- If retired artifact fields appear in Aster fixtures or runtime output, repair
  the producer and expected sealed receipts. Do not add compatibility shims.

## Open Questions

- Which concrete cloud binding mode wins once the cloud tree is available:
  hosted HTTP, subprocess JSON over `runx-cli`, `runx-runtime-service`/FFI, the
  external adapter/plugin protocol, or another reviewed stable boundary.
- Whether `cloud/packages/agent-runner/**` needs the external adapter/plugin
  protocol for hosted custom adapter behavior or can stay on a hosted HTTP
  boundary with generated contracts.
- Where hosted approval routing lives in the cloud tree after the Aster v1 reset
  work is available for inspection.
- Whether Aster needs a dedicated runtime fixture generator or can share the
  generic hosted fixture machinery once that exists.

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-21T15:37:09Z
Ended: 2026-05-21T15:37:09Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Local fact base verifies: `runtime_http.rs`, `aster.rs`, the new agent-step fixture, the replay test, and the absent `cloud/` tree are all as the draft claims, and the cited cargo test was found wired through `tests/external.rs`. The spec has real coherence gaps that should be answered before approval: it calls itself a "cutover" yet the in-scope deliverables are preservation/documentation only, with the actual cloud binding deferred to an unscheduled future pass; there is no "Planned Phases" section for a `size: large, risk_level: high` spec, so `scafld build` has no phases to open; the invariant pointing future cloud bindings at "public connect/registry re-exports" of `runtime_http.rs` is misaligned with the code (the module is `mod`, not `pub mod`, and only `legacy Connect wrapper` wraps it publicly); and the one open acceptance row depends on two sibling specs that are still drafts themselves. Treat these as design/coherence revisions rather than safety blockers.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/runtime_http.rs:1
  - Result: passed
  - Evidence: Verified each declared path: crates/runx-runtime/src/runtime_http.rs (exists; defines HostedHttpClient, HostedTransport, ReqwestHttpTransport with redirect::Policy::none() and bounded request/connect timeouts under feature `async-http`); crates/runx-contracts/src/aster.rs (exists and is exported from crates/runx-contracts/src/lib.rs lines 6,41); fixtures/contracts/aster-control/public-feed-proof.json (exists); fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml (exists, YAML body matches the test expectations including `accepted_command`, sealed receipt id `hrn_rcpt_aster_issue_triage_14`); crates/runx-runtime/tests/external/aster_agent_step.rs (exists, wired via crates/runx-runtime/tests/external.rs line 1-2). The negative claim `no cloud/` was also confirmed (Glob on cloud/** returned no files). One discrepancy worth surfacing in the design challenge: `runtime_http` is declared `mod runtime_http;` in crates/runx-runtime/src/lib.rs:26, not `pub mod`, so `HostedHttpClient` is not directly re-exported at the runtime crate's public surface — only `legacy Connect wrapper` (lib.rs:61) wraps it.
- command audit
  - Grounded in: spec_gap:validation_commands
  - Result: passed
  - Evidence: The validation commands resolve against real files. `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external aster_agent_step` is the documented invocation and the test it targets exists at crates/runx-runtime/tests/external/aster_agent_step.rs with two #[test] functions matching the `aster_agent_step` filter. `cargo test … -p runx-contracts aster` will pick up the aster suite (aster module present at crates/runx-contracts/src/aster.rs and the cited fixture roundtrip test file `crates/runx-contracts/tests/aster_control_fixtures.rs` is referenced from the spec Context block). The Aster-checkout commands (`npm run check`, `node --test scripts/runx-agent-bridge.test.mjs …`, `bash scripts/proving-ground.sh`, `node scripts/summarize-proving-ground.mjs …`) run in `/Users/kam/dev/runx/aster`, a sibling repo outside this OSS checkout, so they cannot be re-verified from here; the spec already calls that out and the dogfood log lines are dated 2026-05-21 and 2026-05-22, matching `updated:` on the spec. The `git diff --check` line in Validation Commands cites only this draft file, which is consistent with harden mode being read-only.
- scope/migration audit
  - Grounded in: spec_gap:scope
  - Result: failed
  - Evidence: The title is `Rust aster runtime cutover` and the Dependencies list (lines 162–180) is shaped like a cutover dependency tree, but the Scope section (lines 144–158) limits in-scope work to (a) OSS-local plan for contract preservation, (b) the external fixture definition (already on disk), (c) boundary notes grounded in runtime_http.rs, and (d) dependency sequencing notes. The actual cutover edges — cloud `agent-runner` binding, hosted approval routing, UI/selector/feed behavior, and `@runxhq/runtime-local` sunset — are explicitly out-of-scope or deferred (Open Questions lines 296–306). The Invariants block goes further and says the cloud binding will be settled by a future, unscheduled pass. Approving this spec therefore does not authorize a cutover; it authorizes a preservation snapshot. This is a coherence problem rather than a safety problem, but it merits either a rename (e.g. `rust-aster-runtime-preservation`) or an explicit `cutover_status: deferred` field so downstream readers do not assume the cutover has been planned.
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: failed
  - Evidence: Seven of eight acceptance criteria are already marked `[x]` in the draft (lines 184–199), and each refers to artifacts that already exist in the tree (verified above). The only unchecked row is `[ ] Aster final publication and issue-to-PR completion … use sealed harness receipt closure/proof through the reusable observer/runner specs`, which depends on `runx-target-repo-runners` and `runx-post-merge-closure-observer`. Both of those specs are still in `.scafld/specs/drafts/` with `status: draft` and `harden_status: in_progress` (verified by reading lines 1–10 of each). That means the one unchecked acceptance criterion cannot be made checkable by any phase in this spec — it can only be satisfied after two sibling drafts are approved and implemented. Combined with the lack of a `## Planned Phases` section (the spec has Current State, Summary, Context, Invariants, Objectives, Scope, Dependencies, Acceptance Criteria, Validation Commands, Rollback And Repair, Open Questions, Harden Rounds — but no phases), there is nothing concrete left for `scafld build` to open after approval. Either the open row should be split into a separate follow-up spec or this draft should add explicit phases (even if the only phase is `verify`/`ratify` on what already exists).
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback And Repair (lines 280–294) is credible for the scope this spec actually covers: it prescribes repairing the cloud binding spec when a cloud-tree checkout is available rather than encoding guessed cloud paths, routes custom adapter/plugin needs through `external-adapter-plugin-protocol-v1` (verified present at `.scafld/specs/active/external-adapter-plugin-protocol-v1.md`), keeps cutover blocked if the external fixture is missing, requires an explicit reviewed replacement boundary if `runtime_http.rs` is bypassed, and forbids compatibility shims. None of these are destructive or hard to reverse — they are documentation/spec-level instructions, consistent with the draft being preservation-shaped.
- design challenge
  - Grounded in: code:crates/runx-runtime/src/lib.rs:26
  - Result: failed
  - Evidence: Three architectural concerns surfaced. (1) The invariant on lines 117–120 says any future cloud binding should `use this boundary through the public connect/registry re-exports or explicitly replace it`, but `runtime_http` is declared `mod runtime_http;` (lib.rs:26), not `pub mod`, and `HostedHttpClient` is not re-exported in the `pub use` block (lib.rs:52–113). The only public consumer is `legacy Connect wrapper` (crates/runx-runtime/src/connect/client.rs:9 imports it; lib.rs:61 re-exports `legacy Connect wrapper`). A future cloud binding that needs raw hosted-HTTP semantics would either have to bend itself into Connect semantics or change the module visibility, so the invariant either needs to name `legacy Connect wrapper` as the canonical boundary or call out that a visibility change is part of any binding pass. (2) Calling this a `cutover` while deferring the binding decision to a future, unscheduled pass risks freezing premature constraints (e.g., the invariant that the cloud `agent-runner` `must not preserve a runtime-local fallback`, line 108) without ever proving them with a concrete cloud-tree inspection. That looks more like a bandaid than the right architectural move — consider re-scoping to a preservation spec and leaving the cutover invariants to the actual binding spec. (3) Acceptance row 8 is structurally coupled to two sibling drafts; if either of those changes shape, this spec quietly goes stale. Worth either dropping that row from here or adding a `blocks_on:` relation.

Issues:
- [high/advisory] `harden-1` scope_coherence - Title says `cutover`, scope only covers preservation/documentation.
  - Status: open
  - Grounded in: spec_gap:scope
  - Evidence: Lines 144–158 list only contract-preservation, fixture definition, boundary notes, and dependency sequencing as in-scope. The actual cutover edges (cloud `agent-runner` binding, hosted approval routing, runtime-local sunset) are explicitly deferred (Invariants lines 102–109; Open Questions lines 296–306) to a future, unscheduled cloud-tree pass.
  - Recommendation: Either rename this draft to reflect what it does (e.g., `rust-aster-runtime-preservation`) or add an explicit `cutover_status: deferred` block at the top of the spec and move the cutover-shaped invariants into the future binding spec so they are not approved in absentia.
  - Question: Is this spec the cutover, or the preservation snapshot that precedes the cutover — and if the latter, can it be renamed before approval?
  - Recommended answer: Rename to a preservation spec and split the cloud-binding cutover into a dedicated future spec triggered when `cloud/**` is locally available; that keeps each spec's acceptance verifiable from its own checkout.
  - If unanswered: Default to keeping the current title but add `cutover_status: deferred` and a one-line scope disclaimer so downstream readers do not assume the cutover decision has been made.
- [high/blocks approval] `harden-2` phase_plan_missing - No `## Planned Phases` section on a `size: large, risk_level: high` spec.
  - Status: open
  - Grounded in: spec_gap:phases
  - Evidence: The draft contains Current State, Summary, Context, Invariants, Objectives, Scope, Dependencies, Acceptance Criteria, Validation Commands, Rollback And Repair, Open Questions, and Harden Rounds — but no phase plan. The CLAUDE contract calls for `scope, ingest, model, materialize, evaluate, verify, ratify` semantics on complex skills, and `scafld build` opens phases one at a time. With seven of eight acceptance criteria already marked `[x]` and the eighth waiting on two other drafts, there is nothing for `scafld build` to materially open after approval.
  - Recommendation: Add an explicit `## Planned Phases` section. Given that most artifacts already exist, the realistic phase list is likely `verify` (re-run the validation commands against current HEAD) and `ratify` (lock the preservation snapshot). If the operator wants this to remain a planning-only spec, switch its size/risk down or convert it into a `note`-shaped doc rather than a buildable task.
  - Question: Which phases should `scafld build` open for this draft once approved — or should it move to a smaller, doc-shaped artifact?
  - Recommended answer: Add `verify` and `ratify` phases pinned to the existing validation commands and the dogfood re-run; defer all binding-related phases to the future cloud-tree spec.
  - If unanswered: Default to a two-phase plan (verify, ratify) keyed off the already-passing cargo and dogfood commands.
- [medium/advisory] `harden-3` invariant_vs_code_mismatch - Invariant cites `public connect/registry re-exports` of `runtime_http.rs`, but the module is private.
  - Status: open
  - Grounded in: code:crates/runx-runtime/src/lib.rs:26
  - Evidence: Lines 117–120 say future cloud bindings should `use this boundary through the public connect/registry re-exports or explicitly replace it`. However `runtime_http` is `mod runtime_http;` at lib.rs:26 (not `pub mod`), and `HostedHttpClient`/`HostedTransport` are not in the `pub use` block (lib.rs:52–113). Only `legacy Connect wrapper` wraps it (connect/client.rs:9, re-exported at lib.rs:61).
  - Recommendation: Name `legacy Connect wrapper` as the canonical hosted boundary in the invariant, or note that a future binding may need to widen `runtime_http` visibility as a reviewed change. Avoid pointing at a public API surface that does not exist.
  - Question: Should the invariant be reworded to point at `legacy Connect wrapper`, or is widening `runtime_http`'s visibility considered part of the future binding scope?
  - Recommended answer: Reword to name `legacy Connect wrapper` as today's public boundary; treat any need to expose `HostedHttpClient` directly as an explicit reviewed change captured in the future cloud-binding spec.
  - If unanswered: Default to renaming the invariant target to `legacy Connect wrapper` and noting that direct `HostedHttpClient` re-exports are a separate review item.
- [medium/advisory] `harden-4` acceptance_coupling - Open acceptance row 8 depends on two sibling drafts still in harden.
  - Status: open
  - Grounded in: spec_gap:acceptance
  - Evidence: Acceptance line 200–202 requires Aster issue-to-PR/post-merge completion to flow through `runx-target-repo-runners` and `runx-post-merge-closure-observer`. Both are in `.scafld/specs/drafts/` with `status: draft` and `harden_status: in_progress` (verified by reading lines 1–10 of each). Nothing in this spec's own scope can make that row checkable.
  - Recommendation: Either drop that row from this draft and capture it as a `follow_up` block, or add an explicit `blocks_on:` field listing the two sibling drafts so reviewers can see the coupling without inferring it.
  - Question: Should acceptance row 8 stay here as an aspirational gate, or move into a follow-up spec keyed to when the sibling drafts archive?
  - Recommended answer: Move it to a follow-up note; keep this spec's acceptance closed against artifacts that exist in this checkout only.
  - If unanswered: Default to leaving the row but adding `blocks_on: [runx-target-repo-runners, runx-post-merge-closure-observer]` so the coupling is explicit.

### round-2

Status: needs_revision
Started: 2026-05-21T15:49:30Z
Ended: 2026-05-21T15:49:30Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 of the draft addresses the structural concerns from round-1: it now carries an explicit `Cutover status: **deferred**` block (mitigates the title-vs-scope mismatch), names `legacy Connect wrapper` rather than imaginary public re-exports of `runtime_http.rs` (matches what's actually exported at lib.rs:61), moves the sibling-spec coupling into a `Deferred Follow-Up Gates` block, and adds Phase 1 `Ratify OSS Preservation Snapshot`. Local artifact verification passes: `runtime_http.rs` is still a private `mod` (lib.rs:26) wrapped publicly only via `legacy Connect wrapper` (connect/client.rs:9, lib.rs:61); the agent-step fixture and `aster_agent_step.rs` test are wired through `tests/external.rs`; `cloud/**` is genuinely absent; both sibling specs (`runx-target-repo-runners`, `runx-post-merge-closure-observer`) are still drafts as the spec acknowledges. One concrete defect remains: Phase 1 acceptance command `p1c` hardcodes the path `.scafld/specs/drafts/rust-aster-runtime-cutover.md`, but `scafld approve` moves the file to `.scafld/specs/active/...` (confirmed pattern in `external-adapter-runtime-wiring-v1.md`), so `rg` over a non-existent file will return non-zero and the phase cannot reach exit_code_zero as written. This is a one-line fix but it makes the only buildable phase non-executable post-approval, so the draft should not be approved until the path is corrected (or made lifecycle-agnostic). Title coherence remains an open advisory concern but is not a safety blocker now that the deferral is explicit.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/lib.rs:26
  - Result: passed
  - Evidence: Re-verified declared local paths: crates/runx-runtime/src/runtime_http.rs (present, still declared as private `mod runtime_http;` at lib.rs:26 — not `pub mod`); crates/runx-contracts/src/aster.rs (present); fixtures/contracts/aster-control/public-feed-proof.json (present); fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml (present, schema runx.skill_run.v1, receipt id hrn_rcpt_aster_issue_triage_14); crates/runx-runtime/tests/external/aster_agent_step.rs (present, wired via tests/external.rs lines 1-2). legacy Connect wrapper is publicly re-exported at lib.rs:61 and wraps HostedHttpClient at connect/client.rs:9 — consistent with the revised invariant. Glob on cloud/** returned no files, confirming the negative claim. Sibling spec frontmatter for runx-target-repo-runners and runx-post-merge-closure-observer confirms both are still status: draft, harden_status: in_progress.
- command audit
  - Grounded in: spec_gap:phases.acceptance
  - Result: failed
  - Evidence: Phase 1 acceptance commands p1a (`scafld validate ... --json`) and p1b (`cargo test ... aster && cargo test ... external aster_agent_step`) resolve against real targets. However p1c is `test ! -d cloud && rg -n "Cutover status: \\*\\*deferred\\*\\*" .scafld/specs/drafts/rust-aster-runtime-cutover.md`. The scafld lifecycle moves spec files from `.scafld/specs/drafts/` to `.scafld/specs/active/` on approve (pattern confirmed in `.scafld/specs/active/external-adapter-runtime-wiring-v1.md` which references its own active/ path post-approval). When `scafld build` opens Phase 1 after approval, the spec will live at `.scafld/specs/active/rust-aster-runtime-cutover.md`; `rg` over the drafts/ path will not find the file, exit non-zero, and the && chain will fail. The phase therefore cannot achieve `exit_code_zero` as written.
- scope/migration audit
  - Grounded in: spec_gap:scope
  - Result: passed
  - Evidence: Round-2 added explicit `Cutover status: **deferred**` (Summary lines 73-76) plus an `Invariants` clause stating cloud binding is deferred until a checkout with `cloud/**` is available (lines 107-109). Scope (lines 150-165) restricts in-scope work to OSS-local preservation, fixture definition, hosted-boundary notes against `runtime_http.rs`/`legacy Connect wrapper`, and dependency sequencing — matching what the draft actually delivers. Out-of-scope explicitly excludes editing/verifying `cloud/**`, implementing the cloud shim, and Aster UI/feed work. The title `Rust aster runtime cutover` still reads as a cutover spec rather than the preservation snapshot it really is, but the deferral disclaimer is now prominent enough that downstream readers will not be misled. Rename is an advisory improvement, no longer a coherence block.
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: The top-level Acceptance Criteria (lines 191-209) are all pre-marked `[x]` because each describes state that already exists in this checkout (contract fixture green, external fixture present, replay test wired, etc.). Item 8 from round-1 — which previously coupled this spec's acceptance to sibling drafts — has been reshaped into `Deferred Follow-Up Gates` (lines 211-217) so the open coupling no longer blocks this spec's acceptance. The only outstanding work is Phase 1, whose Changes/Acceptance rows are unchecked `[ ]` (lines 229-244) and describe rerun + ratify work appropriate for a buildable phase. Acceptance/phase timing is internally consistent; the residual problem is the p1c path bug captured in command audit, not the timing structure.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback And Repair (lines 322-336) remains credible for a preservation spec: it tells future authors to fix cloud binding mistakes in a future cloud-tree spec rather than encoding guessed paths now, routes custom adapter/plugin needs through `external-adapter-plugin-protocol-v1` (verified present at `.scafld/specs/active/external-adapter-plugin-protocol-v1.md`), keeps cutover blocked if the external fixture is missing, requires an explicit reviewed widening if `runtime_http` internals are ever exposed, and forbids compatibility shims for retired packet fields. No destructive operations or shared-state mutations are implied.
- design challenge
  - Grounded in: code:crates/runx-runtime/src/lib.rs:61
  - Result: passed
  - Evidence: Round-1's invariant/code mismatch (claiming `public connect/registry re-exports` of `runtime_http.rs`) is resolved in round-2: lines 122-126 now state `runtime_http.rs` is the internal hosted transport implementation and `legacy Connect wrapper` is today's public wrapper, and that a future cloud binding must either use `legacy Connect wrapper`, widen/replace `runtime_http.rs` in a separately reviewed change, or pick another stable protocol boundary. This matches the code: `mod runtime_http;` at lib.rs:26 is private; `legacy Connect wrapper` is the sole public consumer (connect/client.rs:9) and is re-exported at lib.rs:61. The architectural posture (preservation snapshot with explicit deferral of cloud binding rather than premature freezing) is now coherent. Remaining design concern is purely cosmetic: the spec title `Rust aster runtime cutover` does not match the snapshot it delivers; rename would help, but the explicit `Cutover status: **deferred**` block makes this advisory.

Issues:
- [high/blocks approval] `harden-5` phase_command_path - Phase 1 acceptance command p1c hardcodes the drafts/ path; will fail post-approve once the spec moves to active/.
  - Status: open
  - Grounded in: spec_gap:phases.p1c
  - Evidence: Phase 1 acceptance `p1c` (lines 242-244) is `test ! -d cloud && rg -n "Cutover status: \\*\\*deferred\\*\\*" .scafld/specs/drafts/rust-aster-runtime-cutover.md`. The scafld lifecycle moves spec files from `.scafld/specs/drafts/` to `.scafld/specs/active/` on approve (pattern confirmed by `.scafld/specs/active/external-adapter-runtime-wiring-v1.md:116` and the active/ tree containing peer specs that were once drafts). After approval `scafld build` will run p1c against a path that no longer exists; `rg` will return non-zero and the && chain will fail, so the only buildable phase in this `size: large, risk_level: high` spec cannot reach `exit_code_zero` as written.
  - Recommendation: Rewrite p1c to be lifecycle-agnostic, e.g. `test ! -d cloud && rg -n "Cutover status: \\*\\*deferred\\*\\*" .scafld/specs/drafts/rust-aster-runtime-cutover.md .scafld/specs/active/rust-aster-runtime-cutover.md 2>/dev/null` or use a glob like `.scafld/specs/*/rust-aster-runtime-cutover.md`. Optionally apply the same fix to the `git diff --check` line under Validation Commands (line 261).
  - Question: Should p1c match the spec by task-id in either lifecycle directory rather than the hardcoded drafts/ path?
  - Recommended answer: Yes — replace the hardcoded drafts/ path with a glob that resolves under either drafts/ or active/, so Phase 1 can complete after approve.
  - If unanswered: Default to a lifecycle-agnostic glob (`.scafld/specs/*/rust-aster-runtime-cutover.md`) for both p1c and the equivalent Validation Commands line.
- [medium/advisory] `harden-6` scope_coherence - Title still says `cutover` though the deliverable is now an explicit preservation snapshot.
  - Status: open
  - Grounded in: spec_gap:title
  - Evidence: Frontmatter title remains `Rust aster runtime cutover` (line 12) and Dependencies (lines 167-187) read like a cutover dependency tree, yet round-2 made the deferral explicit (`Cutover status: **deferred**`, lines 73-76) and confined in-scope work to preservation, fixture definition, and boundary notes (lines 150-165). The disclaimer mitigates risk that downstream readers misread the intent, but a rename (e.g. `rust-aster-runtime-preservation-snapshot`) would remove the ambiguity entirely and keep the eventual cutover spec name available.
  - Recommendation: Either rename the task to a preservation-shaped id before approval, or keep the title and add a one-line `Title note: this is the preservation snapshot, not the cutover.` near the top so the deferral is impossible to miss. Both are acceptable; rename is cleaner.
  - Question: Rename to a preservation-shaped task id, or accept the current title with the deferral disclaimer?
  - Recommended answer: Accept current title with the existing `Cutover status: **deferred**` block — rename is optional polish, not a coherence blocker now.
  - If unanswered: Leave the title; the round-2 deferral language is sufficient.
- [low/advisory] `harden-7` phase_plan_density - Single 3-bullet phase for a `size: large, risk_level: high` spec is unusually thin.
  - Status: open
  - Grounded in: spec_gap:phases.size
  - Evidence: The spec carries `size: large` and `risk_level: high` (frontmatter lines 8-9), but Phase 1 (lines 219-244) is the only phase and contains three Changes bullets and three acceptance rows, all rerun/ratify shaped. That's appropriate for a preservation snapshot, but it suggests either the size/risk classification should be downsized (since the actual cutover work is deferred to a future spec) or the spec should add at least a `ratify` step distinct from `verify` to match the canonical phase grammar.
  - Recommendation: Consider lowering `size` to `medium` and `risk_level` to `medium` to reflect what this spec actually does (ratify already-extant artifacts), or split Phase 1 into `verify` (re-run commands) and `ratify` (lock the snapshot + point follow-ups). Either keeps the phase plan honest about scope.
  - Question: Downgrade size/risk to match the preservation deliverable, or split Phase 1 into verify+ratify to make the size honest?
  - Recommended answer: Downgrade size to `medium`, risk to `medium`; the cutover-shaped risk belongs in the future cloud-binding spec.
  - If unanswered: Leave the classification as is; the single-phase plan is acceptable for a ratification snapshot.

### round-3

Status: passed
Started: 2026-05-21T15:59:05Z
Ended: 2026-05-21T15:59:05Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-3 of the draft addresses the only blocker carried over from round-2: phase 1 acceptance `p1c` is now lifecycle-agnostic (`rg ... drafts/... 2>/dev/null || rg ... active/...`), so the only buildable phase can reach `exit_code_zero` after `scafld approve` moves the spec into `active/`. Re-verified against the local tree: `runtime_http` is still `mod runtime_http;` at crates/runx-runtime/src/lib.rs:26 with `legacy Connect wrapper` as its sole public wrapper (lib.rs:61, connect/client.rs:8-11) — matching round-2's reworded invariant. The agent-step fixture exists and the wired replay test (`crates/runx-runtime/tests/external/aster_agent_step.rs`, wired via `tests/external.rs:1-2`) targets it through `cargo test ... --test external aster_agent_step`. `cloud/**` is genuinely absent (Glob returns no files), matching the spec's deferral. Both sibling specs (`runx-target-repo-runners`, `runx-post-merge-closure-observer`) remain `status: draft, harden_status: in_progress` — and the spec correctly moves that coupling out of its own acceptance rows into `Deferred Follow-Up Gates`. Two advisory concerns remain but do not block approval: the title still reads `cutover` though the deliverable is preservation-shaped (mitigated by the prominent `Cutover status: **deferred**` block in Summary); and Phase 1 is a single rerun/ratify phase for a `size: large, risk_level: high` spec (appropriate for a ratification snapshot, but the size/risk classification arguably belongs on the future cloud-binding spec instead). Both are recorded as advisory issues, not approval blockers.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/lib.rs:26
  - Result: passed
  - Evidence: Re-verified all declared local paths: crates/runx-runtime/src/runtime_http.rs is still declared private as `mod runtime_http;` at lib.rs:26 (not `pub mod`); legacy Connect wrapper is the only public wrapper, re-exported at lib.rs:61 and importing HostedHttpClient/HostedTransport at connect/client.rs:8-11. crates/runx-contracts/src/aster.rs exists. fixtures/contracts/aster-control/public-feed-proof.json exists. fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml exists with schema `runx.skill_run.v1` and receipt id hrn_rcpt_aster_issue_triage_14 (matches the test's assertions). crates/runx-runtime/tests/external/aster_agent_step.rs exists and is wired via crates/runx-runtime/tests/external.rs:1-2. Glob on cloud/** returns no files, confirming the deferral premise.
- command audit
  - Grounded in: spec_gap:phases.p1c
  - Result: passed
  - Evidence: Round-3 rewrote p1c (lines 241-244) to `test ! -d cloud && (rg -n "Cutover status: \\*\\*deferred\\*\\*" .scafld/specs/drafts/rust-aster-runtime-cutover.md 2>/dev/null || rg -n "Cutover status: \\*\\*deferred\\*\\*" .scafld/specs/active/rust-aster-runtime-cutover.md)`. The `||` covers the lifecycle move: while the spec lives in drafts/ the first rg succeeds; after `scafld approve` moves it to active/, the first rg fails silently (stderr redirected) and the second rg succeeds, so the && chain exits 0 in both states. p1a (`scafld validate ... --json`) and p1b (cargo test for runx-contracts aster and the external aster_agent_step test) resolve against real targets verified above. The dogfood log lines at the foot of the spec run in /Users/kam/dev/runx/aster (a sibling repo) and cannot be re-checked from this OSS checkout; the spec already calls that out.
- scope/migration audit
  - Grounded in: spec_gap:scope
  - Result: passed
  - Evidence: Scope (lines 150-165) keeps in-scope work to OSS-local Aster contract preservation, the external fixture definition, hosted-boundary notes against runtime_http.rs/legacy Connect wrapper, and dependency sequencing. Out-of-scope explicitly excludes editing/verifying cloud/**, implementing the cloud shim, Aster UI/feed/selector work, and legacy/compat execution shapes. The `Cutover status: **deferred**` block (Summary lines 73-76) and Invariants line 107-109 jointly state that the cloud `agent-runner` binding is open follow-up, not a settled cutover fact. With those disclaimers prominent, downstream readers are not misled by the `cutover` title — though renaming is a clean advisory polish.
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: Top-level Acceptance Criteria (lines 191-209) are all pre-marked `[x]` against artifacts that already exist in this checkout. The earlier coupling to sibling drafts has been moved into `Deferred Follow-Up Gates` (lines 211-217) so it no longer blocks this spec's acceptance. Phase 1 acceptance rows (p1a/p1b/p1c, lines 234-244) are unchecked `[ ]` and describe rerun + ratify work appropriate for a buildable phase. With p1c now lifecycle-agnostic, all phase acceptance rows can reach exit_code_zero post-approval.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback And Repair (lines 322-336) remains credible for a preservation snapshot: it tells future authors to fix cloud binding mistakes in a future cloud-tree spec rather than encoding guessed paths, routes custom adapter/plugin needs through external-adapter-plugin-protocol-v1 (present at .scafld/specs/active/), keeps cutover blocked if the external fixture is missing, requires explicit reviewed visibility-widening if runtime_http internals are ever exposed beyond legacy Connect wrapper, and forbids compatibility shims for retired packet fields. No destructive operations or shared-state mutations are implied; all instructions are spec-level.
- design challenge
  - Grounded in: code:crates/runx-runtime/src/lib.rs:61
  - Result: passed
  - Evidence: Round-2's reworded invariant (lines 122-126) — that runtime_http.rs is internal and legacy Connect wrapper is today's public wrapper, and that future cloud bindings should either use legacy Connect wrapper, widen/replace runtime_http.rs in a separately reviewed change, or pick another stable protocol boundary — matches what's actually in code: `mod runtime_http;` at lib.rs:26 (private), legacy Connect wrapper re-exported at lib.rs:61 (only public consumer at connect/client.rs:8-11). The architectural posture (preservation snapshot with explicit deferral of cloud binding) is coherent: the spec ratifies what's verifiable in this checkout and explicitly punts what isn't. Two residual design-shape concerns are advisory only: (1) the title `Rust aster runtime cutover` lags the preservation deliverable but is mitigated by the prominent deferral block; (2) `size: large, risk_level: high` is somewhat heavy for a one-phase rerun/ratify spec — the cutover-shaped weight may belong on the future cloud-binding spec.

Issues:
- [medium/advisory] `harden-1` scope_coherence - Title still reads `Rust aster runtime cutover` though deliverable is now an explicit preservation snapshot.
  - Status: open
  - Grounded in: spec_gap:title
  - Evidence: Frontmatter title remains `Rust aster runtime cutover` (line 12) and Dependencies (lines 167-187) read like a cutover dependency tree. Round-2 made the deferral explicit (`Cutover status: **deferred**`, lines 73-76) and Scope (lines 150-165) confines in-scope work to preservation, fixture definition, and boundary notes. The deferral block prevents downstream readers being misled, but the residual title/scope drift is real.
  - Recommendation: Either rename to a preservation-shaped task id before approval (e.g. `rust-aster-runtime-preservation-snapshot`, freeing `rust-aster-runtime-cutover` for the actual future binding spec) or keep the title and accept that the explicit deferral block is enough. Rename is cleaner; advisory either way.
  - Question: Rename now to a preservation-shaped task id, or leave the title and let the deferral block do the disambiguation work?
  - Recommended answer: Accept current title; the round-2 deferral block is prominent enough. Reserving `rust-aster-runtime-cutover` for the actual cloud-binding spec would be cleaner, but rename is optional polish, not a correctness blocker.
  - If unanswered: Leave the title as-is; rely on the `Cutover status: **deferred**` Summary block and the `Deferred Follow-Up Gates` section to disambiguate scope.
- [low/advisory] `harden-2` phase_plan_density - Single 3-bullet phase carrying `size: large, risk_level: high` is thin; the heavy classification arguably belongs on the future cloud-binding spec.
  - Status: open
  - Grounded in: spec_gap:phases.size
  - Evidence: Frontmatter has `size: large, risk_level: high` (lines 8-9). Phase 1 (lines 219-244) is the only phase and contains three Changes bullets plus three acceptance rows, all rerun/ratify shaped. That's appropriate for a preservation snapshot, but it suggests the size/risk weight is sized for the deferred cutover work rather than for what this spec actually ratifies.
  - Recommendation: Either downgrade `size` to `medium` and `risk_level` to `medium` so the classification matches what Phase 1 actually does (ratify already-extant artifacts), or split Phase 1 into `verify` (re-run commands) and `ratify` (lock the snapshot, point follow-ups) so the canonical phase grammar shows up.
  - Question: Downgrade size/risk to medium for this preservation snapshot, or split Phase 1 into explicit verify+ratify steps to make the high classification honest?
  - Recommended answer: Downgrade `size` and `risk_level` to `medium`; the cutover-shaped risk belongs on the future cloud-binding spec, which will inspect `cloud/**` and actually commit to a binding mode.
  - If unanswered: Leave the classification and the single phase; this is advisory only and the preservation snapshot is internally consistent.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: command
Output: command.stdout
Summary: Command-provider verification pass. Rechecked that the Aster preservation spec remains grounded in existing OSS artifacts: runtime_http stays private behind registry/target-runner exports, the Aster contract and public-feed fixture exist, the external Aster agent-step fixture matches the runtime replay test, cloud binding remains explicitly deferred because cloud/ is absent, and no retired bridge fields are present in the Aster fixture/test scope. No completion blockers found.

Attack log:
- `Aster external fixture`: verify fixture file exists and is wired through crates/runx-runtime/tests/external.rs -> clean
- `Aster replay assertions`: verify rust-bridge-sealed-skill.yaml fields match aster_agent_step expectations -> clean
- `retired bridge fields`: check fixture/test scope for runId, receiptId, issue_to_pr_outcome, verification_report, and target_effect reintroduction -> clean
- `hosted HTTP boundary`: verify runtime_http remains private and public access is through registry/http and execution/target_runner exports -> clean
- `cloud deferral`: verify cloud/ remains absent and the spec keeps cloud agent-runner binding deferred -> clean
- `ambient drift`: confirm current unrelated MCP/connect/license-boundary drift does not touch Aster contract, fixture, or replay paths -> clean

Findings:
- none

