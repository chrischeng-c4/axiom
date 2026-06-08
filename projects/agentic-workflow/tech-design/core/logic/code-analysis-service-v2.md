---
id: code-analysis-service-v2
type: spec
title: "Agnostic Code Analysis and LLM Enrichment Service"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-14T03:59:43.370787+00:00
updated_at: 2026-02-14T03:59:43.370787+00:00
requirements:
  total: 8
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
    - R8
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Code Analysis Pipeline Flowchart"
changes:
  - file: crates/cclab-sdd/src/mcp/tools/analyze.rs
    action: MODIFY
    context_ref: "codebase_spec#0"
    description: "Enhance sdd_analyze_code_for_spec with LLM enrichment and --quick flag support."
history:
  - timestamp: 2026-02-14T03:59:43.370787+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "This analysis/standardization logic TD supports brownfield semantic coverage and takeover readiness gates."
---

# Agnostic Code Analysis and LLM Enrichment Service

## Overview
<!-- type: doc lang: markdown -->

This specification defines the enhanced Code Analysis Service (v2) used by sdd_analyze_code_for_spec. The service leverages tree-sitter for high-fidelity AST analysis across Python, TypeScript, and Rust to extract structural metadata. It then optionally enriches this metadata using an LLM to generate high-quality requirements, scenarios, and structured diagrams. A fast-path is provided via the --quick flag for AST-only analysis.

## Requirements
<!-- type: doc lang: markdown -->

### R1 - Multi-language Support

```yaml
id: R1
priority: high
status: draft
```

Support parsing and symbol extraction for Python, TypeScript/JavaScript, and Rust source files.

### R2 - AST Metadata Extraction

```yaml
id: R2
priority: high
status: draft
```

Extract functions, classes, methods, fields, decorators, and docstrings using tree-sitter.

### R3 - Spec Type Recommendation

```yaml
id: R3
priority: medium
status: draft
```

Analyze extracted symbols to recommend spec_type (http-api, data-model, event-driven, etc.) based on patterns like decorators or base classes.

### R4 - LLM Enrichment Requirements

```yaml
id: R4
priority: medium
status: draft
```

Use an LLM to transform AST metadata into human-readable requirements with clear descriptions.

### R5 - Scenario Generation

```yaml
id: R5
priority: medium
status: draft
```

Generate Given/When/Then scenarios for identified functions and classes using LLM.

### R6 - Fast-path (AST-only) Execution

```yaml
id: R6
priority: high
status: draft
```

Provide a --quick flag to skip LLM enrichment and return results based solely on AST analysis.

### R7 - Agnostic Analysis Resilience

```yaml
id: R7
priority: low
status: draft
```

Handle mixed file sets and unknown file types gracefully, skipping what cannot be parsed while providing a deterministic summary of skipped items.

### R8 - Structured Diagram Mapping

```yaml
id: R8
priority: medium
status: draft
```

Suggest Generate-compatible structured diagram inputs (flowchart, sequence, class) based on code structure and symbol relationships.

## Acceptance Criteria
<!-- type: doc lang: markdown -->

### Scenario: Multi-language Symbol Extraction

- **GIVEN** A codebase containing .py, .ts, and .rs files.
- **WHEN** Analyzing the mixed file set.
- **THEN** The service extracts symbols (functions, classes) from all three languages correctly using language-specific tree-sitter grammars.

### Scenario: Fast-path AST Analysis (R6)

- **WHEN** The --quick flag is set to true.
- **THEN** Service returns structural metadata and basic requirements derived only from AST and patterns, skipping LLM enrichment.

### Scenario: Full LLM Enrichment (R4, R5)

- **WHEN** The --quick flag is set to false (default).
- **THEN** Service performs AST analysis followed by LLM calls to generate detailed requirements and acceptance scenarios.

### Scenario: Auto-detect Spec Type (R3)

- **WHEN** Analyzing a Python file with FastAPI decorators and analysis_type='auto'.
- **THEN** Service successfully identifies the file as 'http-api' based on FastAPI decorators and base classes.

### Scenario: Resilience to Unknown File Types (R7)

- **GIVEN** An input set containing source code and a .txt file.
- **WHEN** The service encounters an unsupported file extension.
- **THEN** The service processes the source code, skips the .txt file without error, and includes the .txt file in the 'skipped' section of the output summary.

### Scenario: Generate Diagram Suggestion (R8)

- **GIVEN** A class hierarchy with 'extends' relationships in TypeScript.
- **WHEN** Analyzing the file structure.
- **THEN** The service suggests a 'class' diagram input for Generate with correctly mapped 'inheritance' relationships.

### Scenario: Enriched Requirement Generation (R4)

- **GIVEN** A public function with detailed docstrings.
- **WHEN** Full LLM enrichment is enabled.
- **THEN** The resulting requirement description incorporates the semantic meaning from the docstrings rather than just the function signature.

### Scenario: Cross-module Symbol Detection (R2)

- **GIVEN** A set of interdependent Rust modules.
- **WHEN** Analyzing the directory structure.
- **THEN** Symbols are extracted with their full module paths for unambiguous identification.

## Diagrams
<!-- type: doc lang: markdown -->

### Code Analysis Pipeline Flowchart

```mermaid
flowchart TB
    input([Input File Paths & Type])
    treesitter[Tree-sitter AST Parsing & Symbol Extraction]
    patterns[Pattern Detection & Spec Type Recommendation]
    quick_check{Quick Mode?} 
    llm_enrichment[LLM Enrichment (v2)]
    output([Suggested Spec Structure (JSON)])
    input --> treesitter
    treesitter --> patterns
    patterns --> quick_check
    quick_check -->|Yes| output
    quick_check -->|No| llm_enrichment
    llm_enrichment --> output
```
