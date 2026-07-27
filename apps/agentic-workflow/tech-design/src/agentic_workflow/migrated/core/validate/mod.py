"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/validate/mod.md`.

Migrated by batch `semantic-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-mod"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/validate/mod.md"
__legacy_td_digest__ = "sha256:17fef7ad731f7e9a4b0506d1fd56ee8b46d50512f10aa728f2a5ff1cc521af93"


def render_markdown() -> Annotated[str, "sha256:17fef7ad731f7e9a4b0506d1fd56ee8b46d50512f10aa728f2a5ff1cc521af93"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: projects-sdd-src-validate-mod-rs\nfill_sections: [changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: managed-and-semantic-production-gates\n    claim: managed-and-semantic-production-gates\n    coverage: full\n    rationale: \"Validation TDs implement managed and semantic production gates for standardization readiness.\"\n---\n\n# Standardized apps/agentic-workflow/src/validate/mod.rs\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validate/mod.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    description: |\n      Existing source claimed by `aw standardize run`. The code is wrapped\n      in a tracked HANDWRITE block until deterministic generator coverage can\n      replace it with CODEGEN.\n```\n"
