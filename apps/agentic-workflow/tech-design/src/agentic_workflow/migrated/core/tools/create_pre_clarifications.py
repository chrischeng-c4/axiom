"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/create_pre_clarifications.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-create-pre-clarifications"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/create_pre_clarifications.md"
__legacy_td_digest__ = "sha256:d2cb1819ad663ca1ce3da408b28508a80502d5f624592859565d2a2f4aee3148"


def render_markdown() -> Annotated[str, "sha256:d2cb1819ad663ca1ce3da408b28508a80502d5f624592859565d2a2f4aee3148"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: projects-sdd-src-tools-create-pre-clarifications-rs\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# Standardized apps/agentic-workflow/src/tools/create_pre_clarifications.rs\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/create_pre_clarifications.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    description: |\n      Existing source claimed by `aw standardize run`. The code is wrapped\n      in a tracked HANDWRITE block until deterministic generator coverage can\n      replace it with CODEGEN.\n```\n"
