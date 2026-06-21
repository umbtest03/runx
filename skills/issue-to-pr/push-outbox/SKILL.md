---
name: issue-to-pr-push-outbox
version: 0.1.0
description: Publish issue-to-PR outbox entries through the governed Rust thread-outbox-provider front.
runx:
  category: code
source:
  type: thread-outbox-provider
  thread_outbox_provider:
    operation: push
    manifest_path: manifest.json
---
# Issue-to-PR Outbox Publisher

Publishes issue-to-PR outbox entries through the governed Rust provider front.
The Rust adapter constructs the provider frame from graph inputs and supervises
the provider process, credential delivery, redaction, and sealed observation.
