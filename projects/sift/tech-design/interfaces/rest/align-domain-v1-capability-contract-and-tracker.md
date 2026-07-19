---
id: "1650"
summary: Align Sift's canonical capability contract and bounded Domain v1 work graph with the confirmed headless observability scope.
capability_refs:
  - id: developer-and-agent-experience
    role: primary
    gap: offline-contract
    claim: offline-contract
    coverage: partial
    rationale: Agents need one current capability contract and bounded work graph before they can drive Domain v1 delivery deterministically.
fill_sections: [logic, changes]
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

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-domain-v1-capability-contract
    tracker: "1650"
    description: Add the six confirmed Domain v1 capability roots, correct evidence-backed baseline status, and attach bounded child work roots under epic 1157.
```
