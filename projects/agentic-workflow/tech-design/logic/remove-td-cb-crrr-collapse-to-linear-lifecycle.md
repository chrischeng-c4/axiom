---
id: remove-td-cb-crrr-collapse-to-linear-lifecycle
summary: Remove the TD/CB CRRR (review/revise/arbitrate) steps and collapse the lifecycle to a linear td_inited -> create -> td_created -> gen -> cb_genned -> fill -> cb_filled -> merge. Centralize the linear hop in a pure next_phase_command(phase) function and trim is_mergeable to cb_genned|cb_filled.
fill_sections: [logic, unit-test]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: crrr-removal-linear-lifecycle
    claim: crrr-removal-linear-lifecycle
    coverage: partial
    rationale: "The TD/CB lifecycle drops the review/revise/arbitrate ceremony and becomes a linear create -> gen -> fill -> merge progression."
---

# TD: remove TD/CB CRRR, collapse to linear lifecycle

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: crrr-removal-linear-lifecycle
entry: inited
nodes:
  inited: { kind: start, label: "td_inited: handle_create_milestone" }
  created: { kind: process, label: "td_created: next_phase_command -> aw td gen (was: aw td review)" }
  genned: { kind: process, label: "cb_genned: next_phase_command -> aw cb fill" }
  filled: { kind: process, label: "cb_filled: next_phase_command -> aw td merge (no review)" }
  gengate: { kind: decision, label: "run_gen_code: phase == td_created?" }
  mergegate: { kind: decision, label: "is_mergeable: cb_genned | cb_filled?" }
  merged: { kind: terminal, label: "td_merged: post-merge lifecycle" }
  reject: { kind: terminal, label: "error: unexpected phase" }
edges:
  - { from: inited, to: created }
  - { from: created, to: gengate }
  - { from: gengate, to: genned, label: "yes" }
  - { from: gengate, to: reject, label: "no" }
  - { from: genned, to: filled }
  - { from: filled, to: mergegate }
  - { from: mergegate, to: merged, label: "cb_genned|cb_filled" }
  - { from: mergegate, to: reject, label: "other" }
---
flowchart TD
  inited([td_inited create]) --> created[td_created]
  created --> gengate{phase == td_created?}
  gengate -->|yes| genned[cb_genned]
  gengate -->|no| reject([error])
  genned --> filled[cb_filled]
  filled --> mergegate{is_mergeable?}
  mergegate -->|cb_genned/cb_filled| merged([td_merged])
  mergegate -->|other| reject
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: crrr-removal-unit-tests
requirements:
  create_to_gen:
    id: R1
    text: "After create, the linear lifecycle routes td_created straight to `aw td gen` (no review step)."
    kind: functional
    risk: high
    verify: test
  mergeable_linear:
    id: R2
    text: "is_mergeable accepts only cb_genned and cb_filled; non-code phases (td_inited/td_created) are not mergeable."
    kind: functional
    risk: high
    verify: test
  linear_chain:
    id: R3
    text: "next_phase_command encodes the linear chain: cb_genned -> aw cb fill, cb_filled -> aw td merge."
    kind: functional
    risk: medium
    verify: test
elements:
  td_created_dispatches_to_gen:
    kind: test
    type: "rs/#[test]"
  is_mergeable_linear_only_genned_filled:
    kind: test
    type: "rs/#[test]"
  next_phase_command_is_linear:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: td_created_dispatches_to_gen, verifies: create_to_gen }
  - { from: is_mergeable_linear_only_genned_filled, verifies: mergeable_linear }
  - { from: next_phase_command_is_linear, verifies: linear_chain }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "td_created routes to aw td gen"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "is_mergeable accepts only cb_genned|cb_filled"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "next_phase_command is linear"
      risk: medium
      verifymethod: test
    }
    element td_created_dispatches_to_gen {
      type: "rs/#[test]"
    }
    element is_mergeable_linear_only_genned_filled {
      type: "rs/#[test]"
    }
    element next_phase_command_is_linear {
      type: "rs/#[test]"
    }
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract is stable: a pure next_phase_command(phase) encodes td_created->gen, cb_genned->fill, cb_filled->merge; is_mergeable trimmed to cb_genned|cb_filled; review/revise/arbitrate removed. Implementable as the surgical change-list.
- [unit-test] R1-R3 map to pure-function tests (td_created_dispatches_to_gen, is_mergeable_linear_only_genned_filled, next_phase_command_is_linear); bulk deletion guarded by build + full suite.
