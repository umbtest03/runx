---
spec_version: '2.0'
task_id: connect-auth-mit-boundary-v1
created: '2026-05-22T01:19:01+10:00'
updated: '2026-05-21T17:13:56Z'
status: completed
harden_status: passed
size: large
risk_level: high
---

# Connect/auth MIT-vs-private licensing boundary v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T17:13:56Z
Review gate: pass

## Summary

Establish and enforce the licensing boundary for the connect/credential/auth
layers. The MIT OSS crates may contain only credential **consumption** and grant
**enforcement**, including the offline/declared-grant path that lets the runtime
run with no account. Everything else on this axis is **private**: the OAuth
handshake, the `runx connect` browser/polling flow, token custody, hosted and
verified-grant issuance, and all provider-specific (Nango) coupling.

The governing principle is **consumption vs brokerage**: the OSS runtime resolves
an opaque credential reference and enforces a grant; the private infra brokers
OAuth, runs the connect flow, custodies the secret, and mints verified grants.
The OSS-to-private crossing is the *already-defined* credential envelope plus the
opaque `MaterialResolver` contract. This spec does not redefine that crossing. It
owns the licensing classification on top of it, the relocation of the connect/
OAuth surface that currently sits in the OSS Rust workspace
(`crates/runx-runtime/src/connect/`, `crates/runx-cli/src/connect.rs`) into
private infra, and a durable guard that fails when the boundary is violated.

Default split (Phase 1 confirms per file): offline credential **consumption** —
declared grants, credential material from env/config, the opaque resolver,
redaction hygiene, and the connected-auth *requirement* policy — stays MIT. The
connect **brokerage** surface — OAuth handshake/polling, the browser-driven
connect flow, `RUNX_CONNECT_ACCESS_TOKEN` hosted-broker access, `NangoConnection`
/provider types, connection session state, and verified-grant issuance — is
private.

This spec is governance, not a new contract. It is the sibling of
`runx-cli/tests/locality.rs` (which guards "the runtime emits nothing") but for
"the MIT crates broker nothing and custody no secrets."

## License Authority

The authoritative OSS license for this repository is **MIT** (repo root
`LICENSE`). The Rust workspace currently declares `license = "Apache-2.0"` at
`crates/Cargo.toml`, inherited by every crate via `license.workspace = true`.
This is an unintended inconsistency, not a deliberate dual-license posture.

Operator decision recorded for this task (2026-05-22): **standardize the OSS
Rust crates on MIT** to match the repo root. Phase 3 changes the workspace
`license` field from `Apache-2.0` to `MIT`. This is an approved, explicit
license-metadata change, valid because the copyright is controlled by the repo
owner (no third-party contributor relicensing constraint). Apache-2.0's explicit
patent grant is consciously given up; revisit only if downstream legal requires
it (relicensing more permissively or adding a patent grant later remains open).

The boundary guard therefore treats "MIT crate" as "any crate in the OSS Rust
workspace," and the private/proprietary side carries its own (non-MIT) license in
its own repository.

## Context

Sibling specs already own the pieces this boundary sits above. This spec must not
re-open any of them:

- `credential-broker-delivery-contract-v1` (active) owns the delivery primitive:
  opaque `material_ref` resolved through a trusted broker/resolver into scoped
  secret material handed to an adapter. Not redefined here.
- `byo-credential-foundations` (completed) owns the final grant + credential
  envelope shape (`material_kind`/`material_ref`, optional `connection_id`), the
  encrypted BYO store, and BYO verification. Cloud-side. Not redefined here.
- `rust-adapter-credential-delivery` (archived) owns Rust materialization and
  adapter injection. Not redefined here.
- `rust-connect-client` (archived) ported the connect surface (grants + the
  Nango-hosted OAuth polling client) to Rust. This spec audits whether that
  client's Nango/hosted assumptions belong in an MIT crate and, if not, records
  the abstract-or-move decision. It does not re-port the client.
- `external-adapter-plugin-protocol-v1` (active) owns the out-of-process
  execution-adapter boundary. Not redefined here.
- `rust-placeholder-crates-publish` (draft) owns crates.io publish ordering. This
  spec sets the per-crate license classification those publishes must honor; it
  does not own publish mechanics or versioning.

Architectural reference (plan, not contract):
- `plans/runx.md` "Auth Layer (Nango)" — runx never stores raw tokens; Nango is
  the credential store; connected mode brokers OAuth, execution stays local.
- `plans/integrations/pre-oauth-foundations.md` — flags that grants/envelopes
  today hardcode `nango:<provider>:<connection_id>`; that hardcoding is the MIT
  violation this spec must resolve or relocate.

## Objectives

All OSS paths below are relative to the OSS repo root
(`/Users/kam/dev/runx/runx/oss`): Rust is `crates/**`, docs are `docs/**`,
scafld is `.scafld/**`. Sibling repositories are explicit: `../cloud/**` and
`../plans/**`.

- Produce a definitive layer-to-license classification generated from a real
  inventory (`rg`), not a guessed file list: every connect/credential/auth
  surface across `crates/**` (and a read-only survey of `../cloud/packages/**`)
  labelled `mit-oss` or `private`, each with the one-line principle that decided
  it.
- Audit the OSS crates for private-infra coupling: provider-specific (Nango)
  identifiers, the OAuth/connect-flow client, hosted-only logic, verified-grant
  issuer key material, and token custody. For each finding record a keep /
  abstract-behind-trait / move-to-private decision with rationale.
- Confirm and normalize the licensed crossing: OSS resolves an opaque
  `material_ref` through the provider abstraction whose offline/local
  implementation stays MIT; the connected/OAuth/Nango implementation is private.
  No Nango type names, OAuth-flow client, client secrets, token storage, or
  issuer signing keys remain in MIT crates except entries on a documented
  allowlist.
- Make the boundary durable: a single boundary guard backed by a committed
  classification manifest, an MIT-license assertion on the OSS workspace, the
  licensing-boundary doc, and CI wiring. The guard fails on (a) any
  unallowlisted private identifier in MIT Rust sources or tests, and (b) any
  dependency edge from an MIT crate into a private crate.
- Give the private connect/OAuth code a documented home outside the MIT
  workspace. This spec edits only the OSS repo; relocating code into `../cloud`
  is delegated to a separate cloud-owned spec named here as a follow-up.

## Scope

In scope (OSS repo only — all edits land under `crates/**`, `docs/**`, or
`.scafld/**`):
- License classification of the connect/credential/auth layers across the OSS
  crate graph, plus a read-only survey of `../cloud/packages/**` recorded for
  context.
- Audit and remediation decisions (keep/abstract/move) for OSS crates that touch
  connect, credentials, or auth.
- Relocating or abstracting the OSS-side OAuth/connect brokerage surface so the
  MIT crates retain only consumption + enforcement + the offline path.
- Aligning the OSS Rust workspace `license` to MIT (per License Authority),
  a licensing-boundary doc, a classification manifest, the boundary guard, and CI
  wiring for the guard.
- Naming the private home and the follow-up cloud spec that will host moved code.

Out of scope:
- Redefining the credential envelope or grant shape (`byo-credential-foundations`).
- The delivery primitive (`credential-broker-delivery-contract-v1`).
- Rust adapter materialization/injection (`rust-adapter-credential-delivery`).
- Re-implementing the connect client (`rust-connect-client`); this spec moves or
  abstracts it, it does not rebuild the flow.
- OAuth handshake, connect-session, or verification implementation (BYO and cloud
  auth specs).
- Any edit to the sibling `../cloud` repository; cloud relocation is a named
  follow-up cloud spec, not this task.
- crates.io publish ordering, versioning, or release mechanics
  (`rust-placeholder-crates-publish`).
- Marketing/site copy about connect (separate web work).
- The choice of OSS license is settled in License Authority (MIT); not re-opened
  here.

## Dependencies

- `credential-broker-delivery-contract-v1`; the opaque-ref crossing this boundary
  classifies must stay intact.
- `byo-credential-foundations`; the envelope/grant shape is the contract that
  crosses the license boundary.
- `rust-connect-client`; the connect surface under audit.
- `rust-placeholder-crates-publish`; consumes the per-crate license classification
  before any further publish.
- `locality.rs` regression guard; the pattern for the new boundary guard.

## Touchpoints

OSS repo root is `/Users/kam/dev/runx/runx/oss`. Paths are relative to it.

Connect/OAuth surface to classify and relocate-or-abstract (private by default):
- `crates/runx-runtime/src/connect.rs` (module root and connect re-exports)
- `crates/runx-runtime/src/lib.rs` (public connect re-export block)
- `crates/runx-runtime/src/connect/client.rs` (OAuth polling, `RUNX_CONNECT_ACCESS_TOKEN`)
- `crates/runx-runtime/src/connect/types.rs` (`NangoConnection`, `ConnectGrantMaterialKind`)
- `crates/runx-runtime/src/connect/opener.rs` (browser opener)
- `crates/runx-cli/src/connect.rs` and the `connect` arm in `crates/runx-cli/src/main.rs`
- `crates/runx-sdk/src/client.rs` (`connect_list`, `connection_id`)
- `crates/runx-cli/tests/connect.rs`, `crates/runx-sdk/tests/client_cli.rs`
- `crates/runx-cli/tests/launcher.rs` connect routing coverage
- `crates/runx-runtime/tests/connect_client.rs`
- `crates/runx-runtime/tests/connect_support.rs`
- `crates/runx-runtime/tests/connect_policy_integration.rs`

Consumption/enforcement surface to keep MIT (confirm in Phase 1):
- `crates/runx-runtime/src/credentials.rs` (opaque `MaterialResolver` seam)
- `crates/runx-runtime/src/connect/redaction.rs` (non-leakage hygiene)
- `crates/runx-core/src/policy/connected_auth.rs` (connected-auth *requirement*)
- `crates/runx-runtime/tests/connect_secret_redaction.rs`, rewritten in Phase 2
  to exercise redaction helpers directly, not `ConnectClient`

