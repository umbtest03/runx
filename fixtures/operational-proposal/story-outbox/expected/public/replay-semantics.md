# Replay Semantics

idempotency key: source id, provider, source thread ref, workflow id, lane id, milestone id, target ref, proposal id, and content hash.

content hash: normalized public markdown only, excluding private receipt bodies and raw provider payloads.

same-key replay: update or reuse the existing publication_refs entry and preserve locator/comment metadata.

different milestones: create distinct outbox entries so human_gate and final_outcome do not collide.

legacy_published_refresh: published legacy entries refresh into canonical entries only during migration lookup.
preserves_comment_id: true
preserves_locator: true
preserves_receipt_ref: true
writes_canonical_milestone_id: true
no_duplicate_comment: true
