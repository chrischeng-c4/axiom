"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/fillback/graph_imports_source.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-fillback-graph-imports-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/fillback/graph_imports_source.md"
__legacy_projection_digest__ = "sha256:2ea76802804c0718acd9b6754db224b3d9175f5efb9de6bca17a86fc49e2ccfa"


def render_markdown() -> Annotated[str, "sha256:2ea76802804c0718acd9b6754db224b3d9175f5efb9de6bca17a86fc49e2ccfa"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-fillback-graph-imports-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Fillback Graph Imports Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/fillback/graph.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `Dependency` | apps/agentic-workflow/src/fillback/graph.rs | struct | pub | 18 |  |\n| `DependencyGraph` | apps/agentic-workflow/src/fillback/graph.rs | struct | pub | 30 |  |\n| `DependencyType` | apps/agentic-workflow/src/fillback/graph.rs | enum | pub | 41 |  |\n| `GraphStats` | apps/agentic-workflow/src/fillback/graph.rs | struct | pub | 50 |  |\n| `ModuleNode` | apps/agentic-workflow/src/fillback/graph.rs | struct | pub | 68 |  |\n| `external_dependencies` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 239 | external_dependencies(&self) -> Vec<&ModuleNode> |\n| `from_analysis` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 106 | from_analysis(context: &AnalysisContext) -> Self |\n| `from_graph` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 413 | from_graph(graph: &DependencyGraph) -> Self |\n| `internal_modules` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 234 | internal_modules(&self) -> Vec<&ModuleNode> |\n| `new` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 98 | new() -> Self |\n| `to_markdown` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 341 | to_markdown(&self, project_name: &str) -> String |\n| `to_mermaid` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 244 | to_mermaid(&self) -> String |\n| `to_mermaid_compact` | apps/agentic-workflow/src/fillback/graph.rs | function | pub | 297 | to_mermaid_compact(&self) -> String |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap fillback-graph-imports -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/fillback/graph.rs -->\n```rust\nuse crate::fillback::ast::AnalysisContext;\nuse std::collections::{HashMap, HashSet};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/fillback/graph.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:fillback-graph-imports>\"\n    description: \"Source template owns fillback graph analysis imports.\"\n```\n"
