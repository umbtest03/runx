# runx-core

Pure Rust parity kernel for runx state-machine and policy decisions.

This crate currently implements state-machine parity against the TypeScript
oracle fixtures under `fixtures/kernel/state-machine/`. TypeScript remains the
source of truth until a separate cutover spec changes consumers.

`runx-core` must stay free of filesystem, network, subprocess, MCP, adapter,
and CLI presentation behavior.
