---
spec_version: '2.0'
task_id: process-credential-delivery-hardening-v1
created: '2026-05-25T17:51:35+10:00'
updated: '2026-05-25T17:51:35+10:00'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# process-credential-delivery-hardening-v1

## Current State

Status: draft
Current phase: ready for execution
Next: replace or strictly constrain env-based credential delivery for
supervised process adapters.
Reason: `cli-tool` now rejects process-env credential delivery before spawn, but
MCP, external adapters, and outbox providers still have process boundaries where
credentials may be delivered via environment variables.
Blockers: none.
Allowed follow-up command: `scafld exec process-credential-delivery-hardening-v1`
Latest runner update: 2026-05-25T17:51:35+10:00
Review gate: not_started

## Summary

Credentials crossing supervised process boundaries must be brokered by opaque
references, scoped files, or a runtime-owned descriptor channel. Raw secrets
must not be ambient child process environment. Redaction remains defense in
depth, not containment.

## Scope

In scope:
- MCP adapter credential delivery.
- External adapter credential delivery.
- Outbox provider credential delivery.
- Receipt-safe observation and redaction metadata for the selected channel.

Out of scope:
- Provider-specific OAuth flows.
- `cli-tool` env secret rejection, already implemented.

## Acceptance

Profile: strict

Definition of done:
- [ ] `dod1` MCP process delivery does not expose raw secrets through ambient
  child environment.
- [ ] `dod2` External adapter process delivery does not expose raw secrets
  through ambient child environment.
- [ ] `dod3` Outbox provider process delivery does not expose raw secrets
  through ambient child environment.
- [ ] `dod4` Receipts record credential handle/observation metadata without
  leaking secret material.

Validation:
- [ ] `v1` `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_server`
- [ ] `v2` `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external_adapter`
- [ ] `v3` `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test credential_delivery`
- [ ] `v4` focused grep review for `CredentialDelivery::ProcessEnv` in runtime
  adapter spawn paths.

## Review

Reject any patch that treats substring redaction as credential containment or
allows a raw secret to remain in a long-lived child environment.
