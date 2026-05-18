---
spec_version: '2.0'
task_id: rust-connect-client
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# Rust connect client

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Covers `runx connect`
(grants, OAuth, device flow).
Blockers: `rust-runtime-skeleton`.
Allowed follow-up command: `scafld harden rust-connect-client`
Latest runner update: none
Review gate: not_started

## Summary

Port the connect surface (grants, OAuth code/device flow, Nango-hosted
intake, BYO credential intake) to Rust. Today this lives in
`packages/cli/src/commands/connect.ts` and `packages/cli/src/connect-http.ts`,
with cloud-side at `cloud/packages/auth/`. The Rust port consumes the same
cloud HTTP contract.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (connect command)
- `cloud/packages/auth`
- `crates/runx-runtime` (or new `crates/runx-connect-client`)
- `crates/runx-contracts`

Current TypeScript sources:
- `packages/cli/src/commands/connect.ts`
- `packages/cli/src/connect-http.ts`
- `cloud/packages/auth/src/connect-html.ts`

Files impacted:
- `crates/runx-runtime/src/connect/client.rs`
- `crates/runx-runtime/src/connect/flows.rs`
- `fixtures/connect/**`

Invariants:
- Connect tokens never land in receipts or logs as plaintext.
- The `connect` verb is user-facing; `grant` is the internal object.
- Auto-connect on first skill use behavior matches TS.
- Device flow and browser flow behave the same way across languages.

## Objectives

- Port OAuth code, device, and Nango-hosted flows.
- Port BYO credential intake (API_KEY, BASIC, TWO_STEP, JWT modes if/when
  implemented cloud-side).
- Add fixture suite for each flow against a deterministic auth-server mock.

## Scope

In scope:
- Connect client + flow handlers.

Out of scope:
- Cloud-side auth/grant logic (stays TS until separate cutover).
- Credential storage backends beyond what TS supports.

## Dependencies

- `rust-runtime-skeleton`.
- `cloud-http-contract-stabilization` (`.ai/specs/drafts/`) for the
  connect / auth / grant HTTP contract surface.
- `rust-ts-interop-boundary` for the cross-language crossing reference.

## Open Questions

- Browser-flow callback handling on Rust (local HTTP listener vs callback
  URL). Defer to Phase 1.
