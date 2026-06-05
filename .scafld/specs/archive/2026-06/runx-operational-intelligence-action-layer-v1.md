---
spec_version: '2.0'
task_id: runx-operational-intelligence-action-layer-v1
created: '2026-05-27T14:41:04Z'
updated: '2026-06-05T04:15:27Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# Runx Operational Intelligence Action Layer

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-04T22:43:40Z
Review gate: pass

## Summary

This unit locks the cross-repo operational intelligence action layer between
Runx OSS and Nitrosend:

`source event -> context artifact -> signal -> decision -> proposal -> governed action -> outcome story`

Runx owns the generic execution/audit layer: `runx.operational_proposal.v1`,
central references, receipts, policy admission, governed actions, story/outbox,
and final outcome projection. Nitrosend owns product meaning: Slack/GitHub UX,
source hydration, customer/account enrichment, owner routes, labels/projects,
support copy, and provider-specific dispatch.

The completed Runx children already shipped the generic contract, composition,
and story/outbox pieces. The completed Nitrosend integration already proves the
support reply, create issue, build fix, golden path, alert no-action,
escalation, outreach, attached context, and Slack-thread invariants. This spec
closes the remaining parent gap by making the boundary explicit and testable.

## Objectives

- Keep Runx core generic: one proposal envelope, one action/admission spine, no
  support/alert/outreach/product-signal platform families in Runx.
- Keep Nitrosend product-owned: provider UX, source policy, route maps,
  support/customer context, and copy remain outside Runx core.
- Normalize provider UI aliases before they reach durable/workflow
  `action_intent`. Slack may use `triage`, `intake`, and `promote` as button or
  command modes, but the durable Runx-facing action intents are `check`,
  `issue-intake`, and `issue-to-pr`.
- Preserve the original issue-intake to issue-to-PR golden path: originating
  source thread/event -> hydrated context -> optional check -> create/update
  tracking item -> optional build-fix change request without requiring a prior
  check -> human final-change gate -> final outcome posted back.
- Pin live dogfood artifacts with an executable verifier so the source-thread,
  receipt, gate, and safety invariants cannot drift silently.

## Invariant

- Runx core must not hardcode Nitrosend channels, owners, account context,
  labels, project boards, source-specific copy, Slack action ids, or customer
  semantics.
- Nitrosend must not duplicate Runx public contracts. It consumes central Runx
  refs and emits product-local source/action context around them.
- A proposal is reviewable state, not permission to mutate. Customer send,
  final change approval, billing/account mutation, and destructive provider
  mutation always remain separate human-approved gates.
- Every admitted Nitrosend source must produce an intake event and one terminal
  outcome: `action`, `gate`, `send`, `outcome`, `no_action`, or `error`.
- Provider replies default to the originating source thread. Root-channel posts
  are not allowed for these dogfood artifacts.
- The durable/workflow `action_intent` surface is canonical Runx vocabulary.
  Provider-specific aliases are adapter inputs only.

## Scope

In scope:

- Runx parent-spec reconciliation:
  - replace stale tracking-parent wording with the completed cross-repo shape;
  - record that `runx-operational-contracts-v1`,
    `runx-operational-proposal-composition-v1`, and
    `runx-operational-story-outbox-v1` are completed dependencies;
  - record the Nitrosend boundary and exact acceptance checks.
- Nitrosend boundary hardening:
  - add `IssueIntake::ActionIntent` as the single adapter-to-Runx action
    normalizer;
  - update Nitrosend durable/workflow paths to canonicalize `action_intent` in
    `IssueIntake::ThreadRecord`, `IssueIntake::RunxWorkflowInputs`, and
    `IssueIntake::SlackRunxDeduper`;
  - keep Slack payload/button modes unchanged as provider UI;
  - add focused Rails specs for the action normalizer, ledger keys, workflow
    inputs, Slack payloads, and interaction paths.
- Nitrosend dogfood verification:
  - add `scripts/verify-operational-intelligence-dogfood.mjs`;
  - make no-action, escalation, and outreach dogfood artifacts explicitly record
    source-thread story fields.

Out of scope:

- New Runx runtime features.
- Auto-sending customer messages.
- Auto-merging PRs.
- Billing/account/provider destructive mutations.
- Rebuilding hosted GitHub Actions dispatch while billing blocks workflow
  dispatch.
- New Nitrosend source channels or new provider apps.

## Dependencies

- Completed Runx specs:
  - `.scafld/specs/archive/2026-05/runx-operational-contracts-v1.md`;
  - `.scafld/specs/archive/2026-05/runx-operational-proposal-composition-v1.md`;
  - `.scafld/specs/archive/2026-05/runx-operational-story-outbox-v1.md`.
