---
spec_version: '2.0'
task_id: rust-doctor
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T05:36:59Z'
status: completed
harden_status: passed
size: small
risk_level: low
---

# Rust doctor

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T05:36:59Z
Review gate: pass

## Summary

Add a native Rust doctor core for the current machine-actionable `runx doctor`
report. This is a parity slice for existing TypeScript diagnostics, not a new
environment health checker and not a CLI cutover.

The first Rust slice should produce `runx.doctor.v1` reports from deterministic
fixture workspaces, prove those reports against a TypeScript oracle, and expose a
programmatic runtime API that later `runx-cli`, Aster, and dev workflows can
call. The npm TypeScript CLI remains authoritative until a separate CLI cutover
spec wires the command surface to Rust.

## Context

CWD: `.`

Authoritative TypeScript surfaces:
- `packages/cli/src/commands/doctor.ts`
- `packages/cli/src/commands/doctor-structure.ts`
- `packages/cli/src/commands/doctor-types.ts`
- `packages/cli/src/commands/dev.ts`
- `packages/contracts/src/schemas/doctor.ts`
- `schemas/doctor.schema.json`

Rust surfaces for this slice:
- `crates/runx-contracts/src/doctor.rs`
- `crates/runx-contracts/src/lib.rs`
- `crates/runx-runtime/src/doctor.rs`
- `crates/runx-runtime/src/doctor/checks.rs`
- `crates/runx-runtime/src/doctor/fixtures.rs`
- `crates/runx-runtime/src/lib.rs`
- `crates/runx-runtime/tests/doctor.rs`
- `fixtures/doctor/**`
- `scripts/generate-rust-doctor-fixtures.ts`

Existing TypeScript doctor behavior verified during hardening:
- `handleDoctorCommand` resolves a target root, gathers structural, tool, skill,
  and packet diagnostics, computes summary counts, and sorts diagnostics by
  `location.path` then `id`.
- `createDoctorDiagnostic` computes `instance_id` from `id`, `target`,
  `location`, and `evidence`; those IDs are part of the public JSON contract.
- `runx dev` embeds the doctor report before fixture execution.
- `runx doctor --fix` can write safe low-risk repair contents. That write path is
  deliberately out of this first Rust slice.

Current TypeScript diagnostic families:
- Structural: official skills lock freshness, file budgets, cross-package
  `src` reach-ins.
- Tools: removed `tool.yaml`, invalid `manifest.json`, missing deterministic
  fixtures, stale source/schema hashes.
- Skills and graphs: invalid `X.yaml`, missing harness coverage, graph context
  producer/output/envelope/schema/path errors.
- Packets: local packet index errors from `buildLocalPacketIndex`.
- Catalog: `--list-diagnostics` and `--explain <id>` expose stable explanatory
  metadata.

Runtime boundary:
- `runx-runtime` owns filesystem reads and local workspace probing.
- `runx-contracts` owns serializable Rust `runx.doctor.v1` shapes.
- `runx-cli` is still a Cargo launcher that delegates to npm; command routing and
  renderer cutover are out of scope here.

## Objectives

- Add typed Rust contract structs/enums for `runx.doctor.v1` without changing the
  TypeScript schema.
- Add a read-only Rust runtime doctor API for fixture-backed local workspace
  diagnostics.
- Generate TypeScript-oracle doctor fixtures and compare Rust reports directly
  against them.
- Preserve deterministic diagnostic ordering, summary counts, repair metadata,
  and `instance_id` values for the included diagnostics.
- Make follow-up ownership explicit for current TS doctor surfaces not included
  in this first slice.

## Scope

In scope:
- Rust `DoctorReport`, `DoctorDiagnostic`, `DoctorRepair`, `DoctorLocation`, and
  `DoctorSummary` contract types matching `schemas/doctor.schema.json`.
- Read-only Rust runtime checks for this first fixture-backed set:
  - `runx.tool.manifest.removed_format`
  - `runx.tool.fixture.missing`
  - `runx.skill.fixture.missing`
  - `runx.structure.file_budget.exceeded`
  - `runx.structure.cross_package_reach_in`
