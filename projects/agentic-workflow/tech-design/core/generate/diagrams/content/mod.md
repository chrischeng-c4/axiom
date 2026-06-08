---
id: projects-sdd-src-generate-diagrams-content-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/diagrams/content/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/content/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `interaction` | projects/agentic-workflow/src/generate/diagrams/content/mod.rs | module | pub | 8 |  |
| `logic` | projects/agentic-workflow/src/generate/diagrams/content/mod.rs | module | pub | 9 |  |
| `requirement` | projects/agentic-workflow/src/generate/diagrams/content/mod.rs | module | pub | 10 |  |
| `state_machine` | projects/agentic-workflow/src/generate/diagrams/content/mod.rs | module | pub | 11 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/content/mod.rs -->
```rust
//! Per-diagram Content types for Mermaid Plus codegen.
//!
//! Each diagram type has its own explicit Content struct (design decision D3).
//! No universal `Graph<N,E>` — each type is statically typed and XState-free (D8).

pub mod interaction;
pub mod logic;
pub mod requirement;
pub mod state_machine;

pub use interaction::InteractionContent;
pub use logic::LogicContent;
pub use requirement::RequirementContent;
pub use state_machine::StateMachineContent;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/content/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete content module declaration and exports.
```
