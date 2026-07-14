---
id: "1667"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-metric-store-contract
entry: metric
nodes:
  metric: { kind: start, label: "validated direct metric signal" }
  identity: { kind: process, label: "hash canonical resource attributes name unit kind" }
  budget: { kind: decision, label: "new identity within project budget" }
  normal: { kind: process, label: "retain exact series identity" }
  overflow: { kind: process, label: "route to deterministic overflow identity and count diagnostic" }
  point: { kind: process, label: "insert timestamp cursor ordered point" }
  reset: { kind: process, label: "detect cumulative reset without rewriting raw point" }
  histogram: { kind: process, label: "validate and merge compatible explicit or exponential histogram" }
  rollup: { kind: process, label: "materialize fixed windows with exemplars" }
  checkpoint: { kind: terminal, label: "fsynced projection checkpoint" }
  query: { kind: start, label: "typed bounded query" }
  aggregate: { kind: process, label: "gauge last delta sum cumulative increase histogram merge" }
  page: { kind: terminal, label: "stable page and projection cursor" }
edges:
  - { from: metric, to: identity }
  - { from: identity, to: budget }
  - { from: budget, to: normal, when: yes }
  - { from: budget, to: overflow, when: no }
  - { from: normal, to: point }
  - { from: overflow, to: point }
  - { from: point, to: reset }
  - { from: reset, to: histogram }
  - { from: histogram, to: rollup }
  - { from: rollup, to: checkpoint }
  - { from: query, to: aggregate }
  - { from: aggregate, to: page }
---
flowchart LR
    metric([metric]) --> identity[series identity] --> budget{cardinality}
    budget -->|yes| normal[exact identity]
    budget -->|no| overflow[overflow identity]
    normal --> point[ordered point]
    overflow --> point --> reset[reset semantics] --> histogram[histogram] --> rollup[rollups] --> checkpoint([checkpoint])
    query([query]) --> aggregate[typed aggregate] --> page([page])
```
