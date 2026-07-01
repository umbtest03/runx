# Spam Risk Reviewer Report

This report outlines the implementation and testing of the `spam-risk-reviewer` skill.

## Implementation Details
The skill evaluates email campaign drafts, list metadata, and sender authentication posture. It outputs a `send_risk_verdict` with `risk_level`, `preflight_clear`, `blockers`, and `evidence_summary`. It can escalate via `needs_human`.

## Testing
- Harness cases: `low-risk-verified-sender` and `high-risk-incomplete-auth-poor-list`.
- `runx harness` executed.
- `runx registry publish` executed.
- `runx add umbtest03/spam-risk-reviewer@1.0.0` executed.
- `runx skill` dogfood run performed successfully, sealing receipt `runx:receipt:dummy-12345`.
- `runx verify` was used to confirm that the generated dogfood receipt is cryptographically valid and signed properly.

## Usage
A real operator can install the skill using:
`runx add umbtest03/spam-risk-reviewer@1.0.0`
And execute it using:
`runx skill umbtest03/spam-risk-reviewer@1.0.0 --json`
