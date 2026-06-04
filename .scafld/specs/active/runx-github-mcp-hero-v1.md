---
spec_version: '2.0'
task_id: runx-github-mcp-hero-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T07:53:57Z'
status: review
harden_status: not_run
size: medium
risk_level: medium
---

# runx-github-mcp-hero-v1

## Current State

Status: review
Current phase: final
Next: complete
Reason: review gate pass: 4 finding(s), 0 completion blocker(s)
Blockers: none
Allowed follow-up command: `scafld complete runx-github-mcp-hero-v1`
Latest runner update: 2026-06-04T08:07:49Z
Review gate: pass

## Summary

Ship the first hero (executable today, gates nothing): governed GitHub via the MCP
adapter, scope-bounded, receipt-sealed. Grant `repo.read`, the agent reads an
issue/PR fine, attempts an out-of-scope write, admission REFUSES, and the sealed
denial receipt verifies offline. Plus the two read-only siblings that make it read
as a category: governed issue-triage (MCP read path plus the existing triage
packet runner) and a governed PR-review note (one scoped comment via the
public-comment path). The survey found NO `type: mcp` GitHub example anywhere
under `oss/examples` today, so this builds the first one — the demo north-star.md
names as the executable hero.

## Objectives

- A GitHub MCP-source skill that reads under `repo.read`, with a sealed receipt and
  an offline-verifiable governed REFUSAL of an out-of-scope write — the headline demo.
- Wire the existing `issue-triage` skill (already references a real github issue_url
  + comment channel) to the MCP read path; the MCP runner proves no mutation scope
  is exercised, while the existing response runner remains the triage-packet
  emitter.
- A new governed PR-review-note sibling: one scoped comment via the public-comment
  path, refusing any push/merge attempt.
- Each first-party skill sibling reaches a maturity tier with a harness case; the
  harness expectations prove no out-of-scope scope was exercised.

## Scope

In scope:
- The GitHub MCP-source example skill + the refusal demo (grant repo.read → attempt
  write → sealed denial → offline verify with `verify.mjs`).
- The `issue-triage` sibling wired to MCP read; the new PR-review-note sibling.
- Harness cases for the example and both siblings; maturity tiering + lock entries
  for the first-party skill siblings.

Out of scope:
- Non-GitHub hosted provider brokerage (cloud/private, separate).
- Any write beyond the single scoped PR comment; auto-merge; PR-opening (that is the
  TS `issue-to-pr` lane, not this hero).

## Dependencies

- SHIPPED: the MCP adapter (`serve_mcp_json_rpc`) + the MCP-source client, agent-step,
  GitHub connect (the only wired provider), receipts, authority/scopes, the harness.
- The governed-tool-call convention for the scoped-comment mutation (PR-review note).

## Assumptions

- GitHub is the only wired provider (north-star.md); the hero is scoped to it.
- The refusal must be a REAL admission denial (over-scope attempt), not a staged one,
  so the demo's credibility holds.

## Touchpoints

- A new `oss/examples/github-mcp-*` (the first MCP-source example) + harness.
- `oss/skills/issue-triage` (wire to MCP read); a new PR-review-note skill.
- The MCP adapter + GitHub connect path.
- `verify.mjs` (offline-verify the denial receipt).

## Risks

- **Mutation scope creep.** The PR-review note must be gated to exactly the comment
  scope; an over-broad scope undermines the least-privilege story. Mitigation: bound
  the scope + assert refusal of push/merge in the harness.
- **Demo credibility.** A staged refusal reads as theatre. Mitigation: the refusal is
  a genuine admission denial of an out-of-scope act, sealed + offline-verifiable.

## Acceptance

Profile: strict

Validation:
- The GitHub MCP read hero: a scoped read seals a receipt; an out-of-scope write is
  refused at admission with a sealed denial that `verify.mjs` confirms offline.
- The two siblings run: `issue-triage` has both its existing packet-emitting
  response runner and its MCP read runner; the PR-review note posts one scoped
  comment and refuses push/merge.
- `pnpm fixtures:harness:check` + the new harness cases pass; the sibling skills are
  maturity-tiered and locked.

## Phase 1: GitHub MCP read hero + refusal demo

Status: completed
Dependencies: MCP adapter, GitHub connect (shipped)

Objective: the first MCP-source GitHub example with the grant→read→refuse→sealed-

Changes:
- Build the GitHub MCP-source example skill + harness; the refusal demo + run script.

