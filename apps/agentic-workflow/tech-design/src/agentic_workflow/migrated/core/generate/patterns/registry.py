"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/patterns/registry.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-patterns-registry"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/patterns/registry.md"
__legacy_projection_digest__ = "sha256:280648a1923124d80edebe2ae9b431c017f0e9137a1cf9e9e217aecc6a189e3c"


def render_markdown() -> Annotated[str, "sha256:280648a1923124d80edebe2ae9b431c017f0e9137a1cf9e9e217aecc6a189e3c"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-patterns-registry-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/patterns/registry.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/patterns/registry.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `PATTERN_REGISTRY` | apps/agentic-workflow/src/generate/patterns/registry.rs | constant | pub | 20 |  |\n| `pattern_registry` | apps/agentic-workflow/src/generate/patterns/registry.rs | function | pub | 14 | pattern_registry() -> &'static [UxPattern] |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/patterns/registry.rs -->\n```rust\n//! Built-in UX pattern registry.\n//!\n//! Adding a new pattern requires a code change — same principle as the\n//! design system registry in tech_stack.\n//!\n//! Pattern definitions will be added in a future change.\n\nuse super::UxPattern;\n\n/// Built-in pattern registry. Currently empty — patterns added incrementally.\n/// @spec apps/agentic-workflow/tech-design/core/generate/patterns/registry.md#source\npub fn pattern_registry() -> &'static [UxPattern] {\n    static REGISTRY: &[UxPattern] = &[];\n    REGISTRY\n}\n\n/// Alias for backward compatibility\npub const PATTERN_REGISTRY: &[UxPattern] = &[];\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/patterns/registry.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete built-in UX pattern registry facade.\n```\n"
