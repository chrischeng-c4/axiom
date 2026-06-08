---
id: sdd-generate-generators-async-api-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AsyncApiGenerator

## Overview
<!-- type: overview lang: markdown -->

Single AsyncApiGenerator unit struct in generators/async_api.rs. A companion
source template owns the module preamble, `Generator` impl, helper behavior, and
regression tests that previously lived in a managed HANDWRITE gap.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  AsyncApiGenerator:
    type: object
    description: AsyncApiGenerator unit struct (registered in generators/mod.rs).
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/async_api.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - AsyncApiGenerator
    description: Codegen replaces AsyncApiGenerator.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- ok.
