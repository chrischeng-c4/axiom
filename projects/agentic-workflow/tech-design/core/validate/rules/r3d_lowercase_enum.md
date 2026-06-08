---
id: sdd-validate-r3d-lowercase-enum-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# LowercaseEnumRule

## Overview
<!-- type: overview lang: markdown -->

Single LowercaseEnumRule unit struct in validate/rules/r3d_lowercase_enum.rs.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  LowercaseEnumRule:
    type: object
    description: LowercaseEnumRule validation rule (unit struct).
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r3d_lowercase_enum.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - LowercaseEnumRule
    description: Codegen replaces LowercaseEnumRule unit struct.
  - path: projects/agentic-workflow/src/validate/rules/r3d_lowercase_enum.rs
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
