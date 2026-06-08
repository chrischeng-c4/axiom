---
id: sdd-ui-progress-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# ProgressBar

## Overview
<!-- type: overview lang: markdown -->

Single ProgressBar unit struct (placeholder).

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ProgressBar:
    type: object
    description: ProgressBar placeholder unit struct.
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/progress.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ProgressBar
    description: Codegen replaces ProgressBar.
  - path: projects/agentic-workflow/src/ui/progress.rs
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
