"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/shared/services.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-shared-services"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/shared/services.md"
__legacy_projection_digest__ = "sha256:37174aa3591627d8df1069e3eb7269bd48235496e2ce079a93ecd982311136f3"


def render_markdown() -> Annotated[str, "sha256:37174aa3591627d8df1069e3eb7269bd48235496e2ce079a93ecd982311136f3"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-shared-services-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Shared workflow utilities support the single AW CLI across lifecycle phases.\"\n---\n\n# Standardized apps/agentic-workflow/src/shared/services.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/shared/services.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/shared/services.rs -->\n```rust\n//! Shared services\n//!\n//! Re-exports from the original services module for backward compatibility.\n\npub use crate::services::file_service;\npub use crate::services::knowledge_service;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/shared/services.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete shared service re-export module.\n```\n"
