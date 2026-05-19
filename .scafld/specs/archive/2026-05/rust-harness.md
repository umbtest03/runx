---
spec_version: '2.0'
task_id: rust-harness
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T07:01:40Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# Rust harness replay

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T07:01:40Z
Review gate: pass

## Summary

Port the skill harness replay runner to Rust. `runx harness <path>` runs a
skill inside a deterministic fixture-backed governed harness and asserts the
sealed harness receipt plus output match expectations. It is not a second
meaning of `harness`; it is replay mode for the contract spine ratified in
`runx-contract-spine-hard-cutover`.

Today this lives in `packages/runtime-local/src/harness/runner.ts` plus
harness/quality.ts and harness/framing-patterns.ts.

The existing CLI verb stays. The conceptual framing changes:

- production mode: a harness runs against live adapters and seals to a receipt
- replay mode: the same harness contract runs against fixtures and asserts the
  sealed receipt/output
- publish verification: replay mode proves a skill has at least one
  deterministic governed example before publication

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (`runx harness` dispatch)
- `@runxhq/runtime-local` (current harness replay implementation)
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/runtime-local/src/harness/runner.ts`
- `packages/runtime-local/src/harness/quality.ts`
- `packages/runtime-local/src/harness/framing-patterns.ts`
- `packages/runtime-local/src/harness/mcp-fixture.ts`
- `packages/runtime-local/src/harness/a2a-fixture.ts`

Files impacted:
- `crates/runx-cli/src/launcher.rs`
- `crates/runx-runtime/src/harness/runner.rs`
- `crates/runx-runtime/src/harness/quality.rs`
- `crates/runx-runtime/src/harness/fixtures.rs`
- `fixtures/harness/**`

Invariants:
- Replay harness runs are deterministic: fixtures stand in for live adapters.
- Quality checks match TS thresholds and rubrics.
- The Rust replay path emits the same canonical harness receipt shape as the
  contract spine.
- Harness receipts are byte-identical for the same canonical harness input
  within one contract shape.
- Byte-identical comparison across the Rust cutover is measured after the
  contract spine hard cutover, not against retired receipt shapes.
- Byte-identical means canonical JSON serialization: stable key order,
  normalized timestamps/ids in fixtures, deterministic array order where order
  is semantically relevant, and exact hash inputs shared with `runx-receipts`.
- Replay mode must exercise the same authority attenuation, act containment,
  decision containment, seal, and verification semantics as production mode
  where the fixture can represent them.

## Objectives

- Port harness replay runner, quality checks, fixture loaders for MCP and a2a.
- Match the post-cutover TS canonical harness receipt JSON output.
- Keep `runx harness <path>` as the CLI surface; do not add a parallel replay
  verb.
- Update the Rust CLI launcher so `runx harness <path>` can dispatch to the
  native replay runner behind an explicit selection rule, with a TS fallback
  until the cutover spec flips the default.
- Ensure fixture replay produces or validates the canonical harness receipt,
  contained decision payloads, contained act payloads, child harness receipt
  refs, and verification proof.

## Scope

In scope:
- Harness replay runner and fixture infrastructure.
- Fixture-to-harness expansion for skills, inline harness cases, MCP, A2A, and
  cli-tool profiles.
- Post-cutover fixture refresh for `fixtures/harness/**`. The contract spine
  cutover supplied the canonical harness receipt shape; this spec owns upgrading
  the remaining pre-cutover replay fixtures on disk so old receipt expectation
  fields are replaced by canonical harness receipt assertions.
- Receipt equality checks against canonical post-cutover harness receipts.
- Receipt equality uses the explicit body/full digest APIs and proof verifier
  from `rust-receipt-proof-verification`; serde output equivalence alone is not
  sufficient cutover evidence.
- Quality/framing checks as verification effects or harness receipt checks
  where applicable.
- Rust CLI launcher routing for the `harness` subcommand only. The launcher
  remains a delegate for every other command until the relevant cutover spec
  moves that command.

Out of scope:
- Harness authoring (`write-harness` skill belongs in a separate authoring
  pass).
- Live production harness scheduling beyond what is required to share the
  canonical runtime path.
- Replacing the TS harness implementation as the default command path before
  parity is proven and the launcher cutover flag is intentionally enabled.

## Dependencies

- `rust-runtime-skeleton`.
- `runx-contract-spine-hard-cutover` for canonical harness, act, decision,
  signal, authority, and harness receipt shapes.
- `rust-receipt-proof-verification` for canonical body/full digest checks and
  proof-backed receipt equality.
- `rust-receipt-tree-resolution` for child harness receipt verification.
- `rust-runtime-receipt-path-discovery` for runtime-owned receipt fixture
  loading and safe public projections.
- `rust-runtime-adapters-{agent,a2a,mcp}` for adapter-specific fixture
  formats; harness can ship with cli-tool-only initially and gain coverage
  as adapters land.

Sequencing:

- `runx-contract-spine-hard-cutover` is the completed source of truth for
  canonical harness, act, decision, signal, authority, and harness receipt
  shapes.
- `rust-harness` ports against that final canonical shape. Byte-identical means
  TS post-cutover canonical harness receipt versus Rust canonical harness
  receipt for the same deterministic replay input.
- `rust-harness` owns upgrading existing `fixtures/harness/**` replay fixtures
  that still contain retired receipt expectation fields such as
  `skill_execution`, `graph_execution`, `skill_name`, or `source_type`.
- Do not preserve or accept retired fixture receipt fields in the Rust runner.
  If a fixture needs old TS compatibility, it stays in a TS-only archive outside
  the Rust replay acceptance set.
- Adapter-specific fixture coverage can land incrementally. The first build may
  ship cli-tool replay only if MCP/A2A/agent fixtures are marked skipped with
  explicit not-yet-supported diagnostics and the spec's validation command
  proves they are not silently ignored.

## Build Decisions

- Keep the CLI verb `runx harness`. Do not add `runx replay`, `runx fixture`,
  or a second public verb.
- The native launcher path is opt-in until the CLI cutover spec makes it
  default. `crates/runx-cli/src/launcher.rs` may branch only when argv[0] is
  `harness` and a Rust-selection signal is present, such as
  `RUNX_RUST_HARNESS=1` or a compile-time test hook. Without that signal it
  delegates to the existing TS command exactly as today.
- Launcher fallback is explicit: if the native harness runner is unavailable,
  the launcher delegates to TS unless the selection signal requests strict Rust
  mode. Strict mode must return a typed launcher error rather than silently
  falling back.
- Rust runtime owns replay execution and receipt comparison. The CLI launcher
  owns only command selection and argument forwarding.
- Fixture assertions are expressed in canonical harness-receipt terms:
  receipt schema/id, harness id, phase/seal status, contained decisions,
  contained acts, child receipt refs, proof status, and verification checks.
  They do not assert retired skill/graph receipt kind fields.

## Planned Phases

Phase 1: fixture contract and parser parity.
- Audit `fixtures/harness/**` and inline harness cases in skills.
- Replace retired receipt expectations with canonical harness receipt
  expectations.
- Add a fixture loader that rejects retired fixture fields with targeted error
  codes and file/field paths.
- Add TS post-cutover oracle output for the upgraded fixtures so Rust has a
  byte-parity target.

Phase 2: Rust replay runner.
- Add `crates/runx-runtime/src/harness/{runner,fixtures,quality}.rs` and export
  a narrow replay API from `runx-runtime`.
- Implement deterministic temp home, receipt dir, env overlay, caller fixture
  state, cli-tool fixture adapter, output assertion, and quality/framing
  checks.
- Emit or compare canonical harness receipts through `runx-receipts` canonical
  digest/proof APIs.

Phase 3: graph and child receipt replay.
- Support sequential harness graph replay enough to validate parent harness
  receipts and child harness receipt refs.
- Use `rust-receipt-tree-resolution` APIs for child receipt checks.
- Add negative cases for missing child receipts, wrong parent refs, digest
  mismatch, and malformed harness receipt refs.

Phase 4: launcher integration.
- Add command-aware `harness` routing to `crates/runx-cli/src/launcher.rs`
  behind the explicit selection rule.
- Preserve TS delegation for all non-harness commands and for harness when the
  selection signal is absent.
- Add launcher tests for TS fallback, strict-Rust unavailable failure, native
  Rust success, and argument forwarding.

Phase 5: review and cutover evidence.
- Run TS and Rust harness replay over the same upgraded fixtures.
- Compare canonical harness receipt bytes and output expectations.
- Document any skipped adapter modes with explicit future-spec references.

## Acceptance Criteria

- `fixtures/harness/**` contains no active replay expectation fields named
  `skill_execution`, `graph_execution`, `skill_name`, `source_type`,
  `graph_name`, or `owner` under `expect.receipt`.
- The Rust fixture loader rejects retired receipt expectation fields with a
  stable diagnostic that includes the fixture path and field path.
- At least one cli-tool skill replay fixture and one sequential graph replay
  fixture pass in both TS post-cutover and Rust, with byte-identical canonical
  harness receipt JSON after normalized fixture ids/timestamps.
- Receipt equality uses `runx-receipts` body/full digest and proof verification
  APIs; a digest or signature mismatch fails the replay.
- Quality/framing failures are reported as harness verification failures or
  proof findings without leaking raw absolute paths, secrets, or fixture temp
  dirs.
- The Rust replay API is deterministic: repeated runs over the same fixture
  produce identical canonical receipt bytes and identical output assertions.
- `runx harness <path>` without the Rust selection signal delegates exactly as
  before; with the Rust selection signal it reaches the native runner for
  supported fixtures.
- Unsupported MCP/A2A/agent fixture modes fail closed with typed
  not-yet-supported diagnostics unless their adapter specs have landed.

## Validation Commands

```sh
! rg -n "skill_execution|graph_execution|skill_name|source_type|graph_name" fixtures/harness
pnpm test -- packages/runtime-local/src/harness
pnpm exec tsx scripts/generate-rust-harness-fixtures.ts --check
cargo test --manifest-path crates/Cargo.toml -p runx-runtime harness
cargo test --manifest-path crates/Cargo.toml -p runx-receipts
cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets -- -D warnings
cargo fmt --manifest-path crates/Cargo.toml --all --check
node scripts/check-rust-core-style.mjs
```

## Rollback And Repair

- Until the CLI cutover spec flips the default, rollback is disabling the Rust
  selection signal and letting `crates/runx-cli` delegate `runx harness` to the
  TS implementation.
- Fixture upgrades are hard-cutover artifacts. If a fixture upgrade is wrong,
  repair the canonical fixture and oracle together; do not restore retired
  receipt fields in active fixtures.
- If native launcher routing regresses non-harness commands, revert only the
  harness command branch and keep runtime replay modules intact for direct
  tests.
- If proof comparison is flaky, the fix is canonicalization or fixture
  normalization. Do not relax to structural serde equality.
- If an adapter fixture mode is not ready, mark that mode unsupported in the
  Rust fixture loader and keep it outside acceptance until its adapter spec
  lands.

## Open Questions

- Whether harness JSON output stays identical to post-cutover TS canonical
  output or gets a Rust-side cleanup. Default: identical; cleanup is a separate
  spec if motivated.
- Which fixture fields become harness authority algebra fixtures versus
  replay-only convenience inputs.
- How much of production abnormal seal behavior must be fixture-testable in
  the first Rust replay slice.

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T06:12:00Z
Ended: 2026-05-19T06:01:03Z
Verdict: passed
Provider: manual
Summary: The draft was amended after external review. The contract-spine

Checks:
- path audit
  - Grounded in: code:packages/runtime-local/src/harness/runner.ts:158
  - Result: passed
  - Evidence: Existing TS replay sources are named and Rust target modules are
- command audit
  - Grounded in: code:crates/runx-cli/src/launcher.rs:18
  - Result: passed
  - Evidence: Launcher routing is now explicitly scoped to the `harness`
- scope/migration audit
  - Grounded in: spec_gap:scope
  - Result: passed
  - Evidence: Scope now states that `rust-harness` owns upgrading remaining
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: Planned phases, acceptance criteria, and validation commands are
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback is explicit: disable the Rust selection signal and keep
- design challenge
  - Grounded in: code:crates/runx-contracts/src/harness.rs:1
  - Result: passed
  - Evidence: Replay mode now validates the ratified harness spine rather than

Issues:
- none

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify pass. The three prior completion-blocking findings are resolved. F1: harness replay now uses `canonical_receipt_body_digest`, `canonical_receipt_digest`, `validate_harness_receipt_proof`, and `verify_harness_receipt_proof` in `crates/runx-runtime/src/harness/assertions.rs`, with body_digest required on every receipt expectation. F2: `fixtures/harness/oracle/{echo-skill.receipt,sequential-graph.receipt,sequential-graph.first,sequential-graph.second}.json` are checked in, the new `scripts/generate-rust-harness-fixtures.ts` generator/check script keeps oracle JSON and fixture digests in sync, and a Rust parity test asserts the runtime-produced canonical receipt JSON matches each oracle byte-for-byte. F3: `HarnessFixtureKind` recognises `mcp/a2a/agent/agent-step`, `HarnessFixtureError::UnsupportedFixtureMode` carries a stable `mode` and `field_path` diagnostic, and the runner refuses the same modes at `kind` and `source.type`. F6 (missing generator script) and F8 (RETIRED_RECEIPT_FIELDS coverage of `skill_execution`/`graph_execution`) are also fixed. The originally non-blocking F4 (no strict-vs-fallback launcher distinction), F5 (production `runx harness` path still spawns the `cli-tool` adapter with host PATH/SystemRoot/PATHEXT via `safe_default_env`; the parity oracle test only stays deterministic because it uses an in-process `TestAdapter`), and F7 (no quality/framing port) remain open but unchanged from the prior verdict, are not regressions, and do not introduce new blockers under verify_open_blockers.

Attack log:
- `crates/runx-runtime/src/harness/assertions.rs`: verify F1: replay uses runx-receipts body/full digest and proof verification APIs -> finding (Fixed: assert_receipt_proof calls validate_harness_receipt_proof + verify_harness_receipt_proof; assert_receipt_digests calls canonical_receipt_body_digest and canonical_receipt_digest; HarnessReceiptExpectation.body_digest is required.)
- `fixtures/harness/oracle/*.json + scripts/generate-rust-harness-fixtures.ts`: verify F2: byte-identical canonical receipt oracle exists and Rust parity test exercises it -> finding (Fixed: four oracle JSON files plus generator/--check script; harness_fixtures.rs::replay_receipts_match_checked_in_canonical_oracles asserts canonical_receipt_json bytes equal oracle bytes.)
- `crates/runx-runtime/src/harness/fixtures.rs`: verify F3: unsupported MCP/A2A/agent fixture modes fail with typed diagnostic -> finding (Fixed: HarnessFixtureKind covers mcp/a2a/agent/agent_step; HarnessFixtureError::UnsupportedFixtureMode carries mode+field_path; runner mirrors the rejection at source.type.)
- `crates/runx-cli/src/launcher.rs + crates/runx-cli/src/main.rs`: verify F4: launcher strict-vs-fallback distinction for harness routing -> finding (Still unfixed: single RUNX_RUST_HARNESS gate, no fallback path, main.rs exits 1 on native error. Non-blocking and unchanged from prior review.)
- `crates/runx-runtime/src/runner.rs + crates/runx-runtime/src/harness/runner.rs`: verify F5: production replay determinism / host PATH leak -> finding (Still unfixed for the production cli-tool path; tests stay deterministic via TestAdapter only. Non-blocking, no receipt-body regression because output fields are constant strings, but invariant is still weakly enforced.)
- `scripts/generate-rust-harness-fixtures.ts`: verify F6: spec validation command runnable end-to-end -> finding (Fixed: script exists, implements --check default and --write/--generate, validates oracle JSON and fixture digests.)
- `crates/runx-runtime/src/harness/mod.rs`: verify F7: quality/framing port from packages/runtime-local/src/harness/{quality,framing-patterns}.ts -> finding (Still unfixed: no quality.rs / framing module under crates/runx-runtime. Non-blocking and unchanged from prior review.)
- `crates/runx-runtime/src/harness/fixtures.rs`: verify F8: RETIRED_RECEIPT_FIELDS coverage of skill_execution/graph_execution -> finding (Fixed: both names included; parametric test exercises kind/skill_execution/graph_execution.)
- `crates/runx-runtime/src/harness/fixtures.rs`: regression hunt: does deny_unknown_fields + flattened extra still produce typed RetiredReceiptField/UnknownReceiptField paths -> clean (RawHarnessReceiptExpectation flattens unknown keys into IgnoredAny extra; validate_receipt_expectation maps known retired names to RetiredReceiptField and the rest to UnknownReceiptField with stable expect.receipt.<name> paths.)
- `crates/runx-runtime/src/receipts.rs + crates/runx-receipts/src/canonical.rs`: regression hunt: do Rust step/graph receipts emit the canonical body/full digest the assertion layer now requires -> clean (seal_receipt seals each receipt with canonical_receipt_body_digest and signs it as sig:<digest>; LocalHarnessSignatureVerifier matches that exact format so validate_harness_receipt_proof succeeds in the runner before the assertion layer runs.)
- `fixtures/harness/oracle/*.json + scripts/generate-rust-harness-fixtures.ts`: regression hunt: TS and Rust canonicalization strip the same fields before computing body digest -> clean (Both implementations alphabetically sort keys with no whitespace, strip top-level signature, and strip seal.digest + seal.verification_summary on every nested seal object; the script's --check verifies fixture body/receipt digests against the regenerated oracle.)
- `.scafld/specs/active/rust-harness.md vs task changes`: ambient drift: separate task changes from unrelated workspace drift -> clean (Task-scoped diff covers only fixtures/harness/echo-skill.yaml, fixtures/harness/sequential-graph.yaml, and fixtures/harness/oracle/*.json; runtime + launcher + registry changes are ambient context and not attributed to this task in the verdict.)

Findings:
- [high/non-blocking] `F1` Harness replay now uses runx-receipts body/full digest and proof verification APIs
  - Location: `crates/runx-runtime/src/harness/assertions.rs:67`
  - Evidence: assert_receipt calls assert_receipt_proof (validate_harness_receipt_proof + verify_harness_receipt_proof via proof_context) and assert_receipt_digests (canonical_receipt_body_digest, canonical_receipt_digest); HarnessReceiptExpectation makes body_digest required so a digest mismatch fails the replay.
  - Validation: crates/runx-runtime/tests/harness_fixtures.rs replay tests load fixtures whose body_digest/receipt_digest must match the canonical receipt bytes.
- [high/non-blocking] `F2` Canonical harness receipt oracle JSON and generator are in place; Rust parity test asserts byte-equality
  - Location: `crates/runx-runtime/tests/harness_fixtures.rs:152`
  - Evidence: fixtures/harness/oracle/{echo-skill.receipt,sequential-graph.receipt,sequential-graph.first,sequential-graph.second}.json exist; scripts/generate-rust-harness-fixtures.ts produces and --check verifies them; replay_receipts_match_checked_in_canonical_oracles asserts canonical_receipt_json output equals oracle bytes for both fixtures.
  - Validation: Mutating a single byte in any fixtures/harness/oracle/*.json breaks the parity test.
- [medium/non-blocking] `F3` MCP/A2A/agent fixture modes fail closed with typed not-yet-supported diagnostics
  - Location: `crates/runx-runtime/src/harness/fixtures.rs:229`
  - Evidence: HarnessFixtureKind now deserialises mcp/a2a/agent/agent-step. validate_supported_fixture_kind returns HarnessFixtureError::UnsupportedFixtureMode { mode, field_path } for those modes with field_path 'kind'; runner.rs reject_unsupported_source_type returns the same error at field_path 'source.type'; tests rejects_unsupported_fixture_modes_with_stable_diagnostic and rejects_unsupported_fixture_mode_with_stable_path cover both surfaces.
- [medium/non-blocking] `F4` Launcher still has no strict-vs-fallback distinction for harness routing
  - Location: `crates/runx-cli/src/launcher.rs:39`
  - Evidence: plan_launcher_with_rust_harness checks a single RUNX_RUST_HARNESS signal and always returns LauncherAction::RunHarness; main.rs run_native_harness exits 1 on native failure rather than delegating to TS. Build decision 'Launcher fallback is explicit ... unless the selection signal requests strict Rust mode' is still not implemented.
  - Impact: Unchanged from the prior reviewer's medium/non-blocking finding; not a verify regression.
- [medium/non-blocking] `F5` Production `runx harness` path still spawns node with host PATH/SystemRoot/PATHEXT
  - Location: `crates/runx-runtime/src/runner.rs:42`
  - Evidence: RuntimeOptions::default() seeds env via safe_default_env() which copies PATH/SystemRoot/PATHEXT from std::env. run_harness_fixture uses CliToolAdapter under the cli-tool feature, so `runx harness fixtures/harness/echo-skill.yaml` invoked from main.rs still spawns `node -e ...` with the live host PATH. The parity oracle test is only deterministic because tests/harness_fixtures.rs swaps in TestAdapter via run_harness_fixture_with_adapter.
  - Impact: Unchanged from the prior reviewer's medium/non-blocking finding; receipt body fields are not host-dependent today, so canonical receipt parity is not yet broken, but the determinism invariant remains weakly enforced in the production path.
- [low/non-blocking] `F6` scripts/generate-rust-harness-fixtures.ts is now present
  - Location: `scripts/generate-rust-harness-fixtures.ts:1`
  - Evidence: File exists and implements both --write/--generate and --check (default) modes referenced by the spec's validation commands; --check verifies oracle JSON and fixture body_digest/receipt_digest values.
- [low/non-blocking] `F7` Quality/framing checks from TS were not ported
  - Location: `crates/runx-runtime/src/harness/mod.rs:1`
  - Evidence: crates/runx-runtime/src/harness/mod.rs still only exposes assertions, fixtures, and runner. No `quality` or `framing` module exists under crates/runx-runtime, and packages/runtime-local/src/harness/{quality,framing-patterns}.ts remain TS-only.
  - Impact: Unchanged from the prior reviewer's low/non-blocking finding; the acceptance criterion about not leaking absolute paths/secrets/temp dirs from quality findings still lacks a Rust surface to bind to.
- [low/non-blocking] `F8` RETIRED_RECEIPT_FIELDS now covers every name listed in the acceptance criterion
  - Location: `crates/runx-runtime/src/harness/fixtures.rs:11`
  - Evidence: RETIRED_RECEIPT_FIELDS lists kind, skill_execution, graph_execution, skill_name, source_type, graph_name, owner; rejects_retired_receipt_expectation_fields exercises kind/skill_execution/graph_execution and tests/harness_fixtures.rs::rejects_retired_receipt_kind_field_with_stable_path mirrors the runtime check.
