---
id: "1650"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-domain-v1-contract-flow
entry: plan
nodes:
  plan: { kind: start, label: "confirmed Domain v1 plan" }
  evidence: { kind: process, label: "closed baseline WIs and repository evidence" }
  contract: { kind: process, label: "canonical README capability contract" }
  work: { kind: process, label: "bounded child work roots under epic 1157" }
  lifecycle: { kind: process, label: "linear WI to TD to code-check delivery" }
  gates: { kind: decision, label: "capability, EC, recovery, and health gates pass?" }
  done: { kind: terminal, label: "close epic 1157" }
edges:
  - { from: plan, to: contract }
  - { from: evidence, to: contract }
  - { from: contract, to: work }
  - { from: work, to: lifecycle }
  - { from: lifecycle, to: gates }
  - { from: gates, to: done, label: "all verified" }
  - { from: gates, to: work, label: "gap remains" }
---
flowchart TD
    plan([confirmed plan]) --> contract[capability contract]
    evidence[closed WI evidence] --> contract
    contract --> work[bounded work roots]
    work --> lifecycle[WI to TD to code-check]
    lifecycle --> gates{all gates pass?}
    gates -->|yes| done([close epic 1157])
    gates -->|no| work
```
