"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/rpc_api_types.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-rpc-api-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/rpc_api_types.md"
__legacy_td_digest__ = "sha256:b0f4c753ab201265a4d1906790fb31c3a1f29ed2b9c5804e39aaeccc3e5f1e6e"


def render_markdown() -> Annotated[str, "sha256:b0f4c753ab201265a4d1906790fb31c3a1f29ed2b9c5804e39aaeccc3e5f1e6e"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-rpc-api-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# RpcApiGenerator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle RpcApiGenerator unit struct. A companion source template owns the module\npreamble, `Generator` impl, and regression tests that previously lived in a\nmanaged HANDWRITE gap.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  RpcApiGenerator:\n    type: object\n    description: RpcApiGenerator unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/rpc_api.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - RpcApiGenerator\n    description: Codegen replaces RpcApiGenerator.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
