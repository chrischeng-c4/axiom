---
id: "1255"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-pull-subscription-applicability
entry: pull
nodes:
  pull:
    kind: start
    label: "tape subscription pull TOPIC NAME --limit L addresses an existing pull subscription"
  mode:
    kind: decision
    label: "subscription exists and delivery.mode is pull"
  reject:
    kind: terminal
    label: "push or missing subscription is rejected without changing the journal"
  cursor:
    kind: process
    label: "read the existing topic/name checkpoint as the next unread offset; absent checkpoint means offset 0"
  bounded:
    kind: process
    label: "replay at most min(requested limit, max batch) events; caller-driven limit is the backpressure boundary"
  response:
    kind: terminal
    label: "return events, cursor, and next_offset; pull never advances the durable cursor itself"
  ack:
    kind: process
    label: "tape subscription ack TOPIC NAME --offset N delegates to checkpoint advance for the same topic/name"
  stale:
    kind: terminal
    label: "existing stale and beyond-end checkpoint rejections remain the ack safety contract"
  claims:
    kind: terminal
    label: "README and EC performance wording call this bounded pull/replay path; no push worker, lease scheduler, or raft cursor consensus"
edges:
  - { from: pull, to: mode }
  - { from: mode, to: reject, label: "missing or push" }
  - { from: mode, to: cursor, label: "pull" }
  - { from: cursor, to: bounded }
  - { from: bounded, to: response }
  - { from: response, to: ack }
  - { from: ack, to: stale }
  - { from: stale, to: claims }
---
flowchart TD
    pull["subscription pull TOPIC NAME --limit L"] --> mode{"existing pull subscription?"}
    mode -->|missing or push| reject(["reject; journal unchanged"])
    mode -->|pull| cursor["checkpoint topic/name = next unread offset; default 0"]
    cursor --> bounded["replay <= requested/max batch; client pull is backpressure"]
    bounded --> response(["events + cursor + next_offset; no implicit ack"])
    response --> ack["subscription ack delegates to durable checkpoint"]
    ack --> stale(["stale and beyond-end ack rejection unchanged"])
    stale --> claims(["performance claims describe bounded pull/replay only"])
```
