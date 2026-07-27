"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/github_preamble_source.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-github-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/github_preamble_source.md"
__legacy_projection_digest__ = "sha256:6e7d646fb53d3f8b5cc597b68e075174974fd40fa94b71443a5c68339f051254"


def render_markdown() -> Annotated[str, "sha256:6e7d646fb53d3f8b5cc597b68e075174974fd40fa94b71443a5c68339f051254"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-issues-backends-github-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# GitHub Backend Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/issues/backends/github.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `GitHubBackend` | apps/agentic-workflow/src/issues/backends/github.rs | struct | pub | 27 |  |\n| `new` | apps/agentic-workflow/src/issues/backends/github.rs | function | pub | 37 | new(repo: Option<String>) -> Self |\n| `with_host` | apps/agentic-workflow/src/issues/backends/github.rs | function | pub | 44 | with_host(repo: Option<String>, host: Option<String>) -> Self |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! GitHub backend — shells out to the `gh` CLI.\n//!\n//! The CRRR write contract round-trips through GitHub's native attributes\n//! (title, state, body, labels) — see `crate::issues::labels` for the\n//! label-prefix scheme that encodes `phase`, `review_count`, `flagged_sections`,\n//! `fill_retry_count`, `ship_status`, and `ship_commit` as labels on the\n//! GitHub issue. `slug:*` labels are treated as legacy aliases; the GitHub\n//! issue number is the canonical identity.\n//!\n//! Authentication is delegated to the `gh` CLI (user must have run\n//! `gh auth login` beforehand).\n\nuse crate::issues::backend::IssueBackend;\nuse crate::issues::labels;\nuse crate::issues::types::{Issue, IssueFilter, IssuePatch, IssueState, IssueType};\nuse anyhow::{Context, Result, anyhow};\nuse async_trait::async_trait;\nuse serde_json::Value;\nuse std::process::Command;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/github.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n      - \"<handwrite-gap:github-backend-preamble>\"\n    description: \"Source template owns the GitHub backend module docs and imports.\"\n```\n"
