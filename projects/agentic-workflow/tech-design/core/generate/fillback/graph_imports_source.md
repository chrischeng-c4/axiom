---
id: sdd-fillback-graph-imports-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Fillback Graph Imports Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/fillback/graph.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Dependency` | projects/agentic-workflow/src/fillback/graph.rs | struct | pub | 18 |  |
| `DependencyGraph` | projects/agentic-workflow/src/fillback/graph.rs | struct | pub | 30 |  |
| `DependencyType` | projects/agentic-workflow/src/fillback/graph.rs | enum | pub | 41 |  |
| `GraphStats` | projects/agentic-workflow/src/fillback/graph.rs | struct | pub | 50 |  |
| `ModuleNode` | projects/agentic-workflow/src/fillback/graph.rs | struct | pub | 68 |  |
| `external_dependencies` | projects/agentic-workflow/src/fillback/graph.rs | function | pub | 239 | external_dependencies(&self) -> Vec<&ModuleNode> |
| `from_analysis` | projects/agentic-workflow/src/fillback/graph.rs | function | pub | 106 | from_analysis(context: &AnalysisContext) -> Self |
| `from_graph` | projects/agentic-workflow/src/fillback/graph.rs | function | pub | 413 | from_graph(graph: &DependencyGraph) -> Self |
| `internal_modules` | projects/agentic-workflow/src/fillback/graph.rs | function | pub | 234 | internal_modules(&self) -> Vec<&ModuleNode> |
| `new` | projects/agentic-workflow/src/fillback/graph.rs | function | pub | 98 | new() -> Self |
| `to_markdown` | projects/agentic-workflow/src/fillback/graph.rs | function | pub | 341 | to_markdown(&self, project_name: &str) -> String |
| `to_mermaid` | projects/agentic-workflow/src/fillback/graph.rs | function | pub | 244 | to_mermaid(&self) -> String |
| `to_mermaid_compact` | projects/agentic-workflow/src/fillback/graph.rs | function | pub | 297 | to_mermaid_compact(&self) -> String |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-graph-imports -->

<!-- source-snapshot: path=projects/agentic-workflow/src/fillback/graph.rs -->
```rust
use crate::fillback::ast::AnalysisContext;
use std::collections::{HashMap, HashSet};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/graph.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-graph-imports>"
    description: "Source template owns fillback graph analysis imports."
```
