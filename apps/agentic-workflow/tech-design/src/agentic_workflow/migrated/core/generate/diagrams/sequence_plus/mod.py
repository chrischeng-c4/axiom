"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/mod.md`.

Migrated by batch `projection-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-diagrams-sequence-plus-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/mod.md"
__legacy_projection_digest__ = "sha256:af1466105188a66c39ba2094341362a5d2a3ac80d8848d9d6412b499235b5f6c"


def render_markdown() -> Annotated[str, "sha256:af1466105188a66c39ba2094341362a5d2a3ac80d8848d9d6412b499235b5f6c"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-diagrams-sequence-plus-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/diagrams/sequence_plus/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/diagrams/sequence_plus/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/diagrams/sequence_plus/mod.rs -->\n```rust\n//! Sequence+ Diagram Format\n//!\n//! Enhanced sequence diagram definitions with validation and YAML frontmatter.\n//! Supports participants, messages, loops, alt/opt blocks, and activation.\n\nmod generator;\nmod schema;\nmod validator;\n\npub use generator::*;\npub use schema::*;\npub use validator::*;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/diagrams/sequence_plus/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete Sequence+ module facade.\n```\n"
