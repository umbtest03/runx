---
spec_version: '2.0'
task_id: registry-signed-manifest-trust-anchor-v1
created: '2026-05-25T17:51:35+10:00'
updated: '2026-05-25T17:51:35+10:00'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# registry-signed-manifest-trust-anchor-v1

## Current State

Status: draft
Current phase: ready for execution
Next: add a publisher-signed registry trust anchor for installed skills.
Reason: install digest validation currently compares downloaded candidate
content against a digest supplied by the same candidate or caller. That detects
transport corruption only when the caller has an out-of-band digest; it does not
prove publisher intent.
Blockers: none.
Allowed follow-up command: `scafld exec registry-signed-manifest-trust-anchor-v1`
Latest runner update: 2026-05-25T17:51:35+10:00
Review gate: not_started

## Summary

Skill install must verify a digest from a trusted registry or publisher-signed
manifest, not a digest asserted by the downloaded candidate itself. This is a
clean cutover: no legacy self-asserted digest path remains for trusted installs.

## Scope

In scope:
- Registry manifest shape for skill digest, signer identity, key id, and
  signature.
- `runx-runtime` registry install verification.
- CLI install behavior and error messages.
- Fixtures for trusted, tampered, unsigned, and mismatched-manifest installs.

Out of scope:
- Marketplace curation policy and trust-tier assignment.
- Remote key transparency infrastructure beyond the minimal trusted key set
  needed for this cutover.

## Acceptance

Profile: strict

Definition of done:
- [ ] `dod1` Trusted install requires a registry/publisher-signed manifest.
- [ ] `dod2` Candidate-supplied digests are never accepted as the trust anchor.
- [ ] `dod3` Tampered content and mismatched manifest digests fail closed.
- [ ] `dod4` CLI output clearly distinguishes unsigned, unknown-key, invalid
  signature, and digest mismatch failures.

Validation:
- [ ] `v1` `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test registry_install`
- [ ] `v2` `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test registry`
- [ ] `v3` `rg -n "candidate\\.digest|validate_candidate_digest" crates/runx-runtime/src crates/runx-cli/src`

## Review

Reject any solution that keeps a trusted path where downloaded skill content can
authenticate itself.
