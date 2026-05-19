---
spec_version: '2.0'
task_id: rust-nitrosend-dogfood
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T06:13:09Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust nitrosend dogfood

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. First external-shaped
customer for the Rust runtime; honest version of the "runtime has external
adopters" story.
Blockers: `rust-runtime-skeleton`, at least one impure adapter port
(`rust-runtime-adapters-agent` recommended), and the reusable runx core specs
for operational policy, target-repo runners, and post-merge outcome observer.
Allowed follow-up command: `scafld harden rust-nitrosend-dogfood`
Latest runner update: none
Review gate: not_started

## Summary

Preserve nitrosend's existing production runx deployment unchanged when
`runx` becomes the Rust binary. Nitrosend is the deepest existing
production runx user, not a future adopter. The cutover dogfood is
proving that nitrosend's live issue-intake flow keeps working with no
behavioral regression on the Rust runtime.

The actual nitrosend shape today (verified, not assumed):

- `.github/workflows/issue-intake.yml` clones `runxhq/runx` at a pinned
  SHA into `${RUNNER_TEMP}/runx`, runs `pnpm install --frozen-lockfile`
  and `pnpm build`, then invokes `runx` via
  `RUNX_BIN=${RUNNER_TEMP}/runx/packages/cli/dist/index.js`.
- `RUNX_SKILLS_ROOT` points at `${RUNNER_TEMP}/runx/skills` for upstream
  skills, with nitrosend-custom skills layered from
  `nitrosend/skills/nitrosend/` (`issue-intake`, `onboarding`,
  `segment-from-prose`).
- `RUNX_ISSUE_FLOW_POLICY` is bound to
  `nitrosend/config/runx-issue-flow.json` (versioned `2026-05-15`, 205
  lines, cross-repo target routing with submodule workspaces, per-target
  owners, mutating-vs-non-mutating route hints, outcome mode per
  target).
- Triggered on PR events, issue comments, reviews, and manual dispatch
  with `/runx issue-intake` slash commands.
- Cross-repo: source is `nitrosend/nitrosend`; targets are
  `nitrosend/nitrosend` (workspace), `nitrosend/api` (submodule), and
  `nitrosend/app` (submodule). Outcomes flow back via
  `scripts/runx-target-outcome.mjs` (278 lines) plus its test file.
- 40+ completed scafld specs in `nitrosend/.scafld/specs/archive/2026-05/`
  document months of production hardening around this integration.

The latest production dogfood also changed the target shape: Nitrosend should
be a minimal adopter layer over reusable runx core. Repo-local policy and
scripts are acceptable as reference fixtures during migration, but reusable
routing, runner selection, PR dedupe, source-thread publishing, and final
outcome observation belong upstream.
Current reusable non-execution and metadata surfaces that must not drift during
the Rust rewrite: `runx.operational_policy.v1`, pull-request outbox
`metadata.dedupe`, and feed/outbox
`metadata.source_thread.{required,publish_mode,missing_behavior}`.

Execution artifacts after the harness hard cutover are stricter than those
product surfaces. Nitrosend's `issue-intake`, `issue-to-pr`, `onboarding`, and
`segment-from-prose` names remain recognizable skill and operator-facing names,
but persisted or replayed contract artifacts must be canonical `harness`,
`decision`, `act`, and sealed `harness_receipt` objects only. The dogfood must
not preserve or reintroduce legacy peer contracts such as
`runx.issue_to_pr_outcome.v1`, `outcome`, `effect`, or
`verification_report`.

Implication: the cutover dogfood is **preservation**, not adoption. If
nitrosend's existing CI keeps publishing intake comments, routing
targets correctly, and producing equivalent target dispatches plus sealed
closure/proof records on the Rust binary, the dogfood is green.

## Context

CWD: `.` (workspace root for runx; nitrosend repo is the integration
target)

Packages:
- `crates/runx-runtime`
- `crates/runx-cli`
- runx OSS skills: `oss/skills/issue-intake/`, `oss/skills/issue-to-pr/`,
  `oss/skills/work-plan/`, `oss/skills/research/`, plus any other
  upstream skill the nitrosend flow lands on
- nitrosend repo (read-mostly during this spec): the workflow, config,
  scripts, and custom skills enumerated above

Current TypeScript sources (the things the cutover replaces):
- `oss/packages/cli/dist/index.js` (current `RUNX_BIN` target)
- `oss/packages/runtime-local/**` (current execution path)
- `oss/packages/core/**` (current kernel)
- `oss/packages/adapters/**` (current adapter implementations)

