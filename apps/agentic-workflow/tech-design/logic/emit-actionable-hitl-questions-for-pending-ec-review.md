---
id: '1806'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: emit-actionable-hitl-questions-for-pending-ec-review-flow
entry: inspect
nodes:
  inspect: { kind: start, label: "Inspect EC review state" }
  objective: { kind: decision, label: "Objective EC finding exists?" }
  revise: { kind: terminal, label: "Route to bounded EC fill" }
  evidence: { kind: decision, label: "Current human review accepted?" }
  question: { kind: process, label: "Emit typed HITL question" }
  human: { kind: decision, label: "Human accepts or requests revision" }
  persist: { kind: process, label: "Validate and persist human evidence" }
  generate: { kind: terminal, label: "Run EC generation and verification" }
edges:
  - { from: inspect, to: objective }
  - { from: objective, to: revise, label: "yes" }
  - { from: objective, to: evidence, label: "no" }
  - { from: evidence, to: generate, label: "yes" }
  - { from: evidence, to: question, label: "no" }
  - { from: question, to: human, label: "host renders ask_user_question" }
  - { from: human, to: persist, label: "accept" }
  - { from: human, to: revise, label: "request revision" }
  - { from: persist, to: generate }
---
flowchart TD
    inspect([Inspect EC review state]) --> objective{Objective EC finding exists?}
    objective -->|yes| revise([Route to bounded EC fill])
    objective -->|no| evidence{Current human review accepted?}
    evidence -->|yes| generate([Run EC generation and verification])
    evidence -->|no| question[Emit typed HITL question]
    question -->|host renders ask_user_question| human{Human accepts or requests revision}
    human -->|accept| persist[Validate and persist human evidence]
    human -->|request revision| revise
    persist --> generate
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/ec.rs
    action: modify
    section: logic
    impl_mode: codegen
  - path: apps/agentic-workflow/src/cli/llm.rs
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/agentic-workflow/tech-design/logic/emit-actionable-hitl-questions-for-pending-ec-review.md
    action: modify
    section: logic
    impl_mode: hand-written
```
