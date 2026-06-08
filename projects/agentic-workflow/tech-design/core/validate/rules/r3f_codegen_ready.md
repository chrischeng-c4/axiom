---
id: sdd-validate-r3f-codegen-ready-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: cb-and-cold-verification-gates
    claim: cb-and-cold-verification-gates
    coverage: full
    rationale: "Codegen/audit validation TDs implement CB and cold verification gates for production readiness."
---

# CodegenReadyRule

## Overview
<!-- type: overview lang: markdown -->

Single CodegenReadyRule unit struct in validate/rules/r3f_codegen_ready.rs.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  CodegenReadyRule:
    type: object
    description: CodegenReadyRule validation rule (unit struct).
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/rules/r3f_codegen_ready.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - CodegenReadyRule
    description: Codegen replaces CodegenReadyRule unit struct.
  - path: projects/agentic-workflow/src/validate/rules/r3f_codegen_ready.rs
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
