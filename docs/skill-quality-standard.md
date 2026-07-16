# Skill Quality Standard

Public Runx skills are portable execution contracts. A skill earns its place by
making an agent materially better at a recurring job: it may execute a governed
operation, encode a non-obvious workflow, produce a durable artifact, build or
improve other skills, or provide reusable bounded context. Deterministic
provider execution is one valuable shape, not the definition of skill value.

This standard applies to existing core skills and to every proposed addition.
It evaluates the claim a skill actually makes; it does not force every package
into the same architecture.

## Operator-value admission

A core skill must provide at least one of these forms of leverage:

1. **Operation** — crosses a runtime, state, protocol, or provider boundary and
   verifies the effect or readback.
2. **Workflow** — compresses a fragile recurring job with domain procedure,
   authority, gates, handoffs, recovery, and a truthful terminal state.
3. **Artifact** — creates a durable, provenance-bound output such as research,
   content, security analysis, growth intelligence, or a publication packet.
4. **Builder** — makes skills or governed systems easier to design, test,
   review, package, improve, or distribute.
5. **Context** — creates a reusable bounded packet that improves downstream
   decisions without pretending to perform an external action.

Internal runtime rails, fixtures, and owner-local graph stages can remain
non-public. Their value is judged through the canonical skill that owns them.

A package fails admission when it merely renames generic prompting, duplicates
another package without a distinct contract, claims an effect it cannot prove,
or leaves the caller to invent the actual procedure. Research, writing,
planning, and review are not disqualified: they qualify when the package adds
specialized sources, constraints, provenance, structure, evaluation, or
handoffs that materially improve the resulting work.

## Universal bar

Every public skill must have:

- a clear recurring job and a distinct owner in the catalog;
- a bounded default runner with a truthful terminal state;
- explicit inputs, outputs, authority, approvals, and stop conditions;
- a declared artifact or effect that survives beyond model prose;
- provenance for material facts, state, amounts, actors, and recommendations;
- replayable proof appropriate to its archetype;
- no hidden credentials, implicit consent, or unsupported provider claims;
- a concise package containing only files the skill consumes or emits.

Weak implementation is normally an improvement finding, not a deletion
decision. Removal or consolidation additionally requires a named canonical
replacement, consumer and registry migration, preservation of useful evidence,
and explicit product approval.

## Proof by archetype

Proof must match the claim:

| Archetype | Minimum proof |
|---|---|
| Operation | A no-managed-agent fixture or bounded live trial crosses the claimed boundary and verifies runtime or provider readback. |
| Workflow | A realistic path reaches each consequential gate and terminal state; deterministic stages use real fixtures, while supplied agent answers may prove the declared artifact contract. |
| Artifact | A realistic source packet produces schema-valid output with provenance, and a forward test or evaluator checks usefulness for its named consumer. |
| Builder | A fixture produces a valid package, policy, harness, or change artifact and runs the native validator that would accept it. |
| Context | A fixture produces the bounded context packet and a downstream forward test shows that the declared consumer can use it without inventing missing fields. |

`caller.answers` can prove graph wiring and an agent-artifact contract. It
cannot prove a provider mutation, network result, payment, send, publish, or
other external effect. Live destructive proof is never required when a faithful
sandbox plus refusal and approval cases establish the same boundary safely.

## Agent execution and consent

Agent work is valid for judgment and authorship. It must be isolated from
deterministic effects and close into a declared artifact packet.

The normal Runx path yields `needs_agent` to the caller. In-process managed
agent execution requires explicit per-run `--managed-agent` consent, displays
the act count and round budget before execution, and remains bounded. Available
model credentials are capability, not consent. A review must not spend model
tokens merely to prove a deterministic boundary or a supplied-answer contract.

Prepared context is always digest-bound and drift-checked, but it is not always
an approval gate. Safe reads, analysis, planning, and artifact generation are
admitted automatically. Human context approval is reserved for runners whose
selected execution graph declares a mutation; the receipt records an approval
decision only when a human actually supplied one. A skill's own approval step
still gates the specific consequential action at the point of use.

## Required `SKILL.md` structure

Each public skill must tell an agent enough to execute and audit the job without
inventing procedure. It should cover:

- `## What this skill does`
- `## When to use this skill`
- `## When not to use this skill`
- `## Procedure`
- `## Edge cases and stop conditions`
- `## Output schema`
- `## Worked example`
- `## Inputs`

Equivalent headings are acceptable when the same operating information remains
obvious. The document is execution guidance, not a repeated internal review
rubric or marketing page.

## Content bar

- Name the evidence and distinguish source facts from inference.
- Name the authority; intent alone does not grant permission.
- Name the gate before any mutation, charge, delegation, publish, or send.
- Name finality precisely: a plan is not a delivery, and an accepted request is
  not a verified external effect.
- Fail closed on stale evidence, replay, scope mismatch, ambiguous ownership,
  missing consent, or missing provider readback.
- Keep raw secrets out of inputs, logs, artifacts, receipts, and examples.
- Preserve domain boundaries: auditors do not silently repair, planners do not
  claim execution, and provider facades do not replace canonical governance.
- Make recovery and idempotency explicit wherever retries can cause harm.

## Execution profile discipline

`X.yaml` owns executable capability and governance:

- runners, typed inputs and outputs, and default selection;
- agent-versus-deterministic step boundaries;
- tool, adapter, context-skill, and graph wiring;
- authority, approval, scopes, and receipt-act mappings;
- side-effect posture and truthful completion semantics;
- artifact packets and focused harness declarations.

Use the strict profile YAML subset: no anchors, aliases, merge keys, custom
tags, multi-document markers, duplicate mappings, or unknown fields. Do not put
strategy, generated state, secrets, campaign copy, or broad documentation in
the execution profile.

Standalone fixtures should live under `fixtures/` and exercise public runners.
Inline cases remain acceptable where the runtime package already uses them as a
focused evaluator or graph contract, but they should not turn `X.yaml` into a
large scenario archive.

## Catalog and review policy

Capability metadata describes the complete public runner surface of a package,
not only its default runner. `execution` and `completion` state the strongest
effect the package can truthfully close; `requires_adapter` is true when any
public runner crosses an adapter boundary; and `approval` reflects the strictest
human gate required by those runners. The default runner remains the concise
entry path and is reported separately in catalog reviews.

The catalog gate blocks structural dishonesty and unusable packages:

- unresolved or cyclic default closures;
- a claimed adapter with no reachable adapter boundary;
- agent-authored work with no declared artifact packet;
- no executable contract or operation proof;
- a missing product archetype review.

Metadata, provider readback, forward tests, and evidence-depth gaps remain
visible improvement findings. They may prevent a skill from meeting the full
archetype bar without pretending the underlying product capability should be
deleted.

The generated [Core Skill Product Review](core-skill-review.md) records the
current evidence and recommendation for every top-level package. It does not
authorize removal, relocation, or demotion.

Internal packages use two distinct review categories. `internal_fixture`
packages provide deterministic test rails for canonical public skills;
`internal_runtime` packages implement provider-specific execution paths. Both
are evaluated through their parent integration contracts, remain non-public,
and are not exempt from replay, refusal, recovery, or evidence requirements.
