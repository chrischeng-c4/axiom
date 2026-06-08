---
id: sdd-ui-tables-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Table

## Overview
<!-- type: overview lang: markdown -->

Single Table unit struct (placeholder).

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Table:
    type: object
    description: Table placeholder unit struct.
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/tables.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - Table
    description: Codegen replaces Table.
  - path: projects/agentic-workflow/src/ui/tables.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: Module preamble.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- ok.
