"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/db_model_types.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-db-model-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/db_model_types.md"
__legacy_td_digest__ = "sha256:9f029c005eafa4c8022dde071fb64c32d0a35ab1ac7d0a4074de6fc26ca73c89"


def render_markdown() -> Annotated[str, "sha256:9f029c005eafa4c8022dde071fb64c32d0a35ab1ac7d0a4074de6fc26ca73c89"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-db-model-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# DbModelGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle DbModelGenerator unit struct in generators/db_model.rs. A companion\nsource template owns the module preamble, `Generator` impl, helper behavior, and\nregression tests that previously lived in a managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  DbModelGenerator:\n    type: object\n    description: DbModelGenerator unit struct (registered in generators/mod.rs).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/db_model.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - DbModelGenerator\n    description: Codegen replaces DbModelGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
