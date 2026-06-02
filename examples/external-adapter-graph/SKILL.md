---
name: external-adapter-graph
description: External-adapter front example; a graph whose step runs a governed subprocess adapter.
---
# External-adapter graph

A single-step graph that drives an external-adapter sub-skill. The runtime routes
the graph step's `external-adapter` source through the source-adapter registry to
the external-adapter executor, which resolves the manifest, spawns the declared
subprocess under the governed sandbox, exchanges the invocation and response
frames, and seals the reported result.

External-adapter is a graph-step front, not a top-level runner. Run this skill's
inline harness with `runx harness examples/external-adapter-graph`.
