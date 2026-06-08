---
id: sdd-generate-generators-react-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ReactGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/react.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReactGenerator` | projects/agentic-workflow/src/generate/generators/react.rs | struct | pub | 32 |  |
| `new` | projects/agentic-workflow/src/generate/generators/react.rs | function | pub | 40 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-react-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/react.rs -->
```rust

//! React component scaffold generator
//!
//! Generates a React functional component scaffold from a [`WireframeSpec`]
//! (wireframe section type):
//!
//! | Output file                   | Description                                 |
//! |-------------------------------|---------------------------------------------|
//! | `{ComponentName}.tsx`         | React functional component (TypeScript)     |
//! | `{ComponentName}.types.ts`    | TypeScript props interface                  |
//! | `index.ts`                    | Barrel re-export                            |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::Wireframe`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::{PropDef, SpecIR, WireframeNode, WireframeSpec};
use serde::Serialize;

// ---------------------------------------------------------------------------
// ReactGenerator
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/react.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-react-preamble>"
    description: "Source template owns module docs, imports, and the generator section header."
```
