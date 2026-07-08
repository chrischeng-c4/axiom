---
id: sdd-fillback-graph-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Fillback Graph Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/fillback/graph.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Dependency` | apps/agentic-workflow/src/fillback/graph.rs | struct | pub | 18 |  |
| `DependencyGraph` | apps/agentic-workflow/src/fillback/graph.rs | struct | pub | 30 |  |
| `DependencyType` | apps/agentic-workflow/src/fillback/graph.rs | enum | pub | 41 |  |
| `GraphStats` | apps/agentic-workflow/src/fillback/graph.rs | struct | pub | 50 |  |
| `ModuleNode` | apps/agentic-workflow/src/fillback/graph.rs | struct | pub | 68 |  |
| `external_dependencies` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 239 | external_dependencies(&self) -> Vec<&ModuleNode> |
| `from_analysis` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 106 | from_analysis(context: &AnalysisContext) -> Self |
| `from_graph` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 413 | from_graph(graph: &DependencyGraph) -> Self |
| `internal_modules` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 234 | internal_modules(&self) -> Vec<&ModuleNode> |
| `new` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 98 | new() -> Self |
| `to_markdown` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 341 | to_markdown(&self, project_name: &str) -> String |
| `to_mermaid` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 244 | to_mermaid(&self) -> String |
| `to_mermaid_compact` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 297 | to_mermaid_compact(&self) -> String |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-graph-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/fillback/graph.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-graph-runtime>"
    description: "Source template owns fillback graph runtime behavior and tests."
```
