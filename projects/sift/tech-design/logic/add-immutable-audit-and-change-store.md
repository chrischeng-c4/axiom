---
id: "1668"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-audit-change-contract
entry: event
nodes:
  event: { kind: start, label: "durable audit or change event" }
  validate: { kind: process, label: "require actor action target and factual payload" }
  append: { kind: process, label: "append cursor ordered hash chained record" }
  checkpoint: { kind: terminal, label: "immutable projection checkpoint" }
  control: { kind: start, label: "admin hold or export request" }
  commit: { kind: process, label: "single Sift state machine commit" }
  evidence: { kind: process, label: "append audit and change evidence for control mutation" }
  export: { kind: terminal, label: "content hash manifest and bounded records" }
edges:
  - { from: event, to: validate }
  - { from: validate, to: append }
  - { from: append, to: checkpoint }
  - { from: control, to: commit }
  - { from: commit, to: evidence }
  - { from: evidence, to: export }
---
flowchart LR
    event([event]) --> validate[validate] --> append[hash chain] --> checkpoint([checkpoint])
    control([hold/export]) --> commit[state machine] --> evidence[audit/change evidence] --> export([manifest])
```
