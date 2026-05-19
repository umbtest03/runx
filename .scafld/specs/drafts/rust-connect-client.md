---
spec_version: '2.0'
task_id: rust-connect-client
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T08:13:29Z'
status: draft
harden_status: passed
size: medium
risk_level: high
---

# Rust connect client

## Current State

Status: draft
Current phase: hardened
Next: approve after scaffold/tool-catalog cutover lands
Reason: draft created under `plans/rust-takeover.md`. Covers `runx connect`
(grants and the currently implemented hosted OAuth polling flow).
Blockers: scaffold/tool-catalog cutover establishes the canonical Rust CLI
dispatch path. Non-hosted flows are blocked inside this spec until their cloud
fixtures exist.
Allowed follow-up command: `scafld approve rust-connect-client`
Latest runner update: none
Review gate: passed

## Summary

Port the connect surface (grants and the currently observed Nango-hosted
OAuth polling flow) to Rust. Today this lives in
`packages/cli/src/commands/connect.ts` and `packages/cli/src/connect-http.ts`,
with cloud-side at `cloud/packages/auth/`. The Rust port consumes the same
cloud HTTP contract.

Hardening note: the current OSS TypeScript client only implements the hosted
HTTP grant/session client and browser opener. Browser callback, device flow,
credential intake modes, and cloud auth server details are not present in this
repo. The first buildable Rust slice must therefore preserve the current
contract exactly and gate any wider flow behind explicit fixtures and a cloud
contract update. This is a hard cutover spec: after the Rust implementation is
enabled, `runx connect` must not dispatch to the TypeScript client as a
fallback path.

## Context

CWD: `.`

Read-only reference files:
- `packages/cli/src/commands/connect.ts`
- `packages/cli/src/connect-http.ts`
- `packages/cli/src/args.ts`
- `packages/cli/src/dispatch.ts`
- `packages/core/src/policy/authority-proof.ts`
- `packages/core/src/policy/index.ts`
- `crates/runx-core/src/policy/connected_auth.rs`
- `docs/security-authority-proof.md`
- `docs/ts-interop-boundary.md`

Cloud-side reference only, not part of this implementation:
- `cloud/packages/auth/src/connect-html.ts`

Precise implementation impact files after scaffold/tool-catalog cutover:
- `crates/runx-runtime/src/connect/mod.rs`
- `crates/runx-runtime/src/connect/client.rs`
- `crates/runx-runtime/src/connect/types.rs`
- `crates/runx-runtime/src/connect/flows.rs`
- `crates/runx-runtime/src/connect/redaction.rs`
- `crates/runx-runtime/src/connect/opener.rs`
- `crates/runx-runtime/src/connect/mock_server.rs` or
  `crates/runx-runtime/tests/support/connect_mock.rs`
- `crates/runx-runtime/tests/connect_client.rs`
- `crates/runx-runtime/tests/connect_cli.rs`
- `crates/runx-runtime/tests/connect_secret_redaction.rs`
- `crates/runx-runtime/tests/connect_policy_integration.rs`
- `fixtures/connect/README.md`
- `fixtures/connect/contracts/hosted-http-v1/*.json`
- `fixtures/connect/oracles/*.stdout`
- `fixtures/connect/oracles/*.stderr`
- `fixtures/connect/oracles/*.json`
- `fixtures/connect/oracles/*.receipt.json`

Conditional impact files, only if the scaffold/tool-catalog cutover has already
made these the canonical Rust CLI dispatch points:
- `crates/runx-runtime/src/cli/connect.rs`
- `crates/runx-runtime/src/cli/args.rs`
- `crates/runx-runtime/src/cli/dispatch.rs`
- `crates/runx-runtime/src/lib.rs`

Do not modify in this spec execution:
- `packages/cli/src/**`, except deleting the TypeScript connect dispatch only
  after the approved Rust CLI hard cutover spec owns that deletion.
- `packages/core/src/**`.
- `cloud/packages/auth/**`.
- Package manifests, lockfiles, release scripts, or Scafld state directories
  unless a prior scaffold/tool-catalog spec explicitly assigns them.

