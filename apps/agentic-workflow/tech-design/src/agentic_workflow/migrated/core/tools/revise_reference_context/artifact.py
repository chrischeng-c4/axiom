"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/tools/revise_reference_context/artifact.md`.

Migrated by batch `projection-core-tools-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-revise-reference-context-artifact"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/tools/revise_reference_context/artifact.md"
__legacy_projection_digest__ = "sha256:f71d20c2f9833ba5dea835d93600cf63886d6c7c8878923da6ea176a4c896060"


def render_markdown() -> Annotated[str, "sha256:f71d20c2f9833ba5dea835d93600cf63886d6c7c8878923da6ea176a4c896060"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-tools-revise-reference-context-artifact\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# sdd tools revise reference context artifact\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/tools/revise_reference_context.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `artifact_definition` | apps/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 45 | artifact_definition() -> ToolDefinition |\n| `execute_artifact` | apps/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 148 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |\n| `execute_workflow` | apps/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 108 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |\n| `workflow_definition` | apps/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |\n## Source\n<!-- type: source lang: rust -->\n\n````rust\n// ─── Artifact Revise ─────────────────────────────────────────────────────────\n\n/// Execute sdd_artifact_revise_reference_context.\n///\n/// Delegates to `create::execute_artifact()` for writing, then increments revision count.\n/// This ensures auto-approve triggers regardless of whether the revise was done by an agent\n/// or by mainthread (the workflow_common agent-dispatch post-hook only covers the agent path).\npub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {\n    let result = create::execute_artifact(args, project_root)?;\n\n    // Increment revision count so auto-approve (threshold >= 1) can trigger on next review.\n    let change_id = get_required_string(args, \"change_id\")?;\n    let group_id = get_required_string(args, \"group_id\")?;\n    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);\n    let rev_key = format!(\"ref_ctx:{}\", group_id);\n    if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {\n        sm.increment_revision_count(&rev_key);\n        let _ = sm.save();\n    }\n\n    Ok(result)\n}\n````\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/revise_reference_context.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"execute_artifact\"\n    description: \"Artifact writer delegation for revised reference context plus revision-count tracking.\"\n```\n"
