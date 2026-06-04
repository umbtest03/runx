---
spec_version: '2.0'
task_id: runx-github-mcp-hero-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T20:45:11Z'
status: completed
harden_status: not_run
size: medium
risk_level: medium
---

# runx-github-mcp-hero-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-04T20:45:11Z
Review gate: pass

## Summary

Ship the first hero (executable today, gates nothing): governed GitHub via the MCP
adapter, scope-bounded, receipt-sealed. Grant `repo.read`, the agent reads an
issue/PR fine, attempts an out-of-scope write, admission REFUSES, and the sealed
denial receipt verifies offline. Plus the two read-only siblings that make it read
as a category: governed issue-triage (MCP read path plus the existing triage
packet runner) and a governed PR-review note (one scoped comment via the
public-comment path). The survey found NO `type: mcp` GitHub example anywhere
under `oss/examples` today, so this builds the first one — the demo north-star.md
names as the executable hero.

## Objectives

- A GitHub MCP-source skill that reads under `repo.read`, with a sealed receipt and
  an offline-verifiable governed REFUSAL of an out-of-scope write — the headline demo.
- Wire the existing `issue-triage` skill (already references a real github issue_url
  + comment channel) to the MCP read path; the MCP runner proves no mutation scope
  is exercised, while the existing response runner remains the triage-packet
  emitter.
- A new governed PR-review-note sibling: one scoped comment via the public-comment
  path, refusing any push/merge attempt.
- Each first-party skill sibling reaches a maturity tier with a harness case; the
  harness expectations prove no out-of-scope scope was exercised.

## Scope

In scope:
- The GitHub MCP-source example skill + the refusal demo (grant repo.read → attempt
  write → sealed denial → offline verify with `verify.mjs`).
- The `issue-triage` sibling wired to MCP read; the new PR-review-note sibling.
- Harness cases for the example and both siblings; maturity tiering + lock entries
  for the first-party skill siblings.

Out of scope:
- Non-GitHub hosted provider brokerage (cloud/private, separate).
- Any write beyond the single scoped PR comment; auto-merge; PR-opening (that is the
  TS `issue-to-pr` lane, not this hero).

## Dependencies

- SHIPPED: the MCP adapter (`serve_mcp_json_rpc`) + the MCP-source client, agent-step,
  GitHub connect (the only wired provider), receipts, authority/scopes, the harness.
- The governed-tool-call convention for the scoped-comment mutation (PR-review note).

## Assumptions

- GitHub is the only wired provider (north-star.md); the hero is scoped to it.
- The refusal must be a REAL admission denial (over-scope attempt), not a staged one,
  so the demo's credibility holds.

## Touchpoints

- A new `oss/examples/github-mcp-*` (the first MCP-source example) + harness.
- `oss/skills/issue-triage` (wire to MCP read); a new PR-review-note skill.
- The MCP adapter + GitHub connect path.
- `verify.mjs` (offline-verify the denial receipt).

## Risks

- **Mutation scope creep.** The PR-review note must be gated to exactly the comment
  scope; an over-broad scope undermines the least-privilege story. Mitigation: bound
  the scope + assert refusal of push/merge in the harness.
- **Demo credibility.** A staged refusal reads as theatre. Mitigation: the refusal is
  a genuine admission denial of an out-of-scope act, sealed + offline-verifiable.

## Acceptance

Profile: strict

Validation:
- The GitHub MCP read hero: a scoped read seals a receipt; an out-of-scope write is
  refused at admission with a sealed denial that `verify.mjs` confirms offline.
- The two siblings run: `issue-triage` has both its existing packet-emitting
  response runner and its MCP read runner; the PR-review note posts one scoped
  comment and refuses push/merge.
- `pnpm fixtures:harness:check` + the new harness cases pass; the sibling skills are
  maturity-tiered and locked.

## Phase 1: GitHub MCP read hero + refusal demo

Status: completed
Dependencies: MCP adapter, GitHub connect (shipped)

Objective: the first MCP-source GitHub example with the
grant→read→refuse→sealed-denial flow verified offline.

Changes:
- Build the GitHub MCP-source example skill + harness; the refusal demo + run script.

Acceptance:
- [x] `ac1` command - read seals, out-of-scope write refused + verifies offline
  - Command: `RUNX_BIN="${RUNX_BIN:-$(pwd)/crates/target/debug/runx}" examples/github-mcp-hero/run.sh`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-10

## Phase 2: The two read-only siblings (issue-triage + PR-review note)

Status: completed
Dependencies: Phase 1

Objective: the category-defining siblings, each sealed + scope-audited.

Changes:
- Wire `issue-triage` to MCP read; build the scoped PR-review-note skill; harness + maturity for both.

Acceptance:
- [x] `ac2` command - siblings seal + least-privilege holds
  - Command: `RUNX_BIN="${RUNX_BIN:-$(pwd)/crates/target/debug/runx}"; RDIR1="$(mktemp -d)"; RDIR2="$(mktemp -d)"; RUNX_RECEIPT_SIGN_KID=runx-demo-key RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64=QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI= RUNX_RECEIPT_SIGN_ISSUER_TYPE=hosted "$RUNX_BIN" harness skills/issue-triage --receipt-dir "$RDIR1" --json && RUNX_RECEIPT_SIGN_KID=runx-demo-key RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64=QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI= RUNX_RECEIPT_SIGN_ISSUER_TYPE=hosted "$RUNX_BIN" harness skills/pr-review-note --receipt-dir "$RDIR2" --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15

## Rollback

- Additive examples/skills. Remove the github-mcp example + the PR-review-note skill
  and revert the issue-triage wiring; no contract or SourceKind change.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: command
Output: command.stdout
Summary: Command-provider verification passed after scafld build recorded fresh post-blocker evidence. The exact acceptance commands were rerun immediately before the lifecycle repair: examples/github-mcp-hero/run.sh passed and verify.mjs verified the blocked authority_denied receipt offline; skills/issue-triage and skills/pr-review-note harnesses both passed with signed isolated receipt dirs. The previous lifecycle-only blocker is closed.

Attack log:
- `scafld build runx-github-mcp-hero-v1 --json`: confirm a post-blocker build event exists under scafld 2.4.7 -> clean
- `examples/github-mcp-hero/run.sh`: verify exact ac1 acceptance was rerun and passed -> clean
- `skills/issue-triage and skills/pr-review-note`: verify exact ac2 sibling harness acceptance was rerun and passed -> clean
- `LIFECYCLE-1`: confirm prior process blocker is resolved by build event -> clean

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- none
