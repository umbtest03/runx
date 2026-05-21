# Connect/Auth Licensing Boundary

Status: Phase 1 classification snapshot for
`connect-auth-mit-boundary-v1`.

## Governing Principle

Runx OSS crates may consume credential material and enforce declared authority.
They must not broker OAuth, custody secrets, mint verified grants, or couple to a
provider-specific credential broker such as Nango.

The durable crossing is the existing opaque credential channel:
`material_ref` is resolved through `MaterialResolver` into scoped material for a
single execution. Offline declared grants, local env/config material, redaction
hygiene, and connected-auth requirement policy stay MIT. Hosted connect,
browser/polling OAuth, `RUNX_CONNECT_ACCESS_TOKEN` broker access, Nango types,
connection session state, and verified-grant issuance move private or are
abstracted away from MIT crates.

## Phase 1 Inventory

Inventory command:

```sh
rg -n 'nango|oauth|RUNX_CONNECT|connection_id|material_ref|credential|auth' crates --glob '*.rs'
```

The command matched 99 Rust files. The classification below is intentionally
source-tree based; Phase 2 may rewrite or delete private surfaces, but this
snapshot records the current boundary.

## Private Or Move/Abstract

These files currently expose brokerage, hosted connect, Nango/OAuth,
`RUNX_CONNECT_*`, `connection_id`, or public surfaces built around those
concepts.

- `crates/runx-runtime/src/connect/client.rs` - removed from OSS in Phase 2; move private. It owns hosted
  HTTP connect polling, bearer auth, `RUNX_CONNECT_BASE_URL`, and
  `RUNX_CONNECT_ACCESS_TOKEN`.
- `crates/runx-runtime/src/connect/types.rs` - removed from OSS in Phase 2; abstract or move. It exposes
  `NangoConnection`, OAuth state, `connection_id`, and hosted grant material.
- `crates/runx-runtime/src/connect/opener.rs` - removed from OSS in Phase 2; move private with the browser
  connect flow.
- `crates/runx-runtime/src/connect.rs` - rewrite in Phase 2 so MIT exports only
  retained redaction/opaque conversion helpers, or remove the module export.
- `crates/runx-runtime/src/lib.rs` - remove hosted connect brokerage re-exports
  in lockstep with `connect.rs`.
- `crates/runx-cli/src/connect.rs` - delete from OSS or replace with an
  unavailable-in-OSS stub.
- `crates/runx-cli/src/main.rs` - remove or stub the native `runx connect`
  brokerage arm.
- `crates/runx-cli/tests/connect.rs` - rewrite to prove the retained CLI shape
  or delete with the subcommand.
- `crates/runx-cli/tests/launcher.rs` - update routing coverage after the
  connect command is removed or stubbed.
- `crates/runx-runtime/tests/connect_client.rs` - move private with the hosted
  client tests.
- `crates/runx-runtime/tests/connect_support.rs` - move private unless Phase 2
  extracts a pure MIT fixture helper.
- `crates/runx-runtime/tests/connect_policy_integration.rs` - keep only if it
  can test a pure MIT grant-to-local-admission conversion without Nango or
  hosted client fixtures.
- `crates/runx-runtime/tests/connect_secret_redaction.rs` - rewrite to exercise
  `redact_connect_text()` directly; redaction stays MIT, the hosted client does
  not.
- `crates/runx-sdk/src/client.rs` - remove or abstract the connect-list /
  `connection_id` hosted API from the MIT SDK surface.
- `crates/runx-sdk/tests/client_cli.rs` - rewrite after SDK connect surface is
  removed or abstracted.

## Phase 2 API Transition

The MIT runtime no longer exports `ConnectClient`, `ConnectError`,
`ConnectOpener`, `HttpConnectGrant`, `HttpConnectListResponse`,
`HttpConnectPreprovisionRequest`, `HttpConnectReadyResponse`,
`HttpConnectRevokeResponse`, or `load_connect_options_from_env`. The retained
MIT runtime surface is credential consumption/enforcement plus
`redact_connect_text()` for non-leakage hygiene.

