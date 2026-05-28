# source_thread_update

source_ref: support://case/ops-123
source_thread_ref: provider://workspace/thread/ops-123
result_refs: tracking_item=track://issue/77 change_request=change://pr/88
publication_refs: source_thread_update tracking_item_comment change_request_comment

- accepted: Source request accepted for governed runx handling.
- hydrated: Source context summarized from private receipt artifact refs.
- triaged: Decision and rationale are safe for the public story.
- reply_drafted: A concise reply draft is ready for human review.
- ask_for_info: The next human action is to provide the missing account-safe detail.
- proposal_ready: Outreach proposal ready from proposal_kind without using it as a milestone id.
- escalation_proposed: Dev escalation proposed from proposal_kind without accepting a domain id.
- tracking_item_created: Tracking item track://issue/77 created for follow-up.
- spec_ready: Governed spec is ready.
- build_started: Build evidence is being collected.
- review_requested: Adversarial review requested.
- change_request_created: Change request change://pr/88 created.
- review_fixup: Review fixup requested with safe finding summary.
- human_gate: Human final-change gate is required.
- outcome_observed: Provider outcome observed from public status only.
- final_outcome: Final outcome links source, tracking item, and change request.
- no_action: No action needed; rationale is public-safe.
- monitor: Monitor the source thread for a provider outcome.
