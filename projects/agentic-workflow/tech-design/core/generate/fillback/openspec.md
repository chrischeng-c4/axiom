---
id: sdd-fillback-openspec
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# OpenSpecStrategy Type

## Overview
<!-- type: overview lang: markdown -->

Import strategy unit struct in
`projects/agentic-workflow/src/fillback/openspec.rs`. One shape:

- `OpenSpecStrategy` — unit struct with no derives.

Codegen replaces the unit struct declaration. Companion source specs own the
module imports, OpenSpec DTOs, conversion helpers, strategy implementation,
and tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  OpenSpecStrategy:
    type: object
    required: []
    properties: {}
    description: |
      OpenSpec import strategy (parses OpenSpec YAML/JSON specs).
    x-rust-struct:
      derive: []
      unit: true
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/openspec.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - OpenSpecStrategy
    description: |
      Codegen replaces the unit struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Single unit struct; impls hand-written.
- [schema] Standard unit-struct shape.
- [changes] Standard split.
