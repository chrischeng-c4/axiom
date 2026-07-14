---
id: "1665"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-trace-store
entry: raw_span
nodes:
  raw_span: { kind: start, label: "committed span event" }
  normalize: { kind: process, label: "normalize timing status links and events" }
  upsert: { kind: process, label: "idempotent span upsert" }
  topology: { kind: process, label: "resolve parent child topology and gaps" }
  critical: { kind: process, label: "compute deterministic critical path" }
  checkpoint: { kind: terminal, label: "persist trace store checkpoint" }
  query: { kind: start, label: "authorized trace query" }
  wait: { kind: decision, label: "projection cursor ready?" }
  lag: { kind: terminal, label: "projection lag" }
  result: { kind: terminal, label: "complete or explicit partial trace" }
edges:
  - { from: raw_span, to: normalize }
  - { from: normalize, to: upsert }
  - { from: upsert, to: topology }
  - { from: topology, to: critical }
  - { from: critical, to: checkpoint }
  - { from: query, to: wait }
  - { from: wait, to: lag, label: "no" }
  - { from: wait, to: result, label: "yes" }
---
flowchart TD
    raw_span([committed span]) --> normalize[normalize span]
    normalize --> upsert[idempotent upsert]
    upsert --> topology[topology and gaps]
    topology --> critical[critical path]
    critical --> checkpoint([checkpoint])
    query([authorized query]) --> wait{cursor ready?}
    wait -->|no| lag([projection lag])
    wait -->|yes| result([trace result])
```
