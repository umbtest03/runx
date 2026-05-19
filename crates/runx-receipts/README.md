# runx-receipts

Pure Rust harness receipt verification for runx.

The post-cutover receipt model treats a receipt as the sealed proof of a
harness node. This crate validates that shape against the shared
`runx-contracts` types, provides deterministic canonical JSON and digest
helpers, and keeps verification pure. It does not write receipt files or invoke
runtime adapters.

Current verification covers structural invariants: terminal harness seal
presence, top-level seal mirroring, form-specific act payloads, decision and
seal criterion references, child harness receipt references, authority
attenuation proof presence, supplied child receipt resolution, and `sha256:`
hash commitments. Signature checking, persistent child receipt lookup, and full
authority algebra verification are separate runtime integrations.
