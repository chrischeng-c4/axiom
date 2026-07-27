"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/fillback/factory_runtime_source.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-fillback-factory-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/fillback/factory_runtime_source.md"
__legacy_projection_digest__ = "sha256:7313f1bbc834845a4670cc2914d526eaba366317d1db3a17cbc6b24a877b48f7"


def render_markdown() -> Annotated[str, "sha256:7313f1bbc834845a4670cc2914d526eaba366317d1db3a17cbc6b24a877b48f7"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-fillback-factory-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Fillback Factory Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/fillback/factory.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `StrategyFactory` | apps/agentic-workflow/src/fillback/factory.rs | struct | pub | 17 |  |\n| `create` | apps/agentic-workflow/src/fillback/factory.rs | function | pub | 35 | create(strategy_type: &str, source: &Path) -> Result<Box<dyn ImportStrategy>> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap fillback-factory-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/fillback/factory.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:fillback-factory-runtime>\"\n    description: \"Source template owns fillback factory runtime behavior and tests.\"\n```\n"
