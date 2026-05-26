---
spec_version: '2.0'
task_id: runx-oss-trust-boundary-cleanup-v1
created: '2026-05-26T00:00:00Z'
updated: '2026-05-26T11:48:19Z'
status: draft
harden_status: needs_revision
size: large
risk_level: high
---

# runx OSS trust-boundary cleanup v1

## Current State

Status: draft
Current phase: implementation in progress
Next: run queued Rust checks after the active external cargo check releases
the artifact lock; continue phase 2/3/4 tactical validation.
Reason: harden round 2 found stale evidence after the first build slice; this
revision separates retired findings from live blockers.
Blockers: none for the first build slice; remaining blockers are the live
phase items below.
Allowed follow-up command: `continue implementation from the live phase items`
Latest runner update: 2026-05-26T12:40:00Z production signing fallback
removed from governed env parsing, implicit RuntimeOptions::default() deleted,
duplicate packaged shell.exec tree deleted, and surviving shell.exec hardened
with cwd containment, timeout, output caps, and process-group termination.
Review gate: not_started

## Summary

This spec turns the 2026-05-26 deep OSS audit into an executable cleanup plan.
The cleanup goal is not superficial renaming. The end state is a codebase where
trust boundaries are explicit, runtime fallback paths fail closed, credentials
stay opaque, generated artifacts are clearly generated, duplicated helper logic
is collapsed behind one owner, and files touched by the cleanup have one clear
responsibility.

This is a hard cutover spec. It must not add compatibility aliases, legacy
bridges, old vocabulary shims, or mixed old/new code paths. When an old surface
is unsafe or no longer shape-aligned, delete it or replace it with the new
governed surface.

## False-Positive Discipline

Every phase starts with an evidence refresh. Do not edit from stale audit
memory. If a finding no longer reproduces, mark that item "retired by prior
work" in this spec with the command that proved it, then move on. Do not create
cleanup churn for an issue that is already gone.

Required per-item evidence:

- A current file/line or command output showing the issue still exists.
- A named owner boundary: contract, runtime, CLI, adapter, docs, generated
  artifact, or test helper.
- A deletion/refactor target that removes the mixed responsibility instead of
  hiding it behind another wrapper.
- A local test or grep gate that fails before the fix and passes after it.

## Design Rules

- No trust fallback by default. Local-development paths must be explicit and
  impossible to enter accidentally from production or packaged paths.
- No cwd or untrusted PATH authority. A governed runtime must not discover
  executable trust anchors from caller-controlled directories unless an explicit
  dev/test flag says so.
- No unbounded process output. Any child process path that captures output must
  enforce byte limits and timeouts, and must terminate the whole process tree.
- Canonicalize before containment checks. A path is only considered inside a
  workspace after symlinks and existing parents have been resolved.
- One implementation per behavior. Duplicated tools/helpers must be collapsed
  into a package import, shared helper, generated copy, or deleted surface.
- Generated and authored code do not mix. Generated files need clear headers;
  hand-authored schema builders must not survive the Rust-source-of-truth
  contract flip unless a surviving boundary is documented.
- Authority attenuation must be computed, not asserted. A proof record may cite
  the comparison, but the runtime/verifier must recompute the subset relation.
- No compatibility language in public docs or package descriptions after the
  clean cutover.

## Evidence From Current Audit

Commands run on 2026-05-26 before this spec was written:

```sh
nl -ba packages/cli/src/native-runx.ts | sed -n '45,145p'
nl -ba packages/cli/bin/runx | sed -n '65,95p'
nl -ba crates/runx-runtime/src/receipts/signing.rs | sed -n '85,115p'
nl -ba crates/runx-runtime/src/execution/runner.rs | sed -n '50,85p'
nl -ba crates/runx-runtime/src/sandbox.rs | sed -n '250,290p'
nl -ba crates/runx-runtime/src/sandbox.rs | sed -n '490,560p'
nl -ba crates/runx-runtime/src/sandbox.rs | sed -n '780,795p'
nl -ba tools/shell/exec/src/index.ts | sed -n '1,70p'
test ! -e packages/cli/tools
nl -ba crates/runx-runtime/src/credentials.rs | sed -n '385,430p'
nl -ba crates/runx-cli/src/skill.rs | sed -n '110,132p'
nl -ba crates/runx-runtime/src/adapters/mcp/transport.rs | sed -n '228,260p'
nl -ba packages/runtime-local/src/mcp/index.ts | sed -n '455,474p'
nl -ba packages/contracts/src/internal.ts | sed -n '170,210p'
nl -ba packages/contracts/src/schemas/spine.ts | sed -n '1,24p'
nl -ba packages/contracts/src/schemas/spine.ts | sed -n '382,462p'
nl -ba scripts/generate-rust-contract-fixtures.ts | sed -n '900,948p'
nl -ba packages/create-skill/bin/create-skill.js | sed -n '18,38p'
nl -ba docs/api-surface.md | sed -n '1,18p'
nl -ba docs/how-we-test.md | sed -n '10,18p'
rg -n "harness_receipt|harness receipt|act-receipt|act_receipt|runx\\.harness_receipt" \
  crates packages schemas docs scripts --glob '!**/target/**' --glob '!**/dist/**' --glob '!node_modules/**'
```

Retired by the first build slice:

- Native launcher cwd discovery, `RUNX_SKIP_NATIVE_VERIFY`, the size/mtime
  verification cache, and unbounded native stdout/stderr capture are removed.
  Evidence: `pnpm exec vitest run tests/rust-cli-cutover-scripts.test.ts
  packages/cli/src/native-runx.test.ts` passed, and
  `rg -n "RUNX_SKIP_NATIVE_VERIFY|process\.cwd\(\).*crates.*target|native-verify-"
  packages/cli/src packages/cli/bin` has no hit.
- `packages/create-skill/bin/create-skill.js` no longer falls back to `tsx` or
  references a personal workspace path. Evidence:
  `rg -n "tsx|/home/kam|source fallback" packages/create-skill/bin
  packages/create-skill/src packages/create-skill/package.json` has no hit.
- `crates/runx-cli/src/skill.rs` rejects `--secret-env ENV=VALUE` and reads
  `--secret-env ENV` from the process environment. Evidence:
  `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test
  local_credential` passed.
- Generated contract artifacts now carry a generated-file header and immediate
  docs wording cleanup landed. Evidence: `pnpm docs:api`,
  `pnpm contracts:schemas:check`, and `pnpm typecheck` passed.
