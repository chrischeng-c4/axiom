"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/tools/implementation/trailer.md`.

Migrated by batch `projection-core-tools-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-implementation-trailer"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/tools/implementation/trailer.md"
__legacy_projection_digest__ = "sha256:7c8ad5e733f93e129f435f10160a35e038eb24619fa57b3d7bed33c87a48b5af"


def render_markdown() -> Annotated[str, "sha256:7c8ad5e733f93e129f435f10160a35e038eb24619fa57b3d7bed33c87a48b5af"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-tools-implementation-trailer-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# Implementation Tools Trailer\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/tools/implementation.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `create_merge_review_definition` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 687 | create_merge_review_definition() -> ToolDefinition |\n| `create_review_definition` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 477 | create_review_definition() -> ToolDefinition |\n| `execute_create_merge_review` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 781 | execute_create_merge_review(args: &Value, project_root: &Path) -> Result<String> |\n| `execute_create_review` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 559 | execute_create_review(args: &Value, project_root: &Path) -> Result<String> |\n| `execute_list_changed_files` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 356 | execute_list_changed_files(args: &Value, _project_root: &Path) -> Result<String> |\n| `execute_read_all_requirements` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 117 | execute_read_all_requirements(args: &Value, project_root: &Path) -> Result<String> |\n| `execute_read_implementation_summary` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 232 | execute_read_implementation_summary(args: &Value, _project_root: &Path) -> Result<String> |\n| `list_changed_files_definition` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 323 | list_changed_files_definition() -> ToolDefinition |\n| `read_all_requirements_definition` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 93 | read_all_requirements_definition() -> ToolDefinition |\n| `read_implementation_summary_definition` | apps/agentic-workflow/src/tools/implementation.rs | function | pub | 204 | read_implementation_summary_definition() -> ToolDefinition |\n## Source\n<!-- type: source lang: rust -->\n\n````rust\n// End of implementation support MCP tools.\n````\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/implementation.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-trailer>\"\n    description: \"Module trailer replacement that removes the whole-file HANDWRITE closer.\"\n```\n"
