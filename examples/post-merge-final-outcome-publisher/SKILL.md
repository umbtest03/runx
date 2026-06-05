---
name: post-merge-final-outcome-publisher
description: Publish a final provider-state update through the Rust thread-outbox-provider front.
source:
  type: thread-outbox-provider
  thread_outbox_provider:
    operation: push
    manifest_path: manifest.json
    push_path: push.json
---
# Post-Merge Final Outcome Publisher

Publishes the final provider outcome back to the source thread through the
governed Rust thread-outbox-provider front.
