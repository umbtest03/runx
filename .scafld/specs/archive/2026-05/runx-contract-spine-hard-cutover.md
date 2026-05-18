---
spec_version: '2.0'
task_id: runx-contract-spine-hard-cutover
created: '2026-05-18T00:00:00Z'
updated: '2026-05-18T16:52:05Z'
status: completed
harden_status: needs_revision
size: large
risk_level: high
---

# Runx contract spine hard cutover

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-18T16:52:05Z
Review gate: pass

## Summary

Lock the enlightened runx contract model before touching implementation code.

This is a hard cutover, not a dual-shape migration. There are no aliases, no
shadow contracts, and no `.v2` contract ids. The implementation must update
runx, Rust contract parity, hosted/cloud consumers, nitrosend, and Aster in one
coherent pass so the product does not carry two vocabularies or two object
models.

The guiding product constraint is "run anything." The contract spine must not
reduce runx to issue intake, support cases, GitHub PR work, deployment checks,
or Aster's proactive loop. Runx exists to answer one question for every agentic
action:

> By what authority, to what end, what happened, and how do I know?

The contract model is that sentence split across the correct runtime objects:

`harness = attenuated authority + governed boundary + sealed proof`

`act = intent + form + closure`

The harness is the recursive governed execution boundary. It is a graph
instantiated under a grant, carrying attenuated authority, hosting decisions and
acts, and sealing to receipts. A receipt is a sealed harness node. A graph
receipt is a parent harness receipt that links child harness receipts. An act is
the intent, form, and closure payload performed inside a harness; it is not
independently signed, independently sealed, or the universal container.

Two constraints are non-negotiable:

- The nitrosend issue-to-pr flow and Aster's proactive loop are dogfood gates.
  They run runx in production. The cutover must preserve behavior in both.
- Skill and marketplace names are product surface and are not governed by the
  internal contract vocabulary. The recognizable issue-to-pr skill keeps its
  product face; the durable contract records an act with a `revision`
  act form.

Consumer repositories are part of the hard cutover, not later compatibility
work. The expected local/CI checkout layout is controlled by
`RUNX_CONSUMER_REPOS_ROOT` and defaults to `/Users/kam/dev`, with nitrosend at
`$RUNX_CONSUMER_REPOS_ROOT/nitrosend` and Aster at
`$RUNX_CONSUMER_REPOS_ROOT/runx/aster`. A cutover may land only as a coupled
operator session across runx, nitrosend, and Aster branches, with the cutover
gate scanning all three roots before merge.

The harness spine fixes the model holes before implementation:

- Harnesses seal immutably to receipts, and later asynchronous checks are
  represented by child or follow-on harness nodes whose contained act has
  `act.form` set to `verification`.
- Intent output, act form, closure, and proof are bound by explicit criterion
  ids on acts, and harness receipts bind those criteria to verification proof.
- Idempotency and revision control are first-class on harness nodes, so retries
  and concurrent executors cannot silently double-run or overwrite a forming
  harness.
- Redaction, signal authenticity, durable data cutover, and dogfood behavior
  gates are part of the contract plan, not follow-up cleanup.
- Authority algebra, harness lifecycle, receipt semantics, act reference
  semantics, and developer-facing ergonomics are part of the shape. They are
  not implementation details left for the first coding pass.

## Locked Terminology

Ratified vocabulary:

- `harness`: the central governed runtime object. It is the recursive boundary
  that carries attenuated authority, hosts decisions and acts, may spawn child
  harnesses, and seals to receipts.
- `authority`: who or what allowed the harness, under what scope, grant, and
  constraints. Authority must be represented as a decidable algebra so child
  harness authority can be proven to be a subset of parent harness authority.
- `act`: contained harness payload. It records intent, form, and closure only. It is
  not independently signed or sealed.
- `intent`: the end the act was meant to serve, why that end is legitimate, and
  the success criteria used to judge it.
- `form`: the kind of act performed. Initial values are `revision`, `reply`,
  `review`, `observation`, and `verification`.
- `closure`: how the act or decision ended, with reason and time. Closure is
  not a generic bucket for domain detail.
- `proof`: the evidence, receipts, artifacts, verification checks, harness seal,
  and signed envelope that let a stranger recompute and trust the harness node.
- `signal`: world-before-action. Something observed, received, watched,
  reported, scheduled, or triggered.
- `decision`: accountable lifecycle choice. It explains why runx should open,
  continue, spawn, defer, close, decline, or keep watching a harness.
- `Reference`: the one pointer primitive. Wire fields are role-named:
  `source_ref`, `signal_ref`, `act_ref`, `artifact_ref`, `receipt_ref`,
  `harness_ref`, `surface_ref`, `deployment_ref`, `target_ref`,
  `opportunity_ref`,
  `selection_ref`, `decision_ref`, and similar.
- `harness_receipt_ref`: the proof-carrying reference to a sealed harness node.
- `act_ref`: only meaningful when it resolves through a harness receipt and an
  `act_id`; it never proves an act by itself.
- `fingerprint`: recomputable identity for a signal-derived or act-derived
  projection.
- `links`: relationship graph between acts, decisions, signals, and adjacent
  durable objects.
- `verification`: checks that prove whether the intended condition holds.
- `artifact`: durable artifact envelope. Do not add a generic execution-payload
  bucket.
- `revision`, `reply`, `review`, `observation`, `verification`: initial act
  forms. `revision` is the repo/workspace/product mutation form because
  it names deliberate bounded alteration rather than generic change. These are
  not top-level contract families and not product names.
- `target`, `opportunity`, `thesis_assessment`, `selection`,
  `skill_binding`, `target_transition_entry`, `selection_cycle`,
  `reflection_entry`, `feed_entry`: Aster contracts promoted from UI-local
  shapes and doctrine projections.
- `mandate`: parked as a possible authority contract. It enters only if runx
  needs a durable grant of purpose with a strict schema.

Naming rule:

- Phases and forms are fields, never peer contracts. Harness seal state is a
  field; `act.form = "verification"`. Do not create phase-named,
  form-named, or report-shaped peer contracts.
- Bare nouns name stateful lifecycle objects: `harness`, `decision`, `signal`,
  `target`, `opportunity`, `thesis_assessment`, `selection`, `skill_binding`,
  `selection_cycle`. `act` is a contained payload noun, not a free-standing
  lifecycle object.
- `_entry` names born-immutable append-only records: `reflection_entry`,
  `feed_entry`, `target_transition_entry`, ledger-style entries.
- A reflection is not a mutable object; it is one append-only learning record.
  Keep `reflection_entry`.
- Public Aster feed records are `feed_entry`, not table vocabulary.
- Host and harness do not collapse. `host_ref` may identify CLI, CI, IDE,
  hosted API, MCP, or SDK caller context. `harness_ref` identifies the bounded
  execution context and enforcement profile applied to the skill, adapter, or
  checker.

Retired vocabulary:

- The old tracker-style central-object noun in any snake, camel, Pascal, or
  kebab spelling.
- Earlier replacement central-object nouns that still name a thing-to-do rather
  than the authorized, intended, provable character of the thing done.
- Case-file nouns rejected for the run-anything product model.
- Framework-owned pointer type names on the wire.
- Process nouns for identity blocks.
- Combined identity-and-relationship blocks.
- GitHub-PR workflow names as durable act-form or contract names. This does
  not retire recognizable skill/product names.
- Generic terminal-state buckets on central control objects.
- Generic execution-payload buckets where an artifact or harness receipt should
  be cited.
- Bare ambiguous role fields when a role-named reference is meant.

## Naming Boundary

There are two naming layers and they do not share rules.

Product layer: skills, marketplace, docs, UI copy.

- Skills are the human-facing unit. They keep recognizable, marketable names.
- The issue-to-pr skill ships on the runx marketplace under a name customers
  recognize. "Turn an issue into a PR" is good product surface and is kept.
- Product names may reference provider workflows because that is what the
  buyer is looking for.

Contract layer: envelope, harness, act payloads, act forms, decision,
signals, references.

- Uses clean internal vocabulary only.
- `revision` is the repo/workspace/product mutation act form and must not
  be replaced by a provider-workflow name.
- A skill is bound to act forms by mapping, not by sharing a name.
- The issue-to-pr skill may run inside a harness whose contained act uses the
  `revision` act form.
- Append-only public records use `_entry`; table jargon is not contract
  vocabulary. Aster public proof records are `feed_entry`.

Rule: a provider-workflow name appearing on a SKILL.md or marketplace listing
is correct product surface. The same name appearing in a contract schema,
act-form enum, or wire field is a defect the cutover gate must reject. The
decoupling is enforced: the vocabulary gate scans contract code and wire
fixtures, never skill product copy. The gate must include a fixture proving
SKILL.md product copy can retain recognizable provider-workflow language while
contract fixtures reject that same language.

## Core Model

The central runtime contract is `harness`. It is not a generic task, case,
ticket, dossier, or workflow object. It begins when runx instantiates a graph
under an admitted grant. It carries authority, policy, scope admission,
runtime constraints, decisions, acts, child harnesses, and seal state. Once a
harness seals, the receipt is the accountable proof record.

`act` remains important, but it is contained by a harness. An act records
intent, form, and closure: what was attempted and what happened. It has no independent
signature, authority lineage, proof block, or lifecycle outside the harness
that sealed it.

The core sequence is:

1. A `signal` is observed.
2. A proactive system may maintain `target`, `opportunity`, and `selection`
   projections.
3. A `decision` opens, declines, defers, or keeps watching a root harness.
4. The root `harness` runs under admitted authority.
5. Harness-local decisions may continue, spawn a child harness, escalate,
   defer, close, or decline.
6. Acts occur inside harness nodes as intent, form, and closure payloads.
7. A harness may spawn child harnesses only with authority that is a provable
   subset of the parent authority.
8. Every harness seals to a receipt, including abnormal or forced seals.
9. Public feeds and audit surfaces project from the sealed harness receipt tree.

This prevents brittleness from returning in cleaner words. Reactive nitrosend
intake and proactive Aster selection are not forced into one domain shape. They
share envelope, Reference, harness, authority algebra, signals, decisions, acts,
verification, artifacts, receipts, and projection rules; their decision
surfaces remain distinct.

Asynchronous verification rule:

- A sealed harness is never reopened.
- A harness receipt may honestly carry pending external criteria.
- Later deployment, health, human-gate, or observation checks are later child or
  follow-on harness nodes with contained acts whose `act.form` is
  `verification`.
- A public "final" view is a projection over the original sealed harness plus
  later verification harness receipts; it is not a mutation of the original
  sealed harness.
- This is the nitrosend dogfood shape: the `revision` form can be proved
  immediately, while deployment verification can be proved minutes later.
- `revision` is the ratified repo/workspace/product mutation form. `change`
  remains a nested revision-domain word (`change_request`, `change_plan`,
  `target_surfaces`), not the act form.

## Contract Envelope

The signed unit is the harness-node receipt. Acts and decisions are contained
payloads sealed by the harness receipt; they are not independently signed.

Harness receipt envelope shape:

```json
{
  "schema": "runx.harness_receipt.v1",
  "id": "hrn_rcpt_123",
  "created_at": "2026-05-18T00:00:00Z",
  "issuer": {
    "type": "local",
    "kid": "key_1",
    "public_key_sha256": "..."
  },
  "signature": {
    "alg": "Ed25519",
    "value": "..."
  },
  "harness": {},
  "seal": {}
}
```

Rules:

- `schema` is the only contract discriminator.
- `schema_version` is not used.
- Top-level `kind` is not used as a contract discriminator.
- `kind` or `type` may classify nested union variants only when that nested
  field has local meaning.
- The harness receipt envelope is signed. Proof is not bolted onto one
  late-stage object.
- The harness node plus seal are the contract-specific body.
- Free-standing acts and decisions do not carry their own signed envelope.
- Mutable projections cite signed harness receipts. The previous signed receipt
  remains evidence; the new projection has its own source refs and watermark.
- No contract id uses `.vN` for any `N >= 2` in this cutover or future
  breaking revisions. New names enter as their first and only live contract id,
  and superseded names are removed rather than aliased.
- Future breaking changes require a newly named schema id and another hard
  cutover, not a hidden dual-shape version suffix.

## Reference

`Reference` is a shared primitive, not a framework-owned wire noun.

```json
{
  "type": "github_issue",
  "uri": "github://runxhq/runx/issues/123",
  "provider": "github",
  "locator": "runxhq/runx#123",
  "label": "open",
  "observed_at": "2026-05-18T00:00:00Z"
}
```

Rules:

- Field names state the role: `source_ref`, `harness_ref`,
  `harness_receipt_ref`, `act_ref`, `receipt_ref`, `surface_ref`,
  `decision_ref`, and similar.
- `type` is a closed controlled vocabulary, ratified in Phase 2:
  `github_issue`, `github_pull_request`, `github_repo`, `slack_thread`,
  `sentry_event`, `signal`, `act`, `receipt`, `graph_receipt`,
  `harness_receipt`, `artifact`, `verification`, `harness`, `host`,
  `deployment`, `surface`, `target`,
  `opportunity`, `thesis_assessment`, `selection`, `skill_binding`,
  `target_transition_entry`, `selection_cycle`,
  `decision`, `reflection_entry`, `feed_entry`, `principal`, `authority_proof`,
  `scope_admission`, `grant`, `mandate`, `credential`, `webhook_delivery`,
  `redaction_policy`, `external_url`.
- Unknown values are rejected at governed boundaries. New types are added by an
  explicit contract revision, never ad hoc, because there is no mixed-shape
  layer.
- `type` is the only role carrier inside arrays where there is no role-named
  field, such as `fingerprint.derived_from`.
