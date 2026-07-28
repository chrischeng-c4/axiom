"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/cclab_api_preamble.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-cclab-api-preamble"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/cclab_api_preamble.md"
__legacy_td_digest__ = "sha256:f52dbee2438c311939af5790cf88014532dd529e1586b26a59a34705fb17bf4c"


def render_markdown() -> Annotated[str, "sha256:f52dbee2438c311939af5790cf88014532dd529e1586b26a59a34705fb17bf4c"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-cclab-api-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# CclabApiGenerator Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/cclab_api.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `CclabApiGenerator` | apps/agentic-workflow/src/generate/generators/cclab_api.rs | struct | pub | 35 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/cclab_api.rs | function | pub | 43 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-cclab-api-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/generators/cclab_api.rs -->\n```rust\n\n//! cclab.api code generator\n//!\n//! Generates a complete cclab ecosystem feature module from JSON Schema / OpenAPI input:\n//!\n//! | Output file       | Description |\n//! |-------------------|-------------|\n//! | `__init__.py`     | Module docstring |\n//! | `models.py`       | `{Resource}DB(Base)` ORM model using `cclab.pg` types |\n//! | `schemas.py`      | `{Resource}Create/Update/Response/ListResponse` using `cclab.schema` |\n//! | `repository.py`   | Async `{Resource}Repository` with CRUD skeleton |\n//! | `routes.py`       | `cclab.api.Router` with typed handlers |\n//!\n//! Output is structured under `features/{domain}/` following the Conductor BE convention.\n\nuse super::common::{\n    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,\n};\nuse crate::generate::engine::TemplateEngine;\nuse crate::generate::schema::{JsonSchema, SchemaType, StringFormat};\nuse serde::Serialize;\nuse std::collections::BTreeMap;\n\n// ---------------------------------------------------------------------------\n// CclabApiGenerator\n// ---------------------------------------------------------------------------\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/cclab_api.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-cclab-api-preamble>\"\n    description: \"Source template owns module docs, imports, and the generator section header.\"\n```\n"
