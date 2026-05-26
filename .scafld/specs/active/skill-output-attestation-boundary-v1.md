---
spec_version: '2.0'
task_id: skill-output-attestation-boundary-v1
created: '2026-05-25T17:51:35+10:00'
updated: '2026-05-26T04:04:27Z'
status: review
harden_status: not_run
size: medium
risk_level: high
---

# skill-output-attestation-boundary-v1

## Current State

Status: review
Current phase: final
Next: repair
Reason: review gate fail: 3 finding(s), 2 completion blocker(s)
Blockers: none
Allowed follow-up command: `scafld handoff skill-output-attestation-boundary-v1`
Latest runner update: 2026-05-26T04:04:33Z
Review gate: fail

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
Verdict: fail
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode review of skill-output-attestation-boundary-v1. The previously raised low finding on `graph::output_object` is FIXED — graph.rs:163-186 no longer extends parsed skill fields at the top level; it only inserts `raw`, `skill_claim`, `stdout`, `stderr`, `status`. `transition_field_value` (execution.rs:598) still blocks every non-`status` first segment when `skill_claim` is present. Receipt-side `output_refs` (seal.rs:534) still consumes only `output.metadata`. Two issues remain: (1) `graph_payload` (skill_run.rs:573-580) explicitly hoists every `skill_claim` key into the top-level payload object, so the same defense-in-depth class flagged previously now lives in graph payload aggregation — the new regression test never asserts that the malicious `approved`/`proof_ref`/`receipt_id` claims are absent at top-level `payload`, only that they are absent under `step_outputs.decide`. (2) The previous critical workspace_mutation blocker has not been resolved: docs/license-boundary.manifest.json, docs/licensing-boundary.md, and scripts/check-boundaries.mjs remain in task_changes with the identical hash transitions the prior review flagged, even though their content concerns the Connect/Auth MIT licensing boundary and has no relationship to skill output attestation. No regressions to the boundary itself; no convention violations spotted. Workspace was not mutated during this review. Workspace changed during review; review failed closed.

