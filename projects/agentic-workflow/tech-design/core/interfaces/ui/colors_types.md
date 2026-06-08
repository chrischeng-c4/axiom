---
id: sdd-ui-colors-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# ColorScheme

## Overview
<!-- type: overview lang: markdown -->

Single ColorScheme unit struct (placeholder).

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ColorScheme:
    type: object
    description: ColorScheme placeholder unit struct.
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/colors.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ColorScheme
    description: Codegen replaces ColorScheme.
  - path: projects/agentic-workflow/src/ui/colors.rs
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
