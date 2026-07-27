"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/init_change_service_preamble_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-init-change-service-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/init_change_service_preamble_source.md"
__legacy_projection_digest__ = "sha256:fdca2235add15c129a42fa3ee328655d459087f54d75ebf4260dd80d2fff1cc9"


def render_markdown() -> Annotated[str, "sha256:fdca2235add15c129a42fa3ee328655d459087f54d75ebf4260dd80d2fff1cc9"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-init-change-service-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Init Change Service Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/init_change_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `CreateChangeInput` | apps/agentic-workflow/src/services/init_change_service.rs | struct | pub | 15 |  |\n| `CreateChangeResult` | apps/agentic-workflow/src/services/init_change_service.rs | struct | pub | 28 |  |\n| `create_change` | apps/agentic-workflow/src/services/init_change_service.rs | function | pub | 48 | create_change(input: CreateChangeInput, project_root: &Path) -> Result<CreateChangeResult> |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Init change service — business logic for creating new changes.\n//!\n//! Extracted from `mcp/tools/init_change.rs`.\n\nuse crate::state::StateManager;\nuse crate::Result;\nuse std::path::Path;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/init_change_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n      - \"<handwrite-gap:init-change-service-preamble>\"\n    description: \"Source template owns the init-change service docs and imports.\"\n```\n"
