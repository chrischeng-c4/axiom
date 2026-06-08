---
id: sdd-generate-engine-tera-engine
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TemplateEngine Type

## Overview
<!-- type: overview lang: markdown -->

Tera template engine wrapper in
`projects/agentic-workflow/src/generate/engine/tera_engine.rs`. One shape:

- `TemplateEngine` — single private `tera: Tera` field with no
  derives. Wraps the `tera::Tera` engine to provide template
  loading, registration of custom filters, and rendering. All
  behaviour lives on the source-template-owned `impl TemplateEngine` block
  (`new`, `empty`, `add_template`, `render`, etc.).

Codegen replaces the struct declaration. Companion source templates own the
module preamble and runtime implementation blocks that previously lived in
managed HANDWRITE gaps.

This spec exercises:

1. **No-derive struct emission** — `x-rust-struct.derive: []` emits
   `pub struct TemplateEngine { ... }` with no `#[derive(...)]`.
2. **`x-rust-visibility: private`** on the only field — keeps
   `tera: Tera` private (no `pub`) on a public struct.
3. **`x-rust-type: "Tera"`** in `required:` — uses the bare
   foreign type without Option auto-wrapping.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TemplateEngine:
    type: object
    required: [tera]
    description: |
      Tera template engine wrapper. Holds the `tera::Tera` engine
      with custom filters registered. All behaviour is on the
      hand-written impl block.
    properties:
      tera:
        type: string
        x-rust-type: "Tera"
        x-rust-visibility: private
        description: "Underlying tera engine."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/engine/tera_engine.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - TemplateEngine
    description: |
      Codegen replaces the struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Correctly identifies the single struct, its private foreign-type field, lack of derives, and hand-written boundary.
- [schema] Definition is well-formed: `x-rust-struct.derive: []`, `tera` in `required:` with `x-rust-type: "Tera"` to use the bare foreign type, and `x-rust-visibility: private` to keep field non-`pub`.
- [changes] Two entries cleanly split codegen vs hand-written. `replaces` lists the single struct name; hand-written entry covers module-level items and the entire impl block.
