---
id: sdd-generate-generators-flowchart-plus-gen
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# FlowchartPlusGenerator Type

## Overview
<!-- type: overview lang: markdown -->

Unit-struct generator in
`projects/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs`. One shape:

- `FlowchartPlusGenerator` — unit struct with no derives.

Codegen replaces the unit struct declaration. Companion source templates own
the module preamble and runtime implementation blocks that previously lived in
managed HANDWRITE gaps.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FlowchartPlusGenerator:
    type: object
    required: []
    properties: {}
    description: FlowchartPlus generator (unit struct).
    x-rust-struct:
      derive: []
      unit: true
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - FlowchartPlusGenerator
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
