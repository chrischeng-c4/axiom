---
id: "1666"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-error-store
entry: exception
nodes:
  exception: { kind: start, label: "committed exception" }
  group: { kind: process, label: "normalize fingerprint and group occurrence" }
  checkpoint: { kind: terminal, label: "independent durable checkpoint" }
  transition: { kind: start, label: "authorized state transition" }
  commit: { kind: process, label: "single Sift state machine commit" }
  evidence: { kind: process, label: "durable audit and change evidence" }
  result: { kind: terminal, label: "effective open acknowledged resolved or muted state" }
edges:
  - { from: exception, to: group }
  - { from: group, to: checkpoint }
  - { from: transition, to: commit }
  - { from: commit, to: evidence }
  - { from: evidence, to: result }
---
flowchart LR
    exception([exception]) --> group[group occurrence] --> checkpoint([checkpoint])
    transition([transition]) --> commit[state machine] --> evidence[audit/change] --> result([state])
```
