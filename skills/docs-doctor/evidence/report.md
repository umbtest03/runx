# Delivery Report: #77 docs-doctor

## Summary
- **Bounty:** #77 - runx skill: docs-doctor
- **Claimant:** @umbtest03 (Claim ID: 38f2263d-0511-4168-a0b7-37872c0dd9ba)
- **Date:** 2026-07-05
- **runx CLI:** runx-cli 0.6.14

## Artifacts

| # | Artifact | URL/Ref |
|---|----------|---------|
| 1 | public_url | https://runx.ai/x/umbtest03/docs-doctor |
| 2 | source_url | https://github.com/umbtest03/runx/tree/4c2603c3/skills/docs-doctor |
| 3 | pr_url | https://github.com/runxhq/runx/pull/231 |
| 4 | x_yaml | https://raw.githubusercontent.com/umbtest03/runx/4c2603c3/skills/docs-doctor/X.yaml |
| 5 | skill_md | https://raw.githubusercontent.com/umbtest03/runx/4c2603c3/skills/docs-doctor/SKILL.md |
| 6 | evidence_json | https://raw.githubusercontent.com/umbtest03/runx/4c2603c3/skills/docs-doctor/evidence/evidence.json |
| 7 | verification_json | https://raw.githubusercontent.com/umbtest03/runx/4c2603c3/skills/docs-doctor/evidence/verification.json |
| 8 | receipt_ref | sha256:35d95442a83fb2e997efc24094fac349bdaf1f69918aacbdfecb638410f78d88 |
| 9 | report | https://raw.githubusercontent.com/umbtest03/runx/4c2603c3/skills/docs-doctor/evidence/report.md |

## Skill Description
Audits documentation against a product surface and user task matrix. Identifies undocumented commands, endpoints, schemas, and missing or stale documentation with fix proposals. Refuses when all commands are fully documented.

## Verification Results
- **Local harness (WSL):** PASSED (2/2 cases, 0 assertion errors)
- **Hosted harness (api.runx.ai):** PASSED (server-side validation)
- **Registry publish:** umbtest03/docs-doctor@sha-7dbb7270bec3
- **PR:** https://github.com/runxhq/runx/pull/231
- **Dogfood run:** 4 findings (3 undocumented commands, 1 stale doc), status sealed
- **Dogfood receipt:** sha256:35d95442a83fb2e997efc24094fac349bdaf1f69918aacbdfecb638410f78d88
- **Receipt verify:** runx verify -> valid: true, signature_mode: production, receipt_count: 1
- **Install test:** runx add -> installed successfully

## How to Install & Run
```bash
# Install
runx add umbtest03/docs-doctor@sha-7dbb7270bec3 --registry https://api.runx.ai

# Run with review inputs
runx skill umbtest03/docs-doctor@sha-7dbb7270bec3 --registry https://api.runx.ai --input-json docs_corpus='[{"id":"doc1","title":"Install Guide","content":"To install runx..."}]' --input-json product_surface='{"commands":[{"name":"init","description":"Initialize a skill"}]}' --input-json user_task_matrix='[{"task":"Publish a skill","steps":["Install runx CLI"]}]' -i style_policy='Documentation must include code examples.'

# Run and verify receipt
runx skill umbtest03/docs-doctor@sha-7dbb7270bec3 --registry https://api.runx.ai --input-json docs_corpus='[...]' --input-json product_surface='[...]' --input-json user_task_matrix='[...]' -i style_policy='...' --json | runx verify --receipt - --json
```

## TDD Workflow
1. **Plan:** Bounty claimed, task breakdown created
2. **RED:** Tests written (stale-docs, fresh-docs in X.yaml)
3. **GREEN:** Runner implemented to pass both cases
4. **Local harness (WSL):** 2/2 passed, 0 assertion errors
5. **Publish + hosted harness:** api.runx.ai validated server-side
6. **PR:** runxhq/runx#231
7. **Dogfood:** Skill executed with real input, sealed receipt obtained, runx verify passed

## Acceptance Criteria Checklist
- [x] runx CLI 0.6.14 (newer than minimum 0.6.14)
- [x] Claimant GitHub stars runxhq/runx (verified via API)
- [x] Exact package name: docs-doctor
- [x] Public PR against runxhq/runx (PR #231)
- [x] Raw fetchable x_yaml and skill_md from PR head commit
- [x] All artifacts describe same package version (sha-7dbb7270bec3)
- [x] Clean install via runx add
- [x] Local harness passed (WSL)
- [x] Hosted registry harness passed
- [x] Dogfood run with real input, sealed receipt
- [x] Receipt verified via runx verify (valid: true)
- [x] One sealed stale-docs case + one refused fresh-docs case
- [x] Typed inputs (docs_corpus[], product_surface, user_task_matrix[], style_policy) and outputs (findings[], coverage_map, proposal_status, answer)
- [x] No live retrieval, no external fetch, no mutation
- [x] Stale docs detected and reported with fix proposals
- [x] Fully documented surface returns no findings
