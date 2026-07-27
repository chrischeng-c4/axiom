"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/platform_sync/mod_runtime_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-platform-sync-mod-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/platform_sync/mod_runtime_source.md"
__legacy_projection_digest__ = "sha256:43b50cddff6981f588da695bcd8bf8012f4118f349425eef862856dcdc1560d9"


def render_markdown() -> Annotated[str, "sha256:43b50cddff6981f588da695bcd8bf8012f4118f349425eef862856dcdc1560d9"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-platform-sync-mod-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Service interfaces expose project, issue, and platform behavior to the single AW CLI.\"\n---\n\n# Platform Sync Module Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/platform_sync/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `PlatformSyncService` | apps/agentic-workflow/src/services/platform_sync/mod.rs | struct | pub | 38 |  |\n| `load_config` | apps/agentic-workflow/src/services/platform_sync/mod.rs | function | pub | 52 | load_config(&self) -> Result<PlatformConfig> |\n| `new` | apps/agentic-workflow/src/services/platform_sync/mod.rs | function | pub | 48 | new(project_root: PathBuf) -> Self |\n| `payload` | apps/agentic-workflow/src/services/platform_sync/mod.rs | module | pub | 21 |  |\n| `sync` | apps/agentic-workflow/src/services/platform_sync/mod.rs | function | pub | 58 | sync(&self, change_id: &str) -> Result<SyncResult> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap platform-sync-service-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/platform_sync/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:platform-sync-service-runtime>\"\n    description: \"Source template owns platform sync service runtime behavior.\"\n```\n"
