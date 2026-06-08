---
id: sdd-context-builder
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Change/context/git/spec-store logic supports TD/CB artifact lifecycle dispatch and review state."
---

# ContextBuilder Type

## Overview
<!-- type: overview lang: markdown -->

Context builder pipeline orchestrator in
`projects/agentic-workflow/src/context_builder/mod.rs`. One shape with lifetime generic:

- `ContextBuilder<'a>` — 4 reference fields + 1 owned Vec, no derives.

Codegen replaces only the struct declaration. Module imports, the
`impl<'a> ContextBuilder<'a> { new, ... }` block stay hand-written.

This spec exercises:

1. **Lifetime generic** — `x-rust-generics: ["'a"]` produces `struct ContextBuilder<'a>`.
2. **Reference field types** — `&'a ImportGraph` etc. via x-rust-type.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ContextBuilder:
    type: object
    required: [import_graph, call_graph, symbol_tables, type_context, project_files]
    description: Builder that orchestrates the context building pipeline.
    properties:
      import_graph:
        type: object
        x-rust-type: "&'a ImportGraph"
        x-rust-visibility: private
        description: "Import graph reference."
      call_graph:
        type: object
        x-rust-type: "&'a CallGraphIndex"
        x-rust-visibility: private
        description: "Call graph index reference."
      symbol_tables:
        type: object
        x-rust-type: "&'a HashMap<std::path::PathBuf, SymbolTable>"
        x-rust-visibility: private
        description: "Symbol tables keyed by path."
      type_context:
        type: object
        x-rust-type: "Option<&'a TypeContext>"
        x-rust-visibility: private
        description: "Optional type context."
      project_files:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-rust-visibility: private
        description: "Project file list."
    x-rust-struct:
      derive: []
    x-rust-generics: ["'a"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/context_builder/mod.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ContextBuilder
    description: |
      Codegen replaces the struct declaration with the lifetime generic.
  - path: projects/agentic-workflow/src/context_builder/mod.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, submodule
      declarations, imports, the `impl<'a> ContextBuilder<'a>` block,
      and helpers.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] Single struct with lifetime generic, reference fields.
- [schema] Lifetime via x-rust-generics; references via x-rust-type.
- [changes] Standard split.
