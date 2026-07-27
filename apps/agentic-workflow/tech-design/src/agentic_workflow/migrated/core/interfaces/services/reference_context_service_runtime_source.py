"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/reference_context_service_runtime_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-reference-context-service-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/reference_context_service_runtime_source.md"
__legacy_projection_digest__ = "sha256:5ec5a082786fbf6142c57f1f727d3d6f686311830ed7e193578b9f85bca86a2f"


def render_markdown() -> Annotated[str, "sha256:5ec5a082786fbf6142c57f1f727d3d6f686311830ed7e193578b9f85bca86a2f"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-reference-context-service-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Reference Context Service Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/reference_context_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `CreateCodebaseContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 24 |  |\n| `CreateContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | enum | pub | 44 |  |\n| `CreateKnowledgeContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 59 |  |\n| `CreateSpecContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 79 |  |\n| `create_context` | apps/agentic-workflow/src/services/reference_context_service.rs | function | pub | 105 | create_context(input: CreateContextInput, project_root: &Path) -> Result<String> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap standardize:fold-shadow -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/reference_context_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:standardize:fold-shadow>\"\n    description: \"Source template owns reference-context runtime behavior and test module link.\"\n```\n"
