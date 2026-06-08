---
id: lens-class-diagram
type: spec
title: "Lens Code Analyzer Class Diagram"
version: 1
spec_type: algorithm
created_at: 2026-01-31T12:45:00+00:00
updated_at: 2026-01-31T12:45:00+00:00
design_elements:
  has_mermaid: true
  diagrams:
    - type: class
      title: "Lens Architecture"
    - type: flowchart
      title: "Analysis Pipeline"
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This codegen logic TD supports CB lifecycle generation and regenerable artifact production."
---

<spec>

# Lens Code Analyzer Architecture

## Overview
<!-- type: doc lang: markdown -->

Lens (Argus) provides unified multi-language code analysis with LSP + Linting.

## Class Diagram
<!-- type: doc lang: markdown -->

```mermaid
classDiagram
    class Analyzer {
        <<interface>>
        +analyze(file: Path) AnalysisResult
        +get_diagnostics() Vec~Diagnostic~
        +get_symbols() Vec~Symbol~
    }

    class PythonAnalyzer {
        -pyright: PyrightClient
        -ruff: RuffClient
        +analyze(file) AnalysisResult
        +get_type_info(pos) TypeInfo
    }

    class TypeScriptAnalyzer {
        -tsserver: TSServerClient
        -eslint: ESLintClient
        +analyze(file) AnalysisResult
        +get_type_info(pos) TypeInfo
    }

    class RustAnalyzer {
        -rust_analyzer: RAClient
        -clippy: ClippyClient
        +analyze(file) AnalysisResult
        +get_type_info(pos) TypeInfo
    }

    class AnalysisResult {
        +diagnostics: Vec~Diagnostic~
        +symbols: Vec~Symbol~
        +references: Vec~Reference~
    }

    class Diagnostic {
        +severity: Severity
        +message: String
        +range: Range
        +code: String
        +source: String
    }

    class Symbol {
        +name: String
        +kind: SymbolKind
        +range: Range
        +children: Vec~Symbol~
    }

    class RefactoringEngine {
        +rename(symbol, new_name) Edit[]
        +extract_variable(range) Edit[]
        +extract_function(range) Edit[]
        +inline(symbol) Edit[]
    }

    class SemanticSearch {
        +find_usages(symbol) Location[]
        +find_implementations(symbol) Location[]
        +find_references(symbol) Location[]
        +search_by_type(signature) Symbol[]
    }

    Analyzer <|-- PythonAnalyzer
    Analyzer <|-- TypeScriptAnalyzer
    Analyzer <|-- RustAnalyzer
    Analyzer --> AnalysisResult
    AnalysisResult --> Diagnostic
    AnalysisResult --> Symbol
    PythonAnalyzer --> RefactoringEngine
    PythonAnalyzer --> SemanticSearch
```

## Analysis Pipeline
<!-- type: doc lang: markdown -->

```mermaid
flowchart TB
    Input[Source File] --> Detect[Detect Language]
    Detect --> Parse[Parse AST]
    Parse --> TypeCheck[Type Check]
    TypeCheck --> Lint[Lint]
    TypeCheck --> Symbols[Extract Symbols]
    Lint --> Diagnostics[Collect Diagnostics]
    Symbols --> Index[Build Index]
    Diagnostics --> Output[Analysis Result]
    Index --> Output
```

## MCP Tool Flow
<!-- type: doc lang: markdown -->

```mermaid
flowchart LR
    subgraph Tools["MCP Tools"]
        Check[lens_check]
        Symbols[lens_symbols]
        Hover[lens_hover]
        Definition[lens_definition]
        References[lens_references]
    end

    subgraph Core["Lens Core"]
        Analyzer[Language Analyzer]
        Index[Symbol Index]
        Cache[Analysis Cache]
    end

    Check --> Analyzer
    Symbols --> Index
    Hover --> Analyzer
    Definition --> Index
    References --> Index
    Analyzer --> Cache
    Index --> Cache
```

</spec>
