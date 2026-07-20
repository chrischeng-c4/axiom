---
id: '2172'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-measured-performance-applicability
entry: rejected
nodes:
  rejected:
    kind: start
    label: "Independent EC review rejects synthetic performance oracle"
  boundary:
    kind: decision
    label: "Does remediation change Relay domain semantics?"
  exclude_domain:
    kind: terminal
    label: "Stop and create a separate domain WI"
  measure:
    kind: process
    label: "Measure existing durable publish then lease and ack lifecycle"
  oracle:
    kind: process
    label: "Parse machine report independently and enforce pinned floors"
  ec:
    kind: process
    label: "Bind behavior efficiency stability cases through vat and meter"
  done:
    kind: terminal
    label: "Performance EC cannot pass with missing or zero observations"
edges:
  - { from: rejected, to: boundary }
  - { from: boundary, to: exclude_domain, label: "yes" }
  - { from: boundary, to: measure, label: "no" }
  - { from: measure, to: oracle }
  - { from: oracle, to: ec }
  - { from: ec, to: done }
---
flowchart TD
    rejected[rejected synthetic EC] --> boundary{domain semantics change?}
    boundary -->|yes| exclude_domain[separate domain WI]
    boundary -->|no| measure[measure durable lifecycle]
    measure --> oracle[independent parsed oracle]
    oracle --> ec[behavior efficiency stability EC]
    ec --> done[fail closed evidence]
```
