"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/validate/validator/mod.md`.

Migrated by batch `projection-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-validator-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/validate/validator/mod.md"
__legacy_projection_digest__ = "sha256:f016675918909d6dcdbaf96487f89d3e22a03bf9ea78f44f124aaa5c199db773"


def render_markdown() -> Annotated[str, "sha256:f016675918909d6dcdbaf96487f89d3e22a03bf9ea78f44f124aaa5c199db773"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-validator-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: managed-and-semantic-production-gates\n    claim: managed-and-semantic-production-gates\n    coverage: full\n    rationale: \"Validation TDs implement managed and semantic production gates for standardization readiness.\"\n---\n\n# Standardized apps/agentic-workflow/src/validator/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/validator/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `challenge` | apps/agentic-workflow/src/validator/mod.rs | module | pub | 3 |  |\n| `consistency` | apps/agentic-workflow/src/validator/mod.rs | module | pub | 4 |  |\n| `fix` | apps/agentic-workflow/src/validator/mod.rs | module | pub | 5 |  |\n| `format` | apps/agentic-workflow/src/validator/mod.rs | module | pub | 6 |  |\n| `schema` | apps/agentic-workflow/src/validator/mod.rs | module | pub | 7 |  |\n| `semantic` | apps/agentic-workflow/src/validator/mod.rs | module | pub | 8 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/validator/mod.rs -->\n```rust\npub mod challenge;\npub mod consistency;\npub mod fix;\npub mod format;\npub mod schema;\npub mod semantic;\n\npub use challenge::ChallengeValidator;\npub use consistency::ConsistencyValidator;\npub use fix::{AutoFixer, FixResult};\npub use format::SpecFormatValidator;\npub use schema::{\n    validate_frontmatter_content, validate_frontmatter_schema, DocumentType, SchemaValidator,\n};\npub use semantic::SemanticValidator;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validator/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Codegen owns the module facade through a source template.\n```\n"
