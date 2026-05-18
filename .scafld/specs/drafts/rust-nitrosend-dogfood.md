---
spec_version: '2.0'
task_id: rust-nitrosend-dogfood
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T00:00:00Z'
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
(`rust-runtime-adapters-agent` recommended).
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

Implication: the cutover dogfood is **preservation**, not adoption. If
nitrosend's existing CI keeps publishing intake comments, routing
targets correctly, and producing equivalent outcome dispatches on the
Rust binary, the dogfood is green.

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
- `/runx issue-intake` slash-command parsing in nitrosend's wrapper
  scripts produces identical CLI invocations against the Rust binary.
- Pinned-SHA build model (TS) gains a pinned-version binary model
  (Rust) for nitrosend CI; nitrosend pins to a specific Rust binary
  release.
- The flow's published comments, intake artifacts, and target dispatches
  are byte-identical (modulo timestamps and IDs) before and after
  cutover for the same fixture inputs.
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
  outputs.
- Document the env-var and CLI contract nitrosend depends on so the
  Rust CLI preserves them precisely.
- Coordinate with `rust-cli-feature-parity-matrix` so the
  nitrosend-specific surface (slash command parsing, policy file
  binding, skills-root layering) is in the matrix.
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

- `rust-runtime-skeleton`.
- `rust-runtime-skill-execution` (which includes `issue-intake` as a
  real-skill anchor; this spec extends it with nitrosend's wrapper
  layer).
- `rust-approval-gate-parity` (issue-intake decision steps can be
  gated).
- `rust-cli-feature-parity-matrix` (slash-command, env-var, and policy
  binding parity).

## Open Questions

- How nitrosend pins a Rust binary release. Likely a checksummed
  download from the binary CDN established by `rust-cli-rust-cutover`,
  preserving the SHA-pinning safety property nitrosend relies on today.
- Whether nitrosend's wrapper scripts (`runx-target-outcome.mjs`,
  `issue-intake.mjs`, `post-issue-intake-comments.mjs`) stay in
  nitrosend or migrate upstream into runx. Default: stay in nitrosend;
  they are nitrosend's adoption shape and a useful reference for other
  adopters.
- Soak duration before the launcher flip. The honest answer is "until
  one full release cycle of nitrosend production issue-intake events
  completes on the Rust binary side-by-side"; calendar time alone is
  too coarse.