- Positive empty-workspace fixture that returns `status: "success"` with no
  diagnostics.
- TypeScript oracle fixture generation for every in-scope diagnostic and the
  positive empty-workspace case.
- Rust tests that compare whole JSON reports, including `summary`, sorted
  diagnostics, repair arrays, and `instance_id`.

Out of scope:
- `runx doctor` CLI routing, human renderer parity, `--json` command output
  cutover, `--list-diagnostics`, and `--explain`.
- `runx doctor --fix` writes. Rust must not write repairs in this slice.
- New environment probes for Node, git, scafld, network, sandbox capability, or
  config validity unless TypeScript already has an equivalent diagnostic.
- Official skills lock freshness. It depends on registry skill-version hashing
  and belongs with Rust registry/catalog parity.
- Tool manifest stale source/schema hashing. It belongs with Rust tool authoring
  and manifest-build parity.
- Graph packet path validation and packet-index diagnostics. They belong with a
  follow-up Rust packet-index/graph-context doctor slice.
- Receipt proof, receipt store, and authority-proof health diagnostics. Those
  can be added only after their owning Rust policy/receipt specs land.

## Dependencies

- `rust-runtime-skeleton` is complete and establishes `runx-runtime` as the
  impure filesystem/subprocess boundary.
- `rust-contracts-parity` is complete and establishes typed Rust contract module
  patterns.
- Sequence after `rust-policy-authority-proof-parity` so this spec does not
  race the active policy implementation.
- Coordinate with, but do not block on, `rust-receipt-proof-verification` and
  `rust-runtime-receipt-path-discovery`; receipt diagnostics are explicitly out
  of scope.

## Acceptance

Profile: light

Validation:
- [x] `v1` command - TypeScript oracle doctor fixtures are current.
  - Command: `pnpm exec tsx scripts/generate-rust-doctor-fixtures.ts --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-34
- [x] `v2` command - Existing TypeScript doctor tests still pass.
  - Command: `pnpm exec vitest run --config vitest.config.ts packages/cli/src/index.test.ts -t "runx doctor"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-35
- [x] `v3` command - Rust doctor contract types round-trip fixture reports.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts doctor`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `v4` command - Rust runtime doctor reports match the TypeScript oracle.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime doctor`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `v5` command - Rust formatting and clippy pass for touched crates.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml -p runx-contracts -p runx-runtime --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38
- [x] `v6` command - Repository Rust style guard passes.
  - Command: `node scripts/check-rust-core-style.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-39
- [x] `v7` command - Scoped diff has no whitespace errors.
  - Command: `git diff --check -- .scafld/specs/drafts/rust-doctor.md crates/runx-contracts crates/runx-runtime fixtures/doctor scripts/generate-rust-doctor-fixtures.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-40

## Phases

### Phase 1 - Contract And Oracle Fixtures

Goal: establish the wire shape before porting checks.

Tasks:
- Add `crates/runx-contracts/src/doctor.rs` with typed serde models for the
  doctor schema.
- Re-export the models from `crates/runx-contracts/src/lib.rs`.
- Add `scripts/generate-rust-doctor-fixtures.ts` that constructs temporary
  workspaces, runs the existing TypeScript `runCli(["doctor", "--json"])`, and
  writes expected reports under `fixtures/doctor/<case>/expected.json`.
- Include fixtures for the positive empty workspace and every in-scope
  diagnostic.

Exit criteria:
- TypeScript oracle fixtures are byte-stable under `--check`.
- Rust contract tests can deserialize and reserialize every expected report.

### Phase 2 - Runtime Read-Only Checks

Goal: port the first filesystem-backed checks without changing CLI routing.

Tasks:
- Add `runx_runtime::doctor` with a programmatic `run_doctor(root, options)` API.
- Implement only the in-scope diagnostics listed in `## Scope`.
- Compute `instance_id` through an explicit helper that is fixture-proven against
  TypeScript; do not rely on incidental serde map ordering.
