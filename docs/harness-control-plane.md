# Harness Control Plane

runx issue automation is governed by harness receipts, not by a tracker-style
queue object. Source adapters admit signals, decisions open or decline harness
nodes, contained acts perform revision, reply, review, observation, or
verification effects, and each harness seals to a receipt.

## Ownership

OSS owns:

- canonical signal, decision, act, harness, artifact, redaction, verification,
  and harness receipt contracts
- local skill inputs and outputs
- source-thread and outbox packets
- receipts and scafld-backed issue-to-PR execution

Cloud owns:

- hosted harness storage and receipt indexing
- approval inboxes
- authenticated source adapters
- org routing and operational APIs

Consuming repos own:

- Slack, Sentry, GitHub, or file source filters
- target repo policy
- owner suggestion rules
- source-thread notification policy

## Lifecycle

The durable lifecycle is explicit:

1. A signal records the admitted source event.
2. A decision opens, defers, declines, or routes a harness.
3. The harness admits authority and records idempotency.
4. Acts inside the harness record intent and effect.
5. Child harnesses carry attenuated authority.
6. The harness seals normally or abnormally to a receipt.

Every terminal path produces a receipt. Failed, killed, timed out, blocked, and
declined paths are not missing evidence; they are abnormal seals with reason
codes, criterion state, and hash commitments.

## Evidence

Evidence is carried by references, artifacts, verification checks, redaction
records, and receipt commitments. There is no separate evidence bundle
contract. Adapters may hydrate richer provider context before a decision opens
a mutation harness, but the governed boundary is the harness receipt.

## Merge Authority

The issue-to-PR product lane may create PRs and post source-thread updates, but
human merge authority stays outside the default runx mutation authority. A
hosted operator that wants auto-merge needs an explicit policy surface with
separate repo allowlists, branch protections, checks, audit events, and
rollback contracts.
