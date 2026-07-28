"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/state_machine_types.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-state-machine-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/state_machine_types.md"
__legacy_td_digest__ = "sha256:f8854f0d3d0b7410071434a05e4b76f1b9a492fa9c2412b81c429b56cdb54f8d"


def render_markdown() -> Annotated[str, "sha256:f8854f0d3d0b7410071434a05e4b76f1b9a492fa9c2412b81c429b56cdb54f8d"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-state-machine-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# StateMachineGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle StateMachineGenerator unit struct. A companion source template owns the\nmodule preamble, `Generator` impl, and regression tests that previously lived in\na managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  StateMachineGenerator:\n    type: object\n    description: StateMachineGenerator unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/state_machine.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - StateMachineGenerator\n    description: Codegen replaces StateMachineGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
