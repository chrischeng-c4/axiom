"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/workflow_validate.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-workflow-validate"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/workflow_validate.md"
__legacy_td_digest__ = "sha256:5c3e684a021e1b116034e6e5e565d78f70b2ec66419a34dc905ba956e2fa19f4"


def render_markdown() -> Annotated[str, "sha256:5c3e684a021e1b116034e6e5e565d78f70b2ec66419a34dc905ba956e2fa19f4"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-tools-workflow-validate\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# sdd tools workflow validate\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nWorkflow validation gate used by the three-role contract.\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/tech-design/core/tools/workflow_validate/definition.md\n    action: create\n    impl_mode: codegen\n    section: changes\n    description: \"Source-fragment spec for the workflow validation tool definition.\"\n```\n"