The OSS CLI keeps an explicit `runx connect [--json]` unavailable stub. It does
not read `RUNX_CONNECT_*`, perform network brokerage, or render grant material.
The hosted/private CLI distribution owns real connect flows.

The Rust SDK removed `RunxClient::connect_list()` and `ConnectionSummary`. SDK
consumers should use generated provider-neutral credential contracts or the
private hosted client where connect brokerage is required.

## MIT Keep

These matched because they use generic credential, auth, authority, policy,
receipt, adapter, or fixture terminology. They do not broker OAuth or custody
secrets.

- `crates/runx-cli/src/launcher.rs`
- `crates/runx-cli/src/list.rs`
- `crates/runx-cli/src/scaffold.rs`
- `crates/runx-cli/tests/x402_native_dogfood.rs`
- `crates/runx-contracts/src/aster.rs`
- `crates/runx-contracts/src/authority.rs`
- `crates/runx-contracts/src/credential_delivery.rs`
- `crates/runx-contracts/src/external_adapter.rs`
- `crates/runx-contracts/src/harness.rs`
- `crates/runx-contracts/src/host_protocol.rs`
- `crates/runx-contracts/src/lib.rs`
- `crates/runx-contracts/src/maturity.rs`
- `crates/runx-contracts/src/post_merge_observer.rs`
- `crates/runx-contracts/src/post_merge_observer/plan.rs`
- `crates/runx-contracts/src/signal.rs`
- `crates/runx-contracts/src/verification.rs`
- `crates/runx-contracts/tests/credential_delivery_fixtures.rs`
- `crates/runx-contracts/tests/external_adapter_fixtures.rs`
- `crates/runx-contracts/tests/harness_spine_fixtures.rs`
- `crates/runx-contracts/tests/host_protocol_fixtures.rs`
- `crates/runx-contracts/tests/post_merge_observer.rs`
- `crates/runx-contracts/tests/schema_validation.rs`
- `crates/runx-core/src/kernel_eval.rs`
- `crates/runx-core/src/lib.rs`
- `crates/runx-core/src/policy.rs`
- `crates/runx-core/src/policy/authority_proof.rs`
- `crates/runx-core/src/policy/connected_auth.rs`
- `crates/runx-core/src/policy/local.rs`
- `crates/runx-core/src/policy/payment_authority.rs`
- `crates/runx-core/src/policy/public_work.rs`
- `crates/runx-core/src/policy/types.rs`
- `crates/runx-core/tests/policy_fixtures.rs`
- `crates/runx-core/tests/policy_proptest.rs`
- `crates/runx-parser/src/skill.rs`
- `crates/runx-receipts/src/tree.rs`
- `crates/runx-receipts/src/verify.rs`
- `crates/runx-receipts/src/verify/proof.rs`
- `crates/runx-receipts/tests/harness_receipts.rs`
- `crates/runx-runtime/src/adapter.rs`
- `crates/runx-runtime/src/adapters/catalog.rs`
- `crates/runx-runtime/src/adapters/cli_tool.rs`
- `crates/runx-runtime/src/adapters/external_adapter.rs`
- `crates/runx-runtime/src/adapters/mcp/adapter.rs`
- `crates/runx-runtime/src/adapters/mcp/server_skill.rs`
- `crates/runx-runtime/src/adapters/mcp/transport.rs`
- `crates/runx-runtime/src/adapters/mcp/types.rs`
- `crates/runx-runtime/src/config.rs`
- `crates/runx-runtime/src/connect/redaction.rs`
- `crates/runx-runtime/src/credentials.rs`
- `crates/runx-runtime/src/error.rs`
- `crates/runx-runtime/src/execution/harness/runner.rs`
- `crates/runx-runtime/src/execution/runner.rs`
- `crates/runx-runtime/src/execution/runner/authority.rs`
- `crates/runx-runtime/src/execution/runner/inputs.rs`
- `crates/runx-runtime/src/execution/runner/steps.rs`
- `crates/runx-runtime/src/execution/skill_run.rs`
- `crates/runx-runtime/src/execution/target_runner.rs`
- `crates/runx-runtime/src/list.rs`
- `crates/runx-runtime/src/payment_packets.rs`
- `crates/runx-runtime/src/payment_state.rs`
- `crates/runx-runtime/src/post_merge_observer.rs`
- `crates/runx-runtime/src/receipts/seal.rs`
- `crates/runx-runtime/src/registry/local/build.rs`
- `crates/runx-runtime/src/registry/types.rs`
- `crates/runx-runtime/src/runtime_http.rs`
- `crates/runx-runtime/src/scaffold/new.rs`
- `crates/runx-runtime/src/scaffold/templates.rs`
- `crates/runx-runtime/tests/a2a_parity.rs`
- `crates/runx-runtime/tests/agent_parity.rs`
- `crates/runx-runtime/tests/catalog_adapter.rs`
- `crates/runx-runtime/tests/cli_tool_contract.rs`
- `crates/runx-runtime/tests/credential_delivery.rs`
- `crates/runx-runtime/tests/external_adapter.rs`
- `crates/runx-runtime/tests/harness_fixtures.rs`
- `crates/runx-runtime/tests/mcp_adapter.rs`
- `crates/runx-runtime/tests/payment_execution.rs`
- `crates/runx-runtime/tests/payment_ledger_projection.rs`
- `crates/runx-runtime/tests/payment_receipts.rs`
- `crates/runx-runtime/tests/payment_state.rs`
- `crates/runx-runtime/tests/scaffold.rs`
- `crates/runx-runtime/tests/skill_author_runtime_fixtures.rs`
- `crates/runx-runtime/tests/skill_issue_to_pr.rs`
- `crates/runx-runtime/tests/stripe_spt_payment.rs`
- `crates/runx-runtime/tests/target_runner.rs`
- `crates/runx-sdk/src/lib.rs`

