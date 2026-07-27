"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/fillback/speckit_runtime_source.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-fillback-speckit-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/fillback/speckit_runtime_source.md"
__legacy_projection_digest__ = "sha256:4b34b1b871cc9850f5c61297fd42d744e6afcd361135b8982676781406796726"


def render_markdown() -> Annotated[str, "sha256:4b34b1b871cc9850f5c61297fd42d744e6afcd361135b8982676781406796726"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-fillback-speckit-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Fillback Speckit Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/fillback/speckit.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `SpeckitStrategy` | apps/agentic-workflow/src/fillback/speckit.rs | struct | pub | 13 |  |\n| `new` | apps/agentic-workflow/src/fillback/speckit.rs | function | pub | 20 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap fillback-speckit-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/fillback/speckit.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:fillback-speckit-runtime>\"\n    description: \"Source template owns fillback Speckit runtime behavior and tests.\"\n```\n"
