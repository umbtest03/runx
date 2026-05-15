# Developer Issue Inbox

The runx developer inbox is a work-item queue, not another chat stream. Slack,
Sentry, GitHub, file, and API adapters may submit source events, but every
accepted event must become one `runx.work_item.v1` packet with an explicit
state, dedupe fingerprint, triage action, and source-thread locator.

## Admission Policy

Adapters must evaluate source policy before invoking `issue-intake`. A message
containing a word such as `BUG` is not enough to trigger mutation.

Required policy fields:

- `source_provider`: `slack`, `sentry`, `github`, `file`, `api`, or `other`
- `source_locator`: stable provider locator for the event or thread
- `thread_locator`: reply target for gate updates when available
- `allowed_sources`: channel, project, repo, or integration allowlist
- `event_kind`: provider-native event type such as message, alert, issue, or comment
- `fingerprint_strategy`: how the adapter builds the SHA-256 dedupe fingerprint
- `minimum_confidence`: minimum triage confidence for `issue-to-pr`
- `allowed_actions`: subset of `reply-only`, `issue-intake`, `work-plan`, `issue-to-pr`, `manual-review`
- `owner_suggestion_policy`: optional repo-local rule for owner suggestions
- `target_repo_policy`: optional repo-local rule for target repository suggestions

Non-trigger cases:

- general support chatter without a reproducible issue
- Slack keywords outside an allowlisted source
- Sentry alerts below configured frequency or severity thresholds
- reports missing a stable source locator or dedupe fingerprint
- ambiguous requests that need a human target decision
- duplicate events already attached to an open work item

## Queue States

Developer views should group by `work_item_id` and show the next useful gate:

- needs triage: `intake_received`, `dedupe_pending`, `triage_pending`
- needs evidence: `blocked`
- ready to plan: `planning_ready`
- ready to build: `build_ready`
- review running or failed: `review_ready`
- PR ready: `pr_ready`
- waiting for human merge: `merge_gate`
- done: `outcome_merged`, `outcome_closed`, `outcome_rejected`

List views should stay compact: id, state, status summary, source, dedupe
fingerprint, triage action, issue/PR refs, duplicate counts, latest transition,
and timestamps. Full receipts and ledger artifacts remain the evidence layer.

## Routing

runx core may store suggested owner and target repository metadata, but it must
not hardcode people, Slack channels, Sentry projects, or customer repository
names. Consuming repos own those policies.

`issue-intake` chooses the next lane:

- `reply-only`: answer or support guidance, no mutation
- `manual-review`: human decision needed before planning or mutation
- `work-plan`: bigger change, planning first
- `issue-to-pr`: bounded fix, governed PR lane

`issue-to-pr` must preserve the same `work_item` packet through PR packaging
and source-thread story updates. It must stop at the human merge gate.
