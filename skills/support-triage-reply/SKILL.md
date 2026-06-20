---
name: support-triage-reply
version: 0.1.0
description: Classify a bounded support request, choose the safe next path, and draft a customer-ready reply only when a human-gated send is appropriate.
source:
  type: cli-tool
  command: node
  args:
    - run.mjs
links:
  source: https://github.com/runxhq/runx/tree/main/oss/skills/support-triage-reply
runx:
  category: ops
  input_resolution:
    required:
      - support_request
---

# Support Triage Reply

Classify one bounded support request and return a support-safe decision packet.
The skill is designed for day-to-day operator work where support, product, and
engineering signals arrive together, but a customer send must remain a separate
human-approved action.

This skill never sends email, posts to Slack, creates issues, mutates accounts,
or touches billing. It returns a draft and a gated send proposal only when the
request is safe to answer from the supplied context.

## Inputs

- `support_request`: object with `subject`, `body`, optional `customer_name`,
  optional `customer_email`, optional `source`, and optional `refs`.
- `policy`: optional object with `product_name`, `support_signature`,
  `safe_reply_topics`, and `escalation_contacts`.

## Output

The runner emits these top-level fields:

- `classification`: `how_to`, `billing`, `account_access`, `bug`, `abuse`, or
  `unknown`.
- `severity`: `low`, `medium`, `high`, or `critical`.
- `confidence`: number from 0 to 1.
- `recommended_path`: one of `reply_draft`, `request_info`,
  `engineering_intake`, `billing_review`, `account_review`, `abuse_review`, or
  `manual_review`.
- `evidence`: object with matched signals, missing context, source summary, and
  taxonomy coverage.
- `draft_email`: object with `proposed`, `subject`, and `body`. When a reply is
  not safe from the supplied packet, `proposed` is `false` and `reason` explains
  the blocker.
- `send_gate`: object whose `status` is always `requires_human_approval`.

## Decision Rules

Prefer safety over completeness:

- `how_to`: draft a clear support email when the request is answerable from the
  supplied text or common product-safe instructions.
- `billing`: route to billing review unless the supplied context already names
  a public, non-account-specific policy.
- `account_access`: route to account review. Do not ask for passwords, recovery
  secrets, or private tokens.
- `bug`: route to engineering intake when the report includes a failure mode,
  product surface, or reproduction clue.
- `abuse`: route to abuse review and do not draft a customer-facing answer.
- `unknown`: request more information or manual review. Do not invent a fix.

Customer-facing copy must be specific, calm, and sendable. It should include a
greeting, acknowledge the actual request, state the answer or next step, and end
with the configured support signature. Avoid filler, unsupported promises, and
fake certainty.

## Safety Bar

- No customer send occurs inside this skill.
- No private credentials, billing records, account identifiers, or inbox state
  are required.
- A draft is a proposal, not permission. The caller must use a separate
  governed send lane to deliver any message.
- When confidence is low or private account state is needed, return
  `manual_review` or a review-specific route.
