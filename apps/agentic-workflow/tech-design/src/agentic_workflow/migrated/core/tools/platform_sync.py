"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/platform_sync.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-platform-sync"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/platform_sync.md"
__legacy_td_digest__ = "sha256:ff5f40bda52145ef2d84f415ced5ef571e16dbb936999168f2cc10eea1a5abfd"


def render_markdown() -> Annotated[str, "sha256:ff5f40bda52145ef2d84f415ced5ef571e16dbb936999168f2cc10eea1a5abfd"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: projects-sdd-src-tools-platform-sync-rs\nfill_sections: [changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue and platform-sync tool TDs expose AW workflow state through configured issue platforms.\"\n---\n\n# Standardized apps/agentic-workflow/src/tools/platform_sync.rs\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/platform_sync.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    description: |\n      Existing source claimed by `aw standardize run`. The code is wrapped\n      in a tracked HANDWRITE block until deterministic generator coverage can\n      replace it with CODEGEN.\n```\n"
