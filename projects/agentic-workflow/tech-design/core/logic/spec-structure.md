---
id: sdd-fill-order-topdown
main_spec_ref: projects/agentic-workflow/logic/spec-structure.md
merge_strategy: new
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "This logic TD supports TD/CB artifact lifecycle state, authoring, review, validation, or merge behavior."
---

# Sdd Fill Order Topdown

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Requirements
<!-- type: doc lang: markdown -->

### R1: Top-down fill order

Reorder `SectionType::fill_order()` in `projects/agentic-workflow/src/models/spec_rules.rs` so sections fill in human-reasoning order: overview (0) → requirements (1) → scenarios (2) → mindmap (3) → state-machine (4) → interaction (5) → logic (6) → dependency (7) → db-model (8) → schema (9) → rest-api (10) → rpc-api (11) → async-api (12) → cli (13) → wireframe (14) → component (15) → design-token (16) → config (17) → test-plan (18) → changes (19) → doc (20).

**Priority**: high

### R2: ALL_SECTIONS constant updated

Update `ALL_SECTIONS` constant in `projects/agentic-workflow/src/tools/common_change_spec.rs` to reflect the new top-down order matching `fill_order()` values.

**Priority**: high

### R3: Tests updated

Update any tests that assert specific fill order sequences (e.g. `test_all_in_fill_order`, `test_fill_order_*`) to match the new ordering. Tests must pass after the reorder.

**Priority**: high
## Scenarios
<!-- type: doc lang: markdown -->

<!-- TODO -->

## Diagrams
<!-- type: doc lang: markdown -->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

## API Spec
<!-- type: doc lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Schema
<!-- type: schema lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Config
<!-- type: config lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

## Test Plan
<!-- type: doc lang: markdown -->

<!-- TODO -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: projects/agentic-workflow/src/models/spec_rules.rs
    section: source
    action: modify
    impl_mode: codegen
    desc: |
      Reorder fill_order() match arms to: overview=0, requirements=1, scenarios=2,
      mindmap=3, state-machine=4, interaction=5, logic=6, dependency=7, db-model=8,
      schema=9, rest-api=10, rpc-api=11, async-api=12, cli=13, wireframe=14,
      component=15, design-token=16, config=17, test-plan=18, changes=19, doc=20.

  - path: projects/agentic-workflow/src/tools/common_change_spec.rs
    section: source
    action: modify
    impl_mode: codegen
    desc: |
      Update ALL_SECTIONS constant to match new top-down fill order:
      [overview, requirements, scenarios, mindmap, state-machine, interaction,
      logic, dependency, db-model, schema, rest-api, rpc-api, async-api, cli,
      wireframe, component, design-token, config, test-plan, changes, doc]

  - path: projects/agentic-workflow/src/models/spec_rules.rs (tests)
    impl_mode: hand-written
    action: modify
    section: logic
    desc: |
      Update test assertions for fill_order() and all_in_fill_order() to match
      new ordering. Ensure no test hardcodes the old bottom-up order.
  - action: annotate
    section: async-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the async-api section."

  - action: annotate
    section: cli
    impl_mode: hand-written
    description: "Traceability metadata edge for the cli section."

  - action: annotate
    section: component
    impl_mode: hand-written
    description: "Traceability metadata edge for the component section."

  - action: annotate
    section: config
    impl_mode: hand-written
    description: "Traceability metadata edge for the config section."

  - action: annotate
    section: db-model
    impl_mode: hand-written
    description: "Traceability metadata edge for the db-model section."

  - action: annotate
    section: dependency
    impl_mode: hand-written
    description: "Traceability metadata edge for the dependency section."

  - action: annotate
    section: design-token
    impl_mode: hand-written
    description: "Traceability metadata edge for the design-token section."

  - action: annotate
    section: interaction
    impl_mode: hand-written
    description: "Traceability metadata edge for the interaction section."

  - action: annotate
    section: rest-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rest-api section."

  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

  - action: annotate
    section: state-machine
    impl_mode: hand-written
    description: "Traceability metadata edge for the state-machine section."

  - action: annotate
    section: wireframe
    impl_mode: hand-written
    description: "Traceability metadata edge for the wireframe section."

```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
wireframes: []
```

## Component
<!-- type: component lang: yaml -->

```yaml
components: []
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
tokens: []
```

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->