External (nitrosend) files inspected, not modified by this spec:
- `nitrosend/.github/workflows/issue-intake.yml`
- `nitrosend/config/runx-issue-flow.json`
- `nitrosend/scripts/runx-target-outcome.mjs` plus `.test.mjs`
- `nitrosend/skills/nitrosend/{issue-intake,onboarding,segment-from-prose}/`

Files impacted in this spec:
- `fixtures/external/nitrosend/issue-intake/**` (new; deterministic
  snapshot of nitrosend's flow inputs and expected outputs for CI parity)
- `crates/runx-runtime/tests/external/nitrosend_issue_intake.rs` (new;
  runs the fixture against the Rust runtime)
- `scripts/generate-rust-nitrosend-fixtures.ts` (new; TS oracle
  generator; retires when TS sunsets)
- `docs/external-dogfoods.md` (new; documents nitrosend as the cutover
  anchor and lists the env-var contract the cutover preserves)

Invariants:
- `RUNX_BIN`, `RUNX_SKILLS_ROOT`, `RUNX_ISSUE_FLOW_POLICY`, and every
  other env var nitrosend's workflow sets remain semantically identical
  on the Rust binary.
- The hard-cutover execution artifact set is harness/decision/act/receipt
  only. Product copy may say `issue-to-pr`; schema ids, fixture expected
  artifacts, receipt kinds, outbox contract payloads, and persisted replay
  output must not.
- `/runx issue-intake` slash-command parsing in nitrosend's wrapper
  scripts produces identical CLI invocations against the Rust binary.
- Pinned-SHA build model (TS) gains a pinned-version binary model
  (Rust) for nitrosend CI; nitrosend pins to a specific Rust binary
  release.
- The flow's published comments, intake artifacts, and target dispatches
  are byte-identical (modulo timestamps and IDs) before and after
  cutover for the same fixture inputs.
- Target PR completion, deployment verification, source issue closure, and
  source-thread replies are represented as contained act `closure` plus
  receipt-bound verification `proof` on sealed harness receipts. They are not
  modeled as separate outcome/effect/verification-report contracts.
- No nitrosend code change is required for the cutover. Nitrosend may
  *optionally* update its workflow to use a downloaded Rust binary
  instead of `pnpm build`, but the existing pinned-SHA build path keeps
  working until then.
- No live nitrosend production traffic is the primary validation;
  fixture parity in CI is. Production observation is the confirmation
  step, not the test.

## Objectives

- Capture a deterministic fixture suite from nitrosend's current
  production issue-intake flow.
- Run the fixture suite against `runx-runtime` and assert byte-identical
  canonical harness receipts plus output projections.
- Document the env-var and CLI contract nitrosend depends on so the
  Rust CLI preserves them precisely.
- Coordinate with `rust-cli-feature-parity-matrix` so the
  nitrosend-specific surface (slash command parsing, policy file
  binding, skills-root layering) is in the matrix.
- Validate that Nitrosend can express its API/App/workspace routing through
  `runx-operational-policy-config` without a custom parser.
- Validate that `runx policy lint` accepts the Nitrosend-like fixture and that
  `runx policy inspect` redacts raw provider locators.
- Validate that Nitrosend cross-repo issues use `runx-target-repo-runners` and
  preserve Slack/GitHub source-thread metadata through PR creation.
- Validate that target completion and post-merge verification use
  `runx-post-merge-outcome-observer` only after that observer emits canonical
  harness receipt closure/proof artifacts, not a Nitrosend-only observer path
  and not a legacy outcome contract.
- After parity is green, soak the Rust binary in nitrosend production
  via a side-by-side run before the launcher cutover.

## Scope

In scope:
- Fixture suite for nitrosend's issue-intake flow.
- Rust integration test against the fixture.
- Documentation of nitrosend's runx contract surface.
- Coordination with the CLI parity matrix.

Out of scope:
- Changing nitrosend's workflow (preservation is the point).
- Moving nitrosend off the pinned-SHA build before the launcher cutover.
- Migrating nitrosend's custom skills to a different shape.
- Onboarding new nitrosend flows; this spec is scoped to issue-intake.

## Dependencies

- `runx-contract-spine-hard-cutover` as completed source of truth for
  canonical harness, decision, act, signal, authority, reference, proof, and
  harness receipt shapes. This spec consumes the no-compat cutover; it does not
  accept retired artifact names for compatibility.
- `rust-runtime-skeleton`.
- `rust-runtime-skill-execution` (which includes `issue-intake` as a
  real-skill anchor; this spec extends it with nitrosend's wrapper
  layer).
- `rust-approval-gate-parity` (issue-intake decision steps can be
  gated).
- `rust-cli-feature-parity-matrix` (slash-command, env-var, and policy
  binding parity).
- `runx-operational-policy-config` for policy-backed routing and ownership.
- `runx-target-repo-runners` for source-to-target PR creation.
- `runx-post-merge-outcome-observer` only after it is aligned to sealed
  harness receipt closure/proof semantics for final merge, deploy, and
  source-thread publication.
- `rust-receipt-proof-verification`, `rust-receipt-tree-resolution`, and
  `rust-runtime-receipt-path-discovery` before receipts are accepted as final
  evidence.

Sequencing:

- `runx-contract-spine-hard-cutover` lands first and removes retired execution
  artifact vocabulary from active contracts, schemas, fixtures, and hosted
  persistence.
- `rust-harness`, `rust-receipt-proof-verification`,
  `rust-receipt-tree-resolution`, and `rust-runtime-receipt-path-discovery`
  must be available before this spec can claim receipt parity. The nitrosend
  fixture compares sealed harness receipts and proof verification, not a
  best-effort JSON projection.
- `runx-operational-policy-config` lands before fixture capture so the
  Nitrosend routing policy is represented by reusable policy config and not by
  a fixture-only parser.
- `runx-target-repo-runners` lands before target dispatch parity. Dispatch
  fixtures must show target PR creation through the reusable runner path and
  preserve source-thread metadata into harness artifacts and outbox metadata.
- `runx-post-merge-outcome-observer` lands after its no-compat shape is
  hardened. Its target-completion proof must close or fail a contained act on a
  sealed harness receipt; it must not emit `runx.issue_to_pr_outcome.v1`,
  `effect`, or `verification_report` as peer artifacts.
- Nitrosend side-by-side production soak starts only after fixture parity,
  policy lint, target-runner dispatch, and post-merge closure/proof gates are
  green in CI.

## No-Compat Contract Boundary

Allowed product surface:

- Skill directory names, marketplace copy, slash-command text, operator logs,
  source comments, and docs may continue to say `issue-intake`,
  `issue-to-pr`, `target outcome`, or Nitrosend-specific flow names when that
  is the human-facing product language.

Required artifact surface:

- Fixture expected outputs use canonical harness receipts with contained
  decisions and contained acts.
- A target PR creation is an act inside a harness. Dedupe, source-thread, and
  target refs are artifact refs or typed metadata attached to that harness path.
- A target completion, failure, deployment verification, source issue close,
  or source-thread final reply is represented by contained act `closure` and
  receipt-bound verification proof on a sealed harness receipt.
- Proof-bearing act refs resolve as `harness_receipt_ref` plus contained act
  id. No standalone act receipt, effect receipt, verification-report receipt,
  or issue-to-pr outcome receipt is valid.

Forbidden after cutover:

- Active contract ids or fixture artifact kinds named
  `runx.issue_to_pr_outcome.v1`, `issue_to_pr_outcome`, `outcome`, `effect`,
  `verification_report`, `verification-report`, `target_outcome`, or
  `target-effect`.
- Compatibility readers or writers that translate retired outcome/effect
  artifacts into harness receipts at runtime.
- Acceptance that relies on old TS local receipt shapes, retired fixture fields,
  or a parallel Nitrosend-only observer result.

## Acceptance Criteria

- [ ] Nitrosend fixture capture includes at least one workspace target, one API
  submodule target, one App submodule target, one duplicate issue/PR dedupe
  case, one missing-source-thread rejection, one merged-and-verified target
  completion, and one failed-verification target completion.
- [ ] Every fixture expected execution artifact is a canonical harness receipt
  tree: receipt schema/id, harness id, contained decision ids, contained act ids,
  child receipt refs, seal status, proof status, and verification refs. No
  fixture expectation asserts legacy skill/graph/outcome receipt kind fields.
- [ ] `issue-intake` and `issue-to-pr` remain visible in SKILL.md/product copy
  and source-thread comments, but contract schemas, fixture artifact kinds,
  persisted receipts, outbox payload contract ids, and Rust structs do not use
  those names as execution contract identifiers.
- [ ] The Rust runtime rejects any Nitrosend replay fixture containing retired
  execution artifact fields with a stable diagnostic that includes fixture path
  and field path.
- [ ] Target PR creation uses `runx-target-repo-runners`; dedupe state is
  recorded in pull-request outbox `metadata.dedupe` and in the sealed harness
  receipt proof path, not in a Nitrosend-only branch ledger.
- [ ] Target completion and verification use
  `runx-post-merge-outcome-observer` only through closure/proof on sealed
  harness receipts. The observer does not emit
  `runx.issue_to_pr_outcome.v1`, `effect`, or `verification_report`.
- [ ] `runx policy lint` accepts the Nitrosend-like fixture and
  `runx policy inspect` redacts raw provider locators while preserving
  target-repo, owner, source-thread, and publish-mode decisions.
- [ ] Replaying the same Nitrosend fixture twice produces identical canonical
  receipt bytes after normalized fixture ids/timestamps and passes
  `runx-receipts` proof verification.
- [ ] Side-by-side production soak publishes at most one user-visible final
  source-thread update per source issue. Any Rust-side dry-run proof mismatch
  blocks the launcher flip.

## Validation Commands

From this repo (`oss/`):

```sh
pnpm contracts:schemas:check
pnpm fixtures:contracts:check
pnpm fixtures:contracts:keys
pnpm exec tsx scripts/generate-rust-nitrosend-fixtures.ts --check
cargo build --manifest-path crates/Cargo.toml -p runx-cli
cargo test --manifest-path crates/Cargo.toml -p runx-contracts
cargo test --manifest-path crates/Cargo.toml -p runx-receipts
cargo test --manifest-path crates/Cargo.toml -p runx-runtime nitrosend_issue_intake
cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets -- -D warnings
cargo fmt --manifest-path crates/Cargo.toml --all --check
node scripts/check-rust-core-style.mjs
! rg -n "runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:" fixtures/external/nitrosend crates/runx-runtime/tests/external/nitrosend_issue_intake.rs docs/external-dogfoods.md
```

From the runx workspace root (`..` from this repo), with Nitrosend checked out
under `RUNX_CONSUMER_REPOS_ROOT/nitrosend`:

```sh
export RUNX_CONSUMER_REPOS_ROOT="${RUNX_CONSUMER_REPOS_ROOT:-/Users/kam/dev}"
export RUNX_CUTOVER_OSS_REF="${RUNX_CUTOVER_OSS_REF:-$(git -C oss rev-parse HEAD)}"
test -d "$RUNX_CONSUMER_REPOS_ROOT/nitrosend"
RUNX_CUTOVER_EXTRA_ROOTS="$RUNX_CONSUMER_REPOS_ROOT/nitrosend" pnpm cutover:check
node scripts/check-contract-cutover-fixtures.mjs
```

From the Nitrosend repo:

```sh
export RUNX_OSS_REPO="${RUNX_OSS_REPO:-/Users/kam/dev/runx/runx/oss}"
test -d "$RUNX_OSS_REPO"
RUNX_BIN="$RUNX_OSS_REPO/crates/target/debug/runx" \
RUNX_SKILLS_ROOT="$RUNX_OSS_REPO/skills" \
RUNX_ISSUE_FLOW_POLICY="$PWD/config/runx-issue-flow.json" \
node --test scripts/issue-intake.test.mjs scripts/runx-target-outcome.test.mjs
! rg -n "runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:" .github config scripts skills .scafld
```

## Rollback And Repair

- Before the Nitrosend launcher flip, rollback is leaving the existing
  pinned-SHA TypeScript workflow in place and disabling the Rust side-by-side
  lane. No Nitrosend production workflow edit is required to recover.
- After a Rust binary pin is introduced, rollback is reverting only that
  Nitrosend pin and returning `RUNX_BIN` to the known-good pinned TS build or
  previous Rust binary. Do not add a compatibility translator for retired
  outcome/effect/verification-report artifacts.
- If a fixture was captured with retired artifact fields, repair the producer
  and regenerate the fixture/oracle as canonical harness receipts. Do not
  whitelist the retired fields in the Rust fixture loader.
- If product-surface scans accidentally flag `issue-to-pr` in SKILL.md,
  comments, or docs, repair the scanner allow-list for product surfaces while
  keeping contract artifact scans strict.
- If source-thread publishing duplicates comments during soak, disable only
  the Rust publisher and keep dry-run receipt proof generation enabled until
  idempotency and dedupe proof match.
- If proof comparison is flaky, repair canonicalization, timestamp/id
  normalization, or receipt tree resolution. Do not relax validation to
  structural serde equality.
- If target completion semantics regress, keep the repo-local Nitrosend
  observer script as the production path until the reusable observer emits
  sealed harness receipt closure/proof correctly, then remove the duplicated
  path. No legacy observer contract remains after cutover.

## Open Questions

- How nitrosend pins a Rust binary release. Likely a checksummed
  download from the binary CDN established by `rust-cli-rust-cutover`,
  preserving the SHA-pinning safety property nitrosend relies on today.
- Whether nitrosend's wrapper scripts (`runx-target-outcome.mjs`,
  `issue-intake.mjs`, `post-issue-intake-comments.mjs`) stay in
  nitrosend or migrate upstream into runx. Updated default: reusable behavior
  migrates upstream; Nitrosend keeps only product policy, wiring, and fixtures.
- Soak duration before the launcher flip. The honest answer is "until
  one full release cycle of nitrosend production issue-intake events
  completes on the Rust binary side-by-side"; calendar time alone is
  too coarse.