Invariants:
- Connect is authority intake, not skill execution. It creates, lists, and
  revokes grant descriptors that later feed harness authority admission; it
  does not introduce harness, act, decision, or receipt object families.
- Connect tokens never land in receipts or logs as plaintext.
- Hosted access tokens, OAuth codes, device codes, refresh tokens, API keys,
  passwords, client secrets, Basic credentials, JWT signing material, and
  provider credential bodies never land in stdout, stderr, JSON command output,
  receipts, fixture goldens, trace spans, panic payloads, or error strings.
- The `connect` verb is user-facing; `grant` is the internal object.
- Auto-connect on first skill use behavior matches TS for the hosted HTTP
  contract.
- Browser polling flow behaves the same way across languages. Device flow is
  intentionally blocked until a versioned cloud fixture exists.
- The Rust client must not introduce a local credential storage backend in the
  connect command. Persistent credential material remains cloud-owned; local
  runtime receives only grant descriptors during admission and opaque credential
  envelopes after admission.
- Grant descriptors remain authority inputs. The Rust structs must preserve the
  exact fields required by the authority partial order: provider, scopes,
  optional scope family, authority kind, target repo, target locator, grant id,
  status, and connection id.
- Grant matching must preserve exact TS/Rust policy semantics: provider match,
  non-revoked status, exact or `:*`/`*` scope allowance, and exact grant
  reference match for `scope_family`, `authority_kind`, `target_repo`, and
  `target_locator`.

## Objectives

- Port the hosted grant/session HTTP client and Nango-hosted OAuth polling flow
  observed in TypeScript.
- Keep device flow, callback listener, and BYO credential intake unavailable
  until their cloud contracts are versioned with fixtures. Reject attempted use
  with a redacted, non-secret diagnostic.
- Add fixture suite for each flow against a deterministic auth-server mock.
- Preserve current CLI behavior for:
  - `runx connect list`
  - `runx connect revoke <grant-id>`
  - `runx connect <provider> --scope ... --scope-family ... --authority-kind
    read_only|constructive|destructive --target-repo ... --target-locator ...`
  - `--json` shape: `{ "status": "success", "connect": <cloud-response> }`
- Preserve current hosted-service configuration:
  - `RUNX_CONNECT_BASE_URL`
  - `RUNX_CONNECT_ACCESS_TOKEN`
  - `RUNX_CONNECT_OPEN_COMMAND`
  - `RUNX_CONNECT_POLL_INTERVAL_MS`
  - `RUNX_CONNECT_TIMEOUT_MS`
- Remove TypeScript fallback dispatch for `runx connect` only when the
  scaffold/tool-catalog work has made the Rust CLI path canonical. The cutover
  must be atomic: either all listed connect commands are Rust-backed, or the
  spec remains blocked.

## Non-Negotiable Cutover Rules

- No fallback shim from Rust back to `packages/cli/src/connect-http.ts`.
- No dual-read or dual-write behavior for grants or credential material.
- No local credential persistence backend.
- No local callback server.
- No device flow or BYO credential intake implementation without checked-in
  contract fixtures and explicit new acceptance gates.
- No logging of authorization URLs, bearer headers, OAuth/device codes, API
  keys, passwords, client secrets, JWT material, raw credential envelopes, or
  raw cloud error bodies.
- Unknown cloud `status` values are hard errors. They must not be coerced into
  pending, failed, or success.
- No parallel connect receipt or authority-proof contract. If implementation
  touches receipt-writing code, it may only add leak gates around the canonical
  harness receipt path.

## Build-Ready HTTP Contract

The initial Rust client is build-ready only for the TypeScript-observed hosted
HTTP contract below. It must reject or feature-gate any unmodeled status or
field until cloud fixtures define it.

Requests:
- `GET {base_url}/v1/grants`
  - Headers: `Authorization: Bearer <RUNX_CONNECT_ACCESS_TOKEN>`,
    `Accept: application/json`, `Content-Type: application/json`.
  - Response: `{ "grants": HttpConnectGrant[] }`.