Attack log:
- `crates/runx-runtime/src/execution/graph.rs::output_object`: Verify the prior low finding (top-level extend of parsed skill fields) is fixed by inspecting the function body. -> clean (graph.rs:163-186 now inserts only `raw`, `skill_claim`, `stdout`, `stderr`, `status`. No object.extend(fields) call. Consistent with cli_tool::output_object (adapters/cli_tool.rs:331) and harness::skill_output_object (execution/harness/runner.rs:818).)
- `crates/runx-runtime/src/execution/runner/execution.rs::transition_field_value`: Confirm transition gates still reject skill_claim-derived fields except supervisor `status`. -> clean (execution.rs:598 still guards: if outputs contains `skill_claim` and first segment != "status", returns None. Test native_graph_transition_gate_rejects_skill_claim_as_fact exercises the path end-to-end.)
- `crates/runx-runtime/src/receipts/seal.rs::output_refs`: Confirm receipt verification/evidence refs cannot be populated from skill stdout. -> clean (seal.rs:534-549 reads only output.metadata: agent_request_id for source_refs and collect_supervisor_metadata_refs for payment supervisor proof/evidence. Stdout is never consulted. tests/receipt_refs.rs locks this with two regression tests.)
- `crates/runx-runtime/src/execution/skill_run.rs::graph_payload`: Try to inject `receipt_id`, `proof_ref`, `approved` via skill stdout and see if they appear at top-level payload (the prior reviewer's defense-in-depth class, relocated). -> finding (skill_run.rs:573-577 explicitly hoists every skill_claim key into top-level payload. Regression test only asserts the malicious keys are absent under step_outputs.decide, not at top-level payload, so payload.approved/proof_ref/receipt_id remain reachable from malicious stdout.)
- `crates/runx-runtime/tests/skill_run.rs::native_graph_skill_run_pauses_and_resumes_agent_step`: Check whether the malicious-stdout regression for the graph path asserts top-level payload is clean. -> finding (Test on lines 444-485 asserts !decide.contains_key("approved"/"proof_ref"/"receipt_id") but never makes the same assertions on `payload` itself (line 474). Combined with graph_payload's hoist, the assertion gap means the regression doesn't fully prove dod1 at the top-level payload boundary.)
- `task_changes vs declared scope`: Compare task-scoped changed paths against spec scope (runx-runtime skill output parsing, receipt sealing, transition gates, fixtures). -> finding (docs/license-boundary.manifest.json, docs/licensing-boundary.md, scripts/check-boundaries.mjs are inside task_changes with the same hash transitions the previous review flagged as workspace_mutation. Reading the files shows their content concerns Connect/Auth MIT licensing classification and hosted connect brokerage forbidden-terms scanning — entirely unrelated to skill output attestation. Prior critical blocker not resolved.)
- `review-time workspace mutation guard`: Ensure no workspace changes are made during this review pass. -> clean (Review was read-only (Read/Grep/Glob only). No edits performed.)
- `workspace mutation guard`: compare pre-review and post-review workspace snapshots -> finding (added crates/runx-contracts/src/credential_delivery.rs (M 9a52da354bb44245f62c0f39a358e6f8c731e58b51c1f8082d4c43b945f30ae5), added crates/runx-contracts/src/lib.rs (M b926a95244468571171a74863b04b546e8beb2870114e6f7fe6c7e9b1b9aabe8), added crates/runx-contracts/src/schema_artifacts.rs (M 5b0f0247b2522363c3d756b129853add5d91c0d10ee361bf47b4539cd1951463), added crates/runx-contracts/tests/credential_delivery_fixtures.rs (M 2756b6491de0df8deb164a6ba1ca6e55eb6448c72cabadbebcd6fc98fe011eb3), added crates/runx-contracts/tests/schema_validation.rs (M 605ed7f37e228ab51450f65ebbcb7eecd449ea3e998cd6003ab74a2f64951f37), added crates/runx-contracts/tests/schema_wire_compat.rs (M 647d23971c71c27edc1ea3b728f078d79b21192a201e5e6b98b5e6d83e1531f0), added crates/runx-runtime/src/credentials.rs (M c9f9e9b30910c59fabe1b40d6c236031ad6d74efc0d1310721806842dae84714), added fixtures/contracts/credential-delivery/broker-response.json (D deleted), added fixtures/contracts/credential-delivery/observation.json (M 93b7e855587885d8642e443fbeecbe0c173be95a7e8b0c4ba87a059654c366a3), added fixtures/contracts/credential-delivery/profile.json (M 8a26ebc8f423b0b2920e9e7d47a1374b118dc7ab7e645846d82733d3d82ad5af), added fixtures/contracts/credential-delivery/request.json (M 947719f5e7ab180c14837ca0e1ebf141c82b99cabea07f59ccfe31e493ea7948), added fixtures/contracts/credential-delivery/response.json (?? 59a9520fa7dbda16f9036d972f52e717a120b3945a85e9f8dd28e9271e500b8f), changed packages/contracts/src/index.test.ts (M 65b3f33da6ccdcd16a4a449e257fc129e652179334449663e73a92eeefef7b6d -> M 600f403ed3d35829a5454f97689f58cd4480ed28fb130b6bcd132a82bfacc684), changed packages/contracts/src/index.ts (M 7b269322622fb9d4b107ed15f563324453f261ff970306dfa7fb3441f3b0b1be -> M 4904172bfa8eb5171446aca4bc66e856d3b79b67778aff0f9d96d161648ab7c7), added packages/contracts/src/internal.ts (M 3cf0e5237cb53aa6e7f5802273d450105933228ccfc63ab7cb179e93059a989c), added packages/contracts/src/schemas/credential-delivery.test.ts (M 1e5af486cc110add668867ac71c91b82c6cd13f1b972f54e19b1ed3002b80391), added packages/contracts/src/schemas/credential-delivery.ts (M 6b07949daed4da48fb00b39f802d1e1e8ecae8888a266fdaaf4eb476869103fe), added packages/contracts/src/schemas/credentials.test.ts (M cbdd8c198d1cbd7721bacb77e843c709536f17e48710a63e59e5d8a9637e911e), added packages/core/src/policy/index.test.ts (M 3e5a0773b0d7fc6b8058dd93d42a9a739b2fd1947b9dda7116de88683893721d), added packages/runtime-local/src/runner-local/execute-skill.test.ts (M 143960b9cfe6af58b6abd41f39f7fe8af809510b4448ad68689b2b483fc60585), added packages/runtime-local/src/runner-local/kernel-bridge.test.ts (M d216320637c7775f3852fef9974b3337caae1cac95c876f985c43826f367feed), added scripts/generate-kernel-parity-fixtures.ts (M 21a2ac71007f198065175629c2dc6bcdd1ab4fd772a5b3b71801aec1505b8af8), added tests/executor-control-schema-contract.test.ts (M b91bb8727d8270a39a0f2aec9bda9dcd291084504ecd8daca85d22470024a4e2), added tests/runtime-local-auth-security.test.ts (M 7ae48a253904faf5b55b00baa7ef4b3f925ba9367790519b440933cebc43dbf3))

Findings:
- [high/blocks completion] `scope-drift-license-boundary-files` Three out-of-scope files (license boundary docs + boundary check script) remain in task_changes with the same hash transitions the prior review flagged as a workspace_mutation blocker. The blocker was never resolved.
  - Location: `docs/license-boundary.manifest.json`
  - Evidence: task_changes lists: docs/license-boundary.manifest.json (M fb4a4074... -> M 66dc0993...), docs/licensing-boundary.md (M f48d0dcc... -> M a4d520b0...), scripts/check-boundaries.mjs (M 29c55250... -> M 0f1477ef...). These are the same hash pairs the previous review recorded under the workspace_mutation finding. Reading the files (docs/licensing-boundary.md, docs/license-boundary.manifest.json, scripts/check-boundaries.mjs) confirms their content concerns Connect/Auth MIT licensing classification (`connect-auth-mit-boundary-v1`), hosted connect brokerage forbidden-terms scanning, and crate licensing class manifests. None of them touches skill output parsing, skill_claim, supervisor attestation, receipt refs, transition gates, or any term in the declared scope (runx-runtime skill output parsing, receipt sealing, transition gates, receipt-reference collection, attestation-boundary fixtures).
  - Impact: The spec's declared in-scope list is the contract for what this task may modify; the spec's out-of-scope list also explicitly excludes changes to ABI fields other than fact->claim reclassification. Carrying three unrelated changed files into the review violates scope discipline, conflates this task's diff with the parallel connect-auth licensing work, and means the prior review's critical workspace_mutation finding is still active rather than repaired in verify mode.
  - Validation: Either (a) revert the three files to their approval-baseline state and rerun review, or (b) move the changes to the actual task they belong to (connect-auth-mit-boundary-v1) and rerun review with the workspace clean of unrelated diffs.
- [low/non-blocking] `graph-payload-top-level-skill-claim-hoist` graph_payload still hoists every skill_claim key into the top-level payload object. This is the same defense-in-depth pattern the prior review flagged on graph::output_object, just relocated to graph-output aggregation.
  - Location: `crates/runx-runtime/src/execution/skill_run.rs:573`
  - Evidence: crates/runx-runtime/src/execution/skill_run.rs:573-580:
  if let Some(JsonValue::Object(claim)) = step.outputs.get("skill_claim") {
      for (key, value) in claim {
          payload.entry(key.clone()).or_insert_with(|| value.clone());
      }
  }
  for (key, value) in &step.outputs {
      payload.entry(key.clone()).or_insert_with(|| value.clone());
  }

The regression test crates/runx-runtime/tests/skill_run.rs:444-485 feeds malicious answers `{"approved": true, "proof_ref": "receipt-proof:evil:step-output", "receipt_id": "sha256:evil-step-output", "result": {...}}` and asserts step_outputs.decide does NOT contain `approved`/`proof_ref`/`receipt_id` (lines 481-483), but it never asserts the same for the top-level `payload` object that the test resolves on line 474. Given the hoist on line 573, `payload.approved == true`, `payload.proof_ref == "receipt-proof:evil:step-output"`, and `payload.receipt_id == "sha256:evil-step-output"` are reachable directly from a malicious skill stdout. sealed_output (skill_run.rs:1034) then exposes that payload at output.payload to any downstream consumer.
  - Impact: No active bypass: production consumers of authority-sensitive fields use the supervisor-owned channels (transition_field_value gates on skill_claim, receipt output_refs reads only metadata, output.receipt_id at top level is set from receipt.id on line 1025). Risk is the same defense-in-depth class the prior reviewer flagged on graph::output_object: any new consumer that reads `output.payload.<key>` without recognizing it as a skill claim will silently treat skill-controlled JSON as a supervisor-attested fact, regressing dod1/dod3.
  - Validation: Extend `native_graph_skill_run_pauses_and_resumes_agent_step` (or add a dedicated regression) to assert payload does NOT contain top-level `approved`, `proof_ref`, `receipt_id` when the skill emits them. If the hoist must stay for downstream `payload.result` access, restrict it to a declared safe-list rather than blanket `claim.iter()`.
- [critical/blocks completion] `workspace_mutation` Workspace changed during review.
  - Location: `crates/runx-contracts/src/credential_delivery.rs (M 9a52da354bb44245f62c0f39a358e6f8c731e58b51c1f8082d4c43b945f30ae5)`
  - Evidence: workspace changed during review: added crates/runx-contracts/src/credential_delivery.rs (M 9a52da354bb44245f62c0f39a358e6f8c731e58b51c1f8082d4c43b945f30ae5), added crates/runx-contracts/src/lib.rs (M b926a95244468571171a74863b04b546e8beb2870114e6f7fe6c7e9b1b9aabe8), added crates/runx-contracts/src/schema_artifacts.rs (M 5b0f0247b2522363c3d756b129853add5d91c0d10ee361bf47b4539cd1951463), added crates/runx-contracts/tests/credential_delivery_fixtures.rs (M 2756b6491de0df8deb164a6ba1ca6e55eb6448c72cabadbebcd6fc98fe011eb3), added crates/runx-contracts/tests/schema_validation.rs (M 605ed7f37e228ab51450f65ebbcb7eecd449ea3e998cd6003ab74a2f64951f37), added crates/runx-contracts/tests/schema_wire_compat.rs (M 647d23971c71c27edc1ea3b728f078d79b21192a201e5e6b98b5e6d83e1531f0), added crates/runx-runtime/src/credentials.rs (M c9f9e9b30910c59fabe1b40d6c236031ad6d74efc0d1310721806842dae84714), added fixtures/contracts/credential-delivery/broker-response.json (D deleted), added fixtures/contracts/credential-delivery/observation.json (M 93b7e855587885d8642e443fbeecbe0c173be95a7e8b0c4ba87a059654c366a3), added fixtures/contracts/credential-delivery/profile.json (M 8a26ebc8f423b0b2920e9e7d47a1374b118dc7ab7e645846d82733d3d82ad5af), added fixtures/contracts/credential-delivery/request.json (M 947719f5e7ab180c14837ca0e1ebf141c82b99cabea07f59ccfe31e493ea7948), added fixtures/contracts/credential-delivery/response.json (?? 59a9520fa7dbda16f9036d972f52e717a120b3945a85e9f8dd28e9271e500b8f), changed packages/contracts/src/index.test.ts (M 65b3f33da6ccdcd16a4a449e257fc129e652179334449663e73a92eeefef7b6d -> M 600f403ed3d35829a5454f97689f58cd4480ed28fb130b6bcd132a82bfacc684), changed packages/contracts/src/index.ts (M 7b269322622fb9d4b107ed15f563324453f261ff970306dfa7fb3441f3b0b1be -> M 4904172bfa8eb5171446aca4bc66e856d3b79b67778aff0f9d96d161648ab7c7), added packages/contracts/src/internal.ts (M 3cf0e5237cb53aa6e7f5802273d450105933228ccfc63ab7cb179e93059a989c), added packages/contracts/src/schemas/credential-delivery.test.ts (M 1e5af486cc110add668867ac71c91b82c6cd13f1b972f54e19b1ed3002b80391), added packages/contracts/src/schemas/credential-delivery.ts (M 6b07949daed4da48fb00b39f802d1e1e8ecae8888a266fdaaf4eb476869103fe), added packages/contracts/src/schemas/credentials.test.ts (M cbdd8c198d1cbd7721bacb77e843c709536f17e48710a63e59e5d8a9637e911e), added packages/core/src/policy/index.test.ts (M 3e5a0773b0d7fc6b8058dd93d42a9a739b2fd1947b9dda7116de88683893721d), added packages/runtime-local/src/runner-local/execute-skill.test.ts (M 143960b9cfe6af58b6abd41f39f7fe8af809510b4448ad68689b2b483fc60585), added packages/runtime-local/src/runner-local/kernel-bridge.test.ts (M d216320637c7775f3852fef9974b3337caae1cac95c876f985c43826f367feed), added scripts/generate-kernel-parity-fixtures.ts (M 21a2ac71007f198065175629c2dc6bcdd1ab4fd772a5b3b71801aec1505b8af8), added tests/executor-control-schema-contract.test.ts (M b91bb8727d8270a39a0f2aec9bda9dcd291084504ecd8daca85d22470024a4e2), added tests/runtime-local-auth-security.test.ts (M 7ae48a253904faf5b55b00baa7ef4b3f925ba9367790519b440933cebc43dbf3)
  - Impact: The review provider changed the workspace while acting as a read-only reviewer, so its verdict is not trustworthy.
  - Validation: Restore the workspace to the expected state, ensure the provider is read-only, then rerun scafld review.

