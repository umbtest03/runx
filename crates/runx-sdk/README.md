# runx-sdk

CLI-backed Rust SDK for runx.

This crate calls an installed `runx` binary and consumes documented
`runx --json` output. It provides typed helpers for the initial client surface:
skill search, skill run, resume, connect list, host protocol decoding, and
act-assignment construction.

SDK v0 does not execute skills natively and does not replace the TypeScript
runtime. Native runtime support is future work behind a later `native-runtime`
feature once the Rust runtime cutover is complete.

```rust
use runx_sdk::{RunSkillOptions, RunxClient};

let client = RunxClient::new();
let results = client.search_skills("sourcey", None)?;
let report = client.run_skill(
    "skills/sourcey",
    RunSkillOptions::default().with_input("project", "."),
)?;
```

SDK v0 depends on `runx-contracts`, not `runx-core`; that keeps the SDK
shippable before kernel parity is complete.