- Keep repair metadata in the report, but do not apply repairs.
- Add whole-report fixture comparisons in `crates/runx-runtime/tests/doctor.rs`.

Exit criteria:
- Rust runtime doctor reports match the TypeScript expected JSON for all
  fixture cases.

### Phase 3 - Documentation And Follow-Up Map

Goal: make the partial native slice safe to build on.

Tasks:
- Document that the native doctor API is programmatic only until
  `rust-cli-rust-cutover` or a dedicated CLI doctor cutover routes `runx doctor`.
- Add follow-up comments/docs for the deferred surfaces: `--fix`, diagnostic
  catalog, official lock freshness, tool manifest stale hashes, packet index,
  graph packet path validation, receipt proof health, and policy health.

Exit criteria:
- A future implementer can tell which TS diagnostics are ported, which are
  deferred, and which owner spec should add each deferred family.

## Invariants

- TypeScript remains authoritative for doctor fixture generation until a
  dedicated cutover spec changes ownership.
- This spec must not edit TypeScript doctor behavior except the fixture generator
  that invokes it.
- This spec must not edit `runx-cli` command routing or make the Cargo launcher
  call Rust doctor directly.
- Runtime doctor checks are read-only in this slice.
- Rust report parity is whole-JSON fixture parity, not selected-field matching.
- `runx-contracts` must use typed structs/enums plus `runx_contracts::JsonValue`
  carriers where the schema intentionally allows unknown records; production
  source must still satisfy `scripts/check-rust-core-style.mjs`.

## Risks

- Medium: `instance_id` hashing can drift if Rust serializes nested flexible
  records in a different order from the TypeScript object literals. Mitigate with
  TypeScript-oracle fixtures that include the final `instance_id` and whole-JSON
  Rust comparisons.
- Low: the style guard walks every Rust crate, so unrelated in-flight Rust style
  failures can block `v6`. Keep the required crate-local checks in `v3` through
  `v5`; treat `v6` failures outside touched crates as coordination work with the
  owning agent.
- Low: partial doctor parity could be mistaken for CLI readiness. Mitigate by
  keeping command routing out of scope and documenting the programmatic-only API.

## Rollback

- If Rust doctor contract types are wrong, remove `crates/runx-contracts/src/doctor.rs`
  and its re-exports; TypeScript schemas remain canonical.
- If a runtime check diverges from TypeScript, leave the fixture in place, remove
  or disable only the Rust check under development, and keep `runx doctor` routed
  to TypeScript.
- If fixture generation exposes a TypeScript doctor bug, keep this spec open and
  fix TypeScript in a separate doctor-maintenance spec before regenerating the
  Rust oracle.
- Because this slice does not change CLI routing or apply repairs, rollback does
  not require migration, data cleanup, or user-facing command changes.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify pass over the previous discover-mode blockers. All three completion-blocking findings (F1 missing runtime doctor module, F2 vacuous v4 cargo test filter, F3 absent Phase 3 follow-up docs) are now addressed: `crates/runx-runtime/src/doctor.rs` exposes `run_doctor` with deterministic sort, summary, `instance_id` parity helpers, and graceful handling of missing workspace directories; `crates/runx-runtime/tests/doctor.rs` exercises all six fixtures by name (so the v4 `doctor` filter no longer matches zero tests); and `crates/runx-runtime/README.md` enumerates the ported diagnostics, deferred families, and CLI-cutover status. The contract-side fixtures and tests, fixture generator, and TypeScript oracle remain consistent with `schemas/doctor.schema.json`. F4 (BTreeMap key ordering hides byte-order divergence) is unfixed but remains low/non-blocking under the spec invariant `Rust report parity is whole-JSON fixture parity, not selected-field matching`, since hash material is constructed in TS insertion order via parallel JSON strings. Several latent fixture-uncovered divergences from the TS oracle exist (substring-based skill profile heuristics, Rust erroring on malformed tool manifest.json where TS produces `runx.tool.manifest.invalid`, and the untracked empty `empty-success/workspace/` directory) but each falls outside the fixture-parity contract and the verify:fast CI lane, so they are recorded as low/medium non-blocking quality findings.

