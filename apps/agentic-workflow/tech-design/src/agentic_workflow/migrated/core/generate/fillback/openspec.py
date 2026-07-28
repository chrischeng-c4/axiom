"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/fillback/openspec.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-fillback-openspec"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/fillback/openspec.md"
__legacy_td_digest__ = "sha256:9c011b9a3966d3219ea45ca87fd064fb785e3b5b2d53c1d19ee655a49fb8c174"


def render_markdown() -> Annotated[str, "sha256:9c011b9a3966d3219ea45ca87fd064fb785e3b5b2d53c1d19ee655a49fb8c174"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-fillback-openspec\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# OpenSpecStrategy Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nImport strategy unit struct in\n`apps/agentic-workflow/src/fillback/openspec.rs`. One shape:\n\n- `OpenSpecStrategy` — unit struct with no derives.\n\nCodegen replaces the unit struct declaration. Companion source specs own the\nmodule imports, OpenSpec DTOs, conversion helpers, strategy implementation,\nand tests.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  OpenSpecStrategy:\n    type: object\n    required: []\n    properties: {}\n    description: |\n      OpenSpec import strategy (parses OpenSpec YAML/JSON specs).\n    x-rust-struct:\n      derive: []\n      unit: true\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/fillback/openspec.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - OpenSpecStrategy\n    description: |\n      Codegen replaces the unit struct declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Single unit struct; impls hand-written.\n- [schema] Standard unit-struct shape.\n- [changes] Standard split.\n"
