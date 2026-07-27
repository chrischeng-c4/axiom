"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-path-scope-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md"
__legacy_projection_digest__ = "sha256:8c71ecc3056e9c8359339a9b0be47e09169868f20c6f1ff9ed004f5abb24be8e"


def render_markdown() -> Annotated[str, "sha256:8c71ecc3056e9c8359339a9b0be47e09169868f20c6f1ff9ed004f5abb24be8e"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-path-scope-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Path Scope Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/path_scope.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `AllowedScope` | apps/agentic-workflow/src/services/path_scope.rs | struct | pub | 61 |  |\n| `ScopeProject` | apps/agentic-workflow/src/services/path_scope.rs | struct | pub | 40 |  |\n| `ScopeWorkspace` | apps/agentic-workflow/src/services/path_scope.rs | struct | pub | 52 |  |\n| `ScoreScopeConfig` | apps/agentic-workflow/src/services/path_scope.rs | struct | pub | 32 |  |\n| `contains` | apps/agentic-workflow/src/services/path_scope.rs | function | pub | 108 | contains(&self, rel: &str) -> bool |\n| `describe` | apps/agentic-workflow/src/services/path_scope.rs | function | pub | 119 | describe(&self) -> String |\n| `for_project` | apps/agentic-workflow/src/services/path_scope.rs | function | pub | 74 | for_project(project: &ScopeProject) -> Result<Self> |\n| `load_scope` | apps/agentic-workflow/src/services/path_scope.rs | function | pub | 134 | load_scope(root: &Path) -> Result<Option<ScoreScopeConfig>> |\n| `project_by_name` | apps/agentic-workflow/src/services/path_scope.rs | function | pub | 148 | project_by_name(cfg: &'a ScoreScopeConfig, name: &str) -> Option<&'a ScopeProject> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap missing-generator:logic-flowchart-to-rust-and-config-loader -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/path_scope.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:missing-generator:logic-flowchart-to-rust-and-config-loader>\"\n    description: \"Source template owns path-scope runtime behavior and tests.\"\n```\n"
