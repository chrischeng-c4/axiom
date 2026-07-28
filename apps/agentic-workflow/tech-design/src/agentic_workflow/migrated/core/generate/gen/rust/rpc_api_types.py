"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/gen/rust/rpc_api_types.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-gen-rust-rpc-api-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/gen/rust/rpc_api_types.md"
__legacy_td_digest__ = "sha256:c076844057341ae16548dd10e451cf318b94aa82e81210c555480b1d852e1b7e"


def render_markdown() -> Annotated[str, "sha256:c076844057341ae16548dd10e451cf318b94aa82e81210c555480b1d852e1b7e"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-gen-rust-rpc_api-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# RpcApiGenOutput\n\n## Overview\n<!-- type: overview lang: markdown -->\n\n`RpcApiGenOutput` is generated in the canonical Rust codegen module at\n`apps/agentic-workflow/src/generate/gen/rust/rpc_api.rs` and the legacy mirror at\n`apps/agentic-workflow/src/gen/rust/rpc_api.rs`.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  RpcApiGenOutput:\n    type: object\n    description: Output from RPC-API code generation.\n    properties:\n      code:\n        type: string\n        description: The generated async fn signatures with SPEC-REF body markers.\n      spec_refs:\n        type: array\n        items: { type: string }\n        description: SPEC-REF entries emitted.\n    required: [code, spec_refs]\n    x-rust-struct:\n      derive: [Debug, Clone]\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/gen/rust/rpc_api.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - RpcApiGenOutput\n    description: Codegen replaces RpcApiGenOutput.\n  - path: apps/agentic-workflow/src/gen/rust/rpc_api.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - RpcApiGenOutput\n    description: Codegen replaces the legacy mirror of RpcApiGenOutput.\n  - path: apps/agentic-workflow/src/generate/gen/rust/rpc_api.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble, free fns, helpers, tests.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
