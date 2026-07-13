---
id: "1254"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-subscription-resource-flow
entry: request
nodes:
  request:
    kind: start
    label: "Tape adds a first-class subscription resource for one topic"
  cli_create:
    kind: process
    label: "tape subscription create TOPIC NAME --pull or --push ENDPOINT validates exactly one delivery mode and saves the journal"
  model:
    kind: process
    label: "TapeJournal stores Subscription { topic, name, delivery }; delivery is pull or push { endpoint }"
  pull:
    kind: process
    label: "pull subscription uses its name as the existing durable consumer checkpoint identity; create does not move the cursor"
  push:
    kind: terminal
    label: "push subscription records its callback endpoint only; no HTTP delivery, retry, or worker runs in this WI"
  inspect:
    kind: process
    label: "tape subscription list/show/delete reads or mutates the same file-backed journal resource"
  contract:
    kind: process
    label: "spec.rs declares topic-scoped create/list/show/delete routes plus Subscription and delivery schemas in routes/OpenAPI/JSON Schema"
  unchanged:
    kind: terminal
    label: "server.rs live h2c routes, raft replication, push execution, retries, auth expansion, and redelivery remain out of scope"
edges:
  - { from: request, to: cli_create }
  - { from: cli_create, to: model }
  - { from: model, to: pull, label: "mode=pull" }
  - { from: model, to: push, label: "mode=push" }
  - { from: pull, to: inspect }
  - { from: push, to: inspect }
  - { from: inspect, to: contract }
  - { from: contract, to: unchanged }
---
flowchart TD
    request["Tape adds a first-class subscription resource for one topic"] --> cli_create["tape subscription create TOPIC NAME --pull or --push ENDPOINT validates exactly one delivery mode and saves the journal"]
    cli_create --> model["TapeJournal stores Subscription topic/name/delivery"]
    model -->|mode=pull| pull["pull uses subscription name as existing durable consumer checkpoint identity"]
    model -->|mode=push| push(["push records endpoint only; no worker runs"])
    pull --> inspect["subscription list/show/delete uses the file-backed resource"]
    push --> inspect
    inspect --> contract["spec routes/OpenAPI/JSON Schema declare the resource"]
    contract --> unchanged(["live h2c routes, raft state, push execution/retry/redelivery remain out of scope"])
```
