"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/shared/tools.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-shared-tools"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/shared/tools.md"
__legacy_projection_digest__ = "sha256:e7ddb665bd84292417a44045cd4320ef9a277af8e6bcd96484b7cb870107429e"


def render_markdown() -> Annotated[str, "sha256:e7ddb665bd84292417a44045cd4320ef9a277af8e6bcd96484b7cb870107429e"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-shared-tools-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Shared workflow utilities support the single AW CLI across lifecycle phases.\"\n---\n\n# Standardized apps/agentic-workflow/src/shared/tools.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/shared/tools.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/shared/tools.rs -->\n```rust\n//! Shared tool re-exports\n//!\n//! Re-exports from the tools module for external access.\n\npub use crate::tools::analyze;\npub use crate::tools::knowledge;\npub use crate::tools::read;\npub use crate::tools::validate_spec;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/shared/tools.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete shared tool re-export module.\n```\n"
