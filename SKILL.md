---
name: spam-risk-reviewer
description: Evaluates email campaigns for spam risk based on list health, sender auth, and content.
---

# Spam Risk Reviewer

This skill evaluates email campaign attributes and determines if the campaign is safe to send.

- Receives campaign draft details, list metadata, and sender auth posture.
- Determines a risk level (e.g., `pass`, `hold`, `block`).
- Checks if DKIM or SPF fails, or if bounce rates exceed policies.
- Outputs `send_risk_verdict` with blockers and a decision.
- Escalates via `needs_human` flag if risk is high.
- **Never emit any operational proposal**.