- `POST {base_url}/v1/connect/sessions`
  - Body: `HttpConnectPreprovisionRequest`.
  - Response: `HttpConnectStartReadyResponse` or
    `HttpConnectStartOauthResponse`.
- `GET {base_url}/v1/connect/sessions/{flow_id}`
  - Polls an OAuth-required session.
  - Response: `HttpConnectStartReadyResponse`,
    `HttpConnectFlowPendingResponse`, or `HttpConnectFlowFailedResponse`.
- `DELETE {base_url}/v1/grants/{grant_id}`
  - Response: `HttpConnectRevokeResponse`.

Types:
- `HttpConnectGrant`: `grant_id`, optional `principal_id`, `provider`,
  `scopes`, optional `scope_family`, optional `authority_kind`, optional
  `target_repo`, optional `target_locator`, optional `connection_id`, `status`
  (`active` or `revoked`), optional `created_at`.
- `HttpConnectPreprovisionRequest`: `provider`, `scopes`, optional
  `scope_family`, optional `authority_kind`, optional `target_repo`, optional
  `target_locator`.
- Ready response: `status` is `created` or `unchanged`, plus `grant`.
- OAuth-required response: `status: "oauth_required"`, `flow_id`,
  `authorize_url`, optional `poll_after_ms`, optional `expires_at`.
- Pending response: `status: "pending"`, `flow_id`, optional `poll_after_ms`.
- Failed response: `status: "failed"`, `flow_id`, `error`.
- Revoke response: `status: "revoked"`, plus `grant`.

HTTP failure handling:
- Parse JSON response bodies only as JSON; do not string-interpolate raw bodies
  into errors unless they pass the redactor first.
- Prefer cloud `error` fields after redaction; otherwise emit compact
  `HTTP <status>` or byte-count diagnostics.
- Timeout defaults to 60 seconds for OAuth polling, with per-poll delay from
  response `poll_after_ms`, initial `poll_after_ms`, configured interval, then
  750 ms fallback.

Expected request details:
- All JSON request bodies must be serialized from typed structs, not hand-built
  strings.
- `Authorization` is required for every hosted HTTP request. Missing
  `RUNX_CONNECT_ACCESS_TOKEN` must fail before any network attempt.
- `RUNX_CONNECT_BASE_URL` must be parsed as a URL, normalized without logging,
  and rejected if it is missing or invalid.
- Non-2xx responses must not be deserialized as success envelopes.
- Redirects are not part of the client contract. Do not follow redirects unless
  the stabilized cloud fixture explicitly requires them.

## Browser, Callback, and Device Flow Boundaries

Current TS behavior:
- For `oauth_required`, open `authorize_url` with `RUNX_CONNECT_OPEN_COMMAND`,
  macOS `open`, Windows `cmd /c start`, or `xdg-open`.
- The opener receives `RUNX_CONNECT_URL`.
- The CLI then polls `/v1/connect/sessions/{flow_id}` until ready, failed, or
  timeout.

Rust requirements:
- Keep the first implementation poll-based. Do not add a local HTTP callback
  listener until the cloud contract defines callback URL registration, state
  validation, CSRF handling, listener bind host/port, and success/failure page
  shape.
- Device flow is blocked until cloud fixtures define `device_code`,
  `user_code`, `verification_uri`, `verification_uri_complete`, interval,
  expiry, pending, slow-down, denied, and expired statuses. When added, device
  and user codes must be treated as secrets in logs and receipts.
- Shell opener support must avoid logging the URL. If opener execution fails,
  report the command/process name and exit code only, never the URL.

## Config and Storage Boundaries

- `RUNX_CONNECT_ACCESS_TOKEN` is process-secret configuration only. It must not
  be persisted to runx config files, default receipt dirs, fixture output, or
  command JSON.
- `RUNX_CONNECT_BASE_URL` may be persisted as non-secret configuration only if
  the Rust config spec allows it; otherwise keep parity with env-only TS
  configuration.
- Grant descriptors are non-secret enough for admission and user display, but
  they must not contain raw credential material. `connection_id` and grant ids
  are opaque identifiers; do not derive storage paths or credential filenames
  from unvalidated server values.
