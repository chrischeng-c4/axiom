"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/diagrams/state_plus/mod.md`.

Migrated by batch `projection-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-diagrams-state-plus-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/diagrams/state_plus/mod.md"
__legacy_projection_digest__ = "sha256:e2c8870575248945ccd018326ac6d71a669779c593178025e78c9d2c2e9c3fa9"


def render_markdown() -> Annotated[str, "sha256:e2c8870575248945ccd018326ac6d71a669779c593178025e78c9d2c2e9c3fa9"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-diagrams-state-plus-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/diagrams/state_plus/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/diagrams/state_plus/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/diagrams/state_plus/mod.rs -->\n```rust\n//! State+ Machine Format\n//!\n//! XState-compatible state machine definitions with Mermaid stateDiagram-v2 output.\n//! This module provides schema, validation, and generation for state machines.\n\nmod generator;\nmod schema;\nmod validator;\n\npub use generator::*;\npub use schema::*;\npub use validator::*;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/diagrams/state_plus/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete State+ module declaration and exports.\n```\n"