- Env-configured production signing now requires
  `RUNX_RECEIPT_SIGN_ISSUER_TYPE` and rejects `local`/`verifier` issuer types.
  Evidence: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime
  --test receipt_signing` passed.
- Governed runtime signing env parsing no longer falls back to local
  pseudo-signing when signing env is absent, and `RuntimeOptions::default()`
  has been removed so local development signing is explicit. Evidence queued:
  targeted Rust checks after the active external cargo check releases the
  artifact lock; `pnpm typecheck` already passes after the API update.
- `packages/cli/tools` was a generated source mirror with no live in-repo
  consumer and is not shipped by `@runxhq/cli`; the duplicate tree has been
  deleted, `scripts/build-workspace.mjs` no longer recreates it, and Rust tool
  root inference now uses the canonical root `tools` tree. The surviving
  `tools/shell/exec` now bounds output, requires cwd containment under
  `repo_root`/`RUNX_CWD`, applies a default timeout, and terminates the process
  group on Unix. Evidence: the focused shell.exec cases in
  `pnpm exec vitest run tests/tool-step.test.ts packages/cli/src/native-runx.test.ts`
  passed; the broader tool-step file remains blocked by missing
  `RUNX_PARSER_EVAL_BIN`/native parser eval setup in unrelated tests.

Live issues:

- `RUNX_RUST_CLI_BIN` still appears in non-launcher test/MCP/parser helper
  surfaces. The packaged native launcher no longer consumes it; later phases
  must judge the remaining surfaces by their own boundary, not by Phase 1.
- Targeted Rust verification for the signing, sandbox, and credential slices is
  pending while another cargo process owns the artifact lock.
- Sandbox fail-closed behavior, fixed enforcer lookup, canonical containment,
  hashed credential observations, and the duplicate shell.exec deletion have
  landed. Their targeted Rust checks are still pending while another cargo
  process owns the artifact lock.
- `crates/runx-runtime/src/adapters/mcp/transport.rs:235-253` and
  `packages/runtime-local/src/mcp/index.ts:462-471` kill only the direct child.
- `packages/contracts/src/internal.ts:178-210` still exposes a hand-authored
  schema-builder facade after the Rust contract pipeline inversion.
- `packages/contracts/src/schemas/spine.ts:1-24` still imports and uses that
  facade for hand-authored spine schemas.
- `packages/contracts/src/schemas/spine.ts:386-458` represents authority bounds
  as strings/globs and lets subset proof carry an asserted `result: "subset"`.
- `scripts/generate-rust-contract-fixtures.ts:908-947` still has a local
  `stableJson` with `localeCompare` and ASCII/key-order restrictions.
- `docs/how-we-test.md:14-15` still uses "surviving TypeScript package
  boundaries" wording that needs a final public-doc pass once the TypeScript
  boundary cleanup is complete.
- Active source grep found no active `runx.harness_receipt.v1` usage; the
  current adjacent artifact is `act-receipt.schema.json`. Do not treat
  `runx.harness_receipt.v1` as an active cleanup target unless a fresh grep
  proves otherwise.

## Scope

In scope:

- Native launcher trust and binary resolution.
- Receipt signing defaults and production issuer semantics.
- Sandbox enforcement, enforcer discovery, path containment, and temporary
  secret/input cleanup.
- Process execution lifecycle for Rust and surviving TS subprocess paths.
- Shell tool escape hatch ownership and duplicate tool tree collapse.
- Credential reference opacity and secret-delivery CLI shape.
- Remaining TS contract authoring surfaces that conflict with Rust-generated
  contracts.
- Authority-subset verification mechanics where the current shape allows
  asserted proof.
- Canonical JSON fixture generation and generated documentation wording.
- Local decomposition of files directly touched by these fixes when mixed
  responsibility blocks a clean implementation.

Out of scope:

- Broad large-file decomposition. `monolith-decomposition-v1` owns that.
- `runx-rust-95-release-readiness`; another agent owns it.
- Runtime-local/adapters package deletion where `rust-ts-sunset-runtime-local`
  is the owner. This spec may harden surviving paths or delete duplicated unsafe
  tools, but must not spend effort beautifying code scheduled for deletion.
- Wire-shape compatibility. This is a clean cutover; do not keep legacy aliases.
- Renaming `act-receipt` unless a separate current-shape ruling says the
  adapter boundary contract is semantically wrong. This spec may document the
  naming tension but must not create a drive-by contract rename.

## Dependencies

- `monolith-decomposition-v1` for broad file-size/style cleanup.
  Its sandbox/credentials/runner decomposition batches must wait until this
  spec's trust-boundary edits land, otherwise both specs churn the same files.
- `rust-ts-sunset-runtime-local` for deletion of runtime-local/adapters
  compatibility surfaces.
- Archived `rust-contract-pipeline-inversion` for Rust-generated schemas; this
  spec owns cleaning up surviving TypeScript schema-authoring residue.
- Active `runx-rust-95-release-readiness` as a dependency signal only. Do not
  edit that spec here.

## Sequencing Notes From Harden Round 1

- Phase 1 launcher verification is pinned to unconditional hashing on every
  packaged launch. There is no opt-out env and no replacement cache in this
  spec. If later benchmark evidence justifies a cache, it must be a separate
  spec with a digest-derived trust model.
- The launcher loses all dependency on `RUNX_RUST_REGISTRY_BIN`. The current
  registry resolver may keep its separate use of `RUNX_RUST_REGISTRY_BIN` until
  `rust-ts-sunset-runtime-local` deletes or replaces that boundary. After Phase
  1, no launcher path reads it.
- The only non-enforcing sandbox surface is the explicit
  `unrestricted-local-dev` profile with approved escalation. Non-dev profiles
  fail closed by default; `RUNX_SANDBOX_REQUIRE_ENFORCEMENT` is removed as a
  policy signal because required enforcement is no longer opt-in.
- Phase 3 and Phase 5 overlap with `runx-rust-95-release-readiness` Phase 3.
  Before those phases edit `sandbox.rs`, `credentials.rs`, or runner signing
  files, Phase 0 must re-check the active spec/worktree and record whether the
  other agent already landed that slice. If it did, this spec rebases onto that
  state rather than duplicating the work.
- Phase 4 must first prove whether `packages/cli/tools` is dead. If it has no
  in-repo consumer and is not shipped through package exports, the action is
  deletion; only the canonical `tools` implementation is hardened and compiled
  dist assets are emitted at build time. If a hidden consumer appears, stop and
  split a smaller spec.
- Phase 5 must grep active scripts/tests and known consumer repos for
  `--secret-env ENV=VALUE` before deleting the argv-secret form. All active
  call sites must be rewritten in the same change.

## Phase 0: Evidence Refresh And Ownership Map

Status: pending
Dependencies: none

Objective: prove every item is still current immediately before editing.

Changes:

- Re-run the evidence commands in this spec.
- Add a short "Executed Evidence Refresh" section with command summaries and
  any retired findings.
- Mark any vanished finding as retired rather than editing around it.
- Assign each live finding to exactly one owner layer before code changes:
  CLI launcher, Rust runtime, Rust receipts, Rust contracts, TS contracts,
  tool distribution, docs, or tests.

Acceptance:

- [ ] Every live finding has current evidence and a layer owner.
- [ ] Every non-reproducing finding is marked retired with command evidence.
- [ ] No phase below starts from stale line numbers or stale assumptions.

## Phase 1: Native Launcher Trust Boundary

Status: partial
Dependencies: phase 0

Objective: make packaged native execution trust explicit, bounded, and
non-hijackable.

Changes:

- Remove cwd-based discovery from `packages/cli/src/native-runx.ts`.
- Remove `RUNX_RUST_REGISTRY_BIN` as a general runx binary override unless it
  is still a current, documented dev-only test hook; if kept for tests, it must
  be rejected by packaged/default launcher paths.
- Remove `RUNX_SKIP_NATIVE_VERIFY` and the size/mtime verification cache.
  Always hash the selected packaged native binary against `checksums.json`
  before execution.
- Add stdout/stderr byte limits and timeout defaults to native process capture.
- Keep development overrides explicit and named as development/test behavior;
  do not leave hidden production fallbacks.
- Remove `RUNX_RUST_CLI_BIN` from the packaged native launcher. If a dev binary
  override remains for launcher tests, give it a development-only name and
  require an absolute path.

Acceptance:

- [x] `rg -n "RUNX_SKIP_NATIVE_VERIFY|process\\.cwd\\(\\).*crates.*target|RUNX_RUST_REGISTRY_BIN" packages/cli/src packages/cli/bin` returns no packaged launcher bypass.
- [x] Native process stdout/stderr are capped and tested.
- [x] A test proves cwd `crates/target/debug/runx` cannot hijack the wrapper.
- [x] `RUNX_RUST_CLI_BIN` cannot act as an arbitrary packaged/default
  production binary override.
- [ ] `pnpm test:fast -- packages/cli` or the closest current CLI launcher
  test target passes.

## Phase 2: Production Receipt Signing

Status: partial
Dependencies: phase 0

Objective: prevent production runtime paths from silently pseudo-signing
receipts.

Changes:

- Split local-development signing from production signing in API shape.
- Make production runtime construction fail if signing material is required but
  absent.
- Ensure env-configured production signing emits a production/platform issuer
  type, not `Local`.
- Replace default `RuntimeOptions::default()` and store helpers that silently
  choose local signing in governed runtime paths with explicit constructors.
- Add negative tests for missing signing env in production mode and incomplete
  signing env.
- Implemented: `RuntimeReceiptSignatureConfig::from_env` now rejects absent
  signing env, `RuntimeOptions::default()` is deleted, and test/helper call
  sites use explicit `RuntimeOptions::local_development()` when they mean local
  development.

Acceptance:

- [x] No production/governed runtime path can enter local signing by absence of
  env alone.
- [x] Env-configured signing requires a non-local issuer type and rejects
  `local`/`verifier`.
- [x] Tests cover local development signing, production signing, missing env,
  incomplete env, and issuer type across governed runtime constructors.
- [ ] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test receipt_signing`
  passes.
- [ ] `cargo test --manifest-path crates/Cargo.toml -p runx-receipts` passes.

## Phase 3: Sandbox Enforcement And Path Safety

Status: partial
Dependencies: phase 0

Objective: make sandbox policy enforceable, non-spoofable, and symlink-safe.

Changes:

- Fail closed for governed sandbox profiles unless enforcement is explicit
  local development behavior.
- Treat `unrestricted-local-dev` with approved escalation as the only
  non-enforcing local-development sandbox surface. Do not add a second env var
  or compatibility override.
