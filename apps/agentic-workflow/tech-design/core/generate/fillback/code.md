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

Public API manifest for `apps/agentic-workflow/src/fillback/code.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CodeStrategy` | apps/agentic-workflow/src/fillback/code.rs | struct | pub | 27 |  |
| `CodeStrategyConfig` | apps/agentic-workflow/src/fillback/code.rs | struct | pub | 35 |  |
| `analyze_codebase` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 140 | analyze_codebase(&self, source: &Path) -> Result<(AnalysisContext, Vec<ParseError>)> |
| `check_existing_specs` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 371 | check_existing_specs(&self, output_dir: &Path) -> Result<Vec<String>> |
| `confirm_overwrite` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 394 | confirm_overwrite(&self, existing_files: &[String]) -> Result<bool> |
| `display_dependency_graph` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 247 | display_dependency_graph(&self, graph: &DependencyGraph) |
| `display_summary` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 193 | display_summary(&self, context: &AnalysisContext, graph: &DependencyGraph) |
| `generate_specs` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 425 | generate_specs(         &self,         context: &AnalysisContext,         graph: &DependencyGraph,         output_dir: &Path,         clarifications: &HashMap<String, String>,     ) -> Result<Vec<String>> |
| `new` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 66 | new() -> Self |
| `print_parse_errors` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 800 | print_parse_errors(&self, errors: &[ParseError]) |
| `run_clarification` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 261 | run_clarification(&self, context: &AnalysisContext) -> Result<HashMap<String, String>> |
| `with_config` | apps/agentic-workflow/src/fillback/code.rs | function | pub | 72 | with_config(config: CodeStrategyConfig) -> Self |

### Explicit-file adoption contract

An explicitly selected supported source file is a bounded user selection: the
strategy analyzes only that file, never its siblings, and does not apply the
directory discovery scanner's 100 KB ceiling. Rust, Python, JavaScript
(`.js`/`.jsx`/`.mjs`/`.cjs`), TypeScript (`.ts`/`.tsx`), and Go are accepted.
Rust uses the structured `rust-source-unit` Item/Trivia IR; the other languages use a typed
`text-source-unit` with an explicit `source_lang`. Large or sentinel-sensitive
payloads are split into ordered, bounded, base64-encoded partitions using
complete top-level AST boundaries where possible and a deterministic byte/
newline fallback otherwise. Per-partition and whole-source SHA-256 digests make
the representation lossless and corruption-detectable.

A unique existing whole-file CODEGEN owner may be refreshed in place while
preserving metadata and capability references. Before mutation, the importer
checks target, section type, action, absence of `replaces`, canonical typed
Source annotation/fence, decoded replay equality, and the original owner
snapshot. New owners use no-clobber creation. Partial, external, syntactically
ambiguous, concurrently changed, or multiply-owned targets return a HITL
outcome with no mutation. Directory discovery retains its existing file-count
and size ceilings.
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
        description: "Directory or explicit source file to analyze (defaults to current directory)."
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
  - path: apps/agentic-workflow/src/fillback/code.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - CodeStrategyConfig
      - CodeStrategy
    description: |
      Codegen replaces both struct declarations.
  - path: apps/agentic-workflow/src/fillback/code.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-code-strategy-runtime>"
    description: |
      Source template owns code strategy defaults, AST scanning, clarification
      prompts, multi-language explicit-file lossless source-unit adoption,
      AST/fallback partition planning, strict safe-owner refresh/no-clobber
      persistence, HITL routing, spec emission, strategy integration, helpers,
      and corruption/race/marker-fixture tests.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Config + Strategy struct pair.
- [schema] Standard pattern.
- [changes] Standard split; private fixture Config in test module preserved.
