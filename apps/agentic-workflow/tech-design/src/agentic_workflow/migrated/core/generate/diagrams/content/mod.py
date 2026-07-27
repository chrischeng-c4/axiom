"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/diagrams/content/mod.md`.

Migrated by batch `projection-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-diagrams-content-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/diagrams/content/mod.md"
__legacy_projection_digest__ = "sha256:adb016c56e6cf916ac13f19aa7f241c68eadf42dcd500957f6da688af3e7cddd"


def render_markdown() -> Annotated[str, "sha256:adb016c56e6cf916ac13f19aa7f241c68eadf42dcd500957f6da688af3e7cddd"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-diagrams-content-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/diagrams/content/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/diagrams/content/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `interaction` | apps/agentic-workflow/src/generate/diagrams/content/mod.rs | module | pub | 8 |  |\n| `logic` | apps/agentic-workflow/src/generate/diagrams/content/mod.rs | module | pub | 9 |  |\n| `requirement` | apps/agentic-workflow/src/generate/diagrams/content/mod.rs | module | pub | 10 |  |\n| `state_machine` | apps/agentic-workflow/src/generate/diagrams/content/mod.rs | module | pub | 11 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/diagrams/content/mod.rs -->\n```rust\n//! Per-diagram Content types for Mermaid Plus codegen.\n//!\n//! Each diagram type has its own explicit Content struct (design decision D3).\n//! No universal `Graph<N,E>` — each type is statically typed and XState-free (D8).\n\npub mod interaction;\npub mod logic;\npub mod requirement;\npub mod state_machine;\n\npub use interaction::InteractionContent;\npub use logic::LogicContent;\npub use requirement::RequirementContent;\npub use state_machine::StateMachineContent;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/diagrams/content/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete content module declaration and exports.\n```\n"
