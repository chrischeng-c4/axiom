---
id: sdd-parser-scenario-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# ScenarioParser

## Overview
<!-- type: overview lang: markdown -->

Single ScenarioParser unit struct (placeholder).

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ScenarioParser:
    type: object
    required: []
    description: ScenarioParser placeholder unit struct.
    properties: {}
    x-rust-struct:
      derive: []
      unit: true
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/parser/scenario.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ScenarioParser
    description: Codegen replaces ScenarioParser.
  - path: projects/agentic-workflow/src/parser/scenario.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: Module preamble.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
