---
spec_version: '2.0'
task_id: runx-receipt-content-addressing-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T21:42:56Z'
status: completed
harden_status: not_run
size: medium
risk_level: medium
---

# runx-receipt-content-addressing-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-04T21:42:56Z
Review gate: pass

## Summary

Take the two cheap content-addressing moves on receipts now, while they are
near-free, to keep the claim-graph door open without retrofitting later: (1)
content-addressed receipt ids — two runs that did the identical governed thing
produce the same receipt id (free dedup); (2) a shared receipt/resolution envelope
so a verifier can walk a receipt's ancestry offline. Receipts already seal
(Ed25519, verifiable via `governed-spend/verify.mjs`). This spec is deliberately
bounded: it does NOT build the full receipt claim-graph S-tier or the enlightened
tier (both are reserved-draft DO-NOT-BUILD).

## Objectives

- A content-addressed receipt id (deterministic function of the canonical body),
  enabling free dedup of identical governed actions.
- A shared receipt/resolution envelope that lets a verifier walk ancestry offline.
- Both ADDITIVE: no canonical-JSON churn or digest change for existing receipts.

## Scope

In scope:
- The two moves, additive, with fixtures proving existing receipt digests are
  unchanged.

Out of scope:
- The full receipt claim-graph S-tier (reserved draft, DO-NOT-BUILD here).
- The "enlightened" receipt tier (reserved draft, DO-NOT-BUILD).
- Any non-additive change to `runx.receipt.v1` / canonical JSON.

## Dependencies

- The receipt contract (`runx.receipt.v1`) + canonical-JSON oracle; `verify.mjs`.

## Assumptions

- The two moves are additive over the sealed receipt and compounding (cheap now,
  expensive to retrofit), per the reserved-draft analysis.

## Touchpoints

- `crates/runx-contracts/src/receipt.rs` + the canonical-JSON oracle/fixtures;
  `examples/governed-spend/verify.mjs` (ancestry walk).

## Risks

- **Canonical-JSON churn.** A non-additive id/envelope change re-hashes every
  existing receipt. Mitigation: additive-only; gate on unchanged existing digests.
- **Scope creep into the forbidden full claim-graph.** Mitigation: hard-stop at the
  two moves; the S-tier/enlightened tiers stay DO-NOT-BUILD.

## Acceptance

Profile: strict

Validation:
- Two identical governed runs produce the same receipt id (dedup); a verifier walks
  a receipt's ancestry offline.
- Existing receipt digests/canonical oracle are UNCHANGED (`pnpm fixtures:harness:check`,
  the c14n oracle, `cargo nextest run --workspace --all-features` all green).

## Phase 1: Content-addressed receipt id (additive)

Status: completed
Dependencies: receipt contract + c14n oracle

Objective: identical governed actions yield the same receipt id; existing digests

Changes:
- Add the content-addressed id (additive field/derivation); fixtures proving dedup + unchanged existing digests.

Acceptance:
- [x] `ac1` command - dedup holds, existing oracle unchanged
  - Command: `cargo nextest run --manifest-path crates/Cargo.toml -p runx-receipts && pnpm fixtures:harness:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6

## Phase 2: Shared receipt/resolution envelope (offline ancestry)

Status: completed
Dependencies: Phase 1

Objective: a verifier walks a receipt's ancestry offline.

Changes:
- Add the shared resolution envelope (additive); extend `verify.mjs` to walk ancestry.

Acceptance:
- [x] `ac2` command - offline ancestry walk verifies
  - Command: `bash -lc 'set -e; cargo build --manifest-path crates/Cargo.toml -p runx-cli --bin runx --all-features >/dev/null; OUT="$(mktemp -d)"; RDIR="$OUT/receipts"; mkdir -p "$RDIR"; node examples/http-graph/server.mjs >"$OUT/server.log" 2>&1 & SERVER=$!; trap "kill $SERVER 2>/dev/null || true" EXIT; sleep 1; RUNX_RECEIPT_SIGN_KID=runx-demo-key RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64=QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI= RUNX_RECEIPT_SIGN_ISSUER_TYPE=hosted crates/target/debug/runx harness examples/http-graph --receipt-dir "$RDIR" --json >"$OUT/harness.json"; ROOT_ID="$(node -e "const fs=require(\"fs\");const j=JSON.parse(fs.readFileSync(process.argv[1],\"utf8\"));console.log(j.receipt_ids[0] ?? \"\")" "$OUT/harness.json")"; test -n "$ROOT_ID"; node examples/governed-spend/verify.mjs "$RDIR/$ROOT_ID.json" --walk-ancestry --receipt-dir "$RDIR"'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19

## Rollback

- Additive fields; remove them. If any existing digest churns, the change was not
  additive — revert.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: command
Output: command.stdout
Summary: Reviewed receipt ancestry persistence and verifier changes; runtime persists finalized graph step receipts as top-level receipt-store artifacts, verifier walks top-level receipt ancestry and ignores graph-state snapshots, and gates passed.

Attack log:
- `graph receipt persistence`: Child locator must use finalized child digest, not graph-state snapshot; covered by runtime regression and live http-graph ancestry verification. -> clean
- `offline verifier`: Stale runs/*.graph-state.json with same content-address id must not shadow top-level receipt; covered by vitest stale-sidecar regression. -> clean
- `canonical fixtures`: Additive persistence must not churn existing digest/oracle fixtures; runx-receipts nextest and fixtures:harness:check passed. -> clean

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- none