- Completed Nitrosend integration:
  - `/Users/kam/dev/nitrosend/.scafld/specs/archive/2026-05/nitrosend-operational-intelligence-integration-v1.md`;
  - `/Users/kam/dev/nitrosend/.scafld/dogfood/*.json`.
- Nitrosend source files:
  - `/Users/kam/dev/nitrosend/api/app/services/issue_intake/action_intent.rb`;
  - `/Users/kam/dev/nitrosend/api/app/models/issue_intake/thread_record.rb`;
  - `/Users/kam/dev/nitrosend/api/app/services/issue_intake/runx_workflow_inputs.rb`;
  - `/Users/kam/dev/nitrosend/api/app/services/issue_intake/slack_action_payload.rb`;
  - `/Users/kam/dev/nitrosend/api/app/services/issue_intake/slack_runx_deduper.rb`;
  - `/Users/kam/dev/nitrosend/scripts/verify-operational-intelligence-dogfood.mjs`.

## Assumptions

- Slack action ids and signed payload modes are provider UI details; they do not
  define the durable cross-repo action contract.
- Existing GitHub workflow `mode` values may remain `triage`, `intake`, and
  `promote` while `action_intent` carries the canonical Runx action.
- Existing dogfood artifacts are historical evidence and may be shape-corrected
  when the correction only records already-proven facts, such as source-thread
  story publication fields.
- Hosted workflow dispatch remains a deployment/platform blocker, not a
  contract blocker.

## Risks

- **Alias leakage.** Slack aliases could re-enter durable state. Mitigation:
  single Nitrosend normalizer plus specs that assert canonical ledger keys and
  workflow inputs.
- **Product leakage into Runx.** Nitrosend owner/channel/customer concepts could
  creep into Runx. Mitigation: this parent records Runx as generic only and uses
  central references/proposal kind metadata.
- **Dogfood artifact drift.** Live proofs could lose receipts, source-thread
  refs, or safety fields. Mitigation: dedicated dogfood verifier.
- **Hidden authority creep.** Proposal or draft state could be mistaken for send
  or merge authority. Mitigation: verifier asserts customer_send=false,
  auto_merge=false, and human gates on support/build/golden-path artifacts.

## Acceptance

Profile: strict

Validation:
- `scafld validate runx-operational-intelligence-action-layer-v1`
- `for s in runx-operational-contracts-v1 runx-operational-proposal-composition-v1 runx-operational-story-outbox-v1; do scafld status "$s" --json | node -e "let s='';process.stdin.on('data',d=>s+=d).on('end',()=>{const r=JSON.parse(s).result;if(r.status!=='completed')process.exit(1)})" || exit 1; done`
- `cd /Users/kam/dev/nitrosend/api && bin/bundle exec rspec spec/services/issue_intake/action_intent_spec.rb spec/models/issue_intake/thread_record_spec.rb spec/services/issue_intake/runx_workflow_inputs_spec.rb spec/services/issue_intake/slack_action_payload_spec.rb spec/services/issue_intake/slack_command_router_spec.rb spec/services/issue_intake/slack_interaction_router_spec.rb spec/services/issue_intake/slack_reaction_intake_spec.rb`
- `cd /Users/kam/dev/nitrosend && node scripts/verify-operational-intelligence-dogfood.mjs`
- `git diff --check -- .scafld/specs/drafts/runx-operational-intelligence-action-layer-v1.md`
- `cd /Users/kam/dev/nitrosend/api && git diff --check -- app/models/issue_intake/thread_record.rb app/services/issue_intake/action_intent.rb app/services/issue_intake/runx_workflow_inputs.rb app/services/issue_intake/slack_action_payload.rb app/services/issue_intake/slack_runx_deduper.rb spec/services/issue_intake/action_intent_spec.rb spec/models/issue_intake/thread_record_spec.rb spec/services/issue_intake/runx_workflow_inputs_spec.rb spec/services/issue_intake/slack_command_router_spec.rb`
- `cd /Users/kam/dev/nitrosend && git diff --check -- scripts/verify-operational-intelligence-dogfood.mjs .scafld/dogfood/nitrosend-alert-no-action.json .scafld/dogfood/nitrosend-escalation-proposal.json .scafld/dogfood/nitrosend-outreach-proposal.json`

## Phase 1: Boundary Reconciliation

Status: completed
Dependencies: completed Runx operational child specs

Objective: Make the parent spec reflect the actual shipped shape instead of

Changes:
- Record the completed Runx child specs as dependencies, not draft children.
- Record Nitrosend as the live consuming application proof, not a Runx core lane.
- Record hosted workflow dispatch as explicitly out of scope for this boundary lock.

