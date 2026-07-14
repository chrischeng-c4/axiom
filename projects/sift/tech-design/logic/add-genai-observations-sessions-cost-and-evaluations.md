---
id: "1670"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-genai-observability-contract
entry: span
nodes:
  span: { kind: start, label: "governed OTel span with GenAI semantic attributes" }
  classify: { kind: process, label: "classify generation tool agent retrieval or RAG" }
  normalize: { kind: process, label: "normalize provider model tokens cost latency status and correlations" }
  session: { kind: process, label: "group observations across traces by project session" }
  view: { kind: terminal, label: "observation and aggregate session views" }
  evaluation: { kind: start, label: "governed evaluation event" }
  validate: { kind: decision, label: "typed score and trace span or session target valid" }
  append: { kind: process, label: "append immutable evaluation without source mutation" }
  reject: { kind: terminal, label: "typed invalid evaluation error" }
  rebuild: { kind: start, label: "raw replay" }
  equal: { kind: terminal, label: "same observations sessions costs and scores" }
edges:
  - { from: span, to: classify }
  - { from: classify, to: normalize }
  - { from: normalize, to: session }
  - { from: session, to: view }
  - { from: evaluation, to: validate }
  - { from: validate, to: append, label: "yes" }
  - { from: validate, to: reject, label: "no" }
  - { from: append, to: view }
  - { from: rebuild, to: equal }
---
flowchart LR
    span([GenAI span]) --> classify[classify] --> normalize[normalize] --> session[session] --> view([views])
    evaluation([evaluation]) --> validate{valid}
    validate -->|yes| append[append] --> view
    validate -->|no| reject([reject])
    rebuild([raw replay]) --> equal([equal])
```
