"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/dependency_types.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-dependency-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/dependency_types.md"
__legacy_td_digest__ = "sha256:c97c256a27acd035dd02512efb652699c91736463a8431776278e73bbcca5651"


def render_markdown() -> Annotated[str, "sha256:c97c256a27acd035dd02512efb652699c91736463a8431776278e73bbcca5651"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-dependency-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# DependencyGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle DependencyGenerator unit struct in generators/dependency.rs. A companion\nsource template owns the module preamble, `Generator` impl, helper behavior, and\nregression tests that previously lived in a managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  DependencyGenerator:\n    type: object\n    description: DependencyGenerator unit struct (registered in generators/mod.rs).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/dependency.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - DependencyGenerator\n    description: Codegen replaces DependencyGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
