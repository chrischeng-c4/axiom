"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/review.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-review"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/review.md"
__legacy_td_digest__ = "sha256:0f9d482a63072716c3201b06dd824990fcd77ff70098ad4adc5fd21622d09723"


def render_markdown() -> Annotated[str, "sha256:0f9d482a63072716c3201b06dd824990fcd77ff70098ad4adc5fd21622d09723"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: projects-sdd-src-tools-review-rs\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# Standardized apps/agentic-workflow/src/tools/review.rs\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/review.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    description: |\n      Existing source claimed by `aw standardize run`. The code is wrapped\n      in a tracked HANDWRITE block until deterministic generator coverage can\n      replace it with CODEGEN.\n```\n"
