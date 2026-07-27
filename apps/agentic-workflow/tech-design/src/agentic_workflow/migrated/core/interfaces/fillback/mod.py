"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/fillback/mod.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-fillback-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/fillback/mod.md"
__legacy_projection_digest__ = "sha256:e7afc103797a019459b52efb571b22d94b91e9d14d6f9779a39342055c7ddbd9"


def render_markdown() -> Annotated[str, "sha256:e7afc103797a019459b52efb571b22d94b91e9d14d6f9779a39342055c7ddbd9"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-fillback-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: brownfield-takeover-surface\n    claim: brownfield-takeover-surface\n    coverage: full\n    rationale: \"Fillback interfaces support brownfield takeover by deriving TD/spec coverage from existing project artifacts.\"\n---\n\n# Standardized apps/agentic-workflow/src/fillback/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/fillback/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ast` | apps/agentic-workflow/src/fillback/mod.rs | module | pub | 3 |  |\n| `code` | apps/agentic-workflow/src/fillback/mod.rs | module | pub | 4 |  |\n| `factory` | apps/agentic-workflow/src/fillback/mod.rs | module | pub | 5 |  |\n| `graph` | apps/agentic-workflow/src/fillback/mod.rs | module | pub | 6 |  |\n| `openspec` | apps/agentic-workflow/src/fillback/mod.rs | module | pub | 7 |  |\n| `speckit` | apps/agentic-workflow/src/fillback/mod.rs | module | pub | 8 |  |\n| `strategy` | apps/agentic-workflow/src/fillback/mod.rs | module | pub | 9 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap standardize:claim-code -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/fillback/mod.rs -->\n```rust\npub mod ast;\npub mod code;\npub mod factory;\npub mod graph;\npub mod openspec;\npub mod speckit;\npub mod strategy;\n\npub use ast::{\n    AnalysisContext, AstAnalyzer, Import, ModuleInfo, ParseError, SupportedLanguage, Symbol,\n    SymbolKind,\n};\npub use code::{CodeStrategy, CodeStrategyConfig};\npub use factory::StrategyFactory;\npub use graph::{Dependency, DependencyGraph, DependencyType, GraphStats, ModuleNode};\npub use strategy::ImportStrategy;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/fillback/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:standardize:claim-code>\"\n    description: |\n      Source template owns the complete fillback module facade.\n```\n"
