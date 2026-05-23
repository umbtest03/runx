---
spec_version: '2.0'
task_id: runx-receipt-claim-graph-s-tier-v1
created: '2026-05-23T00:00:00Z'
updated: '2026-05-23T00:00:00Z'
status: draft
build_gate: requires_explicit_confirmation
tier: s
size: xl
risk_level: high
---

# runx receipt — S tier (the verifiable claim graph)

> **DO NOT BUILD WITHOUT EXPLICIT CONFIRMATION.** This is a design draft and a
> direction, not a build target. Most of it should not be built until the A+ shape
> (`runx-receipt-aplus-shape-v1`) ships and earns it. Two cheap moves from this tier
> are recommended for early adoption (flagged below); the rest waits.

## The meaning of this tier — from document to data structure

A+ is a *document done right*. S tier is the moment the receipt **stops being a
document and becomes a verifiable data structure**. That is the entire meaning of the
tier, and it is a real phase change, not a polish.

In A+, integrity is something you **assert and then check**: the receipt says "trust
me, here is a signature," and a verifier runs a procedure to decide whether to believe
it. Truth is an *action* the reader performs. In S tier, integrity becomes
**structural** — a property of the address itself. When a claim's `id` *is* the hash
of its content, a reference to it is self-verifying: you cannot point at a tampered
claim, because the pointer and the checksum are the same string. Truth stops being a
procedure you run and becomes a *shape the data has*.

Three consequences follow, and each is a thing A+ has to *do* that S tier gets *for
free*:

1. **Trust without a trusted party.** Anyone holding a claim can verify it and its
   entire ancestry — the grants it cites, the signals that triggered it, the receipts
   it descends from — with no registry to consult, no server to ask, no authority to
   believe. The graph is self-evidencing. This is the git/Merkle/IPFS insight applied
   to governed agent work: provenance is not recorded *about* the data, it *is* the
   data's structure.

2. **Identity collapses into content.** Idempotency, dedup, and "have I seen this
   before" stop being machinery you build (the hand-rolled `idempotency` block) and
   become arithmetic: same content, same address. Two runs that did the identical
   governed thing are *literally the same node*.

3. **The receipt stops being special.** A receipt, a resolution, a signal, a grant, an
   authority-proof are no longer five schemas with five envelopes and five validators.
   They are one primitive — a **signed, content-addressed claim** `{contract, body,
   refs[], issuer, signature}` whose `id = hash(canonical(body))` — distinguished only
   by the *type of body*. The receipt is one node type in a typed, signed,
   content-addressed DAG. The whole governance/proof substrate becomes one graph.

The deep meaning, stated plainly: **S tier is where runx stops *describing* what
happened and becomes *the verifiable structure of* what happened.** Composition,
provenance, tamper-evidence, and dedup are no longer features in a backlog; they are
theorems about a Merkle DAG. Every projection (verification, training corpus, payment
ledger, history, lineage) becomes a *fold or query over the graph* rather than a
bespoke exporter — which is also the structural cure for the "shallow projection"
failure that butchered the trainable export.

## The shape

```
runx.claim.v1   (the one envelope; receipt/resolution/signal/grant/... are typed bodies)
  contract   runx:contract:<hash>          # which grammar this body conforms to (self-identifying)
  body       <typed: the A+ run, or a resolution, or a signal, ...>
  refs[]     runx:<type>:<hash>             # edges: self-verifying because the address is the checksum
  issuer     { type, kid, public_key_sha256 }
  signature  { alg: Ed25519, value }
  # id is NOT stored: id == hash(canonical(body)) under runx.claim.c14n.v1
```

The A+ receipt body is unchanged in *content* — `signed(envelope, run)` — but it now
lives inside this universal claim envelope, its `id` is its content hash, and its
references are content-addressed edges. A resolution is the same envelope with a
verdict body and a `ref` to the receipt. A graph receipt's `lineage.children` are
content-addressed edges to child claims; verifying a graph receipt transitively
verifies the whole tree by address.

## What this buys vs what it costs (honest)

Buys: structural integrity (verification is a property, not a routine), free dedup,
uniform composition, a single envelope/signing/canonicalization primitive for the
whole system, projections as graph folds, offline/third-party verifiability.

Costs (do not gloss): content-addressing forces strict immutability (no edit-in-place;
a "change" is a new node + a supersedes-edge); cycle and dangling-ref handling; a real
content-addressed store; and the genuine risk of **premature or wrong unification** —
receipt, signal, and grant have different lifecycles and consumers, and forcing them
into one envelope can create coupling that hurts later. Unify only where the shared
envelope demonstrably removes cost, not for symmetry.

## The two cheap moves recommended for early (A+) adoption

These two are compounding and expensive to retrofit, so take them even at A+:

1. **Content-addressed ids** — `receipt.id = hash(canonical(run))`. Self-verifying
   references and free dedup, with none of the full-graph machinery.
2. **One shared envelope for receipt + resolution** — the two artifacts that genuinely
   share a lifecycle and trust boundary. Not for signal/grant yet.

The rest of S tier (the universal claim primitive across all body types, the
content-addressed store, projections-as-graph-queries) waits for explicit approval.

## Relationship to the other tiers

A+ is the document; S tier makes it a verifiable structure; enlightened
(`runx-receipt-enlightened-north-star-v1`) dissolves the structure into the system
itself. S tier is the bridge: it is the most you can build *as engineering* before the
remaining moves become *philosophy*.

## Origin

Conversation 2026-05-23: asked for the A+, S, and enlightened shapes. S tier is the
phase change from a well-formed receipt document to a content-addressed signed-claim
graph where integrity is structural. Captured as a direction; build only the two
flagged cheap moves early, and the rest only on explicit confirmation.