- Stop discovering sandbox enforcers from caller-controlled `PATH` in governed
  paths. Use fixed system locations or an explicit absolute configuration with
  tests.
- Validate writable/readable paths using canonicalized existing paths and
  canonicalized existing parents before bind-mount planning.
- Add a symlink-escape negative fixture for writable path planning.
- Ensure temporary input/secret files are cleaned up after process exit on
  success, failure, and timeout.

Acceptance:

- [x] Non-dev sandbox profiles fail when no enforcing runtime exists.
- [x] `bwrap`/`sandbox-exec` lookup no longer reads caller `PATH`.
- [ ] Symlink under workspace pointing outside workspace is rejected for
  writable mounts.
- [ ] Temp input files are removed after success, failure, and timeout.
- [ ] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime sandbox`
  passes.

## Phase 4: Process Lifecycle And Shell Escape Collapse

Status: partial
Dependencies: phase 0

Objective: one bounded process-execution model for surviving subprocess paths.

Changes:

- Introduce or reuse a single process-control helper for timeout, output cap,
  and process-tree termination.
- Apply it to Rust MCP, Rust CLI/external adapter paths where needed, native
  wrapper capture, and any surviving TS subprocess path.
- Delete or replace duplicated `shell.exec` implementations. If `shell.exec`
  survives, it must route through the governed execution surface and carry
  timeout, cwd containment, output cap, and redaction rules.
- Collapse duplicated tool tree copies by package import, generation, or
  deletion. Do not keep two hand-maintained copies.
- Specifically verify `packages/cli/tools` consumers. If none exist, delete
  that duplicate tree and harden only `tools`.
- Implemented: grep found no live `packages/cli/tools` consumer outside tests
  and specs, and `@runxhq/cli` package `files` does not ship tools; the
  duplicate tree was deleted, the build no longer recreates it, and Rust
  inferred tool roots now use only root `tools`. The surviving root
  `shell.exec` tool now has bounded capture, cwd containment, timeout, and Unix
  process-group termination.

Acceptance:

- [x] No surviving shell.exec subprocess capture path has unbounded
  stdout/stderr.
- [x] Surviving shell.exec cannot hang forever without an explicit
  documented long-running harness mode.
- [ ] MCP process-tree termination is tested.
- [x] `tools` and `packages/cli/tools` are no longer two independent source
  trees.
- [ ] `pnpm test:fast` and relevant Rust adapter tests pass.

## Phase 5: Credential Reference Opacity

Status: partial
Dependencies: phase 0

Objective: remove secret-bearing and provider-local material names from public
or semi-public receipt/reference surfaces.

Changes:

- Remove `--secret-env ENV=VALUE`; replace with a non-argv form such as
  `--secret-env ENV` reading from the process environment, or a governed
  credential reference path. Do not keep the old form as an alias.
- Ensure `material_ref` is never embedded raw into observation IDs, request IDs,
  credential URIs, receipt references, or debug output.
- Keep only provider-opaque credential references and hashes in receipts.
- Add sanitizer coverage in `crates/runx-runtime/tests/credential_opacity.rs`
  or the closest existing credential/skill-run test target; the test must
  serialize the produced receipt/log payload and assert the raw provider
  material reference is absent while the opaque credential reference/hash
  remains.

Acceptance:

- [x] Local credential observations use hash-based IDs and credential refs;
  material-ref errors report hashes rather than raw refs.
- [ ] `rg -n "secret-env.*=|ENV=VALUE|material_ref\\}" crates/runx-cli crates/runx-runtime` finds no raw secret/opaque-ref leakage in user-facing paths.
- [x] Negative tests prove old `--secret-env ENV=VALUE` is rejected.
- [x] Public observation serialization tests prove raw material refs do not
  appear.

## Phase 6: Contract Authoring Boundary Cleanup

Status: partial
Dependencies: phase 0

Objective: finish the clean Rust-source-of-truth contract cutover.

Changes:

- Remove surviving hand-authored schema-builder paths that can still define
  contract shape in TypeScript.
- Keep TypeScript contract package behavior as generated artifacts plus
  validator/type exports only.
- Add generated-file headers to generated schema artifacts if absent.
- Ensure any surviving TS schema file is a generated or thin import/export
  boundary, not a parallel source of truth.
- Update package descriptions and docs generation inputs so public docs no
  longer describe compatibility wrappers after cutover.

Acceptance:

- [ ] `rg -n "export const Type|from \"\\.\\./internal\\.js\".*Type|Type\\." packages/contracts/src/schemas packages/contracts/src/internal.ts` returns no hand-authored contract-shape authority, after subtracting any explicitly documented generated/thin-export allowlist.
- [x] Generated artifacts carry a generated-file header.
- [x] `pnpm contracts:schemas:check` passes.
- [ ] `pnpm fixtures:contracts:check` passes.
- [x] `pnpm docs:api` regenerates docs after the immediate package-description
  cleanup.
- [ ] Public docs have a final no-compatibility wording sweep after the
  TypeScript schema facade cleanup.

## Phase 7: Authority Algebra Verification

Status: pending
Dependencies: phase 0

Objective: make authority attenuation a computed invariant, not a narrative
assertion.

Changes:

- Define the authority partial order in Rust core for resource family, verbs,
  bounds, conditions, approvals, credentials, time, fanout/depth/runtime, and
  payment authority where applicable.
- Recompute child-subset-of-parent in runtime/receipt verification instead of
  trusting a serialized `result: "subset"`.
- Preserve the proof record as evidence of the comparison, but make verification
  fail if the recomputed relation disagrees with the record.
- Add negative fixtures for every meaningful dimension: wider verbs, broader
  path glob, broader network destination, later expiry, higher spend, missing
  approval, deeper fanout, and credential escalation.

Acceptance:

- [ ] A forged subset proof with broader child authority fails verification.
- [ ] Existing valid receipt tree fixtures still pass.
- [ ] `cargo test --manifest-path crates/Cargo.toml -p runx-core -p runx-receipts authority`
  passes.

## Phase 8: Canonicalization, Fixtures, And Docs

Status: pending
Dependencies: phase 0

Objective: remove stale local canonicalization and compatibility wording.

Changes:

- Replace fixture-generator local `stableJson`/`localeCompare` logic with the
  canonical JSON implementation or the Rust emitter path.
- Remove stale ASCII/key-order "before cutover" restrictions once canonical
  codepoint ordering is the single implementation.
- Regenerate docs that currently publish compatibility language.
- Add a fixture with non-ASCII object keys if canonical JSON supports it; if
  not, document the actual canonical contract and make the restriction live in
  the canonical layer, not the generator.

Acceptance:

- [ ] `rg -n "localeCompare|hash-stable-codepoint-cutover|compatibility coverage|Sunset TypeScript compatibility" scripts packages docs` has no stale generator/docs hits except archived specs.
- [ ] Canonical JSON tests cover the ordering behavior used by fixtures.
- [ ] Fixture generation check passes.

## Phase 9: Local Boundary Decomposition

Status: pending
Dependencies: phases 1-8 as applicable

Objective: avoid leaving mixed-responsibility files behind in files touched by
this cleanup.

Changes:

- Split only where the cleanup itself exposes mixed responsibilities. Examples:
  sandbox runtime discovery vs path validation vs temp-file lifecycle; receipt
  signing policy vs local development policy; process execution vs adapter
  protocol logic; credential reference construction vs provider binding.
- Do not decompose broad untouched god-files here. Those stay with
  `monolith-decomposition-v1`.
- Remove stale large-file/style waivers in files this spec shrinks below the
  threshold.

Acceptance:

- [ ] Every new helper has one owner and one reason to exist.
- [ ] No new wrapper module exists solely to preserve old names or old paths.
- [ ] `node scripts/check-rust-core-style.mjs` has no new findings caused by
  this spec.

## Final Validation

Run the smallest complete gates needed for touched surfaces, then the standard
fast suite. Do not run heavy gates concurrently.

Required final commands:

```sh
git diff --check
pnpm typecheck
pnpm test:fast
pnpm boundary:check
cargo fmt --manifest-path crates/Cargo.toml --all -- --check
cargo test --manifest-path crates/Cargo.toml -p runx-contracts -p runx-core -p runx-receipts -p runx-runtime
node scripts/check-rust-core-style.mjs
scafld validate runx-oss-trust-boundary-cleanup-v1
```

Add narrower package-specific test commands inside each phase's evidence
section as the implementation discovers exact touched surfaces.

## Review Gate

The review gate must reject the implementation if any of these remain:

- A hidden production fallback to local signing, declared-only sandboxing,
  unchecked native binary execution, or unbounded process output.
- A duplicated implementation where both copies remain hand-maintained.
- A public or receipt surface that emits raw credential material refs.
- A TS hand-authored schema path that can still change canonical contract shape.
- Compatibility/legacy wording in active package docs.
- A fix that only renames an overloaded concept while leaving mixed
  responsibility intact.

## Rollback

Rollback is per phase. Because this is a hard cutover cleanup, rollback should
revert the phase's edits rather than restoring compatibility shims. If a phase
cannot be made safe without reintroducing an old path, stop and split a smaller
spec rather than landing the old path.

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-26T11:28:35Z
Ended: 2026-05-26T11:28:35Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: The draft is well-grounded against current code (every cited path/line verified) and the cutover framing is consistent with the no-legacy invariant. Two issues block approval as written: (1) the Phase 6 acceptance command `pnpm contracts:check` does not exist in `package.json` (closest are `contracts:schemas:check` / `fixtures:contracts:check`), so the phase cannot pass its own gate; and (2) the Phase 1 directive to "replace the cache with a mechanism whose trust still derives from the expected digest" leaves the actual replacement undefined, which on a security boundary is the wrong place to discover the design at build time. Several advisory issues also need resolution before build: overlap with active `runx-rust-95-release-readiness` Phase 3 (same sandbox/credential files), Phase 4 ambiguity over whether `packages/cli/tools/shell/exec` is a true duplicate or dead code (no consumer found in repo), ordering risk against `monolith-decomposition-v1` (which lists `sandbox.rs`, `credentials.rs`, `runner.rs` as decomposition targets), and an unspecified "explicit local development" override surface for non-dev sandbox profiles. Recommend tightening the spec on these points before approving.

Checks:
- path audit
  - Grounded in: code:packages/cli/src/native-runx.ts:54-68 + code:packages/cli/bin/runx:71-87 + code:crates/runx-runtime/src/receipts/signing.rs:92-104 + code:crates/runx-runtime/src/execution/runner.rs:50-83 + code:crates/runx-runtime/src/sandbox.rs:499-561 + code:crates/runx-runtime/src/sandbox.rs:786-792 + code:tools/shell/exec/src/index.ts:20-44 + code:packages/cli/tools/shell/exec/src/index.ts:20-44 + code:crates/runx-runtime/src/credentials.rs:395-426 + code:crates/runx-cli/src/skill.rs:118-129 + code:crates/runx-runtime/src/adapters/mcp/transport.rs:235-253 + code:packages/contracts/src/internal.ts:178-210 + code:packages/contracts/src/schemas/spine.ts:441-458 + code:scripts/generate-rust-contract-fixtures.ts:908-947 + code:packages/create-skill/bin/create-skill.js:21-38
  - Result: passed
  - Evidence: Each cited file:line in the Evidence section was opened and the cited behavior matches the spec's claim (cwd-based binary discovery, fingerprint cache + RUNX_SKIP_NATIVE_VERIFY, env-derived signing defaulting to ReceiptIssuerType::Local, runner.rs defaulting to local_development(), sandbox runtime resolved via find_executable on caller PATH, writable_mount_path canonicalizing only the existing path/parent, identical shell.exec implementations in two trees, material_ref embedded in observation/request IDs + credential URI, parse_secret_env accepting ENV=VALUE, MCP child terminated without process-tree kill, hand-authored Type.* facade in internal.ts consumed by spine.ts, authoritySubsetProofSchema with Type.Literal("subset"), stableJson with localeCompare + ASCII guard, create-skill tsx fallback and hardcoded `/home/kam/dev/runx/oss` workspace hint).
- command audit
  - Grounded in: code:package.json:16-50 + spec:phase6 acceptance + spec:Final Validation
  - Result: failed
  - Evidence: Phase 6 acceptance lists `pnpm contracts:check` and `pnpm docs:api`. `package.json` does not define `contracts:check` — the matching scripts are `contracts:schemas:check`, `fixtures:contracts:check`, and `fixtures:contracts:keys`. `docs:api` exists. The Final Validation block lists `pnpm boundary:check` (exists), `pnpm typecheck` (exists), `pnpm test:fast` (exists), `cargo fmt --manifest-path crates/Cargo.toml --all -- --check` (workspace at crates/Cargo.toml confirmed), and crate package args `-p runx-contracts -p runx-core -p runx-receipts -p runx-runtime` (all four crate names confirmed via `crates/*/Cargo.toml`). Phase 2 acceptance `cargo test ... -p runx-receipts` also resolves because the crate exists. Net: one acceptance command in Phase 6 is wrong and will not pass.
- scope/migration audit
  - Grounded in: spec:Scope + spec:Dependencies + code:.scafld/specs/active/runx-rust-95-release-readiness.md:266-301 + code:.scafld/specs/drafts/monolith-decomposition-v1.md:117-274 + code:.scafld/specs/drafts/rust-ts-sunset-runtime-local-post-sunset-cleanup.md
  - Result: passed
  - Evidence: Spec correctly defers broad decomposition to `monolith-decomposition-v1`, deletion of runtime-local/adapters to the sunset specs, and acknowledges `runx-rust-95-release-readiness` ownership. However the active rust-95 spec's Phase 3 also targets sandbox enforcement, canonicalized writable paths, and credential opacity — the same files (sandbox.rs, credentials.rs) this spec edits. The boundary between 'another agent owns it' and 'this spec hardens surviving paths' is preserved in words but not in file ownership; coordination must be explicit. Recorded as design challenge, not check failure, because the spec does name the overlap up front.
- acceptance timing audit
  - Grounded in: spec:Phase 0 acceptance + spec:Phase 1-9 acceptances + spec:Final Validation
  - Result: passed
  - Evidence: Each phase's acceptance is testable at the time the phase is opened: Phase 0 is grep/evidence-only; Phases 1-5 each cite phase-local rg patterns and narrowly-scoped cargo/pnpm tests; Phase 6 schema check, Phase 7 cargo authority tests, Phase 8 fixture/codepoint checks, and Phase 9 boundary-style check do not depend on un-built artifacts. Final Validation block runs the heavy suite after all phases. No acceptance step requires a file or fixture that is created in a later phase.
- rollback/repair audit
  - Grounded in: spec:Rollback + spec:Review Gate
  - Result: passed
  - Evidence: Rollback is phase-local and explicitly rejects the option of re-introducing the deleted compatibility surface, with the escape hatch of splitting a smaller spec. This is coherent with the hard-cutover framing and matches the no_legacy_code invariant. Review gate lists the actual failure shapes (hidden production fallback, duplicated implementation, raw material_ref leakage, hand-authored schema authority, compatibility wording) rather than restating the phase list.
- design challenge
  - Grounded in: spec:Phase 1 (cache replacement) + spec:Phase 3 (dev override surface) + code:packages/runtime-local/src/runner-local/registry-resolver.ts:240-242
  - Result: passed
  - Evidence: The framing is the right architectural move (collapse trust to digest, fail closed by default, recompute attenuation, generate-not-author). Two design holes remain: Phase 1 says replace the (size,mtime) cache with 'a mechanism whose trust still derives from the expected digest' without naming the mechanism — implementation will define the trust boundary at build time, which is not safe on a launcher hot path. Phase 3 says fail closed for non-dev profiles 'unless enforcement is explicit local development behavior' without specifying which surface signals 'local development' (env var, profile name, both?). Also, `RUNX_RUST_REGISTRY_BIN` survives in `packages/runtime-local/src/runner-local/registry-resolver.ts` as a required registry-resolver input, distinct from the launcher use this spec deletes; the env-var name reuse across two roles is a footgun worth naming explicitly in Phase 1's evidence section.

Issues:
- [high/blocks approval] `harden-1` command_audit - Phase 6 acceptance `pnpm contracts:check` does not exist.
  - Status: open
  - Grounded in: code:package.json:16-50 + spec:Phase 6 acceptance
  - Evidence: `package.json` defines `contracts:schemas:generate`, `contracts:schemas:check`, `fixtures:contracts:check`, `fixtures:contracts:keys`, and `docs:api` — but no `contracts:check`. The trust-boundary spec's Phase 6 acceptance lists `pnpm contracts:check` as the gate. As written the phase cannot pass its own acceptance.
  - Recommendation: Replace `pnpm contracts:check` with the actual gates: `pnpm contracts:schemas:check && pnpm fixtures:contracts:check` (matches the pattern used in `runx-contract-spine-hard-cutover.md` and `scripts/verify-fast.mjs`).
  - Question: Which existing script(s) should Phase 6 acceptance run?
  - Recommended answer: Replace with `pnpm contracts:schemas:check && pnpm fixtures:contracts:check`, plus the existing `pnpm docs:api` regeneration check.
  - If unanswered: Default to `pnpm contracts:schemas:check && pnpm fixtures:contracts:check && pnpm docs:api`.
- [high/blocks approval] `harden-2` design_challenge - Phase 1's replacement for the (size,mtime) verification cache is unspecified.
  - Status: open
  - Grounded in: spec:Phase 1 + code:packages/cli/bin/runx:71-120
  - Evidence: Phase 1 says: 'Remove `RUNX_SKIP_NATIVE_VERIFY`. If hashing every launch is too expensive, replace the cache with a mechanism whose trust still derives from the expected digest, not mutable file metadata alone.' The current cache at `packages/cli/bin/runx:97-119` stores `{sha256, fingerprint}` keyed by hash(binaryPath) and trusts a (size, mtime) match. The spec does not name the replacement (digest-keyed cache file? immutable-after-write check? mmap-and-incremental-hash? always-hash?). On a launcher trust boundary, leaving the mechanism to implementation time is how the original cache was born.
  - Recommendation: Pin the replacement in the spec before build: either (a) always hash on launch (state expected latency budget); (b) cache file whose key includes the expected sha256 and whose existence is the only signal — never trust on-disk metadata; or (c) drop the cache entirely and rely on npm install-time integrity. Whichever is chosen, the spec should say that any cache miss falls through to a full hash and there is no opt-out env.
  - Question: What is the replacement verification mechanism, and what is the latency budget that justifies any cache at all?
  - Recommended answer: Drop the cache; always hash on launch. If a benchmark shows the per-launch cost is unacceptable, fall back to a digest-keyed marker file (`<cache>/<sha256>.verified`) whose presence — and ONLY whose presence — short-circuits re-hashing; do not trust size/mtime.
  - If unanswered: Default to always-hash with no opt-out env.
- [medium/advisory] `harden-3` design_challenge - Phase 3's 'explicit local development' override surface is not defined.
  - Status: open
  - Grounded in: spec:Phase 3 + code:crates/runx-runtime/src/sandbox.rs:499-544
  - Evidence: Today `resolve_sandbox_runtime` returns `SandboxRuntime::Direct` for `SandboxProfile::UnrestrictedLocalDev`, and the `RUNX_SANDBOX_REQUIRE_ENFORCEMENT` env can force enforcement. Phase 3 says 'Fail closed for governed sandbox profiles unless enforcement is explicit local development behavior' without naming the surface. Two plausible options today (profile name = UnrestrictedLocalDev; env var = require-enforcement) point in opposite directions: one is a positive opt-in to laxity by profile, the other is a positive opt-in to strictness by env. The build phase should not invent a third signal.
  - Recommendation: Specify in the spec that the only path to non-enforcing sandbox execution is `SandboxProfile::UnrestrictedLocalDev` (an explicit profile name), and that `RUNX_SANDBOX_REQUIRE_ENFORCEMENT` is removed in favor of failing closed by default for all other profiles. Or, if env-based opt-in remains, invert it: a `RUNX_SANDBOX_ALLOW_UNENFORCED=1` opt-in for governed profiles, never an opt-out.
  - Question: Is the dev override the profile name, an inverted env var, or both — and is `RUNX_SANDBOX_REQUIRE_ENFORCEMENT` deleted as part of this phase?
  - Recommended answer: Profile name only: `UnrestrictedLocalDev` is the sole non-enforcing profile; delete `RUNX_SANDBOX_REQUIRE_ENFORCEMENT` since fail-closed becomes the default for every other profile.
  - If unanswered: Default to profile-name-only with `RUNX_SANDBOX_REQUIRE_ENFORCEMENT` removed.
- [medium/advisory] `harden-4` scope_coordination - Phase 3/5/7 overlap with active `runx-rust-95-release-readiness` Phase 3 must be sequenced, not just declared out-of-scope.
  - Status: open
  - Grounded in: spec:Out of scope + code:.scafld/specs/active/runx-rust-95-release-readiness.md:266-301
  - Evidence: Active rust-95 Phase 3 covers 'enforced sandbox backends … canonicalize and validate existing cwd/writable paths … keep raw credentials out of process env'. Trust-boundary Phase 3/5 cover the same files (`sandbox.rs`, `credentials.rs`) and the same outcomes. The trust-boundary spec says rust-95 is 'another agent's' but the file overlap is real — sequenced builds will merge-conflict, and concurrent builds will fight on the same modules.
  - Recommendation: Add a Sequencing section: either (a) declare trust-boundary blocked until rust-95 Phase 3 lands and rebase from there, or (b) cite the exact subset of sandbox/credential changes rust-95 is allowed to land while this spec is open. Without this, Phase 0's ownership map is doing work the spec hasn't sized.
  - Question: Does trust-boundary build start before or after rust-95 Phase 3 lands?
  - Recommended answer: Trust-boundary build waits for rust-95 Phase 3 to merge, then rebases Phase 3/5 onto that state in its own Phase 0 evidence refresh.
  - If unanswered: Default to: trust-boundary build phases 3 and 5 are blocked until rust-95 Phase 3 is merged; document in Phase 0 evidence refresh.
- [medium/advisory] `harden-5` scope_coordination - Edits to `sandbox.rs`, `credentials.rs`, and `execution/runner.rs` will collide with `monolith-decomposition-v1`.
  - Status: open
  - Grounded in: spec:Phase 9 + code:.scafld/specs/drafts/monolith-decomposition-v1.md:117-274
  - Evidence: `monolith-decomposition-v1` lists `crates/runx-runtime/src/sandbox.rs` (1228 lines), `crates/runx-runtime/src/credentials.rs` (545 lines), and `crates/runx-runtime/src/execution/runner.rs` (423 lines) as priority decomposition targets. Trust-boundary phases 1-5 edit all three. Phase 9 punts broad decomposition but the body churn lands first, so monolith-decomposition will rebase after every batch.
  - Recommendation: Either run monolith-decomposition-v1 after trust-boundary lands (declare the ordering in monolith's Dependencies), or carve out the smallest local splits inside Phase 9 that make the trust-boundary diff readable and stop there. Spec already says the latter — make the ordering explicit so monolith-decomposition does not silently get blocked.
  - Question: Does this spec block `monolith-decomposition-v1`'s sandbox/credentials/runner batches until it lands?
  - Recommended answer: Yes — add a one-line note to Dependencies that `monolith-decomposition-v1` sandbox/credentials/runner batches start after this spec ratifies, to avoid two specs editing the same god-files concurrently.
  - If unanswered: Default to declaring this spec blocks monolith-decomposition-v1's sandbox/credentials/runner batches.
- [medium/advisory] `harden-6` scope_clarity - Phase 4's `shell.exec` 'collapse' likely reduces to deleting a dead copy.
  - Status: open
  - Grounded in: spec:Phase 4 + code:tools/shell/exec/src/index.ts + code:packages/cli/tools/shell/exec/src/index.ts + grep:`packages/cli/tools/shell/exec`
  - Evidence: Repo-wide grep for `packages/cli/tools/shell/exec` finds zero consumers outside this draft spec. `tools/shell/exec` is referenced by `skills/release/X.yaml`, `skills/evolve/X.yaml`, and tests. The two source files are byte-identical at lines 1-46. If no consumer imports the `packages/cli/tools/...` copy, the right action is deletion plus a check that nothing still ships it, not 'collapse'.
  - Recommendation: Reword Phase 4 to: 'Delete `packages/cli/tools/shell/exec` (no in-repo consumer; verify via boundary check and a packaged-CLI test). If a hidden consumer is found during Phase 0 evidence refresh, escalate before deleting.' Then the surviving `tools/shell/exec` is the only thing that needs the timeout/output-cap/redaction hardening described in this phase.
  - Question: Is `packages/cli/tools/shell/exec` shipped via any package's `files`/`exports` or referenced by any external runner? If not, can Phase 4 explicitly delete it?
  - Recommended answer: Confirm in Phase 0 that the duplicate has no consumer, then delete it; harden only the surviving `tools/shell/exec` copy.
  - If unanswered: Default to deletion after Phase 0 verification.
- [low/advisory] `harden-7` consistency - `RUNX_RUST_REGISTRY_BIN` has a second, legitimate use in the registry resolver that the spec does not call out.
  - Status: open
  - Grounded in: spec:Phase 1 + code:packages/runtime-local/src/runner-local/registry-resolver.ts:240-242
  - Evidence: `packages/runtime-local/src/runner-local/registry-resolver.ts:240` requires `env.RUNX_RUST_REGISTRY_BIN` for the native registry boundary. Phase 1 deletes its use as a launcher fallback in `packages/cli/src/native-runx.ts` but does not mention this surviving consumer. A reader of the spec could plausibly delete the env var entirely.
  - Recommendation: Add one sentence to Phase 1: 'The launcher loses its dependency on `RUNX_RUST_REGISTRY_BIN`; the registry resolver in `packages/runtime-local/src/runner-local/registry-resolver.ts` keeps it as the only legitimate consumer (or, if runtime-local sunset moves first, the env var disappears with it). Either way, after this phase, no launcher path reads it.'
  - Question: Is the registry-resolver use still expected to survive this cycle, or is it deleted by `rust-ts-sunset-runtime-local-post-sunset-cleanup`?
  - Recommended answer: Keep it as-is for now; note in Phase 1 that the launcher no longer reads it, and let the sunset spec handle eventual removal.
  - If unanswered: Default to: launcher stops reading it; registry-resolver use stays untouched.
- [low/advisory] `harden-8` design_challenge - `--secret-env ENV=VALUE` removal is an unannounced CLI break.
  - Status: open
  - Grounded in: spec:Phase 5 + code:crates/runx-cli/src/skill.rs:118-129
  - Evidence: Phase 5 deletes the old argv shape and explicitly rejects the alias. Reasonable under no_legacy_code, but the spec does not say whether any external dogfood scripts (`scripts/dogfood-*.mjs`, smoke tests, Nitrosend) feed secret material via this form. If they do, the build phase will discover it via test failure rather than spec.
  - Recommendation: Phase 0 evidence refresh should grep `scripts/`, fixtures, and known consumer repos (Nitrosend) for `--secret-env *=` usage and either rewrite them in the same PR or block on their owners before deleting the parser.
  - Question: Are any active scripts or consumer repos passing `--secret-env ENV=VALUE` today?
  - Recommended answer: Run a grep across `scripts/`, `tests/`, and known consumer repos during Phase 0 evidence refresh and list the call sites; rewrite them in the same change.
  - If unanswered: Default to: Phase 0 must grep and rewrite or block on all known call sites before Phase 5 lands.

### round-2

Status: needs_revision
Started: 2026-05-26T11:48:19Z
Ended: 2026-05-26T11:48:19Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round 1's two blockers are resolved in the body of the spec: Phase 6 acceptance now references the real scripts `pnpm contracts:schemas:check` / `pnpm fixtures:contracts:check` (verified against package.json:21,42), and the "Sequencing Notes From Harden Round 1" pin Phase 1 launcher verification to unconditional hashing with no opt-out cache. Round 2's verification of every cited file/line surfaces a new, material problem the spec did not catch: a significant fraction of the "Evidence From Current Audit" section no longer reproduces — `packages/cli/bin/runx` already hashes on every launch with no `RUNX_SKIP_NATIVE_VERIFY` and no size/mtime cache; `packages/cli/src/native-runx.ts:61-70` already resolves only `RUNX_RUST_CLI_BIN` then `"runx"`, with no `process.cwd()/crates/target/...` discovery, no `RUNX_RUST_REGISTRY_BIN` consumption, and bounded stdout/stderr + timeout already enforced at native-runx.ts:107-136; `crates/runx-cli/src/skill.rs:120-126` already rejects `--secret-env ENV=VALUE`; and `packages/create-skill/bin/create-skill.js:11-27` no longer carries a tsx fallback or a hardcoded `/home/kam/...` workspace path. Because the spec's own "False-Positive Discipline" section forbids cleanup churn for issues that are gone, leaving these as live claims in the Evidence body means Phase 0 will end up retiring much of the work the spec advertises, and approvers will be reading a stale audit. The findings that DO still reproduce are real and well-grounded (signing.rs:92-104 silent local fallback + `ReceiptIssuerType::Local` for env-derived production seed, sandbox.rs:499-580 default-not-fail-closed + caller-PATH `bwrap` discovery, credentials.rs:395-426 raw `material_ref` in observation/request IDs and credential URI, spine.ts:454 asserted `result: Type.Literal("subset")`, two byte-identical `shell.exec` trees, hand-authored Type facade in internal.ts:178, stableJson/localeCompare/ASCII guard in scripts/generate-rust-contract-fixtures.ts:908-947, MCP non-process-tree termination at adapters/mcp/transport.rs:250-253, docs/how-we-test.md:14-15 "surviving TypeScript package boundaries" compatibility wording). Also raising: Phase 1's acceptance grep only checks `RUNX_SKIP_NATIVE_VERIFY|process.cwd().*crates.*target|RUNX_RUST_REGISTRY_BIN` and misses the still-live `RUNX_RUST_CLI_BIN` override at native-runx.ts:62, which accepts any existing file path as the trusted binary — that is the unfinished part of the launcher trust boundary and the gate doesn't see it; rust-95 release-readiness is still at phase1 (not yet built phase3), so the Sequencing Notes' "wait for rust-95 P3 then rebase" coordination is words not pipeline state and the harden gate should not approve until that ordering is made concrete; Phase 6's acceptance grep `Type\.` will still light up on any surviving thin re-export and the spec does not name what is allowed to remain; and the Phase 5 sanitizer-test home (which crate, which test target) is not named, so the acceptance "Receipt serialization tests prove raw material refs do not appear" is not phase-runnable as written. None of these are unsafe to land — they are coherence/scope-clarity issues that will surface as confusion or as a too-permissive acceptance gate. Recommend revising before approve.

Checks:
- path audit
  - Grounded in: code:packages/cli/bin/runx:1-87 + code:packages/cli/src/native-runx.ts:1-140 + code:crates/runx-runtime/src/receipts/signing.rs:1-130 + code:crates/runx-runtime/src/execution/runner.rs:45-94 + code:crates/runx-runtime/src/sandbox.rs:490-590 + code:crates/runx-runtime/src/credentials.rs:380-427 + code:crates/runx-cli/src/skill.rs:100-150 + code:crates/runx-runtime/src/adapters/mcp/transport.rs:225-260 + code:packages/contracts/src/internal.ts:170-215 + code:packages/contracts/src/schemas/spine.ts:1-24,380-460 + code:scripts/generate-rust-contract-fixtures.ts:900-947 + code:packages/create-skill/bin/create-skill.js:1-27 + code:tools/shell/exec/src/index.ts:1-46 + code:packages/cli/tools/shell/exec/src/index.ts:1-46 + code:docs/how-we-test.md:10-25
  - Result: failed
  - Evidence: Live and verified: signing.rs:92-104 still returns local_development() when env absent and uses ReceiptIssuerType::Local for env-derived production seed; runner.rs:49-65 Default => local_development() is still in production path via from_process_env at runner.rs:353 and harness/runner.rs:127; sandbox.rs:499-544 still uses opt-in RUNX_SANDBOX_REQUIRE_ENFORCEMENT and DeclaredPolicyOnly fallback for non-dev profiles, sandbox.rs:546-580 still uses find_executable on caller PATH for bwrap/sandbox-exec; credentials.rs:395-426 still embeds material_ref into observation_id, request_id, and credential URI (only a separate hash field is opaque); adapters/mcp/transport.rs:250-253 still uses start_kill on the direct child only; internal.ts:178-215 still exports the hand-authored Type facade and spine.ts:1-24 + 386-458 still uses it for authority schemas; spine.ts:454 still has result: Type.Literal("subset") allowing asserted-subset proof; scripts/generate-rust-contract-fixtures.ts:908-920 still has local stableJson with localeCompare and 922-947 has ASCII/key-order guard; tools/shell/exec/src/index.ts:1-46 and packages/cli/tools/shell/exec/src/index.ts:1-46 are byte-identical; docs/how-we-test.md:14-15 still says "surviving TypeScript package boundaries". RETIRED — no longer reproduces: packages/cli/bin/runx never contains RUNX_SKIP_NATIVE_VERIFY or a size/mtime cache (verifyNativePackage hashes on every launch at lines 70-73, no env opt-out, no cache anywhere); packages/cli/src/native-runx.ts has no cwd-based discovery — resolveNativeRunxBinary at lines 61-70 only reads RUNX_RUST_CLI_BIN then returns "runx", and stdout/stderr are bounded at lines 44-45 + 118-136 with timeout at 107-111; crates/runx-cli/src/skill.rs:120-126 explicitly errors on `--secret-env ENV=VALUE` and only accepts an env-var name; packages/create-skill/bin/create-skill.js:11-27 has no tsx fallback and no hardcoded workspace path. The spec's Evidence section still asserts these retired findings as live.
- command audit
  - Grounded in: code:package.json:16-60 + spec:Phase 6 acceptance line 416 + spec:Final Validation lines 504-513 + spec:Phase 1 acceptance line 268
  - Result: passed
  - Evidence: Round 1 blocker harden-1 is resolved: spec line 416 now reads `pnpm contracts:schemas:check` and `pnpm fixtures:contracts:check`, which both exist in package.json:21 and :42. `pnpm docs:api` exists at package.json:29. Final Validation commands (`pnpm typecheck`, `pnpm test:fast`, `pnpm boundary:check`, `cargo fmt --manifest-path crates/Cargo.toml --all -- --check`, the four cargo `-p` crate names, `node scripts/check-rust-core-style.mjs`, `scafld validate ...`) all resolve. Phase 1 grep target `RUNX_SKIP_NATIVE_VERIFY|process\.cwd\(\).*crates.*target|RUNX_RUST_REGISTRY_BIN` runs fine but is incomplete — flagged as a separate scope issue, not a command failure.
- scope/migration audit
  - Grounded in: code:.scafld/specs/active/runx-rust-95-release-readiness.md:16-23,266-301 + code:.scafld/specs/drafts/monolith-decomposition-v1.md:117-274 + spec:Dependencies + spec:Sequencing Notes From Harden Round 1
  - Result: passed
  - Evidence: Round 1 advisory harden-4 / harden-5 are addressed in words: spec lines 209-213 explicitly require Phase 0 to re-check the rust-95 worktree before editing sandbox.rs/credentials.rs/runner signing files, and Dependencies lines 185-187 declare monolith-decomposition-v1's sandbox/credentials/runner batches must wait until this spec's trust-boundary edits land. rust-95 is still at phase1 (line 17 of its spec), not phase3 — so the sequencing is real, not retroactive. Recording as passed because the spec acknowledges the overlap; raising the unsettled coordination separately as an issue rather than as a check failure.
- acceptance timing audit
  - Grounded in: spec:Phase 0-9 acceptances + spec:Final Validation
  - Result: passed
  - Evidence: All per-phase acceptances are runnable at the time each phase opens: Phase 0 is grep/evidence-only; Phase 1 grep + CLI test target; Phase 2 narrow `cargo test -p runx-runtime receipts` and `cargo test -p runx-receipts`; Phase 3 narrow `cargo test -p runx-runtime sandbox`; Phase 4 `pnpm test:fast` + rust adapter tests; Phase 5 grep + receipt/CLI tests; Phase 6 schema-check + fixture-check + docs:api; Phase 7 `cargo test -p runx-core -p runx-receipts authority`; Phase 8 fixture/codepoint checks; Phase 9 boundary-style. No phase consumes an artifact a later phase must create. Final Validation block correctly runs the heavy suite after all phases.
- rollback/repair audit
  - Grounded in: spec:Rollback lines 533-536 + spec:Review Gate lines 520-530
  - Result: passed
  - Evidence: Rollback is phase-local and explicitly disallows re-introducing the deleted compatibility surface, with the named escape hatch of splitting a smaller spec rather than landing the old path. This matches the no_legacy_code invariant and the hard-cutover framing. Review Gate enumerates the actual failure shapes (hidden production fallback, duplicated implementation, raw material_ref leakage, hand-authored schema authority, compatibility wording, rename-without-collapse) rather than restating the phase list — coherent guardrail.
- design challenge
  - Grounded in: code:packages/cli/src/native-runx.ts:61-70 + code:packages/cli/bin/runx:1-87 + code:.scafld/specs/active/runx-rust-95-release-readiness.md:17 + spec:Phase 1 acceptance + spec:Phase 5 acceptance + spec:Phase 6 acceptance line 414 + spec:Evidence From Current Audit
  - Result: passed
  - Evidence: The cutover framing is the right architectural move: collapse trust to digest (already executed in bin/runx), fail closed by default on sandbox profiles (still to do), recompute authority attenuation rather than trust an asserted result (still to do), and delete byte-duplicated shell.exec rather than wrap it. Two design holes remain after Round 1 polish: (1) the Phase 1 acceptance grep only watches RUNX_SKIP_NATIVE_VERIFY / cwd / RUNX_RUST_REGISTRY_BIN, but native-runx.ts:62 still reads RUNX_RUST_CLI_BIN as a launcher-trust override that accepts any existing file path — the grep would not catch a regression and the spec does not name this as the documented dev-only hook nor constrain it to non-packaged paths; (2) rust-95 release-readiness is still at phase1 — the trust-boundary spec's Sequencing Notes promise to rebase onto rust-95 Phase 3 in Phase 0, but rust-95 Phase 3 hasn't started, so the build order is encoded as a future check, not as a hard dependency. Approving today would let both agents race on sandbox.rs/credentials.rs.

Issues:
- [medium/advisory] `harden-r2-1` stale_evidence - ~30% of the Evidence section no longer reproduces; spec body still asserts retired findings as live, contradicting its own False-Positive Discipline.
  - Status: open
  - Grounded in: code:packages/cli/bin/runx:1-87 + code:packages/cli/src/native-runx.ts:61-140 + code:crates/runx-cli/src/skill.rs:120-126 + code:packages/create-skill/bin/create-skill.js:11-27 + spec:Evidence From Current Audit + spec:False-Positive Discipline
  - Evidence: Spec line 111-114 claims `packages/cli/bin/runx:71-87 allows RUNX_SKIP_NATIVE_VERIFY and caches trust by size:mtimeMs`. Current bin/runx is 87 lines total and verifyNativePackage at lines 48-74 hashes the packaged binary on every launch with no env opt-out and no cache anywhere. Spec line 107-110 claims `native-runx.ts:54-68` resolves binaries from `process.cwd()/crates/target/...`. Current native-runx.ts:61-70 resolves only RUNX_RUST_CLI_BIN then "runx"; no cwd discovery. Spec line 110-112 claims native-runx.ts captures into unbounded strings; current native-runx.ts:44-45,118-136 enforces RUNX_RUST_CLI_OUTPUT_LIMIT_BYTES and a default 1 MiB cap. Spec line 131-132 claims `skill.rs:118-129 parses --secret-env ENV=VALUE`; current skill.rs:120-126 explicitly errors on `=` in the value. Spec line 143-144 claims `create-skill.js:21-38 falls back to tsx source execution and has a hardcoded personal workspace path`; current create-skill.js:11-27 just fails when dist is missing. Phase 0 covers this in theory, but the spec's own False-Positive Discipline says retired findings must be marked retired in this spec, not left as live claims.
  - Recommendation: Update the Evidence From Current Audit section now during harden: replace each retired finding with a one-liner that names what landed and which commit/test now covers it, OR add a clear preamble that the audit was a snapshot taken before [date] and Phase 0 will fold real-vs-retired into the Executed Evidence Refresh. Adjust each affected phase's Changes list so it no longer reads as work that is already done (Phase 1 cwd-discovery removal, RUNX_SKIP_NATIVE_VERIFY removal, byte/timeout caps; Phase 5 `--secret-env ENV=VALUE` removal). The cutover still has real work (Phase 1 RUNX_RUST_CLI_BIN constraint, Phase 2 signing fallback, Phase 3 sandbox fail-closed + PATH discovery, Phase 5 material_ref embedding, Phase 6 hand-authored Type facade, Phase 7 authority recompute, Phase 8 stableJson) — the spec just shouldn't carry retired findings into approval.
  - Question: Should the Evidence section be refreshed against current main before approve, or should the spec acknowledge upfront that ~30% of cited evidence is already retired and Phase 0 owns the bookkeeping?
  - Recommended answer: Refresh the Evidence section now: mark `bin/runx` cache/env-opt-out, `native-runx.ts` cwd-discovery + unbounded output, `skill.rs` ENV=VALUE parsing, and `create-skill.js` tsx fallback as retired-by-prior-work with the current line range and the gate that now covers each (e.g., bin/runx test, native-runx.test.ts at packages/cli/src/native-runx.test.ts, the skill-cli `--secret-env` test). Also trim each phase's Changes list so it doesn't restate work already landed.
  - If unanswered: Default to: add a one-paragraph preamble above Evidence noting the audit snapshot date and pointing readers to Phase 0's Executed Evidence Refresh as the live source of truth; keep the historical findings for context.
- [medium/advisory] `harden-r2-2` scope_gap - Phase 1's acceptance grep does not cover the surviving `RUNX_RUST_CLI_BIN` launcher override, which is the actual remaining launcher trust footgun.
  - Status: open
  - Grounded in: code:packages/cli/src/native-runx.ts:61-70 + code:packages/cli/src/native-runx.test.ts:28-30 + spec:Phase 1 acceptance line 268 + spec:Sequencing Notes lines 201-204
  - Evidence: packages/cli/src/native-runx.ts:62 reads `env.RUNX_RUST_CLI_BIN` and accepts ANY existing file path as the trusted binary (line 64: `override === "runx" || existsSync(override)`). This is a launcher-trust override that the spec does not name in Phase 1 changes nor in the Phase 1 acceptance grep (`RUNX_SKIP_NATIVE_VERIFY|process.cwd().*crates.*target|RUNX_RUST_REGISTRY_BIN`). The Sequencing Notes carefully scope `RUNX_RUST_REGISTRY_BIN` (registry-resolver gets to keep it, launcher loses it) but say nothing about `RUNX_RUST_CLI_BIN`, which is the env var the launcher actually still consumes and which 12+ test files depend on. Without an explicit Phase 1 stance, a reader could either (a) leave it as-is (footgun) or (b) delete it and break the test/dev workflow.
  - Recommendation: Add one paragraph to Phase 1 naming `RUNX_RUST_CLI_BIN` as the explicit, documented dev/test hook: keep it for test harnesses and IDE plugin, but require it to be rejected when the launcher entry point is `packages/cli/bin/runx` (the packaged path). Update the Phase 1 acceptance grep to assert that `bin/runx` does not consult `RUNX_RUST_CLI_BIN` and that the TS-consumer launchers do. If the answer is instead to keep the current shape, say so in Sequencing Notes so the reviewer is not surprised.
  - Question: Is `RUNX_RUST_CLI_BIN` an intentional dev/test hook, and should the packaged launcher (`bin/runx`) be required to reject it?
  - Recommended answer: Keep `RUNX_RUST_CLI_BIN` as the documented dev/test hook in non-packaged consumers (TS adapters, IDE plugin, MCP serve), and add an explicit assertion in Phase 1 that `packages/cli/bin/runx` does not read it; extend the Phase 1 acceptance grep to cover it.
  - If unanswered: Default to: document `RUNX_RUST_CLI_BIN` as the dev/test hook, keep the bin/runx packaged launcher free of it, and add it to the Phase 1 acceptance grep.
- [medium/advisory] `harden-r2-3` scope_coordination - rust-95 release-readiness is still at phase1 (not yet built phase3); trust-boundary's Sequencing Notes assume rust-95 P3 lands first, but the active spec contradicts that ordering.
  - Status: open
  - Grounded in: code:.scafld/specs/active/runx-rust-95-release-readiness.md:17,266-301 + spec:Sequencing Notes lines 209-213
  - Evidence: runx-rust-95-release-readiness.md:17 reads `Current phase: phase1` and Next: build — its Phase 3 (Fail-Closed Runtime Security at lines 266-301) targets the same `crates/runx-runtime/src/sandbox.rs` and credential paths that trust-boundary Phase 3/5 edit. Trust-boundary's Sequencing Notes at line 209-213 says Phase 0 must "re-check the active spec/worktree and record whether the other agent already landed that slice. If it did, this spec rebases onto that state". But rust-95 Phase 3 has NOT started, so Phase 0 will record "not landed" — meaning trust-boundary either edits the files first (rust-95 must rebase), or trust-boundary waits for an event with no firm date. Either outcome is OK; the spec doesn't pick.
  - Recommendation: Choose one and write it into Dependencies (not just Sequencing Notes): either (a) trust-boundary Phase 3/5 are blocked until rust-95 Phase 3 merges, with an explicit blocker line and the rust-95 task id; or (b) trust-boundary edits first and rust-95 rebases — in which case add a one-line note to rust-95's deps and ask its owner to ack. Whichever you pick, surface it in the Current State `Blockers:` line so `scafld status` reports it.
  - Question: Does trust-boundary build start before or after rust-95 Phase 3 lands?
  - Recommended answer: Trust-boundary edits first (the audit was deeper here and the cleanup is broader) — declare in Dependencies that rust-95 Phase 3 rebases onto trust-boundary's final state, and add a one-liner to rust-95's deps so its owner sees it. Mirror the choice in the Blockers line.
  - If unanswered: Default to: trust-boundary edits first; add a Dependencies line declaring rust-95 Phase 3 rebases after trust-boundary ratifies; reflect in Current State Blockers.
- [low/advisory] `harden-r2-4` acceptance_clarity - Phase 6 acceptance grep `Type\.` is too coarse to express the survival rule the spec actually wants.
  - Status: open
  - Grounded in: code:packages/contracts/src/internal.ts:178-215 + code:packages/contracts/src/schemas/spine.ts:1-24,386-460 + spec:Phase 6 acceptance line 414
  - Evidence: Phase 6 acceptance at line 414 greps for `export const Type|from "../internal.js".*Type|Type\.` across `packages/contracts/src/schemas` + `internal.ts`. But Phase 6 Changes line 405-410 allow "a thin import/export boundary" and "validator/type exports". A thin re-export still has `Type` substrings; if the Type facade in internal.ts:178 is deleted, the grep would have to also delete every downstream use in spine.ts — meaning Phase 6 implicitly deletes spine.ts (or rewrites it to consume the generated artifact). That outcome may be correct but the spec does not say it. The acceptance gate is stricter than the Changes paragraph.
  - Recommendation: Either (a) sharpen the Changes paragraph to say "delete `packages/contracts/src/schemas/spine.ts` and `packages/contracts/src/internal.ts` Type facade; the only surviving TS schema file is a generated artifact + validators index"; or (b) loosen the acceptance grep to match only the actual prohibition (e.g., `export const \w+Schema = Type\.` — hand-authored schema-shape authors — and leave thin re-exports alone). Pick one shape so the build phase does not discover the contract at PR review time.
  - Question: After Phase 6, does `packages/contracts/src/schemas/spine.ts` exist at all, and if so, what does it import?
  - Recommended answer: Spine.ts is deleted; the only TS schema surface is generated `packages/contracts/dist/schemas/*.json` + a thin `packages/contracts/src/schemas/index.ts` that re-exports validators/types from the generated artifact. Tighten the acceptance grep to match only `Type\.` calls (hand-author signal), not the substring.
  - If unanswered: Default to: spine.ts is deleted; acceptance grep matches only `Type\.(Object|Array|Literal|Optional|Union|String|Integer|Number|Boolean|Null)\(` to specifically catch hand-authored schema construction.
- [low/advisory] `harden-r2-5` acceptance_clarity - Phase 5's "sanitizer tests that fail if raw material refs appear in serialized receipts or logs" does not name the crate or test target.
  - Status: open
  - Grounded in: spec:Phase 5 acceptance lines 388-391 + code:crates/runx-runtime/src/credentials.rs:395-426 + code:crates/runx-receipts/Cargo.toml
  - Evidence: Phase 5 Changes line 384-385 says "Add sanitizer tests that fail if raw material refs appear in serialized receipts or logs." Acceptance line 391 says "Receipt serialization tests prove raw material refs do not appear." But material_ref construction is in `crates/runx-runtime/src/credentials.rs`, observation/receipt sealing crosses runtime↔receipts, and there is no named test target. A reviewer cannot tell whether to look in `runx-runtime/tests/credentials*.rs`, `runx-receipts/tests/...`, or a new fixture. Phase 5 acceptance becomes a judgement call.
  - Recommendation: Pin the test home in Phase 5 Changes: e.g., "Add `crates/runx-runtime/tests/credential_ref_opacity.rs` covering observation_id, request_id, credential_refs[].uri, and sealed-receipt JSON — assert no raw material_ref substring appears in any field beyond `material_ref_hash`." Reword acceptance to cite that test by file or by `cargo test -p runx-runtime credential_ref_opacity`.
  - Question: Where do the Phase 5 sanitizer tests live, and what is the runnable command?
  - Recommended answer: Add `crates/runx-runtime/tests/credential_ref_opacity.rs`; the Phase 5 acceptance command becomes `cargo test --manifest-path crates/Cargo.toml -p runx-runtime credential_ref_opacity`.
  - If unanswered: Default to: place sanitizer tests in `crates/runx-runtime/tests/credential_ref_opacity.rs` and reference that target in Phase 5 acceptance.
