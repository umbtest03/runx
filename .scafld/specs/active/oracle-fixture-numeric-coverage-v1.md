---
spec_version: '2.0'
task_id: oracle-fixture-numeric-coverage-v1
created: '2026-05-29T00:00:00Z'
updated: '2026-05-29T00:00:00Z'
status: active
harden_status: not_run
size: small
risk_level: low
---

# Cross-language canonical JSON: oracle coverage expansion

## Current State

Status: active
Current phase: phase1
Next: implement
Reason: The shared oracle file
`fixtures/contracts/canonical-json/runx-stable-json-v1.cases.json` is the only
standing artifact that future contract changes get compared against for
cross-language byte parity. It has five cases. The number domain in the
existing `covered-number-domain` case is `{0, ±integer, ±0.x}`, which covers
none of the float range where Rust and JS canonical writers disagree. String
and container coverage is similarly thin.
Blockers: depends on `canonical-json-float-parity-v1` Phase 1 landing first
so the Rust writer is JS-faithful for floats.

## Summary

Expand the shared oracle from 5 cases to roughly 35 by adding categories
covering the cross-language failure surface: float edge band, string escape
boundaries, container edge shapes. Both `canonical-json.test.ts` and the
Rust `canonical.rs` tests enumerate every case in the oracle and assert byte
equality of `canonical_json_stringify(value) == expected_canonical_json`.

This spec ships no code changes beyond the fixture file and any test
plumbing required to consume the new cases.

## Scope

In:

- Append cases to `fixtures/contracts/canonical-json/runx-stable-json-v1.cases.json`
  in the following categories. Each case carries the existing fields
  (`name`, `value`, `expected_canonical_json`, `expected_utf8_hex`,
  `expected_sha256_hex`, `expected_sha256`).

  Float edge band:
  - `float-tenth` (0.1), `float-third` (0.3), `float-sum-tenths` (0.1+0.2),
    `float-near-msi` (2**53), `float-msi` (2**53 - 1), `float-msi-plus`
    (2**53 + 1, which becomes 2**53 in f64), `float-subnormal`
    (5e-324), `float-large` (1.5e308), `float-tiny-sci` (1e-7),
    `float-large-sci` (1e21), `float-neg-zero` (-0).

  String escape boundaries:
  - `str-empty`, `str-ascii-control-band` (every char U+0000..U+001F as a
    single string), `str-quote`, `str-backslash`, `str-solidus`,
    `str-line-separator` (U+2028), `str-paragraph-separator` (U+2029),
    `str-bmp-surrogate-pair` (U+1F600 = "😀", validates surrogate-pair
    encoding), `str-max-utf8-3-byte`, `str-non-ascii-no-escape` (combining
    chars stay literal).

  Container edge shapes:
  - `obj-empty`, `arr-empty`, `obj-single-empty-string-key`,
    `obj-key-with-special-chars`, `arr-of-empty-arrays`, `obj-deep-3`,
    `arr-mixed-types`, `obj-key-sort-collision` (keys that differ only by
    case + by length).

- Update the `canonical-json.test.ts` `expect(fixture.cases.length)` if it
  was pinning a count (currently uses `it.each`, no count assertion; no
  change needed). Confirmed via existing test code.

Out:

- The `runx-receipt-c14n-v1.oracles.json` file. That oracle covers receipt
  bodies and is regenerated from harness-spine fixtures; expanding it is a
  separate exercise covered by harness fixture growth.
- Generating cases programmatically from Rust. Cases are committed JSON to
  keep the oracle independent of either implementation.

## Acceptance Criteria

1. `pnpm --filter @runxhq/contracts test` passes all cases in the expanded
   oracle, byte-equal.
2. `cargo test -p runx-receipts` passes, including any test that loads the
   shared oracle for parity.
3. The oracle file is valid JSON and every case has all six fields populated.
4. The `expected_utf8_hex` field matches `Buffer.from(expected_canonical_json, "utf8").toString("hex")`
   for every case. The `expected_sha256_hex` matches sha256(expected_canonical_json).
   These invariants are verified by the existing test loop in
   `canonical-json.test.ts`.

## Generating the case values

Cases are produced offline via a one-shot Node script (run by hand, not
committed as automation) that calls `canonicalJsonStringify` on each value,
then derives the `expected_utf8_hex` and `expected_sha256_hex` fields. The
output is hand-reviewed and committed. The script is reproducible from this
spec; it is not added to the repo because the oracle is the source of truth
and the script is a one-off.

## Risk

Low. Additive fixtures; no code path changes. The worst case is a case whose
expected output disagrees with one of the implementations, which surfaces as
a test failure on first run and gets fixed by either correcting the
implementation (if the oracle is right) or correcting the oracle (if the
implementation is right and the oracle was misgenerated).

## Sequencing

Lands after `canonical-json-float-parity-v1` Phase 1. Otherwise the
`float-tiny-sci`, `float-large-sci`, `float-large`, `float-subnormal`,
and `float-near-msi` cases would fail on Rust.
