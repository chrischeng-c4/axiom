"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/mod.md`.

Migrated by batch `projection-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-diagrams-mermaid-plus-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/mod.md"
__legacy_projection_digest__ = "sha256:8add1a5016b58c49366c863d3d9d670684ebc6812d27594f21de525a5cfbf5df"


def render_markdown() -> Annotated[str, "sha256:8add1a5016b58c49366c863d3d9d670684ebc6812d27594f21de525a5cfbf5df"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-diagrams-mermaid-plus-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `migrate` | apps/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs | module | pub | 9 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs -->\n```rust\n//! Mermaid+ State Machine Format\n//!\n//! XState-compatible state machine definitions with Mermaid stateDiagram-v2 output.\n//! This module provides the core types and generator, independent of Lens IR.\n\nmod generator;\npub mod migrate;\nmod schema;\nmod validator;\n\npub use generator::*;\npub use migrate::{\n    apply_block_payload, detect_diagram_kind, enumerate_envelopes, mermaid_equivalent,\n    run_migration, DiagramKind, MigrateState, MigrationEnvelope, MigrationOptions,\n    MIGRATE_TOOL_VERSION, PAYLOAD_DIR,\n};\npub use schema::*;\npub use validator::*;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete Mermaid+ module declaration and exports.\n```\n"
