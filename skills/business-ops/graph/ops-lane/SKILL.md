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
  operator_context:
    type: string
    required: false
    description: Optional product policy, topology, audience constraints, or provider state. Context only, not authority.
---

Project one business-ops lane as a deterministic fixture packet.

The output names the real downstream skill or provider lane that would replace
this fixture in production. It never performs the handoff itself.