- Credential envelopes resolved for execution must remain behind the authority
  boundary. Receipts may include `material_ref_hash` and grant references, but
  never raw `material_ref` if it could identify credential material more
  strongly than the existing authority-proof contract permits.
- Fixture servers may use synthetic tokens and codes, but tests must assert the
  real secret values do not appear in stdout, stderr, JSON output, receipts, or
  failure messages.

## Fixture and Oracle Plan

Required fixture contract directory:
- `fixtures/connect/contracts/hosted-http-v1/list-active.json`
- `fixtures/connect/contracts/hosted-http-v1/list-empty.json`
- `fixtures/connect/contracts/hosted-http-v1/revoke-active.json`
- `fixtures/connect/contracts/hosted-http-v1/start-created.json`
- `fixtures/connect/contracts/hosted-http-v1/start-unchanged.json`
- `fixtures/connect/contracts/hosted-http-v1/start-oauth-required.json`
- `fixtures/connect/contracts/hosted-http-v1/poll-pending.json`
- `fixtures/connect/contracts/hosted-http-v1/poll-created.json`
- `fixtures/connect/contracts/hosted-http-v1/poll-failed.json`
- `fixtures/connect/contracts/hosted-http-v1/error-json.json`
- `fixtures/connect/contracts/hosted-http-v1/error-non-json.txt`
- `fixtures/connect/contracts/hosted-http-v1/unknown-status.json`

Required oracle outputs:
- `fixtures/connect/oracles/list-empty.stdout`
- `fixtures/connect/oracles/list-active.stdout`
- `fixtures/connect/oracles/list-active.json`
- `fixtures/connect/oracles/revoke-active.stdout`
- `fixtures/connect/oracles/revoke-active.json`
- `fixtures/connect/oracles/start-created.stdout`
- `fixtures/connect/oracles/start-created.json`
- `fixtures/connect/oracles/oauth-poll-created.stdout`
- `fixtures/connect/oracles/oauth-poll-created.json`
- `fixtures/connect/oracles/pending-timeout.stderr`
- `fixtures/connect/oracles/failed.stderr`
- `fixtures/connect/oracles/opener-failure.stderr`
- `fixtures/connect/oracles/redaction.receipt.json`

Oracle rules:
- Fixture secrets must use unique sentinel values with stable prefixes, for
  example `SECRET_CONNECT_ACCESS_TOKEN_DO_NOT_LEAK`,
  `SECRET_AUTHORIZE_QUERY_DO_NOT_LEAK`, and
  `SECRET_CREDENTIAL_BODY_DO_NOT_LEAK`.
- Oracles must assert absence as well as presence: each CLI/integration test
  scans stdout, stderr, JSON output, receipt output, trace output, panic output,
  and formatted error strings for every sentinel.
- Mock server tests must assert exact method, path, auth header presence,
  content type, accept header, JSON body, poll order, and revoke target id.
- If an oracle update is required, the implementation must explain the behavior
  change in the spec runner notes before refreshing expected files.

## Scope

In scope:
- Connect client + flow handlers.
- CLI argument parity and human/JSON rendering parity for the existing connect
  commands.
- Redaction and secret-type wrappers around all token, code, credential, and
  authorization-header values.
- Deterministic mock-server fixtures for every observed HTTP status and error
  path.
- Rust CLI cutover for `runx connect` only if the scaffold/tool-catalog work
  already owns the canonical dispatch path.

Out of scope:
- Cloud-side auth/grant logic (stays TS until separate cutover).
- Credential storage backends beyond what TS supports.
- Local HTTP callback listener.
- Device flow and BYO credential intake until the cloud HTTP contract is
  versioned with fixtures for those flows.
- Any receipt writer changes beyond verifying connect output does not leak
  plaintext through the canonical harness receipt path.
- Package manifest churn unless the scaffold/tool-catalog work has already
  exposed the needed crate/module/test target.

## Dependencies

- `rust-runtime-skeleton` completed; this spec must use the established runtime
  crate layout rather than creating a second runtime boundary.
