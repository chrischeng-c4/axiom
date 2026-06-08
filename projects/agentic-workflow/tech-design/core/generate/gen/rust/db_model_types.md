---
id: sdd-gen-rust-db_model-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# DbModelGenOutput

## Overview
<!-- type: overview lang: markdown -->

`DbModelGenOutput` is generated in the canonical Rust codegen module at
`projects/agentic-workflow/src/generate/gen/rust/db_model.rs` and the legacy mirror at
`projects/agentic-workflow/src/gen/rust/db_model.rs`.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  DbModelGenOutput:
    type: object
    description: Output from DB-model code generation.
    properties:
      code:
        type: string
        description: The generated Rust struct(s) with sqlx derives.
    required: [code]
    x-rust-struct:
      derive: [Debug, Clone]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/db_model.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - DbModelGenOutput
    description: Codegen replaces DbModelGenOutput.
  - path: projects/agentic-workflow/src/gen/rust/db_model.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - DbModelGenOutput
    description: Codegen replaces the legacy mirror of DbModelGenOutput.
  - path: projects/agentic-workflow/src/generate/gen/rust/db_model.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: Module preamble, free fns, helpers, tests.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
