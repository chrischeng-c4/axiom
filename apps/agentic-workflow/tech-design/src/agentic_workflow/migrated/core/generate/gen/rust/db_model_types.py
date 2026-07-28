"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/gen/rust/db_model_types.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-gen-rust-db-model-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/gen/rust/db_model_types.md"
__legacy_td_digest__ = "sha256:2ce55f5419a6efa21df69b94f56d8d691e2a280a3cc1b69ed822559a0564e816"


def render_markdown() -> Annotated[str, "sha256:2ce55f5419a6efa21df69b94f56d8d691e2a280a3cc1b69ed822559a0564e816"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-gen-rust-db_model-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# DbModelGenOutput\n\n## Overview\n<!-- type: overview lang: markdown -->\n\n`DbModelGenOutput` is generated in the canonical Rust codegen module at\n`apps/agentic-workflow/src/generate/gen/rust/db_model.rs` and the legacy mirror at\n`apps/agentic-workflow/src/gen/rust/db_model.rs`.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  DbModelGenOutput:\n    type: object\n    description: Output from DB-model code generation.\n    properties:\n      code:\n        type: string\n        description: The generated Rust struct(s) with sqlx derives.\n    required: [code]\n    x-rust-struct:\n      derive: [Debug, Clone]\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/gen/rust/db_model.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - DbModelGenOutput\n    description: Codegen replaces DbModelGenOutput.\n  - path: apps/agentic-workflow/src/gen/rust/db_model.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - DbModelGenOutput\n    description: Codegen replaces the legacy mirror of DbModelGenOutput.\n  - path: apps/agentic-workflow/src/generate/gen/rust/db_model.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble, free fns, helpers, tests.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
