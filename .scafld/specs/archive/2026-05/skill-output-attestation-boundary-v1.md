---
spec_version: '2.0'
task_id: skill-output-attestation-boundary-v1
created: '2026-05-25T17:51:35+10:00'
updated: '2026-05-26T04:22:13Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# skill-output-attestation-boundary-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-26T04:22:13Z
Review gate: pass

## Summary

Separate what a skill says from what the harness can attest. Skill stdout may
remain a claimed payload, but policy decisions, receipt references, and
authority-sensitive gates must consume supervisor-attested facts or explicitly
typed skill claims that cannot masquerade as facts.

## Scope

In scope:
- `runx-runtime` skill output parsing and receipt sealing.
- Transition gates that currently read skill-produced structured output.
- Receipt reference collection from skill payloads.
- Fixture updates proving skill-asserted references do not become attested
  facts without supervisor evidence.

Out of scope:
- Payment rail proof; already handled by the payment supervisor.
- Changing the skill author subprocess ABI except where an output field is
  reclassified from attested fact to skill claim.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Skill-produced stdout is stored as a claim, not a supervisor fact.
- [x] `dod2` Receipt refs used for proof are supervisor-attested or explicitly
  labeled as skill claims.
- [x] `dod3` Policy/transition gates do not trust arbitrary skill JSON as
  authority-sensitive facts.
- [x] `dod4` Regression fixtures show malicious stdout cannot inject proof refs
  or satisfy a gated fact.

Validation:
- [ ] `v1` skill-run output boundary tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test skill_run`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit 0; 12 passed.
  - Status: passed
  - Evidence: skill-produced stdout remains a claim and cannot satisfy supervisor facts
  - Source event: none
  - Last attempt: 2026-05-26T04:05:00Z
  - Checked at: 2026-05-26T04:05:00Z
- [ ] `v2` receipt-reference trust-boundary tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test receipt_refs`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: exit 0; 2 passed.
  - Status: passed
  - Evidence: malicious stdout cannot inject proof refs or satisfy gated facts
  - Source event: none
  - Last attempt: 2026-05-26T04:05:00Z
  - Checked at: 2026-05-26T04:05:00Z
- [ ] `v3` focused trust-boundary grep review
  - Command: `rg -n "output_object|transition_field_value|collect_payload_refs" crates/runx-runtime/src crates/runx-runtime/tests`
  - Expected kind: `reviewed_output`
  - Timeout seconds: none
  - Result: reviewed output.
  - Status: passed
  - Evidence: `execution/graph.rs::output_object` stores parsed stdout under
    `skill_claim` and supervisor-owned `stdout`/`stderr`/`status` overwrite
    any parsed fields; `execution/runner/execution.rs::transition_field_value`
    allows only supervisor `status` when `skill_claim` is present;
    `receipt_refs.rs` verifies stdout proof-like refs are not promoted while
    supervisor metadata payment proof refs remain typed verification refs.
  - Source event: none
  - Last attempt: 2026-05-26T04:05:00Z
  - Checked at: 2026-05-26T04:05:00Z

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode pass for skill-output-attestation-boundary-v1. All three prior findings are repaired with no new regressions.

(1) Prior critical `workspace_mutation`: resolved. The Workspace Classification reports 46 task-scoped changes since baseline, all listed as "removed X (was M …)" entries — i.e., every previously-dirty path is no longer modified. Ambient drift is 0. Recent commits (`feat: clean cutover runtime trust boundaries`, `feat: harden rust runtime cutover`, `chore: align receipt and credential fixtures`, `fix(runtime): harden receipt proof cutover boundaries`) absorbed the prior dirt cleanly.

(2) Prior high `scope-drift-license-boundary-files`: resolved. `docs/license-boundary.manifest.json`, `docs/licensing-boundary.md`, and `scripts/check-boundaries.mjs` no longer appear as dirty in task_changes; they are part of the baseline-dirty-now-clean set.

(3) Prior low `graph-payload-top-level-skill-claim-hoist`: resolved. `crates/runx-runtime/src/execution/skill_run.rs::graph_payload` (lines 537-577) now only inserts `graph`, `graph_status`, `steps`, and `step_outputs` into the payload — the previous `claim.iter().for_each(|(k,v)| payload.entry(k).or_insert(v))` hoist has been removed. The regression test `native_graph_skill_run_pauses_and_resumes_agent_step` (tests/skill_run.rs:474-477) now asserts `payload` top-level does NOT contain `approved`, `proof_ref`, or `receipt_id` when the malicious skill stdout injects those keys, while still verifying decide.skill_claim.result.summary is preserved (lines 478-480) and step_outputs.decide.skill_claim path still exposes the claim (lines 481-487).

Boundary still holds on all four DoD items: `graph::output_object` (graph.rs:163-186) only inserts `raw`, `skill_claim`, `stdout`, `stderr`, `status`; `transition_field_value` (execution.rs:590-609) returns None if any non-`status` first segment is requested when `skill_claim` is present; `seal::output_refs` (seal.rs:534-549) consumes only `output.metadata`; tests/receipt_refs.rs locks stdout proof-like refs out of receipt verification refs and confirms supervisor metadata payment proof remains typed.

No new regressions found in the runtime trust boundary. No convention violations in the changed runtime code. Workspace was not mutated during review (Read/Grep only).

