---
id: aw-reviewed-graph-goal-selection
fill_sections: [overview, requirements, behavior, logic, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: goal-unified-loop-verb
    claim: goal-unified-loop-verb
    coverage: full
    rationale: "Epic and backlog roots must consume one reviewed graph and deterministically agree on the next executable change."
command_refs:
  - command: aw goal wi
  - command: aw goal backlog
  - command: aw wi plan
  - command: aw wi graph
---

# Prioritized Reviewed-Graph Goal Selection

## Overview
<!-- type: overview lang: markdown -->

The accepted and completely published project plan is the scheduling authority
for `aw goal wi <epic>` and `aw goal backlog --project <project>`. Both roots
load the same review digest, publication checkpoint, current issue inventory,
and canonical epic/change graph before selecting a change.

Selection is hierarchical: epic priority chooses project direction first;
dependency readiness and explicit or inherited change priority choose a leaf
only within that epic. The selector is a pure issue-domain policy. Backlog
parking is an outer runtime concern and never changes graph metadata.

## Requirements
<!-- type: requirements lang: yaml -->

```yaml
requirements:
  - id: R1
    text: Epic and backlog roots consume only the current accepted and completely published project graph.
  - id: R2
    text: Both roots select the same next change for the same graph and exclusion set.
  - id: R3
    text: Epic priority is evaluated before dependency readiness and explicit or inherited child priority.
  - id: R4
    text: A blocked high-priority change is reported and skipped without hiding another ready leaf.
  - id: R5
    text: Invalid ownership, change-as-parent, stale graph labels, stale review provenance, and unresolved dependencies fail closed with executable remediation.
  - id: R6
    text: An open epic whose reviewed children are all closed reuses terminal child rollup and is never re-atomized.
  - id: R7
    text: Selection and parking do not write graph metadata and repeated unchanged reads are deterministic.
```

## Behavior
<!-- type: behavior lang: gherkin -->

```gherkin
Feature: choose one ready change from a reviewed epic graph

  Scenario: epic direction precedes child priority
    Given a p0 epic with a ready p1 child
    And a p1 epic with a ready p0 child
    When either epic or backlog goal selection runs
    Then the p0 epic's p1 child is selected

  Scenario: blocked leaf does not hide ready work
    Given the selected epic has one dependency-blocked p0 child and one ready p1 child
    When selection runs
    Then the blocker is retained in the result
    And the ready p1 sibling is selected

  Scenario: terminal epic rolls up
    Given an open reviewed epic whose children are all closed
    When aw goal wi runs for that epic
    Then it emits aw wi close <epic> --push
    And it never emits aw wi atomize

  Scenario: post-publication graph drift fails closed
    Given one accepted completed project-plan transaction
    When an ownership or priority graph label changes
    Then no change root is dispatched
    And the blocked envelope identifies the stale issue and replanning command
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: reviewed-graph-goal-selection
entry: load_review
nodes:
  load_review: { kind: start, label: "load accepted review and completed transaction" }
  verify_graph: { kind: decision, label: "published graph valid and current?" }
  order_epics: { kind: process, label: "order open epics by priority and stable id" }
  inspect_children: { kind: decision, label: "ready child in current epic?" }
  select_child: { kind: process, label: "order ready children by effective priority and stable id" }
  rollup_epic: { kind: terminal, label: "all reviewed children closed: close epic" }
  dispatch: { kind: terminal, label: "dispatch one change root" }
  blocked: { kind: terminal, label: "fail closed with exact remediation" }
edges:
  - { from: load_review, to: verify_graph, label: "digest and inventory" }
  - { from: verify_graph, to: blocked, label: "stale or invalid" }
  - { from: verify_graph, to: order_epics, label: "valid" }
  - { from: order_epics, to: inspect_children, label: "highest direction" }
  - { from: inspect_children, to: rollup_epic, label: "all closed" }
  - { from: inspect_children, to: select_child, label: "ready leaves" }
  - { from: inspect_children, to: order_epics, label: "only blocked or parked" }
  - { from: select_child, to: dispatch, label: "first stable leaf" }
---
flowchart TD
    load_review([accepted review + completed transaction]) --> verify_graph{current valid graph?}
    verify_graph -- no --> blocked([blocked + exact remediation])
    verify_graph -- yes --> order_epics[epic priority then stable id]
    order_epics --> inspect_children{children state}
    inspect_children -- all closed --> rollup_epic([close epic])
    inspect_children -- ready --> select_child[effective child priority then stable id]
    inspect_children -- blocked or parked --> order_epics
    select_child --> dispatch([dispatch one change])
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
inputs:
  review: /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/project-plan.review.json
  manifest: /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/project-plan.manifest.json
  checkpoint: /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/project-plan.transaction.json
roots:
  epic: aw goal wi <epic-id>
  backlog: aw goal backlog --project <project>
dispatch: aw goal wi <change-id>
terminal_epic: aw wi close <epic-id> --push
stale_plan_next: aw wi plan --project <project> --json
invalid_issue_next: aw wi show <issue-id>
```

## Unit Test
<!-- type: unit-test lang: yaml -->

```yaml
gates:
  - cargo test -p agentic-workflow --lib ready_graph::tests -- --nocapture
  - cargo test -p agentic-workflow --lib planning_transaction::tests -- --nocapture
proof:
  - epic priority precedes child priority
  - blocked dependencies do not hide ready siblings
  - all-closed children produce terminal epic selection
  - unresolved dependencies fail closed
  - lifecycle metadata may advance while graph-label drift invalidates publication
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
gates:
  - cargo test -p agentic-workflow --test wi_reviewed_graph_goal_cli_test -- --nocapture
  - cargo test -p agentic-workflow --test cli_tests goal_backlog -- --nocapture
proof:
  - compiled epic and backlog roots select the same change
  - backlog parks a blocked high-priority leaf and dispatches ready work
  - all-closed children produce the terminal epic close command without atomize
  - stale priority labels and invalid ownership fail closed without dispatch
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/ready_graph.rs
    action: create
    section: logic
    impl_mode: handwrite
    description: "Pure deterministic ready-leaf selector over the canonical epic/change graph."
  - path: apps/agentic-workflow/src/issues/planning_transaction.rs
    action: modify
    section: logic
    impl_mode: handwrite
    description: "Read-only admission verifier for completed reviewed graph metadata."
  - path: apps/agentic-workflow/src/cli/run.rs
    action: modify
    section: logic
    impl_mode: handwrite
    description: "Shared reviewed-graph loading, backlog parking, epic dispatch, and terminal rollup."
  - path: apps/agentic-workflow/tests/wi_reviewed_graph_goal_cli_test.rs
    action: create
    section: e2e-test
    impl_mode: handwrite
    description: "Compiled parity, blocker continuation, terminal, stale-plan, and invalid-graph proof."
```
