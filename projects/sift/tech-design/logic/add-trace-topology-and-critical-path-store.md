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
  select: { kind: decision, label: "signal is span?" }
  normalize: { kind: process, label: "normalize span status links events and timing" }
  upsert: { kind: process, label: "idempotent trace span upsert" }
  topology: { kind: process, label: "build deterministic parent child topology" }
  validate: { kind: decision, label: "missing parent or cycle?" }
  partial: { kind: process, label: "mark partial and record topology gaps" }
  critical: { kind: process, label: "compute longest non overlapping critical path" }
  correlate: { kind: process, label: "collect correlation ids for logs errors metrics profiles sessions" }
  checkpoint: { kind: terminal, label: "persist trace snapshot and checkpoint" }
  query: { kind: start, label: "GET trace by id" }
  authorize: { kind: process, label: "authorize project read" }
  wait: { kind: decision, label: "min cursor reached?" }
  lag: { kind: terminal, label: "projection lag" }
  result: { kind: terminal, label: "stable complete or explicit partial trace" }
edges:
  - { from: raw_span, to: select }
  - { from: select, to: normalize, label: "yes" }
  - { from: select, to: checkpoint, label: "no op" }
  - { from: normalize, to: upsert }
  - { from: upsert, to: topology }
  - { from: topology, to: validate }
  - { from: validate, to: partial, label: "yes" }
  - { from: validate, to: critical, label: "no" }
  - { from: partial, to: critical }
  - { from: critical, to: correlate }
  - { from: correlate, to: checkpoint }
  - { from: query, to: authorize }
  - { from: authorize, to: wait }
  - { from: wait, to: lag, label: "timeout" }
  - { from: wait, to: result, label: "ready" }
---
flowchart TD
    raw_span([committed span]) --> select{signal is span?}
    select -->|yes| normalize[normalize span model]
    select -->|no| checkpoint([checkpoint])
    normalize --> upsert[idempotent upsert]
    upsert --> topology[parent child topology]
    topology --> validate{missing parent or cycle?}
    validate -->|yes| partial[mark partial and gaps]
    validate -->|no| critical[critical path]
    partial --> critical
    critical --> correlate[correlation refs]
    correlate --> checkpoint
    query([GET trace]) --> authorize[project read auth]
    authorize --> wait{min cursor reached?}
    wait -->|timeout| lag([projection lag])
    wait -->|ready| result([stable trace result])
```
