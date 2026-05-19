---
spec_version: '2.0'
task_id: rust-local-config
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T00:00:00Z'
status: draft
harden_status: blocked
size: small
risk_level: medium
---

# Rust local config

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Hardened for
implementation readiness after reading the current TS config sources. Covers
`runx config` plus the Rust local config API needed by managed-agent loading and
skill profile resolution.
Blockers: `rust-runtime-skeleton`; choose/land the Rust module boundary because
`crates/runx-runtime/src/config/*` does not exist today.
Allowed follow-up command: `scafld harden rust-local-config`
Latest runner update: none
Review gate: not_started

## Summary

Port local config read/write (config-store, env-var overlay, profile selection)
to Rust. The CLI command entrypoint is
`packages/cli/src/commands/config.ts`, but the authoritative behavior is in
`packages/core/src/config/index.ts`; managed-agent env overlay is in
`packages/adapters/src/agent/index.ts`. `packages/cli/src/runx-state.ts` is not
the config authority; it only owns project/install state helpers.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (config command, runx-state)
- `@runxhq/core` (authoritative config behavior)
- `@runxhq/adapters` (managed-agent env overlay)
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/cli/src/commands/config.ts`
- `packages/core/src/config/index.ts`
- `packages/adapters/src/agent/index.ts`
- `packages/cli/src/runx-state.ts` (advisory only: project state, not local
  config)

Files impacted:
- `crates/runx-runtime/src/config.rs` or
  `crates/runx-runtime/src/config/mod.rs`
- `crates/runx-runtime/src/config/local.rs` if a nested module is chosen
- `crates/runx-runtime/src/config/profile.rs` if a nested module is chosen
- `crates/runx-runtime/src/lib.rs`
- `crates/runx-runtime/Cargo.toml`
- `fixtures/config/**`

Invariants:
- Config file paths match current TS exactly: `RUNX_HOME`, when set, resolves
  through `resolvePathFromUserInput(..., preferExisting: false)`; otherwise the
  global home is `os.homedir()/.runx`. Do not introduce XDG or AppData behavior
  in this spec unless TS changes first.
- `runx config` reads and writes `<runx-home>/config.json`.
- `config.json` is a JSON object and missing files load as `{}`.
- `writeRunxConfigFile` parity: parent directory is created recursively and the
  file is written with mode `0o600` on Unix.
- Supported config keys are exactly `agent.provider`, `agent.model`, and
  `agent.api_key`; unknown keys must not be silently accepted.
- `agent.api_key` is never written directly to `config.json`. The config stores
  `agent.api_key_ref`, and user-facing get/list output reports `[encrypted]`.
- Local agent key storage matches TS: keys live under `<runx-home>/keys/`;
  `local-config-secret` is created once with mode `0o600`; key payloads are
  AES-256-GCM JSON files containing `ref`, `alg`, `iv`, `ciphertext`, and
  `auth_tag`; corrupt or unreadable key payloads produce a specific error that
  includes `runx local agent key corrupted or unreadable at`.
- Managed-agent config resolution preserves TS precedence:
  `RUNX_AGENT_PROVIDER ?? config.agent.provider`; `RUNX_AGENT_MODEL ??
  config.agent.model`; API key from `RUNX_AGENT_API_KEY`, then provider-specific
  `OPENAI_API_KEY`/`ANTHROPIC_API_KEY`, then decrypted local
  `config.agent.api_key_ref`.
- Managed-agent config returns none/undefined-equivalent unless provider, model,
  and API key are all present after trimming. Providers are exactly `openai` and
  `anthropic`.
- Profile selection preserves TS order: skill-local `X.yaml`, then skill-local
  `.runx/profile.json`, then ancestor `bindings/<owner>/<skill>/X.yaml`;
  mismatched manifest `skill` values are errors.

## Objectives

- Port config get/set/list.
- Port profile selection.
- Port managed-agent env overlay and local-key loading parity.
- Add a fixture suite covering path resolution, env overlay precedence, local
  key encryption/decryption, display masking, and profile source order.

## Scope

In scope:
- Local config surface.
- Managed-agent config resolution from local config plus env overlay.
- Local skill profile selection.

Out of scope:
- Cloud-stored config (none today).
- Migration of legacy config locations beyond what TS does.
- Changing TypeScript path behavior to XDG/AppData.
- Switching the production CLI to Rust; that belongs in the cutover spec after
  parity is proven.

## Dependencies

- `rust-runtime-skeleton`.
- Rust crypto dependency selection for AES-256-GCM/base64url must be explicit in
  `crates/runx-runtime/Cargo.toml`; do not fake encryption with hashing or
  plaintext.

## Blockers

- The draft previously specified XDG/AppData path parity, but TS does not do
  that. Build must target current TS behavior (`~/.runx` default) or first land
  a separate TS behavior change and parity fixture update.
- The named Rust files do not exist and `crates/runx-runtime/src/lib.rs` does
  not export a config module. Implementation must first create the module and
  public API before wiring CLI parity.
- Env overlay acceptance cannot be limited to `runx config get/set/list`; it
  must cover `loadManagedAgentConfig` parity because that is where
  `RUNX_AGENT_*`, `OPENAI_API_KEY`, and `ANTHROPIC_API_KEY` override local
  config.
- Secret handling is a build blocker. A Rust implementation that stores
  `agent.api_key` in `config.json`, logs plaintext keys, or only masks display
  while persisting plaintext is not acceptable.

## Sequencing

1. Add the Rust config module and export a narrow API from `runx-runtime`: home
   path resolution, config load/write/update/lookup/mask, local key
   encrypt/decrypt, managed-agent config resolution, and profile resolution.
2. Add parity fixtures/tests for existing TS behavior before switching any CLI
   caller to Rust. Include missing config, malformed JSON, non-object JSON,
   relative `RUNX_HOME` anchored to selected workspace base, and encrypted key
   round-trip.
3. Add managed-agent overlay tests covering env-over-file precedence for
   provider/model/API key and provider-specific API key fallback.
4. Add profile selection tests covering `X.yaml` priority, `.runx/profile.json`
   fallback, workspace binding fallback, and manifest skill mismatch failure.
5. Only after the Rust tests pass, wire the Rust CLI/runtime caller in the
   separate cutover spec. This spec should not remove the TS implementation.

## Acceptance Criteria

- `runx config set agent.provider openai` produces
  `{ "agent": { "provider": "openai" } }` shape parity under
  `<RUNX_HOME>/config.json`.
- `runx config set agent.api_key sk-test-secret` writes no plaintext secret to
  `config.json`, creates `<RUNX_HOME>/keys/local-config-secret`, creates one
  `<RUNX_HOME>/keys/local_agent_key_*.json`, and can decrypt the secret through
  the Rust API.
- `runx config get agent.api_key` and `runx config list` return `[encrypted]`
  for stored local key refs and never include the original key value in JSON or
  human-readable output.
- Missing config files load as empty config; malformed JSON and non-object JSON
  are rejected with path-bearing errors.
- Relative `RUNX_HOME=home` resolves under the selected workspace base, matching
  `packages/core/src/config/index.test.ts`.
- Managed-agent config uses env precedence exactly as listed in Invariants and
  returns none when provider/model/key is incomplete.
- Profile resolution returns source labels equivalent to TS: `skill-profile`,
  `profile-state`, `workspace-bindings`, or `none`.
- The implementation does not change `packages/cli/src/runx-state.ts` behavior.

## Validation Commands

- `pnpm test -- --run packages/core/src/config/index.test.ts packages/adapters/src/agent/index.test.ts packages/cli/src/index.test.ts`
- `cargo test -p runx-runtime config`
- `cargo test -p runx-runtime`

## Open Questions

- Whether the local config module belongs in `runx-runtime` long-term or should
  move to `runx-core`. For this draft, keep implementation in `runx-runtime`
  because that is the existing impacted package, but export a narrow API so it
  can move later without changing callers.
