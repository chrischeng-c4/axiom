---
id: sdd-generate-generators-express-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ExpressGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/express.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ExpressGenerator` | projects/agentic-workflow/src/generate/generators/express.rs | struct | pub | 16 |  |
| `new` | projects/agentic-workflow/src/generate/generators/express.rs | function | pub | 24 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-express-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/express.rs -->
```rust

//! Express.js code generator

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
  - path: projects/agentic-workflow/src/generate/generators/express.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-express-preamble>"
    description: "Source template owns module docs and imports."
```
