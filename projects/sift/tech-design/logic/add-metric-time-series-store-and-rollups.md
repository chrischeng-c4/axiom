---
id: "1667"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-metric-store
entry: metric
nodes:
  metric: { kind: start, label: "committed metric event" }
  identity: { kind: process, label: "canonical project resource attribute series identity" }
  budget: { kind: decision, label: "within project cardinality budget" }
  chunk: { kind: process, label: "temporality aware ordered chunk" }
  overflow: { kind: process, label: "deterministic overflow series and diagnostic" }
  rollup: { kind: process, label: "histogram exemplar and fixed-window rollups" }
  checkpoint: { kind: terminal, label: "independent durable checkpoint" }
  query: { kind: start, label: "typed metric query" }
  aggregate: { kind: process, label: "stable aggregation and pagination" }
  result: { kind: terminal, label: "series rollup and diagnostics" }
edges:
  - { from: metric, to: identity }
  - { from: identity, to: budget }
  - { from: budget, to: chunk, when: yes }
  - { from: budget, to: overflow, when: no }
  - { from: chunk, to: rollup }
  - { from: overflow, to: rollup }
  - { from: rollup, to: checkpoint }
  - { from: query, to: aggregate }
  - { from: aggregate, to: result }
---
flowchart LR
    metric([metric]) --> identity[series identity] --> budget{budget}
    budget -->|yes| chunk[ordered chunk]
    budget -->|no| overflow[overflow series]
    chunk --> rollup[rollups]
    overflow --> rollup --> checkpoint([checkpoint])
    query([query]) --> aggregate[aggregate] --> result([result])
```
