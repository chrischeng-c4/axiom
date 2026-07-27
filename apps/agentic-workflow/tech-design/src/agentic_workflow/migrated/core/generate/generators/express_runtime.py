"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/express_runtime.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-express-runtime"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/express_runtime.md"
__legacy_projection_digest__ = "sha256:842aefa6bb5da99a942f38eecde8ddf0fc57fe0482d27755e39dca8318a5a129"


def render_markdown() -> Annotated[str, "sha256:842aefa6bb5da99a942f38eecde8ddf0fc57fe0482d27755e39dca8318a5a129"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-express-runtime\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# ExpressGenerator Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/express.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ExpressGenerator` | apps/agentic-workflow/src/generate/generators/express.rs | struct | pub | 16 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/express.rs | function | pub | 24 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-express-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/express.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-express-runtime>\"\n    description: \"Source template owns the Express generator runtime and regression tests.\"\n```\n"
