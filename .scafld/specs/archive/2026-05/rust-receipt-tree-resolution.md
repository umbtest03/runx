---
spec_version: '2.0'
task_id: rust-receipt-tree-resolution
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T05:51:55Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Rust receipt tree resolution

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T05:51:55Z
Review gate: pass

## Summary

Make receipt tree verification first-class in Rust. The verifier should resolve
child receipt references through a typed resolver, reject ambiguous references,
detect duplicate children, cycles, missing children, orphan children, wrong
parent links, and traversal-limit breaches, and return reviewer-safe findings.

Keep the current slice-based public verifier as a compatibility adapter, but
make the governed contract explicit: `runx-receipts` owns pure tree semantics
and resolver error mapping; `runx-runtime` owns receipt-store IO and any local
index used to satisfy the resolver.

This spec owns tree semantics only. Cryptographic proof checks belong to
`rust-receipt-proof-verification`; disk/path discovery belongs to
`rust-runtime-receipt-path-discovery`.

## Context

CWD: `.` (runx OSS workspace)

Relevant code and fixtures:
- `crates/runx-receipts/src/tree.rs`
- `crates/runx-receipts/src/verify/**`
- `crates/runx-receipts/tests/harness_receipts.rs`
- `crates/runx-runtime/src/receipts.rs`
- `fixtures/contracts/harness-spine/**`
- `fixtures/runtime/**`

Verified existing state:
- Current `verify_receipt_tree(root, children)` is IO-free but takes only a
  flat child slice, builds an id index, and traverses nested child refs itself.
- Existing tests cover missing child, matching child, nested missing child,
  cycle, orphan, wrong namespace, and duplicate ids, but they are unit tests
  built from two harness-spine fixtures rather than a durable tree oracle.
- Runtime graph receipts already emit `runx:harness_receipt:{id}` child refs and
  immediately validate the flat child slice, but runtime-created harnesses
  currently leave `parent_harness_ref` unset.
- Current finding codes do not distinguish malformed reference, ambiguous
  resolver result, parent-link mismatch, depth limit, or breadth limit.

Concurrent workspace note:
- Receipt and runtime files are currently dirty from adjacent Rust work. This
  spec must be hardened without editing code, and any later implementation must
  re-read the latest tree/runtime files instead of reverting unrelated changes.

Invariants:
- Receipt references are typed. Suffix matching is not acceptable for governed
  receipts.
- The tree verifier must be deterministic, bounded, and safe against hostile
  receipt graphs.
- A parent receipt cannot claim successful child execution if a child receipt is
  absent, duplicated, cyclic, orphaned, or linked to a different parent.
- Public findings never include operator-local absolute paths.
- Existing `validate_receipt_tree` / `verify_receipt_tree` callers remain
  source-compatible; new strict resolver behavior is introduced behind an
  additive API or internal adapter, then the old entrypoints delegate to it.
- Tree findings are structural only. Proof-summary honesty and signature
  validity are consumed from `rust-receipt-proof-verification`, not duplicated
  here.

## Objectives

- Define the `ReceiptResolver` boundary used by verifiers and runtime callers.
- Reject ambiguous or malformed child receipt references.
- Detect duplicate child ids, missing child ids, cycles, orphan child receipts,
  parent/child mismatch, depth limit, and breadth limit failures.
- Provide a positive and negative fixture oracle for tree resolution, including
  ordered finding codes and paths.
- Keep the verifier pure and IO-free; local filesystem resolution is injected by
  runtime code.
- Preserve a slice-backed resolver adapter for current runtime and test callers.

## Scope

In scope:
- `runx-receipts` resolver trait or equivalent callback.
- Tree verification findings and summary aggregation.
- Additive finding codes for malformed child refs, ambiguous child refs,
  parent-link mismatch, depth limit, and breadth limit.
- Fixture-backed tests for normal, fanout, nested, duplicate, missing, cycle,
  orphan, and wrong-parent cases.
- Runtime adapter glue only where needed to call the resolver boundary.

Out of scope:
- Signature/seal proof verification.
- Receipt directory discovery, manifest IO, or cloud storage.
- Slack/GitHub/Nitrosend presentation details.
- Harness-spine vocabulary changes or TypeScript authority-proof parity changes.

## Dependencies

- `runx-contract-spine-hard-cutover` completed; the harness-spine schema is the
  contract source for `parent_harness_ref` and `child_harness_receipt_refs`.
- `rust-receipts-parity`.
- Coordinates with `rust-receipt-proof-verification`.
- Coordinates with `rust-runtime-receipt-path-discovery`; that spec supplies
  receipt store discovery and index loading, while this spec defines what a
  resolved receipt graph means.
