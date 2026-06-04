---
spec_version: '2.0'
task_id: canonical-json-float-parity-v1
created: '2026-05-29T00:00:00Z'
updated: '2026-06-04T20:48:45Z'
status: completed
harden_status: not_run
size: small
risk_level: medium
---

# Cross-language canonical JSON: float parity

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-04T20:48:45Z
Review gate: pass

## Summary

Receipt IDs are content-addressed: `id = sha256(canonical_body)` under the
canonicalization tag `runx.receipt.c14n.v1`. The address depends on byte
equality of the canonical encoding produced independently by Rust
(`crates/runx-receipts/src/canonical.rs`) and TS
(`packages/contracts/src/canonical-json.ts`). The shared oracle file at
`fixtures/contracts/canonical-json/runx-stable-json-v1.cases.json` pins
five known cases; none cover the float range where the two writers disagree.

This spec routes the Rust number leaf through `serde_json::to_string`, which
dispatches through `JsonNumber`'s existing `Serialize` impl (whole-float
short-circuit to integer encoding, plus ryu for non-integer f64). ryu produces
the same shortest-round-trip output as JS `Number.prototype.toString` for the
cases verified below, including the `+` sign on positive exponents.

## Confirmed Divergences

Empirically verified by running the same values through `serde_json` (with
ryu) and Node `JSON.stringify`:

| Value           | Rust `JsonNumber::Display` (current) | JS `JSON.stringify`     | `serde_json::to_string(&JsonNumber)` (proposed) |
| --------------- | ------------------------------------ | ----------------------- | ----------------------------------------------- |
| `1e-7`          | `0.0000001`                          | `1e-7`                  | `1e-7`                                          |
| `1e21`          | `1000…(22 digits)`                   | `1e+21`                 | `1e+21`                                         |
| `1.5e308`       | 309-digit decimal                    | `1.5e+308`              | `1.5e+308`                                      |
| `5e-324`        | 324-digit decimal                    | `5e-324`                | `5e-324`                                        |
| `2**53` (f64)   | `9007199254740992`                   | `9007199254740992`      | `9007199254740992` (via whole-f64 short-circuit) |
| `-0.0`          | `0` (Display short-circuit)          | `0`                     | `0` (whole-f64 short-circuit routes to `i64(0)`) |
| `0.1`           | `0.1`                                | `0.1`                   | `0.1`                                            |
| `1.0000000000000002` | `1.0000000000000002`            | `1.0000000000000002`    | `1.0000000000000002`                            |

The `JsonNumber::Serialize` whole-float routing already aligns Rust and JS for
integer-typed and `-0.0` cases. The defect is isolated to the canonical
writer's bypass of `Serialize`.

## Scope

In:

- `crates/runx-receipts/src/canonical.rs`: replace the Number leaf in
  `write_canonical_json_value` to use `serde_json::to_string(value)`.
- New oracle file `fixtures/contracts/canonical-json/runx-stable-json-v1.numbers.cases.json`
  with the divergent-range cases listed above; loaded by both
  `canonical-json.test.ts` and the Rust canonical test module.
- New differential test `crates/runx-receipts/tests/integration.rs` module
  `canonical_parity` using `proptest` to generate `JsonValue` trees up to
  depth 4 and assert that the Rust canonical output round-trips through
  `serde_json::from_str` → `serde_json::to_string` with byte equality for the
  number leaves. (A full Rust↔Node differential test runs in a separate CI
  job and is out of scope for this spec; the proptest target proves the Rust
  writer is internally consistent with `serde_json`'s leaf encoders, which
  the new oracle then pins to JS-equivalent strings.)

Out:

- Changing the canonicalization tag (`runx.receipt.c14n.v1` stays valid; the
  Rust writer is becoming spec-compliant, not producing a new format). No
  legacy reader work.
- Editing the TS canonical writer. TS is already JS-faithful; Rust is the
  side that needs to align.

## Acceptance Criteria

1. `cargo test -p runx-receipts` passes, including the new `canonical_parity`
   module.
2. `pnpm --filter @runxhq/contracts test` passes, including the new numbers
   oracle.
3. The Rust canonical output for every case in the new oracle file is
   byte-equal to the `expected_canonical_json` field.
4. No existing receipt fixtures in `fixtures/contracts/harness-spine/*.json`
   produce a different digest (verified by re-running
   `receipt_oracle_matches_rust_canonical_json`). If any fixture digest
   changes, that fixture's `expected_canonical_json` and `expected_sha256` are
   updated and the change is called out in the commit.

## Risk

Touches receipt content-addressing. If a current receipt fixture (or live
issued receipt) happens to contain an f64 in the divergent range, its
canonical bytes and digest change. The existing fixtures in
`fixtures/contracts/harness-spine/` are inspected as part of acceptance step 4.

Mitigation: the existing test
`crates/runx-receipts/src/canonical.rs::tests::receipt_oracle_matches_rust_canonical_json`
already enforces byte equality against `runx-receipt-c14n-v1.oracles.json`,
so any fixture digest shift surfaces immediately, not silently.

## Sequencing

Phase 1 (this spec): land the writer fix, the numbers oracle, and the
proptest.

Phase 2 (`oracle-fixture-numeric-coverage-v1`): expand the
`runx-stable-json-v1.cases.json` oracle with additional non-number cases
(string escapes for control chars, U+2028/U+2029, nested containers).
Independent of Phase 1; sequenced after to keep diffs small.

Phase 3 (`yaml-parity-subset-hardening-v1`): unrelated boundary; tracks
separately.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: command
Output: command.stdout
Summary: Canonical JSON float parity is implemented and verified. Rust canonical number emission routes JsonNumber through serde_json::to_string, the numbers oracle is loaded by Rust and TS tests, cargo test -p runx-receipts passed, pnpm --filter @runxhq/contracts test passed, and the oracle digest/UTF-8 invariant check passed for the committed stable JSON cases.

Attack log:
- `crates/runx-receipts/src/canonical.rs`: verify number leaf routes through serde_json::to_string rather than Display -> clean
- `fixtures/contracts/canonical-json/runx-stable-json-v1.numbers.cases.json`: verify number oracle exists and is loaded by Rust and TS tests -> clean
- `cargo test -p runx-receipts`: run Rust receipt/canonical tests -> clean
- `pnpm --filter @runxhq/contracts test`: run TS canonical contract tests -> clean
- `oracle digest fields`: verify expected_utf8_hex and sha256 fields match expected_canonical_json -> clean

Findings:
- none

