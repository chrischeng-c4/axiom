---
id: sdd-generate-generators-flowchart-plus-gen-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# FlowchartPlusGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FlowchartPlusGenerator` | projects/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs | struct | pub | 32 |  |
| `new` | projects/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs | function | pub | 40 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-flowchart-plus-gen-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs -->
```rust

//! Flowchart+ code generator
//!
//! Generates Python function skeletons from a [`FlowchartDef`]
//! (flowchart/logic section type) with YAML metadata:
//!
//! | Output file                 | Description                                    |
//! |-----------------------------|------------------------------------------------|
//! | `{flowchart_id}_flow.py`    | Python function skeletons with `@sdd:implement` markers |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::FlowchartPlus`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::diagrams::{
    FlowchartDef, FlowchartEdgeDef as EdgeDef, FlowchartNodeDef as NodeDef, NodeShape, SemanticType,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::SpecIR;

// ---------------------------------------------------------------------------
// FlowchartPlusGenerator
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-flowchart-plus-gen-preamble>"
    description: "Source template owns FlowchartPlus generator module docs and imports."
```
