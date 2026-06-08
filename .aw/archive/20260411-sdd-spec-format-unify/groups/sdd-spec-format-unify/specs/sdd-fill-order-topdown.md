---
id: sdd-fill-order-topdown
main_spec_ref: crates/sdd/logic/spec-structure.md
merge_strategy: new
fill_sections: [requirements, changes]
filled_sections: [requirements, changes]
create_complete: true
---

# Sdd Fill Order Topdown

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Requirements

<!-- type: requirements lang: markdown -->

### R1: Top-down fill order

Reorder `SectionType::fill_order()` in `crates/sdd/src/models/spec_rules.rs` so sections fill in human-reasoning order: overview (0) → requirements (1) → scenarios (2) → mindmap (3) → state-machine (4) → interaction (5) → logic (6) → dependency (7) → db-model (8) → schema (9) → rest-api (10) → rpc-api (11) → async-api (12) → cli (13) → wireframe (14) → component (15) → design-token (16) → config (17) → test-plan (18) → changes (19) → doc (20).

**Priority**: high

### R2: ALL_SECTIONS constant updated

Update `ALL_SECTIONS` constant in `crates/sdd/src/tools/common_change_spec.rs` to reflect the new top-down order matching `fill_order()` values.

**Priority**: high

### R3: Tests updated

Update any tests that assert specific fill order sequences (e.g. `test_all_in_fill_order`, `test_fill_order_*`) to match the new ordering. Tests must pass after the reorder.

**Priority**: high
## Scenarios
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/sdd/src/models/spec_rules.rs
    action: modify
    desc: |
      Reorder fill_order() match arms to: overview=0, requirements=1, scenarios=2,
      mindmap=3, state-machine=4, interaction=5, logic=6, dependency=7, db-model=8,
      schema=9, rest-api=10, rpc-api=11, async-api=12, cli=13, wireframe=14,
      component=15, design-token=16, config=17, test-plan=18, changes=19, doc=20.

  - path: crates/sdd/src/tools/common_change_spec.rs
    action: modify
    desc: |
      Update ALL_SECTIONS constant to match new top-down fill order:
      [overview, requirements, scenarios, mindmap, state-machine, interaction,
      logic, dependency, db-model, schema, rest-api, rpc-api, async-api, cli,
      wireframe, component, design-token, config, test-plan, changes, doc]

  - path: crates/sdd/src/models/spec_rules.rs (tests)
    action: modify
    desc: |
      Update test assertions for fill_order() and all_in_fill_order() to match
      new ordering. Ensure no test hardcodes the old bottom-up order.
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
