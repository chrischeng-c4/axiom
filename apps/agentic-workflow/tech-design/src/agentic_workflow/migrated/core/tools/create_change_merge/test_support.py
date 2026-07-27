"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/tools/create_change_merge/test-support.md`.

Migrated by batch `projection-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-create-change-merge-test-support"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/tools/create_change_merge/test-support.md"
__legacy_projection_digest__ = "sha256:2eb95b8d6d7796916b278cac291e8dad4516d54ad677e15d0f97b27f9ca91a7b"


def render_markdown() -> Annotated[str, "sha256:2eb95b8d6d7796916b278cac291e8dad4516d54ad677e15d0f97b27f9ca91a7b"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-tools-create-change-merge-test-support\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# sdd tools create change merge test support\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `execute_workflow` | apps/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |\n| `workflow_definition` | apps/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |\n## Source\n<!-- type: source lang: rust -->\n\n````rust\n// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/tools/create_change_merge/test-support.md#source\n// CODEGEN-BEGIN\n#[cfg(test)]\nmod test_support {\n    use super::*;\n    use crate::state::StateManager;\n    use tempfile::TempDir;\n\n    pub(super) fn setup_change(change_id: &str, phase: StatePhase) -> TempDir {\n        let tmp = TempDir::new().unwrap();\n        let change_dir = crate::shared::workspace::change_path(tmp.path(), change_id);\n        std::fs::create_dir_all(&change_dir).unwrap();\n        std::fs::create_dir_all(tmp.path().join(\"tech-design\")).unwrap();\n        // R4: save() needs an issue backing change_id.\n        crate::test_util::write_minimal_issue(tmp.path(), change_id);\n\n        // Write minimal config.toml with required platform sections\n        let config_content = r#\"\n[agentic_workflow.repo_platform]\ntype = \"github\"\nrepo = \"test/repo\"\ndefault_branch = \"main\"\nauto_commit = false\nauto_pr = false\n\n[agentic_workflow.tech_design_platform]\ntype = \"local\"\npath = \"tech-design\"\n\"#;\n        std::fs::write(tmp.path().join(\"aw.toml\"), config_content).unwrap();\n\n        let mut sm = StateManager::load(&change_dir).unwrap();\n        sm.state_mut().phase = phase;\n        sm.save().unwrap();\n\n        tmp\n    }\n}\n// CODEGEN-END\n````\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/create_change_merge.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"tests\"\n    description: \"Shared test setup helper for create-change-merge regression tests.\"\n```\n"
