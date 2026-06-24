---
id: workitem-loop-state-model-additive-foundation
summary: Add a LoopState model carried in the WI body as an `<!-- aw:loop-state -->` block (goal/verifier/iterations/last_result/next_action/status/tried), with lossless round-trip and an `aw wi show` surface, additive and non-breaking alongside the existing CRRR WorkflowProjection.
fill_sections: [logic, unit-test]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: workitem-loop-state-model
    claim: workitem-loop-state-model
    coverage: partial
    rationale: "The WorkItem-first artifact lifecycle gains the loop-state representation the workflow loop reads and writes, mirroring the existing WorkflowProjection block."
---

# TD: WorkItem loop-state model (additive foundation)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workitem-loop-state-serde
entry: parse
nodes:
  parse: { kind: start, label: "parse_loop_state(body)" }
  find: { kind: decision, label: "<!-- aw:loop-state --> block present?" }
  none: { kind: terminal, label: "return None (no loop state yet)" }
  de: { kind: process, label: "serde_yaml::from_str -> LoopState" }
  ret: { kind: terminal, label: "return Some(LoopState)" }
  up: { kind: start, label: "upsert_loop_state(body, state)" }
  ser: { kind: process, label: "serde_yaml::to_string(state) + wrap in <!-- aw:loop-state -->" }
  splice: { kind: decision, label: "block already in body?" }
  replace: { kind: terminal, label: "replace existing block in place" }
  append: { kind: terminal, label: "append new block to body" }
edges:
  - { from: parse, to: find }
  - { from: find, to: none, label: "absent" }
  - { from: find, to: de, label: "present" }
  - { from: de, to: ret }
  - { from: up, to: ser }
  - { from: ser, to: splice }
  - { from: splice, to: replace, label: "yes" }
  - { from: splice, to: append, label: "no" }
---
flowchart TD
  parse([parse_loop_state]) --> find{block present?}
  find -->|absent| none([None])
  find -->|present| de[serde_yaml parse]
  de --> ret([Some LoopState])
  up([upsert_loop_state]) --> ser[serialize + wrap]
  ser --> splice{block in body?}
  splice -->|yes| replace([replace in place])
  splice -->|no| append([append block])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: workitem-loop-state-unit-tests
requirements:
  round_trip:
    id: R1
    text: "LoopState round-trips losslessly through upsert_loop_state -> parse_loop_state across all last_result variants and a multi-iteration log."
    kind: functional
    risk: high
    verify: test
  absent_is_none:
    id: R2
    text: "parse_loop_state returns None when no aw:loop-state block is present (absent block is not an error)."
    kind: functional
    risk: high
    verify: test
  upsert_in_place:
    id: R3
    text: "upsert_loop_state appends when absent and replaces in place when present, without disturbing the existing score:workflow-state block."
    kind: functional
    risk: medium
    verify: test
elements:
  loop_state_round_trips:
    kind: test
    type: "rs/#[test]"
  loop_state_absent_block_is_none:
    kind: test
    type: "rs/#[test]"
  loop_state_upsert_replaces_in_place:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: loop_state_round_trips, verifies: round_trip }
  - { from: loop_state_absent_block_is_none, verifies: absent_is_none }
  - { from: loop_state_upsert_replaces_in_place, verifies: upsert_in_place }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "loop state round-trips losslessly"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "absent block parses to None"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "upsert appends then replaces in place"
      risk: medium
      verifymethod: test
    }
    element loop_state_round_trips {
      type: "rs/#[test]"
    }
    element loop_state_absent_block_is_none {
      type: "rs/#[test]"
    }
    element loop_state_upsert_replaces_in_place {
      type: "rs/#[test]"
    }
```
