"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/clarifications.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-clarifications"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/clarifications.md"
__legacy_td_digest__ = "sha256:4b20b5a562c77989a99c357adae65ea4e14d930484127d9ea0d192d433e8dcdb"


def render_markdown() -> Annotated[str, "sha256:4b20b5a562c77989a99c357adae65ea4e14d930484127d9ea0d192d433e8dcdb"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: projects-sdd-src-tools-clarifications-rs\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# Standardized apps/agentic-workflow/src/tools/clarifications.rs\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/clarifications.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    description: |\n      Existing source claimed by `aw standardize run`. The code is wrapped\n      in a tracked HANDWRITE block until deterministic generator coverage can\n      replace it with CODEGEN.\n```\n"
