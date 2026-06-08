---
id: sdd-generate-generators-state-machine-gen-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# StateMachineGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/state_machine_gen.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `StateMachineGenerator` | projects/agentic-workflow/src/generate/generators/state_machine_gen.rs | struct | pub | 29 |  |
| `new` | projects/agentic-workflow/src/generate/generators/state_machine_gen.rs | function | pub | 37 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-state-machine-gen-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/state_machine_gen.rs -->
```rust

//! State machine code generator
//!
//! Generates Python Enum + transition function from a [`StateMachineDef`]
//! (state-machine section type):
//!
//! | Output file                 | Description                                    |
//! |-----------------------------|------------------------------------------------|
//! | `{machine_id}_states.py`    | Python Enum class + `transition()` function    |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::StateMachinePlus`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::SpecIR;

// ---------------------------------------------------------------------------
// StateMachineGenerator
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/state_machine_gen.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-state-machine-gen-preamble>"
    description: "Source template owns module docs, imports, and the generator section header."
```
