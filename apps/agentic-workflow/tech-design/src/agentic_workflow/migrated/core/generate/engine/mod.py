"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/engine/mod.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-engine-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/engine/mod.md"
__legacy_projection_digest__ = "sha256:c3f29a7c5a3c97b10b94e8c05d2cda60fcf70e29e2b155b2b5e63b22358fe471"


def render_markdown() -> Annotated[str, "sha256:c3f29a7c5a3c97b10b94e8c05d2cda60fcf70e29e2b155b2b5e63b22358fe471"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-engine-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Template Engine Module Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/engine/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap standardize:claim-code -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/engine/mod.rs -->\n```rust\n//! Template Engine Module\n//!\n//! Provides Tera-based template rendering with custom filters.\n\nmod error;\nmod filters;\nmod tera_engine;\n\npub use error::TemplateError;\npub use tera_engine::TemplateEngine;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/engine/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:standardize:claim-code>\"\n    description: \"Source template owns the template engine module facade.\"\n```\n"
