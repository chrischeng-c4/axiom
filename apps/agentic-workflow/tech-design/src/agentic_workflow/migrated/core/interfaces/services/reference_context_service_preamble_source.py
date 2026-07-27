"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/reference_context_service_preamble_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-reference-context-service-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/reference_context_service_preamble_source.md"
__legacy_projection_digest__ = "sha256:425165d0e404b1f451bce93e5b8689bf5a50d70946d09165a8e9ffa8c860a440"


def render_markdown() -> Annotated[str, "sha256:425165d0e404b1f451bce93e5b8689bf5a50d70946d09165a8e9ffa8c860a440"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-reference-context-service-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Reference Context Service Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/reference_context_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `CreateCodebaseContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 24 |  |\n| `CreateContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | enum | pub | 44 |  |\n| `CreateKnowledgeContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 59 |  |\n| `CreateSpecContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 79 |  |\n| `create_context` | apps/agentic-workflow/src/services/reference_context_service.rs | function | pub | 105 | create_context(input: CreateContextInput, project_root: &Path) -> Result<String> |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Context service - Business logic for structured context artifact creation\n//!\n//! Each context type (spec, knowledge, codebase) has its own input struct\n//! with type-specific validation and markdown rendering. The output is a\n//! structured index (what was scanned, what was found, where it lives)\n//! rather than a free-form copy of content.\n\nuse crate::models::context::{DocRef, FileRef, LensResult, PatternRef, SpecRef};\nuse crate::Result;\nuse chrono::Utc;\nuse std::path::Path;\n\n// ---------------------------------------------------------------------------\n// Input structs\n// ---------------------------------------------------------------------------\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/reference_context_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n    description: \"Source template owns reference-context service documentation and imports.\"\n```\n"
