"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/local_tests_source.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-local-tests-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/local_tests_source.md"
__legacy_projection_digest__ = "sha256:ee4160c2734426c88ec42f03ef205ebbbe8dda5ab69c389c1a1334b9c60b21cd"


def render_markdown() -> Annotated[str, "sha256:ee4160c2734426c88ec42f03ef205ebbbe8dda5ab69c389c1a1334b9c60b21cd"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-issues-backends-local-tests-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# Local Backend Tests Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/issues/backends/local.rs` generated from AST during Score force-regeneration standardization. Local compatibility fixtures read each legacy non-epic frontmatter value as canonical `change` while proving the original file and label remain unmodified.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `LocalBackend` | apps/agentic-workflow/src/issues/backends/local.rs | struct | pub | 41 |  |\n| `at` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 58 | at(issues_dir: PathBuf) -> Self |\n| `from_project_root` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 52 | from_project_root(project_root: &Path) -> Self |\n| `issue_path` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 68 | issue_path(&self, issue: &Issue) -> PathBuf |\n| `issues_dir` | apps/agentic-workflow/src/issues/backends/local.rs | function | pub | 63 | issues_dir(&self) -> &Path |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap local-backend-tests -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/local.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:local-backend-tests>\"\n    description: \"Source template owns local backend regression tests, including lossless legacy non-epic to canonical change compatibility reads.\"\n```\n"