## Crate Decisions

- `runx-contracts` - keep MIT. It owns provider-neutral wire contracts, including
  the credential-delivery envelope.
- `runx-core` - keep MIT. It enforces authority and policy requirements; it must
  not issue grants or call providers.
- `runx-parser` - keep MIT. Skill parsing is unrelated to brokerage.
- `runx-receipts` - keep MIT. Receipt verification is provider-neutral.
- `runx-runtime` - keep MIT after Phase 2 removes hosted connect brokerage and
  retains only credential consumption, grant enforcement, redaction, and opaque
  resolver seams.
- `runx-cli` - keep MIT after Phase 2 deletes or stubs the hosted `runx connect`
  brokerage command.
- `runx-sdk` - keep MIT after Phase 2 abstracts or removes the hosted
  connect-list / `connection_id` API surface.

## Deferred Contract Cleanup

`CredentialEnvelope.connection_id` and the matching authority-proof projection
remain as legacy public wire fields for compatibility with the existing
credential envelope. They are allowlisted only as passive metadata fields: MIT
code must not broker OAuth, call Nango, or construct provider-specific
`nango:<provider>:<connection_id>` locators from them. Renaming or removing the
wire fields is a contract migration deferred to
`credential-envelope-opaque-reference-v1`.

## Private Home

The private implementation home is `../cloud/packages/auth`. That package owns
Nango, OAuth, connect HTTP routes, BYO credential custody, grant issuance, and
grant revocation. Read-only context from Phase 1 also identified hosted wiring
in `../cloud/packages/api` and run-auth resolver context in
`../cloud/packages/worker`.

Follow-up cloud work must move or re-home the brokerage implementation there;
this OSS spec records the boundary and remediates only the OSS repository.

## Guard Inputs

The machine-readable guard input is `docs/license-boundary.manifest.json`.
Phase 1 validates that the manifest is structurally complete and covers every
Rust workspace crate. Phase 3 extends the same checker with identifier and
dependency-edge scans.

`crates/deny.toml` may continue to allow Apache-2.0 for transitive third-party
dependencies. That is distinct from the OSS Rust workspace crate metadata,
which Phase 3 aligns to MIT.
