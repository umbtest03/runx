---
spec_version: '2.0'
task_id: rust-kernel-port-orchestration
created: '2026-05-17T00:00:00Z'
updated: '2026-05-21T11:44:59Z'
status: cancelled
harden_status: not_run
size: small
risk_level: medium
---

# Rust kernel port orchestration

## Current State

Status: cancelled
Current phase: decommissioned planning record
Next: done
Reason: cancel
Blockers: none
Allowed follow-up command: `none`
Latest runner update: superseded by archived completed sub-specs and fresh
Review gate: not_started; intentionally not used as completion evidence.

## Summary

This draft is no longer an executable umbrella spec. It is retained only as a
supersession record explaining why the previous multi-phase lifecycle driver
must not be resumed.

The original plan expected live draft sub-specs for contracts bootstrap,
fixtures, state-machine parity, policy parity, and advisory CI governance. Those
work items now exist as archived completed specs, while this orchestration file
still has empty Phase Receipts and no review verdict. Backfilling those receipts
from current files, git history, or archived sub-specs would create false audit
evidence. Treat the observed implementation as current local fact, not as proof
that this orchestration lifecycle ran.

Remaining useful work is split into narrow executable slices. Each slice must be
hardened, approved, executed, reviewed, and completed independently if it is
chosen for execution.

## Current Local Facts

- `crates/runx-core` exists and exports Rust state-machine and policy parity
  surfaces.
- The Rust workspace includes `runx-cli`, `runx-contracts`, `runx-core`,
  `runx-parser`, `runx-receipts`, `runx-runtime`, and `runx-sdk`.
- Kernel fixture docs state that current policy fixtures cover authority proof,
  credential binding, scope admission, public work, local admission, sandbox,
  retry, and graph-scope admission.
- Payment-authority subset logic exists in Rust as
  `runx_core::policy::is_payment_authority_subset`, but current kernel fixture
  docs still describe payment-authority fixture parity as a separate executable
  slice.
- `scripts/count-clean-kernel-prs.ts`, its fixture data, and tests exist. The
  blocking-promotion spec still needs audited advisory-start and five-clean-PR
  evidence before any CI promotion.
- `docs/trusted-kernel-package-truth.md` still says TypeScript is authoritative
  and Rust kernel parity remains advisory in CI until
  `rust-kernel-blocking-promotion` completes.
- This orchestration file's own Phase Receipts and Review gate are empty.

## Replacement Work Queue

Executable slices that replace the useful residue of this draft:

- `rust-kernel-payment-authority-fixture-parity`: add TypeScript-generated
  kernel fixture parity for the existing pure Rust payment-authority subset
  helper. This is limited to fixture/oracle coverage for
  `is_payment_authority_subset`; it must not touch runtime payment execution or
  payment rails.
- `rust-kernel-clean-pr-counter-semantics`: harden the existing clean-kernel PR
  counter semantics before promotion evidence is trusted. This is limited to
  advisory-start evidence handling, qualifying/non-qualifying PR classification,
  required passing evidence, and fixture-mode auditability; it must not flip CI
  from advisory to blocking.
- `rust-kernel-blocking-promotion`: existing draft that owns the later
  advisory-to-blocking CI flip after clean evidence exists. It remains blocked
  until it has explicit advisory-start evidence and five qualifying
  kernel-touching PRs, or a replacement audit explicitly supersedes this
  obsolete orchestration file.

No new docs-coherence slice is created here because the local docs already
carry the refreshed Rust-core, advisory-CI, and TypeScript-authoritative
language. Future docs work should be attached to a concrete drift finding.

## Non-Goals

- Do not reconstruct this spec's missing Phase Receipts.
- Do not mark this spec completed.
- Do not use this file to approve or sequence archived sub-specs.
- Do not write code from this decommissioning update.
- Do not change runtime, MCP, adapter, parser, receipt, SDK, CLI, or payment
  rail behavior from this file.
- Do not change CI advisory/blocking status from this file.

## Acceptance

This file has no executable lifecycle acceptance. It is a pointer record only.

Validation for future agents is inspection-based:

- `rust-kernel-port-orchestration` Current State says obsolete as written and
  forbids harden/approve/execute/review/complete on this task id.
- Remaining executable work is represented by replacement slices rather than
  by phases in this file.
- Empty historical receipts are acknowledged as non-evidence and are not
  backfilled.

## Review

Status: not_started
Verdict: none
Timestamp: none
Review rounds: none
Reviewer mode: none
Reviewer session: none
Round status: none
Override applied: none
Override reason: none
Override confirmed at: none
Reviewed head: none
Reviewed dirty: none
Reviewed diff: none
Blocking count: none
Non-blocking count: none

Reviewer note: a future reviewer must not treat this file as evidence that the
kernel port lifecycle completed. Review the replacement executable slice being
run instead.

Findings:
- none

Passes:
- none

## Phase Receipts

No valid orchestration Phase Receipts exist for this task id.

The previous draft contained placeholder receipt fields. They were not filled
at execution time and must not be reconstructed from current files, archived
sub-specs, or git history. Archived completed sub-specs may be useful local
facts for planning, but they are not receipts for this orchestration task.

## Deviations

- 2026-05-20: The original orchestration lifecycle was superseded by observed
  local repo state and archived sub-specs without populated receipts in this
  file. This update intentionally prevents further execution of the umbrella
  driver and moves remaining work into narrow slices.

## Metadata

Estimated effort hours: none; this spec is no longer executable.
Actual effort hours: none
AI model: none
React cycles: none

Tags:
- rust
- trusted-kernel
- orchestration
- superseded
- governance

## Origin

Source:
- user requested an umbrella spec leveraging scafld features to drive the Rust
  pure-kernel port without data loss across agent sessions.

Repo:
- runxhq/runx

Git:
- none

Sync:
- none

Supersession:
- superseded_by: rust-kernel-payment-authority-fixture-parity
- superseded_by: rust-kernel-clean-pr-counter-semantics
- handoff_to: rust-kernel-blocking-promotion

## Harden Rounds

- none

## Planning Log

- 2026-05-17T00:00:00Z: Drafted as umbrella orchestration for the kernel parity
  sub-specs, with Phase Receipts intended to preserve audit state across
  sessions.
- 2026-05-17T02:10:00Z: Added `rust-contracts-bootstrap` as a pre-kernel gate.
- 2026-05-20T00:00:00Z: Marked obsolete as written. The current repo has Rust
  core surfaces, refreshed docs, and a clean-kernel counter, but this file still
  has empty receipts/review. Remaining useful work now lives in fresh
  executable slices instead of this umbrella driver.
