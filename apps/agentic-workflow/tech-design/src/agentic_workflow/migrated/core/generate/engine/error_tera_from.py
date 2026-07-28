"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/engine/error_tera_from.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-engine-error-tera-from"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/engine/error_tera_from.md"
__legacy_projection_digest__ = "sha256:0d3b54c2bc79921ef43a60bfe70d100042227241e635874b055527e9c53fbfa5"


def render_markdown() -> Annotated[str, "sha256:0d3b54c2bc79921ef43a60bfe70d100042227241e635874b055527e9c53fbfa5"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-engine-error-tera-from\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Template Error Tera Adapter Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/engine/error.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `TemplateError` | apps/agentic-workflow/src/generate/engine/error.rs | enum | pub | 12 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap standardize:fold-shadow -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/engine/error.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:standardize:fold-shadow>\"\n    description: \"Source template owns the tera::Error adapter.\"\n```\n"