Boundary mechanics (this spec creates):
- `crates/Cargo.toml` (workspace `license` → MIT)
- `crates/*/Cargo.toml` (only where a crate overrides `license.workspace`)
- `crates/runx-runtime/tests/` (new boundary guard test)
- `crates/runx-cli/tests/locality.rs` (guard pattern reference)
- `.scafld/scripts/check-license-edges.mjs` (new dependency-edge checker)
- `docs/licensing-boundary.md` and the classification manifest (new)
- repo root `LICENSE`
- `crates/deny.toml` (transitive dependency license allowlist; documented, not
  narrowed by this spec)

Read-only sibling context (no edits this task):
- `../cloud/packages/auth/src/nango-hosted.ts` (the private broker; documented home)
- `../plans/runx.md`, `../plans/integrations/pre-oauth-foundations.md`

## Risks

- Over-moving: pulling consumption or enforcement out of the OSS crates guts the
  standalone runtime's value and breaks the offline/declared-grant path.
- Under-moving: leaving Nango/issuer/custody coupling in an MIT crate publishes
  private assumptions under an open licence. This is the concrete harm and the
  reason the spec exists.
- A string-match-only guard is evadable; the guard must also catch dependency
  edges (an MIT crate must not depend on a private crate).
- Classification drift: a crate added later with no license decision silently
  publishes. The guard must fail on unclassified connect/auth surfaces, not only
  on known bad strings.
- Misclassifying the redaction/policy code: redaction hygiene and the connected-
  auth *requirement* policy are legitimately MIT; only secret custody, OAuth
  brokerage, and verified-grant issuance are private.

## Guard Semantics

The boundary is enforced by one guard with two checks, both reading the same
committed inputs so the spec's words and the build agree.

Inputs:
- Phase 1 creates `docs/license-boundary.manifest.json` and
  `.scafld/scripts/check-license-edges.mjs` with
  `--check manifest-complete`.
- Phase 3 extends the same script with `--check identifiers` and
  `--check edges`.
- `docs/license-boundary.manifest.json` — the classification manifest: for each
  crate, `class: "mit-oss" | "private"`; a `banned_identifiers` list (default:
  `Nango`, `nango`, `NangoConnection`, `nango_connection`, `oauth_poll`,
  `RUNX_CONNECT_ACCESS_TOKEN`, plus any added in Phase 1); and an `allowlist` of
  `{path, identifier, rationale}` exceptions.
- `private_crate_names` — the set of crate names classified `private`.

Check A (identifier scan): for every `mit-oss` crate, scan all `*.rs` under its
`src/` and `tests/` for any `banned_identifiers` hit not on the `allowlist`. Any
hit fails. Tests and fixtures are in scope; historical protocol names survive
only as explicit, rationale-bearing allowlist entries.

Check B (dependency edges): from `cargo metadata`, fail if any `mit-oss` crate
has a dependency edge (normal, build, or dev) into a `private` crate.

Generated/vendored paths are excluded by an explicit `exclude_globs` list in the
manifest, never by silent heuristics. A crate absent from the manifest fails
Check A by default (no silent unclassified crate). `v3`/`v4` below invoke this
one guard, not ad-hoc greps.

Negative fixtures live under
`crates/runx-runtime/tests/fixtures/license_boundary/**`, excluded by an
explicit manifest `exclude_globs` entry. The boundary test exercises violations
by copying a fixture into a temporary scan root or by using an explicit override;
the normal source-tree scan must not fail on the planted fixture itself.

## Public API Transition

`runx-runtime` is pre-1.0 (`0.0.1`), but the connect/auth boundary break is
explicit. Phase 2 may remove or replace the public connect brokerage re-exports
from `crates/runx-runtime/src/lib.rs` and `crates/runx-runtime/src/connect.rs`:
`ConnectClient`, `ConnectError`, `ConnectOpener`, `HttpConnectGrant`,
`HttpConnectListResponse`, `HttpConnectPreprovisionRequest`,
`HttpConnectReadyResponse`, `HttpConnectRevokeResponse`,
`load_connect_options_from_env`, and any `connect::*` re-export that exposes
Nango/OAuth brokerage.

The boundary doc must record each removed symbol and replacement. The retained
MIT runtime contract is the opaque credential envelope plus `MaterialResolver`
for consumption and enforcement. Private brokerage lives in
`../cloud/packages/auth` under a cloud-owned follow-up spec. No public API is
silently deleted.

