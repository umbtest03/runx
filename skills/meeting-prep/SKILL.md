---
name: meeting-prep
description: A skill to synthesize a meeting brief from various contextual inputs.
source:
  type: cli-tool
  command: node
  args:
    - run.mjs
  timeout_seconds: 30
  sandbox:
    profile: readonly
    cwd_policy: skill-directory
inputs:
  event_context:
    type: string
    required: true
  provided_notes:
    type: string
  thread_snippets:
    type: string
  public_link_notes:
    type: string
  expected_version:
    type: string
  idempotency_key:
    type: string
runx:
  category: ops
  input_resolution:
    required:
      - event_context
---

# meeting-prep

This skill generates a meeting brief synthesizing inputs (notes, thread snippets, public link notes) and correctly attributing them.
