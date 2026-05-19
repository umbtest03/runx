---
spec_version: '2.0'
task_id: rust-registry-client
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T06:54:16Z'
status: completed
harden_status: hardened
size: medium
risk_level: medium
---

# Rust registry client

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T06:54:16Z
Review gate: pass

## Summary

Port the registry client (skill search, add, inspect, publish, and list)
to a Rust crate. Today this lives in TS across the CLI dispatch, core registry
modules, and the runner's `registry-resolver.ts` / `skill-install.ts`. The Rust
client speaks the stabilized HTTP contract published by the hosted registry
surface and consumes `runx-contracts::registry` types.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (skill / search / add / publish / list commands)
- `@runxhq/runtime-local` (registry-resolver)
- `crates/runx-registry-client`
- `crates/runx-contracts` (registry shapes)
- `cloud/packages/api` (registry routes)

Current TypeScript sources:
- `packages/cli/src/dispatch.ts` (skill / search / add / publish / list)
- `packages/core/src/registry/http-client.ts`
- `packages/core/src/registry/http-cached-store.ts`
- `packages/core/src/registry/store.ts`
- `packages/core/src/registry/resolve.ts`
- `packages/core/src/registry/trust.ts`
- `packages/runtime-local/src/runner-local/registry-resolver.ts`
- `packages/runtime-local/src/runner-local/skill-install.ts`

Files impacted:
- `crates/runx-registry-client/**`
- `crates/runx-runtime/src/registry.rs` (thin runtime integration only)
- `fixtures/registry/**`

Invariants:
- HTTP contract version is owned by `cloud-http-contract-stabilization`;
  this spec consumes a specific version, it does not negotiate ad-hoc.
- Trust tiers (`first_party`, `verified`, `community`) round-trip identically.
- Registry namespace ownership rules are not duplicated; the client
  consumes server decisions.
- Skill install is idempotent; receipts capture the install action when
  invoked from a chain.
- Remote acquire is the hosted install boundary; direct `GET` reads do not
  increment install counters or materialize local installs.
- Local file writes are atomic and never leave partial `SKILL.md` or
  `.runx/profile.json` files after validation failure.

## Objectives

- Port registry client (search, get, list, publish).
- Port registry resolver used by the runner.
- Port skill install flow.
- Add fixture suite for each surface.
- Provide a single Rust-owned API that TS can call during cutover without
  duplicating registry semantics in two implementations.

## Scope

In scope:
- Client, resolver, install.

Out of scope:
- Cloud-side registry logic.
- Hosted registry namespace / publisher ownership policy.
- Registry signing / attestation hierarchy (already covered by
  `registry-release-distribution-hardening` draft).

## Dependencies

- `rust-runtime-skeleton`.
- `rust-contracts-parity`.
- `cloud-http-contract-stabilization` for the registry HTTP surface.
- `rust-ts-interop-boundary` for the cross-language crossing reference.

## Build Decisions

- Create `crates/runx-registry-client` instead of hiding the work behind a
  `runx-runtime` feature. The registry client has a distinct HTTP, cache, and
  install contract and needs independent fixtures.
- Runtime integration stays thin: runner code calls the registry crate for
  resolution/materialization and continues to own execution receipts.
- The crate must be usable without network features for local file-registry
  resolution and install tests.

## HTTP Contract Boundary

The Rust client must implement only the stabilized registry HTTP surface. Do
not infer extra routes or server behavior from local store internals.

Remote endpoints consumed by this spec:
- `GET /v1/skills?q=<query>&limit=<n>` for search. A blank query omits `q`;
  default limit is `20`; bare-name resolution uses limit `100`.
- `GET /v1/skills/{owner}/{name}` and
  `GET /v1/skills/{owner}/{name}@{version}` for inspect/read. `404` maps to
  `None`; other non-2xx statuses are errors.
- `POST /v1/skills/{owner}/{name}/acquire` for hosted install acquisition.
  Body is JSON with `installation_id`, optional `version`, and `channel`
  defaulting to `cli`. Non-2xx statuses are errors.

Payload validation is strict at the boundary:
- Search requires `status: "success"` and `skills[]` entries with
  `skill_id`, `name`, `owner`, `source_type`, `profile_mode`, `runner_names`,
  `required_scopes`, `tags`, `trust_tier`, `install_command`, and
  `run_command`.
- Read requires `status: "success"` and a `skill` containing `skill_id`,
  `owner`, `name`, `version`, `digest`, `markdown`, `runner_names`,
  `source_type`, `trust_tier`, `required_scopes`, `tags`, `publisher`,
  `attestations`, `install_command`, and `run_command`.
