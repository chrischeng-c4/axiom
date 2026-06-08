---
id: sdd-validate-r3g-rust-type-consistency-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# RustTypeConsistencyRule

## Overview
<!-- type: overview lang: markdown -->

Single RustTypeConsistencyRule unit struct in validate/rules/r3g_rust_type_consistency.rs.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RustTypeConsistencyRule:
    type: object
    description: RustTypeConsistencyRule validation rule (unit struct).
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r3g_rust_type_consistency.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - RustTypeConsistencyRule
    description: Codegen replaces RustTypeConsistencyRule unit struct.
  - path: projects/agentic-workflow/src/validate/rules/r3g_rust_type_consistency.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: Module preamble, impl Rule, helpers, tests.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- ok.
