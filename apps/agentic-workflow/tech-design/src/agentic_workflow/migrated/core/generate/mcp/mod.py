"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/mcp/mod.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-mcp-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/mcp/mod.md"
__legacy_projection_digest__ = "sha256:b7758360143d853300ee5fda3cd57754cb8726f82cbf9b468b2b137836f23f77"


def render_markdown() -> Annotated[str, "sha256:b7758360143d853300ee5fda3cd57754cb8726f82cbf9b468b2b137836f23f77"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-mcp-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/mcp/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/mcp/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/mcp/mod.rs -->\n```rust\n//! MCP Tool Definitions for SDD Generate\n//!\n//! Exposes diagram and spec generation as MCP tools.\n\nmod handlers;\nmod tools;\n\npub use handlers::*;\npub use tools::SddTools;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/mcp/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete SDD generate MCP module facade.\n```\n"
