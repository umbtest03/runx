# SBOM Maker Skill - Delivery Report

## Overview
Published `sbom-maker` runx skill (`umbtest03/sbom-maker@1.2.0`) with a real, verifiable post-publish dogfood run.
This delivery reopens the packet as a **clean, additive-only PR**: it adds `skills/sbom-maker/**`
on top of the current `runxhq/runx` main and **deletes no other skill**.

## Package
- **Skill**: `sbom-maker` | **Owner**: `umbtest03` | **Version**: `1.2.0`
- **Registry ref**: `umbtest03/sbom-maker@1.2.0` | **Digest**: `sha256:f2c60e7aa4cc952e3fd3ed85b2fa951e1c94838c5b0dcef2b026597834a7843e`
- **public_url**: https://runx.ai/x/umbtest03/sbom-maker@1.2.0
- **pr_url**: https://github.com/runxhq/runx/pull/315
- **source_url**: https://github.com/umbtest03/runx/tree/8706672524005ceffb7ed60152de74e3ee108ca7
- **raw X.yaml**: https://raw.githubusercontent.com/umbtest03/runx/8706672524005ceffb7ed60152de74e3ee108ca7/skills/sbom-maker/X.yaml
- **raw SKILL.md**: https://raw.githubusercontent.com/umbtest03/runx/8706672524005ceffb7ed60152de74e3ee108ca7/skills/sbom-maker/SKILL.md

## runx CLI
- `runx --version` -> **runx-cli 0.6.16** (>= 0.6.14 floor). Used for publish, install, dogfood, and verify.

## Fixes vs the 2026-07-14 rejection
1. **Non-destructive PR** - clean branch off current `runxhq/runx` main; only `skills/sbom-maker/**` is added.
2. **evidence_location per format** - `run.mjs` records the real key: `packages["node_modules/<name>"]`
   for the npm v2 packages-map format and `dependencies["<name>"]` for the classic v1 format.
3. **Typed outputs** - `X.yaml` `runners.default.outputs` declares `sbom`, `components`, `license_summary`, `license_risks`.

## Install (clean)
- `runx add umbtest03/sbom-maker@1.2.0 --registry https://api.runx.ai` -> source=remote, status=installed, digest `sha256:f2c60e7aa4cc952e3fd3ed85b2fa951e1c94838c5b0dcef2b026597834a7843e`.

## Harness
- Local source: `runx harness ./skills/sbom-maker` -> **3/3 PASSED, 0 assertion errors** (WSL Linux).
- Registry-downloaded copy: `runx harness ./skills/umbtest03/sbom-maker/1.2.0` -> **3/3 PASSED, 0 errors**.
- Cases: **supported-lockfile** (sealed), **supported-lockfile-classic** (sealed, classic dependencies-map format),
  **unsupported-lockfile** (refused - no SBOM).

## Dogfood (post-publish, real)
- Command: `runx skill umbtest03/sbom-maker@1.2.0 --registry https://api.runx.ai --json -i lockfile_type=npm-shrinkwrap --input-json lockfile='{"name":"demo-app","version":"1.0.0","lockfileVersion":2,"requires":true,"packages":{"":{"name":"demo-app","version":"1.0.0"},"node_modules/lodash":{"version":"4.17.21","license":"MIT","resolved":"https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz"},"node_modules/express":{"version":"4.18.2","license":"GPL-3.0","resolved":"https://registry.npmjs.org/express/-/express-4.18.2.tgz"}}}' -R ./receipts`
- Output: **2 components** grounded per format - express 4.18.2 GPL-3.0 (`packages["node_modules/express"]`),
  lodash 4.17.21 MIT (`packages["node_modules/lodash"]`); license_counts {GPL-3.0:1, MIT:1};
  **1 license risk**: express GPL-3.0 (high).
- Receipt: `runx:receipt:sha256:8d4eda9bad2a398fc2c022b3e452ea72482d88e5f2f99758e962246bfd155bea`
- `runx verify --receipt dogfood_receipt.json --json` -> **valid: true, signature_mode: production, signature: valid**.

## Provenance (single source revision)
- Registry provenance (from the dogfood receipt): registry_source=remote https://api.runx.ai, skill_id=umbtest03/sbom-maker, version=1.2.0, trust_state=trusted, trust_tier=community - the run resolved the published
  package from the remote registry at the exact published version.
- source_url, raw X.yaml, raw SKILL.md and verification.json all resolve at one source revision: commit `8706672524005ceffb7ed60152de74e3ee108ca7`.
- The skill files at `8706672524005ceffb7ed60152de74e3ee108ca7` are byte-identical to the published package `umbtest03/sbom-maker@1.2.0` (matching digest `sha256:f2c60e7aa4cc952e3fd3ed85b2fa951e1c94838c5b0dcef2b026597834a7843e`).
- This report and evidence.json are committed as the direct child of `8706672524005ceffb7ed60152de74e3ee108ca7` and describe that same revision;
  the recorded receipt_ref is the post-publish dogfood run of the published package, not a harness fixture seal.

## What to inspect first
1. `runx verify --receipt dogfood_receipt.json --json` (valid=true, production).
2. `evidence.json` dogfood.output (real SBOM, 2 components, correct evidence_location, GPL-3.0 risk).
3. Raw X.yaml (typed outputs) / SKILL.md / verification.json at source revision `8706672524005ceffb7ed60152de74e3ee108ca7`.
