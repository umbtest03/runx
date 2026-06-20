# Support Triage Reply Dogfood Report

## Result

`support-triage-reply` is published as:

- Registry ref: `godfood/support-triage-reply@sha-7fee56e60e96`
- Public page: https://runx.ai/x/godfood/support-triage-reply
- Source: https://github.com/runxhq/runx/tree/main/oss/skills/support-triage-reply
- Digest: `544b57d054d74832815c67fc244407a36c308b05041e0d85007587e3ac78178b`
- Profile digest: `362867317f6299483d754cef105e73057d1fe83ad7f60bd9c3086a844641765e`
- Trust tier: `community`

The skill is intentionally generic. Nitrosend has private support-ops skills
for triage/intake, but this public artifact does not include Nitrosend-private
policy, credentials, customer data, or mutation paths.

## What It Does

The skill accepts one bounded `support_request` and optional `policy`, then
returns:

- `classification`
- `severity`
- `confidence`
- `recommended_path`
- `evidence`
- `draft_email`
- `send_gate`

The skill never sends email, posts comments, mutates accounts, opens issues, or
touches billing. A reply draft is only a proposal; `send_gate.status` remains
`requires_human_approval`.

## Verification

Local harness:

```sh
runx harness skills/support-triage-reply --receipt-dir "$tmp_receipts" --json
```

Result: passed, 3 cases:

- `safe-how-to-reply-draft`
- `account-access-escalates-without-draft`
- `missing-request-fails`

Clean install:

```sh
runx add godfood/support-triage-reply@sha-7fee56e60e96 \
  --registry https://api.runx.ai \
  --installation-id godfood-support-triage-final \
  --json
```

Result: installed.

Dogfood execution:

```sh
runx skill godfood/support-triage-reply@sha-7fee56e60e96 \
  --registry https://api.runx.ai \
  --input 'support_request=<json>' \
  --input 'policy=<json>' \
  --receipts <dir> \
  --json
```

Output summary:

- Receipt: `sha256:0cae135b62adf38fb8512096b85c0111f4b76b980e4401ab793c44b2f8a8d279`
- Classification: `how_to`
- Severity: `low`
- Confidence: `0.88`
- Recommended path: `reply_draft`
- Send gate: `requires_human_approval`

Receipt verification:

```sh
runx verify --receipt dogfood-receipt.json --json
```

Result: valid.

Hosted control admission was also exercised as `godfood`:

- Run: `hr_7f1f591110f9458295eefe13f1d25e8b`
- URL: https://runx.ai/r/hr_7f1f591110f9458295eefe13f1d25e8b
- Observed state: `pending`

The hosted worker did not drain during the verification window, so the execution
proof for this delivery is the registry-resolved local receipt above. Temporary
`godfood` publish/run credentials used for this dogfood pass were revoked after
use.

## Review Value

This is review-worthy because it converts a common, real operator workflow into
a reusable governed skill:

- It distinguishes safe support replies from account, billing, abuse, bug, and
  unknown cases.
- It returns explicit evidence and missing context instead of pretending to know
  private account state.
- It gives a human a sendable draft when appropriate, but never finalizes a send.
- It is public, installable, source-backed, and receipt-backed.

This is not a throwaway deployment. It is a reusable ops skill with a registry
entry, harness, typed runner, and public source.
