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
  inspect: { kind: start, label: "Inspect the current EC review digest" }
  objective: { kind: decision, label: "Objective EC finding exists?" }
  revise: { kind: terminal, label: "Route to bounded EC fill" }
  evidence: { kind: decision, label: "Current human review accepted?" }
  question: { kind: process, label: "Emit target, six checks, choices, payload, and resume" }
  human: { kind: decision, label: "Human accepts or requests revision" }
  persist: { kind: process, label: "Validate digest-bound human evidence" }
  generate: { kind: terminal, label: "Run EC generation and verification" }
edges:
  - { from: inspect, to: objective }
  - { from: objective, to: revise, label: "yes" }
  - { from: objective, to: evidence, label: "no" }
  - { from: evidence, to: generate, label: "yes" }
  - { from: evidence, to: question, label: "no" }
  - { from: question, to: human, label: "host maps user_question to its native tool" }
  - { from: human, to: persist, label: "accept with summary" }
  - { from: human, to: revise, label: "revision findings" }
  - { from: persist, to: generate }
---
flowchart TD
    inspect([Inspect the current EC review digest]) --> objective{Objective EC finding exists?}
    objective -->|yes| revise([Route to bounded EC fill])
    objective -->|no| evidence{Current human review accepted?}
    evidence -->|yes| generate([Run EC generation and verification])
    evidence -->|no| question[Emit target, six checks, choices, payload, and resume]
    question -->|host maps user_question to native tool| human{Human accepts or requests revision}
    human -->|accept with summary| persist[Validate digest-bound human evidence]
    human -->|revision findings| revise
    persist --> generate
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/capability.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "HitlQuestion"
    gap: "missing-generator:logic"
    tracker: "#1806"
    reason: "HITL envelopes name a host-neutral user_question interaction instead of a host-specific tool."
  - path: apps/agentic-workflow/src/cli/ec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "run_review"
    gap: "missing-generator:logic"
    tracker: "#1806"
    reason: "EC review HITL question requires bounded integration in run_review."
  - path: apps/agentic-workflow/templates/cli/mainthread/CLAUDE.md.tmpl
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "requires_hitl=true"
    gap: "missing-generator:logic"
    tracker: "#1806"
    reason: "Host guidance maps the semantic interaction to the native Claude Code, Codex, or AGY tool."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: emit-actionable-hitl-questions-for-pending-ec-review-verification
requirements:
  human_independence_preserved:
    id: R3
    text: "The structured handoff cannot turn same-agent evidence into production approval."
    kind: security
    risk: high
    verify: ec_review_requires_current_independent_human_evidence
  pending_question_contract:
    id: R1
    text: "A pending production EC review emits a typed HITL question for host-agent rendering."
    kind: functional
    risk: high
    verify: ec_review_pending_emits_structured_hitl_question
  semantic_checklist_visible:
    id: R2
    text: "The pending question names the review target, all six semantic checks, and human decision paths."
    kind: regression
    risk: high
    verify: ec_review_pending_emits_structured_hitl_question
---
flowchart TD
    r1[R1 pending question contract] --> ec_review_pending_emits_structured_hitl_question[ec_review_pending_emits_structured_hitl_question]
    r2[R2 semantic checklist visible] --> ec_review_pending_emits_structured_hitl_question
    r3[R3 human independence preserved] --> ec_review_requires_current_independent_human_evidence[ec_review_requires_current_independent_human_evidence]
```
