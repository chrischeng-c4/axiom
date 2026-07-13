---
id: '1618'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-multithread-runtime
entry: pgpool_cli
nodes:
  pgpool_cli: { kind: start, label: "pgpool CLI process" }
  runtime: { kind: process, label: "Tokio multi-thread runtime" }
  relays: { kind: process, label: "Concurrent frontend and backend relay tasks" }
  invariant: { kind: terminal, label: "Unchanged wire and reset isolation semantics" }
edges:
  - { from: pgpool_cli, to: runtime }
  - { from: runtime, to: relays }
  - { from: relays, to: invariant }
---
flowchart LR
  pgpool_cli([pgpool CLI]) --> runtime[Tokio multi-thread runtime]
  runtime --> relays[concurrent proxy relay tasks]
  relays --> invariant([unchanged wire and reset isolation])
```
