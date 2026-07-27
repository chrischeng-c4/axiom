"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/tools/spec/preamble.md`.

Migrated by batch `projection-core-tools-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-spec-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/tools/spec/preamble.md"
__legacy_projection_digest__ = "sha256:236831d2015bd49fc4137cfb3995a05e52a6d63cc5849118f16ab811640d870b"


def render_markdown() -> Annotated[str, "sha256:236831d2015bd49fc4137cfb3995a05e52a6d63cc5849118f16ab811640d870b"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-tools-spec-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# sdd tools spec preamble\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/tools/spec.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `definition` | apps/agentic-workflow/src/tools/spec.rs | function | pub | 41 | definition() -> ToolDefinition |\n| `execute` | apps/agentic-workflow/src/tools/spec.rs | function | pub | 238 | execute(args: &Value, project_root: &Path) -> Result<String> |\n| `execute_review_spec` | apps/agentic-workflow/src/tools/spec.rs | function | pub | 574 | execute_review_spec(args: &Value, project_root: &Path) -> Result<String> |\n| `review_spec_definition` | apps/agentic-workflow/src/tools/spec.rs | function | pub | 476 | review_spec_definition() -> ToolDefinition |\n## Source\n<!-- type: source lang: rust -->\n\n````rust\n//! create_spec MCP Tool\n//!\n//! Creates a validated spec file with requirements and acceptance criteria.\n#![allow(deprecated)]\n//!\n//! ## Structured Diagrams\n//!\n//! The `diagrams` field accepts structured diagram definitions that are validated\n//! against their corresponding Mermaid tool schemas. This ensures diagrams are\n//! syntactically correct and enables semantic metadata for code generation.\n//!\n//! Supported diagram types:\n//! - `flowchart` - Process flows, algorithms, decision trees (with semantic extensions)\n//! - `sequence` - API interactions, message flows\n//! - `class` - Data structures, domain models\n//! - `state` - State machines, workflow states\n//! - `erd` - Database schemas, entity relationships\n//! - `mindmap` - Concept organization\n//! - `requirement` - Requirement traceability\n//! - `journey` - User journeys\n\nuse super::{get_optional_string, get_required_array, get_required_string, ToolDefinition};\nuse crate::models::spec_rules::{ApiSpecType, SpecType};\nuse crate::models::state::StatePhase;\nuse crate::services::spec_service::{\n    create_spec, ApiSpecData, CreateSpecInput, DiagramData, RequirementData, ScenarioData,\n    SpecChangeData,\n};\nuse crate::Result;\nuse serde_json::{json, Value};\nuse std::path::Path;\nuse std::str::FromStr;\n````\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/spec.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - <module-preamble>\n      - <module-trailer>\n    description: \"Module preamble and whole-file HANDWRITE edge markers.\"\n```\n"
