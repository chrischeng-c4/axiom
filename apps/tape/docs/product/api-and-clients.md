# API and clients

One HTTP contract, discoverable from the binary, with typed clients generated
from it. This area is the README capability `api-cli-clients`. Feature parity
with Cloud Pub/Sub is offered only through this contract.

## One discoverable HTTP contract

- Problem: none open as shipped.
- Who: every client author; agents reading `tape llm`.
- Promise: the node serves exactly the routes `tape spec --format routes`
  lists, over HTTP/1.1 and h2c on one port; `GET /openapi.json` is byte-equal
  to the committed `clients/openapi.json`, and a gate refuses drift;
  TypeScript, Python, and Rust clients are generated from the spec and scoped
  to those routes; the CLI ships the standard command set with `--help` and
  `tape llm`.
- Non-goals: the Google API surface; gRPC.
- Neighbours: none; first section of the area. Every outcome that adds or
  removes a route changes this contract and regenerates the snapshot and the
  clients.
- Status rows: `generated-clients`, `standard-operational-endpoints`.

## Pub/Sub rebaseline

- Problem: the crate description, the OpenAPI info block, the served route
  descriptions, the custom resource documentation, and the committed snapshot
  still describe tape as a replay journal; the cold-restore and disk-full
  runbooks live in the deployment handoff page while the alert rule and the
  operator render point at that page.
- Who: anyone who reads what the binary says about itself; operators
  following an alert to its runbook.
- Promise: every identity string tape emits describes it as the Cloud Pub/Sub
  stand-in, regenerated together with the committed OpenAPI snapshot; the
  runbooks move under `docs/runbooks/` with the alert rule and the operator
  render pointing at the new path; the deployment handoff page is no longer
  tracked. No product behaviour changes.
- Non-goals: any route change; those belong to the outcome that owns the
  route.
- Open: none; the ROADMAP boundary is complete.
- Neighbours: rewrites the prose of One discoverable HTTP contract; the
  runbook move touches [operations.md](operations.md) § Whole-journal backup
  and cold seed.
- Outcome: `pubsub-rebaseline`. Tracking: not assigned.

## Non-goals in this area

- `pubsub-wire-compatibility`: speaking `pubsub.googleapis.com` would tie
  the product to a protocol it does not control.
- `streaming-pull`: no gRPC, so nothing to stream over.