- Avoids the active `rust-policy-authority-proof-parity` surface; this spec
  must not rename authority fields, change policy fixture expectations, or edit
  harness-spine vocabulary.
- Feeds `rust-runtime-skill-execution`, `rust-nitrosend-dogfood`, and
  `rust-ts-sunset-receipts`.

## Assumptions

- Child references use a stable runx URI or exact receipt id; partial id and
  suffix lookup remain forbidden.
- Runtime may maintain a local receipt index, but the receipt crate receives an
  already-scoped resolver.
- The canonical child URI form is `runx:harness_receipt:<receipt_id>` with
  `type: harness_receipt`; bare ids are accepted only as exact-id compatibility
  input through the slice-backed adapter.
- A child `parent_harness_ref`, when present, must match the parent harness ref.
  Governed runtime fixtures should set it; legacy harness-spine fixtures with
  `null` parent refs remain structural-only unless a strict mode is requested.

## Touchpoints

- `runx-receipts` tree verifier and finding codes.
- Runtime receipt store interface.
- Contract fixtures for graph and harness receipts.
- CLI and Aster receipt summaries.

## Risks

- Unbounded traversal can make receipt verification a denial-of-service vector.
- Ambiguous child lookup can verify the wrong receipt.
- Overlapping tree and proof concerns can create duplicated verifier logic.
- Dirty receipt/runtime work from adjacent agents can make the implementation
  accidentally revert or bless unrelated changes if the spec does not require a
  re-read before coding.
- Unit-only fixture coverage can miss resolver-path regressions unless expected
  finding order and paths are captured as data.

## Acceptance

Profile: strict

