# Skill Quality Standard

Public runx skills are agent execution context. They are not marketing pages,
feature lists, or aspirational roadmap entries. They tell an agent how to carry
out a consequential action thoroughly and safely: what world it is operating in,
what authority it has, what evidence it may trust, what procedure to follow,
when to stop, and what artifact to emit.

The public catalog is reserved for agent-facing capabilities: canonical
governed actions and branded provider/tool facades over those actions. A skill
belongs there only when it gives the agent enough context to execute a
deployable capability through a clear authority, gate, finality, and receipt
story. Internal lifecycle phases, graph stages, fixtures, and runtime paths
are not skills in the final design.

Every public `SKILL.md` must be specific enough that an agent can execute it
without inventing procedure and a reviewer can audit the result without trusting
the agent's prose.

## Required Structure

Each public skill must include these sections:

- `## What this skill does`: the concrete consequential action, what it emits,
  and what it explicitly does not do.
- `## When to use this skill`: legitimate use cases and the stage of the
  workflow where the skill belongs.
- `## When not to use this skill`: near-misses, higher-risk alternatives, and
  cases that require a different gate or human decision.
- `## Procedure`: ordered execution steps, including evidence collection,
  authority checks, validation, and final decision.
- `## Edge cases and stop conditions`: ambiguity, stale inputs, replay risk,
  missing authority, missing evidence, secret exposure, and explicit refusal or
  needs-input behavior.
- `## Output schema`: the structured artifact an agent should return.
- `## Worked example`: at least one happy path and enough contrast to show how a
  refusal or needs-input case is represented.
- `## Inputs`: required and optional inputs with operational meaning.

## Content Bar

- Write for execution. The agent should finish the file knowing the domain
  context, governing constraints, expected evidence, safe actions, unsafe
  actions, and exact output shape.
- Name the authority involved. Do not imply permission from intent alone.
- Name the gate. If a mutation, charge, delegation, or production action can
  happen, state the approval/finality condition that must exist first.
- Name the evidence. Every amount, scope, actor, counterparty, source, and
  recommendation must be traceable to an input, a receipt, policy, or a named
  inference.
- Name the stop condition. Missing price, stale receipt, mismatched scope,
  replay, ambiguous counterparty, and missing owner are not warnings to work
  around; they change the decision.
- Keep raw secrets out. Skills describe redaction and hash/reference behavior;
  they never ask the agent to print tokens, keys, credentials, or unredacted
  payment material into receipts or outputs.
- Preserve domain boundaries. A pricing skill does not verify credentials; an
  auditor does not repair; an overlay generator does not fork the wrapped skill.
- Prefer refusal over retrofitting. If the evidence does not support the action,
  the skill returns `needs_input`, `needs_more_evidence`, `reject`, or
  `refused`, not a best-effort artifact.
- Keep authoring scaffolding out of the public skill. Internal review concepts
  like purpose, audience, artifact contract, evidence bar, and strategic bar
  should be expressed through the operating instructions, examples, and schema
  rather than repeated as a visible rubric.

## Catalog Gate

The public catalog test enforces the required sections for every skill with
`catalog.visibility: public`. Runnable internals belong in owner-local graph
stages at `skills/<name>/graph/<stage>/X.yaml`; they are not hidden catalog
skills and should not carry public-skill documentation requirements.

Public skills must also keep their executable proof outside the manifest. A
public `X.yaml` describes runners and authority. Concrete scenarios live in
standalone `fixtures/*.yaml` files with `kind: skill`, `target: ..`, and one
fixture covering every public runner. Inline `harness.cases` are reserved for
internal evaluator/showcase packages, not public catalog packages.

Public skill packages are not dumping grounds. Keep only files the skill
actually consumes or emits: `SKILL.md`, `X.yaml`, small deterministic runners,
schemas, fixtures, and narrowly scoped `context/` or `references/` files.
Avoid `README.md`, changelogs, generated state, screenshots, logs, hidden
provider config, private examples, and broad strategy docs. If a user-facing
guide is needed, publish it as docs outside the skill package.

## Execution Profile Discipline

Use the term **execution profile** for `X.yaml`. The filename stays `X.yaml` for
v1, but public docs and reviews should describe what it is instead of treating
the letter as the concept.

`X.yaml` owns capability and governance:

- named runners and default runner choice;
- typed runner inputs and outputs;
- model-vs-deterministic step boundaries;
- tool, adapter, context-skill, and graph wiring;
- authority, approval, and receipt-act mappings;
- side-effect posture: read, draft, plan, mutate, send, pay, or manual-gated;
- inline `harness.cases` only for internal evaluator packages.

Author `X.yaml` in the strict profile YAML subset: no anchors, aliases, merge
keys, custom tags, multi-document markers, duplicate mapping keys, or unknown
profile fields. Capability mappings should be explicit at the runner that uses
them.

`X.yaml` must not become the home for long strategy, target registries, campaign
copy, generated state, or secrets. Put operating guidance in `SKILL.md`,
`context/`, or `references/`; put deterministic implementation in `tools/` or
explicit runner files. Doctor and catalog review should treat a bloated,
strategy-heavy profile as a maintainability defect even when it parses.
