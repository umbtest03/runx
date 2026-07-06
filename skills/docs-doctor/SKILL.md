---
name: docs-doctor
description: Finds stale product docs by comparing them with the actual surface they describe, emitting finding reports and patch plans.
version: 1.0.1
author: umbtest03
---

# Docs Doctor Skill

## Overview

The `docs-doctor` skill is a governed analysis tool designed to identify stale, missing, or contradictory documentation by rigorously comparing your documentation corpus against your actual product surface. It does not rewrite documentation without explicit approval; instead, it emits a structured analysis including doc findings, a coverage map, and a patch plan. When appropriate, it also provides a gated PR proposal for human or CI review.

## Architecture & Data Flow

This skill operates completely deterministically and securely. It requires four main inputs:
1. **Docs Corpus:** An array of existing documentation pages.
2. **Product Surface:** A definition of the current product state (e.g., commands, endpoints, schemas).
3. **User Task Matrix:** Expected user tasks and goals to ensure documentation is useful.
4. **Style Policy:** Rules regarding documentation style and requirements.

### Workflow
1. **Ingestion:** Reads all inputs via standard input in JSON format.
2. **Analysis:** Iterates through the product surface and matches each command/endpoint/schema against the docs corpus.
3. **Evaluation:** Flags missing documentation and validates existing docs against the style policy.
4. **Emission:** Outputs a structured JSON payload containing the findings, or gracefully refuses execution if the docs are perfectly fresh.

## Input Schema

The skill expects a single JSON object on standard input containing:

| Field | Type | Description |
|-------|------|-------------|
| `docs_corpus` | Array of Objects | Represents the current state of documentation (e.g., `page` and `content`). |
| `product_surface` | Object | The actual technical surface area (`commands`, `endpoints`, `schemas`). |
| `user_task_matrix` | Array of Objects | Expected workflows and tasks the docs should cover. |
| `style_policy` | String / Object | Governance rules for how documentation should be written. |

## Output Schema

When stale documentation is found, the skill emits the following JSON object on standard output:

| Field | Type | Description |
|-------|------|-------------|
| `doc_findings` | Array of Objects | Specific issues found (e.g., `page`, `issue`, `severity`, `doc_evidence`). |
| `coverage_map` | Object | A mapping of product surface items to their documentation status. |
| `patch_plan` | Array of Strings | Human-readable instructions on how to resolve the findings. |
| `docs_pr_proposal` | Object | A structured proposal meant for automated issue-to-pr handoff. |

## Execution Cases

### 1. Stale Docs (Sealed)
If the `product_surface` contains elements that are entirely missing or improperly documented in the `docs_corpus`, the skill will emit the full suite of findings, coverage maps, and patch plans. It will exit with a standard `0` status code.

### 2. Fresh Docs (Refused)
If the documentation is perfectly aligned with the product surface and no issues are detected, the skill acts as a no-op. It will emit a refusal message to standard error ("Refused: Docs already match the product surface") and exit with a non-zero status code (`1`).

## Edge Case Management

- **Empty Docs Corpus:** If no documentation is provided but a product surface exists, the skill flags the entire product surface as undocumented with `high` severity.
- **Empty Product Surface:** If there is no product surface, the skill immediately evaluates the docs as "fresh" (as there are no product features to document) and refuses execution.
- **Malformed Inputs:** The skill assumes structurally sound JSON. If parsing fails, the skill will crash natively, relying on the governed execution environment to capture the exit code.

## Installation and Usage

To install the skill in your local runx environment:
```bash
runx add umbtest03/docs-doctor@1.0.0
```

To run the skill manually (dogfooding):
```bash
cat inputs.json | runx skill umbtest03/docs-doctor@1.0.0 --json > receipt.json
```

To verify the generated receipt:
```bash
runx verify --receipt receipt.json --json
```

## Security & Limitations

This skill adheres strictly to the runx execution principles:
- **Read-Only:** It performs absolutely no mutation on your file system, git repositories, or external services.
- **Network Isolated:** It performs zero external network calls. It relies entirely on the provided input fixtures.
- **No Side Effects:** The `docs_pr_proposal` is purely an informational artifact. It must be explicitly passed to another system (like `issue-to-pr`) to enact changes.

## License
MIT License. Created for the Frantic Bounty program.