"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/validator/mod.md`.

Migrated by batch `projection-core-generate-04`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-validator-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/validator/mod.md"
__legacy_projection_digest__ = "sha256:a98bc0d8bfe2942e4767f5b204b029cb53fc84b19246920eb826d1d9a05a0938"


def render_markdown() -> Annotated[str, "sha256:a98bc0d8bfe2942e4767f5b204b029cb53fc84b19246920eb826d1d9a05a0938"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-validator-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/validator/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/validator/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/validator/mod.rs -->\n```rust\n//! Spec Completeness Validator\n//!\n//! Validates JSON Schemas and SpecIR payloads for completeness before code generation.\n//!\n//! ## Modules\n//!\n//! - [`completeness`] — JSON Schema type/ref/description validation (R1–R3)\n//! - [`spec_ir_validator`] — SpecIR section-type validators (deploy, wireframe,\n//!   component, design-token) with shared registration mechanism\n\nmod completeness;\nmod spec_ir_validator;\n\npub use completeness::{validate_schema, Severity, ValidationIssue, ValidationResult};\npub use spec_ir_validator::{\n    validate_spec_ir, ComponentValidator, DeployValidator, DesignTokenValidator, SpecIRValidator,\n    WireframeValidator,\n};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/validator/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete validator module facade.\n```\n"