- `uri` is the stable address.
- `locator` is provider-native and optional.
- `provider` is optional because not every reference is provider-backed.
- `label` is display text only, never identity.
- All bespoke pointer shapes collapse into `Reference`: evidence refs,
  contained act refs, surface refs, provider refs, source refs, thread refs,
  repo refs, PR refs, deployment refs, and public feed refs.
- `act_ref` is not a standalone proof pointer. A proof-bearing act reference
  must resolve to `{ harness_receipt_ref, act_id }` or to a URI whose resolver
  returns both the sealed harness receipt and the contained act id.
- A projection may display an act id alone only as local display context. Any
  governed decision, feed entry, reflection entry, or API response that relies
  on the act must carry the harness receipt reference that sealed it.

## Signals

`signal` is the observed-input vocabulary.

Signal payloads carry:

- `signal_id`
- `source_ref`
- optional `thread_ref`
- `authenticity`
- `signal_type`: closed controlled vocabulary, ratified in Phase 2:
  `issue_opened`, `issue_comment`, `pull_request_event`, `review_event`,
  `chat_message`, `alert`, `deployment_event`, `schedule_tick`,
  `operator_note`, `system_event`.
- `title`
- `body_preview`
- `observed_at`
- optional `evidence_refs`
- optional provider extension data in a named extension leaf only

Signals do not decide action by themselves. They are evidence for decisions,
projections, and acts.

`authenticity` carries:

- `transport_ref`: delivery or channel Reference, such as webhook delivery,
  Slack event, scheduler run, or operator session.
- `principal_ref`: principal or provider actor when known.
- `verified_by_ref`: verifier principal or service.
- `trust_level`: strict enum, ratified in Phase 2.
- `verified_at`
- optional `signature_refs` and `evidence_refs`

Rules:

- A signal without authenticity may be retained as observation evidence, but it
  cannot authorize an act by itself.
- Webhook, chat, alert, scheduler, and operator-originated signals each get a
  provider-specific verifier under the same authenticity shape.
- Provenance text is not authenticity. Authenticity must cite verifiable
  delivery, signature, token, or operator-session evidence.

## Fingerprint And Links

The previous combined identity/relationship block is dissolved because it did
two jobs.

`fingerprint` is identity:

```json
{
  "algorithm": "sha256",
  "canonicalization": "runx.signal-fingerprint",
  "value": "sha256:...",
  "derived_from": [
    {"type": "signal", "uri": "runx:signal:sig_123"}
  ]
}
```

Rules:

- `fingerprint.value` must be recomputable from `derived_from`.
- Identity is not asserted; it is checkable.
- The canonicalization rule is explicit so Rust, TS, cloud, nitrosend, and
  Aster derive the same value.

`links` is relationship graph:

```json
{
  "duplicate_of": {"type": "act", "uri": "runx:act:act_1"},
  "duplicate_candidates": [],
  "supersedes": [],
  "superseded_by": [],
  "related": []
}
```

Rules:

- Links use `Reference`.
- Candidate arrays do not contain bespoke ids.
- `duplicate_candidates` carries candidate Reference, confidence, observed_at,
  evidence refs, and optional reviewer refs so human dedupe review is preserved
  without reintroducing bespoke id arrays.
- Relationships do not affect recomputable identity.

## Harness

`harness` is the recursive governed boundary. It is the running counterpart of
`graph`, just as `act` is the running counterpart of `skill`. A root harness is
instantiated under a grant. A child harness may be spawned only with authority
that is a provable subset of the parent harness authority.

Harness node payload:

```json
{
  "harness_id": "hrn_123",
  "parent_harness_ref": null,
  "state": "running",
  "host_ref": {"type": "host", "uri": "runx:host:cli"},
  "harness_ref": {"type": "harness", "uri": "runx:harness:local-cli"},
  "authority": {
    "actor_ref": {"type": "principal", "uri": "runx:principal:agent_1"},
    "grant_refs": [],
    "scope_refs": [],
    "policy_refs": [],
    "attenuation": {
      "parent_authority_ref": null,
      "subset_proof": null
    }
  },
  "enforcement": {
    "version": "2026-05-18",
    "enforcement_profile_hash": "sha256:...",
    "sandbox": {
      "profile": "workspace-write",
      "cwd_policy": "workspace",
      "network": "none",
      "filesystem": "workspace_read_artifact_write"
    },
    "redaction_refs": []
  },
  "idempotency": {
    "intent_key": "...",
    "trigger_fingerprint": "sha256:...",
    "content_hash": "sha256:..."
  },
  "revision": {
    "sequence": 1,
    "previous_ref": null
  },
  "signal_refs": [],
  "decisions": [],
  "acts": [],
  "child_harness_receipt_refs": [],
  "seal": null
}
```

Rules:

- Authority lives on harness, not act.
- Proof lives on harness receipt, not act.
- A harness boundary exists where authority is admitted, attenuated, delegated,
  escalated, paused, resumed, or sealed.
- Acts are what happen inside a stable authority boundary.
- Child harness authority must be machine-checkably less than or equal to
  parent harness authority.
- Authority is a decidable algebra with a partial order, not free strings.
- A harness can contain many acts and many decisions.
- A harness seals normally, declined, failed, timed_out, killed, superseded, or
  blocked. Abnormal seals are mandatory receipts, not missing receipts.
- Recompute is recursive per harness node: inputs + decisions + acts + child
  harness receipts + canonicalization produce the same seal.
- A deep harness tree is not globally replayed. Each node is recomputable from
  its declared inputs and child receipt commitments.

## Authority Algebra

Authority is a machine-checkable algebra, not an array of strings.

An authority term owns:

- `term_id`
- `principal_ref`
- `resource_ref`
- `verbs`
- `bounds`
- `conditions`
- `approvals`
- `capabilities`
- `expires_at`
- `issued_by_ref`
- optional `credential_ref`

`verbs` are closed per resource family. `bounds` are structured limits such as
repo path globs, branch patterns, filesystem roots, network destinations,
deployment environments, token audience, max spend, max runtime, max fanout,
and max child depth. `conditions` are decidable predicates over signals,
decisions, host posture, and approval state. `capabilities` cover runtime
enforcement powers such as filesystem read/write, network egress, secret
access, process spawn, provider mutation, and public publication.

Rules:

- Parent-to-child attenuation is a partial order over authority terms.
- A child harness may be admitted only when every child term is less than or
  equal to some parent term after applying bounds and conditions.
- Approval cannot widen authority. Approval may satisfy a condition already
  present in the parent authority.
- Time extension, resource expansion, broader verbs, weaker sandbox/network
  limits, broader credential audience, or higher fanout all require a new root
  grant, not a child harness.
- The subset proof is stored on the child harness and committed in the child
  harness receipt. It cites the parent harness receipt or admitted authority
  reference, the comparison algorithm, and the terms compared.
- Unknown verbs, unknown resource families, free-form trust boundaries, and
  string-only scopes are rejected at governed boundaries.
- Rust and TypeScript must compute the same subset result from the same
  canonical authority payload and fixture set.

## Harness Lifecycle

Harness lifecycle is explicit because the product promise fails hardest when an
agent stalls, dies, or exceeds bounds.

Initial lifecycle states:

- `forming`: intent, decision, and authority admission are being assembled.
- `admitted`: authority subset, policy, and idempotency checks passed.
- `running`: acts may be appended.
- `waiting`: blocked on explicit input, approval, delay, or external signal.
- `delegated`: a child harness is running and this node is awaiting its receipt.
- `sealing`: no new acts or decisions may be appended; receipt material is
  being canonicalized, redacted, and signed.
- `sealed`: immutable normal seal.
- `killed`: abnormal seal after operator or policy kill.
- `timed_out`: abnormal seal after deadline or heartbeat expiry.
- `failed`: abnormal seal after executor, adapter, validation, or policy
  failure.
- `superseded`: immutable seal after a later harness replaces this one.

Rules:

- `sealed`, `killed`, `timed_out`, `failed`, and `superseded` are terminal.
- Every terminal state emits a harness receipt. There is no such thing as an
  unreceipted killed harness.
- A killed, timed-out, failed, or superseded harness receipt must include
  `seal.disposition`, `reason_code`, `summary`, `closed_at`,
  `last_observed_at`, and any partial acts, decisions, child receipts, stream
  hashes, and artifacts available at the point of seal.
- A `waiting` harness records the exact awaited object: input request,
  approval gate, delay, external signal, child harness receipt, or verification
  recipe.
- A `delegated` harness records the child harness receipt refs it requires
  before it can proceed or seal.
- No lifecycle transition is accepted unless it can be derived from the
  previous state, the authority/policy payload, and the cited signal, decision,
  or child receipt.

## Harness Receipt Semantics

Receipt semantics are crisp because `receipt = sealed harness node`.

Signed material:

- harness id and parent harness reference
- schema id and canonicalization id
- admitted authority and child subset proof
- enforcement profile hash and host reference
- decisions and acts contained by the harness
- signal refs, artifact refs, child harness receipt refs, and projection refs
- seal disposition, lifecycle close data, criteria, verification summary, and
  redaction commitments
- issuer, signature algorithm, key id, and signature value

Hash-committed material:

- large artifacts
- stdout/stderr and tool transcripts
- private prompts or model outputs when retention is disallowed
- secret-bearing evidence
- provider payloads too large or sensitive to embed

Public material:

- schema id, receipt id, created/closed times, disposition, public-safe
  summary, public refs, criterion status, verification status, redaction policy
  refs, child receipt refs, and artifact commitments.

Attested-only material:

- provider mutations that cannot be replayed
- external service responses
- human approvals
- hosted worker identity and runtime posture

Rules:

- Recomputable means recomputable from the declared public or private source
  set plus deterministic redaction policy, not global rerun of an agent.
- Each harness receipt can be verified independently, then recursively checked
  against child receipt commitments.
- A graph receipt is not a second shape; it is a parent harness receipt with
  child harness receipt refs and graph-level seal criteria.
- Late verification creates a later verification-form act in a later harness
  receipt. It never edits the earlier receipt.
- The receipt verification API must distinguish `signature_valid`,
  `hash_commitments_valid`, `authority_attenuation_valid`,
  `criteria_bound`, `redaction_valid`, and `external_attestations_present` so
  consumers do not collapse proof into one vague boolean.

## Decision

`decision` is the accountable harness-lifecycle choice. It is not private model
reasoning. It is the public-safe record inside a harness that says why runx
opened, continued, spawned, escalated, deferred, closed, declined, or kept
watching.

Decision payload:

```json
{
  "decision_id": "dec_123",
  "choice": "spawn_child",
  "inputs": {
    "signal_refs": [],
    "target_ref": null,
    "opportunity_refs": [],
    "selection_ref": null
  },
  "proposed_intent": {
    "purpose": "...",
    "legitimacy": "...",
    "success_criteria": [],
    "constraints": []
  },
  "selected_act_id": null,
  "selected_harness_ref": null,
  "justification": {
    "summary": "...",
    "evidence_refs": []
  },
  "closure": {
    "disposition": "closed",
    "reason_code": "selected",
    "summary": "...",
    "closed_at": "2026-05-18T00:00:00Z"
  },
  "artifact_refs": []
}
```

Rules:

- Valid `choice` values are `open`, `continue`, `spawn_child`, `escalate`,
  `defer`, `close`, `decline`, and `monitor`.
- A no-action decision is a first-class governed record. Runx must be able to
  prove why it declined, not only why it acted.
- `justification.summary` is public-safe rationale, not hidden chain-of-thought.
- `selected_act_id` is set only when the decision appends an act in the current
  harness. `selected_harness_ref` is set only when the decision opens or spawns
  a harness.
- A decision can cite Aster target/opportunity/selection projections without
  making those projections acts.
- Decisions are sealed by the harness receipt that contains them. If a decision
  changes, append a later decision and link it through the harness receipt; do
  not mutate the old decision payload.
- `choice: "monitor"` means "continue watching without an immediate external
  act." `act.form: "observation"` means a concrete act performed a watch
  or check. These are intentionally distinct.

## Act

Act payload:

```json
{
  "act_id": "act_123",
  "intent": {
    "purpose": "...",
    "legitimacy": "...",
    "output": {
      "review_packet": "object"
    },
    "success_criteria": [
      {
        "criterion_id": "crit_deployment_health",
        "statement": "Target deployment responds successfully",
        "required": true
      }
    ],
    "constraints": [],
    "derived_from": []
  },
  "form": "revision",
  "summary": "...",
  "closure": {
    "disposition": "closed",
    "reason_code": "revision_published",
    "summary": "Revision was prepared and linked for review.",
    "closed_at": "2026-05-18T00:00:00Z"
  },
  "criterion_bindings": [],
  "source_refs": [],
  "target_refs": [],
  "surface_refs": [],
  "artifact_refs": [],
  "verification_refs": [],
  "harness_refs": [],
  "revision": {
    "change_request": {},
    "change_plan": {},
    "target_surfaces": []
  },
  "performed_at": "2026-05-18T00:00:00Z"
}
```

Rules:

- No act is valid without `intent`.
- No act is valid without `form`, `intent`, and `closure`.
- No act is valid outside a harness.
- An act has no independent `authority`, `proof`, `phase`, or signed envelope.
- `criterion_bindings` connect intent criteria to verification evidence,
  artifacts, and referenced surfaces. The harness receipt proves those
  bindings.
- A criterion can be `verified`, `failed`, `pending`, or `not_applicable`.
  Pending criteria are allowed only when they require asynchronous external
  checks and are listed in the containing harness seal.
- `act.form` selects the strict act form extension. It is not a top-level
  contract taxonomy.
- There is no generic terminal-state bucket and no generic execution-payload
  bucket.

Act reference rules:

- A durable `act_ref` is a compound proof pointer: `harness_receipt_ref` plus
  `act_id`.
- `act_id` uniqueness is scoped to its containing harness unless the URI
  resolver canonically expands it to a harness receipt.
- An API may expose a display-only `act_id`, but any governed write, projection,
  decision, reflection, feed entry, or verification that cites an act must cite
  the sealing harness receipt.
- If a harness is abnormally sealed, contained act refs remain valid but their
  criterion status may be failed, pending, or unknown. Consumers must not infer
  success from the existence of an act ref.
- Public URLs may route by act id for ergonomics, but the backing API resolves
  to the harness receipt and returns the proof context.

Initial `act.form` values:

- `revision`: repo/workspace/product mutation.
- `reply`: governed response without a repo change.
- `review`: governed assessment or approval.
- `observation`: watching or checking a target without immediate mutation.
- `verification`: proving criteria for a prior harness receipt or watched
  condition.

Initial harness `seal.disposition` values:

- `closed`
- `deferred`
- `superseded`
- `declined`
- `blocked`
- `failed`

Rules:

- `act.form`, form details, references, and artifacts record what happened;
  harness `seal` records harness lifecycle close reason and time.
- Failure, block, and decline must carry `reason_code`, `summary`, and
  `closed_at`.
- `seal.disposition` must not be used as a domain result substitute. Domain
  detail belongs under the strict act-form schema.

## Criterion Binding

Intent output, act form, closure, and harness proof are welded by criterion ids.

Rules:

- Every success criterion has a stable `criterion_id`.
- Every sealed harness receipt has `seal.criteria[]` with exactly one entry for
  each criterion id introduced by contained acts.
- Every verification check cites the criterion ids it verifies or fails.
- An act with `act.form: "verification"` that satisfies a pending criterion
  must cite:
  `harness_ref` or `harness_receipt_ref`, `act_id`, `criterion_ids`,
  `verification.checks[]`, and evidence refs.
- Public projections may summarize verification state, but the underlying
  harness receipt tree must preserve the criterion-level bindings.
- It is invalid to mark verification passed without identifying which intended
  criterion passed.

## Verification Act Form

A verification act form is an act with `act.form: "verification"`.

It owns:

- `harness_ref` or `harness_receipt_ref`
- optional `act_id`
- `criterion_ids`
- `checked_refs`
- `verification`
- `observed_refs`
- optional `deployment_ref`

Rules:

- Verification act forms are recorded by child or follow-on harness nodes. They
  never mutate the harness receipt they verify.
- Nitrosend deployment checks, delayed health checks, human gates, and
  scheduled monitors use this shape.
- If verification fails or is missing, the verification-form act records the
  failed or pending criterion ids and the public projection keeps the earlier
  harness open for operator attention without rewriting its receipt.

## Revision Act Form

`revision` is an act form on an act, not a separate routing packet and not
the universal model.

It owns:

- `change_request`
- `change_plan`
- `target_surfaces`
- `invariants`
- `verification`
- `handoff_refs`
- optional provider-specific revision refs such as PRs

Rules:

- GitHub PR is one revision mechanism, not the contract name.
- Target surfaces are typed references with mutability.
- Change request and change plan are strict schemas, not opaque blobs.
- Verification belongs here only when it is form-specific; harness seal proof
  carries the summarized truth.

## Verification And Artifacts

`verification` records proof state:

```json
{
  "status": "passed",
  "checks": [
    {
      "check_id": "check_http_health",
      "criterion_ids": ["crit_deployment_health"],
      "status": "passed",
      "evidence_refs": []
    }
  ],
  "verified_at": "2026-05-18T00:00:00Z",
  "evidence_refs": []
}
```

Rules:

- Verification is not an execution payload.
- Execution material is an artifact, receipt, or harness receipt reference.
- Deployment health, tests, human gates, and public checks are verification
  checks, not terminal-state variants.
- `status` is a strict enum. It must not become an unbounded status bucket.
- `checks[].criterion_ids` is mandatory for checks that claim to satisfy act
  intent. A check that cannot bind to intent is evidence, not proof of success.

Artifacts carry produced outputs:

- files
- patches
- messages
- reviewer packets
- public feed projections
- machine-readable summaries

Artifacts are envelopes with their own schema and References back to the act,
decision, signal, or projection that produced them.

## Harness Enforcement

`harness` names the recursive enforced execution boundary, not the host
environment.

Host examples:

- CLI
- CI
- IDE
- hosted API
- MCP
- SDK caller

Harness examples:

- local CLI sandbox profile
- hosted agent-step boundary
- deterministic tool runner
- verification checker
- approval gate enforcement shell

Harness enforcement owns:

- `harness_ref`
- `version`
- `enforcement_profile_hash`
- optional `enforcer_ref`
- `sandbox.profile`
- `sandbox.cwd_policy`
- `sandbox.network`
- `sandbox.filesystem`
- `redaction_refs`
- stdout/stderr hash commitments when streams are captured
- optional receipt refs that attest the harness setup or teardown

Rules:

- Authority declares permission; harness enforces permission; harness receipts
  attest the enforcement.
- `harness_ref` is a Reference role field. Do not encode harness identity as
  free-form sandbox metadata.
- Harness is not a peer act family and not an act form. It is the runtime
  boundary that contains acts.
- A harness has versions and enforcement profiles. A harness receipt pins the
  concrete version and profile hash it used.
- A receipt records harness enforcement for a node and links to child harness
  receipts.
- Host transport is not harness enforcement. A hosted worker, local CLI, or IDE
  host can all invoke the same harness profile.

## Harness Replay Mode

The existing `runx harness <path>` CLI verb is retained and reinterpreted as
deterministic replay mode for the governed harness spine.

Rules:

- Production mode runs a harness against live adapters and seals to a receipt.
- Replay mode runs the same canonical harness contract against recorded
  fixtures and asserts the sealed receipt and output.
- The fixture runner is not a second concept named harness. It is the harness
  in replay mode.
- Replay fixtures must expand into canonical harness, decision, act, authority,
  verification, artifact, and harness receipt shapes before governed execution.
- Byte-identical replay is scoped to one contract shape. TS-vs-Rust equality is
  measured after the hard cutover against the canonical harness receipt shape,
  not across retired receipt contracts.
- `rust-harness` ports replay mode. It must not preserve retired receipt shapes
  or create a parallel `replay` verb to avoid the terminology collision.
- Publish-time harness coverage is a replay-mode proof that a skill has at
  least one deterministic governed example.

Package disambiguation:

- `@runxhq/contracts` owns the canonical harness, act, decision, signal,
  Reference, artifact, verification, redaction, and harness receipt schemas.
- `@runxhq/runtime-local/harness` owns the local execution loop that runs
  inside a governed harness: agent hooks, runner orchestration, publish-time
  checks, framing/quality checks, and MCP/A2A fixture adapters. It is runtime
  code, not a second contract namespace.
- Runtime-local harness modules must import contract shapes from
  `@runxhq/contracts`. They must not define a second harness wire shape.
- Import paths keep the boundary explicit: contract envelopes come from
  `@runxhq/contracts`; local execution helpers come from
  `@runxhq/runtime-local/harness`.

## Redaction And Public Projection

Public proof must be recomputable without leaking secrets.

Redaction payloads carry:

- `policy_ref`
- `redacted_fields`
- `hash_commitments`
- `canonicalization`
- `performed_by_ref`
- `performed_at`

Rules:

- Reprojection from public material means "recompute the public projection from
  cited sources plus deterministic redaction policy," not "publish every raw
  source byte."
- Omitted secret material must leave a hash commitment when the value matters
  to proof.
- Redaction is deterministic and schema-backed. Ad hoc string deletion is not a
  governed projection.
- Public feed entries cite redaction policy and source refs so a stranger can
  verify what was disclosed, what was omitted, and which hash commitments bind
  the omitted material.
- Secret-bearing artifacts may remain private; their public projections must
  still be sealed and checkable through commitments.

## Evidence Material

Evidence is not a standalone bundle-shaped control contract in the new spine.
It is expressed through role-named References, artifacts, redaction policy,
signals, verification checks, and harness receipt commitments.

Rules:

- Source material cited by a signal, decision, act, verification check,
  artifact, reflection entry, or feed entry uses `evidence_refs`.
- Evidence hydration state is represented as verification/check readiness or
  artifact availability, not as a separate issue-control object.
- Redaction state is represented by `redaction_refs`, redaction policy refs,
  and hash commitments on artifacts or harness receipts.
- Provider-specific evidence details live in typed extension leaves on the
  artifact or signal that owns them.
- A collection of evidence may be an artifact projection when a consumer needs
  a human-readable packet, but it is not an authority object, not an act, and
  not a peer contract family.
- Rust parity for evidence is therefore part of Reference, artifact,
  verification, redaction, signal, and harness receipt parity.

## Aster Contracts

Aster does not get squeezed into `act` or `harness`.

Contracts to promote:

- `target`: durable thing that could be watched or improved.
- `opportunity`: discovered possible valuable act against a target.
- `thesis_assessment`: assessed fit against Aster's doctrine and current thesis.
- `selection`: ranked candidate or slate position consumed by decision. It is
  not the accountable decision.
- `skill_binding`: runx skill binding Aster may choose to exercise.
- `target_transition_entry`: append-only lifecycle transition for a target.
- `selection_cycle`: durable selector cycle record including no-action cycles.
- `decision`: accountable decision connecting opportunity to act or no-action.
- `reflection_entry`: born-immutable learning record after action or no-action.
- `feed_entry`: born-immutable public projection of Aster's proof surface.

Relationships:

- A `target` may produce many `opportunity` records.
- A `selection` selects or orders opportunities.
- A `decision` may cite target, opportunity, and selection.
- A `decision` may open a harness, append an act inside a harness, decline,
  defer, or keep watching.
- A `reflection_entry` may reference target, opportunity, selection, decision,
  harness receipt, contained act, verification, and artifacts.
- Public feed entries are projections over harness receipt trees.

This preserves Aster's proactive decision model and prevents `harness` or `act`
from becoming a generic container for everything Aster knows.

Aster lifecycle requirements:

- `target` carries lifecycle state, authority refs, fingerprint, cooldown, and
  source refs.
- `opportunity` carries target ref, value/risk assessment, freshness, dedupe
  fingerprint, and duplicate candidates.
- `thesis_assessment` carries score, rubric refs, proof strength, authority cost,
  tractability, and evidence refs.
- `selection` carries ranked candidate refs, rank, score, reason, cooldown
  constraints, and source refs. It proposes order; it does not authorize action.
- `skill_binding` carries skill ref, scope family, allowed act forms,
  verification recipe refs, and authority requirements.
- `target_transition_entry` records lifecycle transitions with source refs and
  receipts.
- `selection_cycle` records selector inputs, ranked slate, chosen selection or
  no-action decision, budget bucket, and harness receipt refs.
- `decision` consumes selection and records the accountable harness/act,
  no-action, defer, or monitor choice.
- `reflection_entry` carries `reflection_id`, target/opportunity/selection/
  decision refs, optional harness receipt refs, optional contained act ids,
  `lesson`, `evidence_refs`, and `recorded_at`.
- `feed_entry` carries public-safe summary, cited harness receipt refs,
  contained act ids, decision ids, verification refs, redaction policy ref, and
  display ordering.

The Aster reset doctrine requires target lifecycle, dedupe enforcement,
cooldown computation, and replayable selector state. These are contract
requirements, not UI model preferences.

## Authority Lineage

Harnesses carry authority references:

- `actor_ref`: principal that opened or operated the harness.
- `authority_proof_refs`: authority proof references.
- `scope_refs`: scope admission references.
- `grant_refs`: concrete grants or credentials.
- optional `mandate_ref`: reserved for a future mandate contract.

Rules:

- Authority is not only reconstructable by joins through proof artifacts.
- Signals carry provenance; harnesses carry authority. Acts and decisions are
  interpreted in the authority context of their containing harness.
- Authority references must use first-class Reference types: `principal`,
  `authority_proof`, `scope_admission`, `grant`, `credential`, and `mandate`.
  Do not encode runx principals as generic URLs.
- Child harness authority must include a machine-checkable subset proof against
  parent harness authority.
- Free-form trust boundaries and scope strings are invalid at governed
  attenuation boundaries.
- `mandate` is introduced only if it becomes a durable purpose/authority
  contract with a strict schema and signed envelope.

## Projection Discipline

Durable state contracts are projections over signals, harness receipts,
contained decisions, contained acts, artifacts, authority, and prior
projections.

Rules:

- A projection carries `projector_id`, `source_refs`, and `watermark`.
- Reprojection from the cited sources must produce the same payload.
- Harness tree reprojection is recursive per node: a harness receipt is
  recomputable from declared inputs, decisions, acts, child harness receipts,
  canonicalization, and redaction commitments.
- Public reprojection may use deterministic redaction. In that case the
  projection carries redaction policy ref and hash commitments, and the
  reproducible payload is the redacted public payload.
- State transition history is not duplicated as an unrelated mutable log when
  harness receipts already record the transition evidence.
- If a transition log is needed for human reading, it is a projection with
  source refs, not a second source of truth.

## Source Of Truth And Drift Control

The tractability problem is part of the design.

Rules:

- The source of truth for contract structure is TypeBox under
  `oss/packages/contracts/src/schemas/**`.
- `oss/scripts/generate-contract-schemas.ts` generates
  `oss/schemas/*.schema.json` from the TypeBox source. `pnpm
  contracts:schemas:generate` writes artifacts; `pnpm contracts:schemas:check`
  fails if generated artifacts are stale.
