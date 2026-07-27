"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/ui/tables_types.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-ui-tables-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/ui/tables_types.md"
__legacy_td_digest__ = "sha256:198246581806bcc7f30e2df9b1b21e7737d7226cae45c6ad61592855ee7862d9"


def render_markdown() -> Annotated[str, "sha256:198246581806bcc7f30e2df9b1b21e7737d7226cae45c6ad61592855ee7862d9"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-ui-tables-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure.\"\n---\n\n# Table\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle Table unit struct (placeholder).\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  Table:\n    type: object\n    description: Table placeholder unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/ui/tables.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - Table\n    description: Codegen replaces Table.\n  - path: apps/agentic-workflow/src/ui/tables.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
