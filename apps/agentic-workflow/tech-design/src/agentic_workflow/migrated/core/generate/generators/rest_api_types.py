"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/rest_api_types.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-rest-api-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/rest_api_types.md"
__legacy_td_digest__ = "sha256:d589b4c500bd10e99c1601955779da052557f2ad08911aa3bee8496072df8e6b"


def render_markdown() -> Annotated[str, "sha256:d589b4c500bd10e99c1601955779da052557f2ad08911aa3bee8496072df8e6b"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-rest-api-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# RestApiGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle RestApiGenerator unit struct in generators/rest_api.rs. A companion\nsource template owns the module preamble, `Generator` impl, helper behavior, and\nregression tests that previously lived in a managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  RestApiGenerator:\n    type: object\n    description: RestApiGenerator unit struct (registered in generators/mod.rs).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/rest_api.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - RestApiGenerator\n    description: Codegen replaces RestApiGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
