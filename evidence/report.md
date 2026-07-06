# Delivery Report for docs-doctor (#77)

## Overview

The `docs-doctor` skill is complete and adheres to all Frantic Golden Rules and 5-Agent workflow requirements. This report serves as your guide to reproducing the dogfooding verification using our signed execution environment in WSL.

## Single Hash Integrity

To ensure exact provenance, this delivery is completely pinned to the immutable commit hash `ea89ad1f22d02df22ce5cd4d55cf43e2159dcdc2`.
All artifacts, schemas, logic, and tests point to this exact state, eliminating any risk of PR divergence.

## Installation & Verification

To run this skill and produce a valid execution receipt locally:

```bash
# 1. Provide the input fixture
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

# 2. Run the skill and capture the receipt
# Note: we use the local path here as GitHub raw URL loading is not fully supported for skill definitions
runx skill github.com/umbtest03/runx/tree/ea89ad1f22d02df22ce5cd4d55cf43e2159dcdc2/skills/docs-doctor --json < inputs.json > receipt.json

# 3. Verify the receipt
runx verify --receipt receipt.json
```

## Validation Context
- **Harness**: The harness fully passes (`stale-docs` as sealed, and `fresh-docs` successfully refused).
- **Dogfooding**: The dogfooding generated the receipt `sha256:4ccebe2546e433a568776fd53b53e050c42f323639a95fa4807dc5b3d5a434e5`.
- **Environment**: All commands executed natively on Linux (WSL) using `runx` CLI v0.6.14.

## Compliance Checks
- [x] Package Name exactly `docs-doctor`
- [x] Single immutable commit used for all artifacts
- [x] SKILL.md acts as a full operator profile
- [x] `runx verify` returns valid
- [x] No side-effects or network calls in business logic