The OSS CLI must not keep a hidden hosted brokerage path. Phase 2 either deletes
the native `runx connect` brokerage command routing from `runx-cli`, or replaces
it with a deterministic unavailable-in-OSS stub while the private implementation
moves to the cloud-owned follow-up. The chosen behavior must be recorded in
`docs/licensing-boundary.md`, and `launcher`/`connect` tests must be rewritten or
removed to match it.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` `docs/licensing-boundary.md` plus `docs/license-boundary.manifest.json`
  classify every connect/credential/auth surface found by an `rg` inventory across
  `crates/**` (and a recorded survey of `../cloud/packages/**`) as `mit-oss` or
  `private`, each with the deciding principle.
- [x] `dod2` Each OSS crate touching connect/credential/auth has an explicit
  keep / abstract / move decision with rationale in the doc.
- [x] `dod3` MIT crates contain no unallowlisted private coupling: no Nango/OAuth
  brokerage identifiers, token custody, or issuer key material, and no dependency
  edge into a private crate. Allowlisted exceptions carry a rationale.
- [x] `dod4` The OSS Rust workspace declares `license = "MIT"` (per License
  Authority); no crate silently overrides it back to Apache-2.0.
- [x] `dod5` The boundary guard (Check A + Check B) passes on the current tree and
  fails when a banned identifier or a private dependency edge is introduced
  (proved by a negative fixture in the test).
- [x] `dod6` The private connect/OAuth home and the follow-up cloud spec are named
  in the doc; no OSS crate is left brokering OAuth or custodying secrets.

Validation:
- [x] `v1` Scafld validates this spec.
  - Command: `scafld validate connect-auth-mit-boundary-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:12+10 returned valid=true for
    `.scafld/specs/active/connect-auth-mit-boundary-v1.md`.
- [x] `v2` The boundary guard test passes on the current tree, including its
  negative case.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test license_boundary`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T03:20+10 passed 3 tests, including copied identifier
    and synthetic dependency-edge negative fixtures.
- [x] `v3` Identifier scan (Check A) is clean from the OSS repo root.
  - Command: `node .scafld/scripts/check-license-edges.mjs --check identifiers`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:20+10 returned
    `{"ok":true,"check":"identifiers"}`.
- [x] `v4` Dependency-edge scan (Check B) is clean from the OSS repo root.
  - Command: `cargo metadata --manifest-path crates/Cargo.toml --format-version 1 | node .scafld/scripts/check-license-edges.mjs --check edges`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:20+10 returned
    `{"ok":true,"check":"edges","private_crates":0}`.
  - Negative coverage: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test license_boundary`
    now includes a synthetic `runx-runtime -> runx-private-auth` metadata
    fixture and expects Check B to reject it.

## Phase 1: Classification (Documentation-Only)

Status: active
Dependencies: none

Objective: Documentation-only — no source-code remediation. Generate the

Changes:
- [x] Generate the inventory: `rg -n 'nango|oauth|RUNX_CONNECT|connection_id|material_ref|credential|auth' crates --glob '*.rs'` and record the matched files.
- [x] Label every matched surface `mit-oss` or `private` with the deciding principle; survey `../cloud/packages/**` read-only and record it for context.
- [x] For each OSS crate finding, record a keep / abstract / move decision with rationale.
- [x] Seed `docs/license-boundary.manifest.json` (classes, banned identifiers, allowlist, exclude globs) from the inventory.
- [x] Add `.scafld/scripts/check-license-edges.mjs` with `--check manifest-complete`; Check A/B implementation is a Phase 3 extension.
- [x] Seed manifest `exclude_globs` with `crates/runx-runtime/tests/fixtures/license_boundary/**`.

Acceptance:
- none

## Phase 2: Seam Normalization

Status: completed
Dependencies: Phase 1

Objective: Make the opaque resolver the only OSS-to-private crossing — relocate or

Changes:
- [x] Move or abstract the connect/OAuth brokerage code (`connect/client.rs`, `connect/types.rs`, `connect/opener.rs`, the `runx connect` OAuth arm) per the Phase 1 decisions; OSS retains only the opaque-ref consumption path.
- [x] Edit `crates/runx-runtime/src/connect.rs` and `crates/runx-runtime/src/lib.rs` in lockstep with submodule relocation; record the acknowledged pre-1.0 public API break and replacements in the boundary doc.
- [x] Rewrite `crates/runx-runtime/tests/connect_secret_redaction.rs` so the MIT non-leakage guard exercises `redact_connect_text()` / retained redaction helpers directly, not `ConnectClient` or hosted HTTP request types.
- [x] Move, delete, or rewrite brokerage tests in `crates/runx-runtime/tests/connect_client.rs`, `crates/runx-runtime/tests/connect_support.rs`, and `crates/runx-runtime/tests/connect_policy_integration.rs`. Any retained policy integration test must depend only on a MIT grant-to-local-admission conversion, not Nango/hosted client types.
- [x] Delete native `runx connect` brokerage routing from the OSS CLI, or replace it with an explicit unavailable-in-OSS stub; update `crates/runx-cli/tests/connect.rs` and `crates/runx-cli/tests/launcher.rs` to prove the retained behavior.
- [x] Remove hardcoded `nango:<provider>:<connection_id>` construction or consumption from retained MIT brokerage surfaces. Legacy public `connection_id` wire fields remain as passive metadata only; rename/removal is deferred to `credential-envelope-opaque-reference-v1`.
- [x] Confirm the declared-grant resolver path runs with no account, no hosted broker, and no private dependency. `RUNX_CONNECT_ACCESS_TOKEN` is a BYO hosted broker token path, not an offline path.
- [x] Record each allowlist entry (with rationale) for any historical identifier that must remain in an MIT crate.

Acceptance:
- [x] `p2a` Runtime offline/credential tests pass after relocation.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test credential_delivery --features cli-tool,mcp`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:22+10 passed 11 tests.
- [x] `p2b` The existing non-leakage guard still passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test connect_secret_redaction`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:23+10 passed 2 tests against
    `redact_connect_text()` without hosted client types.
- [x] `p2c` CLI command routing tests for the retained CLI shape pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test launcher && cargo test --manifest-path crates/Cargo.toml -p runx-cli --test connect`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:24+10 passed 21 launcher tests and 3 connect
    stub tests.
- [x] `p2d` SDK no longer exposes the hosted connect-list surface.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-sdk --test client_cli`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:23+10 passed 2 SDK CLI-backed tests after
    removing `RunxClient::connect_list()` and `ConnectionSummary`.

## Phase 3: Guard And Metadata

Status: completed
Dependencies: Phase 2

Objective: Make the boundary durable. OSS-repo edits only.

Changes:
- [x] Set the OSS Rust workspace `license` to `MIT` in `crates/Cargo.toml`; remove any crate-level Apache-2.0 override.
- [x] Extend `.scafld/scripts/check-license-edges.mjs` with Check A
  (identifiers) and Check B (edges); `--check manifest-complete` already exists
  from Phase 1.
- [x] Add the boundary guard test (sibling to `locality.rs`) that runs both checks and includes a negative fixture proving the guard fails on a planted violation.
- [x] Wire the guard into CI (the OSS test/lint workflow).
- [x] Confirm the doc names the private connect/OAuth home (`../cloud/packages/auth`) and the follow-up cloud spec.
- [x] Record that `crates/deny.toml` may allow Apache-2.0 for transitive
  dependencies; that is distinct from the OSS crate metadata and
  connect/auth boundary.

Acceptance:
- [x] `p3a` The boundary guard test passes including its negative case.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test license_boundary`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T03:20+10 passed 3 tests, including identifier and
    dependency-edge negative cases.
- [x] `p3b` Workspace license is MIT.
  - Command: `rg -n 'license\s*=\s*"MIT"' crates/Cargo.toml`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T02:22+10 matched `crates/Cargo.toml:29`.

## Rollback

General rule: if a layer cannot be cleanly classified, mark it **private by
default** and record a named blocker. Never weaken the offline path or the
existing `CredentialDelivery` secret channel to satisfy the boundary.

Per-phase repair:
- Phase 1: delete `docs/licensing-boundary.md` and `docs/license-boundary.manifest.json`; no source changed, so `scafld validate` is the only re-check.
- Phase 2: revert the connect/OAuth relocation commit(s) to restore the moved
  modules and public re-exports in `connect.rs`/`lib.rs`; re-run
  `cargo test -p runx-runtime -p runx-cli` and the redaction guard to confirm
  the offline path is intact.
- Phase 3: restore the previous `crates/Cargo.toml` `license` value, remove `.scafld/scripts/check-license-edges.mjs`, the `license_boundary` test, and the CI step; re-run `scafld validate` and the OSS test suite.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: command
Output: command.stdout
Summary: Command-provider verification pass. Rechecked connect/auth MIT boundary after fixing the external review hygiene findings: hosted runtime connect brokerage files and tests are removed, CLI connect is an explicit MIT-OSS unavailable stub that accepts only --json, SDK no longer exposes connect_list/ConnectionSummary, license-boundary manifest/doc classify every OSS crate and name the private home, Check A and Check B both pass with negative coverage, CI wires the guard, workspace crate license is MIT, and legacy connection_id fields are passive metadata deferred to credential-envelope-opaque-reference-v1. No completion blockers found.

Attack log:
- `brokerage removal`: search retained runtime/CLI/SDK source for ConnectClient, ConnectOpener, HttpConnect*, NangoConnection, RUNX_CONNECT_*, connect_list, and ConnectionSummary -> clean
- `CLI connect stub`: run cargo test -p runx-cli --test connect after tightening parser to only allow --json -> clean
- `license-boundary manifest`: run manifest-complete and confirm dead allowlist entries are pruned -> clean
- `identifier guard`: run identifiers check and verify copied NangoConnection fixture fails in license_boundary test -> clean
- `dependency-edge guard`: run edges check and verify synthetic runx-runtime -> runx-private-auth metadata fails in license_boundary test -> clean
- `CI wiring`: verify .github/workflows/ci.yml runs manifest-complete, identifiers, edges, and license_boundary test -> clean
- `SDK docs/API`: verify SDK README and source no longer advertise or expose connect list APIs -> clean
- `legacy connection_id`: verify retained connection_id is documented as passive metadata with migration deferred to credential-envelope-opaque-reference-v1 -> clean

Findings:
- none

## Origin

User architecture review on 2026-05-22: after reconciling `connect.runx.ai` (a
self-hosted Nango instance) with the runs-stay-local doctrine, the user noted
that OAuth/connect was always intended to be private infra, not MIT core, and
asked which layers do not belong in MIT. The reconciliation holds because runx
never holds the secret and never sees the run; the remaining gap is a governed
licensing boundary so brokerage, custody, and verified-grant issuance never ship
under MIT.

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-21T15:25:39Z
Ended: 2026-05-21T15:25:39Z
Verdict: needs_revision
Provider: codex
Output format: codex.output_file
Summary: Harden verdict: needs revision. The architectural boundary is the right move, but approval is unsafe until the draft fixes executable paths, resolves the Apache-vs-MIT authority conflict, expands the actual audit surface, defines guard semantics, clarifies Phase 1 write permissions, and pins cross-repo ownership.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:139
  - Result: failed
  - Evidence: Spec touchpoints use `oss/crates/...` and `cloud/packages/...` at `.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:139-148`. In this checkout, OSS Rust paths are `crates/...`; sibling cloud paths exist as `../cloud/packages/auth/src/nango-hosted.ts`, not `cloud/...` under the OSS root.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:197
  - Result: failed
  - Evidence: `scafld validate connect-auth-mit-boundary-v1 --json` returned valid=true. However v3 uses `! rg -i 'nango' oss/crates --glob '*.rs'` and v4 pipes to `node oss/.scafld/scripts/check-license-edges.mjs`; neither `oss/crates` nor `oss/.scafld/scripts/check-license-edges.mjs` exists relative to /Users/kam/dev/runx/runx/oss.
- scope/migration audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect/types.rs:30
  - Result: failed
  - Evidence: The actual code includes connect/auth surfaces not listed in touchpoints, including `crates/runx-runtime/src/connect/client.rs`, `crates/runx-runtime/src/connect/types.rs`, `crates/runx-cli/src/connect.rs`, and `crates/runx-cli/src/main.rs`. `rg` found Nango/OAuth/connect material in these files, including `ConnectGrantMaterialKind::NangoConnection` at `crates/runx-runtime/src/connect/types.rs:30-32`.
- acceptance timing audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:221
  - Result: failed
  - Evidence: Strict validation is selected at `.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:169`, but each phase has `Acceptance: - none` at lines 221-222, 241-242, and 259-260. The configured strict profile includes per-phase acceptance item checks, so build phases would have weak or vacuous evidence.
- rollback/repair audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:262
  - Result: failed
  - Evidence: Rollback says to mark ambiguous layers private and not weaken the offline path, but it does not say how to repair partial license metadata changes, moved connect client code, new guard scripts, CI wiring, or cross-repo cloud edits.
- design challenge
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/credentials.rs:99
  - Result: failed
  - Evidence: The boundary principle is sound: private Nango brokerage is concrete in `../cloud/packages/auth/src/nango-hosted.ts:75-87`, while OSS `MaterialResolver` is an opaque resolver at `crates/runx-runtime/src/credentials.rs:99-104`. The draft still needs executable boundaries before approval.

Issues:
- [high/blocks approval] `harden-1` path-command-mismatch - The draft is not executable from the declared checkout because several paths and acceptance commands are wrong.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:139
  - Evidence: Spec paths and commands are mixed between workspace-root style (`oss/crates/**`) and OSS-repo-root style (`crates/**`). From `/Users/kam/dev/runx/runx/oss`, the actual Rust files are under `crates/`, and sibling cloud/plans are under `../cloud` and `../plans`. The v3/v4 commands currently point at nonexistent paths.
  - Recommendation: Rewrite touchpoints and validation commands so they run from `/Users/kam/dev/runx/runx/oss`: v3 should scan `crates`, and v4 should point to an actual future script path such as `.scafld/scripts/check-license-edges.mjs` or `scripts/check-license-edges.mjs`.
  - Question: Should the spec be normalized to the OSS repo root (`crates/...`, `docs/...`, `.scafld/...`) and use `../cloud/...` / `../plans/...` for sibling repositories?
  - Recommended answer: Yes. Normalize all OSS paths to repo-root-relative `crates/...` and make sibling cloud/plans paths explicit as `../cloud/...` and `../plans/...`.
  - If unanswered: Default to OSS-repo-root commands: `crates/**`, `docs/**`, `.scafld/**`, and explicit sibling references `../cloud/**` / `../plans/**` only when cross-repo read/write is intended.
- [critical/blocks approval] `harden-2` license-authority-conflict - The MIT premise conflicts with current Rust crate metadata and cannot be treated as a routine code edit.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/Cargo.toml:29
  - Evidence: The draft assumes MIT at `.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:124` and requires MIT crate metadata at lines 181-182. The current Rust workspace license is `Apache-2.0` at `crates/Cargo.toml:29`, inherited by all crates via `license.workspace = true`; root `LICENSE` is MIT.
  - Recommendation: Add an explicit license-authority section naming the authoritative license source and whether Phase 3 is allowed to change `crates/Cargo.toml` workspace license from Apache-2.0 to MIT. If not approved, revise the task away from MIT-specific crate metadata.
  - Question: Is changing the Rust workspace license metadata from Apache-2.0 to MIT explicitly approved for every MIT-OSS crate in this task?
  - Recommended answer: Yes, this task should explicitly change the Rust workspace license metadata to MIT for crates classified OSS, and the spec should name that as an approved licensing decision because the current workspace metadata says Apache-2.0.
  - If unanswered: Do not approve a spec that silently changes published crate license metadata; require an explicit operator/legal decision first.
- [high/blocks approval] `harden-3` scope-incomplete - The current scope omits boundary-relevant files that already contain private-infra terms.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect/client.rs:139
  - Evidence: The touchpoint list omits actual connect/auth Rust surfaces containing boundary-relevant concepts: `crates/runx-runtime/src/connect/client.rs` handles OAuth polling and `RUNX_CONNECT_ACCESS_TOKEN`; `crates/runx-runtime/src/connect/types.rs` defines `NangoConnection`; `crates/runx-cli/src/main.rs` renders `nango_connection`; `crates/runx-sdk/src/client.rs` exposes `connection_id`.
  - Recommendation: Add those concrete files to touchpoints and require Phase 1 to produce an inventory generated from an `rg` search, not only the initially listed files.
  - Question: Should the audit scope explicitly include the full Rust connect module, CLI connect renderer/parser, SDK connection summary, contracts credential-delivery types, and tests/fixtures?
  - Recommended answer: Yes. The classification must include all Rust connect/credential/auth surfaces, including tests and fixtures, with explicit keep/abstract/move decisions for retained terms like `connection_id`, `oauth`, and `nango_connection`.
  - If unanswered: Default to expanding scope to every file matched by `rg -n 'nango|oauth|RUNX_CONNECT|connection_id|material_ref|credential|auth' crates --glob '*.rs'`, then classify each as keep/abstract/move.
- [high/blocks approval] `harden-4` guard-semantics-unclear - The planned grep either fails on known current fixtures or forces undocumented deletions; guard semantics need to be pinned.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:197
  - Evidence: Acceptance says retained exceptions may be justified in the doc at lines 177-180, but v3 is a blanket `! rg -i 'nango' oss/crates --glob '*.rs'`. Current Rust files include `NangoConnection` in `crates/runx-runtime/src/connect/types.rs:30-32`, CLI rendering at `crates/runx-cli/src/main.rs:317-320`, and tests/fixtures with Nango strings. The guard design does not specify allowlists, generated inventory, or whether tests are subject to the same ban.
  - Recommendation: Define the guard inputs: banned identifier list, scanned roots, excluded/generated paths, test-fixture policy, classification manifest, and private crate marker. Then make v3 invoke the same guard instead of a separate brittle grep.
  - Question: What exact private-infra identifiers are banned in MIT crates, and are tests/fixtures allowed to retain any historical protocol names under a documented allowlist?
  - Recommended answer: Use a single boundary guard backed by the classification doc/manifest. Ban Nango/provider-custody terms by default across production and test Rust, allow only named compatibility fixtures with rationale, and fail on new unclassified hits.
  - If unanswered: Default to a committed allowlist file owned by the licensing-boundary doc; the guard fails on any unallowlisted private term or private dependency edge in production and test Rust sources.
- [medium/blocks approval] `harden-5` phase-acceptance-contradiction - Phase 1 contradicts itself and lacks non-vacuous acceptance evidence.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:211
  - Evidence: Phase 1 is declared read-only at `.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:211-212`, but its changes include committing the classification and audit doc at lines 214-219. With strict per-phase validation, Phase 1 has no acceptance criteria proving the inventory is complete before Phase 2 starts.
  - Recommendation: Either make Phase 1 read-only and move doc creation to Phase 2, or rename it documentation-only and add phase acceptance proving the inventory command and doc were produced.
  - Question: Is Phase 1 truly read-only, or is it allowed to create/update the licensing-boundary doc before remediation begins?
  - Recommended answer: Phase 1 should be documentation-only, not read-only: it may create `docs/licensing-boundary.md` plus a classification manifest, with no source-code remediation until Phase 2.
  - If unanswered: Default to making Phase 1 a documentation-only write phase and add a machine-checkable inventory command as Phase 1 acceptance.
- [high/blocks approval] `harden-6` ownership-boundary-unclear - The private home requirement crosses repository ownership without an execution rule.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:185
  - Evidence: The spec requires a private-infra home outside the MIT-published workspace at lines 96-97 and 185-186. The named private file exists in sibling repo `../cloud/packages/auth/src/nango-hosted.ts`, not inside the OSS repo. The draft does not say whether build agents may edit sibling `../cloud`, only document it, or must coordinate with a separate cloud spec.
  - Recommendation: Pin the ownership boundary. If cross-repo edits are allowed, list exact cloud files and validation commands from the workspace root. If not, make this spec OSS-only and record cloud implementation as an external dependency/follow-up.
  - Question: May this task modify the sibling `../cloud` repository, or should it only document that private home from the OSS side?
  - Recommended answer: This spec should be OSS-only for implementation, but may read `../cloud` for classification and document `../cloud/packages/auth` as the private home; any cloud code move belongs in a separate cloud-owned spec.
  - If unanswered: Default to OSS-only edits: document `../cloud/packages/auth` as private home and open a separate cloud spec for any cloud code moves.
- [medium/advisory] `harden-7` rollback-too-general - Rollback is principled but not operational enough for a high-risk licensing task.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:262
  - Evidence: Rollback at lines 262-267 states the principle for ambiguous classification, but not concrete repair commands for partially applied Phase 2/3 changes such as moved connect client logic, license metadata changes, guard scripts, or CI wiring.
  - Recommendation: Add per-phase rollback bullets and name the operational recovery commands/tests to re-run after rollback.
  - Question: What concrete repair path should a human use if Phase 2 or Phase 3 fails after license metadata or connect-client moves have been applied?
  - Recommended answer: Add concrete rollback: restore previous `crates/Cargo.toml` license metadata, remove the guard test/script/CI step, revert connect-client relocation or abstraction, and re-run `scafld validate`, cargo tests, and the inventory guard.
  - If unanswered: Default to per-phase rollback: revert doc/manifest changes for Phase 1, revert source moves for Phase 2, and restore previous Cargo/license/CI files for Phase 3.

### round-2

Status: needs_revision
Started: 2026-05-21T15:37:09Z
Ended: 2026-05-21T15:37:09Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 verdict: needs revision. Round-1 fixes mostly land — paths normalize to OSS-repo-root, the MIT vs Apache-2.0 conflict is settled in a License Authority section, the guard now has explicit Check A/Check B semantics with a manifest and allowlist, Phase 1 is reframed documentation-only with a concrete acceptance command, and rollback gains per-phase repair. Three blocking gaps remain. (1) Phase 1's acceptance `p1a` invokes `.scafld/scripts/check-license-edges.mjs --check manifest-complete`, but that script is created only in Phase 3 — Phase 1 cannot meet its own acceptance gate. (2) Phase 2 quietly removes a published surface from `runx-runtime`'s public API: `lib.rs:61-65` re-exports `ConnectClient`, `ConnectError`, `ConnectOpener`, `HttpConnect*`, and `load_connect_options_from_env`, all of which the spec's "move to private" decision deletes, violating the `public_api_stable` invariant with no transition/compat plan named. (3) `connect/redaction.rs` is listed both under "relocate (private by default)" touchpoints (line 186) and the redaction test is listed under "keep MIT" (line 194), while the Risks section asserts redaction hygiene is MIT — the classification is ambiguous before Phase 1 begins. Two advisory issues: the negative-fixture's path/exclusion strategy is unnamed, and `crates/runx-runtime/src/connect.rs` (the module root that publicly re-exports all four submodules) is missing from touchpoints. Architecture is sound; cut the timing inversion, name the public-API break, and disambiguate redaction before approval.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:178
  - Result: passed
  - Evidence: Touchpoints now use OSS-repo-root style (`crates/...`, `docs/...`, `.scafld/...`) with sibling repos explicit as `../cloud/**` and `../plans/**`. Verified: `crates/runx-runtime/src/connect/{client,opener,redaction,types}.rs`, `crates/runx-runtime/src/credentials.rs`, `crates/runx-core/src/policy/connected_auth.rs`, `crates/runx-cli/tests/locality.rs`, `crates/runx-runtime/tests/connect_secret_redaction.rs`, and `../cloud/packages/auth/src/nango-hosted.ts` all exist. Round-1 path mismatch is resolved.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:283
  - Result: passed
  - Evidence: v3 (`node .scafld/scripts/check-license-edges.mjs --check identifiers`) and v4 (`cargo metadata --manifest-path crates/Cargo.toml --format-version 1 | node .scafld/scripts/check-license-edges.mjs --check edges`) point at the script Phase 3 creates and run from the OSS repo root. p3b `rg -n 'license\s*=\s*"MIT"' crates/Cargo.toml` will pass once Phase 3 flips the field (currently `Apache-2.0` per `crates/Cargo.toml:29`). p2b targets the existing `connect_secret_redaction` test (file exists). Commands are well-formed for the repo root they declare.
- scope/migration audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect.rs:1
  - Result: failed
  - Evidence: Scope correctly expands to inventory-driven via `rg` and lists the four connect submodules plus `runx-cli/src/main.rs` and `runx-sdk/src/client.rs`. However the module root `crates/runx-runtime/src/connect.rs` (which re-exports all four submodules at lines 10-20) is not in touchpoints, and `crates/runx-runtime/src/lib.rs:61-65` publicly re-exports `ConnectClient`, `ConnectError`, `ConnectOpener`, `HttpConnectGrant`, `HttpConnectListResponse`, `HttpConnectPreprovisionRequest`, `HttpConnectReadyResponse`, `HttpConnectRevokeResponse`, `load_connect_options_from_env`. Phase 2's 'move to private' decision silently deletes these from `runx-runtime`'s public API — a breaking change that conflicts with the `public_api_stable` invariant and is unaddressed in the spec.
- acceptance timing audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:309
  - Result: failed
  - Evidence: Phase 1 acceptance `p1a` runs `node .scafld/scripts/check-license-edges.mjs --check manifest-complete`, but Phase 3's Changes list explicitly says `Add .scafld/scripts/check-license-edges.mjs implementing Check A (identifiers) and Check B (edges) and --check manifest-complete` (line 346). The script does not exist at end-of-Phase-1, so Phase 1 acceptance is non-executable in order. Phase 2 acceptance is OK (p2a/p2b reference existing tests). Phase 3 acceptance is internally consistent.
- rollback/repair audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:359
  - Result: passed
  - Evidence: Per-phase rollback at lines 366-368 now names concrete repair steps: Phase 1 deletes doc + manifest and re-runs `scafld validate`; Phase 2 reverts the connect/OAuth relocation commit(s) and re-runs `cargo test -p runx-runtime -p runx-cli` plus the redaction guard; Phase 3 restores `crates/Cargo.toml` license, removes the script, the `license_boundary` test, and the CI step, then re-runs validate and the OSS test suite. Round-1 generality complaint is addressed.
- design challenge
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/credentials.rs:106
  - Result: passed
  - Evidence: The consumption-vs-brokerage principle is structurally honest: `MaterialResolver` at `credentials.rs:106-111` is genuinely opaque (`material_ref: &str -> ResolvedCredentialMaterial`), while `../cloud/packages/auth/src/nango-hosted.ts` is the concrete Nango broker — the licensed crossing is real, not invented. `ConnectGrantMaterialKind::NangoConnection` at `connect/types.rs:30-33` is the live boundary leak. This is the right architectural move and not a bandaid: a string-only guard is correctly rejected in favor of identifier-scan + dep-edge with a committed manifest, and the OSS license is reconciled to the repo-root MIT (`LICENSE:1`). Remaining gaps are executability, not direction.

Issues:
- [critical/blocks approval] `harden-1` phase-acceptance-timing-inversion - Phase 1 acceptance command depends on a script created only in Phase 3.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:309
  - Evidence: Phase 1 acceptance `p1a` (line 310) runs `node .scafld/scripts/check-license-edges.mjs --check manifest-complete`. Phase 3 Changes (line 346) state `Add .scafld/scripts/check-license-edges.mjs implementing Check A (identifiers) and Check B (edges) and --check manifest-complete`. The script does not exist at Phase 1; the gate is non-executable in declared order. `Glob .scafld/scripts/*.mjs` returned no files.
  - Recommendation: Either (a) move script creation into Phase 1 (Phase 1 may write `.scafld/scripts/check-license-edges.mjs --check manifest-complete` as a doc-shaped check while leaving Check A/B for Phase 3), or (b) make `p1a` a non-script gate that lists every crate present under `crates/` and asserts a matching `class` entry in the manifest via a one-liner the spec specifies inline (e.g., a jq diff), and keep the full script in Phase 3.
  - Question: Should the `manifest-complete` check ship in Phase 1 (separate from the full guard) so Phase 1 has an executable acceptance, or should `p1a` be replaced by a script-free check?
  - Recommended answer: Ship `.scafld/scripts/check-license-edges.mjs --check manifest-complete` in Phase 1 (the manifest is also a Phase 1 artifact, so its validator is in scope), then extend the same script with `--check identifiers` and `--check edges` in Phase 3.
  - If unanswered: Default to shipping `--check manifest-complete` in Phase 1 so the script that validates the manifest is born with the manifest.
- [high/blocks approval] `harden-2` public-api-breaking-change-unaddressed - Phase 2 silently removes published items from `runx-runtime`'s public API with no compat plan; conflicts with `public_api_stable` invariant.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/lib.rs:61
  - Evidence: `runx-runtime/src/lib.rs:61-65` publicly re-exports `ConnectClient`, `ConnectError`, `ConnectOpener`, `HttpConnectGrant`, `HttpConnectListResponse`, `HttpConnectPreprovisionRequest`, `HttpConnectReadyResponse`, `load_connect_options_from_env`. `crates/runx-runtime/src/connect.rs:10-20` re-exports all four submodules including `ConnectGrantMaterialKind::NangoConnection`. Phase 2 Changes (line 324) say `Move or abstract the connect/OAuth brokerage code (connect/client.rs, connect/types.rs, connect/opener.rs ...); OSS retains only the opaque-ref consumption path`. The spec does not say whether the public re-exports survive as empty shells, get deprecated and held for one release, or are deleted outright. Project invariants list `public_api_stable`.
  - Recommendation: Add an explicit subsection (under Phase 2 or Scope) that names, by symbol, every currently-public re-export the relocation removes; for each, choose one of: (i) delete (acknowledged break, requires bump), (ii) keep an MIT-side opaque shim that calls into the resolver and never touches Nango, (iii) deprecate-then-delete across two releases. Coordinate with `rust-placeholder-crates-publish` since this affects the next publishable shape.
  - Question: What is the public-API contract for the `runx-runtime` `connect::*` re-exports after Phase 2: delete, opaque shim, or deprecate-then-delete?
  - Recommended answer: Delete with an acknowledged break (these crates are pre-1.0 — `runx-runtime` is at version `0.0.1` per `crates/Cargo.toml:18`) and add a one-line note in Phase 2 saying so; the boundary doc records which symbols were removed and where their replacement now lives.
  - If unanswered: Default to acknowledged break: name the removed re-exports in the boundary doc and rely on pre-1.0 versioning to license the change.
- [high/blocks approval] `harden-3` classification-ambiguity-pre-phase-1 - `connect/redaction.rs` is classified inconsistently in the spec itself before Phase 1 begins.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:186
  - Evidence: Touchpoints list `crates/runx-runtime/src/connect/redaction.rs` under 'Connect/OAuth surface to classify and relocate-or-abstract (private by default)' at line 186, but Risks (line 222) says 'redaction hygiene and the connected-auth requirement policy are legitimately MIT'. The retained MIT touchpoint group at line 194 lists `crates/runx-runtime/tests/connect_secret_redaction.rs` — a test that exercises redaction. A test cannot stay MIT if the implementation it imports has moved to private; the inversion either traps the test on the wrong side or implies `redaction.rs` actually stays MIT and the touchpoint list is wrong.
  - Recommendation: Move `connect/redaction.rs` out of the 'relocate (private by default)' bullet and into the 'keep MIT' bullet to match the Risks statement, or update Risks if the operator actually wants redaction to move. Either resolution should be made in the spec before Phase 1 starts so the inventory doesn't have to relitigate it.
  - Question: Is `connect/redaction.rs` MIT (matching Risks) or private (matching touchpoint placement)?
  - Recommended answer: MIT. Redaction is non-leakage hygiene and is consumed by the runtime regardless of brokerage. Update Touchpoints to move it under the MIT-keep group alongside its test.
  - If unanswered: Default to MIT for `connect/redaction.rs` and update Touchpoints accordingly.
- [medium/advisory] `harden-4` guard-fixture-policy-undefined - Negative-fixture location and Check A exclusion is unspecified, risking self-collision.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:347
  - Evidence: Phase 3 (line 347) requires a 'negative fixture proving the guard fails on a planted violation', and Guard Semantics (line 240) says 'Tests and fixtures are in scope; historical protocol names survive only as explicit, rationale-bearing allowlist entries.' If the planted-violation fixture lives anywhere under an MIT crate's `tests/`, Check A will fire on it on every run unless excluded via `exclude_globs` or routed through the allowlist. The spec lists `exclude_globs` in the manifest schema but does not commit a fixture location or naming convention.
  - Recommendation: Pin the fixture path (e.g., `crates/runx-runtime/tests/fixtures/license_boundary/planted_violation.rs`) and add the canonical `exclude_globs` entry to the manifest seed in Phase 1, so the fixture is born already excluded. The test then either copies the fixture into a tempdir and runs the guard against it, or invokes the guard with an override path — never against the source tree.
  - Question: Where do the negative fixtures live, and how are they kept invisible to Check A's normal scan?
  - Recommended answer: Park them under a dedicated fixtures dir excluded by an explicit `exclude_globs` entry in the manifest (seeded in Phase 1), and have the boundary test run the guard against a temp copy or with an explicit path override.
  - If unanswered: Default to `crates/runx-runtime/tests/fixtures/license_boundary/**` excluded via the manifest's `exclude_globs`, exercised by the boundary test via a tempdir copy.
- [medium/advisory] `harden-5` touchpoint-coverage-gap - `crates/runx-runtime/src/connect.rs` (module root) is missing from touchpoints despite owning the re-exports the relocation deletes.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect.rs:1
  - Evidence: `crates/runx-runtime/src/connect.rs:1-20` declares the `connect` module and `pub use`s symbols from `client`, `opener`, `redaction`, and `types`. Touchpoints (lines 183-189) name the four submodules and `runx-cli/src/main.rs` but omit `connect.rs` itself, which must be edited (or removed) in Phase 2 because it currently re-exports symbols that will no longer exist.
  - Recommendation: Add `crates/runx-runtime/src/connect.rs` and `crates/runx-runtime/src/lib.rs` (re-exports at line 61-65) to the Touchpoints list so Phase 2 cannot leave dangling references.
  - Question: Should `crates/runx-runtime/src/connect.rs` and the `lib.rs` re-export block at line 61 be explicit touchpoints?
  - Recommended answer: Yes. Add both to the relocate/abstract group; they must be edited in lockstep with the submodule moves.
  - If unanswered: Default to adding both files to touchpoints.
- [low/advisory] `harden-6` acceptance-filter-may-vacuously-pass - Phase 2 acceptance `p2a` uses a name filter that may match zero tests in `runx-cli`.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:331
  - Evidence: `p2a` (line 332) runs `cargo test --manifest-path crates/Cargo.toml -p runx-runtime -p runx-cli credential`. `runx-runtime` has `tests/credential_delivery.rs`; under `runx-cli/tests/` no file or test name matches `credential` (verified via Glob). `cargo test <FILTER>` exits 0 when no tests match the filter, so the `runx-cli` half of this acceptance is vacuous evidence.
  - Recommendation: Either drop `-p runx-cli` from `p2a` and add a separate `p2c` running a concrete CLI test that proves the offline `runx connect --token` path still works after the relocation (e.g., a new integration test under `crates/runx-cli/tests/`), or rename the filter to match an existing CLI test.
  - Question: What concrete CLI test proves the offline `runx connect --token` path survives Phase 2?
  - Recommended answer: Either ship a `crates/runx-cli/tests/connect_offline.rs` and target it explicitly, or drop the `-p runx-cli credential` filter and rely on Phase 2's redaction guard plus a runtime-level offline test.
  - If unanswered: Default to dropping the `-p runx-cli` half of `p2a` and tightening to `-p runx-runtime --test credential_delivery`.

### round-3

Status: needs_revision
Started: 2026-05-21T15:49:30Z
Ended: 2026-05-21T15:49:30Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-3 verdict: needs revision. Round-2 fixes mostly land: Phase 1 now ships `--check manifest-complete` alongside the manifest (timing inversion fixed), the Public API Transition section names the deletions, `connect/redaction.rs` is unambiguously MIT, the negative-fixture path is committed, and p2a/p2c are tightened. Three remaining gaps block approval and three are advisory. (1) p2b targets `crates/runx-runtime/tests/connect_secret_redaction.rs`, but that test directly constructs `ConnectClient` and uses `ConnectError` / `HttpConnectPreprovisionRequest` — precisely the symbols the Public API Transition deletes. The test cannot compile after Phase 2 unless rewritten to exercise `redact_connect_text()` directly or moved with the client; the spec is silent on which. (2) p2c targets `crates/runx-cli/tests/launcher.rs`, which imports `runx_cli::connect::{ConnectAction, ConnectAuthorityKind, ConnectPlan}` and asserts `routes_connect_to_native_plan`; `runx-cli/src/main.rs:39-117` is also wired to `runx_runtime::connect::*`. Phase 2 says the "`runx connect` OAuth arm" moves, but doesn't name what replaces it in MIT — a stub that errors? a feature-gated cut? deletion plus test prune? p2c is non-executable until that's pinned. (3) Touchpoints list only `connect_secret_redaction.rs` from the runtime test suite, but `connect_client.rs`, `connect_support.rs`, and `connect_policy_integration.rs` all import the same brokerage symbols and will compile-break in Phase 2; their disposition needs to be in the plan, not discovered mid-build. Advisories: the "offline path" wording conflates declared-grants (no network) with the `RUNX_CONNECT_ACCESS_TOKEN` route (still requires `RUNX_CONNECT_BASE_URL`, so it hits the hosted broker); `runx connect --token` is referenced three times but isn't an actual CLI flag; and `crates/deny.toml` still allows `Apache-2.0` for transitive deps — fine, but worth noting in the boundary doc so the next reviewer doesn't mistake it for a violation. Architecture remains right; tighten the test/CLI cut plan and clean up the language before approval.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:178
  - Result: passed
  - Evidence: Touchpoints use OSS-repo-root style. Verified existence of crates/runx-runtime/src/connect.rs (line 1-20), crates/runx-runtime/src/lib.rs:61-65 (public re-exports), crates/runx-runtime/src/connect/{client,opener,redaction,types}.rs, crates/runx-runtime/src/credentials.rs (MaterialResolver at line 106), crates/runx-cli/src/connect.rs, crates/runx-cli/tests/{locality,launcher,connect}.rs, crates/runx-runtime/tests/connect_secret_redaction.rs, crates/Cargo.toml. The Phase 3 future paths .scafld/scripts/check-license-edges.mjs and docs/license-boundary.manifest.json are intentionally future files. Round-2 path complaint is resolved.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:283
  - Result: failed
  - Evidence: p1a (line 344) now correctly runs the manifest-complete check that Phase 1 ships (timing inversion fixed). p2a (line 369) targets the real test crates/runx-runtime/tests/credential_delivery.rs with the right feature gates (`#![cfg(all(feature = "cli-tool", any(feature = "mcp", feature = "mcp-rmcp")))]` at line 1). p3b (`rg -n 'license\s*=\s*"MIT"' crates/Cargo.toml`) will succeed once Phase 3 flips line 29 from Apache-2.0 to MIT. However p2b (`cargo test ... --test connect_secret_redaction`) and p2c (`cargo test ... -p runx-cli --test launcher`) are non-executable as written after Phase 2: the named tests directly depend on symbols the Public API Transition deletes. See acceptance timing audit and issues harden-1 / harden-2.
- scope/migration audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/tests/connect_client.rs:4
  - Result: failed
  - Evidence: Touchpoints name only one runtime connect test (connect_secret_redaction.rs at line 196). But three sibling tests under crates/runx-runtime/tests/ also depend on the moving brokerage surface: connect_client.rs:4 imports `ConnectClient, ConnectError, HttpConnectPreprovisionRequest`; connect_support.rs is the shared mock (FailingOpener, MockConnectTransport, RecordingOpener) consumed by all three; connect_policy_integration.rs:8 imports `runx_runtime::connect::connect_grant_to_local_admission`. crates/runx-sdk/tests/client_cli.rs:62 uses `client.connect_list()` and depends on the SDK connect_list API. None of these are listed upfront with a keep/abstract/move decision. Phase 1's `rg` inventory will surface them, but Phase 2's plan needs to dispose of them before it builds, not after.
- acceptance timing audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/tests/connect_secret_redaction.rs:3
  - Result: failed
  - Evidence: p2b runs `cargo test ... --test connect_secret_redaction` and expects exit_code_zero. The test at connect_secret_redaction.rs:3 imports `ConnectClient, ConnectError, HttpConnectPreprovisionRequest` and at lines 16-23 constructs a `ConnectClient::with_transport_and_opener(...)`. Public API Transition (spec line 270-273) explicitly removes `ConnectClient`, `ConnectError`, and `HttpConnectPreprovisionRequest` from `runx-runtime` in Phase 2. The test will not compile at the end of Phase 2 unless the spec also commits to either (a) rewriting it to exercise `redact_connect_text()` directly with no client, or (b) moving it with the client (contradicting the line-196 MIT-keep placement). p2c is parallel: launcher.rs:2 imports `runx_cli::connect::{ConnectAction, ConnectAuthorityKind, ConnectPlan}` and tests `routes_connect_to_native_plan` at line 149 — both vanish if the runx connect arm moves to private.
- rollback/repair audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:407
  - Result: passed
  - Evidence: Per-phase repair at lines 409-414 is operational: Phase 1 deletes docs/licensing-boundary.md and docs/license-boundary.manifest.json; Phase 2 reverts the relocation commit(s) and re-runs `cargo test -p runx-runtime -p runx-cli` plus the redaction guard; Phase 3 restores the previous license value, removes `.scafld/scripts/check-license-edges.mjs`, the `license_boundary` test, and the CI step, then re-runs validate and the OSS test suite. General rule at line 405 (mark ambiguous layers private by default; never weaken the offline path or CredentialDelivery channel) remains intact. Round-2 generality complaint addressed.
- design challenge
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/credentials.rs:106
  - Result: passed
  - Evidence: The consumption-vs-brokerage principle holds structurally: `MaterialResolver` at credentials.rs:106-111 is genuinely opaque (`resolve_material(material_ref: &str) -> ResolvedCredentialMaterial`), while `connect/types.rs:30-33` (`ConnectGrantMaterialKind::NangoConnection`) and `connect/client.rs:329-351` (`load_connect_options_from_env` requiring `RUNX_CONNECT_BASE_URL` + `RUNX_CONNECT_ACCESS_TOKEN`) are concrete brokerage and not consumption. The cloud broker at ../cloud/packages/auth/src/nango-hosted.ts:75-87 is the natural other side of the seam. A single guard with manifest + identifier scan + dep-edge check (with a committed `exclude_globs` for negative fixtures) is the right shape — string-match-only would be evadable, and the dep-edge check catches the harder case. The OSS license reconciliation to MIT is internally consistent (root LICENSE is MIT; deny.toml already allows MIT). Remaining gaps are executability of the cutover, not design direction.

Issues:
- [critical/blocks approval] `harden-1` phase-acceptance-non-executable - p2b targets a test that imports the exact symbols Phase 2 deletes; the test cannot compile at the end of Phase 2 without an explicit rewrite/move directive.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/tests/connect_secret_redaction.rs:3
  - Evidence: Phase 2 acceptance p2b (spec line 372-373) runs `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test connect_secret_redaction`. The test at crates/runx-runtime/tests/connect_secret_redaction.rs:3 has `use runx_runtime::{ConnectClient, ConnectError, HttpConnectPreprovisionRequest};` and at lines 16-23 constructs `ConnectClient::with_transport_and_opener(...)` to drive the redaction assertions. The Public API Transition section (spec lines 270-273) explicitly names `ConnectClient`, `ConnectError`, and `HttpConnectPreprovisionRequest` as removed by Phase 2. The spec lists this test under 'Consumption/enforcement surface to keep MIT' at line 196 but doesn't say how the test gets there once its dependencies leave: redaction.rs itself (line 1-25) is a generic text-redactor with no client dependency, so the test could be rewritten to exercise `redact_connect_text()` directly — but the spec doesn't commit to that.
  - Recommendation: Pin the disposition in Phase 2 Changes: either (a) add a step 'Refactor connect_secret_redaction.rs to exercise `redact_connect_text` directly (no ConnectClient) so non-leakage hygiene stays MIT-testable', or (b) move the test with the client and acknowledge the redaction module's MIT classification stands on its own without an in-tree test, or (c) keep an opaque MIT shim for the deleted types. Whichever, name it before Phase 2 starts so p2b is non-vacuous.
  - Question: What is the disposition of crates/runx-runtime/tests/connect_secret_redaction.rs after Phase 2: rewrite without ConnectClient, move with the client, or keep via an MIT shim?
  - Recommended answer: Rewrite to exercise redact_connect_text() directly. Redaction is a pure text utility (no client dependency) and the MIT-side test should reflect that. Add this as an explicit Phase 2 change and update Touchpoints to flag the rewrite.
  - If unanswered: Default to (a): rewrite the test to exercise redact_connect_text() directly; add to Phase 2 Changes.
- [high/blocks approval] `harden-2` cli-surface-undefined-post-cutover - Phase 2 moves the `runx connect` OAuth arm but doesn't say what replaces it in MIT; p2c (the launcher test) imports symbols that will no longer exist.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-cli/tests/launcher.rs:2
  - Evidence: p2c (spec line 375-376) runs `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test launcher`. launcher.rs:2 has `use runx_cli::connect::{ConnectAction, ConnectAuthorityKind, ConnectPlan};` and line 149 declares `fn routes_connect_to_native_plan()` which asserts `LauncherAction::RunConnect(ConnectPlan { ... })`. crates/runx-cli/src/main.rs:39 routes `LauncherAction::RunConnect(plan) => run_native_connect(plan)` and lines 50-105 implement `run_native_connect` against `runx_runtime::ConnectClient`. Phase 2 Changes (spec line 358) says 'Move or abstract ... the `runx connect` OAuth arm' but doesn't pin the post-cutover shape: stub that errors at runtime? deleted entirely with the launcher test pruned? feature-gated and off in the default OSS build? The spec lists `runx-cli/src/connect.rs` and 'the connect arm in `crates/runx-cli/src/main.rs`' under private/relocate at lines 188 — implying removal — but p2c expects the launcher test to still pass.
  - Recommendation: Add a 'CLI Surface Transition' subsection (or extend Public API Transition) naming the post-cutover state of `runx_cli::connect` and `LauncherAction::RunConnect`. Pick one: (i) delete the subcommand and prune `routes_connect_to_native_plan` from launcher.rs as part of Phase 2; (ii) keep an MIT-side stub that prints 'connect is provided by runx hosted infra' and route through it; (iii) feature-gate the entire CLI connect surface behind an opt-in flag. Whichever option, p2c must target a test that still compiles.
  - Question: What does the `runx connect` subcommand look like in MIT-only OSS after Phase 2: deleted, stub, or feature-gated?
  - Recommended answer: Delete with an acknowledged break (runx-cli is also pre-1.0). Remove the `runx connect` arm from main.rs, delete crates/runx-cli/src/connect.rs and crates/runx-cli/tests/connect.rs, prune `routes_connect_to_native_plan` and the `runx_cli::connect` import from launcher.rs. The boundary doc records that `runx connect` lives in the hosted/private CLI distribution. Phase 2 Changes must enumerate these CLI file deletions so p2c becomes executable on a coherent post-cutover tree.
  - If unanswered: Default to (i): delete the OSS-side `runx connect` subcommand and prune launcher.rs alongside the relocation; record in the boundary doc that connect is a hosted/private CLI feature.
- [medium/blocks approval] `harden-3` incomplete-test-touchpoints - Three additional runtime tests plus an SDK test depend on the moving brokerage surface but are not enumerated in touchpoints with keep/abstract/move decisions.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/tests/connect_client.rs:4
  - Evidence: Beyond the named connect_secret_redaction.rs, the runtime test directory contains: connect_client.rs (line 4: `use runx_runtime::{ConnectClient, ConnectError, HttpConnectPreprovisionRequest};`), connect_support.rs (the shared mock module providing FailingOpener, MockConnectTransport, RecordingOpener, grant_fixture — required by connect_client.rs, connect_secret_redaction.rs, and connect_policy_integration.rs), and connect_policy_integration.rs (line 8: `use runx_runtime::connect::connect_grant_to_local_admission;`). The SDK side has crates/runx-sdk/tests/client_cli.rs:62-74 exercising `client.connect_list()`. Touchpoints (spec lines 183-196) mention `crates/runx-sdk/tests/client_cli.rs` but not the three runtime tests. Phase 1's `rg` inventory will surface them, but Phase 2 has no upfront decision per file — Phase 2 will need to decide mid-build whether to move, rewrite, or delete them.
  - Recommendation: Extend Touchpoints with explicit listings for `crates/runx-runtime/tests/connect_client.rs`, `crates/runx-runtime/tests/connect_support.rs`, and `crates/runx-runtime/tests/connect_policy_integration.rs`, each with a keep/abstract/move decision (default: move with the client, except connect_policy_integration.rs which tests the MIT-keep policy and may need refactoring to drop its `connect_grant_to_local_admission` import or have that helper kept MIT-side).
  - Question: Should `connect_client.rs`, `connect_support.rs`, and `connect_policy_integration.rs` move with the client, or does `connect_policy_integration.rs` need a refactor to keep the MIT-policy test in OSS?
  - Recommended answer: Move connect_client.rs and connect_support.rs to private alongside the client. For connect_policy_integration.rs, classify `connect_grant_to_local_admission` itself: if the conversion helper stays MIT (it's a pure structural mapping from the envelope into LocalAdmissionGrant), the test stays MIT and only the `connect_support::grant_fixture` import gets inlined or kept MIT-side. Record both decisions in Touchpoints before Phase 1 inventory.
  - If unanswered: Default to moving the brokerage tests private and keeping connect_grant_to_local_admission MIT (it's a pure conversion), refactoring connect_policy_integration.rs to inline its fixture.
- [medium/advisory] `harden-4` offline-path-terminology - The spec's 'offline path' wording conflates two distinct routes; the access-token route still requires a hosted broker URL.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect/client.rs:329
  - Evidence: Spec line 31-32, 44-45, and 363 refer to 'the offline/declared-grant path that lets the runtime run with no account' and the `RUNX_CONNECT_ACCESS_TOKEN` mechanism as part of that path. But `load_connect_options_from_env` at connect/client.rs:329-351 requires both `RUNX_CONNECT_BASE_URL` and `RUNX_CONNECT_ACCESS_TOKEN` — it errors with `MissingConfiguration` if either is absent. The access-token route is 'BYO hosted access token, skipping the OAuth browser flow' — it still talks to a hosted broker. The genuinely offline path is declared-grants plus the opaque resolver with no hosted dependency.
  - Recommendation: Disambiguate the terminology. Reserve 'offline' for the declared-grants-only path. Call the env-var route 'BYO hosted access token' or 'pre-authorized hosted credential' so the boundary doc isn't misread as guaranteeing zero network on the access-token path. Phase 2's confirmation step (line 363) should test the truly-offline declared-grant path specifically.
  - Question: Should the spec separate 'offline (declared grants, no network)' from 'BYO hosted token (no OAuth browser, still hits broker)' to keep the boundary claim accurate?
  - Recommended answer: Yes. Update Summary, Default split, and Phase 2's confirmation step to call them out as two distinct routes. The MIT-keep claim is strongest for declared-grants-only; the BYO-token route is still MIT-consumable but is not 'offline'.
  - If unanswered: Default to splitting the wording: declared-grants is 'offline', RUNX_CONNECT_ACCESS_TOKEN is the 'BYO hosted token' path; both are MIT-consumable but only the first runs without the broker.
- [low/advisory] `harden-5` spec-references-nonexistent-cli-flag - `runx connect --token` is referenced 3 times in the spec but no such CLI flag exists in code.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/connect.rs:42
  - Evidence: Spec lines 31, 45, and 363 cite `runx connect --token` as the offline credential mechanism. Grepping `--token` across the OSS tree returns only this spec — no hits in crates/. crates/runx-cli/src/connect.rs:42-60 (`parse_connect_plan`) parses subcommands list/revoke/preprovision plus `--provider`, `--grant-id`, `--scope`, `--authority`, `--target-repo`, `--target-locator`, `--json` — no `--token`. The actual access-token mechanism is the `RUNX_CONNECT_ACCESS_TOKEN` env var consumed by `load_connect_options_from_env`.
  - Recommendation: Replace all three `runx connect --token` references with `RUNX_CONNECT_ACCESS_TOKEN` (or strike them entirely if the access-token route is reframed as 'BYO hosted token' per harden-4). The spec should not name a flag that doesn't exist.
  - Question: Should the spec be edited to reference `RUNX_CONNECT_ACCESS_TOKEN` instead of the non-existent `runx connect --token` flag?
  - Recommended answer: Yes. The env var is the real mechanism; the spec should match the code surface that the boundary actually classifies.
  - If unanswered: Default to replacing every `runx connect --token` reference with `RUNX_CONNECT_ACCESS_TOKEN` env var wording.
- [low/advisory] `harden-6` deny-toml-license-allowlist-untouched - When the workspace flips to MIT, `crates/deny.toml` still allows `Apache-2.0` for transitive deps; this is correct but the boundary doc should explain it.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/deny.toml:26
  - Evidence: crates/deny.toml:26-32 has `[licenses] allow = ["Apache-2.0", "BSD-3-Clause", "MIT", "Unicode-3.0"]`. After Phase 3 flips the workspace `license` from `Apache-2.0` to `MIT`, the deny.toml allowlist intentionally still permits Apache-2.0 for transitive crate dependencies (license compatibility, not license identity). The spec doesn't list deny.toml in Touchpoints and doesn't explain that this is intentional, so a future reviewer might mistake it for an inconsistency.
  - Recommendation: Add a one-line note in the boundary doc (or under License Authority) clarifying that `crates/deny.toml`'s `[licenses] allow` list governs transitive-dependency license compatibility and intentionally remains broader than the workspace's declared license. No code change needed; just an explanatory paragraph so the next pass doesn't get confused.
  - Question: Should the boundary doc note that deny.toml's license allowlist intentionally stays broader than the workspace license?
  - Recommended answer: Yes. Add a single sentence to the boundary doc explaining the deny.toml allowlist governs transitive dependency licenses (compatibility) and the workspace `license` field governs the crate's own metadata (identity). They are different axes.
  - If unanswered: Default to adding that explanatory sentence to the boundary doc when it's written in Phase 1.

### round-4

Status: passed
Started: 2026-05-21T15:59:05Z
Ended: 2026-05-21T15:59:05Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-4 verdict: pass. Round-3's blocking findings are resolved: Phase 2 Changes now explicitly direct the rewrite of `connect_secret_redaction.rs` to exercise `redact_connect_text()` instead of `ConnectClient`; the three sibling runtime tests (`connect_client.rs`, `connect_support.rs`, `connect_policy_integration.rs`) carry explicit move/delete/rewrite directives; the OSS `runx connect` CLI surface has a named delete-or-stub directive with launcher/connect test updates; the offline-vs-BYO-hosted-token terminology is disambiguated; and Phase 3 records the `crates/deny.toml` allowlist clarification. Public API Transition names every deleted re-export, Phase 1 owns `--check manifest-complete` so its own acceptance is executable, and the negative-fixture path is committed under `exclude_globs`. Paths verified against the OSS tree, p2a targets the real `credential_delivery` test with correct feature gates, and rollback is per-phase operational. Advisory issues remain: (1) `connect_grant_to_local_admission` is implicitly MIT-keep but currently consumes `HttpConnectGrant` which the Public API Transition deletes — Phase 2 will need to either lift the conversion onto an MIT-side grant shape or keep `HttpConnectGrant` MIT; (2) `connect/types.rs` mixes the `NangoConnection` enum (private) with `connect_grant_to_local_admission` (MIT-leaning), so Phase 1's inventory will need to split the file or pick one classification with rationale; (3) `p2c`'s `--test connect` target name must be preserved if Phase 2 deletes the CLI connect file outright. None block approval — they are foreseen implementation details the existing Changes lines bound. Architecture remains sound; spec is approval-ready.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect.rs:1
  - Result: passed
  - Evidence: All touchpoints exist relative to the OSS repo root. Verified: crates/runx-runtime/src/connect.rs (declares mod client/opener/redaction/types and re-exports the brokerage surface at lines 10-20), crates/runx-runtime/src/lib.rs:61-65 (public connect re-exports), crates/runx-runtime/src/connect/{client,opener,redaction,types}.rs, crates/runx-runtime/src/credentials.rs (opaque MaterialResolver at line 106), crates/runx-runtime/tests/{connect_secret_redaction,connect_client,connect_support,connect_policy_integration,credential_delivery}.rs, crates/runx-cli/src/connect.rs, crates/runx-cli/tests/{connect,launcher,locality}.rs, crates/runx-sdk/src/client.rs, crates/runx-core/src/policy/connected_auth.rs, crates/Cargo.toml. Sibling reference ../cloud/packages/auth/src/nango-hosted.ts is documented as private home, no edits required. Future files .scafld/scripts/check-license-edges.mjs, docs/licensing-boundary.md, docs/license-boundary.manifest.json, crates/runx-runtime/tests/fixtures/license_boundary/** are intentional Phase 1/3 creations.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:357
  - Result: passed
  - Evidence: p1a (`node .scafld/scripts/check-license-edges.mjs --check manifest-complete`) targets the script Phase 1 ships per Phase 1 Changes line 352. p2a runs `cargo test ... -p runx-runtime --test credential_delivery --features cli-tool,mcp`; crates/runx-runtime/tests/credential_delivery.rs:1 has matching cfg gate. p2b targets the redaction test that Phase 2 Changes line 378-380 rewrites to use redact_connect_text() directly — rewrite is named, so post-Phase-2 compile is in scope. p2c runs `--test launcher && --test connect` which Phase 2 Changes line 387-390 directs to update for the chosen CLI behavior. p3a targets a Phase 3 new test. p3b uses an rg pattern that matches once Phase 3 line 416 flips crates/Cargo.toml from Apache-2.0 to MIT. v3/v4 reference the Phase 3-extended script.
- scope/migration audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect/types.rs:30
  - Result: passed
  - Evidence: Touchpoints now enumerate every Rust file that materially crosses the boundary: connect.rs (module root), lib.rs (re-export block), connect/{client,opener,redaction,types}.rs (submodules including NangoConnection at types.rs:30), runx-cli/src/connect.rs, runx-cli/src/main.rs (connect arm), runx-sdk/src/client.rs, plus the four runtime tests and the CLI tests. Phase 2 Changes (lines 372-395) name explicit dispositions for each: move the brokerage submodules; rewrite redaction test; move/delete/rewrite connect_client.rs, connect_support.rs, connect_policy_integration.rs; delete-or-stub the CLI connect arm. Public API Transition (lines 272-294) enumerates every removed public symbol. Risks (line 226) flags classification drift via the guard's manifest-completeness check.
- acceptance timing audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:347
  - Result: passed
  - Evidence: Round-2's timing inversion is fixed: Phase 1 Changes line 352-353 ships `.scafld/scripts/check-license-edges.mjs --check manifest-complete`, which p1a (line 359) then invokes — script and gate are born together. Phase 3 Changes line 417-419 only extends the same script with Check A and Check B. Phase 2 acceptance commands p2a/p2b/p2c all target tests whose updated form is mandated in Phase 2 Changes, so the gates are non-vacuous post-build. p3a/p3b run on Phase 3 artifacts. Strict profile is honored: each phase has at least one machine-checkable acceptance criterion.
- rollback/repair audit
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:441
  - Result: passed
  - Evidence: Per-phase repair (lines 441-447) names concrete steps. Phase 1: delete docs/licensing-boundary.md and docs/license-boundary.manifest.json, re-run scafld validate (no source touched). Phase 2: revert the relocation commit(s) to restore the moved modules and public re-exports in connect.rs/lib.rs, re-run `cargo test -p runx-runtime -p runx-cli` plus the redaction guard. Phase 3: restore previous crates/Cargo.toml license, remove the script, the license_boundary test, and the CI step, re-run validate and the OSS test suite. General rule (line 437-439): if a layer cannot be cleanly classified, mark it private by default; never weaken the offline path or CredentialDelivery channel.
- design challenge
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/credentials.rs:106
  - Result: passed
  - Evidence: The consumption-vs-brokerage seam is structurally honest. crates/runx-runtime/src/credentials.rs defines MaterialResolver around an opaque `material_ref: &str -> ResolvedCredentialMaterial` (verified at line 106). The brokerage side is concrete: connect/types.rs:30 (`NangoConnection`), connect/client.rs (OAuth polling, `RUNX_CONNECT_ACCESS_TOKEN` env-var), the cloud broker at ../cloud/packages/auth/src/nango-hosted.ts. A guard with manifest + identifier scan + dep-edge check (with negative fixture in tests/fixtures/license_boundary/**) is the right shape — string-only would be evadable. The OSS license reconciliation to MIT matches the root LICENSE. The spec is the right architectural move, not a bandaid: it codifies a boundary the code already implies and makes it durable via CI.

Issues:
- [medium/advisory] `harden-1` classification_ambiguity - connect_grant_to_local_admission is implicitly MIT-keep but consumes HttpConnectGrant, which Public API Transition deletes.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect/types.rs:302
  - Evidence: Phase 2 Changes line 381-386 require: 'Any retained policy integration test must depend only on a MIT grant-to-local-admission conversion, not Nango/hosted client types.' The only such conversion in code is `connect_grant_to_local_admission` at crates/runx-runtime/src/connect/types.rs:302-320, which takes `&HttpConnectGrant`. The Public API Transition (lines 277-281) explicitly names HttpConnectGrant in the may-be-removed set. crates/runx-runtime/tests/connect_policy_integration.rs:8 imports the helper and at line 11 imports `connect_support::grant_fixture` (also moving). To honor both directives, Phase 2 must either (a) lift the conversion onto an MIT-side envelope/grant shape distinct from HttpConnectGrant, (b) keep HttpConnectGrant MIT (and reclassify the file by extracting NangoConnection), or (c) move the helper private and rewrite the policy test against a different MIT seam.
  - Recommendation: Phase 1's classification doc should pin which of (a)/(b)/(c) Phase 2 takes. The natural fit is (b)-with-split: extract NangoConnection (the lone private identifier) into a private location, keep HttpConnectGrant + connect_grant_to_local_admission MIT as wire/conversion utilities consumed by the policy seam. Record the split in the manifest's allowlist or in a refactored connect/types module layout.
  - Question: Should HttpConnectGrant + connect_grant_to_local_admission stay MIT (with NangoConnection extracted to private) or move private alongside the brokerage?
  - Recommended answer: Keep MIT and extract NangoConnection to private. HttpConnectGrant is a wire shape the runtime consumes; the helper is a pure structural conversion. NangoConnection is the only private-brokerage indicator in the file and can be lifted out.
  - If unanswered: Default to keeping HttpConnectGrant and connect_grant_to_local_admission MIT; extract NangoConnection (and any other Nango-specific enum variants) to a private module. Update Phase 1 classification doc to record this split before Phase 2 begins.
- [low/advisory] `harden-2` test_target_robustness - p2c's `cargo test ... -p runx-cli --test connect` requires the connect test target to exist; full deletion of crates/runx-cli/tests/connect.rs breaks the acceptance command.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/connect-auth-mit-boundary-v1.md:404
  - Evidence: Phase 2 Changes line 387-390 says 'Delete native `runx connect` brokerage routing from the OSS CLI, or replace it with an explicit unavailable-in-OSS stub'. If the implementer chooses 'delete' and removes crates/runx-cli/tests/connect.rs, then `cargo test --test connect` errors with `no test target named 'connect'`, not exit_code_zero. The 'or stub' branch leaves the file in place with rewritten assertions, which p2c can pass.
  - Recommendation: Either commit to the stub variant (which keeps the test target name alive) or rename p2c's target to a stable test file that exists in both variants. A small refinement: make p2c run `cargo test -p runx-cli` (no `--test` filter) so it passes regardless of whether the connect file survives, plus a dedicated assertion in the boundary doc that the chosen behavior is recorded.
  - Question: Should Phase 2 commit to the stub variant (preserving crates/runx-cli/tests/connect.rs as a target name) or should p2c drop the --test connect filter?
  - Recommended answer: Drop the `--test connect` filter and run the whole runx-cli test crate so p2c is robust under either delete or stub. The stub vs delete decision can then be made at implementation time without invalidating the acceptance gate.
  - If unanswered: Default to dropping `--test connect` from p2c so the acceptance passes whether the file survives as a stub or is deleted alongside the CLI subcommand.
- [low/advisory] `harden-3` inventory_completeness - Phase 1's rg inventory pattern catches files but the classification doc needs to break apart mixed-class files like connect/types.rs.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/connect/types.rs:302
  - Evidence: Phase 1 Changes line 348 runs `rg -n 'nango|oauth|RUNX_CONNECT|connection_id|material_ref|credential|auth' crates --glob '*.rs'`, then labels matched surfaces mit-oss or private. But crates/runx-runtime/src/connect/types.rs holds both a private indicator (NangoConnection at line 30-33) and the MIT-leaning helper connect_grant_to_local_admission at line 302-320. A file-level label is too coarse here. The manifest schema supports identifier-level allowlists but the Phase 1 Changes don't explicitly call out mixed-class files needing a split.
  - Recommendation: Add a Phase 1 Changes bullet: 'For files that mix mit-oss and private symbols (notably connect/types.rs), record the per-symbol split and either plan a Phase 2 file refactor or seed an explicit allowlist with rationale.' This locks the inventory granularity before remediation begins.
  - Question: Should Phase 1 explicitly require per-symbol classification for mixed-class files, not just per-file labels?
  - Recommended answer: Yes. The inventory should record per-symbol decisions for any file containing both classes; the manifest's allowlist can then carry the MIT-keep entries with rationale, and Phase 2 can plan the file split if needed.
  - If unanswered: Default to extending Phase 1 Changes with the per-symbol-split requirement and seeding allowlist entries for any MIT-keep identifier that co-resides with a private one (e.g., connect_grant_to_local_admission alongside NangoConnection).
