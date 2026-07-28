"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-flowchart-plus-gen"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen.md"
__legacy_td_digest__ = "sha256:39e2ac1ad6a5053e8e6eba225493b480c0e313d3e0b613be013efa734ccc7f1f"


def render_markdown() -> Annotated[str, "sha256:39e2ac1ad6a5053e8e6eba225493b480c0e313d3e0b613be013efa734ccc7f1f"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-flowchart-plus-gen\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# FlowchartPlusGenerator Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nUnit-struct generator in\n`apps/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs`. One shape:\n\n- `FlowchartPlusGenerator` — unit struct with no derives.\n\nCodegen replaces the unit struct declaration. Companion source templates own\nthe module preamble and runtime implementation blocks that previously lived in\nmanaged HANDWRITE gaps.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  FlowchartPlusGenerator:\n    type: object\n    required: []\n    properties: {}\n    description: FlowchartPlus generator (unit struct).\n    x-rust-struct:\n      derive: []\n      unit: true\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - FlowchartPlusGenerator\n    description: |\n      Codegen replaces the unit struct declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- [overview] Single unit struct.\n- [schema] Standard unit-struct shape.\n- [changes] Standard split.\n"
