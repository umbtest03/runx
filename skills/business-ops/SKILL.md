---
name: business-ops
description: Basic business operations graph; route one signal through governed docs, release, issue, send, spend, and audit lanes.
runx:
  category: ops
---

# Business Ops

Run one business signal through a basic governed operations graph.

`business-ops` is intentionally small and deterministic. It does not call
private providers, mutate a project, send messages, or move money. It shows how
a single business signal can be classified, routed through bounded lanes,
stopped at approval where appropriate, and sealed as a graph receipt.

Real teams replace the fixture lane steps with their own skills, policies,
provider tools, approval gates, and verification checks.

## What this skill does

- Classifies a business signal into an operations route.
- Fans the signal out through representative governed lanes: documentation,
  release preparation, issue-to-PR, outbound draft, spend quote, and receipt
  audit.
- Marks which lanes are read-only, which require approval, and which should stop
  before consequential authority such as send, spend, deploy, publish, or merge.
- Produces receipt-backed lane packets so the operator can see what was routed
  and why.

## When to use this skill

- To demonstrate how runx models business operations as composable governed
  lanes.
- To prototype a team-specific ops graph before wiring private provider tools.
- To explain how an agent can route work without gaining ambient authority.
- To smoke-test graph execution, child receipts, and approval boundaries with no
  external account.

## When not to use this skill

- To run a real production launch, incident, release, customer send, or spend
  flow without replacing the fixture lanes.
- To claim that docs were written, a release was prepared, a PR was opened, a
  customer was contacted, or money moved.
- To bypass approval gates. The send, spend, publish, deploy, and merge lanes
  are represented as stops, not completed actions.
- To hide provider state, credentials, customer lists, or private project policy
  in the signal.

## Procedure

1. Receive one `signal` that names the business situation to triage.
2. Run the classify lane and choose representative governed lanes.
3. Project documentation, release, issue, send, spend, and audit lane packets.
4. Mark each lane with the decision and approval posture.
5. Seal the graph receipt with child receipts for each lane.
6. In a real project, replace fixture lanes with project-owned skills, provider
   tools, policies, and readback checks.

## Edge cases and stop conditions

- Return `needs_input` when the signal is missing or too vague to route.
- Return `needs_more_evidence` when a real project graph lacks required project
  context, provider readback, receipt refs, or policy.
- Return `needs_agent` when a lane requires human or model judgment that the
  fixture cannot provide.
- Return `refused` for requests that try to bypass approval, hide consequential
  side effects, or claim completed provider work without proof.
- Return `escalated` for legal, financial, security, customer-impacting, or
  irreversible actions outside the supplied authority.

## Output schema

The graph output contains:

- `graph`: `business-ops`
- `graph_status`: graph completion status
- `steps`: ordered child step summaries with receipt ids
- `step_outputs`: lane packets keyed by step id

Each lane packet contains:

- `lane`: the lane name
- `signal`: the original business signal
- `decision`: route, prepare, draft, quote, verify, or a stop decision
- `summary`: what the lane would do
- `approval`: whether approval is required
- `next`: follow-up gate, command, or verification surface

## Worked example

Input:

```bash
runx skill business-ops \
  -i signal="launch readiness for API v2: docs, release, customer comms, and spend checks" \
  --json
```

The graph routes the signal through docs, release, issue, send, spend, and audit
lanes. Docs and release prepare bounded packets. Send and spend stop at approval
gates. Audit names the receipt/history checks that prove what happened.

## Inputs

- `signal` (required): a concise business operations signal to classify and
  route.
