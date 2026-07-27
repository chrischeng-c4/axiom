"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab_runtime_source.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-gitlab-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab_runtime_source.md"
__legacy_projection_digest__ = "sha256:2b650eba8cab3b0074bcfbd48525aa6dc568a4b291b5e5b3b2f3ae9490db9866"


def render_markdown() -> Annotated[str, "sha256:2b650eba8cab3b0074bcfbd48525aa6dc568a4b291b5e5b3b2f3ae9490db9866"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-issues-backends-gitlab-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n  - id: project-local-td-and-ec-gates\n    role: primary\n    gap: project-label-producer-td-routing\n    claim: project-label-producer-td-routing\n    coverage: partial\n    rationale: \"GitLab WI updates must honor explicit stale project-label removals while preserving unrelated remote labels.\"\n---\n\n# GitLab Backend Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/issues/backends/gitlab.rs` generated from AST during Score force-regeneration standardization.\n\n`IssueBackend::write` remains conservative for unmanaged remote labels;\n`IssueBackend::update` selects the explicit-removal write variant so requested\nunmanaged removals become `glab issue update --unlabel` arguments. Read\nfixtures normalize every legacy non-epic type label to canonical `change`\noutput without mutating the original remote label set.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `GitLabBackend` | apps/agentic-workflow/src/issues/backends/gitlab.rs | struct | pub | 28 |  |\n| `new` | apps/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 38 | new(repo: Option<String>) -> Self |\n| `with_host` | apps/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 45 | with_host(repo: Option<String>, host: Option<String>) -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap gitlab-backend-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/gitlab.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:gitlab-backend-runtime>\"\n    description: \"Source template owns GitLab backend runtime behavior and tests, including legacy type compatibility reads and authoritative IssuePatch label removals.\"\n```\n"
