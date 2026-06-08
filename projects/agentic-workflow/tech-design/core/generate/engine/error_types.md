---
id: sdd-generate-engine-error-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TemplateError

## Overview
<!-- type: overview lang: markdown -->

Single thiserror enum. Companion source templates own the module preamble and
the `From<tera::Error>` adapter that previously lived outside the generated
enum declaration.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TemplateError:
    type: object
    description: Template engine errors.
    x-rust-enum:
      derive: [Debug, "thiserror::Error"]
      variants:
        - name: NotFound
          kind: tuple
          error: "Template not found: {0}"
          fields: [{ rust_type: String }]
        - name: ParseError
          kind: struct
          error: "Template parse error in '{template}': {message}"
          fields: [{ name: template, rust_type: String }, { name: message, rust_type: String }]
        - name: RenderError
          kind: struct
          error: "Template render error in '{template}': {message}"
          fields: [{ name: template, rust_type: String }, { name: message, rust_type: String }]
        - name: TypeMismatch
          kind: struct
          error: "Context type mismatch: expected {expected}, got {actual}"
          fields: [{ name: expected, rust_type: String }, { name: actual, rust_type: String }]
        - name: FilterError
          kind: struct
          error: "Filter error in '{filter}': {message}"
          fields: [{ name: filter, rust_type: String }, { name: message, rust_type: String }]
        - name: DirectoryNotFound
          kind: tuple
          error: "Template directory not found: {0}"
          fields: [{ rust_type: PathBuf }]
        - name: Io
          kind: tuple
          error: "IO error: {0}"
          fields: [{ rust_type: "std::io::Error", error_from: true }]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/engine/error.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - TemplateError
    description: Codegen replaces TemplateError thiserror enum.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
