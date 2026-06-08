---
id: sdd-generate-engine-tera-engine-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TemplateEngine Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/engine/tera_engine.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TemplateEngine` | projects/agentic-workflow/src/generate/engine/tera_engine.rs | struct | pub | 17 |  |
| `add_template` | projects/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 67 | add_template(&mut self, name: &str, content: &str) -> Result<(), TemplateError> |
| `empty` | projects/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 54 | empty() -> Self |
| `has_template` | projects/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 101 | has_template(&self, name: &str) -> bool |
| `new` | projects/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 31 | new(template_dir: impl AsRef<Path>) -> Result<Self, TemplateError> |
| `render` | projects/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 77 | render(         &self,         template: &str,         context: &T,     ) -> Result<String, TemplateError> |
| `template_names` | projects/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 106 | template_names(&self) -> impl Iterator<Item = &str> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-engine-tera-engine-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/engine/tera_engine.rs -->
```rust

//! Tera template engine wrapper

use super::error::TemplateError;
use super::filters;
use serde::Serialize;
use std::path::Path;
use tera::{Context, Tera};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/engine/tera_engine.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-engine-tera-engine-preamble>"
    description: "Source template owns TemplateEngine module docs and imports."
```
