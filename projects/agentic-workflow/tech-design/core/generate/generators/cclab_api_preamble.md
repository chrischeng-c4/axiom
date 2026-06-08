---
id: sdd-generate-generators-cclab-api-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# CclabApiGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/cclab_api.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CclabApiGenerator` | projects/agentic-workflow/src/generate/generators/cclab_api.rs | struct | pub | 35 |  |
| `new` | projects/agentic-workflow/src/generate/generators/cclab_api.rs | function | pub | 43 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-cclab-api-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/cclab_api.rs -->
```rust

//! cclab.api code generator
//!
//! Generates a complete cclab ecosystem feature module from JSON Schema / OpenAPI input:
//!
//! | Output file       | Description |
//! |-------------------|-------------|
//! | `__init__.py`     | Module docstring |
//! | `models.py`       | `{Resource}DB(Base)` ORM model using `cclab.pg` types |
//! | `schemas.py`      | `{Resource}Create/Update/Response/ListResponse` using `cclab.schema` |
//! | `repository.py`   | Async `{Resource}Repository` with CRUD skeleton |
//! | `routes.py`       | `cclab.api.Router` with typed handlers |
//!
//! Output is structured under `features/{domain}/` following the Conductor BE convention.

use super::common::{
    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::schema::{JsonSchema, SchemaType, StringFormat};
use serde::Serialize;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// CclabApiGenerator
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/cclab_api.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-cclab-api-preamble>"
    description: "Source template owns module docs, imports, and the generator section header."
```
