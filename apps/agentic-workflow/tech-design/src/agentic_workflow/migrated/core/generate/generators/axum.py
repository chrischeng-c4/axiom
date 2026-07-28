"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/axum.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-axum"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/axum.md"
__legacy_td_digest__ = "sha256:6f767722e1c9b0b6e27b0bc0a6f952c1a42750e1003b46ced0acf6adcc3ec13b"


def render_markdown() -> Annotated[str, "sha256:6f767722e1c9b0b6e27b0bc0a6f952c1a42750e1003b46ced0acf6adcc3ec13b"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-axum\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# AxumGenerator Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nUnit-struct generator in\n`apps/agentic-workflow/src/generate/generators/axum.rs`. One shape:\n\n- `AxumGenerator` — unit struct with no derives.\n\nCodegen replaces only the unit struct declaration. Module imports,\nthe `impl AxumGenerator { new, ... }` block, the\n`impl Generator for AxumGenerator` (or similar trait impls),\nand tests are owned by sibling source templates that replace the legacy\nHANDWRITE gaps.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  AxumGenerator:\n    type: object\n    required: []\n    properties: {}\n    description: Axum generator (unit struct).\n    x-rust-struct:\n      derive: []\n      unit: true\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/axum.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - AxumGenerator\n    description: |\n      Codegen replaces the unit struct declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- [overview] Single unit struct.\n- [schema] Standard unit-struct shape.\n- [changes] Standard split.\n"
