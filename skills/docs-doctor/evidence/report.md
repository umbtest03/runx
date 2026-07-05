# Delivery Report: #77 docs-doctor

## Summary
- **Bounty:** #77 - runx skill: docs doctor
- **Claimant:** @umbtest03 (Claim ID: 38f2263d-0511-4168-a0b7-37872c0dd9ba)
- **Date:** 2026-07-05
- **runx CLI:** runx-cli 0.6.14

## Artifacts

| # | Artifact | URL/Ref |
|---|----------|---------|
| 1 | public_url | https://runx.ai/x/umbtest03/docs-doctor |
| 2 | source_url | https://github.com/umbtest03/runx/tree/254a3690/skills/docs-doctor |
| 3 | pr_url | https://github.com/runxhq/runx/pull/231 |
| 4 | x_yaml | https://raw.githubusercontent.com/umbtest03/runx/254a3690/skills/docs-doctor/X.yaml |
| 5 | skill_md | https://raw.githubusercontent.com/umbtest03/runx/254a3690/skills/docs-doctor/SKILL.md |
| 6 | evidence_json | https://raw.githubusercontent.com/umbtest03/runx/254a3690/skills/docs-doctor/evidence/evidence.json |
| 7 | verification_json | https://raw.githubusercontent.com/umbtest03/runx/254a3690/skills/docs-doctor/evidence/verification.json |
| 8 | receipt_ref | sha256:35d95442a83fb2e997efc24094fac349bdaf1f69918aacbdfecb638410f78d88 |
| 9 | report | https://raw.githubusercontent.com/umbtest03/runx/254a3690/skills/docs-doctor/evidence/report.md |

## Skill Description
Finds stale product documentation by comparing docs against the actual product surface. Emits grounded findings with severity, evidence, and fix proposals. Never rewrites docs without a proposal.

## Verification Results
- **Local harness (WSL):** PASSED (2/2 cases, 0 assertion errors)
- **Hosted harness (api.runx.ai):** PASSED (server-side validation)
- **Registry publish:** umbtest03/docs-doctor@sha-7dbb7270bec3
- **PR:** https://github.com/runxhq/runx/pull/231 (branch: answer-from-docs)
- **Dogfood run:** 4 findings (3 critical, 1 minor), status sealed
- **Dogfood receipt:** sha256:35d95442a83fb2e997efc24094fac349bdaf1f69918aacbdfecb638410f78d88
- **Receipt verify:** runx verify -> valid: true, signature_mode: production, receipt_count: 1

## How to Install & Run
```bash
# Install
runx add umbtest03/docs-doctor@sha-7dbb7270bec3 --registry https://api.runx.ai

# Run with a product surface
runx skill umbtest03/docs-doctor@sha-7dbb7270bec3 --registry https://api.runx.ai \
  --input-json docs_corpus='[{"id":"x","title":"y","content":"z"}]' \
  --input-json product_surface='{"commands":[{"name":"init","description":"Init"}]}' \
  --input-json user_task_matrix='[]' \
  -i style_policy='Documentation must include examples.'

# Run and verify receipt
runx skill umbtest03/docs-doctor@sha-7dbb7270bec3 ... --json
```

## Acceptance Criteria Checklist
- [x] runx CLI 0.6.14
- [x] Claimant GitHub stars runxhq/runx
- [x] Exact package name: docs-doctor
- [x] Public PR against runxhq/runx (PR #231)
- [x] Raw fetchable x_yaml and skill_md from PR head commit
- [x] All artifacts describe same package version (sha-7dbb7270bec3)
- [x] Clean install via runx add
- [x] Local harness passed (WSL, 2/2)
- [x] Hosted registry harness passed
- [x] Dogfood run with real input, sealed receipt
- [x] Receipt verified via runx verify (valid: true)
- [x] One sealed case (stale-docs) + one refused case (fresh-docs)
- [x] Typed inputs (docs_corpus[], product_surface, user_task_matrix[], style_policy) and outputs (doc_findings[], coverage_map, patch_plan[], docs_pr_proposal)
- [x] No external fetch, no mutation, readonly sandbox
- [x] Every finding includes page, issue, severity, doc evidence, product-surface evidence, proposed fix scope
- [x] observations include finding count, coverage gaps, no-op path, proposal status, harness case names, receipt id
