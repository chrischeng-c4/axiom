---
id: sdd-validate-r3e-impl-mode-misuse-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# ImplModeMisuseRule

## Overview
<!-- type: overview lang: markdown -->

Single ImplModeMisuseRule unit struct in validate/rules/r3e_impl_mode_misuse.rs.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ImplModeMisuseRule:
    type: object
    description: ImplModeMisuseRule validation rule (unit struct).
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r3e_impl_mode_misuse.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ImplModeMisuseRule
    description: Codegen replaces ImplModeMisuseRule unit struct.
  - path: projects/agentic-workflow/src/validate/rules/r3e_impl_mode_misuse.rs
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
