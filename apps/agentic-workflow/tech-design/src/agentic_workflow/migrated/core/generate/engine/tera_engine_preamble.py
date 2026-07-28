"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/engine/tera_engine_preamble.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-engine-tera-engine-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/engine/tera_engine_preamble.md"
__legacy_projection_digest__ = "sha256:7ece17ec64e463fbddd56e675564e8e712471194b3bba20548763afd888a1140"


def render_markdown() -> Annotated[str, "sha256:7ece17ec64e463fbddd56e675564e8e712471194b3bba20548763afd888a1140"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-engine-tera-engine-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# TemplateEngine Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/engine/tera_engine.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `TemplateEngine` | apps/agentic-workflow/src/generate/engine/tera_engine.rs | struct | pub | 17 |  |\n| `add_template` | apps/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 67 | add_template(&mut self, name: &str, content: &str) -> Result<(), TemplateError> |\n| `empty` | apps/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 54 | empty() -> Self |\n| `has_template` | apps/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 101 | has_template(&self, name: &str) -> bool |\n| `new` | apps/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 31 | new(template_dir: impl AsRef<Path>) -> Result<Self, TemplateError> |\n| `render` | apps/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 77 | render(         &self,         template: &str,         context: &T,     ) -> Result<String, TemplateError> |\n| `template_names` | apps/agentic-workflow/src/generate/engine/tera_engine.rs | function | pub | 106 | template_names(&self) -> impl Iterator<Item = &str> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-engine-tera-engine-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/engine/tera_engine.rs -->\n```rust\n\n//! Tera template engine wrapper\n\nuse super::error::TemplateError;\nuse super::filters;\nuse serde::Serialize;\nuse std::path::Path;\nuse tera::{Context, Tera};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/engine/tera_engine.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-engine-tera-engine-preamble>\"\n    description: \"Source template owns TemplateEngine module docs and imports.\"\n```\n"
