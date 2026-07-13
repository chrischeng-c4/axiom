---
id: '1625'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-physical-backend-reader-lifecycle
entry: connect
nodes:
  connect: { kind: start, label: "Fresh physical backend connect" }
  reader: { kind: process, label: "Create one backend FrameReader with the connection" }
  lease: { kind: process, label: "Lease stream and reader through startup or transaction relay" }
  reset: { kind: process, label: "Same reader validates DISCARD ALL response" }
  idle: { kind: terminal, label: "Return clean stream and reader together to idle" }
edges:
  - { from: connect, to: reader }
  - { from: reader, to: lease }
  - { from: lease, to: reset }
  - { from: reset, to: idle }
---
flowchart LR
  connect([fresh physical backend]) --> reader[one backend FrameReader]
  reader --> lease[lease stream and reader]
  lease --> reset[same reader validates reset]
  reset --> idle([return clean stream and reader])
```
