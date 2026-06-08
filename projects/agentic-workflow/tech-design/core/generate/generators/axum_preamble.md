---
id: sdd-generate-generators-axum-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AxumGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/axum.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AxumGenerator` | projects/agentic-workflow/src/generate/generators/axum.rs | struct | pub | 16 |  |
| `new` | projects/agentic-workflow/src/generate/generators/axum.rs | function | pub | 24 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-axum-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/axum.rs -->
```rust

//! Axum code generator

use super::common::{
    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::schema::{JsonSchema, SchemaType};
use serde::Serialize;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/axum.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-axum-preamble>"
    description: "Source template owns module docs and imports."
```
