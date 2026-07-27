"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/gen/mod.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-gen-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/gen/mod.md"
__legacy_projection_digest__ = "sha256:e9f50ec9ab2c12b59a9ce5bedb4ba1ac0b2f53e95ab659aba2bfb0d132092ab1"


def render_markdown() -> Annotated[str, "sha256:e9f50ec9ab2c12b59a9ce5bedb4ba1ac0b2f53e95ab659aba2bfb0d132092ab1"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-gen-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/gen/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/gen/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `operations` | apps/agentic-workflow/src/generate/gen/mod.rs | module | pub | 9 |  |\n| `python` | apps/agentic-workflow/src/generate/gen/mod.rs | module | pub | 10 |  |\n| `rust` | apps/agentic-workflow/src/generate/gen/mod.rs | module | pub | 11 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-managed-markers -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/gen/mod.rs -->\n```rust\n//! Code generators for all target languages.\n//!\n//! Currently implements: Rust (structural + behavioral).\n//! Python and TypeScript translators share the same `AbstractType` enum\n//! but have deferred implementations.\n\npub mod rust;\n\npub use rust::*;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/gen/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: Source template owns the complete language generator module aggregator.\n```\n"