Attack log:
- `previous F1 (runtime doctor module missing)`: Verify `crates/runx-runtime/src/doctor.rs`, `tests/doctor.rs`, and `run_doctor` symbol now exist and exercise the five in-scope diagnostics plus empty-success. -> clean (doctor.rs declares `run_doctor`, sorts diagnostics by location/path then id, and tests/doctor.rs iterates all six fixtures comparing serde_json::Value equality.)
- `previous F2 (vacuous v4 cargo test filter)`: Confirm at least one test name in crates/runx-runtime/tests/ contains the substring `doctor` so that `cargo test -p runx-runtime doctor` is no longer vacuously green. -> clean (`doctor_runtime_matches_all_fixture_reports` matches the filter; the test asserts `actual == expected` per fixture so a regression would fail v4 loudly.)
- `previous F3 (Phase 3 follow-up docs)`: Search crates/runx-runtime/README.md and crates/runx-contracts/src/doctor.rs for documentation of the programmatic-only Rust doctor API and the deferred diagnostic families. -> clean (runx-runtime README has a Doctor section listing ported diagnostics and eight deferred families with their owner spec hint.)
- `previous F4 (BTreeMap-ordered round-trip)`: Re-check whether contract round-trip equality now operates at byte level or still via Value/BTreeMap. -> finding (Still Value/BTreeMap-based; same low-severity risk recorded as F4 here.)
- `instance_id parity helper`: Trace the hand-built hash material string vs the TS `JSON.stringify({id,target,location,evidence})` insertion order for each diagnostic in scope. -> clean (Outer keys ordered id,target,location,evidence; inner target/evidence keys match TS insertion order for all six fixtures (verified against expected.json files).)
- `cross-package reach-in path escape`: Confirm `lexical_normalize` plus `project_segments` rejects `../`-escaping specifiers (no false negatives, no panic). -> clean (ParentDir pops normalized stack and only pushes `..` when stack is empty, so paths outside `<root>/packages` skip the diagnostic; matches TS behavior.)
- `missing-workspace tolerance`: Walk `run_doctor` on a non-existent path to ensure it returns a success report instead of an IO error. -> clean (`read_dir_sorted` swallows NotFound; existence checks gate file budget and cross-package scanners; empty-success fixture round-trips even without a real workspace dir.)
- `TS oracle invariants vs included fixtures`: Diff each expected.json against the TS doctor.ts/doctor-structure.ts builders, including target/evidence/location keys and repair metadata. -> clean (All five failure diagnostics and the empty-success success report align with the TS builders; instance_id digests are produced and recorded by the TS oracle, not the Rust code.)
- `ambient drift attribution`: Cross-reference 38 ambient drift entries against the task's declared scope (contracts/doctor, runtime/doctor, fixtures/doctor, fixture generator). -> clean (Receipt-store/receipt-tree/verify-proof additions are out-of-scope work owned by other Rust specs and are not treated as findings here.)
- `skill profile heuristics`: Compare Rust `.contains("runners:")` and `inline_harness_case_count` against TS YAML parse + harness.cases length. -> finding (Recorded as F2; latent fixture-uncovered divergence.)
- `tool manifest probe failure mode`: Compare Rust JSON probe error path against TS try/catch -> runx.tool.manifest.invalid behavior. -> finding (Recorded as F1; Rust returns RuntimeError where TS returns a diagnostic.)
- `empty-success workspace dir tracking`: List fixtures/doctor/empty-success/ to verify the workspace directory is materialised on disk under git or via .gitkeep. -> finding (Recorded as F3; only expected.json is tracked, so `--check` would fail on clean clone.)
- `v6 rust style guard`: Check that doctor.rs respects no-unwrap, no-HashMap, no-serde_json::Value rules and that `rust-style-allow: large-file`/`long-function` comments are positioned where the guard expects them. -> clean (Only `.unwrap_or`, `.map_or_else`, etc. appear; large-file allow comment is in the top of doctor.rs; long-function allows precede the two oversized fns.)
- `v7 git diff --check (scoped)`: Inspect scoped paths (spec, contracts, runtime, fixtures, generator) for whitespace/trailing-space hazards. -> clean (Spec evidence already recorded as pass; spot-checks of fixture JSON and doctor.rs show no trailing whitespace.)

