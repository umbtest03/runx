# runx-runtime

Placeholder crate for the future native Rust runtime.

This crate will eventually own impure execution behavior: filesystem access,
subprocesses, sandbox enforcement, MCP process handling, adapters, receipt
writing, resume, history, inspect, replay, and diff. The TypeScript local
runtime remains authoritative.

Adapter families are modeled as runtime features until there is a concrete
reason to publish them independently:

- `cli-tool`
- `mcp`
- `a2a`
- `agent`
- `catalog`
