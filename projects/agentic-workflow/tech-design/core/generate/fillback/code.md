---
id: sdd-fillback-code
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Code Strategy Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/fillback/code.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CodeStrategy` | projects/agentic-workflow/src/fillback/code.rs | struct | pub | 27 |  |
| `CodeStrategyConfig` | projects/agentic-workflow/src/fillback/code.rs | struct | pub | 35 |  |
| `analyze_codebase` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 140 | analyze_codebase(&self, source: &Path) -> Result<(AnalysisContext, Vec<ParseError>)> |
| `check_existing_specs` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 371 | check_existing_specs(&self, output_dir: &Path) -> Result<Vec<String>> |
| `confirm_overwrite` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 394 | confirm_overwrite(&self, existing_files: &[String]) -> Result<bool> |
| `display_dependency_graph` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 247 | display_dependency_graph(&self, graph: &DependencyGraph) |
| `display_summary` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 193 | display_summary(&self, context: &AnalysisContext, graph: &DependencyGraph) |
| `generate_specs` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 425 | generate_specs(         &self,         context: &AnalysisContext,         graph: &DependencyGraph,         output_dir: &Path,         clarifications: &HashMap<String, String>,     ) -> Result<Vec<String>> |
| `new` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 66 | new() -> Self |
| `print_parse_errors` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 800 | print_parse_errors(&self, errors: &[ParseError]) |
| `run_clarification` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 261 | run_clarification(&self, context: &AnalysisContext) -> Result<HashMap<String, String>> |
| `with_config` | projects/agentic-workflow/src/fillback/code.rs | function | pub | 72 | with_config(config: CodeStrategyConfig) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  CodeStrategyConfig:
    type: object
    required: [path, module, force, output_dir, quick]
    description: Configuration for the code analysis strategy.
    properties:
      path:
        type: string
        x-rust-type: "Option<String>"
        description: "Path to analyze (defaults to current directory)."
      module:
        type: string
        x-rust-type: "Option<String>"
        description: "Specific module to analyze (optional filter)."
      force:
        type: boolean
        description: "Force overwrite without confirmation."
      output_dir:
        type: string
        x-rust-type: "Option<String>"
        description: "Output directory for specs."
      quick:
        type: boolean
        description: "Quick mode: skip LLM enrichment and use AST-only analysis."
    x-rust-struct:
      derive: [Debug, Clone]

  CodeStrategy:
    type: object
    required: [config]
    description: Code import strategy with AST-based analysis.
    properties:
      config:
        type: object
        x-rust-type: "CodeStrategyConfig"
        x-rust-visibility: private
        description: "Strategy configuration."
    x-rust-struct:
      derive: []
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-code-strategy-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/code.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - CodeStrategyConfig
      - CodeStrategy
    description: |
      Codegen replaces both struct declarations.
  - path: projects/agentic-workflow/src/fillback/code.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-code-strategy-runtime>"
    description: |
      Source template owns code strategy defaults, AST scanning, clarification
      prompts, spec emission, strategy integration, helpers, and tests.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Config + Strategy struct pair.
- [schema] Standard pattern.
- [changes] Standard split; private fixture Config in test module preserved.
