"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/tools/revise_change_spec/artifact.md`.

Migrated by batch `projection-core-tools-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-revise-change-spec-artifact"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/tools/revise_change_spec/artifact.md"
__legacy_projection_digest__ = "sha256:8e3aa0812f5a5a8846d932c3215aede8e31cc11c22966bd08d5238972979d41b"


def render_markdown() -> Annotated[str, "sha256:8e3aa0812f5a5a8846d932c3215aede8e31cc11c22966bd08d5238972979d41b"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-tools-revise-change-spec-artifact\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# sdd tools revise change spec artifact\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/tools/revise_change_spec.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `artifact_definition` | apps/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |\n| `execute_artifact` | apps/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 135 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |\n| `execute_workflow` | apps/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 92 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |\n| `workflow_definition` | apps/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 22 | workflow_definition() -> ToolDefinition |\n## Source\n<!-- type: source lang: rust -->\n\n````rust\n/// Execute sdd_artifact_revise_change_spec.\n///\n/// Delegates to `create::execute_artifact()` — same write behavior.\npub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {\n    let result = create::execute_artifact(args, project_root)?;\n\n    // Increment revision count so auto-approve (threshold >= 1) triggers on next review.\n    let change_id = get_required_string(args, \"change_id\")?;\n    let spec_id = get_required_string(args, \"spec_id\")?;\n    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);\n    let rev_key = format!(\"spec:{}\", spec_id);\n    if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {\n        sm.increment_revision_count(&rev_key);\n        let _ = sm.save();\n    }\n\n    Ok(result)\n}\n````\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/revise_change_spec.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"execute_artifact\"\n    description: \"Revise-change-spec artifact delegation and revision counter update.\"\n```\n"
