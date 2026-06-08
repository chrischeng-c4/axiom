---
id: sdd-generate-generators-sequence-plus-gen-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# SequencePlusGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/sequence_plus_gen.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SequencePlusGenerator` | projects/agentic-workflow/src/generate/generators/sequence_plus_gen.rs | struct | pub | 30 |  |
| `new` | projects/agentic-workflow/src/generate/generators/sequence_plus_gen.rs | function | pub | 38 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-sequence-plus-gen-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/sequence_plus_gen.rs -->
```rust

//! Sequence+ code generator
//!
//! Generates Python async call chain from a [`SequenceDef`]
//! (interaction/sequence section type):
//!
//! | Output file                    | Description                                  |
//! |--------------------------------|----------------------------------------------|
//! | `{sequence_id}_handlers.py`    | Async handler functions with `@sdd:implement` markers |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::SequencePlus`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::diagrams::{ArrowType, MessageDef, SequenceDef};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::SpecIR;

// ---------------------------------------------------------------------------
// SequencePlusGenerator
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/sequence_plus_gen.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-sequence-plus-gen-preamble>"
    description: "Source template owns module docs, imports, and the generator section header."
```