- Acquire requires `status: "success"`, `install_count`, and `acquisition`
  containing `skill_id`, `owner`, `name`, `version`, `digest`, `markdown`,
  `runner_names`, `trust_tier`, `publisher`, and `attestations`; optional
  `profile_document`, `profile_digest`, and `source_metadata` round-trip.
- Unknown JSON fields are tolerated, but missing or mistyped required fields
  produce typed contract errors that include the route and field path.
- URL construction percent-encodes owner/name/version segments and strips one
  trailing slash from the base URL.

## Trust And Ownership

- `trust_tier` is an exact enum: `first_party`, `verified`, `community`.
  Unknown tiers fail validation; tiers are never re-derived from owner on the
  remote path.
- Publisher, source metadata, and attestations are pass-through contract
  fields. The client validates shape but does not upgrade, downgrade, synthesize,
  or remove server-provided trust signals.
- Local registry ingestion may preserve existing local defaults, but remote
  results always consume the server's trust and publisher decisions.
- Namespace ownership, publisher authorization, and first-party reservation
  rules are server-owned. The Rust client must not duplicate allowlists,
  deny-lists, owner-to-tier mappings, or "runx means first_party" logic for
  remote responses.

## Resolution Semantics

- Accepted refs are `runx://skill/<encoded-ref>`, `registry:<ref>`,
  `runx-registry:<ref>`, `<owner>/<name>`, `<owner>/<name>@<version>`, and
  bare `<name>`.
- Explicit `<owner>/<name>` refs bypass search and preserve the requested
  version unless `--version` overrides it.
- Bare names resolve by remote search filtered to exact normalized name. Zero
  matches returns not found. More than one match is an ambiguity error that
  tells the user to use `<owner>/<name>`.
- Local materialization cache paths include owner, name, version, and the first
  16 digest characters, with a `.runx-registry-digest` marker containing both
  `digest` and `profile_digest`.

## Idempotent Install

- Install validates the downloaded markdown first, hashes `SKILL.md`, verifies
  any expected digest, validates `profile_digest`, and checks runner names from
  `X.yaml` before writing files.
- Destination package paths are derived from the install ref when namespaced
  and from the skill name otherwise; each component is lowercased and limited
  to safe path characters.
- Existing `SKILL.md` with the same digest returns `unchanged`; different
  content is an error.
- Existing `.runx/profile.json` with the same content is accepted; different
  profile state is an error.
- New writes use temp-file plus atomic rename. Validation failures and digest
  mismatches must not create destination directories containing partial skill
  files.
- Remote installs require an installation id. The CLI integration obtains it
  from the existing install-state flow before calling acquire.

## Receipt Integration

- A direct `runx skill add` / `runx skill add` command remains a local install action
  and does not invent a skill-execution receipt.
- When a graph or chain performs an install as part of execution, the Rust
  integration records the install as receipt metadata or a ledger event owned
  by the enclosing execution receipt. The recorded fields are `ref`,
  `skill_id`, `version`, `digest`, `profile_digest`, `trust_tier`,
  `publisher`, `source_label`, `destination`, `status`, and remote
  `install_count` when present.
- Receipt data must reference the immutable digest actually installed, not only
  the requested ref or version.
- Receipt integration reuses `runx-runtime` receipt path and signing
  facilities; this spec must not introduce a second receipt store.

## Acceptance Tests

- Remote search success, invalid payload, non-2xx error, trust-tier rejection,
  and add/run command round-trip.
- Remote read success, `404` not found, invalid payload, and versioned URL
  encoding.
- Remote acquire success with profile document, missing installation id,
  non-2xx error, invalid payload, and install-count capture.
- Bare remote name resolution: zero match, one exact match, and ambiguous
  matches.
- Local install idempotency: first install writes, repeat install is
  `unchanged`, conflicting `SKILL.md` errors, conflicting profile state errors.
- Digest/profile digest mismatch does not leave partial files.
- Remote trust tiers and attestations round-trip for `first_party`, `verified`,
  and `community` without client-side owner-derived changes.
- Hosted namespace ownership is not present in the client crate; tests assert
  no remote owner allowlist or tier derivation is used.
- Chain/graph install receipt metadata captures the installed digest and does
  not create a second receipt store.

## Build-Ready Checklist

