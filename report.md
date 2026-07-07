# SBOM Maker Skill - Delivery Report

## Overview
This report documents the published `sbom-maker` runx skill and a real, verifiable
post-publish dogfood run.

## Package
- **Skill**: `sbom-maker` | **Owner**: `umbtest03` | **Version**: `sha-b0be55e0ee89`
- **Registry ref**: `umbtest03/sbom-maker@sha-b0be55e0ee89`
- **public_url**: https://runx.ai/x/umbtest03/sbom-maker@sha-b0be55e0ee89
- **pr_url**: https://github.com/runxhq/runx/pull/261
- **source_url**: https://github.com/umbtest03/runx/tree/66816d9a2ee8d700473738a93e0a40444dd88664
- **raw X.yaml**: https://raw.githubusercontent.com/umbtest03/runx/66816d9a2ee8d700473738a93e0a40444dd88664/skills/sbom-maker/X.yaml
- **raw SKILL.md**: https://raw.githubusercontent.com/umbtest03/runx/66816d9a2ee8d700473738a93e0a40444dd88664/skills/sbom-maker/SKILL.md

## runx CLI
- `runx --version` -> **runx-cli 0.6.16** (>= 0.6.14 floor). Used for install, dogfood, and verify.

## Install (clean)
- `runx add umbtest03/sbom-maker@sha-b0be55e0ee89 --registry https://api.runx.ai` -> source=remote, status=installed.

## Harness
- Local harness: `runx harness ./skills/sbom-maker` -> **2/2 PASSED, 0 assertion errors** (WSL Linux).
- Cases: **supported-lockfile** (sealed - emits SBOM + license summary), **unsupported-lockfile** (refused - no SBOM).

## Dogfood (post-publish, real)
- Command: `runx skill umbtest03/sbom-maker@sha-b0be55e0ee89 --registry https://api.runx.ai --json -i lockfile_type=npm-shrinkwrap --input-json lockfile='{"name":"demo-app","version":"1.0.0","lockfileVersion":2,"requires":true,"packages":{"":{"name":"demo-app","version":"1.0.0"},"node_modules/lodash":{"version":"4.17.21","license":"MIT","resolved":"https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz"},"node_modules/express":{"version":"4.18.2","license":"GPL-3.0","resolved":"https://registry.npmjs.org/express/-/express-4.18.2.tgz"}}}' -R ./receipts`
- Output: **2 components** (express 4.18.2 GPL-3.0, lodash 4.17.21 MIT); license_counts {GPL-3.0:1, MIT:1};
  **1 license risk flagged**: express GPL-3.0 (high).
- Receipt: `runx:receipt:sha256:8f369097cae2d34750a2e23d143f049e130477d82c77e058653106e8f0a7c9ff`
- `runx verify --receipt dogfood_receipt.json --json` -> **valid: true, signature_mode: production, signature: valid**.

## Provenance (single source revision)
- Registry provenance (from the dogfood receipt): registry_source=remote https://api.runx.ai, skill_id=umbtest03/sbom-maker, version=sha-b0be55e0ee89, trust_state=trusted, trust_tier=community — the dogfood run
  resolved the published package from the remote registry at the exact published version.
- source_url, raw X.yaml, raw SKILL.md and verification.json all resolve at one source revision:
  commit `66816d9a2ee8d700473738a93e0a40444dd88664` on the `umbtest03/runx` `sbom-maker` branch.
- The skill files at `66816d9a2ee8d700473738a93e0a40444dd88664` are byte-identical to the published package `umbtest03/sbom-maker@sha-b0be55e0ee89` (matching digest).
- This report and evidence.json are committed as the direct child of `66816d9a2ee8d700473738a93e0a40444dd88664` and describe that same
  revision; the recorded receipt_ref is the post-publish dogfood run of the published package, not a
  harness fixture seal.

## What to inspect first
1. `runx verify --receipt dogfood_receipt.json --json` (valid=true, production).
2. `evidence.json` dogfood.output (real SBOM with 2 components + GPL-3.0 risk).
3. Raw X.yaml / SKILL.md / verification.json at source revision `66816d9a2ee8d700473738a93e0a40444dd88664`.
