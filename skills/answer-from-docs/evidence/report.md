# Delivery Report: #87 answer-from-docs

## Summary
- **Bounty:** #87 - runx skill: answer-from-docs
- **Claimant:** @umbtest03 (Claim ID: 1683cf08-2c26-4a02-ab5c-38c82edb09fc)
- **Date:** 2026-07-05
- **runx CLI:** runx-cli 0.6.14

## Artifacts

| # | Artifact | URL/Ref |
|---|----------|---------|
| 1 | public_url | https://runx.ai/x/umbtest03/answer-from-docs |
| 2 | source_url | https://github.com/umbtest03/runx/tree/64b8bdeb/skills/answer-from-docs |
| 3 | pr_url | https://github.com/runxhq/runx/pull/231 |
| 4 | x_yaml | https://raw.githubusercontent.com/umbtest03/runx/64b8bdeb/skills/answer-from-docs/X.yaml |
| 5 | skill_md | https://raw.githubusercontent.com/umbtest03/runx/64b8bdeb/skills/answer-from-docs/SKILL.md |
| 6 | evidence_json | https://raw.githubusercontent.com/umbtest03/runx/64b8bdeb/skills/answer-from-docs/evidence/evidence.json |
| 7 | verification_json | https://raw.githubusercontent.com/umbtest03/runx/64b8bdeb/skills/answer-from-docs/evidence/verification.json |
| 8 | receipt_ref | sha256:65a86dfa545997d8166f0dca7af0fb4fd479ee179e1ec4dbf2076d62353b11d1 |
| 9 | report | https://raw.githubusercontent.com/umbtest03/runx/64b8bdeb/skills/answer-from-docs/evidence/report.md |

## Skill Description
Answers a natural-language question strictly from a bounded corpus. Returns grounded answers with citations or refuses when corpus lacks coverage.

## Verification Results
- **Local harness (WSL):** PASSED (2/2 cases, 0 assertion errors)
- **Hosted harness (api.runx.ai):** PASSED (server-side validation)
- **Registry publish:** umbtest03/answer-from-docs@sha-1ee1c7040328
- **PR:** https://github.com/runxhq/runx/pull/231
- **Dogfood run:** Grounded answer with citations, status sealed
- **Dogfood receipt:** sha256:65a86dfa545997d8166f0dca7af0fb4fd479ee179e1ec4dbf2076d62353b11d1
- **Receipt verify:** runx verify -> valid: true, signature_mode: production, receipt_count: 1
- **Install test:** runx add -> installed successfully

## How to Install & Run
```bash
# Install
runx add umbtest03/answer-from-docs@sha-1ee1c7040328 --registry https://api.runx.ai

# Run with a question
runx skill umbtest03/answer-from-docs@sha-1ee1c7040328 --registry https://api.runx.ai -i question="What is X?" --input-json corpus='[{"id":"doc1","text":"X is Y."}]'

# Run and verify receipt
runx skill umbtest03/answer-from-docs@sha-1ee1c7040328 --registry https://api.runx.ai -i question="What is X?" --input-json corpus='[...]' --json | runx verify --receipt - --json
```

## TDD Workflow
1. **Plan:** Bounty claimed, task breakdown created
2. **RED:** Tests written (grounded-answer, unanswered-question in X.yaml)
3. **GREEN:** Runner implemented to pass both cases
4. **Local harness (WSL):** 2/2 passed, 0 assertion errors
5. **Publish + hosted harness:** api.runx.ai validated server-side
6. **PR:** runxhq/runx#231
7. **Dogfood:** Skill executed with real input, sealed receipt obtained, runx verify passed

## Acceptance Criteria Checklist
- [x] runx CLI 0.6.14 (newer than minimum 0.6.14)
- [x] Claimant GitHub stars runxhq/runx (verified via API)
- [x] Exact package name: answer-from-docs
- [x] Public PR against runxhq/runx (PR #231)
- [x] Raw fetchable x_yaml and skill_md from PR head commit
- [x] All artifacts describe same package version (sha-1ee1c7040328)
- [x] Clean install via runx add
- [x] Local harness passed (WSL)
- [x] Hosted registry harness passed
- [x] Dogfood run with real input, sealed receipt
- [x] Receipt verified via runx verify (valid: true)
- [x] One sealed grounded case + one failure case
- [x] Typed inputs (question, corpus[]) and outputs (answer, kb_gaps[], grounded)
- [x] No live retrieval, no external fetch, no mutation
- [x] Every answer sentence supported by citation
- [x] Ungrounded questions refused with kb_gaps
