---
name: run-history
description: Read Runx's native receipt history and skill catalog, then return deterministic run outcomes, catalog test coverage, and governance follow-ups without model-authored metrics.
runx:
  category: data
---

# Run History

Read the local Runx ledger through `runx history --json` and the installed
catalog through `runx list skills --ok-only --json`. The runner computes the
report itself; it does not ask an agent to invent commands or transcribe counts.

Use it for operational questions about recent Runx activity, failed or blocked
runs, pending runs, frequently used receipt subjects, and skill entries without
fixtures or inline harness cases. Use `audit-receipt` or `review-receipt` to
inspect one suspicious run, and `least-privilege` when receipt-backed authority
usage is available for grant attenuation.

## Evidence boundary

- Live execution reads only the native history and catalog JSON projections.
- History reads are bounded to 1,000 rows by default and 10,000 at most.
- `history_receipts`, `pending_runs`, and `catalog_items` are replay inputs for
  harnesses and controlled analysis; when supplied, no native history or list
  command is executed.
- Empty history returns `needs_more_evidence` rather than a healthy verdict.
- `closed_rate` is the share of terminal receipt rows whose status is `closed`.
- `refusal_rate` counts `blocked` and `declined` terminal rows. It is reported
  as an observation, not automatically treated as a defect.
- Catalog coverage is the number of native catalog entries declaring at least
  one fixture or inline harness case. It is not a maturity grade.

## Output

`history_report` contains:

- the exact resolved query and native source commands;
- terminal and pending counts, status counts, closed/refusal rates;
- the most frequent receipt subjects;
- catalog entry and test-coverage counts;
- bounded recommendations routed to `review-receipt`, `audit-receipt`, or
  `skill-lab harness`;
- limitations when native projections cannot support a stronger claim.

This skill never executes another skill, changes a grant, or mutates the
receipt store.
