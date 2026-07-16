---
name: review-skill
description: Inspect, safely test, and assess one Runx skill package for capability, trust, and operator readiness. Use when deciding whether to adopt, sandbox, improve, install, or publish a skill; its evidence-only assess runner is available when native test evidence already exists.
runx:
  category: authoring
---

# Review Skill

Evaluate one skill from bounded evidence. The default runner performs native
Runx inspection, runs read-only or planning harnesses, and passes that evidence
to the focused assessment runner. Execute-capable targets are inspected but
never run automatically.

This skill does not mutate the package, publish a report, install a package, or
manufacture missing evidence. Use `assess` directly when a caller already has a
bounded evidence pack.

## Procedure

1. Inspect the supplied skill reference and capture its native capability and
   readiness envelope.
2. Run the native harness only when catalog execution is `read` or `plan`.
3. Compare the documented capability with the inspected runner and catalog
   metadata in the evidence pack.
4. Separate native harness results, provider readback, supplied assertions, and
   unverified claims.
5. Check the happy path, refusal or stop path, authority boundary, artifact or
   effect, provenance, and recovery posture appropriate to the skill type.
6. Return `needs_more_evidence` when the evidence cannot support a trust
   decision. Never upgrade a parse result or prose claim into execution proof.
7. Recommend `adopt`, `adopt_with_caveats`, `sandbox`, `improve`, or `reject`,
   naming the evidence and blocking gaps behind the decision.

## Stop conditions

- Refuse evidence containing secrets, raw credentials, private customer data,
  private inbox content, or provider dumps.
- Do not recommend provider readiness without provider readback.
- Do not recommend publication from private previews, placeholder hosts, dead
  links, or unrelated parent domains.
- Return `needs_more_evidence` when no native inspection, harness, receipt, or
  equivalent bounded source evidence is supplied.
- Reject capabilities that cannot be bounded, audited, or assigned a truthful
  terminal state.

## Output

- `capability_profile`: bounded capability, execution shape, and claimed effect.
- `trust_assessment`: evidence tier, caveats, and unsupported claims.
- `test_matrix`: passed, failed, skipped, and still-required checks.
- `recommendation_report`: decision, rationale, blockers, and next action.

## Inputs

- `skill_ref` (required): package path, registry id, or marketplace id.
- `evidence_pack` (optional for the default runner, required for `assess`):
  inspection, harness, receipts, docs, or source evidence. References are
  preferable to copied private bodies.
- `objective` (optional): decision the evaluation should support.
- `test_constraints` (optional): time, environment, or safety limits.
