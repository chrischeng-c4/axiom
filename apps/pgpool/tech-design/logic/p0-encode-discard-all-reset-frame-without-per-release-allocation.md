---
id: '1620'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-static-discard-all-frame
entry: release
nodes:
  release: { kind: start, label: "Backend release to idle" }
  frame: { kind: process, label: "Write byte-exact static DISCARD ALL Query frame" }
  reader: { kind: process, label: "Use existing FrameReader until ReadyForQuery" }
  idle: { kind: terminal, label: "Reset backend eligible for reuse" }
edges:
  - { from: release, to: frame }
  - { from: frame, to: reader }
  - { from: reader, to: idle }
---
flowchart LR
  release([release backend]) --> frame[static DISCARD ALL frame]
  frame --> reader[existing FrameReader to ReadyForQuery]
  reader --> idle([safe idle backend])
```
