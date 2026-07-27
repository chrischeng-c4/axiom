"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/flowchart_types.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-flowchart-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/flowchart_types.md"
__legacy_td_digest__ = "sha256:6a00f7b4c6c9ee2425c40c049673857ef55e59e60b04f81fdbdd8e0bb42e8e5a"


def render_markdown() -> Annotated[str, "sha256:6a00f7b4c6c9ee2425c40c049673857ef55e59e60b04f81fdbdd8e0bb42e8e5a"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-flowchart-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# FlowchartGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle FlowchartGenerator unit struct. A companion source template owns the\nmodule preamble, `Generator` impl, and regression tests that previously lived in\na managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  FlowchartGenerator:\n    type: object\n    description: FlowchartGenerator unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/flowchart.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - FlowchartGenerator\n    description: Codegen replaces FlowchartGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
