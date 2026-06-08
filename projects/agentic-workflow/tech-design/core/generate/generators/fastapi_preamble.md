---
id: sdd-generate-generators-fastapi-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# FastAPIGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/fastapi.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FastAPIGenerator` | projects/agentic-workflow/src/generate/generators/fastapi.rs | struct | pub | 35 |  |
| `new` | projects/agentic-workflow/src/generate/generators/fastapi.rs | function | pub | 43 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-fastapi-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/fastapi.rs -->
```rust

//! FastAPI code generator
//!
//! Generates a standard FastAPI project layout from a JSON Schema / OpenAPI input:
//!
//! | Output file      | Source section | Description |
//! |------------------|----------------|-------------|
//! | `models.py`      | schema         | Pydantic `BaseModel` definitions |
//! | `schemas.py`     | schema         | Create/Update/Response wrappers (cross-section) |
//! | `routes.py`      | rest-api × schema | `APIRouter` with typed handlers |
//! | `app.py`         | project config | FastAPI app entry-point |
//! | `requirements.txt` | project config | Python dependencies |
//!
//! Cross-section composition (Phase 2): route handlers reference both the base
//! models (`models.py`) and the request/response schemas (`schemas.py`), tying
//! the rest-api and schema sections together.

use super::common::{
    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::schema::{JsonSchema, SchemaType};
use serde::Serialize;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// FastAPI code generator
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/fastapi.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-fastapi-preamble>"
    description: "Source template owns module docs, imports, and the generator section header."
```
