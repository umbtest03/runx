---
name: byo-http-tool
description: HTTP front sub-skill; reads an example CRM account with a delivered local credential.
source:
  type: http
  url: http://127.0.0.1:8734/v1/accounts/{account_id}
  method: GET
  allow_private_network: true
  headers:
    authorization: "Bearer ${secret:EXAMPLE_CRM_TOKEN}"
inputs:
  account_id:
    type: string
    required: true
    description: Example CRM account id to fetch.
---
A non-GitHub provider read over the first-class `http` front. The bearer token is
not stored in the skill or passed on argv; the parent runner declares and resolves
`EXAMPLE_CRM_TOKEN`, the HTTP adapter resolves the
`${secret:...}` header reference, and the sealed receipt records only the
non-secret credential observation.

The loopback URL is fixture-only. `allow_private_network` is the explicit opt-in
for that local fixture; real provider skills should use public provider URLs.
