---
id: sdd-generate-generators-fastapi
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# FastAPIGenerator Type

## Overview
<!-- type: overview lang: markdown -->

Unit-struct generator in
`projects/agentic-workflow/src/generate/generators/fastapi.rs`. One shape:

- `FastAPIGenerator` — unit struct with no derives.

Codegen replaces only the unit struct declaration. Module imports,
the `impl FastAPIGenerator { new, ... }` block, the
`impl Generator for FastAPIGenerator` (or similar trait impls),
and tests are owned by sibling source templates that replace the legacy
HANDWRITE gaps.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FastAPIGenerator:
    type: object
    required: []
    properties: {}
    description: FastAPI generator (unit struct).
    x-rust-struct:
      derive: []
      unit: true
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/fastapi.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - FastAPIGenerator
    description: |
      Codegen replaces the unit struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Single unit struct.
- [schema] Standard unit-struct shape.
- [changes] Standard split.
