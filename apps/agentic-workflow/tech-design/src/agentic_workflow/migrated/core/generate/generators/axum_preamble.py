"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/axum_preamble.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-axum-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/axum_preamble.md"
__legacy_projection_digest__ = "sha256:63506d38a294b529c0102aea22faba1d34de9c1d31d0a1e51ba3f33b879d494f"


def render_markdown() -> Annotated[str, "sha256:63506d38a294b529c0102aea22faba1d34de9c1d31d0a1e51ba3f33b879d494f"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-axum-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# AxumGenerator Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/axum.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `AxumGenerator` | apps/agentic-workflow/src/generate/generators/axum.rs | struct | pub | 16 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/axum.rs | function | pub | 24 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-axum-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/generators/axum.rs -->\n```rust\n\n//! Axum code generator\n\nuse super::common::{\n    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,\n};\nuse crate::generate::engine::TemplateEngine;\nuse crate::generate::schema::{JsonSchema, SchemaType};\nuse serde::Serialize;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/axum.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-axum-preamble>\"\n    description: \"Source template owns module docs and imports.\"\n```\n"
