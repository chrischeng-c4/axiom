---
id: sdd-generate-generators-flowchart-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# FlowchartGenerator

## Overview
<!-- type: overview lang: markdown -->

Single FlowchartGenerator unit struct. A companion source template owns the
module preamble, `Generator` impl, and regression tests that previously lived in
a managed HANDWRITE gap.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FlowchartGenerator:
    type: object
    description: FlowchartGenerator unit struct.
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/flowchart.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - FlowchartGenerator
    description: Codegen replaces FlowchartGenerator.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- ok.
