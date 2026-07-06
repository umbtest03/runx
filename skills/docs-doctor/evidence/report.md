# Docs Doctor Skill - Delivery Report

## Package
- **Skill** `docs-doctor` | **Owner** `umbtest03` | **Version** `sha-169c63676569`
- **Registry ref** `umbtest03/docs-doctor@sha-169c63676569`
- **public_url** https://runx.ai/x/umbtest03/docs-doctor@sha-169c63676569
- **pr_url** https://github.com/runxhq/runx/pull/259
- **source_url** https://github.com/umbtest03/runx/tree/f217906c9ae1f193a3b57acb7c60505bdb073923
- **raw X.yaml** https://raw.githubusercontent.com/umbtest03/runx/f217906c9ae1f193a3b57acb7c60505bdb073923/skills/docs-doctor/X.yaml
- **raw SKILL.md** https://raw.githubusercontent.com/umbtest03/runx/f217906c9ae1f193a3b57acb7c60505bdb073923/skills/docs-doctor/SKILL.md

## runx CLI
`runx --version` -> **runx-cli 0.6.16** (>= 0.6.14). Used for install, dogfood, verify.

## Install (clean)
`runx add umbtest03/docs-doctor@sha-169c63676569 --registry https://api.runx.ai` -> source=remote, status=installed.

## Harness
`runx harness ./skills/docs-doctor` -> **2/2 PASSED, 0 assertion errors** (WSL Linux).
Cases: **stale-docs** (sealed - emits doc_findings, coverage_map, patch_plan, docs_pr_proposal),
**fresh-docs** (refused no-op - docs already match the product surface).

## Dogfood (post-publish, real)
- Command: `runx skill umbtest03/docs-doctor@sha-169c63676569 --registry https://api.runx.ai --json --input-json docs_corpus='[{"page":"cli-reference.md","content":"# CLI Reference\n\nRun the system with `my-app run`."}]' --input-json product_surface='{"commands":[{"name":"my-app run","description":"Start the app"},{"name":"my-app deploy","description":"Deploy to cloud (NEW IN v2)"}],"endpoints":[],"schemas":[]}' --input-json user_task_matrix='[{"task":"Deploying the app","expected_docs":"Users need to know how to deploy."}]' -i style_policy='All new commands must be documented in cli-reference.md.' -R ./receipts`
- Output: **1 doc finding(s)**; coverage gaps: my-app deploy; gated docs_pr_proposal present.
- Receipt: `runx:receipt:sha256:7016659dc1ee1c333f02b9cc5597bd5fd399abb5f3676960004b700451133fec`
- `runx verify --receipt dogfood_receipt.json --json` -> **valid: true, signature_mode: production, signature: valid**.

## Provenance
All bound artifact URLs pin to a single PR head commit on `umbtest03/runx` `docs-doctor-v2` (PR #259).
The skill files are byte-identical to published `sha-169c63676569`. The recorded receipt_ref is the post-publish
dogfood run, not a harness fixture seal.

## What to inspect first
1. `runx verify --receipt dogfood_receipt.json --json` (valid=true, production).
2. `evidence.json` dogfood.output (doc_findings + coverage_map + patch_plan + docs_pr_proposal).
3. Raw X.yaml / SKILL.md at the PR head commit.
