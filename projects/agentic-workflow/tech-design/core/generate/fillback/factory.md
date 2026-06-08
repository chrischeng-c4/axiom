---
id: sdd-fillback-factory
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# StrategyFactory Type

## Overview
<!-- type: overview lang: markdown -->

Factory for creating import strategy instances in
`projects/agentic-workflow/src/fillback/factory.rs`. One shape:

- `StrategyFactory` — unit struct with no fields and no derives.
  Acts as a namespace for the static `create` and private
  `auto_detect` factory functions in the hand-written
  `impl StrategyFactory` block.

Codegen replaces the unit struct declaration. Companion source specs own the
factory imports and the runtime implementation/tests.

This spec exercises:

1. **Unit-struct emission** — `properties: {}` with empty
   `required: []` plus `x-rust-struct.derive: []` produces a bare
   `pub struct StrategyFactory;` declaration. Same shape as
   `AutoApproveHandler`.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  StrategyFactory:
    type: object
    required: []
    properties: {}
    description: |
      Factory for creating import strategy instances. Unit struct;
      behaviour lives on a hand-written impl block with `create`
      and `auto_detect` static methods.
    x-rust-struct:
      derive: []
      unit: true
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/factory.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - StrategyFactory
    description: |
      Codegen replaces the unit struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Correctly identifies the unit struct, namespace role, and hand-written impl boundary.
- [schema] Definition is well-formed: `properties: {}` + `required: []` + `x-rust-struct.derive: []` + `unit: true` matches the AutoApproveHandler precedent.
- [changes] Two entries cleanly split codegen vs hand-written. `replaces` lists the single struct name; hand-written entry covers all imports and the impl block.
