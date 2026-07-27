"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/ui/colors_types.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-ui-colors-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/ui/colors_types.md"
__legacy_td_digest__ = "sha256:c398fde89089b292c8c3acc9124e4d4383158b91e23c030925e8ecbcd39b7d9c"


def render_markdown() -> Annotated[str, "sha256:c398fde89089b292c8c3acc9124e4d4383158b91e23c030925e8ecbcd39b7d9c"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-ui-colors-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure.\"\n---\n\n# ColorScheme\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle ColorScheme unit struct (placeholder).\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ColorScheme:\n    type: object\n    description: ColorScheme placeholder unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/ui/colors.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ColorScheme\n    description: Codegen replaces ColorScheme.\n  - path: apps/agentic-workflow/src/ui/colors.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
