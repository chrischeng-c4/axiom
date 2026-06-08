---
id: projects-sdd-src-fillback-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: brownfield-takeover-surface
    claim: brownfield-takeover-surface
    coverage: full
    rationale: "Fillback interfaces support brownfield takeover by deriving TD/spec coverage from existing project artifacts."
---

# Standardized projects/agentic-workflow/src/fillback/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/fillback/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ast` | projects/agentic-workflow/src/fillback/mod.rs | module | pub | 3 |  |
| `code` | projects/agentic-workflow/src/fillback/mod.rs | module | pub | 4 |  |
| `factory` | projects/agentic-workflow/src/fillback/mod.rs | module | pub | 5 |  |
| `graph` | projects/agentic-workflow/src/fillback/mod.rs | module | pub | 6 |  |
| `openspec` | projects/agentic-workflow/src/fillback/mod.rs | module | pub | 7 |  |
| `speckit` | projects/agentic-workflow/src/fillback/mod.rs | module | pub | 8 |  |
| `strategy` | projects/agentic-workflow/src/fillback/mod.rs | module | pub | 9 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/fillback/mod.rs -->
```rust
pub mod ast;
pub mod code;
pub mod factory;
pub mod graph;
pub mod openspec;
pub mod speckit;
pub mod strategy;

pub use ast::{
    AnalysisContext, AstAnalyzer, Import, ModuleInfo, ParseError, SupportedLanguage, Symbol,
    SymbolKind,
};
pub use code::{CodeStrategy, CodeStrategyConfig};
pub use factory::StrategyFactory;
pub use graph::{Dependency, DependencyGraph, DependencyType, GraphStats, ModuleNode};
pub use strategy::ImportStrategy;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete fillback module facade.
```
