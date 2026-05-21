---
name: pay-quote
description: Refuse a malformed x402 challenge without issuing a quote.
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

Emit a deterministic governed refusal for a malformed x402 challenge fixture.
