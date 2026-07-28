"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/overview_types.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-overview-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/overview_types.md"
__legacy_td_digest__ = "sha256:93432812a70e91331c9ecd5247933d1d33fcf54c9a2f31f84c7e53031e7f4722"


def render_markdown() -> Annotated[str, "sha256:93432812a70e91331c9ecd5247933d1d33fcf54c9a2f31f84c7e53031e7f4722"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-overview-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# OverviewGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle OverviewGenerator unit struct. A companion source template owns the\nmodule preamble, `Generator` impl, helper behavior, and regression tests that\npreviously lived in a managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  OverviewGenerator:\n    type: object\n    description: OverviewGenerator unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/overview.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - OverviewGenerator\n    description: Codegen replaces OverviewGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
