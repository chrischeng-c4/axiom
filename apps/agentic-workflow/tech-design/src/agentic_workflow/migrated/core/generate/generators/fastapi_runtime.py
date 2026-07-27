"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/fastapi_runtime.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-fastapi-runtime"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/fastapi_runtime.md"
__legacy_projection_digest__ = "sha256:a8d5c7f369235b8cb9f76deef15c6b6cf002caa518d0e003a0e76899f94c8b06"


def render_markdown() -> Annotated[str, "sha256:a8d5c7f369235b8cb9f76deef15c6b6cf002caa518d0e003a0e76899f94c8b06"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-fastapi-runtime\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# FastAPIGenerator Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/fastapi.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `FastAPIGenerator` | apps/agentic-workflow/src/generate/generators/fastapi.rs | struct | pub | 35 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/fastapi.rs | function | pub | 43 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-fastapi-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/fastapi.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-fastapi-runtime>\"\n    description: \"Source template owns the FastAPI generator runtime and regression tests.\"\n```\n"
