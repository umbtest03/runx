---
name: pay-reserve
description: Emit a cap-exceeded x402 reservation for authority-admission refusal.
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

Emit a deterministic reservation packet whose spend binding exceeds the
reserved child cap. The rail must not be invoked.