- Rust serde structs live in `oss/crates/runx-contracts/src/**` and are not
  allowed to drift by review convention. Every Rust governed shape is pinned by
  TypeBox-generated JSON fixtures from
  `oss/scripts/generate-rust-contract-fixtures.ts`, fixture key-order checks,
  unknown-field rejection tests, and Rust fixture parity tests.
- Phase 2 must land the schema-check script, generated schema artifacts,
  generated fixture corpus, and Rust parity coverage before any hosted/cloud
  consumer persists the new surface.
- A future Rust type generator may replace hand-authored serde structs, but it
  is not a compatibility mechanism and must consume the same TypeBox source or
  generated JSON schema artifacts. Until then, a Rust field is not accepted
  unless a TypeBox-generated fixture and unknown-field rejection test exercise
  it.
- Rust is a first-class contract consumer. It rejects unknown fields at governed
  boundaries.
- Fixture parity is required for every governed contract.
- CI enforces generated schema freshness, Rust fixture freshness, fixture key
  order, and forbidden retired vocabulary.
- Consumer repos participate in the cutover gate: nitrosend and Aster must not
  carry retired central-object names or old pointer shapes after the cutover.
- Contract drift gates scan active code and fixtures for `.v2`, retired aliases,
  retired central nouns, generic terminal-state buckets, and framework-owned
  pointer wire names.
- Proof gates verify harness receipt signatures, tamper detection, recursive
  recomputation, authority attenuation, criterion binding, idempotency
  rejection, abnormal seal emission, and unknown-field rejection in Rust and TS.

## Cutover Deployment And Durable Data

Hard cutover includes durable data, not only code.

Rules:

- Before deployment, hosted intake is drained or paused so no in-flight record
  is written with retired vocabulary during the cut.
- Existing durable rows are transformed, archived, or deliberately dropped by a
  one-way migration. No read-time alias layer is allowed.
- The migration emits sealed migration harness receipts or artifacts that cite
  source rows, target harnesses/acts/decisions/signals, and hash commitments for
  audit.
- Dedupe keys move to `fingerprint` and harness `idempotency`; transition
  history moves to harness receipts or projections.
- Rollback is a full deployment rollback plus data restore. It is not a
  mixed-shape mode.

Execution requirements:

- The deployment operator records a pre-cut snapshot of hosted receipt,
  harness, dedupe, and public projection stores.
- Hosted intake is paused before migration and remains paused until final
  cutover gates pass.
- The migration script is one-way in code, but rollback keeps the pre-cut
  snapshot and row-map artifact so a full data restore is possible.
- Migration receipts are sealed harness receipts with `act.form =
  "verification"` or `act.form = "observation"` and cite the source row hash,
  target harness receipt, target decisions/acts/signals, and redaction policy.
- The migration gate proves no retired-vocabulary rows remain in the target
  store and every migration receipt validates against `runx.harness_receipt.v1`.

## Cross-Repo Coordination

The hard cutover spans three working trees:

- runx: canonical contracts, Rust parity, cloud/API/UI storage, and Scafld spec.
- nitrosend: pinned runx consumer, issue intake, PR revision, deployment
  verification, and comment posting dogfood.
- Aster: proactive loop, public feed, doctrine/runtime projections.

Rules:

- The cutover lands through coupled branches in one operator session. The runx
  branch cannot merge as "done" until nitrosend and Aster branches pass their
  consumer gates against the same runx workspace.
- `RUNX_CONSUMER_REPOS_ROOT` names the parent directory for consumer repos.
  Gates fail with an actionable missing-repo message when required consumers
  are absent.
- `RUNX_CUTOVER_EXTRA_ROOTS` is populated from that layout and scanned by the
  same retired-vocabulary gate used for runx.
- Package pins must point at the cutover contract build. Consumers may not pin
  a pre-cut `@runxhq/contracts` package or runx SHA after their cutover branch
  is marked ready.
- `RUNX_CUTOVER_OSS_REF` is the expected runx OSS commit for pinned consumers.
  Nitrosend workflow `RUNX_REF` values must match it. Aster may instead use a
  sibling workspace bridge such as `--runx-root`/`RUNX_BIN`; if it adopts a
  package pin later, the same cutover-build rule applies.
- The consumer coordination gate checks both retired vocabulary and these
  pin/bridge rules before the cutover is marked ready.
- Rollback means reverting all three branches plus data restore. No consumer
  keeps a compatibility shim or alias layer.

## Consumer Impact

Nitrosend:

- Issue, Slack, deployment, and alert inputs become `signal` records.
- Issue intake becomes `decision` selecting a harness, act, no-action, defer, or
  monitor choice.
- Issue-to-pr runs inside a harness with an act using the `revision` form.
- The target dispatcher emits later verification harness nodes for deployment
  checks, using criterion ids from the earlier harness receipt.
- PR, deployment, repo, issue, and surface pointers use `Reference`.
- Redelivered webhooks and rerun comments use idempotency to avoid duplicate
  harnesses or acts.

Aster:

- Target/opportunity/thesis-assessment/selection/skill-binding/
  target-transition-entry/selection-cycle/reflection-entry/feed-entry objects
  become contracts.
- Selection feeds decision; decision may open a harness or append an act but is
  not itself an act.
- Feed entries cite harness receipts, contained act ids, contained decision ids,
  verification evidence, and redaction policy.

Cloud:

- Routes and storage use harness, act, decision, signal, and Aster control
  contracts in the same hard cutover.
- Knowledge/public activity/feed projections use `Reference` and signed
  projection envelopes.

Rust:

- Contract structs are generated or centrally derived.
- Manual drift between TS and Rust is not allowed to grow with the new surface.
- Unknown fields are denied on every governed contract boundary.
- Fixture parity includes root harness, child harness, abnormal harness seal,
  contained act, no-action decision, revision form, receipt linkage,
  authority attenuation, and the full Aster control object set.

## Developer Ergonomics

The internal shape can be rigorous without making every user spell every
contract. The public SDK and CLI should expose a small action surface and let
runx create the harness machinery.

Required ergonomic surfaces:

- `run this skill under this grant`
- `open harness from signal`
- `append act`
- `spawn child harness`
- `wait for approval/input`
- `seal harness`
- `verify receipt`
- `project feed entry`

Rules:

- Simple local runs still feel like invoking a skill. The harness, decision,
  contained act, and receipt objects are produced underneath and inspectable
  afterward.
- Hosted APIs may accept a compact command payload only if it deterministically
  expands to canonical harness/decision/act contracts before persistence or
  execution.
- Ergonomic wrappers must never serialize retired vocabulary, aliases, or
  provider-workflow contract names.
- Every ergonomic API has a contract readback endpoint that returns the
  canonical harness receipt, contained decisions, contained acts, and cited
  artifacts.
- Docs and SDK examples teach the short path first, then show the proof path.
  This keeps "run anything" approachable without weakening the governed core.

## Non-Goals

- No dual-shape endpoints.
- No retired vocabulary aliases.
- No `.vN` contract suffixes for `N >= 2`.
- No case-file central contract.
- No generic terminal-state replacement that becomes the same overloaded field.
- No Aster model collapsed into act.
- No replacement of the receipt evidence primitive.
- No mutation of sealed harness receipts for late verification.
- No proof claim that is not bound to intent criteria.
- No public projection that redacts by unverifiable ad hoc deletion.
- No provider-specific fields outside typed extension leaves.
- No broad implementation before this design survives hardening.

## Acceptance

Profile: strict

Validation:
- [x] `gate_1` command - This design spec validates.
  - Command: `SCAFLD_BIN="${SCAFLD_BIN:-/Users/kam/dev/0state/scafld/dist/scafld_2.4.4_darwin_arm64}"; "$SCAFLD_BIN" validate runx-contract-spine-hard-cutover --root .`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-43
- [x] `gate_2` command - Retired central-object vocabulary is absent from active contract code after implementation.
  - Command: `cd .. && pnpm cutover:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-44
- [x] `gate_3` command - No contract id contains `.vN` for `N >= 2`.
  - Command: `cd .. && ! rg -n "runx\\.[a-z0-9_.-]+\\.v[2-9][0-9]*" oss/schemas oss/packages oss/crates cloud`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-45
- [x] `gate_4` command - Generated schemas are fresh after implementation.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-46
- [x] `gate_5` command - Rust contract fixtures are fresh after implementation.
  - Command: `pnpm fixtures:contracts:check && pnpm fixtures:contracts:keys`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-47
- [x] `gate_6` command - Rust contracts reject unknown fields.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-48
- [x] `gate_7` command - Nitrosend and Aster pass the external cutover vocabulary and consumer coordination gates.
  - Command: `cd .. && export RUNX_CONSUMER_REPOS_ROOT="${RUNX_CONSUMER_REPOS_ROOT:-/Users/kam/dev}" && export RUNX_CUTOVER_OSS_REF="${RUNX_CUTOVER_OSS_REF:-$(git -C oss rev-parse HEAD)}" && test -d "$RUNX_CONSUMER_REPOS_ROOT/nitrosend" && test -d "$RUNX_CONSUMER_REPOS_ROOT/runx/aster" && RUNX_CUTOVER_EXTRA_ROOTS="$RUNX_CONSUMER_REPOS_ROOT/nitrosend:$RUNX_CONSUMER_REPOS_ROOT/runx/aster" pnpm cutover:check && node scripts/check-contract-cutover-consumer-pins.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-49
- [x] `gate_8` command - Nitrosend behavior replay proves issue intake, PR revision, delayed deployment verification, and comment posting still match dogfood expectations.
  - Command: `RUNX_CONSUMER_REPOS_ROOT="${RUNX_CONSUMER_REPOS_ROOT:-/Users/kam/dev}"; node --test "$RUNX_CONSUMER_REPOS_ROOT/nitrosend/scripts/issue-intake.test.mjs" "$RUNX_CONSUMER_REPOS_ROOT/nitrosend/scripts/runx-target-outcome.test.mjs"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-50
- [x] `gate_9` command - Aster feed behavior proves feed entries cite harness receipts, contained acts/decisions, verification evidence, and redaction policy.
  - Command: `pnpm test:fast -- packages/contracts/src/index.test.ts -t "Aster feed entry proof bindings"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-51
- [x] `gate_10` command - Proof integrity tests verify harness receipt signatures, tamper detection, recursive recomputation, authority attenuation, criterion binding, idempotency rejection, abnormal seals, and unknown-field rejection.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts && pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-52
- [x] `gate_11` command - Product naming boundary test proves SKILL.md marketplace language is allowed while contract fixtures reject provider-workflow act-form names.
  - Command: `cd .. && pnpm cutover:check && node scripts/check-contract-cutover-fixtures.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-53
- [x] `gate_12` command - Ergonomic API expansion tests prove compact skill-run inputs persist only canonical harness, decision, act, and harness receipt shapes.
  - Command: `cd ../cloud && pnpm test -- packages/api/src/index.test.ts -t "compact harness readback"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-54

## Phase 1: Harden The Model

Status: completed
Dependencies: none

Objective: Prove the harness spine does not collapse runx into one brittle

Changes:
- Challenge harness recursion against reactive nitrosend and proactive Aster.
- Challenge `decision` across open, continue, spawn, escalate, defer, close, decline, and monitor decisions.
- Challenge asynchronous verification using nitrosend merge-then-deploy timing.
- Challenge intent-output/act-form/closure/harness-proof criterion binding.
- Challenge idempotency and forming-harness revision control under webhook replay and executor retry.
- Challenge signed harness receipt and projection discipline against proof artifacts, evidence bundles, handoffs, and public feed entries.
- Challenge authority as a decidable algebra and abnormal seal semantics.
- Challenge act reference resolution through harness receipts.
- Challenge ergonomic APIs so the canonical shape does not make simple skill invocation brittle.
- Decide and prove the contract source-of-truth/codegen path before expanding the new surface.

