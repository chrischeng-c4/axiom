"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/deploy_runtime.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-deploy-runtime"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/deploy_runtime.md"
__legacy_projection_digest__ = "sha256:b4e5e4c3faee64f9ed182740fb0d098390054b0b2986e8e23a252f00df59ef55"


def render_markdown() -> Annotated[str, "sha256:b4e5e4c3faee64f9ed182740fb0d098390054b0b2986e8e23a252f00df59ef55"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-deploy-runtime\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# DeployGenerator Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/deploy.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `DeployGenerator` | apps/agentic-workflow/src/generate/generators/deploy.rs | struct | pub | 30 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/deploy.rs | function | pub | 38 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-deploy-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/deploy.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-deploy-runtime>\"\n    description: \"Source template owns the deploy generator runtime and regression tests.\"\n```\n"