- Scaffold/tool-catalog hard cutover for Rust CLI command routing.
- Checked-in fixtures under `fixtures/connect/contracts/hosted-http-v1/` define
  the hosted connect contract for this spec. Any flow beyond the TS-observed
  `/v1/grants` and `/v1/connect/sessions` surface remains out of scope until a
  separate cloud contract spec adds fixtures for it.
- `rust-ts-interop-boundary` for the cross-language crossing reference.

## Sequencing

1. Confirm scaffold/tool-catalog has established the canonical Rust runtime/CLI
   files and test target names. If not, stop before editing runtime or CLI
   files.
2. Add typed hosted HTTP request/response models and redacted error wrappers.
3. Add the deterministic hosted connect mock and contract fixture loader.
4. Implement list, revoke, preprovision, OAuth-required opener, and polling.
5. Wire CLI rendering and JSON output to match current TypeScript behavior.
6. Add admission integration using hosted grant descriptors only.
7. Run acceptance commands and secret-leak gates.
8. Remove any obsolete TypeScript connect dispatch only under the approved Rust
   CLI cutover owner. If that owner has not landed, leave deletion blocked and
   keep this spec unexecuted.

## Gates

Pre-implementation gates:
- `rust-runtime-skeleton` is complete and the Rust runtime crate layout is
  stable.
- Scaffold/tool-catalog cutover identifies the canonical Rust CLI dispatch
  files.
- Hosted HTTP v1 fixture directory is present or created as part of this spec.
- No local callback, device flow, or BYO credential cloud fixture is required
  by the implementation plan.

Implementation gates:
- All connect HTTP envelopes are typed.
- All secret-like fields use redacted display/debug behavior.
- All opener paths pass the URL through environment only and never log it.
- Unknown status and unsupported flow tests fail closed.
- The Rust path has no callout to TypeScript connect code.

Release gates:
- Acceptance commands below pass on a clean checkout after scaffold/tool-catalog
  work.
- Secret sentinel scan passes across test artifacts.
- Manual review confirms `runx connect` has a single canonical implementation.
- Rollback instructions below are still valid after the implementation branch
  is assembled.

## Acceptance Checks

- Rust unit tests deserialize/serialize all `HttpConnect*` request and response
  shapes above and reject unknown `status` values with redacted errors.
- Mock HTTP tests cover list, revoke, created, unchanged, oauth-required,
  pending-to-created, pending-timeout, failed, non-JSON error body, JSON error
  body, and opener failure.
- Secret-leak tests seed distinct values for access token, bearer header,
  authorize URL query token, flow id, OAuth code/device code fixtures, API key,
  password, client secret, JWT material, and credential body, then assert none
  appear in stdout, stderr, JSON output, receipts, trace logs, panic text, or
  error messages.
- CLI parity tests cover `connect list`, `connect revoke`, provider
  preprovision arguments, missing service configuration, `--json` output, and
  empty-list human guidance.
- Policy integration tests prove hosted grants feed local admission as
  descriptors only and preserve grant-reference matching semantics already
  covered by `packages/core/src/policy/index.ts` and
  `crates/runx-core/src/policy/connected_auth.rs`.
- Unsupported callback, device, and BYO credential attempts fail with redacted
  diagnostics and no network side effects beyond the explicitly supported
  hosted session endpoints.

## Acceptance Commands

Do not invoke real Scafld lifecycle mutation from the connect implementation or
tests. Scafld still drives this spec lifecycle. After scaffold/tool-catalog
work lands, the implementation owner must run:

```sh
cargo test --manifest-path crates/Cargo.toml -p runx-runtime connect_client
cargo test --manifest-path crates/Cargo.toml -p runx-runtime connect_cli
cargo test --manifest-path crates/Cargo.toml -p runx-runtime connect_secret_redaction
cargo test --manifest-path crates/Cargo.toml -p runx-runtime connect_policy_integration
cargo test --manifest-path crates/Cargo.toml -p runx-core connected_auth
```

If the scaffold/tool-catalog work creates a workspace-level command target
instead of package-specific tests, replace the package selector with the
canonical target and record that substitution in the runner notes.

