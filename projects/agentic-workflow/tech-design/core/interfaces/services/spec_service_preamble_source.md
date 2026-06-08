---
id: sdd-interfaces-services-spec-service-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Spec Service Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/spec_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ApiSpecData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 38 |  |
| `CreateSpecInput` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 48 |  |
| `DiagramData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 94 |  |
| `RequirementData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 110 |  |
| `ScenarioData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 124 |  |
| `SpecChangeData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 138 |  |
| `create_spec` | projects/agentic-workflow/src/services/spec_service.rs | function | pub | 435 | create_spec(input: CreateSpecInput, project_root: &Path) -> Result<String> |
| `resolve_section_rules` | projects/agentic-workflow/src/services/spec_service.rs | function | pub | 921 | resolve_section_rules(     requirements_text: &str,     design_system: Option<&DesignSystem>, ) -> Vec<SectionEntry> |
## Source
<!-- type: source lang: rust -->

```rust
//! Spec service - Business logic for spec creation
//!
//! ## Structured Diagrams
#![allow(deprecated)]
//!
//! The service supports structured diagram definitions that are validated and rendered
//! using the embedded generate library. This ensures:
//! 1. Diagrams are syntactically correct
//! 2. Semantic metadata is preserved for code generation
//! 3. Consistent diagram output across the system

use crate::generate::diagrams::{
    class::{generate_class_diagram, ClassInput},
    erd::{generate_erd, ErdInput},
    flowchart::{generate_flowchart, FlowchartInput},
    journey::{generate_journey, JourneyInput},
    mindmap::{generate_mindmap, MindmapInput},
    requirement::{generate_requirement_diagram, RequirementInput},
    sequence::{generate_sequence, SequenceInput},
    state::{generate_state_diagram, StateInput},
};
use crate::models::spec_rules::{
    apply_section_optionality, ApiSpecType, SectionEntry, SectionType, SpecFormatRules, SpecType,
};
use crate::models::tech_stack::DesignSystem;
use crate::Result;
use chrono::Utc;
use serde_json::Value;
use std::path::Path;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/spec_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "<handwrite-gap:spec-service-preamble>"
    description: "Source template owns spec service documentation and imports."
```
