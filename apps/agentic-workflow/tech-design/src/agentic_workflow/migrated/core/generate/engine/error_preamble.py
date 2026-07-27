"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/engine/error_preamble.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-engine-error-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/engine/error_preamble.md"
__legacy_projection_digest__ = "sha256:7ae9cf674ac181dffbeb596ce44e9d87ea5463376e7d30f7c18e05f2c1e92035"


def render_markdown() -> Annotated[str, "sha256:7ae9cf674ac181dffbeb596ce44e9d87ea5463376e7d30f7c18e05f2c1e92035"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-engine-error-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Template Error Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/engine/error.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `TemplateError` | apps/agentic-workflow/src/generate/engine/error.rs | enum | pub | 12 |  |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Template engine error types\n\nuse std::path::PathBuf;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/engine/error.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n    description: \"Source template owns template error module docs and imports.\"\n```\n"