Acceptance:
- [x] `ac1_1` command - Model hardening outcomes are incorporated and deterministic design gates pass without another harden loop.
  - Command: `SCAFLD_BIN="${SCAFLD_BIN:-/Users/kam/dev/0state/scafld/dist/scafld_2.4.4_darwin_arm64}"; "$SCAFLD_BIN" validate runx-contract-spine-hard-cutover --root . && pnpm contracts:schemas:check && cd .. && pnpm cutover:check && node scripts/check-contract-cutover-fixtures.mjs && cd cloud && node scripts/check-harness-data-cutover.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6

## Phase 2: Contract Source And Shared Primitives

Status: completed
Dependencies: phase1

Objective: Establish harness receipt envelope, Reference, fingerprint, links,

Changes:
- Ratify the TypeBox-first contract source under `oss/packages/contracts/src/schemas/**`.
- Add `contracts:schemas:generate` and `contracts:schemas:check` scripts.
- Add shared harness receipt envelope contract.
- Add authority algebra and subset proof contract.
- Add shared Reference contract.
- Add harness contract.
- Add signal contract.
- Add fingerprint and links contracts.
- Add verification and artifact contracts.
- Add generation/parity gates.
- Generate JSON Schema artifacts and Rust contract fixtures from the TypeBox source and exercise them from Rust with unknown-field rejection.

Acceptance:
- [x] `ac2_1` command - TS schema and Rust fixture checks pass.
  - Command: `pnpm contracts:schemas:check && pnpm fixtures:contracts:check && cargo test --manifest-path crates/Cargo.toml -p runx-contracts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-11

## Phase 3: Harness, Decision, Act, And Act Forms

Status: completed
Dependencies: phase2

Objective: Replace the current central object with harness, contained acts,

Changes:
- Add harness contract with forming/running/sealed states, child harness refs, authority attenuation, idempotency, and abnormal seal semantics.
- Add contained act payload contract with intent, form, closure, and role-named references.
- Add act-reference resolver semantics: every proof-bearing act ref resolves through a harness receipt and contained act id.
- Add decision payload contract with open/continue/spawn/escalate/defer/close/ decline/monitor decisions.
- Add the `revision` form detail.
- Add reply/review/observation form details only where needed.
- Replace hosted/cloud route and persistence names.
- Update nitrosend hard-cut consumer surfaces.
- Add fixture-driven tests for product naming boundary and compact hosted harness readback.

Acceptance:
- [x] `ac3_1` command - runx, cloud, and nitrosend focused tests pass.
  - Command: `cd .. && pnpm typecheck && pnpm --dir oss fixtures:contracts:check && export RUNX_CONSUMER_REPOS_ROOT="${RUNX_CONSUMER_REPOS_ROOT:-/Users/kam/dev}" && node --test "$RUNX_CONSUMER_REPOS_ROOT/nitrosend/scripts/issue-intake.test.mjs" "$RUNX_CONSUMER_REPOS_ROOT/nitrosend/scripts/runx-target-outcome.test.mjs"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16

## Phase 4: Durable Data Cutover

Status: completed
Dependencies: phase3

Objective: Move hosted durable data to the harness spine without read-time

Changes:
- Add hosted intake drain/pause procedure.
- Add one-way migration from retired durable rows to harness receipts, decisions, acts, signals, projections, fingerprints, and idempotency keys.
- Emit migration harness receipts or artifacts with source-row hashes and target refs.
- Keep a pre-cut snapshot and row-map artifact for full restore rollback.
- Add tests proving no retired-vocabulary rows remain and migration receipts validate.
- Add `cloud/scripts/check-harness-data-cutover.mjs` as a static cutover gate proving the harness receipt tables, dedupe-key table, idempotency-derived dedupe keys, provenance validation, and absence of retired durable table names are present in the hosted data/API implementation.

Acceptance:
- [x] `ac4_1` command - Durable migration and rollback proof gates pass.
  - Command: `cd ../cloud && node scripts/check-harness-data-cutover.mjs && pnpm test -- packages/db/src/postgres.test.ts -t "harness receipt queue packets"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21

## Phase 5: Aster Control Contracts

Status: completed
Dependencies: phase4

Objective: Promote Aster's proactive control and public feed shapes into signed

Changes:
- Add target, opportunity, thesis_assessment, selection, skill_binding, target_transition_entry, selection_cycle, reflection_entry, and feed_entry contracts.
- Update Aster UI/feed model to consume contract-backed entries.
- Update cloud public/feed projection surfaces to cite signed projections.
- Add fixture-driven contract test proving feed entries carry non-null harness receipt refs, contained act refs, decision refs, verification evidence, and redaction policy refs.

Acceptance:
- [x] `ac5_1` command - Aster contract and UI checks pass.
  - Command: `pnpm test:fast -- packages/contracts/src/index.test.ts -t "Aster feed entry proof bindings" && cd ../cloud && pnpm typecheck`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26

## Rollback

This is a design/RFC spec until implementation starts. Rollback before coding
is deleting or rewriting this draft.

Rollback after implementation begins is not mixed-shape support:

- Revert the coupled runx, nitrosend, and Aster branches together.
- Restore hosted data from the pre-cut snapshot captured in Phase 4.
- Use the Phase 4 row-map artifact to verify restored row counts and hashes.
- Re-run the consumer gates against the restored pre-cut branches.
- Do not keep read aliases, translation shims, or dual contract endpoints.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify pass against the prior review's two completion blockers and five non-blocking model gaps. F1 ("Aster Control Contracts unimplemented") is now resolved: target/opportunity/thesis_assessment/selection/skill_binding/target_transition_entry/selection_cycle/reflection_entry/feedEntry schemas exist as full TypeBox contracts in spine.ts (lines 843–1066), with RUNX_CONTRACT_IDS/RUNX_LOGICAL_SCHEMAS entries, validators (validateTargetContract etc.), exports in index.ts, and runxGeneratedSchemaArtifacts entries. F2 ("hollow feed-entry gate") is resolved: index.test.ts:790 defines `it("Aster feed entry proof bindings", ...)` which exercises all nine Aster contracts plus an explicit negative assertion that `feed_entry.harness_receipt_refs: []` throws. F3 (state↔seal cross-field) and F4 (act.form↔revision/verification detail binding) are enforced by assertHarnessSealState (spine.ts:1272) and assertActFormDetails (spine.ts:1248). F5 (envelope outer seal vs harness.seal) is enforced by sameJsonValue in validateHarnessReceiptContract (spine.ts:1186). F6 (.json scanning) is resolved: check-harness-data-cutover.mjs:114 now includes .json in the scanned extensions. F7 (traversal hygiene) is resolved: collect() at line 105 skips .git/node_modules/dist/build/.turbo/coverage. No regressions introduced. Recorded ambient drift (rust aster.rs/aster_control_fixtures.rs, schemas/*.json, generate-rust-contract-fixtures.ts changes) supports the same cutover and is consistent with task-scope intent.

Attack log:
- `oss/packages/contracts/src/schemas/spine.ts (Aster contracts)`: Verify prior blocker F1: target/opportunity/thesis_assessment/selection/skill_binding/target_transition_entry/selection_cycle/reflection_entry/feed_entry exist as full TypeBox schemas with contract IDs, validators, and generated-artifact entries -> clean (All nine schemas defined (spine.ts 843–1066), validators exported, registered in runxContractSchemas and runxGeneratedSchemaArtifacts (index.ts 484–492, 535–543).)
- `oss/packages/contracts/src/index.test.ts`: Verify prior blocker F2: locate the 'Aster feed entry proof bindings' test referenced by gate_9/ac5_1 -> clean (Test exists at line 790; validates non-null harness_receipt_refs, decision_refs, act_refs (compound), verification_refs, redaction_policy_ref; explicit negative case for empty harness_receipt_refs.)
- `oss/packages/contracts/src/schemas/spine.ts (harnessSchema state↔seal)`: Verify prior F3 cross-field invariant: terminal state requires seal, non-terminal forbids it -> clean (assertHarnessSealState (1272) enforces both directions for sealed/killed/timed_out/failed/superseded.)
- `oss/packages/contracts/src/schemas/spine.ts (actSchema)`: Verify prior F4: act.form revision requires revision detail (and rejects verification detail); verification form requires verification detail (and rejects revision); other forms forbid both -> clean (assertActFormDetails (1248) enforces all three cases; invoked by validateActContract and per-act inside validateHarnessContract.)
- `oss/packages/contracts/src/schemas/spine.ts (harnessReceiptEnvelopeSchema)`: Verify prior F5: outer envelope seal must match inner harness.seal -> clean (sameJsonValue deep equality check in validateHarnessReceiptContract (1186) rejects divergence.)
- `cloud/scripts/check-harness-data-cutover.mjs (assertNoRetiredDurableVocabulary)`: Verify prior F6: JSON files included in retired-vocabulary scan -> clean (Extension list at line 114 now contains .json alongside .sql/.ts/.tsx/.js/.mjs.)
- `cloud/scripts/check-harness-data-cutover.mjs (collect)`: Verify prior F7: directory traversal skips .git/node_modules/dist/build/.turbo/coverage -> clean (Skip list at line 105 covers all six entries.)
- `oss/packages/contracts/src/index.ts exports`: Regression hunt: confirm all new Aster contracts surface from package index -> clean (All 9 schemas and validators re-exported (index.ts 304–333) and registered in runxContractSchemas/runxGeneratedSchemaArtifacts.)
- `oss/packages/contracts/src/schemas/spine.ts (decisionChoices)`: Spec compliance: open/continue/spawn/escalate/defer/close/decline/monitor covered -> clean (Eight choices present; spawn surfaced as spawn_child (matches spec line 688).)

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

Tags:

- contracts
- hard-cutover
- rust
- aster
- nitrosend

## Origin

Created by: Codex
Source: operator terminology ratification and brittleness review

## Harden Rounds

### round-1

Status: in_progress
Started: 2026-05-18T12:59:28Z
Ended: none

Checks:
- none

Issues:
- none

### round-2

Status: in_progress
Started: 2026-05-18T13:25:59Z
Ended: none

Checks:
- none

Issues:
- none

### round-3

Status: passed
Started: 2026-05-18T13:49:32Z
Ended: 2026-05-18T13:49:32Z
Verdict: pass
Provider: gemini
Model: gemini-3-flash-preview
Output format: gemini.mcp_submit_harden
Summary: Harden audit complete. The harness spine model is architecturally sound. All checks passed, with missing scripts and external dependencies identified as intentional or environmental. Proceeding with pass verdict as no issues block approval.

Checks:
- path audit
  - Grounded in: code:package.json, code:crates/runx-contracts/Cargo.toml
  - Result: passed
  - Evidence: Internal paths verified via list_directory. External paths noted as future/out-of-scope for local validation.
- command audit
  - Grounded in: code:package.json, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1219
  - Result: passed
  - Evidence: Missing pnpm scripts (cutover:check, contracts:schemas:check) are identified as intentional future deliverables to be added during Phase 2/3 implementation. Verified via spec grep.
- scope/migration audit
  - Grounded in: code:CONVENTIONS.md, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1280
  - Result: passed
  - Evidence: The 'hard cutover' strategy is explicitly supported by CONVENTIONS.md. Design challenges confirmed harness spine model covers all required domain questions.
- acceptance timing audit
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1263
  - Result: passed
  - Evidence: Acceptance gates are logically placed after implementation phases. scafld harden marked as exit gate for Phase 1.
- rollback/repair audit
  - Grounded in: code:CONVENTIONS.md
  - Result: passed
  - Evidence: Standard git revert/restore is the expected rollback path given the 'hard cutover' constraint in CONVENTIONS.md.
- design challenge
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1280
  - Result: passed
  - Evidence: Harness spine model successfully maps to core domain questions.

Issues:
- [medium/advisory] `harden-1` spec_gap:acceptance - Required acceptance scripts are missing from package.json but are intentional future deliverables.
  - Status: open
  - Grounded in: code:package.json, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1219
  - Evidence: Grep results show these patterns only in the spec file, not in package.json or scripts.
  - Recommendation: Explicitly add 'Add cutover:check and contracts:schemas:check scripts to package.json' to the Phase 2 or Phase 3 changes.
- [low/advisory] `harden-2` spec_gap:acceptance - External path dependencies in acceptance criteria.
  - Status: open
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1243
  - Evidence: Absolute paths like /Users/kam/dev/nitrosend are used in gate_8.
  - Recommendation: Parameterize external roots or document the required environment for running these gates.
- [medium/advisory] `harden-3` spec_gap:migration - Missing explicit data migration/cleanup step in implementation plan.
  - Status: open
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1330
  - Evidence: Spec lacks explicit change item for retired issue-control data cleanup.
  - Recommendation: Add a step to Phase 3 for data cleanup/migration script.

### round-4

Status: error
Started: 2026-05-18T14:06:29Z
Ended: 2026-05-18T14:06:29Z
Summary: provider failed: provider failed: provider produced no submission; Gemini must call mcp_scafld_submit_harden exactly once and final text is ignored: ... task_queues:104:5)

Checks:
- none

Issues:
- none

### round-5

Status: needs_revision
Started: 2026-05-18T15:57:50Z
Ended: 2026-05-18T15:57:50Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: The harness-spine model is coherent and the locked terminology is internally consistent, but four spec gaps are large enough to make the cutover non-executable as written: (1) Phase 2 is required to "decide and prove the contract source-of-truth/codegen path before expanding the new surface" but the spec never picks a generator (TypeBox? OpenAPI? Rust-first derive?) — Phase 2's acceptance gate `pnpm --dir oss contracts:schemas:check && fixtures:contracts:check && cargo test` already presumes that decision is in place; (2) Phase 4 declares only `phase2` as a dependency yet its Aster contracts (`decision`, `feed_entry`, `reflection_entry`) cite harness/decision/contained-act shapes only defined in Phase 3 — ordering is wrong; (3) several acceptance gates do not actually assert what their text claims (gate_9, gate_11, gate_12 are passive `typecheck`/`cutover:check` invocations that cannot prove "feed entries cite harness receipts," "SKILL.md product copy allowed while contract fixtures reject," or "compact skill-run inputs persist only canonical shapes"); (4) the "Cutover Deployment And Durable Data" section is normative but no phase task or gate covers the one-way data migration, migration-receipt emission, or pre-deploy drain. Plus two cross-cutting risks: the "harness" name collides with the existing `oss/packages/runtime-local/src/harness/` runner package (the spec only addresses the CLI verb collision, not the package collision), and the cutover crosses three repos (oss, nitrosend, aster) with no described synchronization mechanism. None of these is fatal to the design — they are spec holes that will be paid for at implementation time. Recommend revising before approval.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/package.json:11, code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover.mjs:1, code:/Users/kam/dev/runx/runx/oss/package.json:38, code:/Users/kam/dev/runx/runx/oss/crates/runx-contracts/Cargo.toml:1, code:/Users/kam/dev/nitrosend/scripts/runx-target-outcome.test.mjs, code:/Users/kam/dev/runx/runx/cloud/packages/api/package.json:2
  - Result: passed
  - Evidence: Workspace-root `cutover:check` script exists at runx/package.json:11. Contract fixture scripts (`fixtures:contracts:check`, `contracts:schemas:check` via `schemas:generate`) exist in oss/package.json. `runx-contracts` crate exists. Nitrosend test paths exist: `issue-intake.test.mjs` and one matching `runx-target-*.test.mjs` (`runx-target-outcome.test.mjs`). `@runx/api` package exists in `cloud/packages/api/` (not OSS), so gate_12's `pnpm --filter @runx/api test -- harness` will be invoked from the workspace root. `contracts:schemas:check` is not yet a named script in oss/package.json — only `schemas:generate` exists; this is acknowledged in round-3 harden-1 as an intentional future deliverable.
- command audit
  - Grounded in: spec_gap:acceptance.gate_9, spec_gap:acceptance.gate_11, spec_gap:acceptance.gate_12, code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover.mjs:40
  - Result: failed
  - Evidence: Three acceptance gates do not assert what their text claims. gate_9 text: 'Aster feed behavior proves feed entries cite harness receipts, contained acts/decisions, verification evidence, and redaction policy' — command is `pnpm typecheck`, which only proves the types compile, not that any concrete feed entry cites the required refs. gate_11 text: 'Product naming boundary test proves SKILL.md marketplace language is allowed while contract fixtures reject provider-workflow act-form names' — command is `pnpm cutover:check`, which scans for banned tokens but does not encode a SKILL.md allow-list fixture (`check-contract-cutover.mjs` has a single `banned` array, no per-path allow-list). gate_12 text: 'compact skill-run inputs persist only canonical harness, decision, act, and harness receipt shapes' — command is `pnpm typecheck && pnpm --filter @runx/api test -- harness`, but no Phase plan adds a harness-named test in `@runx/api`. Gate_3 (`rg -n 'runx\.[a-z0-9_.-]+\.v2' oss cloud`) is correct exit_code_nonzero semantics for 'no match → rg exits 1' — that one passes scrutiny.
- scope/migration audit
  - Grounded in: spec_gap:scope.cross_repo, code:/Users/kam/dev/runx/runx/oss/CLAUDE.md, code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover.mjs:5
  - Result: failed
  - Evidence: The 'Cutover Deployment And Durable Data' section mandates pre-deploy drain, one-way migration with sealed migration receipts, dedupe-key rehoming to `fingerprint` + harness `idempotency`, and full deployment rollback with data restore — but no phase in the plan owns any of those steps. Phases 2–4 only add contracts, hosted route renames, and Aster control objects. The data migration is normatively required and operationally unscoped. Additionally, the cutover spans three repository roots (runx oss, /Users/kam/dev/nitrosend, /Users/kam/dev/runx/aster); gate_7 confirms the cutover-check script supports `RUNX_CUTOVER_EXTRA_ROOTS`, but no mechanism is named for landing the consumer-repo edits atomically with the OSS contract change. The spec calls this a 'hard cutover' yet provides no atomic-cross-repo synchronization plan (no submodule pin, no shared release train, no batched PR coordinator).
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase4_dependency, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1411
  - Result: failed
  - Evidence: Phase 4 declares `Dependencies: phase2`, but Phase 4's promoted contracts (`decision`, `feed_entry`, `reflection_entry`, `selection_cycle`, `target_transition_entry`) all cite harness receipts, contained acts, and the `decision` payload — which only land in Phase 3. Aster `decision` is the same accountable lifecycle object defined in Phase 3 'Add decision payload contract with open/continue/spawn/escalate/defer/close/decline/monitor decisions.' Phase 4 cannot complete its acceptance until Phase 3 has landed, making the declared dependency edge incorrect. Phase 1 acceptance (`scafld harden --mark-passed`) is appropriate for a design-only phase. Phase 2 acceptance presumes a contract-codegen pipeline is already in place — but the SoT/codegen path is itself one of Phase 1's listed challenges, and round-3 did not record evidence that this question was resolved.
- rollback/repair audit
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1432, spec_gap:rollback.midcutover
  - Result: failed
  - Evidence: The Rollback section reads: 'Rollback after implementation begins is not mixed-shape support: revert the hard-cut implementation changes together across runx, Rust fixtures, nitrosend, and Aster.' This is correct in spirit for a hard cutover but operationally hollow: no described checkpoint between Phase 2 (primitives merged, schemas regenerated) and Phase 3/4 (consumers swapped). If Phase 3 partial-lands and is reverted, the OSS contracts package will already have shipped the new envelope, harness, signal, fingerprint, and Reference types — yet no plan describes how to roll back the OSS package version, the regenerated Rust fixtures, or the nitrosend/aster pins to a known-good triple. The data-migration rollback ('full deployment rollback plus data restore') has no inverse migration script named, no source/target row mapping retention requirement, and no test gate.
- design challenge
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/index.ts, code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/agent-hook.ts, code:/Users/kam/dev/runx/runx/oss/skills/write-harness/SKILL.md, spec_gap:terminology.harness_collision
  - Result: failed
  - Evidence: The spec promotes `harness` to mean 'the recursive governed execution boundary that carries attenuated authority and seals to receipts.' The Harness Replay Mode section addresses only the existing `runx harness <path>` CLI verb. It does not address that `oss/packages/runtime-local/src/harness/` is the agent-hook runner package — a third, code-level meaning of 'harness' that names a non-contract runtime concern. After the cutover, the import path `@runxhq/runtime-local/harness` could plausibly point to either the agent-hook runner or the new contract envelope. The locked-terminology rules also say `harness_ref` is the bounded execution context distinct from `host_ref`; collapsing that against the existing `harness/` runner package needs an explicit rename or namespacing decision before Phase 2. Separately, the harness receipt envelope sets `schema = runx.harness_receipt.v1` while the cutover bans `.v2` — the spec is silent on how a future revision is permitted under that rule (the gate is too narrow; a future `runx.harness_receipt.v3` would slip through). Finally, the contract source-of-truth/codegen decision (TypeBox-first? OpenAPI-first? Rust-derive-first? hand-authored JSON Schema?) is listed as a Phase 1 challenge but never resolved in the draft body — Phase 2 cannot execute its acceptance gate without it.

Issues:
- [critical/blocks approval] `harden-r5-1` spec_gap - Contract source-of-truth/codegen path is required by Phase 1 but never decided in the draft.
  - Status: open
  - Grounded in: spec_gap:phases.phase1_unresolved, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1342
  - Evidence: Phase 1 'Changes' list ends with 'Decide and prove the contract source-of-truth/codegen path before expanding the new surface,' and the 'Source Of Truth And Drift Control' section says 'Phase 2 must land one contract source that generates JSON Schema artifacts and Rust serde types before any broad contract expansion. Manual parallel TS/Rust schema evolution is not allowed for the new surface.' The draft body does not pick between TypeBox-first generation (current oss/scripts/generate-contract-schemas.ts approach), OpenAPI-first, a Rust-first serde-derive, or a new IDL. Phase 2's acceptance gate `pnpm --dir oss contracts:schemas:check && pnpm --dir oss fixtures:contracts:check && cargo test --manifest-path oss/crates/Cargo.toml -p runx-contracts` will not be runnable until that pipeline exists; round-3 harden-1 already flagged that `contracts:schemas:check` is not a registered pnpm script.
  - Recommendation: Add a 'Contract Source Of Truth' section to the spec that names exactly one generator pipeline, the source-of-truth file or directory, the TS output and Rust output paths, the canonicalization rule, and the new oss/package.json scripts (`contracts:schemas:check`, `contracts:schemas:generate`). Then either move Phase 2's first acceptance gate behind a new 'Phase 2a: SoT pipeline lands' sub-acceptance or make the pipeline an explicit first Change item in Phase 2.
  - Question: Which generator owns the contract source — TypeBox-first (extend the existing oss/scripts/generate-contract-schemas.ts), an OpenAPI document, Rust serde-derive with a TS exporter, or a fresh IDL — and what is the single source-of-truth path?
  - Recommended answer: Extend the existing TypeBox pipeline (oss/scripts/generate-contract-schemas.ts) as the single source: contract structs live in @runxhq/contracts/src/schemas, generate JSON Schema for runtime validation, and generate Rust serde types via a new oss/scripts/generate-rust-contract-types.ts step that the runx-contracts crate consumes. This is the smallest delta from the current dual-maintenance situation and reuses an already-working generator.
  - If unanswered: Default to extending the existing TypeBox pipeline as above and add a Phase 2 Change item to wire `contracts:schemas:check` into oss/package.json.
- [high/blocks approval] `harden-r5-2` spec_gap - Phase 4 declares only phase2 as a dependency but consumes Phase 3 deliverables (decision, harness, contained acts).
  - Status: open
  - Grounded in: spec_gap:phases.phase4_dependency, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1411
  - Evidence: Phase 4 promotes `decision`, `feed_entry`, `reflection_entry`, `selection_cycle`, and `target_transition_entry` — each of which is documented elsewhere in the spec to cite harness receipts, contained act ids, and the `decision` payload. The `decision` contract itself only lands in Phase 3 ('Add decision payload contract with open/continue/spawn/escalate/defer/close/decline/monitor decisions.'). Aster feed entries are specified to 'cite signed harness receipts, contained act ids, decision ids' — none of those references resolve until Phase 3.
  - Recommendation: Change Phase 4 dependency to `phase3` so the dependency graph matches the typed references the Phase 4 contracts must carry. Alternatively, split Phase 3 into 'Phase 3a: harness, contained act, act forms' and 'Phase 3b: decision, hosted/nitrosend cutover' and let Phase 4 depend on 3a.
  - Question: Should Phase 4 depend on Phase 3 (the simpler fix) or should Phase 3 be split so Aster contracts can land in parallel with hosted/nitrosend cutover?
  - Recommended answer: Change `Dependencies: phase2` to `Dependencies: phase3` on Phase 4. Phase 3 is large but indivisible because the hosted/cloud route renames and the nitrosend consumer hard-cut must land with the decision contract; splitting it would reintroduce a mixed-shape window the cutover explicitly forbids.
  - If unanswered: Default to `Dependencies: phase3` on Phase 4.
- [high/blocks approval] `harden-r5-3` acceptance_mismatch - Acceptance gates 9, 11, and 12 do not actually verify the assertions in their descriptions.
  - Status: open
  - Grounded in: spec_gap:acceptance.gate_9, spec_gap:acceptance.gate_11, spec_gap:acceptance.gate_12, code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover.mjs:40
  - Evidence: gate_9 claims to 'prove feed entries cite harness receipts, contained acts/decisions, verification evidence, and redaction policy' but runs `pnpm typecheck`, which only proves the types compile — a feed_entry whose required refs are all `null` would still pass. gate_11 claims to 'prove SKILL.md marketplace language is allowed while contract fixtures reject provider-workflow act-form names' but runs `pnpm cutover:check`, whose script (`scripts/check-contract-cutover.mjs`) has a single global `banned` array and no per-path allow-list for SKILL.md product copy — so it cannot prove the bidirectional rule the spec requires. gate_12 claims compact skill-run inputs persist only canonical shapes but invokes `pnpm --filter @runx/api test -- harness` against `cloud/packages/api/` where no harness-named test is planned in any phase.
  - Recommendation: Either (a) replace each gate with a concrete fixture-driven test that asserts the claim (e.g. a Vitest case loading a feed_entry fixture and asserting required refs are non-null and well-typed; a SKILL.md allow-list check in check-contract-cutover.mjs with a fixture proving recognizable provider names survive in SKILL.md but are rejected inside contract fixtures; a `@runx/api` harness-readback test that round-trips a compact skill invocation into a canonical harness receipt), or (b) restate each gate's description to match what the command actually verifies and add a separate concrete test for the stronger assertion.
  - Question: For each of gate_9, gate_11, gate_12 — do you want to keep the strong assertion text and replace the command with a real fixture-driven test, or weaken the text to match the current command?
  - Recommended answer: Keep the assertion text and add three concrete tests: (1) a Vitest case under oss/packages/contracts asserting feed_entry fixtures carry non-null harness_receipt_refs, decision_id, verification_refs, and redaction_policy_ref; (2) extend check-contract-cutover.mjs with a SKILL.md allow-list path filter and a fixture pair (SKILL.md with provider workflow name → allowed; contract fixture with same name → rejected); (3) add an @runx/api Vitest case asserting a compact skill-run input expands into a canonical harness/decision/act/receipt persistence path.
  - If unanswered: Default to the strong-assertion option above and add the three tests as Phase 3/Phase 4 Change items.
- [high/blocks approval] `harden-r5-4` spec_gap - The 'Cutover Deployment And Durable Data' section is normative but no phase owns the migration work or gate.
  - Status: open
  - Grounded in: spec_gap:phases.data_migration_missing, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1159
  - Evidence: The section mandates: pre-deploy drain or pause of hosted intake; one-way migration of existing durable rows; emission of sealed migration harness receipts citing source rows, target harnesses/acts/decisions/signals, and hash commitments; rehoming dedupe keys to `fingerprint` and harness `idempotency`; full deployment rollback with data restore. Phases 2–4 add contracts and consumer surface edits but do not include a migration step, do not produce migration receipts, and no acceptance gate verifies the migration. Round-3 harden-3 already flagged this; it remains open.
  - Recommendation: Add an explicit 'Phase 3.5: Durable Data Cutover' (depending on phase3) that owns the drain procedure, the one-way migration script, the migration-receipt emission, the dedupe-key rehoming, and a gate command that asserts (a) no retired-vocabulary rows remain in the target store and (b) migration receipts validate against the harness receipt schema. Update the Rollback section with a concrete inverse-migration or restore procedure.
  - Question: Should the durable data migration be a dedicated phase between phase3 and phase4, or rolled into phase3?
  - Recommended answer: Make it Phase 3.5 with its own acceptance gate. Phase 3 is already at the limit of what can land atomically across runx/cloud/nitrosend; bundling durable-row migration with route renames would make rollback materially harder.
  - If unanswered: Default to adding Phase 3.5 with the gate above.
- [medium/advisory] `harden-r5-5` design_risk - Term 'harness' collides with the existing oss/packages/runtime-local/src/harness/ agent-hook runner package.
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/index.ts, code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/agent-hook.ts
  - Evidence: The 'Harness Replay Mode' section addresses only the `runx harness <path>` CLI verb collision. It does not address that `oss/packages/runtime-local/src/harness/` already names a non-contract runtime package (agent-hook.ts, runner.ts, publish.ts) that runs agent hooks at a totally different abstraction level than the new contract `harness`. After cutover, an import like `@runxhq/runtime-local/harness` is ambiguous between 'the agent-hook runner' and 'the recursive governed execution boundary contract envelope.'
  - Recommendation: Add a 'Harness Package Disambiguation' rule to the Locked Terminology section that either (a) renames the existing runner package to `agent-hooks/` or similar, or (b) reserves the unqualified `harness` symbol for the contract envelope and namespaces the runner as `runtime-local/agent-harness` or `runtime-local/skill-harness`. Pick one before Phase 2 lands.
  - Question: Should the existing oss/packages/runtime-local/src/harness/ package be renamed, or should the new contract envelope use a more specific export path inside @runxhq/contracts?
  - Recommended answer: Rename the existing package to `runtime-local/src/agent-hooks/` and reserve `harness` for the contract envelope. The contract spine is the more load-bearing meaning across runx/cloud/nitrosend/aster and should not be relegated to a sub-name.
  - If unanswered: Default to the rename above as a Phase 2 prerequisite.
- [medium/advisory] `harden-r5-6` design_risk - Cross-repo cutover (oss + nitrosend + aster) has no described atomic synchronization mechanism.
  - Status: open
  - Grounded in: spec_gap:scope.cross_repo, code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover.mjs:5
  - Evidence: Gate_7 confirms `RUNX_CUTOVER_EXTRA_ROOTS` lets the cutover-check script scan external roots, but the spec lives in oss/.scafld/specs/drafts/ and only describes OSS phase changes. Phase 3 says 'Update nitrosend hard-cut consumer surfaces' and Phase 4 says 'Update Aster UI/feed model' — both treat external repos as if they could be touched by the OSS spec. No submodule pin, no shared release train, no batched PR coordinator, and no version-coupling mechanism is described. A hard cutover requires either a single PR across three repos (impossible without a monorepo) or a strict version-coupling discipline.
  - Recommendation: Add a 'Cross-Repo Coordination' section that names the synchronization mechanism: e.g. (a) bump @runxhq/contracts to a new major; (b) coordinate three branch PRs (runx/cutover, nitrosend/runx-cutover, aster/runx-cutover) that all land within the same operator session; (c) the cutover gate enforces that the @runxhq/contracts version pinned by nitrosend and aster matches the OSS major. Without this, a 'hard cutover' across three repos is aspirational.
  - Question: What is the concrete cross-repo synchronization mechanism — coupled branch PRs gated by a single operator session, a package version pin enforced by the cutover gate, or something else?
  - Recommended answer: Coupled branch PRs gated by a single operator session, plus an explicit @runxhq/contracts version pin check inside check-contract-cutover.mjs that fails if nitrosend/aster pin a pre-cutover major.
  - If unanswered: Default to the coupled-branch + version-pin enforcement above and add a Change item to Phase 2.
- [low/advisory] `harden-r5-7` spec_gap - The '.v2 ban' rule is too narrow: future revisions still need a versioning policy.
  - Status: open
  - Grounded in: spec_gap:contract_id.future_revisions, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1279
  - Evidence: gate_3 checks `rg -n 'runx\.[a-z0-9_.-]+\.v2' oss cloud` and the spec body says 'No contract id uses .v2 in this cutover. New names enter as their first and only live contract id, and superseded names are removed rather than aliased.' The harness receipt envelope uses `schema = runx.harness_receipt.v1`. The rule prevents `.v2` specifically but says nothing about `.v3`, `.v4`, or any later breaking change. Future revisions need a policy: either rename the contract entirely (different schema id) or accept versioned suffixes under a stricter rule.
  - Recommendation: Add a 'Contract Revision Policy' rule to Locked Terminology: 'Breaking changes to a contract require a new schema id (e.g. `runx.harness_receipt_v2` as a new contract name), not a versioned suffix on an existing schema. The cutover gate bans any `\.v[2-9][0-9]*` suffix on a runx schema id.' Update the gate_3 regex to match the broader pattern.
  - Question: Do you want to ban all `.vN` suffixes (N≥2) on runx schema ids, or only `.v2` for this cutover?
  - Recommended answer: Ban all `.vN` for N≥2. Versioned suffixes encourage a quiet dual-shape escape hatch and the spec already commits to hard cutover semantics for breaking changes.
  - If unanswered: Default to banning all `\.v[2-9][0-9]*` and broaden the gate_3 regex.
- [low/advisory] `harden-r5-8` spec_gap - Acceptance commands embed absolute host paths (/Users/kam/dev/...) making gates non-portable.
  - Status: open
  - Grounded in: spec_gap:acceptance.absolute_paths, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1295
  - Evidence: gate_7 uses `RUNX_CUTOVER_EXTRA_ROOTS="/Users/kam/dev/nitrosend:/Users/kam/dev/runx/aster"` and gate_8 invokes `node --test /Users/kam/dev/nitrosend/scripts/issue-intake.test.mjs /Users/kam/dev/nitrosend/scripts/runx-target-*.test.mjs`. Round-3 harden-2 already flagged this; the gates will only run on the operator's workstation as written. CI cannot run these without path normalization or a sibling-repo convention.
  - Recommendation: Parameterize the consumer-repo paths via an env var with a sensible default (e.g. `RUNX_CONSUMER_REPOS_ROOT=${RUNX_CONSUMER_REPOS_ROOT:-../}`) and update gate_7 and gate_8 to use `$RUNX_CONSUMER_REPOS_ROOT/nitrosend` / `$RUNX_CONSUMER_REPOS_ROOT/aster`. Document the expected layout in the spec.
  - Question: Should gates assume sibling-repo layout via an env var, or should we relocate the gates to scripts that probe for the consumer repos and skip with a clear message when absent?
  - Recommended answer: Sibling-repo layout via $RUNX_CONSUMER_REPOS_ROOT (default `../`), with a clear skip+error-out path if the directory is missing. This keeps the gate runnable in CI as long as the workflow checks out the three repos side-by-side.
  - If unanswered: Default to the sibling-repo env var approach.

### round-6

Status: needs_revision
Started: 2026-05-18T16:11:28Z
Ended: 2026-05-18T16:11:28Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-5's critical/high blockers are resolved in the spec body: TypeBox is named as the single contract source ("Source Of Truth And Drift Control" section), gate_3 was broadened to `\.v[2-9][0-9]*`, Phase 4 dependency was corrected to `phase3`, `RUNX_CONSUMER_REPOS_ROOT` parameterizes consumer paths, gate_11 now invokes `node scripts/check-contract-cutover-fixtures.mjs` (verified — the script enforces SKILL.md product copy AND act schema banning of provider-workflow names), and a Package Disambiguation block was added for the harness collision. Most declared paths and scripts exist on disk (`scripts/check-contract-cutover.mjs`, `scripts/check-contract-cutover-fixtures.mjs`, `oss/package.json#contracts:schemas:check`, `oss/package.json#fixtures:contracts:check`, `oss/crates/runx-contracts`, both nitrosend test files, the aster repo, and the cloud db/api test files). However, three residual gaps remain. The most serious is the same class of issue round-5 raised for gates 9/11/12 — Phase 4's acceptance gate (`cd ../cloud && pnpm test -- packages/db/migrations.test.ts packages/db/postgres.test.ts -t "harness"`) runs harness-receipt-store tests that already pass on `main` (verified via `rg harness` in those files); it does not assert any of Phase 4's substantive deliverables: hosted intake drain, one-way row migration, sealed migration-receipt emission, dedupe-key rehoming to `fingerprint`/`idempotency`, or "no retired-vocabulary rows remain." Second, the Package Disambiguation block "retains" `@runxhq/runtime-local/harness` as "the replay-mode runtime implementation for governed harness fixtures" — but the package's actual contents (`agent-hook.ts`, `runner.ts`, `publish.ts`, `framing-patterns.ts`, `quality.ts`) are not replay-mode infrastructure; the spec defers the actual rename/namespacing decision to "if a future import becomes ambiguous," which is the ambiguity itself. Third, the Cross-Repo Coordination section requires consumer repos to pin the cutover contract build but no gate enforces that pin (gate_7 only scans for banned vocabulary). Additionally, the rationale for Phase 5 depending on Phase 4 (rather than Phase 3) is not stated; Aster shape contracts conceptually depend on harness/decision/contained-act (Phase 3 deliverables) and could land in parallel with the durable migration unless cloud feed-projection surfaces require the migration first.

Checks:
- path audit
  - Grounded in: code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover.mjs:1, code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover-fixtures.mjs:1, code:/Users/kam/dev/runx/runx/oss/package.json:21, code:/Users/kam/dev/runx/runx/oss/package.json:40, code:/Users/kam/dev/runx/runx/oss/package.json:42, code:/Users/kam/dev/runx/runx/oss/crates/runx-contracts/Cargo.toml:1, code:/Users/kam/dev/nitrosend/scripts/issue-intake.test.mjs, code:/Users/kam/dev/nitrosend/scripts/runx-target-outcome.test.mjs, code:/Users/kam/dev/runx/aster, code:/Users/kam/dev/runx/runx/cloud/packages/db/src/migrations.test.ts, code:/Users/kam/dev/runx/runx/cloud/packages/db/src/postgres.test.ts, code:/Users/kam/dev/runx/runx/cloud/packages/api/src/index.test.ts
  - Result: passed
  - Evidence: Every script and path cited in acceptance gates and phase commands was verified on disk: workspace-root `cutover:check` at runx/package.json:11; `check-contract-cutover-fixtures.mjs` exists; `contracts:schemas:check`, `contracts:schemas:generate`, `fixtures:contracts:check`, `fixtures:contracts:keys` are all named scripts in oss/package.json (round-3 harden-1 is therefore resolved); `oss/crates/runx-contracts/Cargo.toml` exists and `oss/crates/Cargo.toml` workspace exists, so `cargo test --manifest-path crates/Cargo.toml -p runx-contracts` resolves from oss/; both nitrosend test files (`issue-intake.test.mjs`, `runx-target-outcome.test.mjs`) exist; `/Users/kam/dev/runx/aster` is a real working tree (the `runx/aster` segment under $RUNX_CONSUMER_REPOS_ROOT=/Users/kam/dev resolves correctly); cloud test files `migrations.test.ts`, `postgres.test.ts`, and `packages/api/src/index.test.ts` all exist. The contract harness primitives are partially in place: `schemas/harness.schema.json`, `schemas/harness-receipt.schema.json`, `schemas/decision.schema.json`, `schemas/act.schema.json`, `schemas/signal.schema.json`, `schemas/reference.schema.json`, and `packages/contracts/src/schemas/spine.ts` already exist.
- command audit
  - Grounded in: code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover.mjs:40, code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover-fixtures.mjs:14, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1362, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1394
  - Result: passed
  - Evidence: Each acceptance gate command parses cleanly relative to its expected cwd (oss/, then `cd ..` to runx workspace root, or `cd ../cloud` to cloud root). gate_3 `! rg -n 'runx\.[a-z0-9_.-]+\.v[2-9][0-9]*' ...` correctly uses the broadened pattern (resolving round-5 harden-r5-7). gate_11 now actually couples a fixture-driven check (`scripts/check-contract-cutover-fixtures.mjs` verified — line 10 enforces that `oss/skills/issue-to-pr/SKILL.md` retains product copy, and lines 17-28 enforce that the act schema contains `revision` and rejects `issue-to-pr`, satisfying the SKILL.md vs. contract bidirectional rule). gate_8 uses parameterized `$RUNX_CONSUMER_REPOS_ROOT` (resolving round-5 harden-r5-8). The `! rg` negation correctly maps no-match (rg exit 1) → exit 0, matching the `exit_code_zero` semantics.
- scope/migration audit
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1202, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1519, code:/Users/kam/dev/runx/runx/cloud/packages/db/src/migrations.test.ts:14, code:/Users/kam/dev/runx/runx/cloud/packages/db/src/postgres.test.ts:86
  - Result: failed
  - Evidence: The 'Cutover Deployment And Durable Data' section mandates five deliverables for Phase 4: (a) pre-deploy intake drain, (b) one-way row migration, (c) sealed migration harness receipts citing source-row hashes and target refs, (d) dedupe-key rehoming to `fingerprint` and harness `idempotency`, (e) proof that no retired-vocabulary rows remain. Phase 4's acceptance gate ac4_1 (`cd ../cloud && pnpm test -- packages/db/src/migrations.test.ts packages/db/src/postgres.test.ts -t "harness"`) was verified against the cited files: migrations.test.ts contains zero `it()` names matching 'harness' (the string 'harness' only appears in two SQL version literals at lines 35-36, not in test names — under Vitest's `-t` filter that test file contributes zero passing assertions); postgres.test.ts has one `it("persists harness receipt queue packets and enforces lifecycle transitions", ...)` at line 86 that tests the already-shipped PostgresHostedHarnessReceiptStore. Neither file asserts drain, migration, migration-receipt emission, dedupe rehoming, or absence of retired-vocabulary rows. The gate would pass today on `main` with none of Phase 4's substantive work done. This is the same acceptance-mismatch class round-5 flagged for gates 9/11/12 and remains unfixed for the most operationally risky phase.
- acceptance timing audit
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1438, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1469, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1501, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1527
  - Result: passed
  - Evidence: Phase dependencies now read phase1 → phase2 → phase3 → phase4 → phase5. Phase 4's dependency on phase3 (rather than phase2) is correct because Phase 4's migration targets cite harness receipts, decisions, and contained acts which are Phase 3 deliverables (resolving round-5 harden-r5-2). Phase 1's acceptance (`scafld harden --mark-passed`) is appropriate for a design-only phase. Phase 3 acceptance (`pnpm typecheck && pnpm --dir oss fixtures:contracts:check && node --test $NITROSEND_TESTS`) is achievable because the contract source-of-truth (TypeBox) is resolved in the spec body. Note: Phase 5 → Phase 4 dependency is not justified in the spec; Aster's contract shapes (target, opportunity, selection_cycle, feed_entry, reflection_entry) depend conceptually on harness/decision/contained-act (Phase 3), not on the durable migration (Phase 4). The dependency is defensible only if cloud feed-projection surfaces require migrated data — the spec does not say.
- rollback/repair audit
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1550, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1206, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1226
  - Result: passed
  - Evidence: The Rollback section names concrete steps: revert coupled runx/nitrosend/aster branches, restore hosted data from the Phase 4 pre-cut snapshot, use the Phase 4 row-map artifact to verify restored row counts and hashes, re-run consumer gates against restored branches, and no read aliases or translation shims. The spec body's 'Cutover Deployment And Durable Data' section pins the pre-cut snapshot and one-way migration with row-map retention as a requirement, so rollback has a concrete inverse procedure. This resolves round-5 harden-r5-4's rollback dimension at the spec-text level, though the test-gate dimension remains unaddressed (see scope/migration audit above).
- design challenge
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/agent-hook.ts, code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/runner.ts, code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/publish.ts, code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/framing-patterns.ts, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:992, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1234
  - Result: failed
  - Evidence: Two design holes survive round-5 fixes. (1) The Package Disambiguation block (spec line 992) asserts `@runxhq/runtime-local/harness` is 'retained only as the replay-mode runtime implementation for governed harness fixtures.' But the package's actual contents — verified in `oss/packages/runtime-local/src/harness/`: `agent-hook.ts`, `runner.ts`, `publish.ts`, `framing-patterns.ts`, `quality.ts`, `mcp-fixture.ts`, `a2a-fixture.ts` — are agent-hook execution, runner orchestration, framing patterns, quality checks, and adapter fixtures. Only mcp-fixture and a2a-fixture plausibly fit 'replay-mode'; the rest are general-purpose runtime infrastructure. The disambiguation is a rename-by-redefinition that does not match the code. The block then defers: 'If a future import becomes ambiguous, the contract shape keeps the bare harness name; runtime helpers move behind a qualified runtime/replay path.' That punt to 'if future ambiguity arises' IS the round-5 ambiguity. (2) The Cross-Repo Coordination section (spec line 1234) requires consumer repos to pin the cutover contract build and forbids pinning a pre-cut `@runxhq/contracts` package or runx SHA. But gate_7 only scans `RUNX_CUTOVER_EXTRA_ROOTS` for banned vocabulary (verified — `check-contract-cutover.mjs:5` only reads file content under the roots). No gate inspects nitrosend or aster `package.json` for the contracts version pin. The 'must' in the Coordination section is unenforceable. This is not fatal (operator discipline could substitute), but it is the same class of design hole the round-5 reviewer flagged about cross-repo synchronization.

Issues:
- [high/blocks approval] `harden-r6-1` acceptance_mismatch - Phase 4 acceptance gate does not verify Phase 4's substantive deliverables (drain, one-way migration, sealed migration receipts, dedupe rehoming, absence of retired-vocabulary rows).
  - Status: open
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1519, code:/Users/kam/dev/runx/runx/cloud/packages/db/src/migrations.test.ts:14, code:/Users/kam/dev/runx/runx/cloud/packages/db/src/postgres.test.ts:86
  - Evidence: Phase 4 ac4_1 runs `cd ../cloud && pnpm test -- packages/db/src/migrations.test.ts packages/db/src/postgres.test.ts -t "harness"`. Verified against cited files: migrations.test.ts has zero `it()` names matching 'harness' — under Vitest's `-t` filter that file contributes nothing. postgres.test.ts has one `it("persists harness receipt queue packets and enforces lifecycle transitions")` that exercises the already-shipped PostgresHostedHarnessReceiptStore on main. None of (a) intake drain procedure, (b) one-way row migration, (c) sealed migration-receipt emission with source-row hashes, (d) dedupe-key rehoming to fingerprint/idempotency, or (e) proof that no retired-vocabulary rows remain is asserted. The Cutover Deployment And Durable Data section names all five as requirements. The gate would pass today on `main` with zero Phase 4 work done — the same acceptance-mismatch round-5 caught for gates 9/11/12, on the most operationally risky phase.
  - Recommendation: Replace ac4_1 with concrete fixture-driven tests added by Phase 4: (a) a Vitest case asserting a `drain_state` flag is set before migration runs; (b) a Vitest case that seeds a small fixture of legacy rows, runs the one-way migration, and asserts target tables contain harness receipts whose source-row hashes match the seeded rows; (c) a Vitest case loading a migration receipt fixture and asserting it validates against `runx.harness_receipt.v1` and contains `act.form: "verification"` or `"observation"` plus source/target refs; (d) a Vitest case asserting `select count(*) from <retired_table>` is zero or that the retired tables have been dropped. Update Phase 4's Changes list to enumerate these tests, and either keep ac4_1 as `pnpm test -t "phase4 migration"` or invoke the new tests by file.
  - Question: Should ac4_1 be replaced with concrete fixture-driven assertions for drain, row migration, migration-receipt emission, dedupe rehoming, and absence-of-retired-rows, added as Phase 4 Change items?
  - Recommended answer: Yes. Add four new tests under cloud/packages/db/src/migration/phase4.test.ts (or equivalent) asserting each of the four substantive deliverables, and rewrite ac4_1 as `cd ../cloud && pnpm test -- packages/db/src/migration/phase4.test.ts`. Keep the existing harness-receipt-store tests as separate coverage; they exercise the steady-state layer, not the migration event.
  - If unanswered: Default to adding the four phase4-named tests and rewriting ac4_1 to target them.
- [medium/advisory] `harden-r6-2` design_risk - Package Disambiguation block 'retains' `@runxhq/runtime-local/harness` as 'the replay-mode runtime implementation' but the package's actual modules are not replay-mode infrastructure — the rename decision is punted to 'if future ambiguity arises.'
  - Status: open
  - Grounded in: code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/agent-hook.ts, code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/runner.ts, code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/publish.ts, code:/Users/kam/dev/runx/runx/oss/packages/runtime-local/src/harness/framing-patterns.ts, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:992
  - Evidence: Spec line 992-1004 asserts `@runxhq/runtime-local/harness` 'is retained only as the replay-mode runtime implementation for governed harness fixtures' and that its `agent-hook`, `runner`, `publish`, `mcp-fixture`, and `a2a-fixture` modules are 'subordinate runtime adapters.' Verified contents of the package on disk: `agent-hook.ts` (agent hook execution), `runner.ts` (runner orchestration), `publish.ts` (publish flow), `framing-patterns.ts`, `quality.ts`, `mcp-fixture.ts`, `a2a-fixture.ts`. Only the two `*-fixture.ts` modules plausibly match 'replay-mode'; the rest are general-purpose runtime infrastructure. The block then defers: 'If a future import becomes ambiguous, the contract shape keeps the bare harness name; runtime helpers move behind a qualified runtime/replay path.' That defers the rename to 'if future ambiguity arises' — which is the round-5 ambiguity itself. Round-5 harden-r5-5 is therefore not fully resolved.
  - Recommendation: Either (a) make the rename a Phase 2 prerequisite Change item (e.g. rename the runtime package to `runtime-local/src/agent-hooks/` or `runtime-local/src/skill-harness/`, leaving only `mcp-fixture.ts` and `a2a-fixture.ts` under a smaller `runtime-local/src/replay/` path), or (b) rewrite the Package Disambiguation block to truthfully describe what the package contains today (it is the runtime that executes agent hooks, framing, publish, and adapter fixtures under a harness) and document the import-path convention (`@runxhq/runtime-local/harness` for runtime, `@runxhq/contracts` for the contract envelope). Punting to future ambiguity is the ambiguity itself.
  - Question: Do you want to rename `oss/packages/runtime-local/src/harness/` as a Phase 2 prerequisite, or rewrite the Package Disambiguation block to honestly describe the existing package without claiming it is replay-mode?
  - Recommended answer: Rewrite the block, do not rename. The runtime package is too large to rename safely as a Phase 2 prerequisite, and the import paths are already distinct (`@runxhq/contracts` vs `@runxhq/runtime-local/harness`). Replace the 'retained as replay-mode' assertion with: 'The runtime package `@runxhq/runtime-local/harness` owns the local execution loop (agent-hooks, runner, publish, framing, quality, MCP/A2A fixtures) that runs *inside* a contract harness. It imports contract shapes from `@runxhq/contracts` and must not define a second harness wire shape. The contract harness envelope lives only in `@runxhq/contracts`.' Add a follow-up issue (out-of-scope for this cutover) to consider the rename later if a real ambiguity arises.
  - If unanswered: Default to the rewrite above and add a follow-up issue for the rename.
- [medium/advisory] `harden-r6-3` spec_gap - Cross-Repo Coordination section requires consumer repos to pin the cutover contract build, but no gate enforces the version pin.
  - Status: open
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1234, code:/Users/kam/dev/runx/runx/scripts/check-contract-cutover.mjs:5
  - Evidence: Spec body 'Cross-Repo Coordination' (line ~1234) reads: 'Package pins must point at the cutover contract build. Consumers may not pin a pre-cut @runxhq/contracts package or runx SHA after their cutover branch is marked ready.' Verified gate_7 in `scripts/check-contract-cutover.mjs:5-10`: the script only reads file content under `RUNX_CUTOVER_EXTRA_ROOTS` and scans for banned vocabulary patterns. It does not parse `package.json` in consumer roots or compare the pinned `@runxhq/contracts` version against an expected cutover version. The 'must' in the Coordination section is therefore not gated; cross-repo synchronization depends entirely on operator discipline. Round-5 harden-r5-6 partially resolved (the env-var-driven scan exists) but the version-pin enforcement piece remains.
  - Recommendation: Extend `scripts/check-contract-cutover.mjs` (or add a sibling `check-cross-repo-pins.mjs`) to read each extra root's `package.json` and assert the `dependencies.@runxhq/contracts` semver matches the cutover major. Wire that into gate_7 or add a new gate. Alternatively, document the operator checklist explicitly: 'Before marking the cutover ready, manually verify nitrosend/package.json and aster/package.json pin @runxhq/contracts >= <version>.' The spec body should pick one approach.
  - Question: Do you want a programmatic version-pin check (extend check-contract-cutover.mjs to parse consumer package.json) or an explicit operator checklist (manual verification documented in the Coordination section)?
  - Recommended answer: Programmatic check. Hard cutover semantics are too strict to rely on operator memory across three repos. Extend the cutover script to read each extra-root `package.json` and fail if `dependencies.@runxhq/contracts` (or `devDependencies`) does not match a pinned cutover semver supplied via env var (e.g. `RUNX_CUTOVER_CONTRACTS_VERSION`). Add it as a Change item to Phase 2 or Phase 3 alongside the existing cutover-check work.
  - If unanswered: Default to the programmatic check above and add a Phase 3 Change item.
- [low/advisory] `harden-r6-4` spec_gap - Phase 5's dependency on Phase 4 is not justified; Aster contracts conceptually depend on Phase 3 deliverables.
  - Status: open
  - Grounded in: code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1527, code:.scafld/specs/drafts/runx-contract-spine-hard-cutover.md:1056
  - Evidence: Phase 5 declares `Dependencies: phase4`. Phase 5's Changes are: add target/opportunity/thesis_assessment/selection/skill_binding/target_transition_entry/selection_cycle/reflection_entry/feed_entry contracts; update Aster UI/feed model; update cloud public/feed projection surfaces; add fixture-driven test for feed entry proof bindings. The 'Aster Contracts' section (spec line ~1056) shows these contracts cite harness receipts, contained act ids, and decision ids — all Phase 3 deliverables. The only plausible reason to depend on Phase 4 is that 'cloud public/feed projection surfaces' need migrated durable data — but the spec does not say. As written, Phase 5 is gated by a phase whose substance it does not consume.
  - Recommendation: Either (a) state explicitly in Phase 5's Objective or Dependencies note that the cloud feed projection update requires migrated durable data from Phase 4 (so the dependency is correct), or (b) change `Dependencies: phase4` to `Dependencies: phase3` if the Aster work can land in parallel with the migration. The explicit rationale costs one line and saves a future operator from wondering why Phase 5 is blocked.
  - Question: Does Phase 5's cloud feed projection update require Phase 4's migrated durable data, or could Phase 5 land in parallel with Phase 4?
  - Recommended answer: Keep `Dependencies: phase4` and add a single rationale line to Phase 5's Objective: 'Aster feed projections cite signed harness receipts persisted by Phase 4's durable migration; landing Phase 5 before the migration would publish projections against a half-migrated store.' If the cloud projection layer reads from harness receipts written either before or after the migration, then this rationale is wrong and the dependency should be relaxed to phase3.
  - If unanswered: Default to keeping phase4 dependency with the rationale line above.


## Planning Log

- 2026-05-18T00:00:00Z - operator - Rejected brittle central-object naming and
  ratified a hard-cut contract redesign for the run-anything product model.
- 2026-05-18T00:20:00Z - agent - Recentered the design on the harness spine,
  contained acts, decisions, signals, and shared primitives before
  implementation so contract code, Rust parity, nitrosend, and Aster move
  together instead of accreting another dual-shape layer.
