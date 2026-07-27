"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/serverless_workflow_types.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-serverless-workflow-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/serverless_workflow_types.md"
__legacy_td_digest__ = "sha256:2f972b2bcb34053d98032da9943e1d5033a8b92d5e276545848039547a62b4ae"


def render_markdown() -> Annotated[str, "sha256:2f972b2bcb34053d98032da9943e1d5033a8b92d5e276545848039547a62b4ae"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-serverless-workflow-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# ServerlessWorkflowGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle ServerlessWorkflowGenerator unit struct in generators/serverless_workflow.rs.\nA companion source template owns the module preamble, `Generator` impl, helper\nbehavior, and regression tests that previously lived in a managed HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ServerlessWorkflowGenerator:\n    type: object\n    description: ServerlessWorkflowGenerator unit struct (registered in generators/mod.rs).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/serverless_workflow.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ServerlessWorkflowGenerator\n    description: Codegen replaces ServerlessWorkflowGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
