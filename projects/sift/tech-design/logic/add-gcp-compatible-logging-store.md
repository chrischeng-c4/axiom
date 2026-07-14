---
id: "1664"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-logging-store
entry: raw_log
nodes:
  raw_log: { kind: start, label: "committed raw log event" }
  select: { kind: decision, label: "signal is log?" }
  normalize: { kind: process, label: "normalize jsonPayload and OTel body" }
  dedupe: { kind: process, label: "upsert by event id and raw cursor" }
  index: { kind: process, label: "index fixed text keyword range fields in embedded Lumen" }
  retain: { kind: process, label: "apply independent record retention" }
  checkpoint: { kind: terminal, label: "atomically persist logging snapshot and checkpoint" }
  query: { kind: start, label: "POST logs query or GET logs tail" }
  authorize: { kind: process, label: "authorize project read" }
  wait: { kind: decision, label: "min cursor reached?" }
  lag: { kind: terminal, label: "projection lag with Retry After" }
  candidates: { kind: process, label: "resolve full text candidates" }
  filters: { kind: process, label: "apply typed time project resource severity correlation and attribute filters" }
  page: { kind: terminal, label: "stable raw cursor page and resume cursor" }
edges:
  - { from: raw_log, to: select }
  - { from: select, to: normalize, label: "yes" }
  - { from: select, to: checkpoint, label: "no op" }
  - { from: normalize, to: dedupe }
  - { from: dedupe, to: index }
  - { from: index, to: retain }
  - { from: retain, to: checkpoint }
  - { from: query, to: authorize }
  - { from: authorize, to: wait }
  - { from: wait, to: lag, label: "timeout" }
  - { from: wait, to: candidates, label: "ready" }
  - { from: candidates, to: filters }
  - { from: filters, to: page }
---
flowchart TD
    raw_log([committed raw log]) --> select{signal is log?}
    select -->|yes| normalize[normalize structured body]
    select -->|no| checkpoint([checkpoint unchanged])
    normalize --> dedupe[versioned idempotent upsert]
    dedupe --> index[embedded Lumen fixed fields]
    index --> retain[independent retention]
    retain --> checkpoint[atomic state plus checkpoint]
    query([logs query or tail]) --> authorize[project read authorization]
    authorize --> wait{min cursor reached?}
    wait -->|timeout| lag([projection lag])
    wait -->|ready| candidates[text candidates]
    candidates --> filters[typed filters]
    filters --> page([stable cursor page])
```
