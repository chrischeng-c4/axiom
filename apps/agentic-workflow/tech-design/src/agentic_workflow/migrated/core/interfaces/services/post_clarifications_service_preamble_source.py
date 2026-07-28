"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service_preamble_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-post-clarifications-service-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service_preamble_source.md"
__legacy_projection_digest__ = "sha256:68c8728b154a70ccd8159d518dcc0ca92ef65ccadcf61a6bfa8039a2bcc4f0df"


def render_markdown() -> Annotated[str, "sha256:68c8728b154a70ccd8159d518dcc0ca92ef65ccadcf61a6bfa8039a2bcc4f0df"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-post-clarifications-service-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Post Clarifications Service Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/post_clarifications_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `Contradiction` | apps/agentic-workflow/src/services/post_clarifications_service.rs | struct | pub | 15 |  |\n| `CreatePostClarificationsInput` | apps/agentic-workflow/src/services/post_clarifications_service.rs | struct | pub | 28 |  |\n| `PostClarificationsResult` | apps/agentic-workflow/src/services/post_clarifications_service.rs | struct | pub | 39 |  |\n| `PostQuestion` | apps/agentic-workflow/src/services/post_clarifications_service.rs | struct | pub | 50 |  |\n| `create_post_clarifications` | apps/agentic-workflow/src/services/post_clarifications_service.rs | function | pub | 66 | create_post_clarifications(     input: CreatePostClarificationsInput,     project_root: &Path, ) -> Result<PostClarificationsResult> |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Post-clarifications service — business logic for spec_clarifications.md.\n//!\n//! Extracted from `mcp/tools/clarifications.rs` (post_clarifications part).\n\nuse crate::Result;\nuse chrono::Local;\nuse std::path::Path;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/post_clarifications_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n    description: \"Source template owns post-clarifications service docs and imports.\"\n```\n"