Acceptance:
- [x] `ac1` command - read seals, out-of-scope write refused + verifies offline
  - Command: `RUNX_BIN="${RUNX_BIN:-$(pwd)/crates/target/debug/runx}" examples/github-mcp-hero/run.sh`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-10

## Phase 2: The two read-only siblings (issue-triage + PR-review note)

Status: completed
Dependencies: Phase 1

Objective: the category-defining siblings, each sealed + scope-audited.

Changes:
- Wire `issue-triage` to MCP read; build the scoped PR-review-note skill; harness + maturity for both.

Acceptance:
- [x] `ac2` command - siblings seal + least-privilege holds
  - Command: `RUNX_BIN="${RUNX_BIN:-$(pwd)/crates/target/debug/runx}"; RDIR1="$(mktemp -d)"; RDIR2="$(mktemp -d)"; RUNX_RECEIPT_SIGN_KID=runx-demo-key RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64=QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI= RUNX_RECEIPT_SIGN_ISSUER_TYPE=hosted "$RUNX_BIN" harness skills/issue-triage --receipt-dir "$RDIR1" --json && RUNX_RECEIPT_SIGN_KID=runx-demo-key RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64=QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI= RUNX_RECEIPT_SIGN_ISSUER_TYPE=hosted "$RUNX_BIN" harness skills/pr-review-note --receipt-dir "$RDIR2" --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15

## Rollback

- Additive examples/skills. Remove the github-mcp example + the PR-review-note skill
  and revert the issue-triage wiring; no contract or SourceKind change.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-8
Output: claude.mcp_submit_review
Summary: Verify mode: the sole completion blocker (F1 — hero never offline-verified the denial with verify.mjs) is genuinely repaired. run.sh now (a) walks the receipt dir in Node, asserting seal.disposition==="blocked" && seal.reason_code==="authority_denied", exiting non-zero if no such denial receipt exists (run.sh:53-56), and (b) invokes the real `examples/governed-spend/verify.mjs` against that receipt (run.sh:58). verify.mjs recomputes the canonical body hash and checks the Ed25519 signature with only the Node stdlib, and exits 1 on NOT VERIFIED; under `set -e` that propagates, so the demo now fails loudly if the denial is not cryptographically verifiable. The decorative, unasserted grep is gone — the offline-verification credibility claim is actually demonstrated. Regression check on the ambient (out-of-scope) canonical.rs/number-parity change: it only alters f64 encoding outside ~[1e-7,1e21]; a GitHub denial receipt carries only strings and integers, which serialize identically under JS JSON.stringify and Rust serde, so verify.mjs still produces a matching body hash — no regression to the hero path. The genuine admission-denial mechanism (provider_permission string-set scope diff) and the pr-review-note least-privilege bounding remain intact and unchanged. The remaining findings (F2 issue-triage MCP-read emits no triage packet; F3 github-mcp-hero absent from official-skills lock; F4 named auditors not invoked by ac commands) are all non-blocking, unchanged from the prior pass, and do not gate completion — F3 is defensible since the hero is an example, not an official skill. The uncommitted canonical.rs/yaml.rs/canonical-json fixture changes are unrelated cutover/number-parity drift outside task scope and do not affect hero behavior. Blocker cleared; verdict pass.

Attack log:
- `oss/examples/github-mcp-hero/run.sh`: Verify F1: does the hero now offline-verify the sealed denial with verify.mjs, and is it asserted (fails non-zero on missing denial or NOT VERIFIED)? -> finding (F1 fixed. Node walk asserts blocked+authority_denied and exits non-zero if absent (run.sh:53-56); verify.mjs invoked at run.sh:58 and exits 1 on NOT VERIFIED, propagating under set -e. Decorative grep removed.)
- `oss/crates/runx-receipts/src/canonical.rs (ambient, out-of-scope)`: Regression: does the uncommitted JsonNumber->ryu canonical number change break verify.mjs's recomputed body hash for the denial receipt? -> clean (Change only affects f64 outside ~[1e-7,1e21]. GitHub denial receipt carries only strings (issue_number "241", timestamps) and integers, which JS JSON.stringify and Rust serde encode identically. verify.mjs still matches.)
- `oss/examples/github-mcp-hero/run.sh`: Shell correctness of the F1 fix under set -e: command-substitution failure handling and verify.mjs exit propagation. -> clean (DENIAL_RECEIPT empty-check (line 53) guards the substitution; verify.mjs non-zero exit propagates under set -e at line 58. No pipes in the critical path.)
- `oss/examples/governed-spend/verify.mjs`: Does the reused verifier actually recompute the hash and verify the signature independently (not trust runx), and exit non-zero on failure? -> clean (canon() recursively sorts keys, strips signature/digest/metadata, recomputes sha256 body hash, verifies Ed25519 with Node stdlib only; process.exit(allPass?0:1). Genuine offline verification.)
- `oss/skills/issue-triage/X.yaml`: Verify F2: does the MCP-read wiring emit a triage packet with no mutation scope? -> finding (F2 still open (non-blocking). mcp-read runner is a bare read graph (no artifacts wrap_as); no mutation scope (good). Grants only repo.read with verb read.)
- `oss/packages/cli/src/official-skills.lock.json`: Verify F3: are the skills maturity-tiered and locked per acceptance? -> finding (F3 still open (non-blocking). issue-triage + pr-review-note locked; github-mcp-hero (an example) absent — defensible exclusion but acceptance wording mismatch.)
- `oss/.scafld/specs/active/runx-github-mcp-hero-v1.md ac commands`: Verify F4: are the named receipt/least-privilege auditors invoked by acceptance? -> finding (F4 still open (low, non-blocking). ac1/ac2 run only run.sh + harness; harness expectations enforce least-privilege indirectly.)
- `workspace dirty set (canonical.rs, yaml.rs, canonical-json fixtures/test)`: Scope/ambient drift: do uncommitted changes outside task scope affect the hero or constitute task changes? -> clean (Unrelated cutover/number-parity drift outside declared scope (oss/examples/github-mcp-*, oss/skills/issue-triage, verify.mjs). Does not alter hero behavior; context only.)

Findings:
- [high/non-blocking] `F1` Hero now offline-verifies the sealed denial with verify.mjs as strict acceptance requires
  - Location: `oss/examples/github-mcp-hero/run.sh:58`
  - Evidence: run.sh:19-56 locates the denial receipt via a Node walk asserting seal.disposition==="blocked" && seal.reason_code==="authority_denied", exits non-zero if none found; run.sh:58 invokes node examples/governed-spend/verify.mjs against it. verify.mjs recomputes the canonical body hash + verifies the Ed25519 signature with only the Node stdlib and process.exit(1) on NOT VERIFIED, which propagates under set -e. The prior unasserted decorative grep is removed.
  - Validation: Read run.sh and verify.mjs end-to-end; the denial-not-found and NOT-VERIFIED paths both exit non-zero. Spec ac1 re-run recorded exit 0.
- [medium/non-blocking] `F2` issue-triage mcp-read runner still seals only a read receipt and emits no triage packet
  - Location: `oss/skills/issue-triage/X.yaml:105`
  - Evidence: The mcp-read runner (X.yaml:105-131) is a bare read graph with no artifacts.wrap_as/packet; the triage packet runx.issue.triage.v1 is produced only by the respond agent-task runner (X.yaml:143-145), which does not exercise the MCP read path. Unchanged from the prior review.
  - Impact: The triage sibling demonstrates a governed read but not a triage-packet-from-MCP-read artifact. Non-blocking.
- [medium/non-blocking] `F3` github-mcp-hero absent from official-skills lock; only issue-triage and pr-review-note are locked
  - Location: `oss/packages/cli/src/official-skills.lock.json`
  - Evidence: Lock contains runx/issue-triage (line 73) and runx/pr-review-note (line 148); no github-mcp-hero entry. github-mcp-hero is an example (catalog.kind: graph, visibility: public), so excluding it from the official-skills lock is defensible, but the acceptance wording 'three skills...locked' is not literally satisfied.
  - Impact: Either a missing lock entry or an acceptance-wording mismatch for the headline hero. Non-blocking.
- [low/non-blocking] `F4` Named receipt-auditor/least-privilege-auditor are not invoked by either acceptance command
  - Location: `oss/.scafld/specs/active/runx-github-mcp-hero-v1.md:134`
  - Evidence: ac1/ac2 only run run.sh and `runx harness`; neither invokes an auditor. The harness expectations (sealed read; blocked write/merge with authority_denied) do enforce the least-privilege property indirectly, so the property is checked even though the named auditors are not run.
  - Impact: Least-privilege confirmation relies on harness expectations rather than the auditors the spec names. Non-blocking.

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
