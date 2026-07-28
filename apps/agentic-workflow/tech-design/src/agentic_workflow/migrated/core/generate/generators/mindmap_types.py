"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/mindmap_types.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-mindmap-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/mindmap_types.md"
__legacy_td_digest__ = "sha256:c87088014a9ee83270ab9e25fecdfa70bb10ed290726d0c99d94ffcd0f56fbe1"


def render_markdown() -> Annotated[str, "sha256:c87088014a9ee83270ab9e25fecdfa70bb10ed290726d0c99d94ffcd0f56fbe1"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-mindmap-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# MindmapGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle MindmapGenerator unit struct. A companion source template owns the module\npreamble, `Generator` impl, helper behavior, and regression tests that\npreviously lived in a managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  MindmapGenerator:\n    type: object\n    description: MindmapGenerator unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/mindmap.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - MindmapGenerator\n    description: Codegen replaces MindmapGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
