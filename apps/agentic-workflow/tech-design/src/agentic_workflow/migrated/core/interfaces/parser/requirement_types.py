"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/parser/requirement_types.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-parser-requirement-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/parser/requirement_types.md"
__legacy_td_digest__ = "sha256:656db7fad89fcabb02302da6ecad03582c6079cb458faf96d5b9657640af0141"


def render_markdown() -> Annotated[str, "sha256:656db7fad89fcabb02302da6ecad03582c6079cb458faf96d5b9657640af0141"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-parser-requirement-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure.\"\n---\n\n# RequirementParser\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle RequirementParser unit struct (placeholder).\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  RequirementParser:\n    type: object\n    required: []\n    description: RequirementParser placeholder unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n      unit: true\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/parser/requirement.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - RequirementParser\n    description: Codegen replaces RequirementParser.\n  - path: apps/agentic-workflow/src/parser/requirement.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
