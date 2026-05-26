---
name: pay-fulfill-rail
description: Deterministically fulfill or partially mutate the x402 idempotency fixture rail spend.
source:
  type: cli-tool
  command: sh
  args:
    - ./run.sh
  timeout_seconds: 10
  sandbox:
    profile: workspace-write
    cwd_policy: skill-directory
    writable_paths:
      - "{{ env.RUNX_PAYMENT_RAIL_COUNT_PATH }}"
    env_allowlist:
      - RUNX_PAYMENT_RAIL_COUNT_PATH
      - RUNX_X402_IDEMPOTENCY_KEY
      - RUNX_X402_RAIL_MODE
inputs: {}
---

Emit deterministic mock rail packets for x402 idempotency replay and recovery.
