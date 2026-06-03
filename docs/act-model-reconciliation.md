# Act-model reconciliation

This note settles a recurring question: do the runx core contracts still need an
"act-model" reconciliation, or is the live shape final? Short answer: **the live
contracts are the reconciled model.** The `work_item` / `sealed_act` / `effect.form`
vocabulary that appears in some plans is aspirational naming, not a pending reshape
of the core contracts.

## The live model (source of truth)

The normalized act-model is implemented and shipping in `crates/runx-contracts`:

- **Act** (`src/act.rs:130`) — a unit of governed work. Its kind is **`ActForm`**
  (`src/act.rs:49`): `revision`, `reply`, `review`, `observation`, `verification`.
  An Act carries `criterion_bindings`, `change_request`/`change_plan`
  (`RevisionDetails`), target/source/artifact refs, and a closure.
- **Decision** (`src/decision.rs:64`) — a governed choice, with a `Closure`
  (`ClosureDisposition`: `closed`, `deferred`, `superseded`, `declined`, `blocked`,
  `failed`, `killed`, `timed_out`).
- **Signal** (`src/signal.rs:64`) — an inbound trigger, with a `SignalSchema` and a
  `SignalTrustLevel`.
- **Receipt** (`src/receipt.rs:321`) — the sealed, flat record of a run. It inlines
  `signals: Vec<Reference>`, `decisions: Vec<Decision>`, `acts: Vec<ReceiptAct>`, a
  `seal`, `authority`, `subject`, `idempotency`, and optional `lineage`. The journal
  is a *projection* computed on demand from the receipt
  (`runx-runtime/src/journal.rs::project_receipt_journal`), not a separately
  persisted artifact.

The `runx.receipt.v1` cutover is live and green; these types are what the runtime
emits and seals.

## Plan vocabulary -> live contracts

Some plans (`plans/runx.md`, `plans/aster.md`) use an alternate vocabulary. It maps
onto the live shape with no contract change required:

| Plan term      | Live contract                                                        |
| -------------- | -------------------------------------------------------------------- |
| `work_item`    | the run's `Subject` + its `Act`s (the unit of governed work)         |
| `sealed_act`   | an `Act` carried in a sealed `Receipt` (`Receipt.acts` + `Receipt.seal`) — sealing is a property of the Receipt, not a separate `SealedAct` type |
| `effect.form`  | `ActForm`                                                            |
| `decision`     | `Decision`                                                           |
| `signal`       | `Signal`                                                             |

There is intentionally **no** `SealedAct` type and **no** `effect.form` enum in the
code: introducing aliases for unused vocabulary would be speculative cruft. If a
future version adopts that naming, it can be added then.

## Conclusion

v1 finalizes the act/decision/signal/receipt model; the alternate plan vocabulary is
future-aspirational, not an outstanding reconciliation against the core contracts.
The remaining act-model work is in *enforcement and adoption* (e.g. aster gating its
dispatch on declared act forms and verification status), not in reshaping these
types. Those live on the aster/consumer side, not in the kernel contracts.
