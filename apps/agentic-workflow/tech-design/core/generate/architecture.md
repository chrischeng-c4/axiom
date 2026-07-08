---
id: generate-architecture
type: spec
title: "Generate Diagram Generator Architecture"
version: 1
spec_type: utility
created_at: 2026-01-31T12:45:00+00:00
updated_at: 2026-01-31T12:45:00+00:00
design_elements:
  has_mermaid: true
  diagrams:
    - type: flowchart
      title: "Generate Generation Pipeline"
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Codegen TDs support CB lifecycle generation and regenerable artifact production."
---

<spec>

# Generate Diagram Generator Architecture

## Overview
<!-- type: overview lang: markdown -->

Generate provides diagram and specification generation from structured input.

## Generation Pipeline
<!-- type: diagram lang: mermaid -->

```mermaid
flowchart TB
    subgraph Input["Input Types"]
        Semantic[Semantic Input]
        Code[Code Analysis]
        Spec[Spec Definition]
    end

    subgraph Generate["Generate Engine"]
        Parser[Input Parser]
        Transformer[AST Transformer]
        Generator[Output Generator]
    end

    subgraph Output["Output Formats"]
        Mermaid[Mermaid Diagrams]
        OpenAPI[OpenAPI 3.1]
        AsyncAPI[AsyncAPI 2.6]
        OpenRPC[OpenRPC 1.3]
        Workflow[Serverless Workflow]
    end

    Semantic --> Parser
    Code --> Parser
    Spec --> Parser
    Parser --> Transformer
    Transformer --> Generator
    Generator --> Mermaid
    Generator --> OpenAPI
    Generator --> AsyncAPI
    Generator --> OpenRPC
    Generator --> Workflow
```

## Mermaid Diagram Types
<!-- type: diagram lang: mermaid -->

```mermaid
flowchart LR
    subgraph Diagrams["Supported Diagrams"]
        Flowchart[flowchart]
        Sequence[sequence]
        Class[class]
        State[state]
        ERD[erd]
        Mindmap[mindmap]
        Requirement[requirement]
        Journey[journey]
    end

    Input[Semantic Input] --> Flowchart
    Input --> Sequence
    Input --> Class
    Input --> State
    Input --> ERD
    Input --> Mindmap
    Input --> Requirement
    Input --> Journey
```

## Code-to-Diagram Flow
<!-- type: diagram lang: mermaid -->

```mermaid
sequenceDiagram
    participant User
    participant CLI as CLI Server
    participant Generate as Generate Engine
    participant Lens as Lens Analyzer

    User->>CLI: lens_code_to_mermaid(file, type)
    CLI->>Lens: analyze(file)
    Lens-->>CLI: AST + Symbols
    CLI->>Generate: generate(ast, type)
    Generate->>Generate: transform_to_semantic()
    Generate->>Generate: render_mermaid()
    Generate-->>CLI: Mermaid code
    CLI-->>User: diagram
```

</spec>
