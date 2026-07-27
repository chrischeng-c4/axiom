"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/create_reference_context.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-create-reference-context"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/create_reference_context.md"
__legacy_td_digest__ = "sha256:5dd5b520506f022782fdad3516acbe5865261691f1e149ab3f194dce2945439c"


def render_markdown() -> Annotated[str, "sha256:5dd5b520506f022782fdad3516acbe5865261691f1e149ab3f194dce2945439c"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-tools-create-reference-context\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# sdd tools create reference context\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nCreate-reference-context workflow and artifact tool code ownership.\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/tech-design/core/tools/create_reference_context/definitions.md\n    action: create\n    impl_mode: codegen\n    section: changes\n    description: \"Source-fragment spec for create-reference-context tool definitions.\"\n  - path: apps/agentic-workflow/tech-design/core/tools/create_reference_context/artifact.md\n    action: create\n    impl_mode: codegen\n    section: changes\n    description: \"Source-fragment spec for create-reference-context artifact flow.\"\n```\n"