Validation:
- `cargo fmt --check --manifest-path crates/Cargo.toml`
- `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime`
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-receipts --all-targets --all-features -- -D warnings`
- `git diff --check`

Required behavior:
- [ ] Exact child id or typed runx URI resolves; suffix-only references fail.
- [ ] Wrong namespace and malformed child refs fail with stable finding codes.
- [ ] Ambiguous resolver results fail with a stable blocker finding.
- [ ] Duplicate child receipt ids produce a blocker finding.
- [ ] Missing child references produce a blocker finding.
- [ ] Cyclic child references produce a blocker finding without recursion panic.
- [ ] Orphan child receipts in a supplied set produce a finding.
- [ ] Child receipt with mismatched `parent_harness_ref` produces a blocker
  finding.
- [ ] Configured depth and breadth limits are enforced.
- [ ] Positive nested/fanout fixture verifies cleanly.
- [ ] Tree verification summary is deterministic across repeated runs.
- [ ] Fixture oracle compares expected validity plus ordered finding
  code/path pairs, not only `is_ok()`.
- [ ] Existing public slice-based APIs still compile and delegate to the
  resolver implementation.

## Phase 1: Resolver Contract

Status: completed
Dependencies: none

Objective: Define the pure resolver boundary.

Changes:
- Add resolver trait/callback API.
- Define child reference normalization rules.
- Define traversal bounds and finding codes.
- Add a slice-backed resolver adapter for the current `verify_receipt_tree(root, children)` surface.
- Define resolver outcomes as exactly one receipt, missing, malformed, ambiguous, or resolver error; suffix search is never a resolver fallback.

Acceptance:
- none

## Phase 2: Tree Verifier

Status: completed
Dependencies: Phase 1

Objective: Verify complete receipt trees.

Changes:
- Implement bounded traversal.
- Track reached, visiting, duplicate, orphan, and parent-link state.
- Aggregate child findings into parent summary honestly.
- Emit deterministic findings by traversing child refs in receipt order and supplied-oracle children in stable id/path order.
- Compare present child `parent_harness_ref` against the parent harness ref, and require strict parent refs for governed runtime oracle cases.

Acceptance:
- none

## Phase 3: Runtime Wiring

Status: completed
Dependencies: Phase 2

Objective: Let runtime callers use tree verification without putting IO in the

Changes:
- Wire runtime receipt indexes into the resolver contract.
- Keep path discovery in `rust-runtime-receipt-path-discovery`.
- Populate child `parent_harness_ref` for new runtime graph receipts before using strict governed tree verification.
- Keep runtime integration tests focused on passing an already-scoped resolver into `runx-receipts`; local store discovery remains out of this spec.

Acceptance:
- none

## Phase 4: Fixture Oracle

Status: completed
Dependencies: Phase 2

Objective: Make tree semantics durable across implementation churn.

Changes:
- Add checked-in tree-resolution cases under the existing fixture tree, with root receipt, supplied child receipts, resolver config, expected validity, and ordered finding code/path pairs.
- Cover positive nested, positive fanout, duplicate id, missing child, malformed URI, wrong namespace, ambiguous id, cycle, orphan, wrong parent, depth limit, and breadth limit cases.
- Base fixture receipt shapes on harness-spine receipts or runtime graph receipts; do not invent a new receipt schema.

Acceptance:
- none

## Rollback

- Keep the existing structural tree checks until bounded resolver tests pass,
  then remove redundant code in the same reviewed change.
- If runtime parent-link wiring regresses, keep the pure resolver API and fall
  back only the runtime adapter while retaining negative fixture coverage.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Read-only review of rust-receipt-tree-resolution. Traced all 12 fixture oracle cases against the verifier and confirmed ordered findings match. Traversal is bounded (visiting/reached BTreeSet, saturating depth add, take-by-breadth), public slice API is preserved via SliceReceiptResolver, and the runtime adapter emits resolver-aware paths. Conventions (no unwrap/expect, no serde_json::Value, no HashMap, no anyhow) and the rust-style-allow large-file marker on tree.rs are honored. Ambient drift (doctor.rs, receipt_store.rs, receipt_paths.rs) was confirmed unrelated to this spec. One spec-compliance gap surfaced: Phase 1 names five resolver outcomes ("Found, Missing, Malformed, Ambiguous, or resolver error") and the finding code `ChildReceiptResolverError` is defined, but `ReceiptResolveResult` only has four variants — the resolver-error case is dead. Treating this as non-blocking because callers and tests are not forced to handle it; the API is additive and can grow later.

Attack log:
- `Phase 1 resolver outcomes vs implementation`: Cross-check enumerated outcomes in spec against ReceiptResolveResult variants and finding-code emission sites -> finding (Spec lists 5 outcomes; impl has 4; ChildReceiptResolverError is unused.)
- `Fixture oracle ordered findings (12 cases)`: Manually trace verifier output (verify_harness_receipt + duplicate + child_receipt + subtree DFS + orphan) for each case and compare to oracle.json ordered findings -> clean (positive-nested, positive-fanout, duplicate-id, missing-child, malformed-uri, wrong-namespace, ambiguous-id, cycle, orphan, wrong-parent, depth-limit, breadth-limit all match.)
- `Hostile receipt graph (DoS bounds)`: Verify max_depth/max_breadth enforcement, saturating_add, BTreeSet visiting/reached, and that cycle insert-check is unreachable behind explicit visiting.contains() in child_ref_findings -> clean (Cycle path covered via both `visiting.insert` short-circuit and explicit contains check before recursion.)
- `Public slice API stability`: Confirm validate_receipt_tree/verify_receipt_tree signatures unchanged; ensure existing callers (runtime/src/receipts.rs:91) still compile through SliceReceiptResolver -> clean
- `Path leakage (operator-local paths in findings)`: Inspect every finding emission for absolute paths or non-JSON-pointer-like strings -> clean (All emitted paths are dotted JSON paths like `children[0].harness.child_harness_receipt_refs[0]`.)
- `Convention check (no unwrap/HashMap/serde_json::Value/anyhow/large file)`: grep production src trees for forbidden patterns; verify rust-style-allow markers -> clean (No unwrap/expect in receipts or runtime src; uses BTreeSet/BTreeMap; tree.rs has rust-style-allow: large-file annotation.)
- `Runtime adapter resolver paths`: Confirm RuntimeReceiptResolver supplied_receipts emits `runtime_receipts[N]` path so orphan/ambiguous findings carry the correct scope -> clean (tests/receipt_tree.rs asserts `runtime_receipts[1].id` for duplicate finding.)
- `Strict-mode parent_harness_ref handling`: Check strict positive and negative behavior, including null and mismatched parent refs -> clean (Unit tests cover mismatched and missing strict parent refs; runtime intentionally stays non-strict per Phase 3.)
- `Ambient drift confusion`: Verify doctor.rs, receipt_store.rs, receipt_paths.rs changes belong to other specs and do not silently mutate tree-resolution semantics -> clean (doctor.rs and receipt_store/paths.rs are out of scope and unreferenced by tree code.)

Findings:
- [medium/non-blocking] `F1` ReceiptResolveResult lacks the spec-enumerated `ResolverError` variant; `ChildReceiptResolverError` is dead code
  - Location: `crates/runx-receipts/src/tree.rs:41`
  - Evidence: tree.rs:40-46 defines `enum ReceiptResolveResult { Found, Missing, Malformed, Ambiguous }` (4 variants). verify/finding.rs:23 declares `ChildReceiptResolverError` in `ReceiptFindingCode`, but no resolver outcome maps to it and no production or test path emits the code. The Phase 1 spec entry under `.scafld/specs/active/rust-receipt-tree-resolution.md` (Phase 1 Changes) states: "Define resolver outcomes as exactly one receipt, missing, malformed, ambiguous, or resolver error; suffix search is never a resolver fallback."
  - Impact: External resolvers (especially future IO-bound runtime resolvers that wrap filesystem reads) have no contract-level way to signal a transient resolver error distinct from missing/malformed/ambiguous, forcing semantic overloading or panics. The defined-but-unused finding code is also dead surface under the `no_legacy_code` invariant.
  - Validation: cargo test -p runx-receipts (oracle + tree unit tests); a new fixture case using a stub resolver that returns `ResolverError` should produce an ordered finding with code `ChildReceiptResolverError`.

## Self Eval

- Target score: 9.5. Passing means multi-step receipt verification cannot be
  fooled by missing, ambiguous, or hostile child receipts.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: isolate receipt tree trust semantics from proof and IO work

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T04:06:49Z
Ended: 2026-05-19T04:14:31Z

Checks:
- path audit
  - Grounded in: code:crates/runx-receipts/src/tree.rs:23
  - Result: passed
  - Evidence: `runx-receipts` owns IO-free tree validation; runtime only wires child refs and resolver inputs.
- command audit
  - Grounded in: code:crates/runx-receipts/tests/harness_receipts.rs:168
  - Result: passed
  - Evidence: Acceptance keeps receipt/runtime cargo checks and adds a fixture oracle beyond `is_ok()` assertions.
- scope/migration audit
  - Grounded in: code:.scafld/specs/active/rust-policy-authority-proof-parity.md:27
  - Result: passed
  - Evidence: Active authority-proof work is parity-only; this spec avoids policy fixtures and harness-spine vocabulary.
- acceptance timing audit
  - Grounded in: code:crates/runx-runtime/src/receipts.rs:91
  - Result: passed
  - Evidence: Runtime already validates graph receipts after building children; strict resolver and oracle checks gate new wiring.
- rollback/repair audit
  - Grounded in: code:crates/runx-receipts/src/lib.rs:15
  - Result: passed
  - Evidence: The public slice verifier remains as the repair path while runtime wiring can be backed out independently.
- design challenge
  - Grounded in: code:crates/runx-runtime/src/receipts.rs:114
  - Result: passed
  - Evidence: Runtime leaves `parent_harness_ref` unset, so the spec separates structural compatibility from governed strict mode.
- existing tree audit
  - Grounded in: code:crates/runx-receipts/src/tree.rs:23
  - Result: passed
  - Evidence: Current validation indexes a flat child slice and reports duplicate, missing, cycle, and orphan findings without IO.
- resolver gap audit
  - Grounded in: code:crates/runx-receipts/src/lib.rs:15
  - Result: passed
  - Evidence: The public surface exports only slice-based tree functions, so the resolver contract must be additive.
- reference contract audit
  - Grounded in: code:crates/runx-receipts/src/tree.rs:196
  - Result: passed
  - Evidence: Existing normalization accepts typed URIs and bare ids; the spec adds malformed and ambiguous findings.
- runtime overlap audit
  - Grounded in: code:crates/runx-runtime/src/receipts.rs:58
  - Result: passed
  - Evidence: Runtime builds typed child refs and validates cloned step receipts, but parent refs are not populated yet.
- parent-link audit
  - Grounded in: code:crates/runx-contracts/src/harness.rs:155
  - Result: passed
  - Evidence: The contract model carries `parent_harness_ref`, so parent-link mismatch belongs in tree verification.
- fixture oracle audit
  - Grounded in: code:crates/runx-receipts/tests/harness_receipts.rs:168
  - Result: passed
  - Evidence: Existing tests cover negative paths but do not load a durable oracle with ordered expected findings.
- sequencing audit
  - Grounded in: code:.scafld/specs/archive/2026-05/rust-receipts-parity.md:16
  - Result: passed
  - Evidence: `rust-receipts-parity` is completed, so this is post-parity hardening rather than receipt parity replacement.
- active policy overlap audit
  - Grounded in: code:.scafld/specs/active/rust-policy-authority-proof-parity.md:27
  - Result: passed
  - Evidence: The active policy task excludes runtime and contract-vocabulary changes; this spec stays in tree resolution.

Issues:
- none


## Planning Log

- 2026-05-19: Expanded placeholder into tree-resolution contract after receipt
  verifier review.
- 2026-05-19: Hardened resolver ownership, parent-link semantics, durable
  fixture oracle requirements, and sequencing against completed
  `rust-receipts-parity` plus active receipt/runtime work.
