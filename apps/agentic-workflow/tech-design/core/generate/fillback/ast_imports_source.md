---
id: sdd-fillback-ast-imports-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Fillback AST Imports Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/fillback/ast.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AnalysisContext` | apps/agentic-workflow/src/fillback/ast.rs | struct | pub | 21 |  |
| `AstAnalyzer` | apps/agentic-workflow/src/fillback/ast.rs | struct | pub | 32 |  |
| `Import` | apps/agentic-workflow/src/fillback/ast.rs | struct | pub | 40 |  |
| `ModuleInfo` | apps/agentic-workflow/src/fillback/ast.rs | struct | pub | 52 |  |
| `ParseError` | apps/agentic-workflow/src/fillback/ast.rs | struct | pub | 68 |  |
| `StructField` | apps/agentic-workflow/src/fillback/ast.rs | struct | pub | 78 |  |
| `SupportedLanguage` | apps/agentic-workflow/src/fillback/ast.rs | enum | pub | 91 |  |
| `Symbol` | apps/agentic-workflow/src/fillback/ast.rs | struct | pub | 102 |  |
| `SymbolKind` | apps/agentic-workflow/src/fillback/ast.rs | enum | pub | 130 |  |
| `display_name` | apps/agentic-workflow/src/fillback/ast.rs | function | pub | 179 | display_name(&self) -> &'static str |
| `external_dependencies` | apps/agentic-workflow/src/fillback/ast.rs | function | pub | 222 | external_dependencies(&self) -> Vec<String> |
| `from_extension` | apps/agentic-workflow/src/fillback/ast.rs | function | pub | 156 | from_extension(ext: &str) -> Option<Self> |
| `new` | apps/agentic-workflow/src/fillback/ast.rs | function | pub | 208 | new() -> Self |
| `new` | apps/agentic-workflow/src/fillback/ast.rs | function | pub | 246 | new() -> Result<Self> |
| `parse_file` | apps/agentic-workflow/src/fillback/ast.rs | function | pub | 265 | parse_file(         &mut self,         path: &Path,         content: &str,     ) -> std::result::Result<ModuleInfo, ParseError> |
| `total_symbols` | apps/agentic-workflow/src/fillback/ast.rs | function | pub | 217 | total_symbols(&self) -> usize |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-ast-imports -->

<!-- source-snapshot: path=apps/agentic-workflow/src/fillback/ast.rs -->
```rust
use crate::generate::diagrams::content::logic::{FlowEdge, FlowNode, FlowNodeKind, LogicContent};
use crate::Result;
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Parser, Tree};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/fillback/ast.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-ast-imports>"
    description: "Source template owns fillback AST analysis imports."
```
