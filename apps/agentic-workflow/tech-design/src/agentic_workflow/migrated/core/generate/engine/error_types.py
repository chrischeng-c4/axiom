"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/engine/error_types.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-engine-error-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/engine/error_types.md"
__legacy_td_digest__ = "sha256:d9b80a7fa3572a9545d75d317baa42a2efecec87e90ab526ad8b66b2740a217a"


def render_markdown() -> Annotated[str, "sha256:d9b80a7fa3572a9545d75d317baa42a2efecec87e90ab526ad8b66b2740a217a"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-engine-error-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# TemplateError\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle thiserror enum. Companion source templates own the module preamble and\nthe `From<tera::Error>` adapter that previously lived outside the generated\nenum declaration.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  TemplateError:\n    type: object\n    description: Template engine errors.\n    x-rust-enum:\n      derive: [Debug, \"thiserror::Error\"]\n      variants:\n        - name: NotFound\n          kind: tuple\n          error: \"Template not found: {0}\"\n          fields: [{ rust_type: String }]\n        - name: ParseError\n          kind: struct\n          error: \"Template parse error in '{template}': {message}\"\n          fields: [{ name: template, rust_type: String }, { name: message, rust_type: String }]\n        - name: RenderError\n          kind: struct\n          error: \"Template render error in '{template}': {message}\"\n          fields: [{ name: template, rust_type: String }, { name: message, rust_type: String }]\n        - name: TypeMismatch\n          kind: struct\n          error: \"Context type mismatch: expected {expected}, got {actual}\"\n          fields: [{ name: expected, rust_type: String }, { name: actual, rust_type: String }]\n        - name: FilterError\n          kind: struct\n          error: \"Filter error in '{filter}': {message}\"\n          fields: [{ name: filter, rust_type: String }, { name: message, rust_type: String }]\n        - name: DirectoryNotFound\n          kind: tuple\n          error: \"Template directory not found: {0}\"\n          fields: [{ rust_type: PathBuf }]\n        - name: Io\n          kind: tuple\n          error: \"IO error: {0}\"\n          fields: [{ rust_type: \"std::io::Error\", error_from: true }]\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/engine/error.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - TemplateError\n    description: Codegen replaces TemplateError thiserror enum.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
