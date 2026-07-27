"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/ui/progress_types.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-ui-progress-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/ui/progress_types.md"
__legacy_td_digest__ = "sha256:02cd9447db0709f0fb98f2af98636f310f7c9615bae4623c1b4dde22b50c89d3"


def render_markdown() -> Annotated[str, "sha256:02cd9447db0709f0fb98f2af98636f310f7c9615bae4623c1b4dde22b50c89d3"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-ui-progress-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure.\"\n---\n\n# ProgressBar\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle ProgressBar unit struct (placeholder).\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ProgressBar:\n    type: object\n    description: ProgressBar placeholder unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/ui/progress.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ProgressBar\n    description: Codegen replaces ProgressBar.\n  - path: apps/agentic-workflow/src/ui/progress.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
