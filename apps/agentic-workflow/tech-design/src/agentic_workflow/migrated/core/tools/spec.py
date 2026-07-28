"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/tools/spec.md`.

Migrated by batch `projection-core-tools-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-spec"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/tools/spec.md"
__legacy_projection_digest__ = "sha256:e86110e7c0e89ada4e2938f778190c6a4db1673c7d9f6aa1ed62e9c4bb0b58b9"


def render_markdown() -> Annotated[str, "sha256:e86110e7c0e89ada4e2938f778190c6a4db1673c7d9f6aa1ed62e9c4bb0b58b9"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-tools-spec-rs\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# Standardized apps/agentic-workflow/src/tools/spec.rs\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/spec.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    description: |\n      Existing source claimed by `aw standardize run`. The code is wrapped\n      in a tracked HANDWRITE block until deterministic generator coverage can\n      replace it with CODEGEN.\n```\n"
