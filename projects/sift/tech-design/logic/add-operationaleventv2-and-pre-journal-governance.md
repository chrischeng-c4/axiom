---
id: "1657"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-operational-event-v2-governance
entry: input
nodes:
  input: { kind: start, label: "v1 or v2 operational event" }
  upcast: { kind: process, label: "upcast v1 into canonical V2" }
  validate: { kind: decision, label: "V2 schema and project scope valid?" }
  reject: { kind: terminal, label: "reject before durability" }
  policy: { kind: process, label: "apply typed-attribute and content policy" }
  redact: { kind: process, label: "truncate and redact sensitive content" }
  journal: { kind: process, label: "serialize governed V2 for durable journal" }
  ack: { kind: terminal, label: "return durable cursor" }
edges:
  - { from: input, to: upcast }
  - { from: upcast, to: validate }
  - { from: validate, to: reject, label: "no" }
  - { from: validate, to: policy, label: "yes" }
  - { from: policy, to: redact }
  - { from: redact, to: journal }
  - { from: journal, to: ack }
---
flowchart TD
    input([v1 or v2 event]) --> upcast[upcast to canonical V2]
    upcast --> validate{schema and project valid?}
    validate -->|no| reject([reject before durability])
    validate -->|yes| policy[apply typed attribute and content policy]
    policy --> redact[truncate and redact]
    redact --> journal[durably serialize governed V2]
    journal --> ack([return durable cursor])
```
