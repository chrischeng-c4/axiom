---
id: '1877'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lossless-wire-relay-tag-coverage
entry: bounded_frame
nodes:
  bounded_frame: { kind: start, label: "length-bounded frontend or backend frame" }
  classify: { kind: decision, label: "known control frame or opaque relay frame" }
  validate: { kind: process, label: "validate minimum structure and configured bounds" }
  relay: { kind: process, label: "preserve exact bytes across session and relay paths" }
  cancel: { kind: terminal, label: "deliberately reject unsupported CancelRequest" }
edges:
  - { from: bounded_frame, to: classify }
  - { from: classify, to: validate }
  - { from: validate, to: relay }
  - { from: classify, to: cancel, label: cancel }
---
flowchart TD
  bounded_frame[bounded wire frame] --> classify{recognized control frame?}
  classify --> validate[validate structure and limits]
  validate --> relay[verbatim relay]
  classify -->|CancelRequest| cancel[deliberate rejection]
```
