"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github_runtime_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-platform-sync-github-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github_runtime_source.md"
__legacy_projection_digest__ = "sha256:35995da3e1d01776af5de6710b34159ac3a630e2ded80f51febe66336b26034d"


def render_markdown() -> Annotated[str, "sha256:35995da3e1d01776af5de6710b34159ac3a630e2ded80f51febe66336b26034d"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-platform-sync-github-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Service interfaces expose project, issue, and platform behavior to the single AW CLI.\"\n---\n\n# Platform Sync GitHub Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/platform_sync/github.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `GitHubProvider` | apps/agentic-workflow/src/services/platform_sync/github.rs | struct | pub | 17 |  |\n| `can_sync` | apps/agentic-workflow/src/services/platform_sync/github.rs | function | pub | 43 | can_sync(&self) -> bool |\n| `new` | apps/agentic-workflow/src/services/platform_sync/github.rs | function | pub | 29 | new(config: PlatformConfig) -> Self |\n| `sync` | apps/agentic-workflow/src/services/platform_sync/github.rs | function | pub | 63 | sync(&self, payload: &SyncPayload) -> Result<SyncResult> |\n| `with_token` | apps/agentic-workflow/src/services/platform_sync/github.rs | function | pub | 37 | with_token(mut self, project_root: &std::path::Path) -> Result<Self> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap platform-sync-github-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/platform_sync/github.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:platform-sync-github-runtime>\"\n    description: \"Source template owns platform sync GitHub provider runtime and tests.\"\n```\n"
