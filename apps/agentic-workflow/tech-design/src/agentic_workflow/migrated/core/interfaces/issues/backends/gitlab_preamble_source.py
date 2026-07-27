"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab_preamble_source.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-gitlab-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab_preamble_source.md"
__legacy_projection_digest__ = "sha256:eb13c0e4b72c5ad2458741477ba3f92f7d71ed1edee9b24d87d514d4c17e3cce"


def render_markdown() -> Annotated[str, "sha256:eb13c0e4b72c5ad2458741477ba3f92f7d71ed1edee9b24d87d514d4c17e3cce"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-issues-backends-gitlab-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# GitLab Backend Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/issues/backends/gitlab.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `GitLabBackend` | apps/agentic-workflow/src/issues/backends/gitlab.rs | struct | pub | 28 |  |\n| `new` | apps/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 38 | new(repo: Option<String>) -> Self |\n| `with_host` | apps/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 45 | with_host(repo: Option<String>, host: Option<String>) -> Self |\n## Source\n<!-- type: source lang: rust -->\n```rust\n//! GitLab backend -- shells out to the `glab` CLI.\n//!\n//! The CRRR write contract round-trips through GitLab's native attributes\n//! (title, state, description, labels) — see `crate::issues::labels` for\n//! the label-prefix scheme that encodes the rest of `Issue`'s CRRR state.\n//! `slug:*` labels are treated as legacy aliases; the GitLab issue iid is\n//! the canonical identity.\n//!\n//! Authentication is delegated to `glab auth login`. Self-hosted hosts go\n//! through the `GITLAB_HOST` environment variable.\n\n// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R6\n\nuse crate::issues::backend::IssueBackend;\nuse crate::issues::labels;\nuse crate::issues::types::{Issue, IssueFilter, IssuePatch, IssueState, IssueType};\nuse anyhow::{Context, Result, anyhow};\nuse async_trait::async_trait;\nuse serde_json::Value;\nuse std::process::Command;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/gitlab.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n      - \"<handwrite-gap:gitlab-backend-preamble>\"\n    description: \"Source template owns the GitLab backend module docs and imports.\"\n```\n"