Required CLI oracle checks, using the deterministic mock server:

```sh
RUNX_CONNECT_BASE_URL=http://127.0.0.1:<mock-port> \
RUNX_CONNECT_ACCESS_TOKEN=SECRET_CONNECT_ACCESS_TOKEN_DO_NOT_LEAK \
cargo run --manifest-path crates/Cargo.toml -p runx-runtime -- connect list

RUNX_CONNECT_BASE_URL=http://127.0.0.1:<mock-port> \
RUNX_CONNECT_ACCESS_TOKEN=SECRET_CONNECT_ACCESS_TOKEN_DO_NOT_LEAK \
cargo run --manifest-path crates/Cargo.toml -p runx-runtime -- connect list --json

RUNX_CONNECT_BASE_URL=http://127.0.0.1:<mock-port> \
RUNX_CONNECT_ACCESS_TOKEN=SECRET_CONNECT_ACCESS_TOKEN_DO_NOT_LEAK \
cargo run --manifest-path crates/Cargo.toml -p runx-runtime -- connect revoke grant_fixture_active --json

RUNX_CONNECT_BASE_URL=http://127.0.0.1:<mock-port> \
RUNX_CONNECT_ACCESS_TOKEN=SECRET_CONNECT_ACCESS_TOKEN_DO_NOT_LEAK \
RUNX_CONNECT_OPEN_COMMAND=/path/to/test-opener \
cargo run --manifest-path crates/Cargo.toml -p runx-runtime -- connect github --scope repo:read --json
```

Secret scan gate:

```sh
! rg 'SECRET_|Bearer |authorize_url|oauth_code|device_code|client_secret|password|api_key|credential_body' \
  target/ fixtures/connect/oracles/
```

The secret scan command must return no leaked sentinel values in generated
artifacts or oracles. It may match source fixture definitions only when run
against the source fixture directory intentionally; test artifacts and expected
outputs must remain clean.

## Rollback Plan

- Revert only the Rust connect client, CLI wiring, and connect fixtures from
  this spec's implementation commit.
- Restore the previous canonical CLI dispatch owner from the scaffold/tool-
  catalog cutover commit; do not revive a TypeScript fallback inside the Rust
  path.
- Remove any new hosted connect fixture/oracle files added by this spec if the
  Rust implementation is reverted.
- Keep cloud-side auth and package manifests untouched unless their own specs
  explicitly changed them.
- Re-run the pre-existing runtime and policy test suite to confirm admission
  behavior returns to the pre-connect-client state.

## Build Readiness Verdict

Ready to execute after scaffold/tool-catalog cutover lands. The current repo
evidence is enough to build the hosted grant/session client and browser-opener
polling flow, but not enough to build local callback, device flow, or BYO
credential intake safely. Those flows remain explicitly blocked until a cloud
contract spec checks in fixtures for them.

## Open Questions

- None for approval. Hosted HTTP fixtures live under
  `fixtures/connect/contracts/hosted-http-v1/`; the first Rust implementation
  lives under `crates/runx-runtime/src/connect/`; callback, device flow, and
  BYO credential intake are separate future cloud-contract work.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T08:13:29Z
Ended: 2026-05-19T08:13:29Z

Checks:
- authority-boundary audit
  - Grounded in: code:packages/cli/src/connect-http.ts:6 and
    code:crates/runx-core/src/policy/connected_auth.rs:1
  - Result: passed
  - Evidence: The spec now frames connect as authority intake feeding later
    harness admission, not as skill execution or a parallel receipt family.
- cutover-sequencing audit
  - Grounded in: spec:rust-runtime-skeleton and spec:rust-tool-catalogs
  - Result: passed
  - Evidence: Runtime skeleton is treated as completed; the only approval
    blocker is the active scaffold/tool-catalog CLI dispatch cutover.
- secret-leak audit
  - Grounded in: code:packages/cli/src/connect-http.ts:89
  - Result: passed
  - Evidence: Acceptance requires typed HTTP envelopes, redacted errors,
    sentinel-based leak checks, and a failing secret scan over generated
    artifacts and oracles.

Issues:
- none
