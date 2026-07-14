---
id: '1632'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-lease-owned-capacity
entry: acquire
nodes:
  acquire: { kind: start, label: "Acquire idle or fresh backend" }
  lease: { kind: process, label: "Lease owns permit and active RAII token" }
  relay: { kind: process, label: "Relay session or one transaction" }
  return: { kind: process, label: "Drop active token then reset stream" }
  idle: { kind: process, label: "Park stream and permit in idle collection" }
  close: { kind: process, label: "Drop stream and permit then wake waiters" }
  dropped: { kind: terminal, label: "Unreleased lease RAII frees capacity once" }
edges:
  - { from: acquire, to: lease }
  - { from: lease, to: relay }
  - { from: relay, to: return, label: "ReturnToIdle" }
  - { from: return, to: idle, label: "reset ReadyForQuery idle" }
  - { from: return, to: close, label: "reset failure" }
  - { from: relay, to: close, label: "Close" }
  - { from: lease, to: dropped, label: "lease dropped" }
---
flowchart LR
  acquire([acquire]) --> lease[lease owns permit + active token]
  lease --> relay[relay]
  relay -->|return| return[drop active token; reset]
  return -->|clean| idle[park stream + permit]
  return -->|bad| close[drop stream + permit]
  relay -->|close| close
  lease -->|unreleased drop| dropped([free once + notify])
```
