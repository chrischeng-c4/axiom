"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/github_runtime_source.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-github-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/github_runtime_source.md"
__legacy_projection_digest__ = "sha256:443ff065a3435d5f114c436897fd870201dc960ca4d4a6e33c3a202129c145cf"


def render_markdown() -> Annotated[str, "sha256:443ff065a3435d5f114c436897fd870201dc960ca4d4a6e33c3a202129c145cf"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-issues-backends-github-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n  - id: project-local-td-and-ec-gates\n    role: primary\n    gap: project-label-producer-td-routing\n    claim: project-label-producer-td-routing\n    coverage: partial\n    rationale: \"GitHub WI updates must honor explicit stale project-label removals while preserving unrelated remote labels.\"\n---\n\n# GitHub Backend Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/issues/backends/github.rs` generated from AST during Score force-regeneration standardization.\n\n`IssueBackend::write` remains conservative for unmanaged remote labels;\n`IssueBackend::update` selects the explicit-removal write variant so requested\nunmanaged removals are included in the same REST label-set patch. A resulting\nempty label set is encoded with `gh api -F labels[]`; omitting the labels field\ncontinues to mean that labels are unchanged. Read fixtures normalize every\nlegacy non-epic type label to canonical `change` output without mutating the\noriginal remote label set.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `GitHubBackend` | apps/agentic-workflow/src/issues/backends/github.rs | struct | pub | 27 |  |\n| `new` | apps/agentic-workflow/src/issues/backends/github.rs | function | pub | 37 | new(repo: Option<String>) -> Self |\n| `with_host` | apps/agentic-workflow/src/issues/backends/github.rs | function | pub | 44 | with_host(repo: Option<String>, host: Option<String>) -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap github-backend-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/github.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:github-backend-runtime>\"\n    description: \"Source template owns GitHub backend runtime behavior and tests, including legacy type compatibility reads, authoritative IssuePatch label removals, and explicit empty-array encoding.\"\n```\n"
