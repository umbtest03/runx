# Receipt Verify Corpus

This corpus pins the `runx.verify_verdict.v1` machine verdict emitted by
`runx verify --receipt <path|-> --json`.

Each case directory contains:

- `receipt.json`: the input document, including malformed input where relevant
- `expected.json`: the exact verdict expected from the CLI and library API
- `case.json`: the case metadata and signature mode

`verifier.json` carries only the fixture key id and public key needed to replay
production-signed cases. It never contains signing material.

Hosted notary surfaces must replay this corpus through the pinned `runx` binary
instead of reimplementing receipt verification in another language.
