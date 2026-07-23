---
id: '2446'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: python-first-self-hosting-admission
entry: root
nodes:
  root: { kind: start, label: "aw goal wi, capability, or backlog" }
  identity: { kind: decision, label: "project is agentic-workflow?" }
  normal: { kind: process, label: "run the generic root engine" }
  artifact: { kind: decision, label: "current artifact phase" }
  authored: { kind: process, label: "hand-author and validate EC or TD" }
  generated: { kind: process, label: "generate, fill, and check CB" }
  worker: { kind: decision, label: "selected worker verb succeeds?" }
  fallback: { kind: process, label: "repair only the bounded current change" }
  proof: { kind: process, label: "focused regression and Refs issue trailer" }
  resume: { kind: process, label: "re-enter root and follow next.command" }
  terminal: { kind: terminal, label: "completion.workflow_complete is true" }
edges:
  - { from: root, to: identity }
  - { from: identity, to: normal, label: "yes or no" }
  - { from: normal, to: artifact }
  - { from: artifact, to: authored, label: "EC or TD" }
  - { from: artifact, to: generated, label: "CB" }
  - { from: authored, to: worker }
  - { from: generated, to: worker }
  - { from: worker, to: resume, label: "yes" }
  - { from: worker, to: fallback, label: "no" }
  - { from: fallback, to: proof }
  - { from: proof, to: resume }
  - { from: resume, to: terminal }
---
```

The root dispatcher must not special-case `agentic-workflow` at admission. It resolves the same WI, capability, or reviewed backlog state as any other project. The existing self-health gate partition remains visible, but its policy mode becomes `python_first_lifecycle`; health also emits `fallback_mode=bounded_direct_repair`, trigger `selected_worker_verb_is_broken`, scope `current_change_only`, required trailer `Refs #<issue>`, and `direct_repair_default=false`. The fallback is an operator recovery contract and never a root-runner response.
