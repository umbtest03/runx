---
name: byo-http-graph
description: BYO credential portfolio example; a graph step reads a non-GitHub provider over HTTP.
---
# BYO HTTP graph

A single-step graph that drives `../byo-http-tool`. It proves the OSS side of
the BYO portfolio seam: a locally supplied credential reaches a graph-step HTTP
front as a scoped secret header, the provider read executes, and the receipt
seals the response plus non-secret credential-delivery observation.

Run `examples/byo-http-graph/run.sh` to store an isolated demo profile through
stdin, start the local example CRM fixture, and execute the graph with
`runx skill --profile demo`.
