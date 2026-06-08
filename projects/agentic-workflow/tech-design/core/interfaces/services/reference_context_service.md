---
id: sdd-services-reference-context
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Reference Context Service Types

## Overview
<!-- type: overview lang: markdown -->

Input types for the reference-context service in
`projects/agentic-workflow/src/services/reference_context_service.rs`. Four shapes:

- `CreateSpecContextInput` — spec context (change_id, complexity, iteration u32, scanned_groups, specs Vec<SpecRef>, dependencies, gaps).
- `CreateKnowledgeContextInput` — knowledge context (similar shape with docs/patterns/pitfalls).
- `CreateCodebaseContextInput` — codebase context (with files, lens_results, dependency_graph).
- `CreateContextInput` — dispatching enum:
  - `Spec(CreateSpecContextInput)` (tuple variant)
  - `Knowledge(CreateKnowledgeContextInput)` (tuple variant)
  - `Codebase(CreateCodebaseContextInput)` (tuple variant)
  - `Gap { change_id, context_type, content }` (struct variant)

All structs derive `[Debug, Clone]`. Codegen replaces all four type
declarations. Companion source templates own the module documentation, imports,
rendering, validation, file writing, dispatch behavior, and test-module link.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  CreateSpecContextInput:
    type: object
    required: [change_id, complexity, iteration, scanned_groups, specs, dependencies, gaps]
    description: Input for creating a spec context artifact.
    properties:
      change_id:
        type: string
        description: "Change identifier."
      complexity:
        type: string
        description: "Complexity tier."
      iteration:
        type: integer
        x-rust-type: "u32"
        description: "Iteration number."
      scanned_groups:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Scanned spec groups."
      specs:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SpecRef>"
        description: "Specs found."
      dependencies:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Dependencies."
      gaps:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Identified gaps."
    x-rust-struct:
      derive: [Debug, Clone]

  CreateKnowledgeContextInput:
    type: object
    required: [change_id, complexity, iteration, scanned_categories, docs, patterns, pitfalls]
    description: Input for creating a knowledge context artifact.
    properties:
      change_id:
        type: string
        description: "Change identifier."
      complexity:
        type: string
        description: "Complexity tier."
      iteration:
        type: integer
        x-rust-type: "u32"
        description: "Iteration number."
      scanned_categories:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Scanned knowledge categories."
      docs:
        type: array
        items: { type: object }
        x-rust-type: "Vec<DocRef>"
        description: "Documents found."
      patterns:
        type: array
        items: { type: object }
        x-rust-type: "Vec<PatternRef>"
        description: "Patterns identified."
      pitfalls:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Pitfalls."
    x-rust-struct:
      derive: [Debug, Clone]

  CreateCodebaseContextInput:
    type: object
    required: [change_id, complexity, iteration, lens_tools_used, files, lens_results, dependency_graph]
    description: Input for creating a codebase context artifact.
    properties:
      change_id:
        type: string
        description: "Change identifier."
      complexity:
        type: string
        description: "Complexity tier."
      iteration:
        type: integer
        x-rust-type: "u32"
        description: "Iteration number."
      lens_tools_used:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Lens tools invoked."
      files:
        type: array
        items: { type: object }
        x-rust-type: "Vec<FileRef>"
        description: "Files scanned."
      lens_results:
        type: array
        items: { type: object }
        x-rust-type: "Vec<LensResult>"
        description: "Lens results."
      dependency_graph:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Dependency graph entries."
    x-rust-struct:
      derive: [Debug, Clone]

  CreateContextInput:
    type: string
    enum: [Spec, Knowledge, Codebase, Gap]
    description: Unified enum for dispatching to the correct creation path.
    x-rust-enum:
      derive: []
      variants:
        - name: Spec
          kind: tuple
          fields:
            - { rust_type: CreateSpecContextInput }
        - name: Knowledge
          kind: tuple
          fields:
            - { rust_type: CreateKnowledgeContextInput }
        - name: Codebase
          kind: tuple
          fields:
            - { rust_type: CreateCodebaseContextInput }
        - name: Gap
          kind: struct
          doc: "Gap analysis artifacts (free-form markdown)."
          fields:
            - { name: change_id, rust_type: String }
            - { name: context_type, rust_type: String }
            - { name: content, rust_type: String }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/reference_context_service.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - CreateSpecContextInput
      - CreateKnowledgeContextInput
      - CreateCodebaseContextInput
      - CreateContextInput
    description: |
      Codegen replaces all four type declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 3 input structs + 1 dispatch enum (mixed tuple + struct variants).
- [schema] All in `required:`; foreign-type fields via x-rust-type.
- [changes] Standard split with all four in `replaces`.
