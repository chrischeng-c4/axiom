---
id: "1255"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-pull-subscription-contract
entry: pull_request
nodes:
  pull_request:
    kind: start
    label: "PullSubscriptionBatch uses the durable topic/name checkpoint as a next-offset cursor"
  validate_limit:
    kind: decision
    label: "--limit defaults to 100 and must not exceed MAX_PULL_BATCH=1000"
  limit_error:
    kind: terminal
    label: "oversized request returns SubscriptionError::PullBatchTooLarge without replaying or advancing state"
  resolve_pull:
    kind: decision
    label: "topic/name resolves to SubscriptionDelivery::Pull"
  mode_error:
    kind: terminal
    label: "missing or push resource returns SubscriptionError; push remains endpoint metadata only"
  replay_window:
    kind: process
    label: "read checkpoint offset or 0, then replay the bounded event window; next_offset is last event offset + 1 or cursor when empty"
  pull_result:
    kind: terminal
    label: "emit PullSubscriptionBatch { events, cursor, next_offset, limit }; no cursor mutation occurs on pull"
  ack_request:
    kind: process
    label: "subscription ack confirms the resource is pull, then calls put_checkpoint(topic, name, offset)"
  ack_result:
    kind: terminal
    label: "checkpoint success is the durable ack; stale and beyond-end errors propagate unchanged through SubscriptionAckError"
  inventory:
    kind: terminal
    label: "offline spec declares pull and ack route schemas; live h2c handlers, leases, push workers, and raft cursor consensus stay excluded"
edges:
  - { from: pull_request, to: validate_limit }
  - { from: validate_limit, to: limit_error, label: "limit > 1000" }
  - { from: validate_limit, to: resolve_pull, label: "bounded" }
  - { from: resolve_pull, to: mode_error, label: "missing or push" }
  - { from: resolve_pull, to: replay_window, label: "pull" }
  - { from: replay_window, to: pull_result }
  - { from: pull_result, to: ack_request }
  - { from: ack_request, to: ack_result }
  - { from: ack_result, to: inventory }
---
flowchart TD
    pull_request["PullSubscriptionBatch from topic/name checkpoint cursor"] --> validate_limit{"limit <= 1000; default 100?"}
    validate_limit -->|oversized| limit_error(["PullBatchTooLarge; no state changes"])
    validate_limit -->|bounded| resolve_pull{"subscription delivery=pull?"}
    resolve_pull -->|missing or push| mode_error(["SubscriptionError; no delivery"])
    resolve_pull -->|pull| replay_window["replay bounded window from checkpoint or 0; compute next_offset"]
    replay_window --> pull_result(["events/cursor/next_offset; pull never advances cursor"])
    pull_result --> ack_request["ack validates pull then delegates to put_checkpoint"]
    ack_request --> ack_result(["durable ack or stale/beyond-end rejection"])
    ack_result --> inventory(["offline pull/ack schema only; no h2c/lease/push/raft implementation"])
```
