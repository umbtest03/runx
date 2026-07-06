# Delivery Report for docs-doctor (#77)

## Overview

The `docs-doctor` skill is complete and adheres to all Frantic Golden Rules and 5-Agent workflow requirements. This report serves as your guide to reproducing the dogfooding verification using our signed execution environment in WSL.
This delivery rectifies previous coherence issues by merging all evidence and source code into a single immutable commit on the `docs-doctor-v2` PR branch, and uses a post-publish `runx skill` dogfood run with the latest `runx-cli 0.6.16`.

## Single Hash Integrity

To ensure exact provenance, this delivery is completely pinned to the PR head commit of branch `docs-doctor-v2`.
All artifacts, schemas, logic, and tests point to this exact state, eliminating any risk of PR divergence.
The package `umbtest03/docs-doctor@sha-20d6c18021dc` was published from this source state.

## Installation & Verification

To run this skill and produce a valid execution receipt locally without private context:

```bash
# 1. Install the published skill
runx add umbtest03/docs-doctor@sha-20d6c18021dc

# 2. Provide the input fixture
cat << 'EOF' > inputs.json
{
  "docs_corpus": [
    {
      "page": "cli-reference.md",
      "content": "# CLI Reference\n\nRun the system with `my-app run`."
    }
  ],
  "product_surface": {
    "commands": [
      {
        "name": "my-app run",
        "description": "Start the app"
      },
      {
        "name": "my-app deploy",
        "description": "Deploy to cloud (NEW IN v2)"
      }
    ],
    "endpoints": [],
    "schemas": []
  },
  "user_task_matrix": [
    {
      "task": "Deploying the app",
      "expected_docs": "Users need to know how to deploy."
    }
  ],
  "style_policy": "All new commands must be documented in cli-reference.md."
}
EOF

# 3. Run the skill and capture the receipt
runx skill umbtest03/docs-doctor@sha-20d6c18021dc --registry https://api.runx.ai --json < inputs.json > receipt.json

# 4. Verify the receipt
runx verify --receipt receipt.json --json
```

## Validation Context
- **Harness**: The hosted registry harness passes with 2 cases (`stale-docs` as sealed, and `fresh-docs` successfully refused).
- **Dogfooding**: The dogfooding of the published package generated the receipt `sha256:d018744ddd0dc40567d2d7a87d82d0f4dbe8b257548917f778f9e84568b2993f`.
- **Environment**: All commands executed natively on Linux (WSL) using `runx` CLI v0.6.16.

## Compliance Checks
- [x] Package Name exactly `docs-doctor`
- [x] Published to live registry at `https://runx.ai/x/umbtest03/docs-doctor@sha-20d6c18021dc`
- [x] Single immutable commit used for all artifacts
- [x] SKILL.md acts as a full operator profile
- [x] `runx verify` returns valid for post-publish dogfood
- [x] GitHub repository `runxhq/runx` starred by `umbtest03`
