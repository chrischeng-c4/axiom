"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/revise_change_impl.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-revise-change-impl"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/revise_change_impl.md"
__legacy_td_digest__ = "sha256:563d344bcbd1801ace42a01a7a238adc5a4dbb9c07462c0240420f57a79b8db0"


def render_markdown() -> Annotated[str, "sha256:563d344bcbd1801ace42a01a7a238adc5a4dbb9c07462c0240420f57a79b8db0"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: projects-sdd-src-tools-revise-change-impl-rs\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# Standardized apps/agentic-workflow/src/tools/revise_change_impl.rs\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/revise_change_impl.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    description: |\n      Existing source claimed by `aw standardize run`. The code is wrapped\n      in a tracked HANDWRITE block until deterministic generator coverage can\n      replace it with CODEGEN.\n```\n"
