---
id: sdd-generate-generators-doc-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# DocGenerator

## Overview
<!-- type: overview lang: markdown -->

Single DocGenerator unit struct. A companion source template owns the module
preamble, `Generator` impl, helper behavior, and regression tests that
previously lived in a managed HANDWRITE gap.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  DocGenerator:
    type: object
    description: DocGenerator unit struct.
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/doc.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - DocGenerator
    description: Codegen replaces DocGenerator.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- ok.
