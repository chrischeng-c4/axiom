---
id: sdd-generate-generators-changes-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ChangesGenerator

## Overview
<!-- type: overview lang: markdown -->

Single ChangesGenerator unit struct in generators/changes.rs. A companion source
template owns the module preamble, `Generator` impl, helper behavior, and
regression tests that previously lived in a managed HANDWRITE gap.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ChangesGenerator:
    type: object
    description: ChangesGenerator unit struct (registered in generators/mod.rs).
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/changes.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ChangesGenerator
    description: Codegen replaces ChangesGenerator.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- ok.
