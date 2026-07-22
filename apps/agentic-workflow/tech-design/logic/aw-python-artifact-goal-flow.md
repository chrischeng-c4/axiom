---
id: aw-python-artifact-goal-flow
summary: "Route python-v1 work-item, capability, and backlog roots through one EC-first artifact/gate phase table."
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: python-artifact-goal-flow
    claim: python-artifact-goal-flow
    coverage: partial
    rationale: "Every root must select the same Python artifact phase without changing legacy project routing."
---

# Python Artifact Goal Flow

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-python-artifact-goal-flow
entry: resolve
nodes:
  resolve: { kind: start, label: "resolve project artifact_model and tracker phase" }
  legacy: { kind: terminal, label: "use unchanged legacy lifecycle table" }
  ec_check: { kind: process, label: "author direct EC Python then aw ec check" }
  review: { kind: process, label: "independent digest-bound EC review" }
  td: { kind: process, label: "author/check Python TD then generate target" }
  core: { kind: process, label: "unit tests then behavior/security EC" }
  operational: { kind: process, label: "stability/efficiency EC after generation" }
  terminal: { kind: terminal, label: "code-check then close change" }
  contract: { kind: process, label: "stale oracle/evidence returns to EC check" }
  efficiency: { kind: process, label: "efficiency red routes to Rust production target" }
edges:
  - { from: resolve, to: legacy, label: "legacy" }
  - { from: resolve, to: ec_check, label: "python-v1 ec_missing" }
  - { from: ec_check, to: review }
  - { from: review, to: td }
  - { from: td, to: core }
  - { from: core, to: operational }
  - { from: operational, to: terminal }
  - { from: resolve, to: contract, label: "stale contract or invalid oracle" }
  - { from: resolve, to: efficiency, label: "efficiency red" }
---
flowchart TD
  resolve([resolve model plus phase]) -->|legacy| legacy([legacy table])
  resolve -->|python-v1| ec_check[EC check]
  ec_check --> review[independent EC review]
  review --> td[TD check and target generation]
  td --> core[unit tests then behavior/security]
  core --> operational[stability/efficiency]
  operational --> terminal([code-check then close])
  resolve -->|stale/oracle invalid| contract[repair EC]
  resolve -->|efficiency red| efficiency[Rust target remediation]
```

`python-v1` is an explicit artifact-model adapter, never a file heuristic. The
single phase table is called by `aw goal wi`; backlog reaches it by selecting
and probing the same WI; capability invokes it for its active WI. Legacy
projects receive no new phase interpretation. A successful digest-bound review
returns `aw ec lock --project <project> --wi <wi>`; the lock transition itself
persists `aw td check <td-root> --project <project> --wi <wi>`, so a root cannot
loop forever on a project-global lock operation.

The canonical progression is `ec_missing` → `ec_checked` → `ec_reviewed` →
`td_compiled` → `td_generated` → `unit_green` → `ec_core_green` →
`ec_operational_green` → `code_checked`. Behavior/security red, stability red,
and efficiency red are separate recovery branches. A stale contract or invalid
oracle always routes to EC check/review rather than product adaptation. EC
review HITL is surfaced as a HITL envelope, not a false terminal action.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-python-artifact-goal-flow-unit-tests
requirements:
  phase_table: { id: R1, text: "python-v1 phases emit EC-first runnable commands through terminal code-check.", kind: contract, risk: high, verify: "cargo test -p agentic-workflow python_artifact_goal_routing -- --nocapture" }
  dimensions: { id: R2, text: "red dimensions and stale contracts route to their distinct owner.", kind: regression, risk: high, verify: "cargo test -p agentic-workflow python_artifact_goal_routing -- --nocapture" }
  legacy: { id: R3, text: "legacy projects remain on the existing lifecycle table.", kind: regression, risk: high, verify: "cargo test -p agentic-workflow python_artifact_goal_routing -- --nocapture" }
elements:
  python_artifact_goal_routing_uses_one_ec_first_phase_table: { kind: test, type: "rs/#[test]" }
  python_artifact_goal_routing_separates_red_dimensions_and_contract_repairs: { kind: test, type: "rs/#[test]" }
  python_artifact_goal_routing_keeps_legacy_projects_on_the_legacy_table: { kind: test, type: "rs/#[test]" }
relations:
  - { from: python_artifact_goal_routing_uses_one_ec_first_phase_table, verifies: phase_table }
  - { from: python_artifact_goal_routing_separates_red_dimensions_and_contract_repairs, verifies: dimensions }
  - { from: python_artifact_goal_routing_keeps_legacy_projects_on_the_legacy_table, verifies: legacy }
---
requirementDiagram
  requirement R1 { id: R1 text: "EC-first phase table" risk: high verifymethod: test }
  requirement R2 { id: R2 text: "dimension-aware recovery" risk: high verifymethod: test }
  requirement R3 { id: R3 text: "legacy compatibility" risk: high verifymethod: test }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/run.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the shared python-v1 artifact/gate resolver and route WI/backlog envelopes through it."
  - path: apps/agentic-workflow/src/cli/capability.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Route active capability WIs through the shared python-v1 resolver while retaining legacy dispatch."
  - path: apps/agentic-workflow/src/cli/ec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Persist direct Python EC checks and staged core/operational verdict transitions on the owning WI."
  - path: apps/agentic-workflow/src/cli/cb.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Persist the core EC handoff after target-native Python generation when an owning WI is supplied."
  - path: apps/agentic-workflow/src/cli/td.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Persist target-native generation after a checked Python TD project owned by a WI."
```
