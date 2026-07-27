"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/local_preamble_source.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-local-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/local_preamble_source.md"
__legacy_projection_digest__ = "sha256:9881ac797572d99a7006b6813ce2f87f54537d36cf3fa1aff2bd841bc368da86"


def render_markdown() -> Annotated[str, "sha256:9881ac797572d99a7006b6813ce2f87f54537d36cf3fa1aff2bd841bc368da86"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-issues-backends-local-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# Local Backend Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/issues/backends/local.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `LocalBackend` | apps/agentic-workflow/src/issues/backends/local.rs | struct | pub | 41 |  |\n| `at` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 58 | at(issues_dir: PathBuf) -> Self |\n| `from_project_root` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 52 | from_project_root(project_root: &Path) -> Self |\n| `issue_path` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 68 | issue_path(&self, issue: &Issue) -> PathBuf |\n| `issues_dir` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 63 | issues_dir(&self) -> &Path |\n## Source\n<!-- type: source lang: rust -->\n```rust\n//! Local filesystem backend — reads and writes `{issues_dir}/{open,closed}/*.md`.\n//!\n//! Issues are physically separated into `open/` and `closed/` subdirectories,\n//! mirroring GitHub/GitLab's two-state model. Each issue is a Markdown file\n//! with YAML frontmatter. Project-root instances store lifecycle working\n//! copies under `/tmp/aw/workspaces/<workspace>/issues`; remote read-through\n//! cache instances live under `/tmp/aw/issues`. Tracker-backed issues use the\n//! tracker-local number (`github_id` / `gitlab_id`) as their canonical file\n//! key; legacy title slugs remain readable as aliases when they already exist\n//! on disk.\n\nuse crate::issues::backend::IssueBackend;\nuse crate::issues::types::{Issue, IssueFilter, IssuePatch, IssueState, IssueType};\nuse crate::parser::frontmatter::parse_document;\nuse anyhow::{Context, Result};\nuse async_trait::async_trait;\nuse serde::{Deserialize, Serialize};\nuse std::collections::HashMap;\nuse std::path::{Path, PathBuf};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/local.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n    description: \"Source template owns the local backend module docs and imports.\"\n```\n"
