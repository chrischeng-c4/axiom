"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/architecture.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-architecture"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/architecture.md"
__legacy_td_digest__ = "sha256:62b90de1596b5cd83bf0b29f32f05e3fc9d959d4ea549025d54cfcdfcff5d8ce"


def render_markdown() -> Annotated[str, "sha256:62b90de1596b5cd83bf0b29f32f05e3fc9d959d4ea549025d54cfcdfcff5d8ce"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: generate-architecture\ntype: spec\ntitle: \"Generate Diagram Generator Architecture\"\nversion: 1\nspec_type: utility\ncreated_at: 2026-01-31T12:45:00+00:00\nupdated_at: 2026-01-31T12:45:00+00:00\ndesign_elements:\n  has_mermaid: true\n  diagrams:\n    - type: flowchart\n      title: \"Generate Generation Pipeline\"\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Codegen TDs support CB lifecycle generation and regenerable artifact production.\"\n---\n\n<spec>\n\n# Generate Diagram Generator Architecture\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nGenerate provides diagram and specification generation from structured input.\n\n## Generation Pipeline\n<!-- type: diagram lang: mermaid -->\n\n```mermaid\nflowchart TB\n    subgraph Input[\"Input Types\"]\n        Semantic[Semantic Input]\n        Code[Code Analysis]\n        Spec[Spec Definition]\n    end\n\n    subgraph Generate[\"Generate Engine\"]\n        Parser[Input Parser]\n        Transformer[AST Transformer]\n        Generator[Output Generator]\n    end\n\n    subgraph Output[\"Output Formats\"]\n        Mermaid[Mermaid Diagrams]\n        OpenAPI[OpenAPI 3.1]\n        AsyncAPI[AsyncAPI 2.6]\n        OpenRPC[OpenRPC 1.3]\n        Workflow[Serverless Workflow]\n    end\n\n    Semantic --> Parser\n    Code --> Parser\n    Spec --> Parser\n    Parser --> Transformer\n    Transformer --> Generator\n    Generator --> Mermaid\n    Generator --> OpenAPI\n    Generator --> AsyncAPI\n    Generator --> OpenRPC\n    Generator --> Workflow\n```\n\n## Mermaid Diagram Types\n<!-- type: diagram lang: mermaid -->\n\n```mermaid\nflowchart LR\n    subgraph Diagrams[\"Supported Diagrams\"]\n        Flowchart[flowchart]\n        Sequence[sequence]\n        Class[class]\n        State[state]\n        ERD[erd]\n        Mindmap[mindmap]\n        Requirement[requirement]\n        Journey[journey]\n    end\n\n    Input[Semantic Input] --> Flowchart\n    Input --> Sequence\n    Input --> Class\n    Input --> State\n    Input --> ERD\n    Input --> Mindmap\n    Input --> Requirement\n    Input --> Journey\n```\n\n## Code-to-Diagram Flow\n<!-- type: diagram lang: mermaid -->\n\n```mermaid\nsequenceDiagram\n    participant User\n    participant CLI as CLI Server\n    participant Generate as Generate Engine\n    participant Lens as Lens Analyzer\n\n    User->>CLI: lens_code_to_mermaid(file, type)\n    CLI->>Lens: analyze(file)\n    Lens-->>CLI: AST + Symbols\n    CLI->>Generate: generate(ast, type)\n    Generate->>Generate: transform_to_semantic()\n    Generate->>Generate: render_mermaid()\n    Generate-->>CLI: Mermaid code\n    CLI-->>User: diagram\n```\n\n</spec>\n"
