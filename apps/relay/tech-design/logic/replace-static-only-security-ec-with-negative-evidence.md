---
id: '2175'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-security-negative-evidence-applicability
entry: rejected
nodes:
  rejected:
    kind: start
    label: "Independent review rejects static-only Relay security evidence"
  shared:
    kind: decision
    label: "Is a shared security mechanism missing?"
  shared_wi:
    kind: terminal
    label: "Stop and route a separate lib WI"
  classify:
    kind: process
    label: "Align capability with Lumen SecurityTool dimensions"
  negative:
    kind: process
    label: "Exercise Relay auth admission peer TLS and K8s rejection posture"
  stability:
    kind: process
    label: "Exercise last-known-good rotation and trusted peer continuity"
  done:
    kind: terminal
    label: "Behavior security and stability evidence fail closed"
edges:
  - { from: rejected, to: shared }
  - { from: shared, to: shared_wi, label: "yes" }
  - { from: shared, to: classify, label: "no" }
  - { from: classify, to: negative }
  - { from: negative, to: stability }
  - { from: stability, to: done }
---
flowchart TD
    rejected[static-only EC rejected] --> shared{shared mechanism missing?}
    shared -->|yes| shared_wi[separate lib WI]
    shared -->|no| classify[SecurityTool contract]
    classify --> negative[Relay negative journeys]
    negative --> stability[rotation and peer stability]
    stability --> done[fail-closed evidence]
```
