"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/doc_types.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-doc-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/doc_types.md"
__legacy_td_digest__ = "sha256:58e277c23442d133e0334a70fb9a76e0db35efd833ffe37855a897c44317449a"


def render_markdown() -> Annotated[str, "sha256:58e277c23442d133e0334a70fb9a76e0db35efd833ffe37855a897c44317449a"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-doc-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# DocGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle DocGenerator unit struct. A companion source template owns the module\npreamble, `Generator` impl, helper behavior, and regression tests that\npreviously lived in a managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  DocGenerator:\n    type: object\n    description: DocGenerator unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/doc.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - DocGenerator\n    description: Codegen replaces DocGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