Findings:
- [medium/non-blocking] `F1` Rust tool manifest probe errors on malformed JSON where TS reports a diagnostic
  - Location: `crates/runx-runtime/src/doctor.rs:248`
  - Evidence: `discover_tool_diagnostics` calls `serde_json::from_str::<ToolManifestProbe>(&manifest_contents).map_err(|source| RuntimeError::json(...))?;` and bubbles the error up. The TS authority in `packages/cli/src/commands/doctor.ts:144-260` wraps both `parseToolManifestJson` and `validateToolManifest` in a try/catch and emits a `runx.tool.manifest.invalid` diagnostic instead. So a workspace with one syntactically broken `manifest.json` causes `run_doctor` to return `Err(RuntimeError::Json)` instead of producing a `DoctorReport` with the invalid-manifest diagnostic (and any later sibling diagnostics).
  - Impact: Future callers (`runx-cli`, Aster, dev workflows) that invoke `run_doctor` against real workspaces will see hard failures where the TypeScript oracle returns a structured failure report. This contradicts the spec objective of a programmatic API that is interchangeable with the TS oracle, even though no fixture exercises an invalid manifest.
  - Validation: After fix: a fixture with a malformed `tools/<ns>/<name>/manifest.json` should produce a `DoctorReport` whose diagnostics list `runx.tool.manifest.invalid` and which still surfaces sibling diagnostics, matching the TS oracle.
- [medium/non-blocking] `F2` Skill harness coverage uses substring heuristics that diverge from the TS YAML parser
  - Location: `crates/runx-runtime/src/doctor.rs:552`
  - Evidence: `inline_harness_case_count` returns `1` when the file contains both `harness:` and `cases:` substrings, and `discover_skill_diagnostics` gates on `contents.contains("runners:")`. The TS authority (`packages/cli/src/commands/doctor.ts:266-327` together with `validateRunnerManifest`) parses YAML and counts `manifest.harness?.cases.length`, and emits `runx.skill.profile.invalid` when the file fails validation entirely. So a profile with `harness:\n  cases: []` registers as `harness_case_count: 1` in Rust but `0` in TS (Rust suppresses the diagnostic the TS oracle would emit); a profile without `runners:` is silently skipped by Rust where TS would emit `runx.skill.profile.invalid`.
  - Impact: On real workspaces the Rust runtime can hide or fabricate `runx.skill.fixture.missing` diagnostics relative to the TS oracle, weakening the parity guarantee the programmatic API advertises. The included fixtures do not exercise either edge.
  - Validation: After fix: add a fixture whose skill profile is malformed YAML and another whose harness.cases array is empty, and assert Rust matches the TS oracle byte-for-byte for both.
- [low/non-blocking] `F3` empty-success workspace directory is empty and not tracked by git so the fixture generator's --check fails on a clean clone
  - Location: `scripts/generate-rust-doctor-fixtures.ts:146`
  - Evidence: `runDoctorFixture` requires the workspace directory to exist when `--check` is set: `if (!existsSync(workspacePath)) { if (check) { throw new Error(\`fixture workspace is missing: ...\`); } ... }`. The `empty-success` case has zero `files`, so git tracks only `fixtures/doctor/empty-success/expected.json` (confirmed by globbing `fixtures/doctor/**/*` — only the expected.json appears). On a fresh checkout the directory does not exist, so `pnpm exec tsx scripts/generate-rust-doctor-fixtures.ts --check` (the v1 acceptance command) would throw. The Rust runtime test still passes because `run_doctor` returns an empty diagnostic set on a missing root, but the TS oracle acceptance command is locally fragile.
  - Impact: v1's recorded `exit code was 0` reflects a local run after the workspace dir was implicitly created during generation. Anyone re-running v1 on a clean clone will see it fail. The CI verify:fast pipeline does not invoke this script, so CI does not catch the brittleness.
  - Validation: After fix: deleting `fixtures/doctor/empty-success/workspace/` and re-running the `--check` command must still exit 0.