Acceptance:
- [x] `p1_ac1` command - Parent validates.
  - Command: `scafld validate runx-operational-intelligence-action-layer-v1`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `p1_ac2` command - Runx children are completed.
  - Command: `for s in runx-operational-contracts-v1 runx-operational-proposal-composition-v1 runx-operational-story-outbox-v1; do scafld status "$s" --json | node -e "let s='';process.stdin.on('data',d=>s+=d).on('end',()=>{const r=JSON.parse(s).result;if(r.status!=='completed')process.exit(1)})" || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Nitrosend Action Intent Normalization

Status: completed
Dependencies: Phase 1

Objective: Keep provider UI aliases out of the durable/workflow action layer.

Changes:
- Add `IssueIntake::ActionIntent`.
- Normalize `triage -> check`, `intake -> issue-intake`, and `promote -> issue-to-pr` before dedupe, durable ledger state, and workflow input creation.
- Keep Slack signed payload modes unchanged so existing button/action ids remain usable.

Acceptance:
- [x] `p2_ac1` command - Focused Rails specs pass.
  - Command: `cd /Users/kam/dev/nitrosend/api && bin/bundle exec rspec spec/services/issue_intake/action_intent_spec.rb spec/models/issue_intake/thread_record_spec.rb spec/services/issue_intake/runx_workflow_inputs_spec.rb spec/services/issue_intake/slack_action_payload_spec.rb spec/services/issue_intake/slack_command_router_spec.rb spec/services/issue_intake/slack_interaction_router_spec.rb spec/services/issue_intake/slack_reaction_intake_spec.rb`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-12

## Phase 3: Dogfood Evidence Lock

Status: completed
Dependencies: Phase 2

Objective: Pin the live operational-intelligence scenarios with executable

Changes:
- Add `scripts/verify-operational-intelligence-dogfood.mjs`.
- Require source refs, source-thread refs, source-thread story fields, receipts, safety gates, customer-send/merge gates, tracking/change refs, and no-action closure in the dogfood artifacts.
- Shape-correct the three artifacts that had Slack publication refs but were missing explicit story fields.

Acceptance:
- [x] `p3_ac1` command - Dogfood verifier passes.
  - Command: `cd /Users/kam/dev/nitrosend && node scripts/verify-operational-intelligence-dogfood.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17

## Phase 4: Close

Status: completed
Dependencies: Phase 3

Objective: Prove the cross-repo boundary lock is clean and commit it.

Changes:
- none

Acceptance:
- [x] `p4_ac1` command - Diff whitespace checks pass.
  - Command: `git diff --check -- .scafld/specs/drafts/runx-operational-intelligence-action-layer-v1.md && cd /Users/kam/dev/nitrosend/api && git diff --check -- app/models/issue_intake/thread_record.rb app/services/issue_intake/action_intent.rb app/services/issue_intake/runx_workflow_inputs.rb app/services/issue_intake/slack_action_payload.rb app/services/issue_intake/slack_runx_deduper.rb spec/services/issue_intake/action_intent_spec.rb spec/models/issue_intake/thread_record_spec.rb spec/services/issue_intake/runx_workflow_inputs_spec.rb spec/services/issue_intake/slack_command_router_spec.rb && cd /Users/kam/dev/nitrosend && git diff --check -- scripts/verify-operational-intelligence-dogfood.mjs .scafld/dogfood/nitrosend-alert-no-action.json .scafld/dogfood/nitrosend-escalation-proposal.json .scafld/dogfood/nitrosend-outreach-proposal.json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22

## Rollback

- Runx rollback is reverting this parent spec.
- Nitrosend API rollback is reverting the `IssueIntake::ActionIntent`
  normalizer and the focused test updates.
- Nitrosend dogfood rollback is reverting the verifier script and the three
  explicit story-field additions. This does not mutate live Slack, GitHub,
  customer, billing, or provider state.

## Review

Status: completed
Verdict: pass
Mode: verify
Summary: Human-reviewed override accepted: operator-directed cross-repo boundary lock; focused acceptance passed: scafld validate, completed Runx child specs, Nitrosend RSpec 63/63, dogfood verifier, diff checks

Attack log:
- `review gate`: manual human audit -> clean (operator-directed cross-repo boundary lock; focused acceptance passed: scafld validate, completed Runx child specs, Nitrosend RSpec 63/63, dogfood verifier, diff checks)

Findings:
- none

## Self Eval

- The work is a boundary lock and small adapter cleanup, not a new operational
  platform. The Runx core remains generic; Nitrosend remains the product layer.

## Metadata

Labels: runx, nitrosend, operational-intelligence, cross-repo, action-layer
