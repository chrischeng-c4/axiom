"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/axum_runtime.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-axum-runtime"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/axum_runtime.md"
__legacy_projection_digest__ = "sha256:aaeb8790a0c1a993afc05aad7cc6b17dc6430c23d48b6b4333ebcea068cbfb1a"


def render_markdown() -> Annotated[str, "sha256:aaeb8790a0c1a993afc05aad7cc6b17dc6430c23d48b6b4333ebcea068cbfb1a"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-axum-runtime\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# AxumGenerator Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/axum.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `AxumGenerator` | apps/agentic-workflow/src/generate/generators/axum.rs | struct | pub | 16 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/axum.rs | function | pub | 24 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-axum-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/axum.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-axum-runtime>\"\n    description: \"Source template owns the Axum generator runtime and regression tests.\"\n```\n"