- [x] Crate ownership selected.
- [x] HTTP route and payload contract listed.
- [x] Trust tiers and ownership boundary fixed.
- [x] Idempotent install behavior specified.
- [x] Receipt integration specified without duplicated receipt storage.
- [x] Fixture and test matrix defined.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Prior blockers F-REG-1 and F-REG-2 are now addressed. F-REG-1: `split_skill_id` (http.rs:240-253) rejects `.` and `..` segments via `is_dot_segment` before URL construction; `encode_segment` percent-encodes `.` to `%2E`; the new test `dot_path_segments_are_rejected_before_url_construction` (client.rs:113-126) asserts the call returns `InvalidSkillId` and that `transport.requests().is_empty()`, so no URL is constructed. The remaining suffix path segments (`<name>@<version>` and `/acquire`) cannot decode to a bare `.` or `..` because the literal `@` / `/acquire` separators always sit alongside non-dot characters, so rust-url's double-dot normalization (parser.rs:1319-1334) cannot apply. The exploit primitive identified by the prior review is closed at the validation boundary. F-REG-2: `include_str!` now loads all four declared fixtures — `search-success.json` (client.rs:330-333), `acquire-success.json` (client.rs:335-338), `echo-SKILL.md` (client.rs:313), and `echo-X.yaml` (client.rs:315). Modifying any of them deterministically affects test outcomes; they are no longer dead data. The prior F1 (InstallError export) and F3 (runtime registry receipt metadata captures installed digest, not advertised) remain clean. Workspace baseline matches the recorded task scope; ambient_drift is none; runx-runtime depends on runx-registry-client and exposes `RegistryInstallMetadataInput` / `registry_install_receipt_metadata` from lib.rs:49. No new blockers introduced.

Attack log:
- `crates/runx-registry-client/src/http.rs split_skill_id + encode_segment`: Verify F-REG-1: path traversal via percent-encoded dot segments under rust-url's `..` normalization -> clean (split_skill_id rejects '.' and '..' for owner/name before URL construction (http.rs:240-253). encode_segment encodes '.' to %2E (http.rs:259-270). The only path segments are encoded owner, name (or name@version), and the literal /v1/skills/ and /acquire — none can percent-decode to a bare '.' or '..' segment, so rust-url's double-dot shortening (parser.rs:1319-1334) cannot apply. Test dot_path_segments_are_rejected_before_url_construction (client.rs:113-126) asserts InvalidSkillId AND empty request log.)
- `fixtures/registry/{install,remote}/* + tests/client.rs`: Verify F-REG-2: fixtures are loaded by Rust tests so they cannot rot silently -> clean (tests/client.rs uses include_str! for echo-SKILL.md, echo-X.yaml, search-success.json, and acquire-success.json (client.rs:313, 315, 330-338). Modifying any fixture changes a test.)
- `crates/runx-runtime/src/registry.rs + tests/registry.rs`: Re-verify prior F3: receipt metadata records installed digest, omits acquisition fields when absent -> clean (Test asserts metadata.get('digest') == sha256:installed (the install digest), not sha256:remote-advertised. Absent acquisition omits publisher and install_count. runx-runtime exposes RegistryInstallMetadataInput/registry_install_receipt_metadata (lib.rs:49).)
- `crates/runx-registry-client/src/install.rs + refs.rs`: Filesystem traversal via crafted ref/skill_name into safe_skill_package_parts; partial writes on validation failure -> clean (safe_path_part maps '.'/'..' trimmed results to 'skill' (refs.rs:140); urlencoding_decode-then-split-by-'/' produces individually sanitized parts. Validation (digest, profile_digest, manifest binding) all runs before fs::create_dir_all (install.rs:188); profile_digest_mismatch_leaves_no_partial_install asserts destination_root.exists() == false on failure.)
- `crates/runx-registry-client/src/payload.rs`: Strict payload validation: unknown trust tier, missing fields, wrong types in publisher/attestations/source_metadata -> clean (trust_tier_field rejects unknown values with $.<path>.trust_tier field path (payload.rs:233-249). publisher_field, attestations_field, source_metadata_field require objects/arrays and surface route+JSON-pointer on mismatch. require_literal_status rejects non-success status.)
- `Cross-crate domain boundaries`: Registry client must not reach into runtime/cli/cloud; runx-core/receipts/parser must not depend on registry client -> clean (runx-registry-client deps: runx-parser, reqwest, serde, sha2, thiserror, url. Only runx-runtime depends on runx-registry-client (runtime/Cargo.toml:31). Workspace member registered (crates/Cargo.toml:8,19).)
- `crates/runx-registry-client lint cleanliness`: Workspace clippy lints (unwrap_used, expect_used, panic, dbg, print, todo, unimplemented) deny — confirm new crate is clean -> clean (Sources use .ok_or_else, ?, and pattern matches; no .unwrap()/.expect/panic!/dbg!/println!/eprintln!/todo!/unimplemented! observed in src/.)
- `Workspace task-scope vs ambient drift`: Detect ambient changes outside declared task scope masquerading as task changes; re-check prior workspace_mutation blocker -> clean (Workspace classifier reports ambient_drift: none. Harness/journal/CLI history additions are classified as task_changes by scafld (consistent with prior review). No code-level evidence of mid-review mutation; that determination is session-level and outside read-only review scope.)

Findings:
- none
