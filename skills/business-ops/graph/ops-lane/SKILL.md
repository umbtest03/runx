---
name: ops-lane
description: Deterministic fixture lane for the business-ops graph example.
source:
  type: cli-tool
  command: node
  args:
    - run.mjs
  timeout_seconds: 10
  sandbox:
    profile: readonly
    cwd_policy: skill-directory
inputs:
  lane:
    type: string
    required: true
    description: Lane name to project.
  signal:
    type: string
    required: true
    description: Business signal being routed.
---

Project one business-ops lane as a deterministic fixture packet.
