---
id: "1670"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-genai-observability-applicability
entry: event
nodes:
  event: { kind: start, label: "canonical span or evaluation event" }
  governed: { kind: decision, label: "pre-journal content policy already applied" }
  kind: { kind: decision, label: "GenAI semantic attributes or evaluation signal" }
  observe: { kind: process, label: "specialize generation tool agent retrieval or RAG observation" }
  evaluate: { kind: process, label: "append typed immutable score linkage" }
  group: { kind: terminal, label: "session token cost latency and quality views" }
  ignore: { kind: terminal, label: "not a GenAI observation" }
  reject: { kind: terminal, label: "reject content-governance violation" }
edges:
  - { from: event, to: governed }
  - { from: governed, to: kind, label: "yes" }
  - { from: governed, to: reject, label: "no" }
  - { from: kind, to: observe, label: "GenAI span" }
  - { from: kind, to: evaluate, label: "evaluation" }
  - { from: kind, to: ignore, label: "other" }
  - { from: observe, to: group }
  - { from: evaluate, to: group }
---
flowchart LR
    event([event]) --> governed{governed}
    governed -->|yes| kind{kind}
    governed -->|no| reject([reject])
    kind -->|span| observe[observation] --> group([views])
    kind -->|evaluation| evaluate[score] --> group
    kind -->|other| ignore([ignore])
```
