"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/fastapi_preamble.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-fastapi-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/fastapi_preamble.md"
__legacy_projection_digest__ = "sha256:a517c3783ddc37562ab1a9183e93fb9a6827ebe096934501b34b5c2909c605d3"


def render_markdown() -> Annotated[str, "sha256:a517c3783ddc37562ab1a9183e93fb9a6827ebe096934501b34b5c2909c605d3"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-fastapi-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# FastAPIGenerator Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/fastapi.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `FastAPIGenerator` | apps/agentic-workflow/src/generate/generators/fastapi.rs | struct | pub | 35 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/fastapi.rs | function | pub | 43 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-fastapi-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/generators/fastapi.rs -->\n```rust\n\n//! FastAPI code generator\n//!\n//! Generates a standard FastAPI project layout from a JSON Schema / OpenAPI input:\n//!\n//! | Output file      | Source section | Description |\n//! |------------------|----------------|-------------|\n//! | `models.py`      | schema         | Pydantic `BaseModel` definitions |\n//! | `schemas.py`     | schema         | Create/Update/Response wrappers (cross-section) |\n//! | `routes.py`      | rest-api × schema | `APIRouter` with typed handlers |\n//! | `app.py`         | project config | FastAPI app entry-point |\n//! | `requirements.txt` | project config | Python dependencies |\n//!\n//! Cross-section composition (Phase 2): route handlers reference both the base\n//! models (`models.py`) and the request/response schemas (`schemas.py`), tying\n//! the rest-api and schema sections together.\n\nuse super::common::{\n    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,\n};\nuse crate::generate::engine::TemplateEngine;\nuse crate::generate::schema::{JsonSchema, SchemaType};\nuse serde::Serialize;\nuse std::collections::BTreeMap;\n\n// ---------------------------------------------------------------------------\n// FastAPI code generator\n// ---------------------------------------------------------------------------\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/fastapi.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-fastapi-preamble>\"\n    description: \"Source template owns module docs, imports, and the generator section header.\"\n```\n"