Attack log:
- `crates/runx-runtime/src/execution/graph.rs::output_object`: Confirm parsed stdout is stored under skill_claim and never extended at top level. -> clean (graph.rs:163-186 inserts only raw, skill_claim, stdout, stderr, status. Supervisor-owned stdout/stderr/status overwrite any parsed top-level keys. Matches cli_tool.rs:331-334 and harness/runner.rs:818-823.)
- `crates/runx-runtime/src/execution/runner/execution.rs::transition_field_value`: Try to read non-status first segments when skill_claim is present. -> clean (execution.rs:590-609 — returns None when run.outputs contains `skill_claim` and the first segment is not `status`. Test native_graph_transition_gate_rejects_skill_claim_as_fact exercises this end-to-end.)
- `crates/runx-runtime/src/receipts/seal.rs::output_refs`: Try to promote stdout claims (rail_proof, verification, signal) into receipt verification/evidence refs. -> clean (seal.rs:534-549 reads only output.metadata — agent_request_id for source_refs and collect_supervisor_metadata_refs for payment supervisor proof/evidence. Stdout is never consulted. tests/receipt_refs.rs:11-42 locks the malicious-stdout case; tests/receipt_refs.rs:46-94 confirms the supervisor metadata path remains typed as Verification.)
- `crates/runx-runtime/src/execution/skill_run.rs::graph_payload`: Reproduce prior low finding: hoist skill_claim keys into top-level graph payload via malicious step stdout. -> clean (skill_run.rs:537-577 now only inserts graph, graph_status, steps, and step_outputs. The prior hoist (claim.iter() into payload.entry) has been removed. Prior low finding is fixed.)
- `crates/runx-runtime/tests/skill_run.rs::native_graph_skill_run_pauses_and_resumes_agent_step`: Verify regression test asserts payload top-level cleanliness, not just step_outputs. -> clean (tests/skill_run.rs:474-477 asserts payload does NOT contain `approved`, `proof_ref`, `receipt_id`; lines 481-487 also assert step_outputs.decide does not contain them at the top level while preserving skill_claim path. Prior low finding's regression gap is closed.)
- `Workspace classification: baseline dirty -> task changes`: Confirm prior critical workspace_mutation paths are no longer present as dirty. -> clean (All 46 task_changes entries are of the form 'removed X (was M …)'. Ambient drift outside task scope is 0. Recent commits (7777466, 4b64b13, 20fe909, 03ff65f) absorbed the prior dirty state.)
- `Workspace scope: docs/license-boundary.manifest.json, docs/licensing-boundary.md, scripts/check-boundaries.mjs`: Confirm prior high scope-drift-license-boundary-files is not still active. -> clean (Each of these paths is listed as 'removed (was M …)' in task_changes — they are no longer modified in the workspace and do not appear as new edits.)
- `Convention Check + Dark Patterns (sweep of changed runtime trust-boundary code)`: Look for races, hardcodes, test logic in production, or convention violations in graph.rs, execution.rs, skill_run.rs, seal.rs, harness/runner.rs. -> clean (No test conditionals in production paths. Supervisor metadata reads use existing typed helpers (string_field, payment_supervisor_proof_from_metadata). transition_field_value is pure and uses iter().rev() for last-write-wins semantics consistent with the rest of execution.rs. No secret/hardcode leakage.)
- `Review-time workspace mutation guard`: Ensure this review pass did not mutate the workspace. -> clean (Only Read/Grep/Glob calls were used during this review.)

Findings:
- [critical/non-blocking] `prior-workspace-mutation` Prior critical workspace_mutation blocker is resolved.
  - Location: `crates/runx-contracts/src/credential_delivery.rs`
  - Evidence: Workspace Classification reports 46 task-scoped changes since baseline, all of the form 'removed X (was M …)' — every previously-dirty file is no longer modified. Ambient drift outside task scope is 0. Recent commits (7777466 feat: clean cutover runtime trust boundaries, 4b64b13 feat: harden rust runtime cutover, 20fe909 chore: align receipt and credential fixtures, 03ff65f fix(runtime): harden receipt proof cutover boundaries) absorbed the prior workspace mutations.
  - Impact: Prior review's read-only contract was honored only retroactively; the diff is now committed and no longer floats outside any task scope.
  - Validation: Re-confirmed by inspecting the Task Changes Since Approval Baseline section and the recent commit log.
- [high/non-blocking] `prior-scope-drift-license-boundary` Prior high scope-drift-license-boundary-files blocker is resolved.
  - Location: `docs/license-boundary.manifest.json`
  - Evidence: Task Changes Since Approval Baseline shows docs/license-boundary.manifest.json, docs/licensing-boundary.md, and scripts/check-boundaries.mjs all as 'removed … (was M …)' — they are no longer dirty in the workspace and do not appear as new modifications.
  - Impact: No conflation of skill-output-attestation work with connect-auth licensing work; task diff is no longer cross-contaminated.
  - Validation: Re-confirmed by reading the Task Changes section against the prior review's listed hashes.
- [low/non-blocking] `prior-graph-payload-top-level-hoist` Prior low graph-payload top-level skill_claim hoist is resolved.
  - Location: `crates/runx-runtime/src/execution/skill_run.rs:537`
  - Evidence: crates/runx-runtime/src/execution/skill_run.rs:537-577 — graph_payload now only inserts `graph`, `graph_status`, `steps`, and `step_outputs`. The prior hoist (`if let Some(JsonValue::Object(claim)) = step.outputs.get("skill_claim") { for (key, value) in claim { payload.entry(key.clone()).or_insert_with(...) } }`) is gone. tests/skill_run.rs:474-477 now explicitly asserts `payload` does NOT contain `approved`, `proof_ref`, or `receipt_id` when the skill emits a malicious answers payload with those keys, while preserving the decide.skill_claim.result.summary check on line 480 and the decide step output assertions on lines 481-487.
  - Impact: Defense-in-depth boundary now extends to the graph aggregation layer; downstream consumers of output.payload cannot read skill-injected `approved`/`proof_ref`/`receipt_id` as if they were supervisor facts.
  - Validation: Re-confirmed by reading skill_run.rs:537-577 and tests/skill_run.rs:444-489.

