"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/schema/mod.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-schema-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/schema/mod.md"
__legacy_projection_digest__ = "sha256:259b636cd5820fe89fc6f3d2a5b504f0c25c98db09c5be1e612b467e83905163"


def render_markdown() -> Annotated[str, "sha256:259b636cd5820fe89fc6f3d2a5b504f0c25c98db09c5be1e612b467e83905163"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-schema-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/schema/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/schema/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-managed-markers -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/schema/mod.rs -->\n```rust\n//! JSON Schema Core Implementation\n//!\n//! Provides strongly-typed structures for JSON Schema Draft 7 and Draft 2020-12.\n\nmod parser;\nmod types;\n\npub use parser::*;\npub use types::*;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/schema/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: Source template owns the complete JSON Schema module facade.\n```\n"
