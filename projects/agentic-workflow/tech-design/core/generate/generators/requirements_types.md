---
id: sdd-generate-generators-requirements-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# RequirementsGenerator

## Overview
<!-- type: overview lang: markdown -->

Single RequirementsGenerator unit struct. A companion source template owns the
module preamble, `Generator` impl, helper behavior, and regression tests that
previously lived in a managed HANDWRITE gap.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RequirementsGenerator:
    type: object
    description: RequirementsGenerator unit struct.
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/requirements.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - RequirementsGenerator
    description: Codegen replaces RequirementsGenerator.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- ok.
