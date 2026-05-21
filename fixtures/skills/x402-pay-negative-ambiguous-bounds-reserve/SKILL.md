---
name: pay-reserve
description: Refuse ambiguous x402 quote bounds without issuing a reservation.
source:
  type: cli-tool
  command: sh
  args:
    - ./run.sh
  timeout_seconds: 10
  sandbox:
    profile: readonly
    cwd_policy: skill-directory
inputs: {}
---

Emit a deterministic governed refusal for ambiguous x402 bounds.