- [low/non-blocking] `F4` Hash-material JSON is hand-mirrored alongside JsonObject builders without an automated sync check
  - Location: `crates/runx-runtime/src/doctor.rs:439`
  - Evidence: Every diagnostic builder constructs the structured `target`/`evidence` as a `JsonObject` and the hash-material as a parallel hand-formatted JSON string (see `DiagnosticParts.target_json`/`evidence_json` and call sites at lines 91-122, 173-208, 280-302, 320-347, 399-429). The two representations must stay byte-equivalent to keep `instance_id` parity with the TS `JSON.stringify` order. Fixture tests catch outright drift, but any code path that constructs a diagnostic outside the fixture set (e.g., future diagnostics or workspace edge cases) silently risks divergent hashes without any compile-time or runtime guard.
  - Impact: Maintenance burden grows linearly with future diagnostics. Combined with the BTreeMap ordering in `serde_json` (previous F4), `serde_json::to_string(&report)` already does not byte-match the TS oracle, even though Value comparison passes; this is fine under the current invariant but easy to regress as soon as anyone consumes the wire bytes.
  - Validation: After fix: a single canonicalizer derives both `JsonObject` and hash material from one declarative struct, and adding a new diagnostic in a unit test cannot produce mismatched representations.

## Self Eval

- Target score: 9.0. Passing means Rust has a trustworthy doctor report core
  without pretending the CLI or deferred diagnostic families are ported.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: native doctor core after runtime skeleton, before CLI cutover

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: passed
Started: 2026-05-19T04:08:22Z
Ended: 2026-05-19T04:13:20Z

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/lib.rs:3
  - Result: passed
  - Evidence: Runtime owns impure filesystem/subprocess boundaries, so future doctor workspace probes belong under `crates/runx-runtime`; the spec now marks Rust doctor files as future files and keeps serializable doctor models in `runx-contracts`.
- command audit
  - Grounded in: code:package.json:33
  - Result: passed
  - Evidence: Acceptance uses existing workspace commands (`pnpm exec vitest`, Cargo workspace manifest, and the existing Rust style guard) plus one future fixture generator declared in scope.
- scope/migration audit
  - Grounded in: code:crates/runx-cli/src/launcher.rs:31
  - Result: passed
  - Evidence: The Cargo launcher currently delegates to npm/Node; the spec keeps CLI routing and renderer cutover out of scope and limits this slice to a programmatic Rust runtime API.
- acceptance timing audit
  - Grounded in: code:packages/cli/src/index.test.ts:1377
  - Result: passed
  - Evidence: Existing TypeScript doctor tests define the oracle behavior first; Rust acceptance runs TS fixture generation before Rust contract/runtime tests.
- rollback/repair audit
  - Grounded in: code:packages/cli/src/commands/doctor.ts:70
  - Result: passed
  - Evidence: TypeScript `--fix` can write repairs, but this Rust slice is read-only and does not change CLI routing, so rollback is removal of Rust modules/fixtures while the TS command keeps working.
- design challenge
  - Grounded in: code:packages/cli/src/commands/doctor.ts:43
  - Result: passed
  - Evidence: The original draft invented environment probes and overclaimed all-check byte parity; the revised design follows the real TS structural/tool/skill/packet diagnostic pipeline and limits Rust work to a low-risk fixture-backed subset.

Issues:
- none
