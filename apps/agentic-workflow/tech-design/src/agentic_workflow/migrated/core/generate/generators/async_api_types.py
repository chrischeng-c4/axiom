"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/async_api_types.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-async-api-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/async_api_types.md"
__legacy_td_digest__ = "sha256:37a3b186f39dd47a071dba32412a756ad874009e5cd881b098186673b43d4fbd"


def render_markdown() -> Annotated[str, "sha256:37a3b186f39dd47a071dba32412a756ad874009e5cd881b098186673b43d4fbd"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-async-api-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# AsyncApiGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle AsyncApiGenerator unit struct in generators/async_api.rs. A companion\nsource template owns the module preamble, `Generator` impl, helper behavior, and\nregression tests that previously lived in a managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  AsyncApiGenerator:\n    type: object\n    description: AsyncApiGenerator unit struct (registered in generators/mod.rs).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/async_api.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - AsyncApiGenerator\n    description: Codegen replaces AsyncApiGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
