"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/sequence_types.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-sequence-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/sequence_types.md"
__legacy_td_digest__ = "sha256:326e985bc609680cfe0565fd5607ab25c6409f149ab1ff6382561211e31fe18a"


def render_markdown() -> Annotated[str, "sha256:326e985bc609680cfe0565fd5607ab25c6409f149ab1ff6382561211e31fe18a"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-sequence-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# SequenceGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle SequenceGenerator unit struct. A companion source template owns the\nmodule preamble, `Generator` impl, and regression tests that previously lived in\na managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  SequenceGenerator:\n    type: object\n    description: SequenceGenerator unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/sequence.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - SequenceGenerator\n    description: Codegen replaces SequenceGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
