"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/phase_transition.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-phase-transition"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/phase_transition.md"
__legacy_td_digest__ = "sha256:b901dddb9ac99a35da3df08e6e7638b5bacc856620d75490d948caa9fc5af600"


def render_markdown() -> Annotated[str, "sha256:b901dddb9ac99a35da3df08e6e7638b5bacc856620d75490d948caa9fc5af600"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-tools-phase-transition\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# sdd tools phase transition\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPhase parsing, canonical string conversion, ordering, and transition validation for the legacy SDD change lifecycle.\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/tech-design/core/tools/phase_transition/source.md\n    action: create\n    impl_mode: codegen\n    section: changes\n    description: \"Source-fragment spec that owns the phase transition logic block.\"\n  - path: apps/agentic-workflow/src/tools/phase_transition.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:missing-generator:sdd-phase-transition-logic>\"\n    description: \"Replace the tracked HANDWRITE phase transition logic with a source-generated CODEGEN block.\"\n```\n"
