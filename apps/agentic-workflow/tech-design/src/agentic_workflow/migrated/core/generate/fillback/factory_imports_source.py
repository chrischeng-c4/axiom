"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/fillback/factory_imports_source.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-fillback-factory-imports-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/fillback/factory_imports_source.md"
__legacy_projection_digest__ = "sha256:d49b9418bd2403fd777d73f716935120914d978d4baab7137c41461c6c344d07"


def render_markdown() -> Annotated[str, "sha256:d49b9418bd2403fd777d73f716935120914d978d4baab7137c41461c6c344d07"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-fillback-factory-imports-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Fillback Factory Imports Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/fillback/factory.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `StrategyFactory` | apps/agentic-workflow/src/fillback/factory.rs | struct | pub | 17 |  |\n| `create` | apps/agentic-workflow/src/fillback/factory.rs | function | pub | 35 | create(strategy_type: &str, source: &Path) -> Result<Box<dyn ImportStrategy>> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap fillback-factory-imports -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/fillback/factory.rs -->\n```rust\nuse crate::fillback::code::CodeStrategy;\nuse crate::fillback::openspec::OpenSpecStrategy;\nuse crate::fillback::speckit::SpeckitStrategy;\nuse crate::fillback::strategy::ImportStrategy;\nuse crate::Result;\nuse colored::Colorize;\nuse std::path::Path;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/fillback/factory.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:fillback-factory-imports>\"\n    description: \"Source template owns fillback factory imports.\"\n```\n"
